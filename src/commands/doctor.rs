use crate::db;
use crate::utils::logger::{info, task};
use anyhow::Result;
use std::path::Path;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};
use std::io;

#[derive(Debug)]
struct ProjectHealth {
    project_id: i32,
    project_name: String,
    project_path: String,
    path_exists: bool,
    cache_dirs_exist: Vec<(String, bool)>,
    issues: Vec<String>,
}

enum PopupState {
    None,
    ConfirmRemove,
}

struct DoctorApp {
    project_healths: Vec<ProjectHealth>,
    selected: usize,
    exit: bool,
    healthy_count: usize,
    total_issues: usize,
    popup_state: PopupState,
    status_message: String,
}

impl DoctorApp {
    fn new(project_healths: Vec<ProjectHealth>, healthy_count: usize, total_issues: usize) -> Self {
        Self {
            project_healths,
            selected: 0,
            exit: false,
            healthy_count,
            total_issues,
            popup_state: PopupState::None,
            status_message: String::new(),
        }
    }

    fn show_remove_confirmation(&mut self) {
        self.popup_state = PopupState::ConfirmRemove;
    }

    fn hide_popup(&mut self) {
        self.popup_state = PopupState::None;
    }

    fn is_popup_visible(&self) -> bool {
        !matches!(self.popup_state, PopupState::None)
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    fn remove_current_project(&mut self) -> Result<()> {
        if self.selected < self.project_healths.len() {
            let project_id = self.project_healths[self.selected].project_id;
            let project_name = self.project_healths[self.selected].project_name.clone();

            let conn = db::connect()?;
            db::remove_project(&conn, project_id)?;
            self.project_healths.remove(self.selected);

            // Update counts
            self.healthy_count = self
                .project_healths
                .iter()
                .filter(|p| p.issues.is_empty())
                .count();
            self.total_issues = self.project_healths.iter().map(|p| p.issues.len()).sum();

            self.set_status(format!("Removed project '{}' from tracking", project_name));

            // Adjust selection if needed
            if self.selected >= self.project_healths.len() && self.selected > 0 {
                self.selected = self.project_healths.len() - 1;
            }

            if self.project_healths.is_empty() {
                self.exit = true;
            }
        }
        Ok(())
    }
}

fn render_remove_popup(f: &mut Frame, project_name: &str, project_id: i32) {
    let area = f.size();
    let popup_width = 60;
    let popup_height = 9;

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_area);

    let popup_text = vec![
        Line::raw(""),
        Line::styled(
            "Remove Project from Tracking?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Project: "),
            Span::styled(project_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("ID: "),
            Span::styled(project_id.to_string(), Style::default().fg(Color::Cyan)),
        ]),
        Line::raw(""),
        Line::styled(
            "This will stop tracking this project.",
            Style::default().fg(Color::Gray),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::styled("y/d", Style::default().fg(Color::Green)),
            Span::raw(" = Yes  |  "),
            Span::styled("n/Esc", Style::default().fg(Color::Red)),
            Span::raw(" = No"),
        ]),
    ];

    let popup = Paragraph::new(popup_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Confirm "),
        )
        .alignment(Alignment::Center);

    f.render_widget(popup, popup_area);
}

fn render_doctor_ui(f: &mut Frame, app: &DoctorApp) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Min(10),
        ])
        .split(size);

    // Header with summary
    let summary_text = vec![
        Line::styled(
            "Project Health Report",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Total Projects: "),
            Span::styled(
                app.project_healths.len().to_string(),
                Style::default().fg(Color::White),
            ),
            Span::raw("    "),
            Span::raw("Healthy: "),
            Span::styled(
                app.healthy_count.to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw("    "),
            Span::raw("Issues: "),
            Span::styled(
                app.total_issues.to_string(),
                Style::default().fg(if app.total_issues > 0 {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ]),
        Line::raw(""),
        if app.total_issues == 0 {
            Line::styled(
                "All projects are healthy!",
                Style::default().fg(Color::Green),
            )
        } else {
            Line::styled(
                "Some projects need attention",
                Style::default().fg(Color::Yellow),
            )
        },
    ];

    let summary_block = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .alignment(Alignment::Center);
    f.render_widget(summary_block, chunks[0]);

    // Main projects table
    let header = ["ID", "Name", "Path Status", "Cache Status", "Issues"];
    let rows: Vec<Row> = app
        .project_healths
        .iter()
        .enumerate()
        .map(|(i, health)| {
            let is_selected = i == app.selected;
            let row_style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else if health.issues.is_empty() {
                Style::default()
            } else {
                Style::default().fg(Color::Yellow)
            };

            // Path status
            let path_status = if health.path_exists {
                Span::styled("OK", Style::default().fg(Color::Green))
            } else {
                Span::styled("MISSING", Style::default().fg(Color::Red))
            };

            // Cache status
            let total_caches = health.cache_dirs_exist.len();
            let healthy_caches = health
                .cache_dirs_exist
                .iter()
                .filter(|(_, exists)| *exists)
                .count();
            let cache_status = if total_caches == 0 {
                Span::styled("None", Style::default().fg(Color::Gray))
            } else if healthy_caches == total_caches {
                Span::styled(
                    format!("{}/{} OK", healthy_caches, total_caches),
                    Style::default().fg(Color::Green),
                )
            } else if healthy_caches == 0 {
                Span::styled(
                    format!("{}/{} MISSING", healthy_caches, total_caches),
                    Style::default().fg(Color::Red),
                )
            } else {
                Span::styled(
                    format!("{}/{} WARN", healthy_caches, total_caches),
                    Style::default().fg(Color::Yellow),
                )
            };

            // Issues count
            let issues_count = health.issues.len();
            let issues_display = if issues_count == 0 {
                Span::styled("None", Style::default().fg(Color::Green))
            } else {
                Span::styled(issues_count.to_string(), Style::default().fg(Color::Red))
            };

            Row::new(vec![
                Cell::from(health.project_id.to_string()).style(row_style),
                Cell::from(health.project_name.clone()).style(row_style),
                Cell::from(path_status).style(row_style),
                Cell::from(cache_status).style(row_style),
                Cell::from(issues_display).style(row_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(header)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title("Projects"))
    .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
    .highlight_symbol(">> ");

    f.render_stateful_widget(
        table,
        chunks[1],
        &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)),
    );

    // Details panel
    let details_text = if app.project_healths.is_empty() {
        vec![Line::raw("No projects found")]
    } else {
        let health = &app.project_healths[app.selected];
        let mut lines = vec![
            Line::styled(
                format!(
                    "Project: {} (ID: {})",
                    health.project_name, health.project_id
                ),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::from(vec![
                Span::raw("Path: "),
                Span::styled(
                    &health.project_path,
                    if health.path_exists {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
                Span::raw(" ["),
                if health.path_exists {
                    Span::styled("OK", Style::default().fg(Color::Green))
                } else {
                    Span::styled("NOT FOUND", Style::default().fg(Color::Red))
                },
                Span::raw("]"),
            ]),
            Line::raw(""),
        ];

        // Cache directories
        lines.push(Line::styled(
            "Cache Directories:",
            Style::default().fg(Color::Yellow),
        ));
        if health.cache_dirs_exist.is_empty() {
            lines.push(Line::raw("  No cache directories configured"));
        } else {
            for (cache_dir, exists) in &health.cache_dirs_exist {
                let status = if *exists { "OK" } else { "MISSING" };
                let style = if *exists {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };
                lines.push(Line::from(vec![
                    Span::raw("  ["),
                    Span::styled(status, style),
                    Span::raw("] "),
                    Span::styled(cache_dir, style),
                ]));
            }
        }

        // Issues
        lines.push(Line::raw(""));
        if health.issues.is_empty() {
            lines.push(Line::styled(
                "No issues found",
                Style::default().fg(Color::Green),
            ));
        } else {
            lines.push(Line::styled(
                "Issues Found:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
            for issue in &health.issues {
                lines.push(Line::from(vec![
                    Span::raw("  * "),
                    Span::styled(issue, Style::default().fg(Color::Red)),
                ]));
            }

            lines.push(Line::raw(""));
            lines.push(Line::styled("Actions:", Style::default().fg(Color::Cyan)));
            lines.push(Line::raw(
                "  * Press 'd' to remove this project from tracking",
            ));
            if !health.path_exists {
                lines.push(Line::raw(
                    "  * Use 'nspira add' to re-add with correct path",
                ));
            }
        }

        // Add status message if present
        if !app.status_message.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::styled("Status:", Style::default().fg(Color::Cyan)));
            lines.push(Line::styled(
                &app.status_message,
                Style::default().fg(Color::White),
            ));
        }
        lines
    };

    let details_block = Paragraph::new(details_text)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .alignment(Alignment::Left);
    f.render_widget(details_block, chunks[2]);

    // Render popup if visible - THIS WAS MISSING!
    if app.is_popup_visible()
        && let PopupState::ConfirmRemove = app.popup_state
        && app.selected < app.project_healths.len()
    {
        let project_name = &app.project_healths[app.selected].project_name;
        let project_id = app.project_healths[app.selected].project_id;
        render_remove_popup(f, project_name, project_id);
    }
}

fn run_doctor_tui(
    project_healths: Vec<ProjectHealth>,
    healthy_count: usize,
    total_issues: usize,
) -> Result<()> {
    if project_healths.is_empty() {
        info("No projects found in database.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = DoctorApp::new(project_healths, healthy_count, total_issues);

    while !app.exit {
        terminal.draw(|f| render_doctor_ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Handle popup input
            if app.is_popup_visible() {
                match app.popup_state {
                    PopupState::ConfirmRemove => match key.code {
                        KeyCode::Char('y')
                        | KeyCode::Char('d')
                        | KeyCode::Char('Y')
                        | KeyCode::Char('D') => {
                            if let Err(e) = app.remove_current_project() {
                                app.set_status(format!("Error removing project: {}", e));
                            }
                            app.hide_popup();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.hide_popup();
                            app.set_status("Cancelled removal");
                        }
                        _ => {}
                    },
                    PopupState::None => {}
                }
            } else {
                // Normal navigation
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.exit = true,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.selected + 1 < app.project_healths.len() {
                            app.selected += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected > 0 {
                            app.selected -= 1;
                        }
                    }
                    KeyCode::Home => app.selected = 0,
                    KeyCode::End => {
                        if !app.project_healths.is_empty() {
                            app.selected = app.project_healths.len() - 1;
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if !app.project_healths.is_empty() {
                            app.show_remove_confirmation();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn run() -> Result<()> {
    task("Running health check...");

    let conn = db::connect()?;
    let projects = db::get_all_projects(&conn)?;
    if projects.is_empty() {
        info("No projects found in database.");
        return Ok(());
    }

    info(&format!("Checking {} tracked projects...", projects.len()));

    let mut healthy_count = 0;
    let mut total_issues = 0;
    let mut project_healths = Vec::new();

    // Check each project
    for project in &projects {
        let mut health = ProjectHealth {
            project_id: project.id,
            project_name: project.name.clone(),
            project_path: project.path.clone(),
            path_exists: false,
            cache_dirs_exist: Vec::new(),
            issues: Vec::new(),
        };

        // Check if project path exists
        health.path_exists = Path::new(&project.path).exists();
        if !health.path_exists {
            health
                .issues
                .push(format!("Project path does not exist: {}", project.path));
        }

        // Check each cache directory
        for cache_dir in &project.cache_dirs {
            let exists = Path::new(cache_dir).exists();
            health.cache_dirs_exist.push((cache_dir.clone(), exists));
            if !exists {
                health
                    .issues
                    .push(format!("Cache directory does not exist: {}", cache_dir));
            }
        }

        // Count healthy projects
        if health.issues.is_empty() {
            healthy_count += 1;
        }
        total_issues += health.issues.len();

        project_healths.push(health);
    }

    // Run TUI
    run_doctor_tui(project_healths, healthy_count, total_issues)?;

    Ok(())
}

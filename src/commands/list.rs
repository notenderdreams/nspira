use crate::db::{get_all_projects, remove_project, update_last_cleaned};
use crate::utils::{clean_dir, get_dir_size, human_readable_size};
use chrono::Utc;
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
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table},
};
use std::io;

enum PopupState {
    None,
    ConfirmDelete,
    CleaningProgress,
}

enum RightPanelView {
    StatsAndHelp,
    CacheDirectories,
}

struct App {
    selected: usize,
    selected_items: Vec<usize>,
    exit: bool,
    status_message: String,
    popup_state: PopupState,
    total_cache_size: u64,
    right_panel_view: RightPanelView,
    cleaning_progress: f64,
    cleaning_current_project: String,
    cleaning_projects_total: usize,
    cleaning_projects_done: usize,
}

impl App {
    fn new(total_cache_size: u64) -> Self {
        Self {
            selected: 0,
            selected_items: Vec::new(),
            exit: false,
            status_message: String::new(),
            popup_state: PopupState::None,
            total_cache_size,
            right_panel_view: RightPanelView::StatsAndHelp,
            cleaning_progress: 0.0,
            cleaning_current_project: String::new(),
            cleaning_projects_total: 0,
            cleaning_projects_done: 0,
        }
    }

    fn toggle_select(&mut self) {
        if let Some(pos) = self.selected_items.iter().position(|&i| i == self.selected) {
            self.selected_items.remove(pos);
        } else {
            self.selected_items.push(self.selected);
        }
    }

    fn toggle_select_all(&mut self, total: usize) {
        if self.selected_items.len() == total {
            self.selected_items.clear();
        } else {
            self.selected_items = (0..total).collect();
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    fn show_delete_confirmation(&mut self) {
        self.popup_state = PopupState::ConfirmDelete;
    }

    fn show_cleaning_progress(&mut self, total: usize) {
        self.cleaning_projects_total = total;
        self.cleaning_projects_done = 0;
        self.cleaning_progress = 0.0;
        self.cleaning_current_project.clear();
        self.popup_state = PopupState::CleaningProgress;
    }

    fn hide_popup(&mut self) {
        self.popup_state = PopupState::None;
    }

    fn is_popup_visible(&self) -> bool {
        !matches!(self.popup_state, PopupState::None)
    }

    fn update_total_cache_size(&mut self, size: u64) {
        self.total_cache_size = size;
    }

    fn toggle_right_panel(&mut self) {
        self.right_panel_view = match self.right_panel_view {
            RightPanelView::StatsAndHelp => RightPanelView::CacheDirectories,
            RightPanelView::CacheDirectories => RightPanelView::StatsAndHelp,
        };
    }

    fn update_cleaning_progress(&mut self, current_project: &str, done: usize) {
        self.cleaning_current_project = current_project.to_string();
        self.cleaning_projects_done = done;
        self.cleaning_progress = if self.cleaning_projects_total > 0 {
            (done as f64 / self.cleaning_projects_total as f64) * 100.0
        } else {
            0.0
        };
    }
}

fn render_delete_popup(f: &mut Frame, count: usize, is_multiple: bool) {
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

    let popup_text = if is_multiple {
        vec![
            Line::raw(""),
            Line::styled(
                "⚠️  Remove Multiple Projects?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::from(vec![
                Span::raw("Selected: "),
                Span::styled(
                    format!("{} projects", count),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::raw(""),
            Line::styled(
                "This will stop tracking these projects.",
                Style::default().fg(Color::Gray),
            ),
            Line::from(vec![
                Span::styled("y/d", Style::default().fg(Color::Green)),
                Span::raw(" = Yes  |  "),
                Span::styled("n/Esc", Style::default().fg(Color::Red)),
                Span::raw(" = No"),
            ]),
        ]
    } else {
        vec![
            Line::raw(""),
            Line::styled(
                "⚠️  Remove Project from Tracking?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::styled(
                "Current selection will be removed.",
                Style::default().fg(Color::Cyan),
            ),
            Line::raw(""),
            Line::styled(
                "This will stop tracking this project.",
                Style::default().fg(Color::Gray),
            ),
            Line::from(vec![
                Span::styled("y/d", Style::default().fg(Color::Green)),
                Span::raw(" = Yes  |  "),
                Span::styled("n/Esc", Style::default().fg(Color::Red)),
                Span::raw(" = No"),
            ]),
        ]
    };

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

fn render_cleaning_progress(f: &mut Frame, app: &App) {
    let area = f.size();
    let popup_width = 60;
    let popup_height = 12;

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_area);

    let progress_text = vec![
        Line::raw(""),
        Line::styled(
            "🧹 Cleaning Cache Directories",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Progress: "),
            Span::styled(
                format!(
                    "{}/{}",
                    app.cleaning_projects_done, app.cleaning_projects_total
                ),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Current: "),
            Span::styled(
                &app.cleaning_current_project,
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::raw(""),
    ];

    let progress_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(" Cleaning Progress ");

    let progress_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
        .percent(app.cleaning_progress as u16)
        .label(format!("{:.1}%", app.cleaning_progress));

    let progress_paragraph = Paragraph::new(progress_text)
        .block(progress_block)
        .alignment(Alignment::Center);

    f.render_widget(progress_paragraph, popup_area);

    // Render gauge in the bottom part of the popup
    let gauge_area = ratatui::layout::Rect {
        x: popup_area.x + 2,
        y: popup_area.y + popup_area.height - 4,
        width: popup_area.width - 4,
        height: 3,
    };

    f.render_widget(progress_gauge, gauge_area);
}

pub fn run() -> anyhow::Result<()> {
    let mut projects = get_all_projects()?;
    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    // Calculate initial total cache size
    let mut total_cache_size = 0u64;
    for project in &projects {
        for cache_dir in &project.cache_dirs {
            total_cache_size += get_dir_size(cache_dir);
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(total_cache_size);

    while !app.exit {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);

            // Table - now showing project path instead of cache size
            let header = ["ID", "Name", "Path", "Size", "Last Cleaned"];
            let rows: Vec<Row> = projects
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let is_selected = app.selected_items.contains(&i);
                    let marker = if is_selected { "✓" } else { " " };

                    // Calculate total size for all cache dirs
                    let total_size: u64 = p.cache_dirs.iter().map(|cd| get_dir_size(cd)).sum();

                    Row::new(vec![
                        Cell::from(format!("{} {}", marker, p.id)),
                        Cell::from(p.name.clone()),
                        Cell::from(p.path.clone()),
                        Cell::from(human_readable_size(total_size)),
                        Cell::from(p.last_cleaned.clone()),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(6),
                    Constraint::Length(20),
                    Constraint::Length(25),
                    Constraint::Length(10),
                    Constraint::Length(18),
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
                chunks[0],
                &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)),
            );

            // Right Panel: Toggle between stats/help and cache directories
            match app.right_panel_view {
                RightPanelView::StatsAndHelp => {
                    let mut help_text = vec![
                        Line::styled(
                            "Statistics",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Line::from(vec![
                            Span::raw("Projects: "),
                            Span::styled(
                                projects.len().to_string(),
                                Style::default().fg(Color::Green),
                            ),
                        ]),
                        Line::from(vec![
                            Span::raw("Total Size: "),
                            Span::styled(
                                human_readable_size(app.total_cache_size),
                                Style::default().fg(Color::Yellow),
                            ),
                        ]),
                        Line::raw(""),
                        Line::styled(
                            "Controls",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Line::from(vec![
                            Span::styled("↑/↓ or j/k", Style::default().fg(Color::Yellow)),
                            Span::raw(" - Move"),
                        ]),
                        Line::from(vec![
                            Span::styled("Space", Style::default().fg(Color::Yellow)),
                            Span::raw(" - Toggle select"),
                        ]),
                        Line::from(vec![
                            Span::styled("a", Style::default().fg(Color::Yellow)),
                            Span::raw(" - Select/Unselect all"),
                        ]),
                        Line::from(vec![
                            Span::styled("Enter", Style::default().fg(Color::Yellow)),
                            Span::raw(" - Clean selected"),
                        ]),
                        Line::from(vec![
                            Span::styled("d", Style::default().fg(Color::Yellow)),
                            Span::raw(" - Remove tracking"),
                        ]),
                        Line::from(vec![
                            Span::styled("Tab", Style::default().fg(Color::Yellow)),
                            Span::raw(" - View cache dirs"),
                        ]),
                        Line::from(vec![
                            Span::styled("q", Style::default().fg(Color::Yellow)),
                            Span::raw(" - Quit"),
                        ]),
                    ];

                    // Add status message if present
                    if !app.status_message.is_empty() {
                        help_text.push(Line::raw(""));
                        help_text.push(Line::styled(
                            "Status",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ));
                        help_text.push(Line::styled(
                            &app.status_message,
                            Style::default().fg(Color::White),
                        ));
                    }

                    let help_block = Paragraph::new(help_text)
                        .block(Block::default().borders(Borders::ALL).title("Info"));
                    f.render_widget(help_block, chunks[1]);
                }
                RightPanelView::CacheDirectories => {
                    let cache_text = if app.selected_items.is_empty() {
                        // Show cache directories for the currently selected project
                        if app.selected < projects.len() {
                            let project = &projects[app.selected];
                            let mut lines = vec![
                                Line::styled(
                                    format!("Cache Directories for '{}'", project.name),
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Line::raw(""),
                            ];

                            for (idx, cache_dir) in project.cache_dirs.iter().enumerate() {
                                let size = get_dir_size(cache_dir);
                                lines.push(Line::from(vec![
                                    Span::styled(
                                        format!("{}. ", idx + 1),
                                        Style::default().fg(Color::Yellow),
                                    ),
                                    Span::raw(cache_dir),
                                ]));
                                lines.push(Line::from(vec![
                                    Span::raw("   Size: "),
                                    Span::styled(
                                        human_readable_size(size),
                                        Style::default().fg(Color::Green),
                                    ),
                                ]));
                                lines.push(Line::raw(""));
                            }

                            lines.push(Line::raw(""));
                            lines.push(Line::from(vec![
                                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                                Span::raw(" - Back to stats"),
                            ]));

                            lines
                        } else {
                            vec![Line::raw("No project selected")]
                        }
                    } else {
                        // Show count of selected projects
                        vec![
                            Line::styled(
                                "Multiple Projects Selected",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Line::raw(""),
                            Line::from(vec![
                                Span::styled(
                                    format!("{}", app.selected_items.len()),
                                    Style::default().fg(Color::Yellow),
                                ),
                                Span::raw(" projects selected"),
                            ]),
                            Line::raw(""),
                            Line::styled(
                                "Deselect to view cache directories",
                                Style::default().fg(Color::Gray),
                            ),
                            Line::raw(""),
                            Line::from(vec![
                                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                                Span::raw(" - Back to stats"),
                            ]),
                        ]
                    };

                    let cache_block = Paragraph::new(cache_text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Cache Directories"),
                    );
                    f.render_widget(cache_block, chunks[1]);
                }
            }

            // Render popup if visible
            match app.popup_state {
                PopupState::ConfirmDelete => {
                    let count = if app.selected_items.is_empty() {
                        1
                    } else {
                        app.selected_items.len()
                    };
                    let is_multiple = !app.selected_items.is_empty();
                    render_delete_popup(f, count, is_multiple);
                }
                PopupState::CleaningProgress => {
                    render_cleaning_progress(f, &app);
                }
                PopupState::None => {}
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle popup input
                if app.is_popup_visible() {
                    match app.popup_state {
                        PopupState::ConfirmDelete => match key.code {
                            KeyCode::Char('y')
                            | KeyCode::Char('d')
                            | KeyCode::Char('Y')
                            | KeyCode::Char('D') => {
                                let projects_to_delete: Vec<usize> =
                                    if app.selected_items.is_empty() {
                                        vec![app.selected]
                                    } else {
                                        app.selected_items.clone()
                                    };

                                let mut total_size_freed = 0u64;
                                let mut removed_count = 0;
                                let mut errors = Vec::new();

                                let mut sorted_indexes = projects_to_delete.clone();
                                sorted_indexes.sort_by(|a, b| b.cmp(a));

                                for &idx in &sorted_indexes {
                                    if idx < projects.len() {
                                        let project_id = projects[idx].id;
                                        let project_cache_size: u64 = projects[idx]
                                            .cache_dirs
                                            .iter()
                                            .map(|cd| get_dir_size(cd))
                                            .sum();

                                        if let Err(e) = remove_project(project_id) {
                                            errors.push(format!(
                                                "Error removing {}: {}",
                                                projects[idx].name, e
                                            ));
                                        } else {
                                            total_size_freed += project_cache_size;
                                            projects.remove(idx);
                                            removed_count += 1;
                                        }
                                    }
                                }

                                app.update_total_cache_size(
                                    app.total_cache_size.saturating_sub(total_size_freed),
                                );

                                if errors.is_empty() {
                                    if removed_count == 1 {
                                        app.set_status("✓ Removed 1 project from tracking");
                                    } else {
                                        app.set_status(format!(
                                            "✓ Removed {} projects from tracking",
                                            removed_count
                                        ));
                                    }
                                } else {
                                    app.set_status(format!(
                                        "⚠ Removed {} projects, {} errors",
                                        removed_count,
                                        errors.len()
                                    ));
                                }

                                if app.selected >= projects.len() && app.selected > 0 {
                                    app.selected = projects.len() - 1;
                                }

                                app.selected_items.clear();

                                if projects.is_empty() {
                                    app.exit = true;
                                }

                                app.hide_popup();
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.hide_popup();
                                app.set_status("✗ Cancelled removal");
                            }
                            _ => {}
                        },
                        PopupState::CleaningProgress => {
                            // Allow cancelling cleaning with Escape
                            if key.code == KeyCode::Esc {
                                app.hide_popup();
                                app.set_status("✗ Cleaning cancelled");
                            }
                        }
                        PopupState::None => {}
                    }
                } else {
                    // Normal navigation
                    match key.code {
                        KeyCode::Char('q') => app.exit = true,
                        KeyCode::Tab => app.toggle_right_panel(),
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.selected + 1 < projects.len() {
                                app.selected += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.selected > 0 {
                                app.selected -= 1;
                            }
                        }
                        KeyCode::Char(' ') => app.toggle_select(),
                        KeyCode::Char('a') => app.toggle_select_all(projects.len()),
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            if projects.is_empty() {
                                app.set_status("⚠ No projects to remove");
                            } else {
                                app.show_delete_confirmation();
                            }
                        }
                        KeyCode::Enter => {
                            let selected_indexes: Vec<usize> = app.selected_items.clone();
                            if selected_indexes.is_empty() {
                                app.set_status("⚠ No projects selected to clean");
                            } else {
                                // Show progress popup
                                app.show_cleaning_progress(selected_indexes.len());

                                // Force immediate UI update to show the progress popup
                                terminal.draw(|f| {
                                    let size = f.size();
                                    let chunks = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints([
                                            Constraint::Percentage(70),
                                            Constraint::Percentage(30),
                                        ])
                                        .split(size);
                                    // ... (same table rendering as above)
                                    render_cleaning_progress(f, &app);
                                })?;

                                let mut total_freed = 0;
                                let mut current_index = 0;

                                for &i in &selected_indexes {
                                    current_index += 1;
                                    let proj = &projects[i];

                                    // Update progress
                                    app.update_cleaning_progress(&proj.name, current_index);
                                    terminal.draw(|f| render_cleaning_progress(f, &app))?;

                                    // Clean cache directories
                                    for cache_dir in &proj.cache_dirs {
                                        let size = get_dir_size(cache_dir);
                                        clean_dir(cache_dir)?;
                                        total_freed += size;
                                    }

                                    update_last_cleaned(proj.id)?;
                                    projects[i].last_cleaned = Utc::now().to_rfc3339();
                                }

                                app.update_total_cache_size(
                                    app.total_cache_size.saturating_sub(total_freed),
                                );
                                app.hide_popup();

                                app.set_status(format!(
                                    "✓ Cleaned {} project(s) — freed {}",
                                    selected_indexes.len(),
                                    human_readable_size(total_freed)
                                ));

                                app.selected_items.clear();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

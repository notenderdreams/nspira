use std::io;
use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Clear},
    Terminal, Frame,
};
use crate::db::{get_all_projects, update_last_cleaned, remove_project};
use crate::utils::{clean_dir, get_dir_size, human_readable_size};

enum PopupState {
    None,
    ConfirmDelete,
}

struct App {
    selected: usize,
    selected_items: Vec<usize>,
    exit: bool,
    status_message: String,
    popup_state: PopupState,
    total_cache_size: u64,
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

    fn hide_popup(&mut self) {
        self.popup_state = PopupState::None;
    }

    fn is_popup_visible(&self) -> bool {
        !matches!(self.popup_state, PopupState::None)
    }

    fn update_total_cache_size(&mut self, size: u64) {
        self.total_cache_size = size;
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
                Span::styled(format!("{} projects", count), Style::default().fg(Color::Cyan)),
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
                .title(" Confirm ")
        )
        .alignment(Alignment::Center);

    f.render_widget(popup, popup_area);
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
        total_cache_size += get_dir_size(&project.cache_dir);
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

            // Table
            let header = ["ID", "Name", "Path", "Size", "Last Cleaned"];
            let rows: Vec<Row> = projects
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let is_selected = app.selected_items.contains(&i);
                    let marker = if is_selected { "✓" } else { " " };
                    Row::new(vec![
                        Cell::from(format!("{} {}", marker, p.id)),
                        Cell::from(p.name.clone()),
                        Cell::from(p.path.clone()),
                        Cell::from(human_readable_size(get_dir_size(&p.cache_dir))),
                        Cell::from(p.last_cleaned.clone()),
                    ])
                })
                .collect();

            let table = Table::new(rows, [
                Constraint::Length(6),
                Constraint::Length(20),
                Constraint::Length(25),
                Constraint::Length(10),
                Constraint::Length(18),
            ])
                .header(
                    Row::new(header)
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .bottom_margin(1),
                )
                .block(Block::default().borders(Borders::ALL).title("Projects"))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol(">> ");

            f.render_stateful_widget(table, chunks[0], &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)));

            // Right Panel: stats + help
            let help_text = vec![
                Line::styled("Statistics", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Line::from(vec![
                    Span::raw("Projects: "),
                    Span::styled(projects.len().to_string(), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::raw("Total Size: "),
                    Span::styled(
                        human_readable_size(app.total_cache_size),
                        Style::default().fg(Color::Yellow)
                    ),
                ]),
                Line::raw(""),
                Line::styled("Controls", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
                    Span::styled("q", Style::default().fg(Color::Yellow)),
                    Span::raw(" - Quit"),
                ]),
            ];

            let mut final_help_text = help_text;

            // Add status message if present
            if !app.status_message.is_empty() {
                final_help_text.push(Line::raw(""));
                final_help_text.push(Line::styled("Status", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                final_help_text.push(Line::styled(&app.status_message, Style::default().fg(Color::White)));
            }

            let help_block = Paragraph::new(final_help_text)
                .block(Block::default().borders(Borders::ALL).title("Info"));
            f.render_widget(help_block, chunks[1]);

            // Render popup if visible
            if let PopupState::ConfirmDelete = app.popup_state {
                let count = if app.selected_items.is_empty() { 1 } else { app.selected_items.len() };
                let is_multiple = !app.selected_items.is_empty();
                render_delete_popup(f, count, is_multiple);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle popup input
                if app.is_popup_visible() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('d') | KeyCode::Char('Y') | KeyCode::Char('D') => {
                            if let PopupState::ConfirmDelete = app.popup_state {
                                let projects_to_delete: Vec<usize> = if app.selected_items.is_empty() {
                                    // Delete only the currently highlighted project
                                    vec![app.selected]
                                } else {
                                    // Delete all selected projects
                                    app.selected_items.clone()
                                };

                                let mut total_size_freed = 0u64;
                                let mut removed_count = 0;
                                let mut errors = Vec::new();

                                // Sort in reverse order to delete from the end first
                                let mut sorted_indexes = projects_to_delete.clone();
                                sorted_indexes.sort_by(|a, b| b.cmp(a));

                                for &idx in &sorted_indexes {
                                    if idx < projects.len() {
                                        let project_id = projects[idx].id;
                                        let project_cache_size = get_dir_size(&projects[idx].cache_dir);

                                        if let Err(e) = remove_project(project_id) {
                                            errors.push(format!("Error removing {}: {}", projects[idx].name, e));
                                        } else {
                                            total_size_freed += project_cache_size;
                                            projects.remove(idx);
                                            removed_count += 1;
                                        }
                                    }
                                }

                                // Update total cache size
                                app.update_total_cache_size(app.total_cache_size.saturating_sub(total_size_freed));

                                // Set status message
                                if errors.is_empty() {
                                    if removed_count == 1 {
                                        app.set_status("✓ Removed 1 project from tracking");
                                    } else {
                                        app.set_status(format!("✓ Removed {} projects from tracking", removed_count));
                                    }
                                } else {
                                    app.set_status(format!("⚠ Removed {} projects, {} errors", removed_count, errors.len()));
                                }

                                // Adjust selection if needed
                                if app.selected >= projects.len() && app.selected > 0 {
                                    app.selected = projects.len() - 1;
                                }

                                // Clear selected items
                                app.selected_items.clear();

                                // Exit if no projects left
                                if projects.is_empty() {
                                    app.exit = true;
                                }
                            }
                            app.hide_popup();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.hide_popup();
                            app.set_status("✗ Cancelled removal");
                        }
                        _ => {}
                    }
                } else {
                    // Normal navigation
                    match key.code {
                        KeyCode::Char('q') => app.exit = true,
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
                                let mut total_freed = 0;
                                for &i in &selected_indexes {
                                    let proj = &projects[i];
                                    let size = get_dir_size(&proj.cache_dir);
                                    clean_dir(&proj.cache_dir)?;
                                    update_last_cleaned(proj.id)?;
                                    total_freed += size;
                                    projects[i].last_cleaned = Utc::now().to_rfc3339();
                                }

                                // Update total cache size
                                app.update_total_cache_size(app.total_cache_size.saturating_sub(total_freed));

                                app.set_status(format!(
                                    "✓ Cleaned {} project(s) — freed {}",
                                    selected_indexes.len(),
                                    human_readable_size(total_freed)
                                ));

                                // Clear selections after cleaning
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
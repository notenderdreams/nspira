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
    ConfirmDelete(usize),
}

struct App {
    selected: usize,
    selected_items: Vec<usize>,
    exit: bool,
    status_message: String,
    popup_state: PopupState,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            selected_items: Vec::new(),
            exit: false,
            status_message: String::new(),
            popup_state: PopupState::None,
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
        self.popup_state = PopupState::ConfirmDelete(self.selected);
    }

    fn hide_popup(&mut self) {
        self.popup_state = PopupState::None;
    }

    fn is_popup_visible(&self) -> bool {
        !matches!(self.popup_state, PopupState::None)
    }
}

fn render_delete_popup(f: &mut Frame, project_name: &str) {
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
            "⚠️  Remove Project from Tracking?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Project: "),
            Span::styled(project_name, Style::default().fg(Color::Cyan)),
        ]),
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
    ];

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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new();

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

            // Right Panel: status / help
            let help_text = vec![
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
                Line::raw(""),
                Line::styled("Status:", Style::default().add_modifier(Modifier::BOLD)),
                Line::raw(app.status_message.clone()),
            ];
            let help_block = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Info"));
            f.render_widget(help_block, chunks[1]);

            // Render popup if visible
            if let PopupState::ConfirmDelete(idx) = app.popup_state {
                if idx < projects.len() {
                    render_delete_popup(f, &projects[idx].name);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle popup input
                if app.is_popup_visible() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('d') | KeyCode::Char('Y') | KeyCode::Char('D') => {
                            if let PopupState::ConfirmDelete(idx) = app.popup_state {
                                if idx < projects.len() {
                                    let project_id = projects[idx].id;
                                    let project_name = projects[idx].name.clone();

                                    if let Err(e) = remove_project(project_id) {
                                        app.set_status(format!("Error removing project: {}", e));
                                    } else {
                                        projects.remove(idx);
                                        app.set_status(format!("Removed project '{}'", project_name));

                                        // Adjust selection if needed
                                        if app.selected >= projects.len() && app.selected > 0 {
                                            app.selected -= 1;
                                        }

                                        // Clear selected items that are now invalid
                                        app.selected_items.retain(|&i| i < projects.len());

                                        // Exit if no projects left
                                        if projects.is_empty() {
                                            app.exit = true;
                                        }
                                    }
                                }
                            }
                            app.hide_popup();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.hide_popup();
                            app.set_status("Cancelled removal");
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
                            app.show_delete_confirmation();
                        }
                        KeyCode::Enter => {
                            let selected_indexes: Vec<usize> = app.selected_items.clone();
                            if selected_indexes.is_empty() {
                                app.set_status("No projects selected to clean");
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
                                app.set_status(format!(
                                    "Cleaned {} project(s) — freed {}",
                                    selected_indexes.len(),
                                    human_readable_size(total_freed)
                                ));
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
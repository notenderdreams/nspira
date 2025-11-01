use std::io;
use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Terminal,
};
use crate::db::{get_all_projects, update_last_cleaned};
use crate::utils::{clean_dir, get_dir_size, human_readable_size};

struct App {
    selected: usize,
    selected_items: Vec<usize>,
    exit: bool,
    status_message: String,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            selected_items: Vec::new(),
            exit: false,
            status_message: String::new(),
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
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

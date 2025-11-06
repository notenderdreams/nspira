use crate::core::{DetectedProject, ScanConfig, Scanner};
use crate::db;
use crate::utils::logger::{info, success, task};
use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::io;

struct ScanApp {
    detected_projects: Vec<DetectedProject>,
    selected: usize,
    selected_items: Vec<usize>,
    exit: bool,
    status_message: String,
}

impl ScanApp {
    fn new(detected_projects: Vec<DetectedProject>) -> Self {
        Self {
            detected_projects,
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

    fn toggle_select_all(&mut self) {
        if self.selected_items.len() == self.detected_projects.len() {
            self.selected_items.clear();
        } else {
            self.selected_items = (0..self.detected_projects.len()).collect();
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    fn add_selected_projects(&self) -> Result<()> {
        let conn = db::connect()?;
        let mut added_count = 0;

        for &idx in &self.selected_items {
            if idx < self.detected_projects.len() {
                let project = &self.detected_projects[idx];

                // Convert PathBuf vector to Vec<String>
                let cache_paths: Vec<String> = project
                    .cache_dirs
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                // Add project to database
                let project_id =
                    db::add_project(&conn, &project.name, project.path.to_str().unwrap())?;

                // Add cache directories
                for cache_path in cache_paths {
                    db::add_cache_directory(&conn, project_id, &cache_path)?;
                }

                added_count += 1;
            }
        }

        if added_count > 0 {
            success(&format!("Added {} project(s) to tracking", added_count));
        }

        Ok(())
    }
}

fn render_scan_ui(f: &mut Frame, app: &ScanApp) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(6)])
        .split(size);

    // Main table
    let header = ["Select", "Name", "Type", "Path", "Cache Dirs"];
    let rows: Vec<Row> = app
        .detected_projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = app.selected_items.contains(&i);
            let marker = if is_selected { "✓" } else { " " };
            let selection_style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(marker).style(selection_style),
                Cell::from(p.name.clone()),
                Cell::from(p.project_type.clone()),
                Cell::from(p.path.to_string_lossy().to_string()),
                Cell::from(p.cache_dirs.len().to_string()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Percentage(50),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(header)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Detected Projects"),
    )
    .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
    .highlight_symbol(">> ");

    f.render_stateful_widget(
        table,
        chunks[0],
        &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)),
    );

    // Help panel
    let help_text = vec![
        Line::styled(
            "Controls",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(vec![
            Span::styled("↑/↓ or j/k", Style::default().fg(Color::Yellow)),
            Span::raw(" - Navigate"),
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
            Span::raw(" - Add selected projects"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" - Quit without adding"),
        ]),
    ];

    let mut status_lines = help_text;

    // Add selection info
    if !app.selected_items.is_empty() {
        status_lines.insert(
            0,
            Line::from(vec![
                Span::raw("Selected: "),
                Span::styled(
                    format!("{} project(s)", app.selected_items.len()),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        );
        status_lines.insert(1, Line::raw(""));
    }

    // Add status message if present
    if !app.status_message.is_empty() {
        status_lines.push(Line::raw(""));
        status_lines.push(Line::styled(
            &app.status_message,
            Style::default().fg(Color::White),
        ));
    }

    let help_block =
        Paragraph::new(status_lines).block(Block::default().borders(Borders::ALL).title("Info"));
    f.render_widget(help_block, chunks[1]);
}

fn run_scan_tui(detected_projects: Vec<DetectedProject>) -> Result<()> {
    if detected_projects.is_empty() {
        info("No new projects detected");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = ScanApp::new(detected_projects);

    while !app.exit {
        terminal.draw(|f| render_scan_ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') => app.exit = true,
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.selected + 1 < app.detected_projects.len() {
                        app.selected += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.selected > 0 {
                        app.selected -= 1;
                    }
                }
                KeyCode::Char(' ') => app.toggle_select(),
                KeyCode::Char('a') => app.toggle_select_all(),
                KeyCode::Enter => {
                    if app.selected_items.is_empty() {
                        app.set_status("⚠ No projects selected");
                    } else {
                        match app.add_selected_projects() {
                            Ok(_) => app.exit = true,
                            Err(e) => app.set_status(format!("Error adding projects: {}", e)),
                        }
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// Moved to core::scanner module

pub fn run() -> Result<()> {
    task("Loading scan patterns...");
    let config = ScanConfig::load()?;
    info(&format!(
        "Loaded {} project patterns",
        config.patterns.len()
    ));

    task("Starting filesystem scan...");
    let start_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    info(&format!("Scanning from: {}", start_path.display()));
    println!();

    let tracked_paths = get_tracked_paths()?;
    let scanner = Scanner::new(config, tracked_paths);
    let detected = scanner.scan(&start_path, 4)?;

    // Run TUI for project selection
    run_scan_tui(detected)?;

    Ok(())
}

fn get_tracked_paths() -> Result<HashSet<PathBuf>> {
    let projects = crate::core::ProjectManager::get_all()?;
    Ok(projects
        .into_iter()
        .map(|p| PathBuf::from(p.path))
        .collect())
}

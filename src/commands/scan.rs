use crate::db::{add_project, get_all_projects};
use crate::utils::logger::{ask_input, info, success, task};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};
use std::io;

#[derive(Debug, Deserialize, Serialize)]
struct ProjectPattern {
    name: String,
    identifier: String,
    cache_dirs: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScanConfig {
    patterns: Vec<ProjectPattern>,
    skip_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
struct DetectedProject {
    name: String,
    path: PathBuf,
    project_type: String,
    cache_dirs: Vec<PathBuf>,
}

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
                add_project(&project.name, project.path.to_str().unwrap(), cache_paths)?;

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

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

impl ScanConfig {
    fn load() -> Result<Self> {
        // Try to load from user config first
        let config_path = dirs::config_dir().map(|d| d.join("nspira").join("patterns.json"));

        if let Some(path) = &config_path {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                return Ok(serde_json::from_str(&content)?);
            }
        }

        // Fallback to embedded default from lib.rs
        Ok(serde_json::from_str(crate::DEFAULT_PATTERNS)?)
    }
}

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
    let detected = scan_filesystem(&start_path, &config, &tracked_paths)?;

    // Run TUI for project selection
    run_scan_tui(detected)?;

    Ok(())
}

fn get_tracked_paths() -> Result<HashSet<PathBuf>> {
    let projects = get_all_projects()?;
    Ok(projects
        .into_iter()
        .map(|p| PathBuf::from(p.path))
        .collect())
}

fn scan_filesystem(
    start_path: &Path,
    config: &ScanConfig,
    tracked_paths: &HashSet<PathBuf>,
) -> Result<Vec<DetectedProject>> {
    let mut detected = Vec::new();
    let skip_set: HashSet<String> = config.skip_dirs.iter().cloned().collect();
    let mut scanned_count = 0;

    let walker = WalkDir::new(start_path)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !skip_set.contains(name.as_ref())
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            continue;
        }

        scanned_count += 1;
        if scanned_count % 100 == 0 {
            print!("\rScanning... {} directories checked", scanned_count);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }

        let path = entry.path();

        // Skip if already tracked
        if tracked_paths.contains(path) {
            continue;
        }

        // Check if this directory matches any pattern
        if let Some(project) = detect_project(path, config) {
            detected.push(project);
        }
    }

    if scanned_count > 0 {
        println!("\rScanned {} directories", scanned_count);
    }

    Ok(detected)
}

fn detect_project(path: &Path, config: &ScanConfig) -> Option<DetectedProject> {
    for pattern in &config.patterns {
        let identifier_path = path.join(&pattern.identifier);

        if identifier_path.exists() {
            // Find which cache directories actually exist
            let mut found_caches = Vec::new();

            for cache_dir in &pattern.cache_dirs {
                let cache_path = path.join(cache_dir);
                if cache_path.exists() && cache_path.is_dir() {
                    found_caches.push(cache_path);
                }
            }

            // Only return if we found at least one cache directory
            if !found_caches.is_empty() {
                let project_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                return Some(DetectedProject {
                    name: project_name,
                    path: path.to_path_buf(),
                    project_type: pattern.name.clone(),
                    cache_dirs: found_caches,
                });
            }
        }
    }

    None
}

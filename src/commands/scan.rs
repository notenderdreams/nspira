use crate::core::{DetectedProject, ScanConfig, Scanner};
use crate::db;
use crate::ui::{
    components::{TableConfig, create_scan_help},
    init_terminal, poll_event, restore_terminal,
    views::{ListAction, ListView},
};
use crate::utils::logger::{info, success, task};
use anyhow::Result;
use ratatui::layout::Constraint;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    task("Loading configuration...");
    let app_config = crate::config::Config::load()?;

    // Configure thread pool for parallel operations
    crate::utils::parallel::configure_thread_pool(app_config.scan.parallelism)?;

    task("Loading scan patterns...");
    let mut scan_config = ScanConfig::load()?;

    // Merge skip directories from app config
    for skip_dir in &app_config.scan.skip_directories {
        if !scan_config.skip_dirs.contains(skip_dir) {
            scan_config.skip_dirs.push(skip_dir.clone());
        }
    }

    info(&format!(
        "Loaded {} project patterns",
        scan_config.patterns.len()
    ));

    task("Starting filesystem scan...");
    let start_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    info(&format!("Scanning from: {}", start_path.display()));
    info(&format!("Max depth: {}", app_config.scan.max_depth));
    println!();

    let tracked_paths = get_tracked_paths()?;
    let scanner = Scanner::new(scan_config, tracked_paths);
    let detected = scanner.scan(&start_path, app_config.scan.max_depth)?;

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

fn run_scan_tui(detected_projects: Vec<DetectedProject>) -> Result<()> {
    if detected_projects.is_empty() {
        info("No new projects detected");
        return Ok(());
    }

    let table_config = TableConfig::new("Detected Projects")
        .headers(vec!["Select", "Name", "Type", "Path", "Cache Dirs"])
        .constraints(vec![
            Constraint::Length(8),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Percentage(50),
            Constraint::Length(12),
        ]);

    let mut list_view = ListView::new(detected_projects.clone(), table_config);
    let mut terminal = init_terminal()?;

    loop {
        terminal.draw(|f| {
            // Custom render for scan with specific help
            let size = f.size();
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(70),
                    ratatui::layout::Constraint::Percentage(30),
                ])
                .split(size);

            // Render table
            let rows = list_view
                .items
                .iter()
                .enumerate()
                .map(|(i, project)| {
                    let is_selected = list_view.state.is_selected(i);
                    let marker = if is_selected { "✓" } else { " " };

                    crate::ui::components::create_row_with_selection(
                        vec![
                            marker.to_string(),
                            project.name.clone(),
                            project.project_type.clone(),
                            project.path.to_string_lossy().to_string(),
                            project.cache_dirs.len().to_string(),
                        ],
                        is_selected,
                        false,
                    )
                })
                .collect();

            let table = crate::ui::components::TableWidget::new(list_view.table_config.clone())
                .rows(rows)
                .build();

            let mut table_state = ratatui::widgets::TableState::default()
                .with_selected(Some(list_view.state.selected));
            f.render_stateful_widget(table, chunks[0], &mut table_state);

            // Render scan-specific help
            let help = create_scan_help(
                list_view.state.selected_count(),
                list_view.items.len(),
                &list_view.state.status_message,
            );
            help.render(f, chunks[1]);

            // Render confirmation popup if needed (scan doesn't use delete confirmation)
            if list_view.show_confirmation {
                let popup = crate::ui::components::confirmation_popup(
                    "Confirm Addition",
                    "Add selected projects to tracking?",
                    if list_view.state.selected_count() > 0 {
                        Some(list_view.state.selected_count())
                    } else {
                        Some(1)
                    },
                );
                popup.render(f);
            }
        })?;

        if let Some(key) = poll_event()? {
            match list_view.handle_key(key) {
                ListAction::Quit => break,
                ListAction::ExecuteAction => {
                    if list_view.state.selected_items.is_empty() {
                        list_view.state.set_status("⚠ No projects selected");
                    } else {
                        match add_selected_projects(
                            &list_view.items,
                            &list_view.state.selected_items,
                        ) {
                            Ok(_) => break,
                            Err(e) => list_view
                                .state
                                .set_status(format!("Error adding projects: {}", e)),
                        }
                    }
                }
                ListAction::ConfirmAction => {
                    // Not used in scan
                }
                ListAction::None => {}
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn add_selected_projects(
    detected_projects: &[DetectedProject],
    selected_items: &[usize],
) -> Result<()> {
    let conn = db::connect()?;
    let mut added_count = 0;

    for &idx in selected_items {
        if idx < detected_projects.len() {
            let project = &detected_projects[idx];

            // Convert PathBuf vector to Vec<String>
            let cache_paths: Vec<String> = project
                .cache_dirs
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            // Add project to database
            let project_id = db::add_project(&conn, &project.name, project.path.to_str().unwrap())?;

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

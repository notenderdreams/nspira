use crate::db;
use crate::ui::{
    components::{TableConfig, create_doctor_help},
    init_terminal, poll_event, restore_terminal,
    views::{ListAction, ListView},
};
use crate::utils::logger::{info, task};
use anyhow::Result;
use ratatui::layout::Constraint;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProjectHealth {
    pub project_id: i32,
    pub project_name: String,
    #[allow(dead_code)]
    pub project_path: String,
    pub path_exists: bool,
    pub cache_dirs_exist: Vec<(String, bool)>,
    pub issues: Vec<String>,
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

    let mut project_healths = Vec::new();
    let mut healthy_count = 0;
    let mut total_issues = 0;

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

fn run_doctor_tui(
    mut project_healths: Vec<ProjectHealth>,
    mut healthy_count: usize,
    mut total_issues: usize,
) -> Result<()> {
    let conn = db::connect()?;
    let table_config = TableConfig::new("Project Health")
        .headers(vec!["ID", "Name", "Path Status", "Cache Status", "Issues"])
        .constraints(vec![
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(8),
        ]);

    let mut list_view = ListView::new(project_healths.clone(), table_config);
    let mut terminal = init_terminal()?;

    loop {
        terminal.draw(|f| {
            // Custom render for doctor with specific help
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
                .map(|(i, health)| {
                    let is_selected = list_view.state.is_selected(i);

                    // Path status
                    let path_status = if health.path_exists { "OK" } else { "MISSING" };

                    // Cache status
                    let total_caches = health.cache_dirs_exist.len();
                    let healthy_caches = health
                        .cache_dirs_exist
                        .iter()
                        .filter(|(_, exists)| *exists)
                        .count();
                    let cache_status = if total_caches == 0 {
                        "None".to_string()
                    } else if healthy_caches == total_caches {
                        format!("{}/{} OK", healthy_caches, total_caches)
                    } else if healthy_caches == 0 {
                        format!("{}/{} MISSING", healthy_caches, total_caches)
                    } else {
                        format!("{}/{} WARN", healthy_caches, total_caches)
                    };

                    // Issues count
                    let issues_count = if health.issues.is_empty() {
                        "None".to_string()
                    } else {
                        health.issues.len().to_string()
                    };

                    crate::ui::components::create_row_with_selection(
                        vec![
                            health.project_id.to_string(),
                            health.project_name.clone(),
                            path_status.to_string(),
                            cache_status,
                            issues_count,
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

            // Render doctor-specific help
            let help = create_doctor_help(
                list_view.state.selected_count(),
                list_view.items.len(),
                healthy_count,
                total_issues,
                &list_view.state.status_message,
            );
            help.render(f, chunks[1]);

            // Render confirmation popup if needed
            if list_view.show_confirmation {
                let popup = crate::ui::components::confirmation_popup(
                    "Confirm Removal",
                    "Remove broken projects from tracking?",
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
                ListAction::ConfirmAction => {
                    // Remove selected projects
                    let projects_to_remove: Vec<usize> =
                        if list_view.state.selected_items.is_empty() {
                            vec![list_view.state.selected]
                        } else {
                            list_view.state.selected_items.clone()
                        };

                    let mut removed_count = 0;
                    let mut sorted_indexes = projects_to_remove.clone();
                    sorted_indexes.sort_by(|a, b| b.cmp(a));

                    for &idx in &sorted_indexes {
                        if idx < project_healths.len() {
                            let project_id = project_healths[idx].project_id;
                            if db::remove_project(&conn, project_id).is_ok() {
                                project_healths.remove(idx);
                                removed_count += 1;
                            }
                        }
                    }

                    if removed_count > 0 {
                        // Recalculate stats
                        healthy_count = project_healths
                            .iter()
                            .filter(|p| p.issues.is_empty())
                            .count();
                        total_issues = project_healths.iter().map(|p| p.issues.len()).sum();

                        list_view
                            .state
                            .set_status(format!("✓ Removed {} project(s)", removed_count));
                        list_view.items = project_healths.clone();

                        if list_view.state.selected >= project_healths.len()
                            && list_view.state.selected > 0
                        {
                            list_view.state.selected = project_healths.len() - 1;
                        }
                        list_view.state.selected_items.clear();

                        if project_healths.is_empty() {
                            break;
                        }
                    }
                }
                ListAction::ExecuteAction => {
                    list_view
                        .state
                        .set_status("Use 'd' to remove problematic projects");
                }
                ListAction::None => {}
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

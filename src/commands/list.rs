use crate::db;
use crate::ui::{
    components::{ProgressPopup, TableConfig, create_project_list_help},
    views::{ListAction, ListView},
    init_terminal, poll_event, restore_terminal,
};
use crate::utils::{get_dir_size, human_readable_size};
use anyhow::Result;
use chrono::Utc;
use ratatui::layout::Constraint;

pub fn run() -> Result<()> {
    let conn = db::connect()?;
    let mut projects = db::get_all_projects(&conn)?;
    
    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    let table_config = TableConfig::new("Projects")
        .headers(vec!["ID", "Name", "Path", "Size", "Last Cleaned"])
        .constraints(vec![
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(25),
            Constraint::Length(10),
            Constraint::Length(18),
        ]);

    let mut list_view = ListView::new(projects.clone(), table_config);
    let mut terminal = init_terminal()?;
    let mut progress_popup: Option<ProgressPopup> = None;

    loop {
        terminal.draw(|f| {
            if let Some(ref progress) = progress_popup {
                progress.render(f);
            } else {
                // Custom render for project list with specific help
                let size = f.size();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints([ratatui::layout::Constraint::Percentage(70), ratatui::layout::Constraint::Percentage(30)])
                    .split(size);

                // Render table
                let rows = list_view.items
                    .iter()
                    .enumerate()
                    .map(|(i, project)| {
                        let is_selected = list_view.state.is_selected(i);
                        let marker = if is_selected { "✓" } else { " " };
                        let total_size: u64 = project.cache_dirs.iter().map(|cd| get_dir_size(cd)).sum();
                        
                        crate::ui::components::create_row_with_selection(
                            vec![
                                format!("{} {}", marker, project.id),
                                project.name.clone(),
                                project.path.clone(),
                                human_readable_size(total_size),
                                project.last_cleaned.clone(),
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

                // Render custom help
                let help = create_project_list_help(
                    list_view.state.selected_count(),
                    list_view.items.len(),
                    &list_view.state.status_message,
                );
                help.render(f, chunks[1]);

                // Render confirmation popup if needed
                if list_view.show_confirmation {
                    let popup = crate::ui::components::confirmation_popup(
                        "Confirm Removal",
                        "Remove selected projects from tracking?",
                        if list_view.state.selected_count() > 0 {
                            Some(list_view.state.selected_count())
                        } else {
                            Some(1)
                        },
                    );
                    popup.render(f);
                }
            }
        })?;

        if let Some(key) = poll_event()? {
            if progress_popup.is_some() {
                if key == crossterm::event::KeyCode::Esc {
                    progress_popup = None;
                    list_view.state.set_status("Operation cancelled");
                }
                continue;
            }

            match list_view.handle_key(key) {
                ListAction::Quit => break,
                ListAction::ExecuteAction => {
                    let selected_indexes: Vec<usize> = list_view.state.selected_items.clone();
                    if !selected_indexes.is_empty() {
                        progress_popup = Some(ProgressPopup::new("Cleaning Cache", selected_indexes.len()));
                        
                        let mut total_freed = 0u64;
                        for (current_index, &i) in selected_indexes.iter().enumerate() {
                            let proj = &projects[i];
                            
                            if let Some(ref mut progress) = progress_popup {
                                progress.update(&proj.name, current_index + 1);
                            }
                            
                            // Force UI update
                            terminal.draw(|f| {
                                if let Some(ref progress) = progress_popup {
                                    progress.render(f);
                                }
                            })?;

                            // Clean cache directories in parallel
                            let cache_dirs: Vec<String> = proj.cache_dirs.clone();
                            let freed = crate::core::CacheManager::clean_multiple(&cache_dirs)?;
                            total_freed += freed;

                            db::update_project_last_cleaned(&conn, proj.id)?;
                            // Update the project in our local copy
                            if let Some(project) = projects.get_mut(i) {
                                project.last_cleaned = Utc::now().to_rfc3339();
                            }
                        }

                        progress_popup = None;
                        list_view.state.set_status(format!(
                            "✓ Cleaned {} project(s) — freed {}",
                            selected_indexes.len(),
                            human_readable_size(total_freed)
                        ));
                        list_view.state.selected_items.clear();
                    }
                }
                ListAction::ConfirmAction => {
                    let projects_to_delete: Vec<usize> = if list_view.state.selected_items.is_empty() {
                        vec![list_view.state.selected]
                    } else {
                        list_view.state.selected_items.clone()
                    };

                    let mut removed_count = 0;
                    let mut sorted_indexes = projects_to_delete.clone();
                    sorted_indexes.sort_by(|a, b| b.cmp(a));

                    for &idx in &sorted_indexes {
                        if idx < projects.len() {
                            let project_id = projects[idx].id;
                            if db::remove_project(&conn, project_id).is_ok() {
                                projects.remove(idx);
                                removed_count += 1;
                            }
                        }
                    }

                    if removed_count > 0 {
                        list_view.state.set_status(format!("✓ Removed {} project(s)", removed_count));
                        list_view.items = projects.clone();
                        
                        if list_view.state.selected >= projects.len() && list_view.state.selected > 0 {
                            list_view.state.selected = projects.len() - 1;
                        }
                        list_view.state.selected_items.clear();

                        if projects.is_empty() {
                            break;
                        }
                    }
                }
                ListAction::None => {}
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}
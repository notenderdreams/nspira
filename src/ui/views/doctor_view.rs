use crate::commands::doctor::ProjectHealth;
use crate::db;
use crate::ui::{
    TerminalGuard, TerminalEvent, poll_event,
    components::{
        HeaderWidget, TableConfig, TableWidget, build_table_row, confirmation_popup,
        create_doctor_hints, help_popup, project_detail_popup,
    },
    model::UiDoctorItem,
    state::{AppState, StatusType},
};
use crate::utils::human_readable_size;
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
};

pub fn run_doctor_view(
    project_healths: Vec<ProjectHealth>,
    mut healthy_count: usize,
    mut total_issues: usize,
) -> Result<()> {
    if project_healths.is_empty() {
        println!("No projects found in database.");
        return Ok(());
    }

    // Pre-compute health & size data in parallel
    let mut ui_healths = UiDoctorItem::from_healths(&project_healths);

    let mut guard = TerminalGuard::new()?;
    let mut state = AppState::new();
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut filter_issues_only = false;

    loop {
        let display_healths: Vec<(usize, &UiDoctorItem)> = ui_healths
            .iter()
            .enumerate()
            .filter(|(_, h)| {
                if filter_issues_only {
                    !h.issues.is_empty()
                } else {
                    true
                }
            })
            .collect();

        let total_items = display_healths.len();
        state.clamp_selection(total_items);

        if table_state.selected().unwrap_or(0) >= total_items && total_items > 0 {
            table_state.select(Some(total_items - 1));
        }

        // Render pass
        guard.terminal_mut().draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(area);

            // Header
            HeaderWidget::new("Doctor Health Report")
                .metric("Total", ui_healths.len().to_string())
                .metric("Healthy", healthy_count.to_string())
                .metric("Issues", total_issues.to_string())
                .search_query(if filter_issues_only { Some("Filter: Issues Only") } else { None })
                .render(f, chunks[0]);

            // Table
            let rows = display_healths
                .iter()
                .enumerate()
                .map(|(disp_idx, (orig_idx, health))| {
                    let is_cursor = table_state.selected() == Some(disp_idx);
                    let is_checked = state.is_selected(*orig_idx);

                    let path_status = if health.path_exists { "✓ OK" } else { "✖ MISSING" };

                    let total_caches = health.cache_dirs.len();
                    let healthy_caches = health.cache_dirs.iter().filter(|(_, exists, _)| *exists).count();
                    let cache_status = if total_caches == 0 {
                        "None".to_string()
                    } else if healthy_caches == total_caches {
                        format!("{}/{} OK", healthy_caches, total_caches)
                    } else {
                        format!("{}/{} WARN", healthy_caches, total_caches)
                    };

                    let issue_text = if health.issues.is_empty() {
                        "None".to_string()
                    } else {
                        format!("⚠️ {} issue(s)", health.issues.len())
                    };

                    build_table_row(
                        vec![
                            health.project_id.to_string(),
                            health.project_name.clone(),
                            path_status.to_string(),
                            cache_status,
                            issue_text,
                        ],
                        is_checked,
                        is_cursor,
                    )
                })
                .collect();

            let table_config = TableConfig::new("Project Health Checks")
                .headers(vec!["ID", "Name", "Path Status", "Cache Status", "Issues Found"])
                .constraints(vec![
                    Constraint::Length(6),
                    Constraint::Percentage(25),
                    Constraint::Length(15),
                    Constraint::Length(16),
                    Constraint::Percentage(30),
                ]);

            TableWidget::new(table_config)
                .rows(rows)
                .render(f, chunks[1], &mut table_state);

            // Status bar
            let status_msg = if state.status_message.is_empty() {
                format!("Viewing {} projects", display_healths.len())
            } else {
                state.status_message.clone()
            };

            create_doctor_hints(&status_msg, &state.status_type).render(f, chunks[2]);

            // Confirmation popup
            if state.show_help {
                help_popup().render(f);
            } else if state.show_confirmation {
                let count = if state.selected_count() > 0 { state.selected_count() } else { 1 };
                confirmation_popup(
                    "Confirm Removal",
                    "Remove selected broken project(s) from tracking database?",
                    count,
                    Some("This removes the record from tracking; files on disk are untouched."),
                )
                .render(f);
            } else if state.show_detail && !display_healths.is_empty() {
                if let Some(curr_idx) = table_state.selected() {
                    if let Some((_, health)) = display_healths.get(curr_idx) {
                        let cache_details: Vec<(String, u64)> = health
                            .cache_dirs
                            .iter()
                            .map(|(dir, _, sz)| (dir.clone(), *sz))
                            .collect();

                        project_detail_popup(
                            &health.project_name,
                            &health.project_path,
                            &cache_details,
                            &human_readable_size(health.total_size),
                        )
                        .render(f);
                    }
                }
            }
        })?;

        // Controls (16ms tick)
        if let Some(evt) = poll_event(16)? {
            match evt {
                TerminalEvent::Key(key) => {
                    if state.show_help {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter) {
                            state.show_help = false;
                        }
                        continue;
                    }

                    if state.show_detail {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Char('i')) {
                            state.show_detail = false;
                        }
                        continue;
                    }

                    if state.show_confirmation {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Enter => {
                                state.show_confirmation = false;

                                let targets: Vec<usize> = if state.selected_count() == 0 {
                                    if let Some(sel) = table_state.selected() {
                                        if let Some((orig_idx, _)) = display_healths.get(sel) {
                                            vec![*orig_idx]
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    state.selected_items.iter().cloned().collect()
                                };

                                let conn = db::connect()?;
                                let mut removed = 0;
                                let mut sorted_targets = targets;
                                sorted_targets.sort_by(|a, b| b.cmp(a));

                                for idx in sorted_targets {
                                    if idx < ui_healths.len() {
                                        let pid = ui_healths[idx].project_id;
                                        if db::remove_project(&conn, pid).is_ok() {
                                            ui_healths.remove(idx);
                                            removed += 1;
                                        }
                                    }
                                }

                                healthy_count = ui_healths.iter().filter(|p| p.issues.is_empty()).count();
                                total_issues = ui_healths.iter().map(|p| p.issues.len()).sum();

                                state.selected_items.clear();
                                state.set_status(
                                    format!("Removed {} project(s)", removed),
                                    StatusType::Success,
                                );

                                if ui_healths.is_empty() {
                                    break;
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                state.show_confirmation = false;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            state.show_help = true;
                        }
                        KeyCode::Char('f') => {
                            filter_issues_only = !filter_issues_only;
                            state.set_status(
                                if filter_issues_only { "Showing issues only" } else { "Showing all projects" },
                                StatusType::Info,
                            );
                        }
                        KeyCode::Char('i') => {
                            if !display_healths.is_empty() {
                                state.show_detail = true;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if total_items > 0 {
                                let curr = table_state.selected().unwrap_or(0);
                                table_state.select(Some((curr + 1).min(total_items - 1)));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if total_items > 0 {
                                let curr = table_state.selected().unwrap_or(0);
                                table_state.select(Some(curr.saturating_sub(1)));
                            }
                        }
                        KeyCode::Home | KeyCode::Char('g') => {
                            if total_items > 0 {
                                table_state.select(Some(0));
                            }
                        }
                        KeyCode::End | KeyCode::Char('G') => {
                            if total_items > 0 {
                                table_state.select(Some(total_items - 1));
                            }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(sel) = table_state.selected() {
                                if let Some((orig_idx, _)) = display_healths.get(sel) {
                                    if state.selected_items.contains(orig_idx) {
                                        state.selected_items.remove(orig_idx);
                                    } else {
                                        state.selected_items.insert(*orig_idx);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Delete => {
                            if !display_healths.is_empty() {
                                state.show_confirmation = true;
                            }
                        }
                        KeyCode::Esc => state.selected_items.clear(),
                        _ => {}
                    }
                }
                TerminalEvent::Resize(_, _) => {}
            }
        }
    }

    Ok(())
}

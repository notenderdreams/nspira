use crate::core::DetectedProject;
use crate::db;
use crate::ui::{
    TerminalGuard, TerminalEvent, poll_event,
    components::{
        HeaderWidget, TableConfig, TableWidget, build_table_row, confirmation_popup,
        create_scan_hints, help_popup, project_detail_popup, search_input_popup,
    },
    model::UiScanItem,
    state::{AppState, StatusType},
};
use crate::utils::human_readable_size;
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
};

pub fn run_scan_view(detected: Vec<DetectedProject>) -> Result<()> {
    if detected.is_empty() {
        println!("No untracked projects found during filesystem scan.");
        return Ok(());
    }

    // Pre-calculate sizes in parallel before starting UI loop
    let ui_scan = UiScanItem::from_detected(&detected);

    let mut guard = TerminalGuard::new()?;
    let mut state = AppState::new();
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    loop {
        // Pure in-memory filtering (0 I/O)
        let display_projects: Vec<(usize, &UiScanItem)> = ui_scan
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if state.search_query.is_empty() {
                    true
                } else {
                    let q = state.search_query.to_lowercase();
                    p.name.to_lowercase().contains(&q)
                        || p.project_type.to_lowercase().contains(&q)
                        || p.path.to_string_lossy().to_lowercase().contains(&q)
                }
            })
            .collect();

        let total_items = display_projects.len();
        state.clamp_selection(total_items);

        if table_state.selected().unwrap_or(0) >= total_items && total_items > 0 {
            table_state.select(Some(total_items - 1));
        }

        let total_detected_size: u64 = display_projects.iter().map(|(_, p)| p.total_size).sum();

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
            HeaderWidget::new("Scan Results")
                .metric("Detected", display_projects.len().to_string())
                .metric("Potential Cache", human_readable_size(total_detected_size))
                .search_query(if state.is_searching || !state.search_query.is_empty() {
                    Some(state.search_query.as_str())
                } else {
                    None
                })
                .render(f, chunks[0]);

            // Table
            let rows = display_projects
                .iter()
                .enumerate()
                .map(|(disp_idx, (orig_idx, proj))| {
                    let is_checked = state.is_selected(*orig_idx);
                    let is_cursor = table_state.selected() == Some(disp_idx);
                    let marker = if is_checked { "✓" } else { " " };

                    build_table_row(
                        vec![
                            marker.to_string(),
                            proj.name.clone(),
                            proj.project_type.clone(),
                            proj.path.to_string_lossy().to_string(),
                            human_readable_size(proj.total_size),
                            proj.cache_dirs.len().to_string(),
                        ],
                        is_checked,
                        is_cursor,
                    )
                })
                .collect();

            let table_config = TableConfig::new("Discovered Projects")
                .headers(vec!["", "Name", "Type", "Path", "Cache Size", "Caches"])
                .constraints(vec![
                    Constraint::Length(3),
                    Constraint::Percentage(20),
                    Constraint::Length(12),
                    Constraint::Percentage(45),
                    Constraint::Length(12),
                    Constraint::Length(8),
                ]);

            TableWidget::new(table_config)
                .rows(rows)
                .render(f, chunks[1], &mut table_state);

            // Status Bar
            let status_msg = if state.status_message.is_empty() {
                format!("Selected {}/{}", state.selected_count(), ui_scan.len())
            } else {
                state.status_message.clone()
            };

            create_scan_hints(&status_msg, &state.status_type).render(f, chunks[2]);

            // Modal Overlays
            if state.show_help {
                help_popup().render(f);
            } else if state.is_searching {
                search_input_popup(&state.search_query).render(f);
            } else if state.show_confirmation {
                let count = if state.selected_count() > 0 { state.selected_count() } else { 1 };
                confirmation_popup(
                    "Track Selected Projects",
                    "Add selected projects to tracking database?",
                    count,
                    None,
                )
                .render(f);
            } else if state.show_detail && !display_projects.is_empty() {
                if let Some(curr_idx) = table_state.selected() {
                    if let Some((_, proj)) = display_projects.get(curr_idx) {
                        let cache_details: Vec<(String, u64)> = proj
                            .cache_dirs
                            .iter()
                            .map(|(cd, sz)| (cd.to_string_lossy().to_string(), *sz))
                            .collect();

                        project_detail_popup(
                            &proj.name,
                            &proj.path.to_string_lossy(),
                            &cache_details,
                            &human_readable_size(proj.total_size),
                        )
                        .render(f);
                    }
                }
            }
        })?;

        // Fast Event loop (16ms)
        if let Some(evt) = poll_event(16)? {
            match evt {
                TerminalEvent::Key(key) => {
                    if state.show_help {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter) {
                            state.show_help = false;
                        }
                        continue;
                    }

                    if state.is_searching {
                        match key.code {
                            KeyCode::Enter => state.is_searching = false,
                            KeyCode::Esc => {
                                state.is_searching = false;
                                state.search_query.clear();
                            }
                            KeyCode::Backspace => {
                                state.search_query.pop();
                            }
                            KeyCode::Char(c) => state.search_query.push(c),
                            _ => {}
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
                                        if let Some((orig_idx, _)) = display_projects.get(sel) {
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
                                let mut added = 0;
                                for idx in targets {
                                    if idx < detected.len() {
                                        let p = &detected[idx];
                                        if let Ok(pid) = db::add_project(&conn, &p.name, p.path.to_str().unwrap_or("")) {
                                            for cd in &p.cache_dirs {
                                                let _ = db::add_cache_directory(&conn, pid, cd.to_str().unwrap_or(""));
                                            }
                                            added += 1;
                                        }
                                    }
                                }

                                state.set_status(
                                    format!("Added {} project(s) to tracking!", added),
                                    StatusType::Success,
                                );
                                break;
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
                        KeyCode::Char('/') => state.is_searching = true,
                        KeyCode::Char('i') => {
                            if !display_projects.is_empty() {
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
                                if let Some((orig_idx, _)) = display_projects.get(sel) {
                                    if state.selected_items.contains(orig_idx) {
                                        state.selected_items.remove(orig_idx);
                                    } else {
                                        state.selected_items.insert(*orig_idx);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('a') => state.toggle_select_all(ui_scan.len()),
                        KeyCode::Char('v') => state.invert_selection(ui_scan.len()),
                        KeyCode::Enter => {
                            if !display_projects.is_empty() {
                                state.show_confirmation = true;
                            }
                        }
                        KeyCode::Esc => {
                            if !state.search_query.is_empty() {
                                state.search_query.clear();
                            } else {
                                state.selected_items.clear();
                            }
                        }
                        _ => {}
                    }
                }
                TerminalEvent::Resize(_, _) => {}
            }
        }
    }

    Ok(())
}

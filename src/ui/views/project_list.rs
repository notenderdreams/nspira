use crate::db::{self};
use crate::ui::{
    TerminalGuard, TerminalEvent, poll_event,
    components::{
        HeaderWidget, ProgressPopup, StatsWidget, TableConfig, TableWidget,
        build_table_row, confirmation_popup, create_project_hints, help_popup, project_detail_popup, search_input_popup,
    },
    model::UiProjectItem,
    state::{AppState, SortMode, StatusType, ViewTab},
};
use crate::utils::human_readable_size;
use anyhow::Result;
use chrono::Utc;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
};
use rusqlite::Connection;

pub fn run_project_list_view(conn: &Connection) -> Result<()> {
    let projects = db::get_all_projects(conn)?;

    if projects.is_empty() {
        println!("No tracked projects found. Use `nspira init` or `nspira scan` to add projects.");
        return Ok(());
    }

    // Pre-calculate project sizes in parallel across all CPU cores
    let mut ui_projects = UiProjectItem::from_projects(&projects);

    let mut guard = TerminalGuard::new()?;
    let mut state = AppState::new();
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut progress_popup: Option<ProgressPopup> = None;

    loop {
        // Fast, purely in-memory filter and sort (0 I/O)
        let mut display_projects: Vec<(usize, &UiProjectItem)> = ui_projects
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if state.search_query.is_empty() {
                    true
                } else {
                    let q = state.search_query.to_lowercase();
                    p.name.to_lowercase().contains(&q)
                        || p.path.to_lowercase().contains(&q)
                        || p.cache_dirs.iter().any(|d| d.path.to_lowercase().contains(&q))
                }
            })
            .collect();

        match state.sort_mode {
            SortMode::Name => display_projects.sort_by(|a, b| a.1.name.cmp(&b.1.name)),
            SortMode::Size => display_projects.sort_by(|a, b| b.1.total_size.cmp(&a.1.total_size)),
            SortMode::LastCleaned => display_projects.sort_by(|a, b| b.1.last_cleaned.cmp(&a.1.last_cleaned)),
            SortMode::Id => display_projects.sort_by(|a, b| a.1.id.cmp(&b.1.id)),
        }

        let total_items = display_projects.len();
        state.clamp_selection(total_items);

        if table_state.selected().unwrap_or(0) >= total_items && total_items > 0 {
            table_state.select(Some(total_items - 1));
        }

        let total_size_all: u64 = display_projects.iter().map(|(_, p)| p.total_size).sum();

        // High-performance render pass
        guard.terminal_mut().draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header banner
                    Constraint::Min(5),    // Main table or stats
                    Constraint::Length(3), // Status & hotkey bar
                ])
                .split(area);

            // 1. Render Header
            let search_opt = if state.is_searching || !state.search_query.is_empty() {
                Some(state.search_query.as_str())
            } else {
                None
            };

            HeaderWidget::new("Tracked Projects")
                .metric("Total", display_projects.len().to_string())
                .metric("Cache Size", human_readable_size(total_size_all))
                .metric("Sort", state.sort_mode.label())
                .active_tab(state.active_tab)
                .search_query(search_opt)
                .render(f, chunks[0]);

            // 2. Render Main Body
            match state.active_tab {
                ViewTab::Projects => {
                    let rows = display_projects
                        .iter()
                        .enumerate()
                        .map(|(disp_idx, (orig_idx, proj))| {
                            let is_checked = state.is_selected(*orig_idx);
                            let is_cursor = table_state.selected() == Some(disp_idx);
                            let marker = if is_checked { "✓" } else { " " };

                            build_table_row(
                                vec![
                                    format!("{} {}", marker, proj.id),
                                    proj.name.clone(),
                                    proj.path.clone(),
                                    human_readable_size(proj.total_size),
                                    proj.cache_dirs.len().to_string(),
                                    proj.formatted_last_cleaned.clone(),
                                ],
                                is_checked,
                                is_cursor,
                            )
                        })
                        .collect();

                    let table_config = TableConfig::new("Projects List")
                        .headers(vec!["ID", "Name", "Path", "Size", "Caches", "Last Cleaned"])
                        .constraints(vec![
                            Constraint::Length(7),
                            Constraint::Percentage(20),
                            Constraint::Percentage(38),
                            Constraint::Length(12),
                            Constraint::Length(8),
                            Constraint::Length(15),
                        ]);

                    TableWidget::new(table_config)
                        .rows(rows)
                        .render(f, chunks[1], &mut table_state);
                }
                ViewTab::Stats => {
                    StatsWidget::new(&ui_projects).render(f, chunks[1]);
                }
            }

            // 3. Render Status Bar
            let status_msg = if state.status_message.is_empty() {
                format!("Selected {}/{}", state.selected_count(), ui_projects.len())
            } else {
                state.status_message.clone()
            };

            create_project_hints(&status_msg, &state.status_type).render(f, chunks[2]);

            // 4. Render Modals & Overlays
            if let Some(ref progress) = progress_popup {
                progress.render(f);
            } else if state.show_help {
                help_popup().render(f);
            } else if state.is_searching {
                search_input_popup(&state.search_query).render(f);
            } else if state.show_confirmation {
                let count = if state.selected_count() > 0 {
                    state.selected_count()
                } else {
                    1
                };
                confirmation_popup(
                    "Confirm Action",
                    "Proceed with action on selected projects?",
                    count,
                    Some("This action will operate on your disk cache directories!"),
                )
                .render(f);
            } else if state.show_detail && !display_projects.is_empty() {
                if let Some(curr_idx) = table_state.selected() {
                    if let Some((_, proj)) = display_projects.get(curr_idx) {
                        let cache_details: Vec<(String, u64)> = proj
                            .cache_dirs
                            .iter()
                            .map(|cd| (cd.path.clone(), cd.size))
                            .collect();

                        project_detail_popup(
                            &proj.name,
                            &proj.path,
                            &cache_details,
                            &human_readable_size(proj.total_size),
                        )
                        .render(f);
                    }
                }
            }
        })?;

        // Fast Event Polling (16ms = ~60 FPS responsiveness)
        if let Some(evt) = poll_event(16)? {
            match evt {
                TerminalEvent::Key(key) => {
                    // Help modal mode
                    if state.show_help {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter) {
                            state.show_help = false;
                        }
                        continue;
                    }

                    // Search input modal mode
                    if state.is_searching {
                        match key.code {
                            KeyCode::Enter => {
                                state.is_searching = false;
                                state.set_status("Filter applied", StatusType::Info);
                            }
                            KeyCode::Esc => {
                                state.is_searching = false;
                                state.search_query.clear();
                                state.set_status("Filter cleared", StatusType::Info);
                            }
                            KeyCode::Backspace => {
                                state.search_query.pop();
                            }
                            KeyCode::Char(c) => {
                                state.search_query.push(c);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Detail popup mode
                    if state.show_detail {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Char('i')) {
                            state.show_detail = false;
                        }
                        continue;
                    }

                    // Confirmation modal mode
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

                                let mut removed = 0;
                                let mut sorted_targets = targets;
                                sorted_targets.sort_by(|a, b| b.cmp(a));

                                for idx in sorted_targets {
                                    if idx < ui_projects.len() {
                                        let pid = ui_projects[idx].id;
                                        if db::remove_project(conn, pid).is_ok() {
                                            ui_projects.remove(idx);
                                            removed += 1;
                                        }
                                    }
                                }

                                state.selected_items.clear();
                                state.set_status(
                                    format!("Removed {} project(s) from tracking", removed),
                                    StatusType::Success,
                                );

                                if ui_projects.is_empty() {
                                    break;
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                state.show_confirmation = false;
                                state.set_status("Action cancelled", StatusType::Info);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // View controls
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            state.show_help = true;
                        }
                        KeyCode::Tab => {
                            state.active_tab = match state.active_tab {
                                ViewTab::Projects => ViewTab::Stats,
                                ViewTab::Stats => ViewTab::Projects,
                            };
                        }
                        KeyCode::Char('1') => state.active_tab = ViewTab::Projects,
                        KeyCode::Char('2') => state.active_tab = ViewTab::Stats,
                        KeyCode::Char('/') => {
                            state.is_searching = true;
                        }
                        KeyCode::Char('s') => {
                            state.sort_mode = state.sort_mode.next();
                            state.set_status(format!("Sorted by {}", state.sort_mode.label()), StatusType::Info);
                        }
                        KeyCode::Char('i') => {
                            if !display_projects.is_empty() {
                                state.show_detail = true;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if total_items > 0 {
                                let curr = table_state.selected().unwrap_or(0);
                                let next = (curr + 1).min(total_items - 1);
                                table_state.select(Some(next));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if total_items > 0 {
                                let curr = table_state.selected().unwrap_or(0);
                                let prev = curr.saturating_sub(1);
                                table_state.select(Some(prev));
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
                        KeyCode::Char('a') => {
                            state.toggle_select_all(ui_projects.len());
                        }
                        KeyCode::Char('v') => {
                            state.invert_selection(ui_projects.len());
                        }
                        KeyCode::Char('c') | KeyCode::Enter => {
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

                            if !targets.is_empty() {
                                let mut total_freed = 0u64;
                                progress_popup = Some(ProgressPopup::new("Cleaning Cache", targets.len()));

                                for (step, &orig_idx) in targets.iter().enumerate() {
                                    if orig_idx < ui_projects.len() {
                                        let proj = &ui_projects[orig_idx];

                                        if let Some(ref mut prog) = progress_popup {
                                            prog.update(&proj.name, step + 1, total_freed);
                                        }

                                        guard.terminal_mut().draw(|f| {
                                            if let Some(ref prog) = progress_popup {
                                                prog.render(f);
                                            }
                                        })?;

                                        let paths: Vec<String> = proj.cache_dirs.iter().map(|c| c.path.clone()).collect();
                                        let freed = crate::core::CacheManager::clean_multiple(&paths)?;
                                        total_freed += freed;

                                        let _ = db::update_project_last_cleaned(conn, proj.id);

                                        // Update in-memory cached state immediately
                                        let now_str = Utc::now().to_rfc3339();
                                        ui_projects[orig_idx].last_cleaned = now_str.clone();
                                        ui_projects[orig_idx].formatted_last_cleaned = format_cleaned_date(&now_str);
                                        ui_projects[orig_idx].total_size = 0;
                                        for cd in &mut ui_projects[orig_idx].cache_dirs {
                                            cd.size = 0;
                                        }
                                    }
                                }

                                progress_popup = None;
                                state.selected_items.clear();
                                state.set_status(
                                    format!("✓ Cleaned {} project(s) — freed {}", targets.len(), human_readable_size(total_freed)),
                                    StatusType::Success,
                                );
                            } else {
                                state.set_status("No projects selected to clean", StatusType::Warning);
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Delete => {
                            if !display_projects.is_empty() {
                                state.show_confirmation = true;
                            }
                        }
                        KeyCode::Esc => {
                            if !state.search_query.is_empty() {
                                state.search_query.clear();
                                state.set_status("Filter cleared", StatusType::Info);
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

fn format_cleaned_date(raw_date: &str) -> String {
    if raw_date == "Never" || raw_date.is_empty() {
        "Never".to_string()
    } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(raw_date) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        raw_date.chars().take(10).collect()
    }
}

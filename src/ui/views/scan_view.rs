use crate::core::DetectedProject;
use crate::db;
use crate::ui::{
    TerminalEvent, TerminalGuard,
    components::{
        HeaderWidget, TREE_KEYS, TableConfig, TableWidget, confirmation_popup, create_scan_hints,
        help_popup, project_detail_popup, search_input_popup,
    },
    model::Entry,
    poll_event,
    state::StatusType,
    tree::{TreeList, path_width, tree_columns, tree_rows},
};
use crate::utils::human_readable_size;
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use rusqlite::Connection;
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Track,
    Ignore,
}

#[derive(Clone, Copy, PartialEq)]
enum Modal {
    None,
    Help,
    Detail,
    Confirm(Action),
}

const ACTION_KEYS: &[(&str, &str)] = &[
    ("Enter / t", "Track selected (or cursor) projects"),
    ("x", "Ignore in future scans (project or cache dir)"),
    ("i", "Project details"),
    ("q", "Quit"),
];

pub fn run_scan_view(detected: Vec<DetectedProject>) -> Result<()> {
    let conn = db::connect()?;
    let mut entries = Entry::from_detected(&detected);
    entries.sort_by_key(|e| std::cmp::Reverse(e.total_size()));

    let mut guard = TerminalGuard::new()?;
    let mut list = TreeList::new();
    let mut modal = Modal::None;
    let mut status = (String::new(), StatusType::Info);

    loop {
        let order: Vec<usize> = (0..entries.len()).filter(|&i| entries[i].matches(&list.query)).collect();
        let visible = list.visible(&entries, &order);
        list.clamp(visible.len());
        let cursor = list.cursor(&visible);
        let targets = list.targets(&entries, cursor);
        let target_dirs: usize = targets.values().map(Vec::len).sum();

        guard.terminal_mut().draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
                .split(f.size());

            let total: u64 = order.iter().map(|&i| entries[i].total_size()).sum();
            HeaderWidget::new("Scan")
                .metric("Found", order.len().to_string())
                .metric("Cache", human_readable_size(total))
                .search_query((list.searching || !list.query.is_empty()).then_some(list.query.as_str()))
                .render(f, chunks[0]);

            let (headers, widths) = tree_columns("Type");
            let rows = tree_rows(&entries, &visible, &list, path_width(chunks[1].width), |e| e.kind.clone());
            TableWidget::new(TableConfig::new("Untracked projects").headers(headers).constraints(widths))
                .rows(rows)
                .render(f, chunks[1], &mut list.table);

            let msg = if !status.0.is_empty() {
                status.0.clone()
            } else if !list.selected.is_empty() {
                format!("{} cache dir(s) in {} project(s) selected", target_dirs, targets.len())
            } else {
                String::new()
            };
            create_scan_hints(&msg, &status.1).render(f, chunks[2]);

            match modal {
                Modal::Help => help_popup(&[("Navigation", TREE_KEYS), ("Actions", ACTION_KEYS)]).render(f),
                Modal::Detail => {
                    if let Some(node) = cursor {
                        let e = &entries[node.project()];
                        let caches: Vec<(String, u64)> =
                            e.caches.iter().map(|c| (c.path.clone(), c.size)).collect();
                        project_detail_popup(&e.name, &e.path, &caches, &human_readable_size(e.total_size()))
                            .render(f);
                    }
                }
                Modal::Confirm(Action::Track) => confirmation_popup(
                    "Track projects",
                    format!("Track {} project(s) with {} cache dir(s)?", targets.len(), target_dirs),
                    targets.len(),
                    None,
                )
                .render(f),
                Modal::Confirm(Action::Ignore) => confirmation_popup(
                    "Ignore",
                    "Skip these in future scans? Fully selected projects are ignored with their whole folder.",
                    targets.len(),
                    Some("Undo with `nspira ignore remove <path>`."),
                )
                .render(f),
                Modal::None if list.searching => search_input_popup(&list.query).render(f),
                Modal::None => {}
            }
        })?;

        let Some(TerminalEvent::Key(key)) = poll_event(250)? else {
            continue;
        };
        let code = key.code;

        match modal {
            Modal::Help | Modal::Detail => {
                if matches!(code, KeyCode::Esc | KeyCode::Char('q' | '?' | 'i') | KeyCode::Enter) {
                    modal = Modal::None;
                }
                continue;
            }
            Modal::Confirm(action) => {
                match code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        modal = Modal::None;
                        status = match action {
                            Action::Track => {
                                let n = track_targets(&conn, &mut entries, &targets)?;
                                (format!("Now tracking {} project(s)", n), StatusType::Success)
                            }
                            Action::Ignore => {
                                let n = ignore_targets(&conn, &mut entries, &targets)?;
                                (format!("Ignored {} path(s)", n), StatusType::Success)
                            }
                        };
                        list.reset();
                        if entries.is_empty() {
                            break;
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        modal = Modal::None;
                        status = ("Cancelled".into(), StatusType::Info);
                    }
                    _ => {}
                }
                continue;
            }
            Modal::None => {}
        }

        if list.searching {
            list.handle_search_key(code);
            continue;
        }

        status.0.clear();
        if list.handle_key(code, &entries, &visible, &order) {
            continue;
        }

        match code {
            KeyCode::Char('q') => break,
            KeyCode::Char('?') => modal = Modal::Help,
            KeyCode::Char('i') if cursor.is_some() => modal = Modal::Detail,
            KeyCode::Enter | KeyCode::Char('t') if target_dirs > 0 => modal = Modal::Confirm(Action::Track),
            KeyCode::Char('x') if target_dirs > 0 => modal = Modal::Confirm(Action::Ignore),
            _ => {}
        }
    }

    Ok(())
}

/// Add targeted projects (with only their targeted caches) to tracking.
fn track_targets(conn: &Connection, entries: &mut Vec<Entry>, targets: &BTreeMap<usize, Vec<usize>>) -> Result<usize> {
    let mut added = 0;
    for (&p, cs) in targets.iter().rev() {
        let e = &entries[p];
        let caches: Vec<String> = cs.iter().map(|&c| e.caches[c].path.clone()).collect();
        if db::get_project_by_path(conn, &e.path)?.is_none() {
            db::add_project(conn, &e.name, &e.path, &caches)?;
            added += 1;
        }
        entries.remove(p);
    }
    Ok(added)
}

/// Ignore whole projects when fully targeted, otherwise individual cache dirs.
fn ignore_targets(conn: &Connection, entries: &mut Vec<Entry>, targets: &BTreeMap<usize, Vec<usize>>) -> Result<usize> {
    let mut ignored = 0;
    for (&p, cs) in targets.iter().rev() {
        if cs.len() == entries[p].caches.len() {
            db::add_ignored(conn, &entries[p].path)?;
            entries.remove(p);
            ignored += 1;
        } else {
            for &c in cs.iter().rev() {
                db::add_ignored(conn, &entries[p].caches[c].path)?;
                entries[p].caches.remove(c);
                ignored += 1;
            }
        }
    }
    Ok(ignored)
}

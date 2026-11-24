use crate::core::{CleanTotals, clean_project};
use crate::db;
use crate::ui::{
    TerminalEvent, TerminalGuard,
    components::{
        HeaderWidget, ProgressPopup, StatsWidget, TREE_KEYS, TableConfig, TableWidget,
        confirmation_popup, create_project_hints, help_popup, project_detail_popup,
        search_input_popup,
    },
    model::{Entry, format_cleaned},
    poll_event,
    state::{SortMode, StatusType, ViewTab},
    tree::{TreeList, path_width, tree_columns, tree_rows},
};
use crate::utils::{get_dir_size, human_readable_size, logger};
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use rusqlite::Connection;
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Clean,
    Untrack,
}

#[derive(Clone, Copy, PartialEq)]
enum Modal {
    None,
    Help,
    Detail,
    Confirm(Action),
}

const ACTION_KEYS: &[(&str, &str)] = &[
    ("Enter", "Expand / collapse project"),
    ("c", "Clean selected (or cursor) caches"),
    ("d / Del", "Stop tracking project or cache dir"),
    ("i", "Project details"),
    ("s", "Cycle sort: name, size, cleaned, id"),
    ("Tab / 1 / 2", "Projects / stats"),
    ("q", "Quit"),
];

fn sorted_order(entries: &[Entry], sort: SortMode, query: &str) -> Vec<usize> {
    let mut order: Vec<usize> = (0..entries.len()).filter(|&i| entries[i].matches(query)).collect();
    match sort {
        SortMode::Name => order.sort_by_key(|&i| entries[i].name.to_lowercase()),
        SortMode::Size => order.sort_by_key(|&i| std::cmp::Reverse(entries[i].total_size())),
        SortMode::LastCleaned => {
            order.sort_by(|&a, &b| entries[b].last_cleaned.cmp(&entries[a].last_cleaned))
        }
        SortMode::Id => order.sort_by_key(|&i| entries[i].id),
    }
    order
}

pub fn run_project_list_view(conn: &Connection) -> Result<()> {
    let projects = db::get_all_projects(conn)?;
    if projects.is_empty() {
        logger::info("No tracked projects. Use `nspira init` or `nspira scan` to add some.");
        return Ok(());
    }
    let confirm_clean = crate::config::Config::load()
        .map(|c| c.clean.confirm_before_clean)
        .unwrap_or(true);

    let mut entries = Entry::from_projects(&projects);
    let mut guard = TerminalGuard::new()?;
    let mut list = TreeList::new();
    let mut tab = ViewTab::Projects;
    let mut sort = SortMode::Name;
    let mut modal = Modal::None;
    let mut status = (String::new(), StatusType::Info);

    loop {
        let order = sorted_order(&entries, sort, &list.query);
        let visible = list.visible(&entries, &order);
        list.clamp(visible.len());
        let cursor = list.cursor(&visible);
        let targets = list.targets(&entries, cursor);
        let target_dirs: usize = targets.values().map(Vec::len).sum();
        let target_bytes: u64 = targets
            .iter()
            .flat_map(|(&p, cs)| cs.iter().map(move |&c| (p, c)))
            .map(|(p, c)| entries[p].caches[c].size)
            .sum();

        guard.terminal_mut().draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
                .split(f.size());

            let filtered_bytes: u64 = order.iter().map(|&i| entries[i].total_size()).sum();
            HeaderWidget::new("Tracked")
                .metric("Projects", order.len().to_string())
                .metric("Cache", human_readable_size(filtered_bytes))
                .metric("Sort", sort.label())
                .active_tab(tab)
                .search_query((list.searching || !list.query.is_empty()).then_some(list.query.as_str()))
                .render(f, chunks[0]);

            match tab {
                ViewTab::Projects => {
                    let (headers, widths) = tree_columns("Last cleaned");
                    let rows = tree_rows(&entries, &visible, &list, path_width(chunks[1].width), |e| {
                        format_cleaned(e.last_cleaned.as_deref())
                    });
                    TableWidget::new(TableConfig::new("Projects").headers(headers).constraints(widths))
                        .rows(rows)
                        .render(f, chunks[1], &mut list.table);
                }
                ViewTab::Stats => StatsWidget::new(&entries).render(f, chunks[1]),
            }

            let msg = if !status.0.is_empty() {
                status.0.clone()
            } else if !list.selected.is_empty() {
                format!("{} cache dir(s) selected, {}", target_dirs, human_readable_size(target_bytes))
            } else {
                String::new()
            };
            create_project_hints(&msg, &status.1).render(f, chunks[2]);

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
                Modal::Confirm(Action::Clean) => confirmation_popup(
                    "Clean caches",
                    format!(
                        "Delete the contents of {} cache dir(s), {}?",
                        target_dirs,
                        human_readable_size(target_bytes)
                    ),
                    targets.len(),
                    Some("Files inside these directories are permanently deleted."),
                )
                .render(f),
                Modal::Confirm(Action::Untrack) => confirmation_popup(
                    "Stop tracking",
                    format!(
                        "Stop tracking {} cache dir(s)? Fully selected projects are removed.",
                        target_dirs
                    ),
                    targets.len(),
                    Some("Nothing on disk is deleted."),
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
                            Action::Clean => clean_status(clean_targets(conn, &mut guard, &mut entries, &targets)?),
                            Action::Untrack => {
                                let (projects, dirs) = untrack_targets(conn, &mut entries, &targets)?;
                                list.reset();
                                (
                                    format!("Untracked {} project(s), {} cache dir(s)", projects, dirs),
                                    StatusType::Success,
                                )
                            }
                        };
                        list.selected.clear();
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
        if tab == ViewTab::Projects && list.handle_key(code, &entries, &visible, &order) {
            continue;
        }

        match code {
            KeyCode::Char('q') => break,
            KeyCode::Char('?') => modal = Modal::Help,
            KeyCode::Tab => {
                tab = if tab == ViewTab::Projects { ViewTab::Stats } else { ViewTab::Projects }
            }
            KeyCode::Char('1') => tab = ViewTab::Projects,
            KeyCode::Char('2') => tab = ViewTab::Stats,
            KeyCode::Char('s') => sort = sort.next(),
            KeyCode::Enter => {
                if let Some(node) = cursor {
                    let p = node.project();
                    if !list.expanded.remove(&p) {
                        list.expanded.insert(p);
                    }
                }
            }
            KeyCode::Char('i') if cursor.is_some() => modal = Modal::Detail,
            KeyCode::Char('c') if target_dirs > 0 => {
                if confirm_clean {
                    modal = Modal::Confirm(Action::Clean);
                } else {
                    status = clean_status(clean_targets(conn, &mut guard, &mut entries, &targets)?);
                    list.selected.clear();
                }
            }
            KeyCode::Char('d') | KeyCode::Delete if cursor.is_some() => {
                modal = Modal::Confirm(Action::Untrack)
            }
            _ => {}
        }
    }

    Ok(())
}

fn clean_status(t: CleanTotals) -> (String, StatusType) {
    let mut msg = format!("Cleaned {} dir(s), freed {}", t.dirs, human_readable_size(t.bytes));
    if t.failed > 0 {
        msg.push_str(&format!(", {} failed", t.failed));
        return (msg, StatusType::Warning);
    }
    (msg, StatusType::Success)
}

/// Clean the targeted caches with a progress popup; refresh sizes afterwards.
fn clean_targets(
    conn: &Connection,
    guard: &mut TerminalGuard,
    entries: &mut [Entry],
    targets: &BTreeMap<usize, Vec<usize>>,
) -> Result<CleanTotals> {
    let steps = targets.values().map(Vec::len).sum();
    let mut progress = ProgressPopup::new("Cleaning", steps);
    guard.terminal_mut().draw(|f| progress.render(f))?;

    let mut totals = CleanTotals::default();
    let mut step = 0;
    for (&p, cs) in targets {
        let Some(id) = entries[p].id else { continue };
        let paths: Vec<String> = cs.iter().map(|&c| entries[p].caches[c].path.clone()).collect();
        let freed_before = totals.bytes;
        let mut freed = 0;
        let t = clean_project(conn, id, &paths, |path, result| {
            step += 1;
            freed += *result.as_ref().unwrap_or(&0);
            progress.update(path, step, freed_before + freed);
            let _ = guard.terminal_mut().draw(|f| progress.render(f));
        })?;
        totals.add(t);

        for &c in cs {
            let cache = &mut entries[p].caches[c];
            cache.size = get_dir_size(&cache.path);
        }
        entries[p].last_cleaned = Some(chrono::Utc::now().to_rfc3339());
    }
    Ok(totals)
}

/// Untrack whole projects when all their caches are targeted, otherwise just
/// the targeted cache dirs. Returns (projects removed, cache dirs removed).
fn untrack_targets(
    conn: &Connection,
    entries: &mut Vec<Entry>,
    targets: &BTreeMap<usize, Vec<usize>>,
) -> Result<(usize, usize)> {
    let (mut projects, mut dirs) = (0, 0);
    // Highest index first so earlier indices stay valid while removing.
    for (&p, cs) in targets.iter().rev() {
        let Some(id) = entries[p].id else { continue };
        if cs.len() == entries[p].caches.len() {
            db::remove_project(conn, id)?;
            entries.remove(p);
            projects += 1;
        } else {
            for &c in cs.iter().rev() {
                db::remove_cache_dir(conn, id, &entries[p].caches[c].path)?;
                entries[p].caches.remove(c);
                dirs += 1;
            }
        }
    }
    Ok((projects, dirs))
}

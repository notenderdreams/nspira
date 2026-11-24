//! Shared state for the expandable project -> cache-dir tables (list + scan views).

use crate::ui::model::{Entry, tilde};
use crate::ui::theme::Theme;
use crate::utils::human_readable_size;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint},
    style::Style,
    text::Line,
    widgets::{Cell, Row, TableState},
};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node {
    Project(usize),
    Cache(usize, usize),
}

impl Node {
    pub fn project(self) -> usize {
        match self {
            Node::Project(p) | Node::Cache(p, _) => p,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    All,
    Some,
    None,
}

#[derive(Default)]
pub struct TreeList {
    pub table: TableState,
    pub expanded: HashSet<usize>,
    /// (project index, cache index) pairs.
    pub selected: HashSet<(usize, usize)>,
    pub query: String,
    pub searching: bool,
}

impl TreeList {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.table.select(Some(0));
        s
    }

    /// Visible rows for projects in `order` (already sorted/filtered).
    pub fn visible(&self, entries: &[Entry], order: &[usize]) -> Vec<Node> {
        let mut rows = Vec::new();
        for &p in order {
            rows.push(Node::Project(p));
            if self.expanded.contains(&p) {
                rows.extend((0..entries[p].caches.len()).map(|c| Node::Cache(p, c)));
            }
        }
        rows
    }

    pub fn cursor(&self, visible: &[Node]) -> Option<Node> {
        self.table.selected().and_then(|i| visible.get(i).copied())
    }

    pub fn clamp(&mut self, len: usize) {
        let i = self.table.selected().unwrap_or(0);
        self.table.select(Some(i.min(len.saturating_sub(1))));
    }

    pub fn check(&self, entries: &[Entry], p: usize) -> Check {
        let n = entries[p].caches.len();
        let k = (0..n).filter(|c| self.selected.contains(&(p, *c))).count();
        match k {
            0 => Check::None,
            k if k == n => Check::All,
            _ => Check::Some,
        }
    }

    /// Selected caches grouped by project; falls back to the cursor row.
    pub fn targets(&self, entries: &[Entry], cursor: Option<Node>) -> BTreeMap<usize, Vec<usize>> {
        let mut out: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        if self.selected.is_empty() {
            match cursor {
                Some(Node::Project(p)) => {
                    out.insert(p, (0..entries[p].caches.len()).collect());
                }
                Some(Node::Cache(p, c)) => {
                    out.insert(p, vec![c]);
                }
                None => {}
            }
        } else {
            for &(p, c) in &self.selected {
                out.entry(p).or_default().push(c);
            }
            out.values_mut().for_each(|v| v.sort_unstable());
        }
        out
    }

    /// Forget selection/expansion; call after entries are removed or reindexed.
    pub fn reset(&mut self) {
        self.selected.clear();
        self.expanded.clear();
    }

    /// Text input while the filter prompt is open.
    pub fn handle_search_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => self.searching = false,
            KeyCode::Esc => {
                self.searching = false;
                self.query.clear();
            }
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Char(c) => self.query.push(c),
            _ => {}
        }
        self.table.select(Some(0));
    }

    /// Navigation, expand/collapse, selection and filter keys.
    /// Returns false when the key is not handled here.
    pub fn handle_key(&mut self, key: KeyCode, entries: &[Entry], visible: &[Node], order: &[usize]) -> bool {
        let len = visible.len();
        let cur = self.table.selected().unwrap_or(0);
        let page = 10;
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.table.select(Some((cur + 1).min(len.saturating_sub(1)))),
            KeyCode::Up | KeyCode::Char('k') => self.table.select(Some(cur.saturating_sub(1))),
            KeyCode::PageDown => self.table.select(Some((cur + page).min(len.saturating_sub(1)))),
            KeyCode::PageUp => self.table.select(Some(cur.saturating_sub(page))),
            KeyCode::Home | KeyCode::Char('g') => self.table.select(Some(0)),
            KeyCode::End | KeyCode::Char('G') => self.table.select(Some(len.saturating_sub(1))),
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(Node::Project(p)) = self.cursor(visible)
                    && !self.expanded.insert(p)
                    && !entries[p].caches.is_empty()
                {
                    self.table.select(Some(cur + 1)); // already open: step into it
                }
            }
            KeyCode::Left | KeyCode::Char('h') => match self.cursor(visible) {
                Some(Node::Cache(p, _)) => {
                    self.expanded.remove(&p);
                    if let Some(i) = visible.iter().position(|n| *n == Node::Project(p)) {
                        self.table.select(Some(i));
                    }
                }
                Some(Node::Project(p)) => {
                    self.expanded.remove(&p);
                }
                None => {}
            },
            KeyCode::Char(' ') => {
                match self.cursor(visible) {
                    Some(Node::Project(p)) => {
                        let all = self.check(entries, p) == Check::All;
                        for c in 0..entries[p].caches.len() {
                            if all {
                                self.selected.remove(&(p, c));
                            } else {
                                self.selected.insert((p, c));
                            }
                        }
                    }
                    Some(Node::Cache(p, c)) if !self.selected.remove(&(p, c)) => {
                        self.selected.insert((p, c));
                    }
                    Some(Node::Cache(..)) => {}
                    None => {}
                }
                self.table.select(Some((cur + 1).min(len.saturating_sub(1))));
            }
            KeyCode::Char('a') => {
                let all: Vec<(usize, usize)> = order
                    .iter()
                    .flat_map(|&p| (0..entries[p].caches.len()).map(move |c| (p, c)))
                    .collect();
                if all.iter().all(|k| self.selected.contains(k)) {
                    self.selected.clear();
                } else {
                    self.selected.extend(all);
                }
            }
            KeyCode::Char('/') => self.searching = true,
            KeyCode::Esc if !self.query.is_empty() => self.query.clear(),
            KeyCode::Esc => self.selected.clear(),
            _ => return false,
        }
        true
    }
}

/// Column layout shared by the tree tables; `extra` is the view-specific column.
pub fn tree_columns(extra_header: &'static str) -> (Vec<&'static str>, Vec<Constraint>) {
    (
        vec!["", "Name", "Path", extra_header, "Size", "Caches"],
        vec![
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Length(17),
            Constraint::Length(10),
            Constraint::Length(6),
        ],
    )
}

/// Width left for the Path column in a table of `table_width`
/// (fixed columns, spacing, highlight symbol and borders subtracted).
pub fn path_width(table_width: u16) -> usize {
    let flexible = (table_width as usize).saturating_sub(3 + 17 + 10 + 6 + 5 + 2 + 2);
    flexible * 2 / 3
}

/// Keep the end of a long path, which is the part that identifies it.
fn ellipsize_start(s: String, max: usize) -> String {
    let n = s.chars().count();
    if n <= max || max < 4 {
        return s;
    }
    let tail: String = s.chars().skip(n - (max - 3)).collect();
    format!("...{}", tail)
}

fn right(s: String) -> Cell<'static> {
    Cell::from(Line::from(s).alignment(Alignment::Right))
}

/// Build table rows for `visible`; `extra` fills the view-specific column.
pub fn tree_rows(
    entries: &[Entry],
    visible: &[Node],
    list: &TreeList,
    path_width: usize,
    extra: impl Fn(&Entry) -> String,
) -> Vec<Row<'static>> {
    visible
        .iter()
        .map(|node| match *node {
            Node::Project(p) => {
                let e = &entries[p];
                let check = list.check(entries, p);
                let mark = match check {
                    Check::All => "[x]",
                    Check::Some => "[-]",
                    Check::None => "[ ]",
                };
                let fold = if e.caches.is_empty() {
                    "  "
                } else if list.expanded.contains(&p) {
                    "- "
                } else {
                    "+ "
                };
                let style = if check == Check::None { Style::default() } else { Theme::checked_row_style() };
                Row::new(vec![
                    Cell::from(mark),
                    Cell::from(format!("{}{}", fold, e.name)),
                    Cell::from(ellipsize_start(tilde(&e.path), path_width)),
                    Cell::from(extra(e)),
                    right(human_readable_size(e.total_size())),
                    right(e.caches.len().to_string()),
                ])
                .style(style)
            }
            Node::Cache(p, c) => {
                let e = &entries[p];
                let cache = &e.caches[c];
                let checked = list.selected.contains(&(p, c));
                let style = if checked { Theme::checked_row_style() } else { Theme::muted() };
                Row::new(vec![
                    Cell::from(if checked { "[x]" } else { "[ ]" }),
                    Cell::from(""),
                    Cell::from(ellipsize_start(format!("  {}", e.relative(&cache.path)), path_width)),
                    Cell::from(""),
                    right(human_readable_size(cache.size)),
                    Cell::from(""),
                ])
                .style(style)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::model::CacheEntry;

    fn entry(n: usize) -> Entry {
        Entry {
            id: None,
            name: "p".into(),
            path: "/p".into(),
            kind: String::new(),
            last_cleaned: None,
            caches: (0..n).map(|i| CacheEntry { path: format!("/p/c{}", i), size: 1 }).collect(),
        }
    }

    #[test]
    fn expand_and_select_project_then_cache() {
        let entries = vec![entry(2), entry(1)];
        let order = vec![0, 1];
        let mut t = TreeList::new();

        let v = t.visible(&entries, &order);
        assert_eq!(v.len(), 2);
        t.handle_key(KeyCode::Right, &entries, &v, &order);
        let v = t.visible(&entries, &order);
        assert_eq!(v, vec![Node::Project(0), Node::Cache(0, 0), Node::Cache(0, 1), Node::Project(1)]);

        // Space on a project selects all its caches.
        t.handle_key(KeyCode::Char(' '), &entries, &v, &order);
        assert_eq!(t.check(&entries, 0), Check::All);

        // Space on a cache row toggles just that cache.
        t.table.select(Some(1));
        t.handle_key(KeyCode::Char(' '), &entries, &v, &order);
        assert_eq!(t.check(&entries, 0), Check::Some);
        assert_eq!(t.targets(&entries, None).get(&0), Some(&vec![1]));
    }

    #[test]
    fn targets_fall_back_to_cursor() {
        let entries = vec![entry(3)];
        let t = TreeList::new();
        assert_eq!(t.targets(&entries, Some(Node::Project(0))).get(&0), Some(&vec![0, 1, 2]));
        assert_eq!(t.targets(&entries, Some(Node::Cache(0, 2))).get(&0), Some(&vec![2]));
    }

    #[test]
    fn left_on_cache_collapses_and_jumps_to_parent() {
        let entries = vec![entry(2)];
        let order = vec![0];
        let mut t = TreeList::new();
        t.expanded.insert(0);
        t.table.select(Some(2));
        let v = t.visible(&entries, &order);
        t.handle_key(KeyCode::Left, &entries, &v, &order);
        assert!(!t.expanded.contains(&0));
        assert_eq!(t.table.selected(), Some(0));
    }
}

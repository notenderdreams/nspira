use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    Projects,
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Size,
    LastCleaned,
    Id,
}

impl SortMode {
    pub fn next(&self) -> Self {
        match self {
            SortMode::Name => SortMode::Size,
            SortMode::Size => SortMode::LastCleaned,
            SortMode::LastCleaned => SortMode::Id,
            SortMode::Id => SortMode::Name,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Name => "Name (A-Z)",
            SortMode::Size => "Size (Largest)",
            SortMode::LastCleaned => "Last Cleaned",
            SortMode::Id => "ID",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

/// Rich application state for TUI views
#[derive(Debug, Clone)]
pub struct AppState {
    pub selected: usize,
    pub selected_items: HashSet<usize>,
    pub active_tab: ViewTab,
    pub sort_mode: SortMode,
    pub search_query: String,
    pub is_searching: bool,
    pub show_detail: bool,
    pub show_confirmation: bool,
    pub show_help: bool,
    pub status_message: String,
    pub status_type: StatusType,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            selected_items: HashSet::new(),
            active_tab: ViewTab::Projects,
            sort_mode: SortMode::Name,
            search_query: String::new(),
            is_searching: false,
            show_detail: false,
            show_confirmation: false,
            show_help: false,
            status_message: String::new(),
            status_type: StatusType::Info,
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, total_items: usize) {
        if total_items > 0 && self.selected + 1 < total_items {
            self.selected += 1;
        }
    }

    /// Jump to first item
    pub fn move_first(&mut self) {
        self.selected = 0;
    }

    /// Jump to last item
    pub fn move_last(&mut self, total_items: usize) {
        if total_items > 0 {
            self.selected = total_items - 1;
        }
    }

    /// Toggle selection of current item index
    pub fn toggle_select(&mut self) {
        if self.selected_items.contains(&self.selected) {
            self.selected_items.remove(&self.selected);
        } else {
            self.selected_items.insert(self.selected);
        }
    }

    /// Toggle selection of all items
    pub fn toggle_select_all(&mut self, total_items: usize) {
        if self.selected_items.len() == total_items {
            self.selected_items.clear();
        } else {
            self.selected_items = (0..total_items).collect();
        }
    }

    /// Invert selection of all items
    pub fn invert_selection(&mut self, total_items: usize) {
        let mut new_selected = HashSet::new();
        for i in 0..total_items {
            if !self.selected_items.contains(&i) {
                new_selected.insert(i);
            }
        }
        self.selected_items = new_selected;
    }

    /// Ensure selection bounds are valid after items list change
    pub fn clamp_selection(&mut self, total_items: usize) {
        if total_items == 0 {
            self.selected = 0;
            self.selected_items.clear();
        } else if self.selected >= total_items {
            self.selected = total_items - 1;
        }
        self.selected_items.retain(|&idx| idx < total_items);
    }

    /// Set status with type
    pub fn set_status(&mut self, msg: impl Into<String>, status_type: StatusType) {
        self.status_message = msg.into();
        self.status_type = status_type;
    }

    /// Clear status
    pub fn clear_status(&mut self) {
        self.status_message.clear();
        self.status_type = StatusType::Info;
    }

    /// Check if index is selected
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_items.contains(&index)
    }

    /// Get count of selected items
    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_selection_navigation() {
        let mut state = AppState::new();
        assert_eq!(state.selected, 0);

        state.move_down(5);
        assert_eq!(state.selected, 1);

        state.move_last(5);
        assert_eq!(state.selected, 4);

        state.move_down(5); // should not exceed 4
        assert_eq!(state.selected, 4);

        state.move_up();
        assert_eq!(state.selected, 3);

        state.move_first();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_toggle_select_and_all() {
        let mut state = AppState::new();
        state.toggle_select();
        assert!(state.is_selected(0));

        state.toggle_select();
        assert!(!state.is_selected(0));

        state.toggle_select_all(3);
        assert_eq!(state.selected_count(), 3);

        state.toggle_select_all(3);
        assert_eq!(state.selected_count(), 0);

        state.invert_selection(3);
        assert_eq!(state.selected_count(), 3);
    }

    #[test]
    fn test_sort_mode_cycle() {
        let mode = SortMode::Name;
        assert_eq!(mode.next(), SortMode::Size);
        assert_eq!(mode.next().next(), SortMode::LastCleaned);
        assert_eq!(mode.next().next().next(), SortMode::Id);
        assert_eq!(mode.next().next().next().next(), SortMode::Name);
    }
}

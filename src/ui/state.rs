/// Common application state for TUI apps
#[derive(Debug, Clone)]
pub struct AppState {
    pub selected: usize,
    pub selected_items: Vec<usize>,
    pub exit: bool,
    pub status_message: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            selected_items: Vec::new(),
            exit: false,
            status_message: String::new(),
        }
    }

    /// Toggle selection of current item
    pub fn toggle_select(&mut self) {
        if let Some(pos) = self.selected_items.iter().position(|&i| i == self.selected) {
            self.selected_items.remove(pos);
        } else {
            self.selected_items.push(self.selected);
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

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, max_items: usize) {
        if self.selected + 1 < max_items {
            self.selected += 1;
        }
    }

    /// Set status message
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message.clear();
    }

    /// Check if item is selected
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_items.contains(&index)
    }

    /// Get selected count
    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

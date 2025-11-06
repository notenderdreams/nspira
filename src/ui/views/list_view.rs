use crate::ui::{
    components::{confirmation_popup, create_list_help, create_row_with_selection, TableConfig, TableWidget},
    state::AppState,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
    Frame,
};

/// Generic list view that can be used for different data types
pub struct ListView<T> {
    pub state: AppState,
    pub items: Vec<T>,
    pub table_config: TableConfig,
    pub show_confirmation: bool,
}

impl<T> ListView<T> {
    pub fn new(items: Vec<T>, table_config: TableConfig) -> Self {
        Self {
            state: AppState::new(),
            items,
            table_config,
            show_confirmation: false,
        }
    }

    pub fn render<F>(&self, f: &mut Frame, row_mapper: F)
    where
        F: Fn(&T, usize, bool) -> Vec<String>,
    {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(size);

        // Create table rows
        let rows = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = self.state.is_selected(i);
                let cells = row_mapper(item, i, is_selected);
                create_row_with_selection(cells, is_selected, false)
            })
            .collect();

        // Render table
        let table = TableWidget::new(self.table_config.clone()).rows(rows).build();

        let mut table_state = TableState::default().with_selected(Some(self.state.selected));
        f.render_stateful_widget(table, chunks[0], &mut table_state);

        // Render help panel
        let help = create_list_help(
            self.state.selected_count(),
            self.items.len(),
            &self.state.status_message,
        );
        help.render(f, chunks[1]);

        // Render confirmation popup if needed
        if self.show_confirmation {
            let popup = confirmation_popup(
                "Confirm Action",
                "Are you sure you want to proceed?",
                if self.state.selected_count() > 0 {
                    Some(self.state.selected_count())
                } else {
                    Some(1)
                },
            );
            popup.render(f);
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> ListAction {
        use crossterm::event::KeyCode;

        if self.show_confirmation {
            match key {
                KeyCode::Char('y') | KeyCode::Char('d') | KeyCode::Char('Y') | KeyCode::Char('D') => {
                    self.show_confirmation = false;
                    return ListAction::ConfirmAction;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_confirmation = false;
                    self.state.set_status("Action cancelled");
                    return ListAction::None;
                }
                _ => return ListAction::None,
            }
        }

        match key {
            KeyCode::Char('q') => ListAction::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.move_down(self.items.len());
                ListAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.move_up();
                ListAction::None
            }
            KeyCode::Char(' ') => {
                self.state.toggle_select();
                ListAction::None
            }
            KeyCode::Char('a') => {
                self.state.toggle_select_all(self.items.len());
                ListAction::None
            }
            KeyCode::Enter => {
                if self.state.selected_count() == 0 && !self.items.is_empty() {
                    self.state.set_status("No items selected");
                    ListAction::None
                } else {
                    ListAction::ExecuteAction
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if self.items.is_empty() {
                    self.state.set_status("No items to delete");
                    ListAction::None
                } else {
                    self.show_confirmation = true;
                    ListAction::None
                }
            }
            _ => ListAction::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListAction {
    None,
    Quit,
    ExecuteAction,
    ConfirmAction,
}
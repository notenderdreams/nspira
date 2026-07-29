use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState},
};

#[derive(Debug, Clone)]
pub struct TableConfig {
    pub title: String,
    pub headers: Vec<String>,
    pub constraints: Vec<Constraint>,
    pub highlight_symbol: String,
}

impl TableConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            headers: Vec::new(),
            constraints: Vec::new(),
            highlight_symbol: "❯ ".to_string(),
        }
    }

    pub fn headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.headers = headers.into_iter().map(|h| h.into()).collect();
        self
    }

    pub fn constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn highlight_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.highlight_symbol = symbol.into();
        self
    }
}

pub struct TableWidget<'a> {
    config: TableConfig,
    rows: Vec<Row<'a>>,
    total_items: usize,
}

impl<'a> TableWidget<'a> {
    pub fn new(config: TableConfig) -> Self {
        Self {
            config,
            rows: Vec::new(),
            total_items: 0,
        }
    }

    pub fn rows(mut self, rows: Vec<Row<'a>>) -> Self {
        self.total_items = rows.len();
        self.rows = rows;
        self
    }

    pub fn render(
        self,
        f: &mut Frame,
        area: Rect,
        state: &mut TableState,
    ) {
        let header = Row::new(
            self.config
                .headers
                .iter()
                .map(|h| Cell::from(h.clone()).style(Theme::header_style()))
                .collect::<Vec<_>>(),
        )
        .style(Style::default().bg(Theme::BG_HEADER))
        .height(1)
        .bottom_margin(1);

        let table = Table::new(self.rows, self.config.constraints)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(Theme::BORDER_TYPE)
                    .border_style(Style::default().fg(Theme::BORDER))
                    .title(format!(" {} ", self.config.title))
                    .title_style(Theme::title_style()),
            )
            .highlight_style(Theme::selected_row_style())
            .highlight_symbol(self.config.highlight_symbol.clone());

        f.render_stateful_widget(table, area, state);

        // Render scrollbar if total items exceed visible viewport
        if self.total_items > 0 {
            let mut scrollbar_state = ScrollbarState::new(self.total_items)
                .position(state.selected().unwrap_or(0));
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼")),
                area,
                &mut scrollbar_state,
            );
        }
    }
}

/// Utility for building formatted table rows
pub fn build_table_row<'a>(
    cells: Vec<String>,
    is_checked: bool,
    is_cursor: bool,
) -> Row<'a> {
    let base_style = if is_cursor {
        Theme::selected_row_style()
    } else if is_checked {
        Theme::checked_row_style()
    } else {
        Style::default().fg(Theme::TEXT)
    };

    let cells: Vec<Cell> = cells
        .into_iter()
        .map(|content| Cell::from(content).style(base_style))
        .collect();

    Row::new(cells)
}

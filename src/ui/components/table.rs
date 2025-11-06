use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

/// Configuration for a table widget
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
            highlight_symbol: ">> ".to_string(),
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

/// Reusable table widget
pub struct TableWidget {
    config: TableConfig,
    rows: Vec<Row<'static>>,
}

impl TableWidget {
    pub fn new(config: TableConfig) -> Self {
        Self {
            config,
            rows: Vec::new(),
        }
    }

    pub fn rows(mut self, rows: Vec<Row<'static>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn build(self) -> Table<'static> {
        let header = Row::new(
            self.config
                .headers
                .iter()
                .map(|h| Cell::from(h.clone()))
                .collect::<Vec<_>>(),
        )
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

        let highlight_symbol = self.config.highlight_symbol.clone();

        Table::new(self.rows, self.config.constraints)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.config.title),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol(highlight_symbol)
    }
}

/// Helper to create a row with selection styling
pub fn create_row_with_selection(
    cells: Vec<String>,
    is_selected: bool,
    is_highlighted: bool,
) -> Row<'static> {
    let style = if is_highlighted {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else if is_selected {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let cells: Vec<Cell> = cells
        .into_iter()
        .map(|cell| Cell::from(cell).style(style))
        .collect();

    Row::new(cells)
}
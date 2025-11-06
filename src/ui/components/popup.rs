use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Configuration for popup widgets
#[derive(Debug, Clone)]
pub struct PopupConfig {
    pub title: String,
    pub width: u16,
    pub height: u16,
    pub border_color: Color,
}

impl PopupConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 60,
            height: 9,
            border_color: Color::Yellow,
        }
    }

    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
}

/// Reusable popup widget
pub struct PopupWidget {
    config: PopupConfig,
    content: Vec<Line<'static>>,
}

impl PopupWidget {
    pub fn new(config: PopupConfig) -> Self {
        Self {
            config,
            content: Vec::new(),
        }
    }

    pub fn content(mut self, lines: Vec<Line<'static>>) -> Self {
        self.content = lines;
        self
    }

    pub fn render(self, f: &mut Frame) {
        let area = f.size();
        let popup_area = self.centered_rect(area);

        f.render_widget(Clear, popup_area);

        let popup = Paragraph::new(self.content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.config.border_color))
                    .title(format!(" {} ", self.config.title)),
            )
            .alignment(Alignment::Center);

        f.render_widget(popup, popup_area);
    }

    fn centered_rect(&self, area: Rect) -> Rect {
        let popup_x = (area.width.saturating_sub(self.config.width)) / 2;
        let popup_y = (area.height.saturating_sub(self.config.height)) / 2;

        Rect {
            x: popup_x,
            y: popup_y,
            width: self.config.width,
            height: self.config.height,
        }
    }
}

/// Create a confirmation popup
pub fn confirmation_popup(
    title: impl Into<String>,
    message: impl Into<String>,
    count: Option<usize>,
) -> PopupWidget {
    let config = PopupConfig::new(title).border_color(Color::Red);

    let mut lines = vec![
        Line::raw(""),
        Line::styled(
            message.into(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
    ];

    if let Some(count) = count {
        lines.push(Line::from(vec![
            Span::raw("Selected: "),
            Span::styled(
                format!("{} item(s)", count),
                Style::default().fg(Color::Cyan),
            ),
        ]));
        lines.push(Line::raw(""));
    }

    lines.extend(vec![Line::styled(
        "This action cannot be undone.",
        Style::default().fg(Color::Gray),
    )]);

    PopupWidget::new(config).content(lines)
}

/// Create an info popup
pub fn info_popup(title: impl Into<String>, lines: Vec<Line<'static>>) -> PopupWidget {
    let config = PopupConfig::new(title).border_color(Color::Blue);
    PopupWidget::new(config).content(lines)
}

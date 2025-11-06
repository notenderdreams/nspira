use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
};

use super::PopupConfig;

/// Progress popup widget
pub struct ProgressPopup {
    config: PopupConfig,
    progress: f64,
    current_item: String,
    completed: usize,
    total: usize,
}

impl ProgressPopup {
    pub fn new(title: impl Into<String>, total: usize) -> Self {
        let config = PopupConfig::new(title)
            .size(60, 12)
            .border_color(Color::Blue);

        Self {
            config,
            progress: 0.0,
            current_item: String::new(),
            completed: 0,
            total,
        }
    }

    pub fn update(&mut self, current_item: impl Into<String>, completed: usize) {
        self.current_item = current_item.into();
        self.completed = completed;
        self.progress = if self.total > 0 {
            (completed as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };
    }

    pub fn render(&self, f: &mut Frame) {
        let area = f.size();
        let popup_area = self.centered_rect(area);

        f.render_widget(Clear, popup_area);

        let progress_text = vec![
            Line::raw(""),
            Line::styled(
                "Processing...",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::from(vec![
                Span::raw("Progress: "),
                Span::styled(
                    format!("{}/{}", self.completed, self.total),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("Current: "),
                Span::styled(&self.current_item, Style::default().fg(Color::Yellow)),
            ]),
            Line::raw(""),
        ];

        let progress_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(format!(" {} ", self.config.title));

        let progress_paragraph = Paragraph::new(progress_text)
            .block(progress_block)
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(progress_paragraph, popup_area);

        // Render gauge in the bottom part of the popup
        let gauge_area = Rect {
            x: popup_area.x + 2,
            y: popup_area.y + popup_area.height - 4,
            width: popup_area.width - 4,
            height: 3,
        };

        let progress_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .percent(self.progress as u16)
            .label(format!("{:.1}%", self.progress));

        f.render_widget(progress_gauge, gauge_area);
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

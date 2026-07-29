use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
};

pub struct ProgressPopup {
    title: String,
    progress: f64,
    current_item: String,
    completed: usize,
    total: usize,
    freed_bytes: u64,
}

impl ProgressPopup {
    pub fn new(title: impl Into<String>, total: usize) -> Self {
        Self {
            title: title.into(),
            progress: 0.0,
            current_item: String::new(),
            completed: 0,
            total,
            freed_bytes: 0,
        }
    }

    pub fn update(&mut self, current_item: impl Into<String>, completed: usize, freed_bytes: u64) {
        self.current_item = current_item.into();
        self.completed = completed;
        self.freed_bytes = freed_bytes;
        self.progress = if self.total > 0 {
            (completed as f64 / self.total as f64) * 100.0
        } else {
            100.0
        };
    }

    pub fn render(&self, f: &mut Frame) {
        let area = f.size();
        let popup_area = self.centered_rect(area);

        // 1. Render backdrop shadow
        let backdrop = Block::default().style(Style::default().bg(Color::Rgb(15, 23, 42)));
        f.render_widget(backdrop, popup_area);
        f.render_widget(Clear, popup_area);

        let freed_str = crate::utils::human_readable_size(self.freed_bytes);

        let max_target_len = (popup_area.width.saturating_sub(22)) as usize;
        let display_target = if self.current_item.len() > max_target_len && max_target_len > 5 {
            format!("...{}", &self.current_item[self.current_item.len() - (max_target_len - 3)..])
        } else {
            self.current_item.clone()
        };

        let text = vec![
            Line::raw(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Executing Cache Cleanup Operation...",
                    Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Completed      : "),
                Span::styled(
                    format!("{}/{} ({:.0}%)", self.completed, self.total, self.progress),
                    Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   │   Freed: "),
                Span::styled(freed_str, Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("  Current Target : "),
                Span::styled(display_target, Style::default().fg(Theme::WARNING)),
            ]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(Theme::PRIMARY))
            .title(format!(" {} ", self.title))
            .title_style(Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD));

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, popup_area);

        // Render progress gauge bar at bottom of popup
        let gauge_area = Rect {
            x: popup_area.x + 3,
            y: popup_area.y + popup_area.height - 2,
            width: popup_area.width.saturating_sub(6),
            height: 1,
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Theme::SUCCESS).bg(Color::Rgb(30, 41, 59)))
            .percent(self.progress.clamp(0.0, 100.0) as u16)
            .label(format!("{:.1}%", self.progress));

        f.render_widget(gauge, gauge_area);
    }

    fn centered_rect(&self, area: Rect) -> Rect {
        let width = (area.width * 60 / 100).max(52).min(area.width);
        let height = 11u16.min(area.height);

        let popup_x = (area.width.saturating_sub(width)) / 2;
        let popup_y = (area.height.saturating_sub(height)) / 2;

        Rect {
            x: popup_x,
            y: popup_y,
            width,
            height,
        }
    }
}

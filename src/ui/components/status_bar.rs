use crate::ui::state::StatusType;
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct KeyHint {
    pub key: String,
    pub desc: String,
}

impl KeyHint {
    pub fn new(key: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            desc: desc.into(),
        }
    }
}

pub struct StatusBarWidget<'a> {
    hints: Vec<KeyHint>,
    status_msg: &'a str,
    status_type: &'a StatusType,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(status_msg: &'a str, status_type: &'a StatusType) -> Self {
        Self {
            hints: Vec::new(),
            status_msg,
            status_type,
        }
    }

    pub fn hint(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.hints.push(KeyHint::new(key, desc));
        self
    }

    pub fn render(self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(Theme::BORDER));

        let mut line_spans = Vec::new();

        if !self.status_msg.is_empty() {
            let (prefix, color) = match self.status_type {
                StatusType::Success => ("✓ ", Theme::SUCCESS),
                StatusType::Error => ("✖ ", Theme::DANGER),
                StatusType::Warning => ("⚠ ", Theme::WARNING),
                StatusType::Info => ("ℹ ", Theme::INFO),
            };

            line_spans.push(Span::styled(
                format!(" {} ", prefix),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            line_spans.push(Span::styled(
                format!("{} ", self.status_msg),
                Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD),
            ));
            line_spans.push(Span::raw(" │ "));
        }

        for (i, hint) in self.hints.iter().enumerate() {
            if i > 0 {
                line_spans.push(Span::raw("  "));
            }
            line_spans.push(Span::styled(
                format!("[{}]", hint.key),
                Theme::help_key_style(),
            ));
            line_spans.push(Span::raw(" "));
            line_spans.push(Span::styled(
                &hint.desc,
                Theme::help_desc_style(),
            ));
        }

        let paragraph = Paragraph::new(Line::from(line_spans))
            .block(block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, area);
    }
}

/// Helper for generating hotkey bar for project manager
pub fn create_project_hints<'a>(status: &'a str, status_type: &'a StatusType) -> StatusBarWidget<'a> {
    StatusBarWidget::new(status, status_type)
        .hint("↑/↓", "Nav")
        .hint("Space", "Select")
        .hint("a", "All")
        .hint("c/Enter", "Clean")
        .hint("d", "Delete")
        .hint("/", "Filter")
        .hint("s", "Sort")
        .hint("i", "Details")
        .hint("?", "Help")
        .hint("q", "Quit")
}

/// Helper for scan view hotkeys
pub fn create_scan_hints<'a>(status: &'a str, status_type: &'a StatusType) -> StatusBarWidget<'a> {
    StatusBarWidget::new(status, status_type)
        .hint("↑/↓", "Nav")
        .hint("Space", "Select")
        .hint("a", "All")
        .hint("Enter", "Add Tracked")
        .hint("/", "Filter")
        .hint("?", "Help")
        .hint("q", "Quit")
}

/// Helper for doctor view hotkeys
pub fn create_doctor_hints<'a>(status: &'a str, status_type: &'a StatusType) -> StatusBarWidget<'a> {
    StatusBarWidget::new(status, status_type)
        .hint("↑/↓", "Nav")
        .hint("d", "Remove Broken")
        .hint("f", "Filter Issues")
        .hint("?", "Help")
        .hint("q", "Quit")
}

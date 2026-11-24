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

        // Width available inside borders; everything is trimmed to fit on one line.
        let mut remaining = area.width.saturating_sub(2) as usize;
        let mut line_spans = Vec::new();

        if !self.status_msg.is_empty() {
            let (tag, color) = match self.status_type {
                StatusType::Success => ("OK", Theme::SUCCESS),
                StatusType::Error => ("ERR", Theme::DANGER),
                StatusType::Warning => ("WARN", Theme::WARNING),
                StatusType::Info => ("INFO", Theme::INFO),
            };
            let tag_txt = format!(" {} ", tag);
            remaining = remaining.saturating_sub(tag_txt.chars().count() + 1);
            line_spans.push(Span::styled(
                tag_txt,
                Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ));

            // Reserve room for at least the first hint; truncate long messages.
            let msg_budget = remaining.saturating_sub(12).max(10);
            let msg = truncate(self.status_msg, msg_budget);
            remaining = remaining.saturating_sub(msg.chars().count() + 2);
            line_spans.push(Span::styled(
                format!(" {}  ", msg),
                Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD),
            ));
        }

        // Add hints until the line is full; the rest is reachable via `?`.
        for hint in &self.hints {
            let w = hint.key.chars().count() + hint.desc.chars().count() + 4;
            if w > remaining {
                break;
            }
            remaining -= w;
            line_spans.push(Span::styled(format!("{} ", hint.key), Theme::help_key_style()));
            line_spans.push(Span::styled(format!("{}  ", hint.desc), Theme::help_desc_style()));
        }

        f.render_widget(
            Paragraph::new(Line::from(line_spans)).block(block).alignment(Alignment::Left),
            area,
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", t)
    }
}

/// Hotkey bar for the tracked-projects view
pub fn create_project_hints<'a>(status: &'a str, status_type: &'a StatusType) -> StatusBarWidget<'a> {
    StatusBarWidget::new(status, status_type)
        .hint("Space", "Select")
        .hint("l/h", "Expand")
        .hint("c", "Clean")
        .hint("d", "Untrack")
        .hint("/", "Filter")
        .hint("s", "Sort")
        .hint("Tab", "Stats")
        .hint("?", "Help")
        .hint("q", "Quit")
}

/// Hotkey bar for the scan results view
pub fn create_scan_hints<'a>(status: &'a str, status_type: &'a StatusType) -> StatusBarWidget<'a> {
    StatusBarWidget::new(status, status_type)
        .hint("Space", "Select")
        .hint("l/h", "Expand")
        .hint("Enter", "Track")
        .hint("x", "Ignore")
        .hint("/", "Filter")
        .hint("?", "Help")
        .hint("q", "Quit")
}

/// Helper for doctor view hotkeys
pub fn create_doctor_hints<'a>(status: &'a str, status_type: &'a StatusType) -> StatusBarWidget<'a> {
    StatusBarWidget::new(status, status_type)
        .hint("Up/Dn", "Nav")
        .hint("d", "Remove Broken")
        .hint("f", "Filter Issues")
        .hint("?", "Help")
        .hint("q", "Quit")
}

use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub struct PopupWidget {
    title: String,
    width_percent: u16,
    min_width: u16,
    height: Option<u16>,
    border_color: Color,
    alignment: Alignment,
    content: Vec<Line<'static>>,
}

impl PopupWidget {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width_percent: 70,
            min_width: 50,
            height: None,
            border_color: Theme::PRIMARY,
            alignment: Alignment::Left,
            content: Vec::new(),
        }
    }

    pub fn size(mut self, width_percent: u16, height: u16) -> Self {
        self.width_percent = width_percent;
        self.height = Some(height);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn content(mut self, lines: Vec<Line<'static>>) -> Self {
        self.content = lines;
        self
    }

    pub fn render(self, f: &mut Frame) {
        let area = f.size();
        let popup_area = self.calculate_rect(area);

        // 1. Render backdrop shadow / dimming box over screen
        let backdrop_block = Block::default().style(Style::default().bg(Color::Rgb(15, 23, 42)));
        f.render_widget(backdrop_block, popup_area);
        f.render_widget(Clear, popup_area);

        // 2. Render main dialog popup paragraph
        let popup = Paragraph::new(self.content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(Theme::BORDER_TYPE)
                    .border_style(Style::default().fg(self.border_color))
                    .title(format!(" {} ", self.title))
                    .title_style(Style::default().fg(self.border_color).add_modifier(Modifier::BOLD)),
            )
            .alignment(self.alignment);

        f.render_widget(popup, popup_area);
    }

    fn calculate_rect(&self, area: Rect) -> Rect {
        // Calculate max line length to size dialog dynamically
        let max_content_width = self
            .content
            .iter()
            .map(|l| l.width())
            .max()
            .unwrap_or(40) as u16;

        let desired_width = (max_content_width + 6)
            .max(self.min_width)
            .max(area.width * self.width_percent / 100);
        let width = desired_width.min(area.width.saturating_sub(4));

        let computed_height = self
            .height
            .unwrap_or_else(|| (self.content.len() as u16 + 3).max(7));
        let height = computed_height.min(area.height.saturating_sub(2));

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

/// Confirmation popup modal factory
pub fn confirmation_popup(
    title: impl Into<String>,
    action_prompt: impl Into<String>,
    item_count: usize,
    warning: Option<&str>,
) -> PopupWidget {
    let mut lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                action_prompt.into(),
                Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  Selected Target(s): "),
            Span::styled(
                format!("{} project(s)", item_count),
                Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    if let Some(warn) = warning {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  ⚠️  ", Style::default().fg(Theme::DANGER)),
            Span::styled(
                warn.to_string(),
                Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[y / Enter]",
            Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Confirm   ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[n / Esc]",
            Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Cancel", Style::default().fg(Theme::TEXT)),
    ]));

    let height = lines.len() as u16 + 2;

    PopupWidget::new(title)
        .border_color(Theme::WARNING)
        .size(60, height)
        .alignment(Alignment::Left)
        .content(lines)
}

/// Project detail modal showing cache directories and sizes
pub fn project_detail_popup(
    project_name: &str,
    project_path: &str,
    cache_dirs: &[(String, u64)],
    total_size_str: &str,
) -> PopupWidget {
    let mut lines = vec![
        Line::from(vec![
            Span::raw("  Project Name : "),
            Span::styled(project_name.to_string(), Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("  Project Path : "),
            Span::styled(project_path.to_string(), Style::default().fg(Theme::TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::raw("  Total Cache  : "),
            Span::styled(total_size_str.to_string(), Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        ]),
        Line::raw("  ──────────────────────── Cache Directories ────────────────────────"),
    ];

    if cache_dirs.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("No active cache directories registered.", Style::default().fg(Theme::TEXT_MUTED)),
        ]));
    } else {
        for (dir, size) in cache_dirs {
            lines.push(Line::from(vec![
                Span::styled("   • ", Style::default().fg(Theme::ACCENT)),
                Span::styled(dir.clone(), Style::default().fg(Theme::TEXT)),
                Span::raw(" ("),
                Span::styled(
                    crate::utils::human_readable_size(*size),
                    Style::default().fg(Theme::SUCCESS),
                ),
                Span::raw(")"),
            ]));
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "Press [Esc / q / Enter] to close details",
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));

    let height = (lines.len() as u16 + 2).clamp(10, 24);

    PopupWidget::new(format!("Project Details: {}", project_name))
        .border_color(Theme::PRIMARY)
        .size(68, height)
        .alignment(Alignment::Left)
        .content(lines)
}

/// Live search input popup modal
pub fn search_input_popup(current_query: &str) -> PopupWidget {
    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("🔍 Search Filter: ", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD)),
            Span::styled(current_query.to_string(), Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Theme::PRIMARY)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Type characters to filter  •  ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("[Enter]", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(" Apply  •  ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("[Esc]", Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD)),
            Span::styled(" Clear", Style::default().fg(Theme::TEXT_MUTED)),
        ]),
    ];

    PopupWidget::new("Filter Projects")
        .border_color(Theme::WARNING)
        .size(55, 7)
        .alignment(Alignment::Left)
        .content(lines)
}

/// Quick help modal cheat sheet
pub fn help_popup() -> PopupWidget {
    let lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled("NAVIGATION & SELECTION", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   ↑ / k / Down / j ", Style::default().fg(Theme::WARNING)),
            Span::raw(" Move cursor up / down"),
        ]),
        Line::from(vec![
            Span::styled("   Home / g / End / G ", Style::default().fg(Theme::WARNING)),
            Span::raw(" Jump to top / bottom"),
        ]),
        Line::from(vec![
            Span::styled("   Space            ", Style::default().fg(Theme::WARNING)),
            Span::raw(" Toggle select item"),
        ]),
        Line::from(vec![
            Span::styled("   a / v            ", Style::default().fg(Theme::WARNING)),
            Span::raw(" Select all / Invert selection"),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("ACTIONS & VIEWS", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   c / Enter        ", Style::default().fg(Theme::SUCCESS)),
            Span::raw(" Clean cache of selected project(s)"),
        ]),
        Line::from(vec![
            Span::styled("   d / Delete       ", Style::default().fg(Theme::DANGER)),
            Span::raw(" Remove project(s) from tracking"),
        ]),
        Line::from(vec![
            Span::styled("   /                ", Style::default().fg(Theme::ACCENT)),
            Span::raw(" Live search filter"),
        ]),
        Line::from(vec![
            Span::styled("   s                ", Style::default().fg(Theme::SECONDARY)),
            Span::raw(" Cycle sort mode (Name, Size, Cleaned, ID)"),
        ]),
        Line::from(vec![
            Span::styled("   i                ", Style::default().fg(Theme::PRIMARY)),
            Span::raw(" Inspect project cache details"),
        ]),
        Line::from(vec![
            Span::styled("   Tab / 1 / 2      ", Style::default().fg(Theme::TEXT)),
            Span::raw(" Switch between Projects and Stats view"),
        ]),
        Line::from(vec![
            Span::styled("   ? / h            ", Style::default().fg(Theme::WARNING)),
            Span::raw(" Toggle this help cheat sheet"),
        ]),
        Line::from(vec![
            Span::styled("   q / Esc          ", Style::default().fg(Theme::DANGER)),
            Span::raw(" Quit / Dismiss modal"),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Press [Esc / q / ?] to dismiss help", Style::default().fg(Theme::TEXT_MUTED)),
        ]),
    ];

    let height = lines.len() as u16 + 2;

    PopupWidget::new("Keyboard Shortcuts Cheat Sheet")
        .border_color(Theme::PRIMARY)
        .size(65, height)
        .alignment(Alignment::Left)
        .content(lines)
}

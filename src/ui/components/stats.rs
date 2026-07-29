use crate::ui::model::UiProjectItem;
use crate::ui::theme::Theme;
use crate::utils::human_readable_size;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;

pub struct StatsWidget<'a> {
    projects: &'a [UiProjectItem],
}

impl<'a> StatsWidget<'a> {
    pub fn new(projects: &'a [UiProjectItem]) -> Self {
        Self { projects }
    }

    pub fn render(self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut total_bytes: u64 = 0;
        let mut type_breakdown: HashMap<String, (usize, u64)> = HashMap::new();
        let mut largest_project: Option<(&UiProjectItem, u64)> = None;

        for proj in self.projects {
            let proj_bytes: u64 = proj.total_size;
            total_bytes += proj_bytes;

            // Detect ecosystem type
            let proj_type = if proj.path.contains("node") || proj.cache_dirs.iter().any(|d| d.path.contains("node_modules")) {
                "Node.js"
            } else if proj.cache_dirs.iter().any(|d| d.path.contains("target")) {
                "Rust / Java"
            } else if proj.cache_dirs.iter().any(|d| d.path.contains("build") || d.path.contains(".gradle")) {
                "Gradle / Mobile"
            } else if proj.cache_dirs.iter().any(|d| d.path.contains("__pycache__") || d.path.contains("venv")) {
                "Python"
            } else {
                "Custom / Other"
            };

            let entry = type_breakdown.entry(proj_type.to_string()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += proj_bytes;

            if largest_project.is_none() || proj_bytes > largest_project.unwrap().1 {
                largest_project = Some((proj, proj_bytes));
            }
        }

        // Left Panel: Overview Stats
        let mut left_lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::raw(" Total Tracked Projects:  "),
                Span::styled(
                    self.projects.len().to_string(),
                    Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw(" Total Occupied Cache:    "),
                Span::styled(
                    human_readable_size(total_bytes),
                    Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::raw(""),
        ];

        if let Some((lp, lp_size)) = largest_project {
            left_lines.push(Line::from(vec![
                Span::raw(" Largest Cache Project:   "),
                Span::styled(&lp.name, Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::raw(" ("),
                Span::styled(human_readable_size(lp_size), Style::default().fg(Theme::SUCCESS)),
                Span::raw(")"),
            ]));
        }

        let left_block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(" Overview Metrics ")
            .title_style(Theme::title_style());

        f.render_widget(Paragraph::new(left_lines).block(left_block), chunks[0]);

        // Right Panel: Ecosystem Breakdown
        let mut right_lines = vec![Line::raw("")];

        for (ptype, (count, bytes)) in &type_breakdown {
            let pct = if total_bytes > 0 {
                (*bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            right_lines.push(Line::from(vec![
                Span::styled(format!(" • {:<15}", ptype), Style::default().fg(Theme::SECONDARY)),
                Span::raw(format!(" {:>2} proj  │ ", count)),
                Span::styled(
                    format!("{:<9}", human_readable_size(*bytes)),
                    Style::default().fg(Theme::SUCCESS),
                ),
                Span::styled(
                    format!(" ({:.1}%)", pct),
                    Style::default().fg(Theme::TEXT_MUTED),
                ),
            ]));
        }

        let right_block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(" Breakdown by Ecosystem ")
            .title_style(Theme::title_style());

        f.render_widget(Paragraph::new(right_lines).block(right_block), chunks[1]);
    }
}

use crate::ui::state::ViewTab;
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct HeaderWidget<'a> {
    title: &'a str,
    subtitle: &'a str,
    metrics: Vec<(&'a str, String)>,
    active_tab: ViewTab,
    search_query: Option<&'a str>,
}

impl<'a> HeaderWidget<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: "Cache Manager",
            metrics: Vec::new(),
            active_tab: ViewTab::Projects,
            search_query: None,
        }
    }

    pub fn metric(mut self, label: &'a str, value: impl Into<String>) -> Self {
        self.metrics.push((label, value.into()));
        self
    }

    pub fn active_tab(mut self, tab: ViewTab) -> Self {
        self.active_tab = tab;
        self
    }

    pub fn search_query(mut self, query: Option<&'a str>) -> Self {
        self.search_query = query;
        self
    }

    pub fn render(self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(Theme::BORDER));

        let mut spans = vec![
            Span::styled(" 🌿 ", Style::default().fg(Theme::SUCCESS)),
            Span::styled(
                self.title,
                Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" ({}) ", self.subtitle), Style::default().fg(Theme::TEXT_MUTED)),
            Span::raw(" │ "),
        ];

        // Add metrics
        for (i, (label, val)) in self.metrics.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("   "));
            }
            spans.push(Span::styled(format!("{}: ", label), Style::default().fg(Theme::TEXT_MUTED)));
            spans.push(Span::styled(
                val.to_string(),
                Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD),
            ));
        }

        // Add tabs
        spans.push(Span::raw(" │ "));
        let p_style = if self.active_tab == ViewTab::Projects {
            Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Theme::TEXT_MUTED)
        };
        let s_style = if self.active_tab == ViewTab::Stats {
            Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Theme::TEXT_MUTED)
        };

        spans.push(Span::styled("[1: Projects]", p_style));
        spans.push(Span::raw(" "));
        spans.push(Span::styled("[2: Stats]", s_style));

        // Add search query if searching
        if let Some(query) = self.search_query {
            spans.push(Span::raw(" │ "));
            spans.push(Span::styled("🔍 ", Style::default().fg(Theme::WARNING)));
            spans.push(Span::styled(
                format!("Filter: \"{}\"", query),
                Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
            ));
        }

        let paragraph = Paragraph::new(Line::from(spans))
            .block(block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, area);
    }
}

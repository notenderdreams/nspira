use crate::ui::state::ViewTab;
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, block::Title},
};

pub struct HeaderWidget<'a> {
    title: &'a str,
    metrics: Vec<(&'a str, String)>,
    active_tab: Option<ViewTab>,
    search_query: Option<&'a str>,
}

impl<'a> HeaderWidget<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            metrics: Vec::new(),
            active_tab: None,
            search_query: None,
        }
    }

    pub fn metric(mut self, label: &'a str, value: impl Into<String>) -> Self {
        self.metrics.push((label, value.into()));
        self
    }

    /// Show the Projects/Stats tab switcher (project list view only).
    pub fn active_tab(mut self, tab: ViewTab) -> Self {
        self.active_tab = Some(tab);
        self
    }

    pub fn search_query(mut self, query: Option<&'a str>) -> Self {
        self.search_query = query;
        self
    }

    pub fn render(self, f: &mut Frame, area: Rect) {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Title::from(Span::styled(
                format!(" nspira / {} ", self.title),
                Theme::title_style(),
            )));

        // Tabs live in the border so they never compete with metrics for width.
        if let Some(tab) = self.active_tab {
            let style = |active: bool| {
                if active {
                    Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(Theme::TEXT_MUTED)
                }
            };
            block = block.title(
                Title::from(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(" 1 Projects ", style(tab == ViewTab::Projects)),
                    Span::raw(" "),
                    Span::styled(" 2 Stats ", style(tab == ViewTab::Stats)),
                    Span::raw(" "),
                ]))
                .alignment(Alignment::Right),
            );
        }

        let mut spans: Vec<Span> = Vec::new();
        for (i, (label, val)) in self.metrics.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  |  ", Style::default().fg(Theme::BORDER)));
            }
            spans.push(Span::styled(format!("{}: ", label), Style::default().fg(Theme::TEXT_MUTED)));
            spans.push(Span::styled(
                val.clone(),
                Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD),
            ));
        }

        if let Some(query) = self.search_query {
            spans.push(Span::styled("  |  ", Style::default().fg(Theme::BORDER)));
            spans.push(Span::styled(
                format!("Filter: \"{}\"", query),
                Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
            ));
        }

        f.render_widget(
            Paragraph::new(Line::from(spans)).block(block),
            area,
        );
    }
}

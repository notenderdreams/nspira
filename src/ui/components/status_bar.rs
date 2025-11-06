use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Status bar widget for displaying help and status information
pub struct StatusBar {
    title: String,
    lines: Vec<Line<'static>>,
}

impl StatusBar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
        }
    }

    pub fn add_line(mut self, line: Line<'static>) -> Self {
        self.lines.push(line);
        self
    }

    pub fn add_control(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.lines.push(Line::from(vec![
            Span::styled(key.into(), Style::default().fg(Color::Yellow)),
            Span::raw(" - "),
            Span::raw(description.into()),
        ]));
        self
    }

    pub fn add_section_title(mut self, title: impl Into<String>) -> Self {
        self.lines.push(Line::raw(""));
        self.lines.push(Line::styled(
            title.into(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        self
    }

    pub fn add_info(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.lines.push(Line::from(vec![
            Span::raw(label.into()),
            Span::raw(": "),
            Span::styled(value.into(), Style::default().fg(Color::Green)),
        ]));
        self
    }

    pub fn add_status(mut self, message: impl Into<String>) -> Self {
        let message = message.into();
        if !message.is_empty() {
            self.lines.push(Line::raw(""));
            self.lines.push(Line::styled(
                "Status",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            self.lines.push(Line::styled(
                message,
                Style::default().fg(Color::White),
            ));
        }
        self
    }

    pub fn render(self, f: &mut Frame, area: Rect) {
        let help_block = Paragraph::new(self.lines)
            .block(Block::default().borders(Borders::ALL).title(self.title));
        f.render_widget(help_block, area);
    }
}

/// Create a standard help panel for list-like interfaces
pub fn create_list_help(
    selected_count: usize,
    total_count: usize,
    status_message: &str,
) -> StatusBar {
    let mut status_bar = StatusBar::new("Info")
        .add_section_title("Statistics")
        .add_info("Total", total_count.to_string())
        .add_info("Selected", selected_count.to_string())
        .add_section_title("Controls")
        .add_control("↑/↓ or j/k", "Navigate")
        .add_control("Space", "Toggle select")
        .add_control("a", "Select/Unselect all")
        .add_control("Enter", "Execute action")
        .add_control("d", "Delete selected")
        .add_control("q", "Quit");

    if !status_message.is_empty() {
        status_bar = status_bar.add_status(status_message);
    }

    status_bar
}

/// Create help panel for project list (cleaning context)
pub fn create_project_list_help(
    selected_count: usize,
    total_count: usize,
    status_message: &str,
) -> StatusBar {
    let mut status_bar = StatusBar::new("Info")
        .add_section_title("Statistics")
        .add_info("Projects", total_count.to_string())
        .add_info("Selected", selected_count.to_string())
        .add_section_title("Controls")
        .add_control("↑/↓ or j/k", "Navigate")
        .add_control("Space", "Toggle select")
        .add_control("a", "Select/Unselect all")
        .add_control("Enter", "Clean selected")
        .add_control("d", "Remove from tracking")
        .add_control("q", "Quit");

    if !status_message.is_empty() {
        status_bar = status_bar.add_status(status_message);
    }

    status_bar
}

/// Create help panel for scan results (adding context)
pub fn create_scan_help(
    selected_count: usize,
    total_count: usize,
    status_message: &str,
) -> StatusBar {
    let mut status_bar = StatusBar::new("Info")
        .add_section_title("Statistics")
        .add_info("Detected", total_count.to_string())
        .add_info("Selected", selected_count.to_string())
        .add_section_title("Controls")
        .add_control("↑/↓ or j/k", "Navigate")
        .add_control("Space", "Toggle select")
        .add_control("a", "Select/Unselect all")
        .add_control("Enter", "Add to tracking")
        .add_control("q", "Quit without adding");

    if !status_message.is_empty() {
        status_bar = status_bar.add_status(status_message);
    }

    status_bar
}

/// Create help panel for doctor (health check context)
pub fn create_doctor_help(
    _selected_count: usize,
    total_count: usize,
    healthy_count: usize,
    issues_count: usize,
    status_message: &str,
) -> StatusBar {
    let mut status_bar = StatusBar::new("Health Report")
        .add_section_title("Statistics")
        .add_info("Total Projects", total_count.to_string())
        .add_info("Healthy", healthy_count.to_string())
        .add_info("Issues Found", issues_count.to_string())
        .add_section_title("Controls")
        .add_control("↑/↓ or j/k", "Navigate")
        .add_control("d", "Remove broken project")
        .add_control("q", "Quit");

    if !status_message.is_empty() {
        status_bar = status_bar.add_status(status_message);
    }

    status_bar
}
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// Color palette and design tokens for nspira
pub struct Theme;

impl Theme {
    // Primary colors
    pub const PRIMARY: Color = Color::Rgb(56, 189, 248);     // Sky blue
    pub const SECONDARY: Color = Color::Rgb(192, 132, 252);  // Purple/Lavender
    pub const ACCENT: Color = Color::Rgb(251, 146, 60);      // Orange accent

    // Status colors
    pub const SUCCESS: Color = Color::Rgb(74, 222, 128);     // Emerald green
    pub const WARNING: Color = Color::Rgb(250, 204, 21);     // Yellow
    pub const DANGER: Color = Color::Rgb(248, 113, 113);     // Soft red
    pub const INFO: Color = Color::Rgb(96, 165, 250);       // Blue

    // Neutral colors
    pub const TEXT: Color = Color::Rgb(243, 244, 246);       // Light gray text
    pub const TEXT_MUTED: Color = Color::Rgb(156, 163, 175); // Dim gray
    pub const BORDER: Color = Color::Rgb(75, 85, 99);        // Dark gray border
    pub const BORDER_FOCUS: Color = Self::PRIMARY;
    pub const BG_HEADER: Color = Color::Rgb(30, 41, 59);     // Slate 800
    pub const HIGHLIGHT_BG: Color = Color::Rgb(51, 65, 85);  // Slate 700

    // Border type
    pub const BORDER_TYPE: BorderType = BorderType::Rounded;

    // Styles
    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_style() -> Style {
        Style::default()
            .fg(Self::SECONDARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_row_style() -> Style {
        Style::default()
            .bg(Self::HIGHLIGHT_BG)
            .fg(Self::TEXT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn checked_row_style() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    pub fn help_key_style() -> Style {
        Style::default()
            .fg(Self::WARNING)
            .add_modifier(Modifier::BOLD)
    }

    pub fn help_desc_style() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }
}

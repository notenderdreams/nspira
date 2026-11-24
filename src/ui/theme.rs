use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// ANSI 16 colors only, on the terminal's default background, so the TUI
/// follows whatever palette (light or dark) the terminal uses.
pub struct Theme;

impl Theme {
    pub const PRIMARY: Color = Color::Cyan;
    pub const SECONDARY: Color = Color::Magenta;
    pub const ACCENT: Color = Color::Yellow;

    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const DANGER: Color = Color::Red;
    pub const INFO: Color = Color::Blue;

    pub const TEXT: Color = Color::Reset;
    pub const TEXT_MUTED: Color = Color::DarkGray;
    pub const BORDER: Color = Color::DarkGray;

    pub const BORDER_TYPE: BorderType = BorderType::Rounded;

    pub fn title_style() -> Style {
        Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD)
    }

    pub fn header_style() -> Style {
        Style::default().fg(Self::SECONDARY).add_modifier(Modifier::BOLD)
    }

    /// Cursor row: reversed video works on any palette.
    pub fn selected_row_style() -> Style {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    }

    pub fn checked_row_style() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    pub fn muted() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }

    pub fn help_key_style() -> Style {
        Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD)
    }

    pub fn help_desc_style() -> Style {
        Self::muted()
    }
}

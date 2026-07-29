pub mod components;
pub mod model;
pub mod state;
pub mod theme;
pub mod views;

pub use components::*;
pub use model::*;
pub use state::*;
pub use theme::*;
pub use views::*;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, Stdout},
    panic,
    sync::Once,
};

static INIT_PANIC_HOOK: Once = Once::new();

/// Terminal guard that automatically restores terminal settings on drop
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        INIT_PANIC_HOOK.call_once(|| {
            let original_hook = panic::take_hook();
            panic::set_hook(Box::new(move |panic_info| {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                original_hook(panic_info);
            }));
        });

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Initialize terminal guard
pub fn init_terminal() -> Result<TerminalGuard> {
    TerminalGuard::new()
}

/// Event returned by poll_event
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Key(KeyEvent),
    Resize(u16, u16),
}

/// Poll for keyboard/terminal events with proper key press filtering
pub fn poll_event(timeout_ms: u64) -> Result<Option<TerminalEvent>> {
    if event::poll(std::time::Duration::from_millis(timeout_ms))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Check for Ctrl+C emergency exit
                if key.code == crossterm::event::KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    return Ok(Some(TerminalEvent::Key(KeyEvent::new(
                        crossterm::event::KeyCode::Char('q'),
                        KeyModifiers::NONE,
                    ))));
                }
                Ok(Some(TerminalEvent::Key(key)))
            }
            Event::Resize(w, h) => Ok(Some(TerminalEvent::Resize(w, h))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

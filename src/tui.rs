//! Core TUI implementation
//!
//! This module provides the TUI functionality, handling terminal setup,
//! rendering, cleanup, and state management in an atomic, non-blocking way.

use std::io::{self, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Rect, Size},
    prelude::Line,
    style::Style,
    widgets::Block,
    Terminal,
};

use crate::{
    colors::theme,
    error::{OxittyError, OxittyResult},
    state::AtomicState,
};

/// Terminal user interface manager
pub struct Tui<S: AtomicState> {
    /// Terminal instance for rendering
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Application state
    state: S,
}

impl<S: AtomicState> Tui<S> {
    /// Creates a new TUI instance
    pub fn new(state: S) -> OxittyResult<Self> {
        // Check if we're in a real terminal
        if !Self::is_real_terminal() {
            return Err(OxittyError::terminal(
                "terminal check",
                (0, 0),
                "Not a real terminal or terminal capabilities not available".to_string(),
            )
            .into());
        }

        let terminal = Self::setup_terminal()?;
        Ok(Self { terminal, state })
    }

    /// Check if we're in a real terminal
    fn is_real_terminal() -> bool {
        // Check if stdout is a tty
        if !atty::is(atty::Stream::Stdout) {
            return false;
        }

        // Check terminal environment
        match std::env::var("TERM") {
            Ok(term) if term == "dumb" => false,
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Sets up the terminal for TUI operation
    fn setup_terminal() -> OxittyResult<Terminal<CrosstermBackend<Stdout>>> {
        let mut stdout = io::stdout();

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
            OxittyError::terminal(
                "terminal setup",
                (0, 0),
                format!("Failed to setup terminal: {}", e),
            )
        })?;

        terminal::enable_raw_mode().map_err(|e| {
            OxittyError::terminal(
                "terminal setup",
                (0, 0),
                format!("Failed to enable raw mode: {}", e),
            )
        })?;

        Terminal::new(CrosstermBackend::new(stdout)).map_err(|e| {
            OxittyError::terminal(
                "terminal setup",
                (0, 0),
                format!("Failed to create terminal: {}", e),
            )
            .into()
        })
    }

    /// Restores terminal to original state
    fn restore_terminal(&mut self) -> OxittyResult<()> {
        terminal::disable_raw_mode().map_err(|e| {
            OxittyError::terminal(
                "terminal cleanup",
                (0, 0),
                format!("Failed to disable raw mode: {}", e),
            )
        })?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| {
            OxittyError::terminal(
                "terminal cleanup",
                (0, 0),
                format!("Failed to restore terminal: {}", e),
            )
        })?;

        Ok(self.terminal.show_cursor().map_err(|e| {
            OxittyError::terminal(
                "terminal cleanup",
                (0, 0),
                format!("Failed to show cursor: {}", e),
            )
        })?)
    }

    /// Renders a frame using the provided render function
    pub fn render<F>(&mut self, render_fn: F) -> OxittyResult<()>
    where
        F: FnOnce(&S::Snapshot, Rect, &mut ratatui::Frame<'_>),
    {
        let snapshot = self.state.snapshot();

        Ok(self
            .terminal
            .draw(|frame| {
                let area = frame.area();
                render_fn(&snapshot, area, frame);
            })
            .map(|_| ())
            .map_err(|e| {
                OxittyError::terminal(
                    "rendering",
                    (0, 0),
                    format!("Failed to render frame: {}", e),
                )
            })?)
    }

    /// Returns a reference to the terminal
    pub fn terminal(&self) -> &Terminal<CrosstermBackend<Stdout>> {
        &self.terminal
    }

    /// Returns a reference to the state
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Returns the terminal size
    pub fn size(&self) -> OxittyResult<Size> {
        Ok(self.terminal.size().map_err(|e| {
            OxittyError::terminal(
                "terminal size",
                (0, 0),
                format!("Failed to get terminal size: {}", e),
            )
        })?)
    }

    /// Flushes the terminal
    pub fn flush(&mut self) -> OxittyResult<()> {
        Ok(self.terminal.flush().map_err(|e| {
            OxittyError::terminal(
                "terminal flush",
                (0, 0),
                format!("Failed to flush terminal: {}", e),
            )
        })?)
    }

    /// Returns the default theme style
    pub fn style() -> Style {
        Style::default()
            .fg(theme::text::PRIMARY.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style with primary text color
    pub fn primary() -> Style {
        Style::default()
            .fg(theme::text::PRIMARY.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style with secondary text color
    pub fn secondary() -> Style {
        Style::default()
            .fg(theme::text::SECONDARY.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for error messages
    pub fn error() -> Style {
        Style::default()
            .fg(theme::status::ERROR.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for warning messages
    pub fn warning() -> Style {
        Style::default()
            .fg(theme::status::WARNING.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for info messages
    pub fn info() -> Style {
        Style::default()
            .fg(theme::status::INFO.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for success messages
    pub fn success() -> Style {
        Style::default()
            .fg(theme::status::SUCCESS.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for UI borders
    pub fn border() -> Style {
        Style::default()
            .fg(theme::background::ELEVATION_3.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for focused elements
    pub fn focus() -> Style {
        Style::default()
            .fg(theme::void::PURPLE.into())
            .bg(theme::background::BASE.into())
    }

    /// Returns a style for void elements
    pub fn void() -> Style {
        Style::default()
            .fg(theme::void::GREEN.into())
            .bg(theme::background::BASE.into())
    }

    /// Creates a themed block with the given title
    pub fn block(title: impl Into<String>) -> Block<'static> {
        Block::default()
            .title(Line::from(title.into()))
            .style(Self::primary())
            .border_style(Self::border())
    }
}

impl<S: AtomicState> Drop for Tui<S> {
    fn drop(&mut self) {
        if let Err(e) = self.restore_terminal() {
            eprintln!("Failed to restore terminal: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock terminal setup
    fn setup_mock_terminal() {
        // Force non-interactive environment
        std::env::remove_var("TERM");
        std::env::remove_var("COLORTERM");
        std::env::remove_var("TERMINFO");
        std::env::remove_var("TERMINFO_DIRS");

        // Set minimal dumb terminal
        std::env::set_var("TERM", "dumb");
    }

    #[derive(Debug, Clone)]
    struct TestSnapshot {
        running: bool,
    }

    impl crate::state::StateSnapshot for TestSnapshot {
        fn should_quit(&self) -> bool {
            !self.running
        }
    }

    #[derive(Debug)]
    struct TestState {
        running: AtomicBool,
    }

    impl AtomicState for TestState {
        type Snapshot = TestSnapshot;

        fn snapshot(&self) -> Self::Snapshot {
            TestSnapshot {
                running: self.running.load(Ordering::Acquire),
            }
        }

        fn quit(&self) {
            self.running.store(false, Ordering::Release);
        }

        fn is_running(&self) -> bool {
            self.running.load(Ordering::Acquire)
        }
    }

    #[test]
    fn test_terminal_not_available() {
        setup_mock_terminal();

        let state = TestState {
            running: AtomicBool::new(true),
        };

        let result = Tui::new(state);
        assert!(
            result.is_err(),
            "Expected TUI creation to fail in mock environment"
        );

        if let Err(e) = result {
            let err_msg = e.to_string().to_lowercase();
            assert!(
                err_msg.contains("terminal") || err_msg.contains("tty"),
                "Expected terminal-related error, got: {}",
                err_msg
            );
        }
    }

    #[test]
    fn test_tui_creation() {
        setup_mock_terminal();

        let state = TestState {
            running: AtomicBool::new(true),
        };

        let result = Tui::new(state);
        assert!(
            result.is_err(),
            "Expected TUI creation to fail in mock environment"
        );

        if let Err(e) = result {
            let err_msg = e.to_string().to_lowercase();
            assert!(
                err_msg.contains("terminal") || err_msg.contains("tty"),
                "Expected terminal-related error, got: {}",
                err_msg
            );
        }
    }

    #[test]
    fn test_theme_styles() {
        // Test primary style
        let style = Tui::<TestState>::primary();
        assert_eq!(style.fg, Some(theme::text::PRIMARY.into()));
        assert_eq!(style.bg, Some(theme::background::BASE.into()));

        // Test error style
        let style = Tui::<TestState>::error();
        assert_eq!(style.fg, Some(theme::status::ERROR.into()));
        assert_eq!(style.bg, Some(theme::background::BASE.into()));

        // Test border style
        let style = Tui::<TestState>::border();
        assert_eq!(style.fg, Some(theme::background::ELEVATION_3.into()));
        assert_eq!(style.bg, Some(theme::background::BASE.into()));
    }

    #[test]
    fn test_themed_block() {
        let title = "Test";
        let themed_block = Tui::<TestState>::block(title);

        // Create styles we expect the block to be created with
        let expected_style = Tui::<TestState>::primary();
        let expected_border = Tui::<TestState>::border();

        // Create a reference block with same styles to compare
        let reference_block = Block::default()
            .style(expected_style)
            .border_style(expected_border)
            .title(Line::from(title));

        // Assert our themed block matches the reference
        assert_eq!(themed_block, reference_block);
    }
}

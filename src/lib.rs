#![forbid(unsafe_code)]

//! # Oxitty
//!
//! An async-first terminal UI framework for Rust.
//!
//! ## Overview
//!
//! The framework is built around a few main concepts:
//!
//! - **Atomic State Management**: Thread-safe state handling with snapshot-based updates
//! - **Event-Driven Architecture**: Non-blocking event processing with custom event support
//! - **Themed Rendering**: Consistent and customizable terminal UI theming
//! - **Async-First Design**: Built on `smol` for efficient async operations
//!
//! ## Core Components
//!
//! - [`App`]: Main application orchestrator coordinating state, events, and rendering
//! - [`Tui`]: Terminal interface manager handling setup, cleanup, and rendering
//! - [`EventHandler`]: Async event processing system
//! - [`AtomicState`]: Thread-safe state management trait
//! - [`StateSnapshot`]: Immutable state snapshot trait
//! - [`Color`]: RGBA color management with theme support
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use oxitty::{App, AtomicState, StateSnapshot};
//! use std::{sync::atomic::{AtomicBool, Ordering}, time::Duration};
//!
//! // Define your application state
//! #[derive(Debug)]
//! struct MyState {
//!     running: AtomicBool,
//! }
//!
//! #[derive(Debug, Clone)]
//! struct MySnapshot {
//!     running: bool,
//! }
//!
//! impl StateSnapshot for MySnapshot {
//!     fn should_quit(&self) -> bool {
//!         !self.running
//!     }
//! }
//!
//! impl AtomicState for MyState {
//!     type Snapshot = MySnapshot;
//!
//!     fn snapshot(&self) -> Self::Snapshot {
//!         MySnapshot {
//!             running: self.running.load(Ordering::Acquire),
//!         }
//!     }
//!
//!     fn quit(&self) {
//!         self.running.store(false, Ordering::Release);
//!     }
//!
//!     fn is_running(&self) -> bool {
//!         self.running.load(Ordering::Acquire)
//!     }
//! }
//!
//! // Create and run your application
//! fn main() -> oxitty::OxittyResult<()> {
//!     let state = MyState {
//!         running: AtomicBool::new(true),
//!     };
//!
//!     smol::block_on(async {
//!         let mut app = App::new(state, Duration::from_millis(50))?;
//!         app.run(|snapshot, area, frame| {
//!             // Your render logic here
//!         }).await
//!     })
//! }
//! ```
//!
//! ## Module Organization
//!
//! - `app`: Application orchestration and lifecycle management
//! - `colors`: Color system with theme support
//! - `error`: Error types and handling
//! - `event`: Event processing system
//! - `state`: State management traits
//! - `tui`: Terminal interface management
//!
//! ## Feature Highlights
//!
//! - Zero-cost abstractions for efficient terminal manipulation
//! - Type-safe atomic state management
//! - Non-blocking event processing with custom event support
//! - Comprehensive color theming system
//! - Async-first design with `smol` runtime
//! - Memory-safe operations with `#[forbid(unsafe_code)]`
//!
//! ## Error Handling
//!
//! The library uses [`OxittyResult`] and [`OxittyError`] for comprehensive error
//! handling with detailed diagnostics via `miette`.
//!
//! ## Color Theming
//!
//! The [`Color`] and [`ThemeColorize`] types provide a rich color management system
//! with support for RGBA colors, color space conversions, and semantic theming.

/// Re-exports of core components
pub use app::App;
pub use colors::{Color, ThemeColorize};
pub use error::{OxittyError, OxittyResult};
pub use event::{Event, EventHandler};
pub use state::{AtomicState, StateSnapshot};
pub use tui::Tui;

/// Application orchestration module
pub mod app;
/// Color system and theme management
pub mod colors;
/// Error types and handling
pub mod error;
/// Event processing system
pub mod event;
/// State management traits
pub mod state;
/// Terminal interface management
pub mod tui;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_state_integration() {
        #[derive(Debug)]
        struct TestState {
            running: AtomicBool,
        }

        #[derive(Debug, Clone)]
        struct TestSnapshot {
            running: bool,
        }

        impl StateSnapshot for TestSnapshot {
            fn should_quit(&self) -> bool {
                !self.running
            }
        }

        impl AtomicState for TestState {
            type Snapshot = TestSnapshot;

            fn snapshot(&self) -> Self::Snapshot {
                TestSnapshot {
                    running: self.running.load(Ordering::Acquire),
                }
            }

            fn quit(&self) {
                self.running.store(false, Ordering::Release)
            }

            fn is_running(&self) -> bool {
                self.running.load(Ordering::Acquire)
            }
        }

        let state = TestState {
            running: AtomicBool::new(true),
        };

        assert!(state.is_running());
        let snapshot = state.snapshot();
        assert!(!snapshot.should_quit());

        state.quit();
        assert!(!state.is_running());
        let snapshot = state.snapshot();
        assert!(snapshot.should_quit());
    }
}

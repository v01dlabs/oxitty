#![forbid(unsafe_code)]

pub use app::App;
pub use colors::{Color, ThemeColorize};
pub use error::{OxittyError, OxittyResult};
pub use event::{Event, EventHandler};
pub use state::{AtomicState, StateSnapshot};
pub use tui::Tui;

pub mod app;
pub mod colors;
pub mod error;
pub mod event;
pub mod state;
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

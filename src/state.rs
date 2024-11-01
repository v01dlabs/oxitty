//! Core state management traits and implementations

use std::fmt::Debug;

/// Represents an immutable snapshot of application state
pub trait StateSnapshot: Clone + Send + Debug + 'static {
    /// Returns whether the application should quit
    fn should_quit(&self) -> bool;
}

/// Represents a thread-safe atomic application state
pub trait AtomicState: Send + Sync + Debug + 'static {
    /// The type of snapshot this state produces
    type Snapshot: StateSnapshot;

    /// Take a consistent snapshot of the current state
    fn snapshot(&self) -> Self::Snapshot;

    /// Signal the application to quit
    fn quit(&self);

    /// Check if the application is still running
    fn is_running(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug, Clone)]
    struct TestSnapshot {
        running: bool,
    }

    impl StateSnapshot for TestSnapshot {
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
    fn test_state_lifecycle() {
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

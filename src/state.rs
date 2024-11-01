//! Core state management traits and implementations for Oxitty.
//!
//! This module provides a thread-safe state management system with atomic
//! operations and consistent snapshot capabilities. The design allows for:
//!
//! * Thread-safe state mutations
//! * Immutable state snapshots
//! * Atomic lifecycle management
//! * Type-safe state access
//!
//! # Architecture
//!
//! The state system is built around two key traits:
//!
//! * [`StateSnapshot`] - Represents an immutable point-in-time view of the application state
//! * [`AtomicState`] - Provides thread-safe access to mutable application state
//!
//! # Example
//!
//! ```rust
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use oxitty::state::{AtomicState, StateSnapshot};
//!
//! #[derive(Debug, Clone)]
//! struct MySnapshot {
//!     running: bool
//! }
//!
//! impl StateSnapshot for MySnapshot {
//!     fn should_quit(&self) -> bool {
//!         !self.running
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct MyState {
//!     running: AtomicBool
//! }
//!
//! impl AtomicState for MyState {
//!     type Snapshot = MySnapshot;
//!
//!     fn snapshot(&self) -> Self::Snapshot {
//!         MySnapshot {
//!             running: self.running.load(Ordering::Acquire)
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
//! ```

use std::fmt::Debug;

/// Represents an immutable snapshot of application state.
///
/// This trait defines the interface for immutable state snapshots that can be
/// safely shared across threads. Implementors must be `Clone` to allow for
/// efficient snapshot distribution and `Send` for thread safety.
pub trait StateSnapshot: Clone + Send + Debug + 'static {
    /// Returns whether the application should quit based on this snapshot's state.
    ///
    /// This method provides a consistent view of the application's lifecycle state
    /// at the time the snapshot was taken.
    fn should_quit(&self) -> bool;
}

/// Represents thread-safe atomic application state.
///
/// This trait defines the interface for thread-safe state management, providing
/// atomic operations for state mutation and consistent snapshot creation. Implementors
/// must be both `Send` and `Sync` for safe concurrent access.
pub trait AtomicState: Send + Sync + Debug + 'static {
    /// The type of snapshot this state produces.
    type Snapshot: StateSnapshot;

    /// Takes a consistent snapshot of the current state.
    ///
    /// Creates an immutable point-in-time view of the application state that can
    /// be safely shared across threads.
    fn snapshot(&self) -> Self::Snapshot;

    /// Signals the application to quit.
    ///
    /// This method should atomically update the state to indicate that the
    /// application should terminate.
    fn quit(&self);

    /// Checks if the application is still running.
    ///
    /// Returns the current running state of the application, using appropriate
    /// atomic operations for thread safety.
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

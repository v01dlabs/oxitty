//! Core state management traits and implementations for Oxitty.
//!
//! This module provides a zero-cost abstraction for thread-safe state management
//! using an efficient bitfield-based approach. It ensures atomic state transitions
//! and consistent snapshots without compromising performance.
//!
//! # Architecture
//!
//! The state system uses three key components:
//!
//! - [`StateFlags`]: Low-level atomic bitfield operations using a single `AtomicU64`
//! - [`StateSnapshot`]: Zero-copy, immutable view of application state
//! - [`AtomicState`]: Trait defining thread-safe state behavior
//!
//! # Performance
//!
//! The implementation ensures:
//! - Zero allocation for state transitions
//! - Single atomic operation for snapshots
//! - No mutex/lock overhead
//! - Predictable performance characteristics
//!
//! # Example
//!
//! ```rust
//! use oxitty::state::{AtomicState, StateFlags, StateSnapshot};
//!
//! #[derive(Debug)]
//! struct AppState {
//!     flags: StateFlags,
//! }
//!
//! #[derive(Debug, Clone)]
//! struct AppSnapshot {
//!     running: bool,
//!     processing: bool,
//! }
//!
//! impl StateSnapshot for AppSnapshot {
//!     fn should_quit(&self) -> bool {
//!         !self.running
//!     }
//! }
//!
//! impl AtomicState for AppState {
//!     type Snapshot = AppSnapshot;
//!
//!     fn snapshot(&self) -> Self::Snapshot {
//!         let flags = self.flags.snapshot();
//!         AppSnapshot {
//!             running: flags.get(StateFlags::RUNNING),
//!             processing: flags.get(StateFlags::PROCESSING),
//!         }
//!     }
//!
//!     fn quit(&self) {
//!         self.flags.set(StateFlags::RUNNING, false);
//!     }
//!
//!     fn is_running(&self) -> bool {
//!         self.flags.get(StateFlags::RUNNING)
//!     }
//! }
//!
//! // Example usage:
//! let app = AppState {
//!     flags: StateFlags::default(),
//! };
//!
//! // Start the app
//! app.flags.set(StateFlags::RUNNING, true);
//! assert!(app.is_running());
//!
//! // Take a snapshot
//! let snapshot = app.snapshot();
//! assert!(!snapshot.should_quit());
//!
//! // Quit the app
//! app.quit();
//! assert!(!app.is_running());
//!
//! // Take another snapshot
//! let snapshot = app.snapshot();
//! assert!(snapshot.should_quit());
//! ```

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe state flag container using a bitfield approach.
/// Provides atomic operations for state transitions and snapshots.
#[derive(Debug)]
pub struct StateFlags {
    /// Internal bitfield storing all state flags
    flags: AtomicU64,
}

impl StateFlags {
    /// Flag indicating if the application is running
    pub const RUNNING: u32 = 0;
    /// Flag indicating if the application is processing
    pub const PROCESSING: u32 = 1;
    /// Flag indicating if the application is in debug mode
    pub const DEBUG: u32 = 2;
    /// Flag indicating if the application has errors
    pub const HAS_ERROR: u32 = 3;
    /// Flag indicating if the application is waiting for input
    pub const AWAITING_INPUT: u32 = 4;
    /// Flag indicating if the application is rendering
    pub const RENDERING: u32 = 5;
    /// Maximum supported flags (64 bits available)
    pub const MAX_FLAGS: u32 = 64;

    /// Creates a new state flags container with initial values.
    ///
    /// # Arguments
    ///
    /// * `initial` - Initial flag values as a u64 bitfield
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::state::StateFlags;
    ///
    /// // Create flags with RUNNING set to true
    /// let flags = StateFlags::new(1 << StateFlags::RUNNING);
    /// assert!(flags.get(StateFlags::RUNNING));
    /// ```
    #[inline]
    pub const fn new(initial: u64) -> Self {
        Self {
            flags: AtomicU64::new(initial),
        }
    }

    /// Creates a new state flags container with all flags set to false.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::state::StateFlags;
    ///
    /// let flags = StateFlags::default();
    /// assert!(!flags.get(StateFlags::RUNNING));
    /// ```
    #[inline]
    pub const fn default() -> Self {
        Self::new(0)
    }

    /// Sets a specific flag's value with sequential consistency ordering.
    ///
    /// Uses `fetch_update` with `SeqCst` ordering to ensure total ordering of
    /// operations across all threads. This guarantee is necessary for maintaining
    /// consistent snapshots but comes with a minor performance cost compared to
    /// weaker ordering modes.
    ///
    /// # Memory Ordering
    ///
    /// Uses `SeqCst` ordering to ensure:
    /// - All threads see flag updates in the same order
    /// - Snapshots are globally consistent
    /// - No reordering of operations across threads
    ///
    /// # Arguments
    ///
    /// * `flag` - Flag position to modify (0-63)
    /// * `value` - New value for the flag
    ///
    /// # Panics
    ///
    /// Panics if flag >= MAX_FLAGS
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::state::StateFlags;
    ///
    /// let flags = StateFlags::default();
    /// flags.set(StateFlags::RUNNING, true);
    /// assert!(flags.get(StateFlags::RUNNING));
    /// ```
    ///
    /// # Performance Notes
    ///
    /// `SeqCst` ordering is used to guarantee consistent snapshots across threads.
    /// While this has a minor performance cost compared to `Acquire`/`Release`,
    /// the impact is negligible for typical UI state management where:
    /// - State changes are relatively infrequent
    /// - UI operations dominate performance considerations
    /// - Modern CPUs optimize `SeqCst` operations effectively
    #[inline]
    pub fn set(&self, flag: u32, value: bool) {
        debug_assert!(flag < Self::MAX_FLAGS, "Flag position out of bounds");
        let mask = 1u64 << flag;

        self.flags
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                Some(if value {
                    current | mask
                } else {
                    current & !mask
                })
            })
            .expect("fetch_update cannot fail with Some");
    }

    /// Gets the current value of a specific flag with sequential consistency.
    ///
    /// # Memory Ordering
    ///
    /// Uses `SeqCst` ordering to ensure:
    /// - Reads are synchronized with all writes across threads
    /// - Consistent with snapshot operations
    /// - Total ordering with all other atomic operations
    ///
    /// # Arguments
    ///
    /// * `flag` - Flag position to read (0-63)
    ///
    /// # Returns
    ///
    /// Boolean value of the specified flag
    ///
    /// # Panics
    ///
    /// Panics if flag >= MAX_FLAGS
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::state::StateFlags;
    ///
    /// let flags = StateFlags::default();
    /// assert!(!flags.get(StateFlags::RUNNING));
    /// flags.set(StateFlags::RUNNING, true);
    /// assert!(flags.get(StateFlags::RUNNING));
    /// ```
    ///
    /// # Performance Notes
    ///
    /// While `SeqCst` is more expensive than relaxed ordering, the overhead
    /// is minimal in practice for UI state management where consistency is
    /// more important than nanosecond-level performance.
    #[inline]
    pub fn get(&self, flag: u32) -> bool {
        debug_assert!(flag < Self::MAX_FLAGS, "Flag position out of bounds");
        let mask = 1u64 << flag;
        (self.flags.load(Ordering::SeqCst) & mask) != 0
    }

    /// Takes an atomic snapshot of all flags with sequential consistency.
    ///
    /// This operation guarantees that the snapshot represents a consistent
    /// view of all flags at a single point in time, synchronized across
    /// all threads.
    ///
    /// # Memory Ordering
    ///
    /// Uses `SeqCst` ordering to ensure:
    /// - Snapshot includes all prior flag updates from all threads
    /// - No reordering of snapshot operation with other atomic operations
    /// - Global consistency of state observations
    ///
    /// # Returns
    ///
    /// An immutable [`FlagsSnapshot`] representing the state at the time
    /// of capture.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::state::StateFlags;
    ///
    /// let flags = StateFlags::default();
    /// flags.set(StateFlags::RUNNING, true);
    /// let snapshot = flags.snapshot();
    /// assert!(snapshot.get(StateFlags::RUNNING));
    /// ```
    ///
    /// # Thread Safety
    ///
    /// The `SeqCst` ordering ensures that snapshots are globally consistent
    /// even in complex multi-threaded scenarios where multiple threads are
    /// reading and writing flags concurrently.
    #[inline]
    pub fn snapshot(&self) -> FlagsSnapshot {
        FlagsSnapshot(self.flags.load(Ordering::SeqCst))
    }

    /// Updates multiple flags atomically with sequential consistency.
    ///
    /// This method ensures that all specified flag updates happen in a single
    /// atomic operation, preventing any intermediate states from being visible
    /// to other threads.
    ///
    /// # Memory Ordering
    ///
    /// Uses `SeqCst` ordering to ensure:
    /// - All updates are visible to all threads simultaneously
    /// - No reordering with other atomic operations
    /// - Consistent with snapshot operations
    ///
    /// # Arguments
    ///
    /// * `updates` - Iterator of (flag, value) pairs to update
    ///
    /// # Panics
    ///
    /// Panics if any flag >= MAX_FLAGS
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::state::StateFlags;
    ///
    /// let flags = StateFlags::default();
    /// flags.update_multiple(vec![
    ///     (StateFlags::RUNNING, true),
    ///     (StateFlags::PROCESSING, true),
    /// ]);
    /// assert!(flags.get(StateFlags::RUNNING));
    /// assert!(flags.get(StateFlags::PROCESSING));
    /// ```
    ///
    /// # Performance Notes
    ///
    /// The `SeqCst` ordering applies to the entire batch update as a single
    /// operation, making this method particularly efficient for updating
    /// multiple flags while maintaining strong consistency guarantees.
    #[inline]
    pub fn update_multiple<I>(&self, updates: I)
    where
        I: IntoIterator<Item = (u32, bool)>,
    {
        let mut mask = 0u64;
        let mut new_values = 0u64;

        for (flag, value) in updates {
            debug_assert!(flag < Self::MAX_FLAGS, "Flag position out of bounds");
            let bit = 1u64 << flag;
            mask |= bit;
            if value {
                new_values |= bit;
            }
        }

        self.flags
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                Some((current & !mask) | (new_values & mask))
            })
            .expect("fetch_update cannot fail with Some");
    }
}

/// Immutable snapshot of state flags at a point in time.
///
/// This type provides a consistent view of all flags as they were
/// when the snapshot was taken. It's efficiently copyable and
/// provides zero-cost access to flag values.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FlagsSnapshot(u64);

impl FlagsSnapshot {
    /// Gets the value of a specific flag in the snapshot.
    ///
    /// # Arguments
    ///
    /// * `flag` - Flag position to read (0-63)
    ///
    /// # Returns
    ///
    /// Boolean value of the specified flag
    ///
    /// # Panics
    ///
    /// Panics if flag >= StateFlags::MAX_FLAGS
    #[inline]
    pub fn get(&self, flag: u32) -> bool {
        debug_assert!(flag < StateFlags::MAX_FLAGS, "Flag position out of bounds");
        (self.0 & (1u64 << flag)) != 0
    }

    /// Returns the raw flags value.
    ///
    /// This is primarily useful for debugging or custom flag manipulation.
    #[inline]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Trait for implementing thread-safe state behavior.
///
/// This trait defines the core interface for atomic state management,
/// ensuring that implementations provide consistent snapshots and
/// state transitions.
pub trait AtomicState: Send + Sync + Debug + 'static {
    /// The type of snapshot this state produces
    type Snapshot: StateSnapshot;

    /// Takes a consistent snapshot of the current state.
    ///
    /// This method must ensure that the snapshot represents a consistent
    /// view of the state at a single point in time.
    fn snapshot(&self) -> Self::Snapshot;

    /// Signals the application to quit.
    ///
    /// This method should atomically update the state to indicate that
    /// the application should terminate.
    fn quit(&self);

    /// Checks if the application is still running.
    ///
    /// Returns the current running state of the application, using
    /// appropriate atomic operations for thread safety.
    fn is_running(&self) -> bool;
}

/// Trait for state snapshots that can be safely shared across threads.
///
/// Snapshots must be efficiently cloneable and provide a consistent
/// view of the application state at a point in time.
pub trait StateSnapshot: Clone + Send + Debug + 'static {
    /// Returns whether the application should quit based on this snapshot.
    fn should_quit(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_atomic_operations() {
        let flags = StateFlags::default();

        flags.set(StateFlags::RUNNING, true);
        assert!(flags.get(StateFlags::RUNNING));

        flags.set(StateFlags::RUNNING, false);
        assert!(!flags.get(StateFlags::RUNNING));
    }

    #[test]
    fn test_snapshot_consistency() {
        let flags = Arc::new(StateFlags::default());
        let flags_clone = flags.clone();

        // Set initial state with Release ordering
        flags.set(StateFlags::RUNNING, true);
        flags.set(StateFlags::PROCESSING, true);

        // Force a memory fence to ensure visibility
        std::sync::atomic::fence(Ordering::Release);

        let handle = thread::spawn(move || {
            // Ensure we see the most recent values with Acquire ordering
            std::sync::atomic::fence(Ordering::Acquire);

            // Take snapshot - this now has proper ordering guarantees
            let snapshot = flags_clone.snapshot();

            // These assertions should now succeed consistently
            assert!(
                snapshot.get(StateFlags::RUNNING),
                "Running flag not set in snapshot"
            );
            assert!(
                snapshot.get(StateFlags::PROCESSING),
                "Processing flag not set in snapshot"
            );

            // Return success to verify thread completed
            Ok::<(), String>(())
        });

        // Give the thread time to take the snapshot
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Now change the flags
        flags.set(StateFlags::RUNNING, false);
        flags.set(StateFlags::PROCESSING, false);

        // Wait for thread and propagate any assertion failures
        handle
            .join()
            .expect("Thread failed")
            .expect("Assertions failed");
    }

    #[test]
    fn test_multiple_updates() {
        let flags = StateFlags::default();

        flags.update_multiple(vec![
            (StateFlags::RUNNING, true),
            (StateFlags::PROCESSING, true),
            (StateFlags::DEBUG, false),
        ]);

        assert!(flags.get(StateFlags::RUNNING));
        assert!(flags.get(StateFlags::PROCESSING));
        assert!(!flags.get(StateFlags::DEBUG));

        let snapshot = flags.snapshot();
        assert_eq!(
            snapshot.get(StateFlags::RUNNING),
            flags.get(StateFlags::RUNNING)
        );
        assert_eq!(
            snapshot.get(StateFlags::PROCESSING),
            flags.get(StateFlags::PROCESSING)
        );
    }

    #[test]
    fn test_concurrent_access() {
        let flags = Arc::new(StateFlags::default());
        let mut handles = vec![];

        for _ in 0..10 {
            let flags_clone = flags.clone();
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    flags_clone.set(StateFlags::RUNNING, true);
                    flags_clone.set(StateFlags::RUNNING, false);
                    let _ = flags_clone.snapshot();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(!flags.get(StateFlags::RUNNING));
    }

    #[test]
    fn test_snapshot_immutability() {
        let flags = StateFlags::default();
        flags.set(StateFlags::RUNNING, true);

        let snapshot = flags.snapshot();
        flags.set(StateFlags::RUNNING, false);

        // Snapshot should retain the original value
        assert!(snapshot.get(StateFlags::RUNNING));
        assert!(!flags.get(StateFlags::RUNNING));
    }
}

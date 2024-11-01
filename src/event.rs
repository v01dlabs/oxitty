//! Event handling system for the TUI framework
//!
//! This module provides a non-blocking, zero-copy event handling system
//! that processes terminal and custom events asynchronously. Built around a
//! bounded channel architecture, it prevents memory exhaustion from event
//! queuing while maintaining high performance.
//!
//! # Architecture
//!
//! The event system consists of three main components:
//!
//! - [`Event`]: Represents different types of terminal and custom events
//! - [`EventHandler`]: Manages event processing and distribution
//! - [`CloneableAny`]: Enables type-safe cloning of custom event types
//!
//! # Examples
//!
//! ```rust
//! use std::time::Duration;
//! use oxitty::event::{Event, EventHandler};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let handler = EventHandler::new();
//!
//! // Start event polling in background
//! handler.run(Duration::from_millis(50)).await?;
//!
//! // Check for events
//! if let Some(event) = handler.try_recv()? {
//!     match event {
//!         Event::Key(key) => println!("Key: {:?}", key),
//!         Event::Mouse(mouse) => println!("Mouse: {:?}", mouse),
//!         Event::Resize(w, h) => println!("Resize: {}x{}", w, h),
//!         Event::Custom(_) => println!("Custom event"),
//!         Event::Quit => println!("Quit"),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use smol::channel::{bounded, Receiver, Sender};
use std::{
    any::Any,
    clone::Clone,
    fmt::Debug,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::error::{OxittyError, OxittyResult};

/// Maximum number of pending events in the channel.
///
/// This limit prevents memory exhaustion from event queuing while still allowing
/// for reasonable event buffering.
const MAX_EVENTS: usize = 1024;

/// Terminal events that can occur during application execution.
///
/// This enum represents all possible event types that can flow through the event system,
/// including keyboard input, mouse interactions, terminal resizes, custom events,
/// and quit signals.
///
/// # Examples
///
/// ```rust
/// use oxitty::event::Event;
/// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
///
/// let key_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
/// let resize_event = Event::Resize(80, 24);
/// let quit_event = Event::Quit;
/// ```
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press events containing keyboard input information
    Key(KeyEvent),
    /// Mouse interaction events containing position and button information
    Mouse(MouseEvent),
    /// Terminal resize events containing new dimensions (width, height)
    Resize(u16, u16),
    /// Custom events for application-specific needs.
    /// Can contain any type implementing CloneableAny + Send
    Custom(Box<dyn CloneableAny + Send>),
    /// Event indicating the event loop should terminate
    Quit,
}

/// A trait for cloning `Any` trait objects in a type-safe manner.
///
/// This trait enables custom event types to be cloned while maintaining type safety
/// through the type system. It extends `Any` to allow for dynamic typing while
/// requiring `Clone` semantics.
///
/// # Examples
///
/// ```rust
/// use oxitty::event::CloneableAny;
///
/// #[derive(Debug, Clone)]
/// struct CustomEvent {
///     message: String,
/// }
///
/// // Implementation is automatically provided for types that are
/// // 'static + Any + Clone + Send + Debug
/// ```
pub trait CloneableAny: Any + Debug {
    /// Clones the `Any` trait object and returns a boxed clone.
    fn clone_box(&self) -> Box<dyn CloneableAny + Send>;
}

impl<T> CloneableAny for T
where
    T: 'static + std::any::Any + Clone + Send + Debug,
{
    fn clone_box(&self) -> Box<dyn CloneableAny + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneableAny + Send> {
    fn clone(&self) -> Box<dyn CloneableAny + Send> {
        self.clone_box()
    }
}

/// Handles event processing and distribution in an asynchronous manner.
///
/// `EventHandler` provides a non-blocking interface for processing terminal
/// and custom events. It uses a bounded channel to prevent memory exhaustion
/// and provides efficient event delivery.
///
/// # Thread Safety
///
/// The handler is thread-safe and can be shared across threads using an `Arc`.
/// The internal channel ensures thread-safe communication.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use oxitty::event::EventHandler;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let handler = EventHandler::new();
///
/// // Process events with 50ms tick rate
/// handler.run(Duration::from_millis(50)).await?;
///
/// // Check for events without blocking
/// while let Some(event) = handler.try_recv()? {
///     println!("Event: {:?}", event);
/// }
///
/// // Shutdown gracefully
/// handler.stop();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct EventHandler {
    /// Sender for event channel
    tx: Sender<Event>,
    /// Receiver for event channel
    rx: Receiver<Event>,
    /// Flag indicating if the event handler is running
    running: AtomicBool,
}

impl EventHandler {
    /// Creates a new event handler with a bounded channel.
    ///
    /// The channel capacity is limited by `MAX_EVENTS` to prevent memory
    /// exhaustion while maintaining reasonable event buffering.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::event::EventHandler;
    ///
    /// let handler = EventHandler::new();
    /// assert!(handler.is_running());
    /// ```
    pub fn new() -> Self {
        let (tx, rx) = bounded(MAX_EVENTS);
        Self {
            tx,
            rx,
            running: AtomicBool::new(true),
        }
    }

    /// Attempts to send an event through the channel without blocking.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to send through the channel
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if sent successfully, or an error if the channel
    /// is full or closed.
    ///
    /// # Errors
    ///
    /// Returns a `ChannelClosed` error if the channel has been closed.
    pub fn try_send(&self, event: Event) -> OxittyResult<()> {
        self.tx
            .try_send(event)
            .map_err(|_| OxittyError::channel_closed("event channel", (0, 0)).into())
    }

    /// Non-blocking attempt to receive an event from the channel.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Event))` - An event was available
    /// * `Ok(None)` - No events were ready
    /// * `Err(_)` - The channel has been closed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::event::EventHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = EventHandler::new();
    ///
    /// if let Some(event) = handler.try_recv()? {
    ///     println!("Received: {:?}", event);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_recv(&self) -> OxittyResult<Option<Event>> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(smol::channel::TryRecvError::Empty) => Ok(None),
            Err(_) => Err(OxittyError::channel_closed("event channel", (0, 0)).into()),
        }
    }

    /// Starts the event polling task.
    ///
    /// Runs an asynchronous loop that polls for terminal events and
    /// distributes them through the channel. The loop continues until
    /// `stop()` is called.
    ///
    /// # Arguments
    ///
    /// * `tick_rate` - Duration to wait between polling attempts
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when stopped cleanly, or an error if event
    /// polling fails.
    pub async fn run(&self, tick_rate: Duration) -> OxittyResult<()> {
        while self.running.load(Ordering::Acquire) {
            // Poll for crossterm events
            if self.poll_events(tick_rate)? {
                match self.read_event()? {
                    CrosstermEvent::Key(key) => {
                        self.try_send(Event::Key(key))?;
                    }
                    CrosstermEvent::Mouse(mouse) => {
                        self.try_send(Event::Mouse(mouse))?;
                    }
                    CrosstermEvent::Resize(width, height) => {
                        self.try_send(Event::Resize(width, height))?;
                    }
                    _ => {}
                }
            }

            // Allow other tasks to run
            smol::future::yield_now().await;
        }

        Ok(())
    }

    /// Polls for terminal events.
    ///
    /// # Arguments
    ///
    /// * `tick_rate` - Maximum duration to wait for an event
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - An event is available
    /// * `Ok(false)` - No event available within tick rate
    /// * `Err(_)` - Polling failed
    fn poll_events(&self, tick_rate: Duration) -> OxittyResult<bool> {
        crossterm::event::poll(tick_rate).map_err(|e| {
            OxittyError::terminal(
                "event polling",
                (0, 0),
                format!("Failed to poll events: {}", e),
            )
            .into()
        })
    }

    /// Reads a terminal event.
    ///
    /// # Returns
    ///
    /// Returns the next available terminal event or an error if
    /// reading fails.
    ///
    /// # Errors
    ///
    /// Returns a terminal error if reading fails.
    fn read_event(&self) -> OxittyResult<CrosstermEvent> {
        crossterm::event::read().map_err(|e| {
            OxittyError::terminal(
                "event reading",
                (0, 0),
                format!("Failed to read event: {}", e),
            )
            .into()
        })
    }

    /// Stops the event handler gracefully.
    ///
    /// Sets the running flag to false, which will cause the event
    /// polling loop to terminate after the current iteration.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Checks if the event handler is currently running.
    ///
    /// # Returns
    ///
    /// Returns true if the handler is running, false otherwise.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use smol::block_on;

    #[test]
    fn test_event_handler_lifecycle() {
        let handler = EventHandler::new();
        assert!(handler.is_running());

        handler.stop();
        assert!(!handler.is_running());
    }

    #[test]
    fn test_event_sending() {
        let handler = EventHandler::new();

        // Test key event
        let key_event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        assert!(handler.try_send(key_event).is_ok());

        // Test receiving the sent event
        let received = block_on(async { handler.try_recv() }).unwrap();

        assert!(matches!(received, Some(Event::Key(_))));
    }

    #[test]
    fn test_channel_capacity() {
        let handler = EventHandler::new();

        // Fill the channel to capacity
        for _ in 0..MAX_EVENTS {
            let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
            assert!(handler.try_send(event).is_ok());
        }

        // Next send should fail
        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        assert!(handler.try_send(event).is_err());
    }
}

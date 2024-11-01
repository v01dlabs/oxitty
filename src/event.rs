//! Event handling system
//!
//! This module provides a non-blocking, zero-copy event handling system
//! that processes terminal and custom events asynchronously.

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

/// Maximum number of pending events in the channel
const MAX_EVENTS: usize = 1024;

/// Terminal events that can occur
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press events
    Key(KeyEvent),
    /// Mouse interaction events
    Mouse(MouseEvent),
    /// Terminal resize events
    Resize(u16, u16),
    /// Custom events for application-specific needs
    Custom(Box<dyn CloneableAny + Send>),
    /// Event indicating the event loop should terminate
    Quit,
}

/// A trait for cloning `Any` trait objects
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

/// Handles event processing and distribution
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
    /// Creates a new event handler with bounded channel
    pub fn new() -> Self {
        let (tx, rx) = bounded(MAX_EVENTS);
        Self {
            tx,
            rx,
            running: AtomicBool::new(true),
        }
    }

    /// Attempts to send an event through the channel without blocking
    pub fn try_send(&self, event: Event) -> OxittyResult<()> {
        self.tx
            .try_send(event)
            .map_err(|_| OxittyError::channel_closed("event channel", (0, 0)).into())
    }

    /// Non-blocking attempt to receive an event
    pub fn try_recv(&self) -> OxittyResult<Option<Event>> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(smol::channel::TryRecvError::Empty) => Ok(None),
            Err(_) => Err(OxittyError::channel_closed("event channel", (0, 0)).into()),
        }
    }

    /// Starts the event polling task
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

    /// Polls for terminal events
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

    /// Reads a terminal event
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

    /// Stops the event handler
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Checks if the event handler is running
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

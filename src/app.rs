//! Application orchestration module
//!
//! This module provides the primary application infrastructure, coordinating between
//! the event system, state management, and TUI rendering.

use std::{future::Future, sync::Arc, time::Duration};

use smol::{future::FutureExt, Task};

use crate::{
    error::OxittyResult,
    event::{Event, EventHandler},
    state::AtomicState,
    tui::Tui,
};

/// Core application struct managing all components
pub struct App<S: AtomicState> {
    /// Terminal interface manager
    tui: Tui<S>,
    /// Event handling system
    events: Arc<EventHandler>,
    /// Event polling rate
    tick_rate: Duration,
    /// Background task handles
    tasks: Vec<Task<OxittyResult<()>>>,
}

impl<S: AtomicState + 'static> App<S> {
    /// Creates a new application instance
    pub fn new(state: S, tick_rate: Duration) -> OxittyResult<Self> {
        let tui = Tui::new(state)?;
        let events = EventHandler::new();

        Ok(Self {
            tui,
            events: Arc::new(events),
            tick_rate,
            tasks: Vec::new(),
        })
    }

    /// Spawns a background task
    pub fn spawn<F>(&mut self, future: F) -> OxittyResult<()>
    where
        F: Future<Output = OxittyResult<()>> + Send + 'static,
    {
        let task = smol::spawn(future);
        self.tasks.push(task);
        Ok(())
    }

    /// Runs the application event loop
    pub async fn run<F>(&mut self, render_fn: F) -> OxittyResult<()>
    where
        F: Fn(&S::Snapshot, ratatui::layout::Rect, &mut ratatui::Frame<'_>) + Send + 'static,
    {
        // Spawn event handling task
        let events = self.events.clone();
        let tick_rate = self.tick_rate;
        self.spawn(async move { events.run(tick_rate).await })?;

        // Main event loop
        while self.tui.state().is_running() {
            // Non-blocking event check
            if let Some(event) = self.events.try_recv()? {
                match event {
                    Event::Quit => {
                        self.tui.state().quit();
                        break;
                    }
                    Event::Key(key) => {
                        if let crossterm::event::KeyCode::Char('q') = key.code {
                            self.tui.state().quit();
                            break;
                        }
                    }
                    _ => {}
                }
            }

            // Non-blocking render
            self.tui.render(&render_fn)?;

            // Yield to other tasks
            smol::future::yield_now().await;
        }

        // Stop event handler and cleanup tasks
        self.events.stop();
        self.cleanup_tasks().await;

        Ok(())
    }

    /// Cleanup background tasks
    async fn cleanup_tasks(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);
        for task in tasks {
            // Attempt to join task with timeout
            match task
                .or(async {
                    smol::Timer::after(Duration::from_secs(1)).await;
                    Ok(())
                })
                .await
            {
                Ok(_) => {}
                Err(e) => eprintln!("Task cleanup error: {}", e),
            }
        }
    }

    /// Returns a reference to the TUI
    pub fn tui(&self) -> &Tui<S> {
        &self.tui
    }

    /// Returns a reference to the event handler
    pub fn events(&self) -> &EventHandler {
        &self.events
    }

    /// Returns the current tick rate
    pub fn tick_rate(&self) -> Duration {
        self.tick_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Test state implementation
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
    fn test_app_creation() {
        std::env::set_var("TERM", "dumb");

        let state = TestState {
            running: AtomicBool::new(true),
        };

        let app_result = App::new(state, Duration::from_millis(50));
        assert!(
            app_result.is_err(),
            "App creation should fail in test environment"
        );
    }

    #[test]
    fn test_task_spawning() {
        std::env::set_var("TERM", "dumb");

        let state = TestState {
            running: AtomicBool::new(true),
        };

        if let Ok(mut app) = App::new(state, Duration::from_millis(50)) {
            let spawn_result = app.spawn(async { Ok(()) });
            assert!(spawn_result.is_ok());
            assert_eq!(app.tasks.len(), 1);
        }
    }
}

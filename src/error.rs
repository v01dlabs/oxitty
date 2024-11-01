//! Error handling infrastructure for Oxitty.
//!
//! This module provides a comprehensive error handling system built on top of the
//! [miette] crate. It defines custom error types specific to terminal operations,
//! IO operations, initialization, events, and channel communication.
//!
//! # Error Types
//!
//! The module primarily revolves around the [`OxittyError`] enum, which encompasses
//! all possible error scenarios in the application. Each variant provides detailed
//! context including source code location and descriptive messages.
//!
//! # Examples
//!
//! ```
//! use oxitty::error::{OxittyError, OxittyResult};
//!
//! fn simulate_terminal_init(should_fail: bool) -> OxittyResult<()> {
//!     if should_fail {
//!         return Err(OxittyError::terminal(
//!             "terminal initialization",
//!             (0, 10),
//!             "Failed to initialize terminal"
//!         ).into());
//!     }
//!     Ok(())
//! }
//!
//! // Example usage:
//! let success_result = simulate_terminal_init(false);
//! assert!(success_result.is_ok());
//!
//! let error_result = simulate_terminal_init(true);
//! assert!(error_result.is_err());
//! ```
//!
//! For operations that might fail, use the [`OxittyResult`] type alias:
//!
//! ```
//! use oxitty::error::OxittyResult;
//!
//! fn perform_io_operation(succeed: bool) -> OxittyResult<String> {
//!     if succeed {
//!         Ok("success".to_string())
//!     } else {
//!         Err(oxitty::error::OxittyError::io(
//!             "file operation",
//!             (0, 10),
//!             "Simulated IO error"
//!         ).into())
//!     }
//! }
//!
//! // Example usage:
//! let success = perform_io_operation(true);
//! assert!(success.is_ok());
//! assert_eq!(success.unwrap(), "success");
//!
//! let failure = perform_io_operation(false);
//! assert!(failure.is_err());
//! ```

use miette::{Diagnostic, SourceSpan};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};

/// Custom error types for the Oxitty application.
///
/// This enum implements the [`Diagnostic`] trait from miette, providing rich
/// error reporting capabilities including source code context and error spans.
///
/// # Examples
///
/// Creating a terminal error:
/// ```
/// use oxitty::error::OxittyError;
///
/// let error = OxittyError::terminal(
///     "terminal setup",
///     (0, 10),
///     "Failed to enter alternate screen"
/// );
/// ```
#[derive(Debug, Diagnostic)]
pub enum OxittyError {
    /// Represents errors related to terminal operations.
    ///
    /// This variant is used when terminal-specific operations fail, such as
    /// entering alternate screen, enabling raw mode, or terminal rendering.
    ///
    /// # Fields
    /// * `src` - The source code context where the error occurred
    /// * `err_span` - The span in the source code pointing to the error location
    /// * `msg` - A detailed error message describing what went wrong
    #[diagnostic(code(oxitty::terminal), url(docsrs))]
    Terminal {
        #[source_code]
        src: String,
        #[label("error occurred here")]
        err_span: SourceSpan,
        msg: String,
    },

    /// Represents Input/Output operation errors.
    ///
    /// Used for file operations, network operations, or any other I/O related failures.
    ///
    /// # Fields
    /// * `src` - The source code context where the error occurred
    /// * `err_span` - The span in the source code pointing to the error location
    /// * `msg` - A detailed error message describing what went wrong
    #[diagnostic(code(oxitty::io), url(docsrs))]
    Io {
        #[source_code]
        src: String,
        #[label("io error occurred here")]
        err_span: SourceSpan,
        msg: String,
    },

    /// Represents initialization errors.
    ///
    /// Used when the application fails to initialize properly, such as loading
    /// configuration files or setting up required resources.
    ///
    /// # Fields
    /// * `path` - The file path related to the initialization error
    /// * `src` - The source code context where the error occurred
    /// * `err_span` - The span in the source code pointing to the error location
    /// * `msg` - A detailed error message describing what went wrong
    #[diagnostic(code(oxitty::init), url(docsrs))]
    InitError {
        path: PathBuf,
        #[source_code]
        src: String,
        #[label("initialization failed here")]
        err_span: SourceSpan,
        msg: String,
    },

    /// Represents event system errors.
    ///
    /// Used when errors occur in the event handling system, such as failed event
    /// dispatch or invalid event data.
    ///
    /// # Fields
    /// * `src` - The source code context where the error occurred
    /// * `err_span` - The span in the source code pointing to the error location
    /// * `msg` - A detailed error message describing what went wrong
    #[diagnostic(code(oxitty::event), url(docsrs))]
    Event {
        #[source_code]
        src: String,
        #[label("event error occurred here")]
        err_span: SourceSpan,
        msg: String,
    },

    /// Represents channel communication errors.
    ///
    /// Used when a channel is closed unexpectedly during communication between
    /// different parts of the application.
    ///
    /// # Fields
    /// * `src` - The source code context where the error occurred
    /// * `err_span` - The span in the source code pointing to the error location
    #[diagnostic(code(oxitty::channel), url(docsrs))]
    ChannelClosed {
        #[source_code]
        src: String,
        #[label("channel closed")]
        err_span: SourceSpan,
    },
}

/// A type alias for Results using OxittyError.
///
/// This type alias simplifies the use of Result types throughout the application
/// by incorporating the application's custom error type.
///
/// # Examples
///
/// ```
/// use oxitty::error::OxittyResult;
///
/// fn some_operation() -> OxittyResult<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type OxittyResult<T> = miette::Result<T>;

impl Display for OxittyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            OxittyError::Terminal { msg, .. } => write!(f, "Terminal error: {}", msg),
            OxittyError::Io { msg, .. } => write!(f, "IO error: {}", msg),
            OxittyError::InitError { msg, .. } => write!(f, "Initialization error: {}", msg),
            OxittyError::Event { msg, .. } => write!(f, "Event error: {}", msg),
            OxittyError::ChannelClosed { .. } => write!(f, "Channel closed"),
        }
    }
}

impl Error for OxittyError {}

impl OxittyError {
    /// Creates a new terminal error.
    ///
    /// # Arguments
    ///
    /// * `src` - Source code context where the error occurred
    /// * `err_span` - Location in the source code where the error occurred
    /// * `msg` - Detailed error message
    ///
    /// # Examples
    ///
    /// ```
    /// use oxitty::error::OxittyError;
    ///
    /// let error = OxittyError::terminal(
    ///     "terminal setup",
    ///     (0, 10),
    ///     "Failed to enter alternate screen"
    /// );
    /// ```
    pub fn terminal(
        src: impl Into<String>,
        err_span: impl Into<SourceSpan>,
        msg: impl Into<String>,
    ) -> Self {
        Self::Terminal {
            src: src.into(),
            err_span: err_span.into(),
            msg: msg.into(),
        }
    }

    /// Creates a new IO error.
    ///
    /// # Arguments
    ///
    /// * `src` - Source code context where the error occurred
    /// * `err_span` - Location in the source code where the error occurred
    /// * `msg` - Detailed error message
    ///
    /// # Examples
    ///
    /// ```
    /// use oxitty::error::OxittyError;
    ///
    /// let error = OxittyError::io(
    ///     "file operation",
    ///     (5, 15),
    ///     "Failed to read configuration file"
    /// );
    /// ```
    pub fn io(
        src: impl Into<String>,
        err_span: impl Into<SourceSpan>,
        msg: impl Into<String>,
    ) -> Self {
        Self::Io {
            src: src.into(),
            err_span: err_span.into(),
            msg: msg.into(),
        }
    }

    /// Creates a new initialization error.
    ///
    /// # Arguments
    ///
    /// * `path` - File path related to the initialization error
    /// * `src` - Source code context where the error occurred
    /// * `err_span` - Location in the source code where the error occurred
    /// * `msg` - Detailed error message
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use oxitty::error::OxittyError;
    ///
    /// let error = OxittyError::init(
    ///     PathBuf::from("/config/app.conf"),
    ///     "initialization",
    ///     (0, 10),
    ///     "Failed to load configuration"
    /// );
    /// ```
    pub fn init(
        path: impl Into<PathBuf>,
        src: impl Into<String>,
        err_span: impl Into<SourceSpan>,
        msg: impl Into<String>,
    ) -> Self {
        Self::InitError {
            path: path.into(),
            src: src.into(),
            err_span: err_span.into(),
            msg: msg.into(),
        }
    }

    /// Creates a new event error.
    ///
    /// # Arguments
    ///
    /// * `src` - Source code context where the error occurred
    /// * `err_span` - Location in the source code where the error occurred
    /// * `msg` - Detailed error message
    ///
    /// # Examples
    ///
    /// ```
    /// use oxitty::error::OxittyError;
    ///
    /// let error = OxittyError::event(
    ///     "event handling",
    ///     (15, 25),
    ///     "Invalid event data received"
    /// );
    /// ```
    pub fn event(
        src: impl Into<String>,
        err_span: impl Into<SourceSpan>,
        msg: impl Into<String>,
    ) -> Self {
        Self::Event {
            src: src.into(),
            err_span: err_span.into(),
            msg: msg.into(),
        }
    }

    /// Creates a new channel closed error.
    ///
    /// # Arguments
    ///
    /// * `src` - Source code context where the error occurred
    /// * `err_span` - Location in the source code where the error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use oxitty::error::OxittyError;
    ///
    /// let error = OxittyError::channel_closed(
    ///     "event channel",
    ///     (20, 30)
    /// );
    /// ```
    pub fn channel_closed(src: impl Into<String>, err_span: impl Into<SourceSpan>) -> Self {
        Self::ChannelClosed {
            src: src.into(),
            err_span: err_span.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_error_creation() {
        let err = OxittyError::terminal(
            "terminal init".to_string(),
            (0, 12),
            "failed to initialize terminal".to_string(),
        );

        match err {
            OxittyError::Terminal { src, err_span, msg } => {
                assert_eq!(src, "terminal init");
                assert_eq!(err_span, (0, 12).into());
                assert_eq!(msg, "failed to initialize terminal");
            }
            _ => panic!("Wrong error variant"),
        }
    }
}

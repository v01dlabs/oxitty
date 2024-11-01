use miette::{Diagnostic, SourceSpan};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};

#[derive(Debug, Diagnostic)]
pub enum OxittyError {
    #[diagnostic(code(oxitty::terminal), url(docsrs))]
    Terminal {
        #[source_code]
        src: String,
        #[label("error occurred here")]
        err_span: SourceSpan,
        msg: String,
    },

    #[diagnostic(code(oxitty::io), url(docsrs))]
    Io {
        #[source_code]
        src: String,
        #[label("io error occurred here")]
        err_span: SourceSpan,
        msg: String,
    },

    #[diagnostic(code(oxitty::init), url(docsrs))]
    InitError {
        path: PathBuf,
        #[source_code]
        src: String,
        #[label("initialization failed here")]
        err_span: SourceSpan,
        msg: String,
    },

    #[diagnostic(code(oxitty::event), url(docsrs))]
    Event {
        #[source_code]
        src: String,
        #[label("event error occurred here")]
        err_span: SourceSpan,
        msg: String,
    },

    #[diagnostic(code(oxitty::channel), url(docsrs))]
    ChannelClosed {
        #[source_code]
        src: String,
        #[label("channel closed")]
        err_span: SourceSpan,
    },
}

pub type OxittyResult<T> = miette::Result<T>;

// Helper functions to create errors with context
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

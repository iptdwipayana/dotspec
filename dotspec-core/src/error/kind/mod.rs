mod io;
mod semantic;
mod syntax;
mod traits;

use std::borrow::Cow;

pub use io::Error as IoError;
pub use semantic::Error as SemanticError;
pub use syntax::Error as SyntaxError;
pub use traits::ErrorCode;
pub use traits::ErrorDescription;
pub use traits::ErrorInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Io(IoError),
    Semantic(SemanticError),
    Syntax(SyntaxError),
}

impl ErrorCode for ErrorKind {
    fn code(&self) -> &'static str {
        match self {
            Self::Io(e) => e.code(),
            Self::Semantic(e) => e.code(),
            Self::Syntax(e) => e.code(),
        }
    }
}

impl ErrorDescription for ErrorKind {
    fn message(&self) -> Cow<'static, str> {
        match self {
            Self::Io(e) => e.message(),
            Self::Semantic(e) => e.message(),
            Self::Syntax(e) => e.message(),
        }
    }

    fn label(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::Io(e) => e.label(),
            Self::Semantic(e) => e.label(),
            Self::Syntax(e) => e.label(),
        }
    }

    fn help(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::Io(e) => e.help(),
            Self::Semantic(e) => e.help(),
            Self::Syntax(e) => e.help(),
        }
    }
}

impl From<IoError> for ErrorKind {
    fn from(value: IoError) -> Self {
        Self::Io(value)
    }
}
impl From<SemanticError> for ErrorKind {
    fn from(value: SemanticError) -> Self {
        Self::Semantic(value)
    }
}
impl From<SyntaxError> for ErrorKind {
    fn from(value: SyntaxError) -> Self {
        Self::Syntax(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kinds_delegates_to_inner_error() {
        let kind: ErrorKind = IoError::FileNotFound.into();
        assert_eq!(kind.code(), "IO001");
        let kind: ErrorKind = SemanticError::DuplicatePath { path: String::from("foo") }.into();
        assert_eq!(kind.code(), "SEM001");
        let kind: ErrorKind = SyntaxError::UnclosedSection.into();
        assert_eq!(kind.code(), "SYN001");
    }

    #[test]
    fn some_error_kinds_has_label() {
        let kinds: Vec<ErrorKind> = vec![
            SemanticError::DuplicatePath { path: String::from("foo") }.into(),
            SyntaxError::UnclosedSection.into(),
        ];
        for kind in kinds {
            assert!(kind.label().is_some(), "Expected label to return Some: {kind:?}");
        }
    }

    #[test]
    fn some_error_kinds_has_no_label() {
        let kinds: Vec<ErrorKind> = vec![
            IoError::FileNotFound.into(),
        ];
        for kind in kinds {
            assert!(kind.label().is_none(), "Expected label to return None: {kind:?}");
        }
    }
}
mod io;
mod semantic;
mod syntax;
mod traits;

pub use io::Error as IoError;
pub use semantic::Error as SemanticError;
pub use syntax::Error as SyntaxError;
pub use traits::{
    ErrorCode,
    ErrorDescription,
    ErrorInfo,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Io(IoError),
    Semantic(SemanticError),
    Syntax(SyntaxError),
}

impl ErrorCode for ErrorKind {
    fn code(&self) -> &'static str {
        match self {
            Self::Io(io) => io.code(),
            Self::Semantic(sem) => sem.code(),
            Self::Syntax(syn) => syn.code(),
        }
    }
}

impl ErrorDescription for ErrorKind {
    fn message(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Io(io) => io.message(),
            Self::Semantic(sem) => sem.message(),
            Self::Syntax(syn) => syn.message(),
        }
    }

    fn label(&self) -> Option<std::borrow::Cow<'static, str>> {
        match self {
            Self::Io(io) => io.label(),
            Self::Semantic(sem) => sem.label(),
            Self::Syntax(syn) => syn.label(),
        }
    }
    fn help(&self) -> Option<std::borrow::Cow<'static, str>> {
        match self {
            Self::Io(io) => io.help(),
            Self::Semantic(sem) => sem.help(),
            Self::Syntax(syn) => syn.help(),
        }
    }
}

impl From<IoError> for ErrorKind {
    fn from(value: IoError) -> Self { Self::Io(value) }
}
impl From<SemanticError> for ErrorKind {
    fn from(value: SemanticError) -> Self { Self::Semantic(value) }
}
impl From<SyntaxError> for ErrorKind {
    fn from(value: SyntaxError) -> Self { Self::Syntax(value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn err_kind_delegates_to_the_inner_error() {
        let kind: ErrorKind = IoError::FileNotFound.into();
        assert_eq!(kind.code(), "IO000");
        assert_eq!(kind.message(), "file not found");

        let kind: ErrorKind = SemanticError::DuplicatePath { path: "foo".into() }.into();
        assert_eq!(kind.code(), "SEM000");
        assert_eq!(kind.message(), "duplicate path `foo`");

        let kind: ErrorKind = SyntaxError::UnclosedSection.into();
        assert_eq!(kind.code(), "SYN000");
        assert_eq!(kind.message(), "unclosed section");
    }

    #[test]
    fn sem_and_syn_err_has_label() {
        let kind: ErrorKind = SemanticError::DuplicatePath { path: "foo".into() }.into();
        assert_eq!(kind.code(), "SEM000");
        assert_eq!(kind.label(), Some(std::borrow::Cow::Borrowed("already defined")));

        let kind: ErrorKind = SyntaxError::UnclosedSection.into();
        assert_eq!(kind.code(), "SYN000");
        assert_eq!(kind.label(), Some(std::borrow::Cow::Borrowed("expect a closing `]` on section header")));
    }

    #[test]
    fn io_err_has_no_label() {
        let kind: ErrorKind = IoError::FileNotFound.into();
        assert_eq!(kind.code(), "IO000");
        assert_eq!(kind.label(), None);
    }
}
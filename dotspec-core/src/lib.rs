mod error;
mod span;

pub use span::Span;
pub use error::{
    ErrorKind,
    IoError,
    SemanticError,
    SyntaxError,
    Excerpt,
    Diagnostic,
    ErrorCode,
    ErrorDescription,
    ErrorInfo
};
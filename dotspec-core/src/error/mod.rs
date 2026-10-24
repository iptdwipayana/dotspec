mod diagnostic;
mod excerpt;
mod kind;

pub use diagnostic::{Diagnostic};
pub use excerpt::{Excerpt};
pub use kind::{
    ErrorKind,
    IoError,
    SemanticError,
    SyntaxError,
    ErrorCode,
    ErrorDescription,
    ErrorInfo
};
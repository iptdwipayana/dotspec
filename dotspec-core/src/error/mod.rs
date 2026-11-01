mod diagnostic;
mod excerpt;
mod kind;

pub use diagnostic::Diagnostic;
pub use excerpt::Excerpt;
pub use kind::Kind;
pub use kind::IoError;
pub use kind::SemanticError;
pub use kind::SyntaxError;
pub use kind::ErrorCode;
pub use kind::ErrorDescription;
pub use kind::ErrorInfo;
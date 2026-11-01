mod error;
mod span;

pub use span::Span;
pub use error::Diagnostic;
pub use error::Excerpt;
pub use error::Kind;
pub use error::IoError;
pub use error::SemanticError;
pub use error::SyntaxError;
pub use error::ErrorCode;
pub use error::ErrorDescription;
pub use error::ErrorInfo;
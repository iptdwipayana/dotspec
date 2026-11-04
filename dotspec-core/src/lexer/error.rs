use crate::span::Span;
use crate::error::SyntaxError;

#[derive(Debug)]
pub(super) struct Error {
    pub kind: SyntaxError,
    pub span: Span,
}

impl Error {
    pub(super) fn new(kind: SyntaxError, span: Span) -> Self {
        Self { kind, span }
    }
}
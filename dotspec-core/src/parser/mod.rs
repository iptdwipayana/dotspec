mod entry;
mod stream;
mod value;

use crate::error::Diagnostic;
use crate::source::Source;
use crate::span::Span;

use stream::TokenStream;

pub(crate) struct Parser<'a> {
    text: &'a str,
    stream: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text, stream: TokenStream::new(text) }
    }

    fn error(&self, kind: crate::error::SyntaxError, span: Span) -> Diagnostic {
        Source::new(self.text).diagnostic(kind, span)
    }
}
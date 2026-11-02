use crate::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Dot,
    Identifier,
    Whitespace,
    Newline,
    Comment,
    Unknown,
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span,
    pub text: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, span: Span, text: &'a str) -> Self {
        Self { kind, span, text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_kind_text_and_span() {
        let token = Token::new(TokenKind::Identifier, Span::new(1, 1, 3), "abc");
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.span, Span::new(1, 1, 3));
        assert_eq!(token.text, "abc");
    }

    #[test]
    fn is_copy() {
        let a = Token::new(TokenKind::Dot, Span::new(1, 1, 1), ".");
        let b = a;
        assert_eq!(a, b);
    }
}
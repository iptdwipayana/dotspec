use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind<'a> {
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Dot,
    Integer,
    Float,
    String { fragments: Vec<super::string::Fragment<'a>> }, 
    Identifier,
    Whitespace,
    Newline,
    Comment,
    Unknown,
    Eof,
}

impl TokenKind<'_> {
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
    pub text: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, span: Span, text: &'a str) -> Self {
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
    fn integer_and_float_are_different_kinds() {
        let integer = Token::new(TokenKind::Integer, Span::new(1, 1, 1), "1");
        let float = Token::new(TokenKind::Float, Span::new(1, 1, 3), "1.0");
        assert_ne!(integer.kind, float.kind);
    }

    #[test]
    fn a_string_kind_keeps_its_fragments() {
        use super::super::interpolation::Interpolation;
        use super::super::string::Fragment;

        let fragments = vec![
            Fragment::Escape('\n', Span::new(1, 2, 2)),
            Fragment::Text(std::borrow::Cow::Borrowed("abc"), Span::new(1, 3, 3)),
            Fragment::Interpolation(
                Interpolation::Env {
                    path: "foo",
                    default: Some("bar"),
                },
                Span::new(1, 4, 4)
            )
        ];

        let token = Token::new(
            TokenKind::String { fragments: fragments.clone() }, 
            Span::new(1, 1, 5),
            "\"\\nabc#{env.FOO}\""
        );

        assert!(token.kind.is_string());
        assert_eq!(token.kind, TokenKind::String { fragments: fragments.clone() });
    }

    #[test]
    fn every_string_kind_reports_its_span_from_their_fragments() {
        use super::super::interpolation::Interpolation;
        use super::super::string::Fragment;

        let span = Span::new(2, 3, 4);
        assert_eq!(Fragment::Escape('a', span).span(), span);
        assert_eq!(Fragment::Text(std::borrow::Cow::Borrowed("ab"), span).span(), span);
        assert_eq!(Fragment::Interpolation(Interpolation::Config { path: "a" }, span).span(), span);
    }

    #[test]
    fn only_string_kind_is_a_string() {
        assert_eq!(TokenKind::Identifier.is_string(), false);
        assert_eq!(TokenKind::String { fragments: Vec::new() }.is_string(), true);
    }
}
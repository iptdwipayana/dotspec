mod array;
mod entry;
mod object;
mod stream;
mod value;

#[cfg(test)]
mod testing;

use crate::error::Diagnostic;
use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::source::Source;
use crate::span::Span;

use stream::TokenStream;

/// Arrays and objects may nest at most this deep, counted together.
/// Without a limit, input like `[[[[…` would overflow the stack and abort.
const MAX_DEPTH: usize = 128;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Spacing {
    Any,
    Inline,
}

pub(crate) struct Parser<'a> {
    text: &'a str,
    stream: TokenStream<'a>,
    depth: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text, stream: TokenStream::new(text), depth: 0 }
    }

    fn error(&self, kind: crate::error::SyntaxError, span: Span) -> Diagnostic {
        Source::new(self.text).diagnostic(kind, span)
    }

    fn skip(&mut self, spacing: Spacing) -> Result<(), Diagnostic> {
        match spacing {
            Spacing::Any => self.stream.skip_spacing(),
            Spacing::Inline => self.stream.skip_inline_space(),
        }
    }

    fn enter(&mut self, open: &Token<'a>) -> Result<(), Diagnostic> {
        if self.depth >= MAX_DEPTH {
            return Err(self.error(crate::error::SyntaxError::NestingTooDeep, open.span));
        }
        self.depth += 1;
        Ok(())
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }

    fn span_since(&self, start: Span, before: usize) -> Span {
        Span::new(start.line, start.column, self.stream.consumed() - before)
    }

    fn next_starts_value(&mut self) -> Result<bool, Diagnostic> {
        Ok(matches!(
            self.stream.peek()?.kind,
            TokenKind::Integer | TokenKind::Float
                | TokenKind::Identifier
                | TokenKind::LeftBracket
                | TokenKind::LeftBrace
                | TokenKind::String { .. }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Entry;
    use crate::ast::Expr;
    use super::testing::{show, syntax_error};

    fn parse(text: &str) -> Expr<'_> {
        Parser::new(text)
            .parse_value()
            .unwrap_or_else(|e| panic!("parse error in {text:?}: {e:?}"))
    }

    fn failure(text: &str) -> (crate::error::SyntaxError, Span) {
        let diagnostic = Parser::new(text)
            .parse_value()
            .err()
            .unwrap_or_else(|| panic!("expected a parse error for {text:?}"));
        syntax_error(&diagnostic)
    }

    fn entry(text: &str) -> Entry<'_> {
        Parser::new(text)
            .parse_entry()
            .unwrap_or_else(|e| panic!("parse error in {text:?}: {e:?}"))
    }

    fn array(levels: usize) -> String {
        format!("{}1{}", "[".repeat(levels), "]".repeat(levels))
    }

    /// `pairs` arrays and `pairs` objects, nested in turn: `[{a:[{a:1}]}]`.
    fn alternating(pairs: usize) -> String {
        format!("{}1{}", "[{a:".repeat(pairs), "}]".repeat(pairs))
    }

    #[test]
    fn nesting_up_to_the_limit_is_allowed() {
        assert_eq!(parse(&array(MAX_DEPTH)).span, Span::new(1, 1, 257));
    }

    #[test]
    fn one_level_past_the_limit_points_at_that_bracket() {
        assert_eq!(
            failure(&array(MAX_DEPTH + 1)),
            (crate::error::SyntaxError::NestingTooDeep, Span::new(1, 129, 1))
        );
    }

    #[test]
    fn arrays_and_objects_count_together() {
        assert_eq!(parse(&alternating(64)).span, Span::new(1, 1, 385));

        let too_deep = format!("{}[1]", "[{a:".repeat(64));
        assert_eq!(
            failure(&too_deep),
            (crate::error::SyntaxError::NestingTooDeep, Span::new(1, 257, 1))
        );
    }

    #[test]
    fn an_object_past_the_limit_is_rejected_too() {
        let text = format!("{}{{", "[".repeat(MAX_DEPTH));
        assert_eq!(
            failure(&text),
            (crate::error::SyntaxError::NestingTooDeep, Span::new(1, 129, 1))
        );
    }

    #[test]
    fn siblings_do_not_add_up() {
        let text = format!("[{}]", "[1],".repeat(300));
        assert_eq!(show(&parse("[[1], [2]]")), "[[1], [2]]");
        assert!(Parser::new(&text).parse_value().is_ok());

        let objects = format!("[{}]", "{a: 1},".repeat(300));
        assert!(Parser::new(&objects).parse_value().is_ok());
    }

    const MULTILINE: &str = "multiline: [\n  \"alpha\",        # comments are allowed here\n  \"beta\",\n  \"gamma\",        # trailing comma is allowed\n]";
    const PRIMARY: &str = "primary: {\n  host: \"db1.internal\",   # comment inside an object\n  port: 5432,\n  tls: true,\n}";
    const REPLICAS: &str = "replicas: [\n  { host: \"r1\", port: 5432 },\n  { host: \"r2\", port: 5433 },\n]";

    #[test]
    fn the_single_line_collections_of_example_spec_parse() {
        let table = [
            ("empty: []", "[]"),
            ("ports: [80, 443, 8080]", "[80, 443, 8080]"),
            ("mixed: [\"a\", 1, true, 2.5]", "[\"a\", 1, true, 2.5]"),
            ("nested: [[1, 2], [3, 4]]", "[[1, 2], [3, 4]]"),
            ("trailing: [1, 2, 3,]", "[1, 2, 3]"),
            ("empty: {}", "{}"),
            (
                "inline: { host: \"localhost\", port: 5432 }",
                "{host: \"localhost\", port: 5432}",
            ),
        ];
        for (text, expected) in table {
            assert_eq!(show(&entry(text).value), expected, "{text}");
        }
    }

    #[test]
    fn the_multiline_collections_of_example_spec_parse() {
        let table = [
            (MULTILINE, "[\"alpha\", \"beta\", \"gamma\"]"),
            (PRIMARY, "{host: \"db1.internal\", port: 5432, tls: true}"),
            (
                REPLICAS,
                "[{host: \"r1\", port: 5432}, {host: \"r2\", port: 5433}]",
            ),
        ];
        for (text, expected) in table {
            assert_eq!(show(&entry(text).value), expected, "{text}");
        }
    }

    #[test]
    fn a_multiline_entry_spans_every_character() {
        for text in [MULTILINE, PRIMARY, REPLICAS] {
            assert_eq!(
                entry(text).span,
                Span::new(1, 1, text.chars().count()),
                "{text}"
            );
        }
    }
}
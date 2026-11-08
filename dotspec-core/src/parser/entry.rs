use crate::ast::Entry;
use crate::ast::Key;
use crate::error::Diagnostic;
use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::span::Span;
use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_entry(&mut self) -> Result<Entry<'a>, Diagnostic> {
        let key = self.parse_key()?;
        let colon = self.expect_colon(&key)?;

        self.stream.skip_inline_space()?;
        if self.value_is_missing()? {
            return Err(self.error(crate::error::SyntaxError::MissingValue, after(colon.span)));
        }

        let value = self.parse_value()?;
        let span = cover(key.span, value.span);
        Ok(Entry { key, value, span })
    }

    fn parse_key(&mut self) -> Result<Key<'a>, Diagnostic> {
        let token = self.stream.advance()?;
        match token.kind {
            TokenKind::Identifier => Ok(Key { name: token.text, span: token.span }),
            _ => Err(self.error(crate::error::SyntaxError::InvalidStatementStart, token.span)),
        }
    }

    fn expect_colon(&mut self, key: &Key<'a>) -> Result<Token<'a>, Diagnostic> {
        self.stream.skip_inline_space()?;
        if matches!(self.stream.peek()?.kind, TokenKind::Colon) {
            return self.stream.advance();
        }
        Err(self.error(crate::error::SyntaxError::MissingColon, after(key.span)))
    }

    fn value_is_missing(&mut self) -> Result<bool, Diagnostic> {
        Ok(matches!(
            self.stream.peek()?.kind,
            TokenKind::Newline | TokenKind::Comment | TokenKind::Eof
        ))
    }
}

fn after(span: Span) -> Span {
    Span::new_point(span.line, span.column + span.length)
}

fn cover(start: Span, end: Span) -> Span {
    debug_assert!(start.line == end.line && start.column <= end.column);
    Span::new(
        start.line,
        start.column,
        end.column - start.column + end.length
    )
} 

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::ast::ExprKind;
    use crate::error::Kind;
    use crate::lexer::Fragment;

    use crate::error::SyntaxError;

    fn entry(text: &str) -> Entry<'_> {
        Parser::new(text)
            .parse_entry()
            .unwrap_or_else(|e| panic!("parse error in {text:?}: {e:?}"))
    }

    fn diagnostic(text: &str) -> Diagnostic {
        Parser::new(text)
            .parse_entry()
            .err()
            .unwrap_or_else(|| panic!("expected a parse error for {text:?}"))
    }

    fn failure(text: &str) -> (SyntaxError, Span) {
        let diagnostic = diagnostic(text);
        let Kind::Syntax(error) = diagnostic.kind().clone() else {
            panic!("expected a syntax error for {text:?}");
        };
        (error, diagnostic.excerpt().expect("has an excerpt").span())
    }

    #[test]
    fn the_roadmap_example_becomes_an_entry_with_spans() {
        let entry = entry("debug: false");
        assert_eq!(entry.key.name, "debug");
        assert_eq!(entry.key.span, Span::new(1, 1, 5));
        assert_eq!(entry.value.kind, ExprKind::Boolean(false));
        assert_eq!(entry.value.span, Span::new(1, 8, 5));
        assert_eq!(entry.span, Span::new(1, 1, 12));
    }

    #[test]
    fn spaces_around_the_colon_are_allowed() {
        let entry = entry("timeout : 2.5");
        assert_eq!(entry.key.span, Span::new(1, 1, 7));
        assert_eq!(entry.value.kind, ExprKind::Float(2.5));
        assert_eq!(entry.value.span, Span::new(1, 11, 3));
        assert_eq!(entry.span, Span::new(1, 1, 13));
    }

    #[test]
    fn tabs_around_the_colon_are_allowed() {
        let entry = entry("a\t:\t1");
        assert_eq!(entry.value.kind, ExprKind::Integer(1));
        assert_eq!(entry.span, Span::new(1, 1, 5));
    }

    #[test]
    fn no_space_after_the_colon_is_fine() {
        assert_eq!(entry("a:1").value.kind, ExprKind::Integer(1));
    }

    #[test]
    fn keys_may_use_dashes_underscores_and_digits() {
        assert_eq!(entry("app-name: 1").key.name, "app-name");
        assert_eq!(entry("_a1: 1").key.name, "_a1");
    }

    #[test]
    fn numbers_are_converted() {
        assert_eq!(entry("big: 1_000").value.kind, ExprKind::Integer(1000));
        assert_eq!(entry("offset: -5").value.kind, ExprKind::Integer(-5));
    }

    #[test]
    fn a_string_value_keeps_its_fragments() {
        let ExprKind::String(fragments) = entry("name: \"a#{b}\"").value.kind else {
            panic!("expected a string");
        };
        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0], Fragment::Text(Cow::Borrowed("a"), Span::new(1, 8, 1)));
    }

    #[test]
    fn a_multiline_string_makes_the_entry_span_every_character() {
        let entry = entry("m: \"\"\"\nabc\n\"\"\"");
        assert_eq!(entry.value.span, Span::new(1, 4, 11));
        assert_eq!(entry.span, Span::new(1, 1, 14));
        dbg!(entry);
    }

    #[test]
    fn the_entry_stops_right_after_the_value() {
        let mut parser = Parser::new("a: 1 b: 2");
        let entry = parser.parse_entry().expect("valid input");
        assert_eq!(entry.span, Span::new(1, 1, 4));
        assert_eq!(
            parser.stream.peek().expect("valid input").kind,
            TokenKind::Whitespace
        );
    }

    #[test]
    fn a_trailing_comment_and_newline_are_left_alone() {
        let mut parser = Parser::new("a: 1 # c\nb: 2");
        parser.parse_entry().expect("valid input");
        parser.stream.skip_inline_space().expect("valid input");
        assert_eq!(
            parser.stream.peek().expect("valid input").kind,
            TokenKind::Comment
        );
    }

    #[test]
    fn a_missing_colon_points_right_after_the_key() {
        for text in ["key 1", "key", "key   ", "key\n: 1", "key # c"] {
            assert_eq!(
                failure(text),
                (SyntaxError::MissingColon, Span::new_point(1, 4)),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_missing_value_points_right_after_the_colon() {
        for text in ["key:", "key:   ", "key: # c", "key:\n1", "key : ", "key:\r\n1"] {
            let column = text.find(':').expect("has a colon") + 2;
            assert_eq!(
                failure(text),
                (SyntaxError::MissingValue, Span::new_point(1, column)),
                "{text:?}"
            );
        }
    }

    #[test]
    fn collections_and_unknown_words_are_invalid_values_for_now() {
        let table = [("key: foo", 3), ("key: [1]", 1), ("key: {}", 1), ("key: ]", 1)];
        for (text, length) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::InvalidValue, Span::new(1, 6, length)),
                "{text}"
            );
        }
    }

    #[test]
    fn a_bad_key_is_an_invalid_statement_start() {
        let table = [("1: 2", 1), ("\"a\": 1", 3), (": 1", 1), ("@: 1", 1), ("[a]", 1)];
        for (text, length) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::InvalidStatementStart, Span::new(1, 1, length)),
                "{text}"
            );
        }
    }

    #[test]
    fn empty_input_is_an_invalid_statement_start_with_a_zero_width_span() {
        assert_eq!(
            failure(""),
            (SyntaxError::InvalidStatementStart, Span::new_point(1, 1))
        );
    }

    #[test]
    fn lexer_errors_win_because_the_stream_lexes_ahead() {
        assert_eq!(
            failure("a: 007"),
            (SyntaxError::UnexpectedLeadingZero, Span::new(1, 4, 3))
        );
        assert_eq!(
            failure("key 007"),
            (SyntaxError::UnexpectedLeadingZero, Span::new(1, 5, 3))
        );
    }

    #[test]
    fn errors_carry_the_source_line() {
        let diagnostic = diagnostic("a: foo");
        assert_eq!(diagnostic.excerpt().expect("has an excerpt").snippet(), "a: foo");
    }

    #[test]
    fn an_error_on_the_first_line_of_a_longer_text_shows_only_that_line() {
        let diagnostic = diagnostic("key:\n1");
        let excerpt = diagnostic.excerpt().expect("has an excerpt");
        assert_eq!(excerpt.snippet(), "key:");
        assert_eq!(excerpt.span(), Span::new_point(1, 5));
    }
}
use std::borrow::Cow;

use crate::ast::Expr;
use crate::ast::ExprKind;
use crate::error::Diagnostic;
use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::span::Span;
use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_value(&mut self) -> Result<Expr<'a>, Diagnostic> {
        let Token { kind, text, span } = self.stream.advance()?;

        let kind = match kind { 
            TokenKind::Float => ExprKind::Float(self.float(text, span)?),
            TokenKind::Integer => ExprKind::Integer(self.integer(text, span)?),
            TokenKind::String { fragments } => ExprKind::String(fragments),
            TokenKind::Identifier if text == "true" => ExprKind::Boolean(true),
            TokenKind::Identifier if text == "false" => ExprKind::Boolean(false),
            _ => return Err(self.error(crate::error::SyntaxError::InvalidValue, span)),
        };
        Ok(Expr { kind, span })
    }

    fn float(&self, text: &str, span: Span) -> Result<f64, Diagnostic> {
        without_separators(text)
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| self.error(crate::error::SyntaxError::FloatOutOfRange, span))
    }

    fn integer(&self, text: &str, span: Span) -> Result<i64, Diagnostic> {
        without_separators(text)
            .parse::<i64>()
            .map_err(|_| self.error(crate::error::SyntaxError::IntegerOutOfRange, span))
    }
}

fn without_separators(text: &str) -> Cow<'_, str> {
    if text.contains('_') { Cow::Owned(text.replace('_', "")) } else { Cow::Borrowed(text) }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::error::Kind;
    use crate::lexer::Fragment;

    use crate::error::SyntaxError;

    fn value(text: &str) -> Expr<'_> {
        Parser::new(text)
            .parse_value()
            .unwrap_or_else(|e| panic!("parse error in {text:?}: {e:?}"))
    }

    fn kind(text: &str) -> ExprKind<'_> {
        value(text).kind
    }

    fn failure(text: &str) -> (SyntaxError, Span) {
        let diagnostic = Parser::new(text)
            .parse_value()
            .err()
            .unwrap_or_else(|| panic!("expected a parse error for {text:?}"));
        let Kind::Syntax(error) = diagnostic.kind().clone() else {
            panic!("expected a syntax error for {text:?}");
        };
        (error, diagnostic.excerpt().expect("has an excerpt").span())
    }

    #[test]
    fn integers_are_converted() {
        let table = [
            ("0", 0),
            ("42", 42),
            ("-5", -5),
            ("-0", 0),
            ("1_000_000", 1_000_000),
            ("9223372036854775807", i64::MAX),
            ("-9223372036854775808", i64::MIN),
        ];
        for (text, expected) in table {
            assert_eq!(kind(text), ExprKind::Integer(expected), "{text}");
        }
    }

    #[test]
    fn floats_are_converted() {
        let table = [
            ("2.5", 2.5),
            ("-0.5", -0.5),
            ("1e3", 1000.0),
            ("6.02E+23", 6.02e23),
            ("1.5e-4", 1.5e-4),
            ("1_000.000_1", 1000.0001),
            ("1e1_0", 1e10),
            ("1e-999", 0.0),
        ];
        for (text, expected) in table {
            assert_eq!(kind(text), ExprKind::Float(expected), "{text}");
        }
    }

    #[test]
    fn booleans_are_identifiers_that_match_exactly() {
        assert_eq!(kind("true"), ExprKind::Boolean(true));
        assert_eq!(kind("false"), ExprKind::Boolean(false));
    }

    #[test]
    fn a_string_keeps_its_fragments() {
        assert_eq!(
            kind("\"hi\""),
            ExprKind::String(vec![Fragment::Text(Cow::Borrowed("hi"), Span::new(1, 2, 2))])
        );
        assert_eq!(kind("\"\""), ExprKind::String(Vec::new()));
    }

    #[test]
    fn a_multiline_string_is_a_string_too() {
        let ExprKind::String(fragments) = kind("\"\"\"\nabc\n\"\"\"") else {
            panic!("expected a string");
        };
        assert_eq!(
            fragments,
            [Fragment::Text(Cow::Borrowed("abc"), Span::new(2, 1, 3))]
        );
    }

    #[test]
    fn the_span_covers_the_whole_token() {
        assert_eq!(value("-0.5").span, Span::new(1, 1, 4));
        assert_eq!(value("false").span, Span::new(1, 1, 5));
        assert_eq!(value("\"hi\"").span, Span::new(1, 1, 4));
    }

    #[test]
    fn only_the_first_token_is_consumed() {
        let mut parser = Parser::new("1 2");
        parser.parse_value().expect("valid input");
        assert_eq!(
            parser.stream.peek().expect("valid input").kind,
            TokenKind::Whitespace
        );
    }

    #[test]
    fn anything_else_is_an_invalid_value_on_its_first_token() {
        let table = [
            ("foo", 3),
            ("True", 4),
            ("FALSE", 5),
            ("truex", 5),
            ("[", 1),
            ("{", 1),
            ("]", 1),
            ("}", 1),
            (":", 1),
            (",", 1),
            (".", 1),
            ("@", 1),
            ("\n", 1),
            ("# c", 3),
        ];
        for (text, length) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::InvalidValue, Span::new(1, 1, length)),
                "{text:?}"
            );
        }
    }

    #[test]
    fn the_end_of_input_is_an_invalid_value_with_a_zero_width_span() {
        assert_eq!(
            failure(""),
            (SyntaxError::InvalidValue, Span::new_point(1, 1))
        );
    }

    #[test]
    fn lexer_errors_pass_through() {
        assert_eq!(
            failure("007"),
            (SyntaxError::UnexpectedLeadingZero, Span::new(1, 1, 3))
        );
        assert_eq!(
            failure("\"abc"),
            (SyntaxError::UnclosedString, Span::new_point(1, 5))
        );
    }
}
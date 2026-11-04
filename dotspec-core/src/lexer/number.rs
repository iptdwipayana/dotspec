use crate::span::Span;
use crate::error::SyntaxError;
use super::cursor::Cursor;
use super::error::Error;
use super::token::TokenKind;

pub(super) fn is_digit(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_digit())
}

pub(super) fn lex(cursor: &mut Cursor<'_>) -> Result<TokenKind, Error> {
    let offset = cursor.offset();
    let line = cursor.line();
    let column = cursor.column();

    if cursor.peek() == Some('-') {
        if !is_digit(cursor.peek_at(1)) {
            return Err(Error::new(SyntaxError::MalformedNumber, Span::new(
                line,
                column,
                1
            )));
        }
        cursor.bump();
    }

    integer(cursor)?;

    let mut kind = TokenKind::Integer;
    if cursor.peek() == Some('.') && is_digit(cursor.peek_at(1)) {
        cursor.bump();
        digits(cursor);
        kind = TokenKind::Float;
    }
    if matches!(cursor.peek(), Some('e' | 'E')) {
        exponent(cursor, offset, line, column)?;
        kind = TokenKind::Float;
    }

    let text = cursor.slice(offset);
    check_range(
        text,
        kind,
        Span::new(line, column, text.chars().count())
    )?;
    Ok(kind)
}

fn digits(cursor: &mut Cursor<'_>) {
    cursor.bump();
    loop {
        match cursor.peek() {
            Some(c) if c.is_ascii_digit() => { cursor.bump(); },
            Some('_') if is_digit(cursor.peek_at(1)) => { cursor.bump(); },
            _ => break,
        }
    }
}

fn integer(cursor: &mut Cursor<'_>) -> Result<(), Error> {
    let line = cursor.line();
    let column = cursor.column();
    let offset = cursor.offset();
    let starts_with_zero = cursor.peek() == Some('0');

    digits(cursor);

    if starts_with_zero && cursor.offset() - offset > 1 {
        return Err(Error::new(SyntaxError::UnexpectedLeadingZero, Span::new(
            line, 
            column, 
            cursor.slice(offset).chars().count()
        )));
    }
    Ok(())
}

fn exponent(cursor: &mut Cursor<'_>, offset: usize, line: usize, column: usize) -> Result<(), Error> {
    cursor.bump();
    if matches!(cursor.peek(), Some('+' | '-')) {
        cursor.bump();
    }
    if is_digit(cursor.peek()) {
        digits(cursor);
        return Ok(());
    }

    Err(Error::new(SyntaxError::MalformedNumber, Span::new(
        line,
        column,
        cursor.slice(offset).chars().count(),
    )))
}

fn check_range(text: &str, kind: TokenKind, span: Span) -> Result<(), Error> {
    let clean: String = text.chars().filter(|&c| c != '_').collect();

    let error = match kind {
        TokenKind::Integer if clean.parse::<i64>().is_err() => SyntaxError::IntegerOutOfRange,
        TokenKind::Float if clean.parse::<f64>().is_ok_and(|v| !v.is_finite()) => SyntaxError::FloatOutOfRange,
        _ => return Ok(()), 
    };
    Err(Error::new(error, span))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Kind;
    use crate::lexer::Lexer;
    use TokenKind::*;

    fn tokens(text: &str) -> Vec<(TokenKind, &str)> {
        Lexer::new(text)
            .map(|t| t.expect("lexes without any error"))
            .filter(|t| t.kind != Eof)
            .map(|t| (t.kind, t.text))
            .collect()
    }

    fn failure(text: &str) -> (SyntaxError, Span) {
        let diagnostic = Lexer::new(text).find_map(Result::err).expect("expected a lexer error");
        let Kind::Syntax(error) = diagnostic.kind().clone() else {
            panic!("expected a syntax error");
        };
        (error, diagnostic.excerpt().expect("has an excerpt").span())
    }

    #[test]
    fn lexes_integers() {
        for text in ["0", "42", "-5", "-0"] {
            assert_eq!(tokens(text), [(Integer, text)], "{text}");
        }
    }

    #[test]
    fn lexes_floats() {
        for text in ["2.5", "-0.5", "1e3", "6.02E+23", "1.5e-4", "1e-999"] {
            assert_eq!(tokens(text), [(Float, text)], "{text}");
        }
    }

    #[test]
    fn lexes_digit_separators_and_keeps_them_in_the_text() {
        for text in ["1_000_000", "-1_000"] {
            assert_eq!(tokens(text), [(Integer, text)], "{text}");
        }
        for text in ["1_000.000_1", "1e1_0", "1_0.2_5"] {
            assert_eq!(tokens(text), [(Float, text)], "{text}");
        }
    }

    #[test]
    fn the_span_covers_the_sign_and_every_character() {
        let lexed = Lexer::new("  -0.5")
            .collect::<Result<Vec<_>, _>>()
            .expect("valid input");
        assert_eq!(lexed[1].span, Span::new(1, 3, 4));
    }

    #[test]
    fn the_i64_limits_are_accepted() {
        for text in ["9223372036854775807", "-9223372036854775808"] {
            assert_eq!(tokens(text), [(Integer, text)], "{text}");
        }
    }

    #[test]
    fn a_dot_without_a_digit_after_it_is_not_a_fraction() {
        assert_eq!(tokens("1."), [(Integer, "1"), (Dot, ".")]);
        assert_eq!(
            tokens("123.abc"),
            [(Integer, "123"), (Dot, "."), (Identifier, "abc")]
        );
    }

    #[test]
    fn letters_after_a_number_start_an_identifier() {
        assert_eq!(tokens("1abc"), [(Integer, "1"), (Identifier, "abc")]);
    }

    #[test]
    fn a_misplaced_underscore_ends_the_number() {
        assert_eq!(tokens("1_"), [(Integer, "1"), (Identifier, "_")]);
        assert_eq!(tokens("1__0"), [(Integer, "1"), (Identifier, "__0")]);
        assert_eq!(tokens("1_abc"), [(Integer, "1"), (Identifier, "_abc")]);
        assert_eq!(tokens("0_"), [(Integer, "0"), (Identifier, "_")]);
        assert_eq!(tokens("1e1_"), [(Float, "1e1"), (Identifier, "_")]);
    }

    #[test]
    fn an_underscore_next_to_the_dot_or_e_is_not_a_separator() {
        assert_eq!(
            tokens("1_.5"),
            [(Integer, "1"), (Identifier, "_"), (Dot, "."), (Integer, "5")]
        );
        assert_eq!(
            tokens("1._5"),
            [(Integer, "1"), (Dot, "."), (Identifier, "_5")]
        );
        assert_eq!(tokens("1_e3"), [(Integer, "1"), (Identifier, "_e3")]);
    }

    #[test]
    fn a_dash_inside_an_identifier_is_not_a_sign() {
        assert_eq!(tokens("a-1"), [(Identifier, "a-1")]);
    }

    #[test]
    fn a_plus_without_a_digit_stays_unknown() {
        assert_eq!(tokens("+a"), [(Unknown, "+"), (Identifier, "a")]);
    }

    #[test]
    fn leading_zeros_point_at_the_digits_only() {
        let leading_zero = SyntaxError::UnexpectedLeadingZero;
        assert_eq!(failure("007"), (leading_zero, Span::new(1, 1, 3)));
        assert_eq!(failure("-007"), (leading_zero, Span::new(1, 2, 3)));
        assert_eq!(failure("00"), (leading_zero, Span::new(1, 1, 2)));
        assert_eq!(failure("0_1"), (leading_zero, Span::new(1, 1, 3)));
        assert_eq!(failure("007.5"), (leading_zero, Span::new(1, 1, 3)));
    }

    #[test]
    fn a_leading_plus_is_malformed() {
        let malformed = SyntaxError::MalformedNumber;
        assert_eq!(failure("+5"), (malformed, Span::new(1, 1, 1)));
        assert_eq!(failure("  +5"), (malformed, Span::new(1, 3, 1)));
    }

    #[test]
    fn a_sign_without_a_digit_is_malformed() {
        for text in ["-", "-abc", "-.5"] {
            assert_eq!(
                failure(text),
                (SyntaxError::MalformedNumber, Span::new(1, 1, 1)),
                "{text}"
            );
        }
    }

    #[test]
    fn an_incomplete_exponent_is_malformed() {
        for text in ["1e", "1e+", "1E-", "-1e", "2.5e"] {
            assert_eq!(
                failure(text),
                (SyntaxError::MalformedNumber, Span::new(1, 1, text.len())),
                "{text}"
            );
        }
        assert_eq!(
            failure("1e_3"),
            (SyntaxError::MalformedNumber, Span::new(1, 1, 2))
        );
    }

    #[test]
    fn integers_outside_i64_are_rejected_under_the_whole_number() {
        for text in [
            "99999999999999999999",
            "9223372036854775808",
            "-9223372036854775809",
            "1_000_000_000_000_000_000_000",
        ] {
            assert_eq!(
                failure(text),
                (SyntaxError::IntegerOutOfRange, Span::new(1, 1, text.len())),
                "{text}"
            );
        }
    }

    #[test]
    fn floats_that_overflow_are_rejected() {
        for text in ["1e999", "-1e999", "1e1_000"] {
            assert_eq!(
                failure(text),
                (SyntaxError::FloatOutOfRange, Span::new(1, 1, text.len())),
                "{text}"
            );
        }
    }

    #[test]
    fn the_error_carries_the_source_line() {
        let diagnostic = Lexer::new("a: 1\nb: 007")
            .find_map(Result::err)
            .expect("expected a lexer error");
        let excerpt = diagnostic.excerpt().expect("has an excerpt");
        assert_eq!(excerpt.span(), Span::new(2, 4, 3));
        assert_eq!(excerpt.snippet(), "b: 007");
    }

    #[test]
    fn tokens_before_the_error_are_still_yielded() {
        let results: Vec<bool> = Lexer::new("ab 007").map(|t| t.is_ok()).collect();
        assert_eq!(results, [true, true, false]);
        assert_eq!(
            failure("ab 007"),
            (SyntaxError::UnexpectedLeadingZero, Span::new(1, 4, 3))
        );
    }

    #[test]
    fn the_iterator_ends_after_an_error() {
        let mut lexer = Lexer::new("007 x");
        assert!(matches!(lexer.next(), Some(Err(_))));
        assert!(lexer.next().is_none());
    }
}
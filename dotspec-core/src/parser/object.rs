use super::Parser;
use super::Spacing;
use crate::ast::Expr;
use crate::ast::ExprKind;
use crate::lexer::TokenKind;
use crate::error::Diagnostic;

impl<'a> Parser<'a> {
    pub(super) fn parse_object(&mut self) -> Result<Expr<'a>, Diagnostic> {
        let before = self.stream.consumed();
        let open = self.stream.advance()?;
        self.enter(&open)?;

        let mut entries = Vec::new();
        self.skip(Spacing::Any)?;

        loop {
            let span = self.stream.peek()?.span;
            match self.stream.peek()?.kind {
                TokenKind::RightBrace => break,
                TokenKind::Comma => {
                    return Err(self.error(
                        crate::error::SyntaxError::UnexpectedObjectComma,
                        span
                    ));
                },
                TokenKind::Eof | TokenKind::RightBracket => return Err(self.unclosed_object()),
                TokenKind::Identifier => {},
                _ => return Err(self.error(
                    crate::error::SyntaxError::InvalidObjectKey, 
                    span
                )),
            }

            entries.push(self.parse_object_entry()?);
            self.skip(Spacing::Any)?;

            let starts_value = self.next_starts_value()?;
            match self.stream.peek()?.kind {
                TokenKind::Comma => {
                    self.stream.advance()?;
                    self.skip(Spacing::Any)?;
                },
                TokenKind::RightBrace => break,
                _ if starts_value => {
                    return Err(self.error(
                        crate::error::SyntaxError::MissingObjectComma, 
                        self.stream.end()
                    ));
                },
                _ => return Err(self.unclosed_object()),
            }
        }

        self.stream.advance()?;
        self.leave();
        Ok(Expr {
            kind: ExprKind::Object(entries),
            span: self.span_since(open.span, before),
        })
    }

    fn unclosed_object(&self) -> Diagnostic {
        self.error(crate::error::SyntaxError::UnclosedObject, self.stream.end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Entry;
    use crate::span::Span;
    use crate::parser::testing::show;
    use crate::parser::testing::syntax_error;

    use crate::error::SyntaxError;

    fn parse(text: &str) -> Expr<'_> {
        Parser::new(text)
            .parse_value()
            .unwrap_or_else(|e| panic!("parse error in {text:?}: {e:?}"))
    }

    fn shown(text: &str) -> String {
        show(&parse(text))
    }

    fn entries(text: &str) -> Vec<Entry<'_>> {
        match parse(text).kind {
            ExprKind::Object(entries) => entries,
            other => panic!("expected an object for {text:?}, got {other:?}"),
        }
    }

    fn failure(text: &str) -> (SyntaxError, Span) {
        let diagnostic = Parser::new(text)
            .parse_value()
            .err()
            .unwrap_or_else(|| panic!("expected a parse error for {text:?}"));
        syntax_error(&diagnostic)
    }

    #[test]
    fn an_empty_object_has_no_entries() {
        for text in ["{}", "{ }", "{\n}", "{ # nothing\n}"] {
            assert_eq!(shown(text), "{}", "{text:?}");
        }
    }

    #[test]
    fn entries_are_separated_by_commas() {
        assert_eq!(shown("{ a: 1 }"), "{a: 1}");
        assert_eq!(shown("{a:1}"), "{a: 1}");
        assert_eq!(shown("{ a: 1, b: 2 }"), "{a: 1, b: 2}");
        assert_eq!(shown("{ a : 1 , b : 2 }"), "{a: 1, b: 2}");
    }

    #[test]
    fn one_trailing_comma_is_allowed() {
        assert_eq!(shown("{ a: 1, }"), "{a: 1}");
        assert_eq!(shown("{a: 1,}"), "{a: 1}");
    }

    #[test]
    fn the_inline_example_of_example_spec_parses() {
        assert_eq!(
            shown("{ host: \"localhost\", port: 5432 }"),
            "{host: \"localhost\", port: 5432}"
        );
    }

    #[test]
    fn objects_nest() {
        assert_eq!(shown("{ a: { b: { c: 1 } } }"), "{a: {b: {c: 1}}}");
        assert_eq!(shown("{ a: {}, b: {} }"), "{a: {}, b: {}}");
    }

    #[test]
    fn arrays_can_be_values() {
        assert_eq!(shown("{ a: [1, 2], b: [] }"), "{a: [1, 2], b: []}");
        assert_eq!(shown("{ a: [{ b: 1 }] }"), "{a: [{b: 1}]}");
    }

    #[test]
    fn a_mix_of_arrays_and_objects_nests_in_any_order() {
        assert_eq!(
            shown("[{ a: [1, { b: 2 }] }]"),
            "[{a: [1, {b: 2}]}]"
        );
    }

    #[test]
    fn entries_can_be_laid_out_over_several_lines() {
        assert_eq!(shown("{\n  a: 1,\n  b: 2,\n}"), "{a: 1, b: 2}");
        assert_eq!(shown("{ a:\n1 }"), "{a: 1}");
        assert_eq!(shown("{\n  a\n  :\n  1\n  ,\n  b: 2\n}"), "{a: 1, b: 2}");
    }

    #[test]
    fn comments_are_allowed_between_any_two_tokens() {
        let text = "{ # open\n  a: 1, # one\n  # on its own line\n  b: 2 # two\n  # before the end\n}";
        assert_eq!(shown(text), "{a: 1, b: 2}");
    }

    #[test]
    fn duplicate_keys_are_kept_for_the_semantic_stage() {
        assert_eq!(shown("{ a: 1, a: 2 }"), "{a: 1, a: 2}");
    }

    #[test]
    fn keys_may_use_dashes_underscores_and_the_words_true_and_false() {
        assert_eq!(
            shown("{ app-name: 1, _x1: 2, true: 3 }"),
            "{app-name: 1, _x1: 2, true: 3}"
        );
    }

    #[test]
    fn the_object_stops_after_the_closing_brace() {
        let mut parser = Parser::new("{ a: 1 } x");
        parser.parse_value().expect("valid input");
        assert_eq!(
            parser.stream.peek().expect("valid input").kind,
            TokenKind::Whitespace
        );
    }

    #[test]
    fn the_span_covers_both_braces() {
        assert_eq!(parse("{}").span, Span::new(1, 1, 2));
        assert_eq!(parse("{ a: 1 }").span, Span::new(1, 1, 8));
    }

    #[test]
    fn keys_values_and_entries_have_their_own_spans() {
        let entries = entries("{ a: 1 }");
        assert_eq!(entries[0].key.span, Span::new(1, 3, 1));
        assert_eq!(entries[0].value.span, Span::new(1, 6, 1));
        assert_eq!(entries[0].span, Span::new(1, 3, 4));
    }

    #[test]
    fn entries_on_later_lines_start_at_their_own_position() {
        let entries = entries("{\n  a: 1,\n  b: 2\n}");
        assert_eq!(entries[0].span, Span::new(2, 3, 4));
        assert_eq!(entries[1].span, Span::new(3, 3, 4));
    }

    #[test]
    fn a_multiline_value_makes_its_entry_span_every_character() {
        let entry_text = "m: \"\"\"\nx\n\"\"\"";
        let text = format!("{{ {entry_text} }}");
        let entries = entries(&text);
        assert_eq!(entries[0].span, Span::new(1, 3, entry_text.chars().count()));
        assert_eq!(parse(&text).span, Span::new(1, 1, text.chars().count()));
    }

    #[test]
    fn a_missing_colon_points_right_after_the_key() {
        let table = [
            ("{ a }", Span::new_point(1, 4)),
            ("{ a\n}", Span::new_point(1, 4)),
            ("{ a 1 }", Span::new_point(1, 4)),
            ("{ a, b: 1 }", Span::new_point(1, 4)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::MissingColon, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_missing_value_points_right_after_the_colon() {
        let table = [
            ("{ a: }", Span::new_point(1, 5)),
            ("{ a:, b: 1 }", Span::new_point(1, 5)),
            ("{ a:\n}", Span::new_point(1, 5)),
            ("{ a:", Span::new_point(1, 5)),
            ("{ a: # c\n}", Span::new_point(1, 5)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::MissingValue, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_bad_value_is_an_invalid_value() {
        let table = [
            ("{ a: ] }", Span::new(1, 6, 1)),
            ("{ a: foo }", Span::new(1, 6, 3)),
            ("{ a: : }", Span::new(1, 6, 1)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::InvalidValue, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_bad_key_points_at_the_offending_token() {
        let table = [
            ("{ 1: \"a\" }", Span::new(1, 3, 1)),
            ("{ \"a\": 1 }", Span::new(1, 3, 3)),
            ("{ [1]: 2 }", Span::new(1, 3, 1)),
            ("{ : 1 }", Span::new(1, 3, 1)),
            ("{ 1.5: 2 }", Span::new(1, 3, 3)),
            ("{ . }", Span::new(1, 3, 1)),
            ("{ @ }", Span::new(1, 3, 1)),
            ("{ a: 1, 2: 3 }", Span::new(1, 9, 1)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::InvalidObjectKey, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_missing_comma_points_right_after_the_entry() {
        let table = [
            ("{ a: 1 b: 2 }", Span::new_point(1, 7)),
            ("{ a: 1\nb: 2 }", Span::new_point(1, 7)),
            ("{ a: \"x\" \"y\" }", Span::new_point(1, 9)),
            ("{ a: 1 2 }", Span::new_point(1, 7)),
            ("{ a: [1] b: 2 }", Span::new_point(1, 9)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::MissingObjectComma, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn an_unclosed_object_points_right_after_the_last_piece() {
        let table = [
            ("{", Span::new_point(1, 2)),
            ("{ a: 1", Span::new_point(1, 7)),
            ("{ a: 1,", Span::new_point(1, 8)),
            ("{ a: 1   ", Span::new_point(1, 7)),
            ("{ a: 1 # c\n\n", Span::new_point(1, 7)),
            ("{ a: {", Span::new_point(1, 7)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::UnclosedObject, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_closer_of_the_wrong_kind_means_the_object_is_not_closed() {
        let table = [
            ("{]", Span::new_point(1, 2)),
            ("{ a: 1 ]", Span::new_point(1, 7)),
            ("{ a: 1, ]", Span::new_point(1, 8)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::UnclosedObject, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_stray_comma_points_at_the_comma() {
        let table = [
            ("{,}", Span::new(1, 2, 1)),
            ("{ , a: 1 }", Span::new(1, 3, 1)),
            ("{ a: 1,, }", Span::new(1, 8, 1)),
            ("{ a: 1, , b: 2 }", Span::new(1, 9, 1)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::UnexpectedObjectComma, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn lexer_errors_inside_an_object_pass_through() {
        assert_eq!(
            failure("{ a: 007 }"),
            (SyntaxError::UnexpectedLeadingZero, Span::new(1, 6, 3))
        );
    }

    #[test]
    fn an_error_on_a_later_line_carries_that_line() {
        let diagnostic = Parser::new("{\n  a: 1\n  b: 2\n}")
            .parse_value()
            .err()
            .expect("an error");
        let excerpt = diagnostic.excerpt().expect("has an excerpt");
        assert_eq!(excerpt.snippet(), "  a: 1");
        assert_eq!(excerpt.span(), Span::new_point(2, 7));
    }
}
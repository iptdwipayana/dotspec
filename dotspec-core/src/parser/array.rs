use super::Parser;
use super::Spacing;
use crate::ast::Expr;
use crate::ast::ExprKind;
use crate::lexer::TokenKind;
use crate::error::Diagnostic;

impl<'a> Parser<'a> {
    pub(super) fn parse_array(&mut self) -> Result<Expr<'a>, Diagnostic> {
        let before = self.stream.consumed();
        let open = self.stream.advance()?;
        self.enter(&open)?;

        let mut items = Vec::new();
        self.skip(Spacing::Any)?;

        loop {
            let span = self.stream.peek()?.span;
            match self.stream.peek()?.kind {
                TokenKind::RightBracket => break,
                TokenKind::Comma => {
                    return Err(self.error(
                        crate::error::SyntaxError::UnexpectedArrayComma, 
                        span
                    ));
                },
                TokenKind::Eof | TokenKind::RightBrace => return Err(self.unclosed_array()),
                _ => {}
            }

            items.push(self.parse_value()?);
            self.skip(Spacing::Any)?;

            let starts_value = self.next_starts_value()?;
            match self.stream.peek()?.kind {
                TokenKind::Comma => {
                    self.stream.advance()?;
                    self.skip(Spacing::Any)?;
                },
                TokenKind::RightBracket => break,
                _ if starts_value => {
                    return Err(self.error(
                        crate::error::SyntaxError::MissingArrayComma, 
                        self.stream.end()
                    ));
                },
                _ => return Err(self.unclosed_array())
            }
        }

        self.stream.advance()?;
        self.leave();
        Ok(Expr {
            kind: ExprKind::Array(items),
            span: self.span_since(open.span, before),
        })
    }

    fn unclosed_array(&self) -> Diagnostic {
        self.error(crate::error::SyntaxError::UnclosedArray, self.stream.end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SyntaxError;
    use crate::span::Span;
    use crate::parser::testing::show;
    use crate::parser::testing::syntax_error;

    fn parse(text: &str) -> Expr<'_> {
        Parser::new(text)
            .parse_value()
            .unwrap_or_else(|e| panic!("parse error in {text:?}: {e:?}"))
    }

    fn shown(text: &str) -> String {
        show(&parse(text))
    }

    fn items(text: &str) -> Vec<Expr<'_>> {
        match parse(text).kind {
            ExprKind::Array(items) => items,
            other => panic!("expected an array for {text:?}, got {other:?}"),
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
    fn an_empty_array_has_no_items() {
        for text in ["[]", "[ ]", "[\n]", "[ # nothing\n]"] {
            assert_eq!(shown(text), "[]", "{text:?}");
        }
    }

    #[test]
    fn items_are_separated_by_commas() {
        assert_eq!(shown("[1, 2, 3]"), "[1, 2, 3]");
        assert_eq!(shown("[1,2,3]"), "[1, 2, 3]");
        assert_eq!(shown("[ 1 , 2 , 3 ]"), "[1, 2, 3]");
    }

    #[test]
    fn one_trailing_comma_is_allowed() {
        assert_eq!(shown("[1, 2, 3,]"), "[1, 2, 3]");
        assert_eq!(shown("[1,]"), "[1]");
        assert_eq!(shown("[1, ]"), "[1]");
    }

    #[test]
    fn items_can_be_of_different_types() {
        assert_eq!(shown("[\"a\", 1, true, 2.5]"), "[\"a\", 1, true, 2.5]");
    }

    #[test]
    fn arrays_nest() {
        assert_eq!(shown("[[1, 2], [3, 4]]"), "[[1, 2], [3, 4]]");
        assert_eq!(shown("[[]]"), "[[]]");
        assert_eq!(shown("[[], []]"), "[[], []]");
        assert_eq!(shown("[[[1]]]"), "[[[1]]]");
    }

    #[test]
    fn objects_can_be_items() {
        assert_eq!(shown("[{ a: 1 }, { b: 2 }]"), "[{a: 1}, {b: 2}]");
    }

    #[test]
    fn items_can_be_laid_out_over_several_lines() {
        assert_eq!(shown("[\n  1,\n  2,\n]"), "[1, 2]");
        assert_eq!(shown("[1\n, 2]"), "[1, 2]");
        assert_eq!(shown("[\n\n1\n\n,\n\n2\n\n]"), "[1, 2]");
    }

    #[test]
    fn comments_are_allowed_between_any_two_tokens() {
        let text = "[ # open\n  1, # one\n  # on its own line\n  2 # two\n  # before the end\n]";
        assert_eq!(shown(text), "[1, 2]");
    }

    #[test]
    fn a_hash_inside_a_string_is_not_a_comment() {
        assert_eq!(shown("[\"a # b\"]"), "[\"a # b\"]");
    }

    #[test]
    fn the_array_stops_after_the_closing_bracket() {
        let mut parser = Parser::new("[1] x");
        parser.parse_value().expect("valid input");
        assert_eq!(
            parser.stream.peek().expect("valid input").kind,
            TokenKind::Whitespace
        );
    }

    #[test]
    fn the_span_covers_both_brackets() {
        assert_eq!(parse("[]").span, Span::new(1, 1, 2));
        assert_eq!(parse("[1, 2]").span, Span::new(1, 1, 6));
        assert_eq!(parse("[ 1 , 2 , ]").span, Span::new(1, 1, 11));
    }

    #[test]
    fn the_span_of_a_multiline_array_counts_every_character() {
        let text = "[\n  1,\n  2,\n]";
        assert_eq!(parse(text).span, Span::new(1, 1, text.chars().count()));
    }

    #[test]
    fn items_have_their_own_spans() {
        let items = items("[1, [2], \"x\"]");
        assert_eq!(items[0].span, Span::new(1, 2, 1));
        assert_eq!(items[1].span, Span::new(1, 5, 3));
        assert_eq!(items[2].span, Span::new(1, 10, 3));
    }

    #[test]
    fn nested_arrays_know_where_they_start_on_later_lines() {
        let items = items("[\n  [1],\n  [2, 3]\n]");
        assert_eq!(items[0].span, Span::new(2, 3, 3));
        assert_eq!(items[1].span, Span::new(3, 3, 6));
    }

    #[test]
    fn a_missing_comma_points_right_after_the_item() {
        let table = [
            ("[1 2]", Span::new_point(1, 3)),
            ("[1\n2]", Span::new_point(1, 3)),
            ("[\"a\" \"b\"]", Span::new_point(1, 5)),
            ("[[1] [2]]", Span::new_point(1, 5)),
            ("[1 true]", Span::new_point(1, 3)),
            ("[1 {}]", Span::new_point(1, 3)),
            ("[1 # c\n2]", Span::new_point(1, 3)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::MissingArrayComma, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn an_unclosed_array_points_right_after_the_last_piece() {
        let table = [
            ("[", Span::new_point(1, 2)),
            ("[1", Span::new_point(1, 3)),
            ("[1,", Span::new_point(1, 4)),
            ("[1, 2", Span::new_point(1, 6)),
            ("[1, 2   ", Span::new_point(1, 6)),
            ("[1, 2 # c\n\n", Span::new_point(1, 6)),
            ("[ \n # c", Span::new_point(1, 2)),
            ("[[1]", Span::new_point(1, 5)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::UnclosedArray, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_closer_of_the_wrong_kind_means_the_array_is_not_closed() {
        let table = [
            ("[}", Span::new_point(1, 2)),
            ("[1 }", Span::new_point(1, 3)),
            ("[1,}", Span::new_point(1, 4)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::UnclosedArray, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_stray_comma_points_at_the_comma() {
        let table = [
            ("[,]", Span::new(1, 2, 1)),
            ("[,1]", Span::new(1, 2, 1)),
            ("[ ,]", Span::new(1, 3, 1)),
            ("[1,,2]", Span::new(1, 4, 1)),
            ("[1, ,]", Span::new(1, 5, 1)),
            ("[1,\n,]", Span::new(2, 1, 1)),
        ];
        for (text, span) in table {
            assert_eq!(
                failure(text),
                (SyntaxError::UnexpectedArrayComma, span),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_bad_item_is_an_invalid_value() {
        let table = [
            ("[foo]", Span::new(1, 2, 3)),
            ("[1, foo]", Span::new(1, 5, 3)),
            ("[:]", Span::new(1, 2, 1)),
            ("[@]", Span::new(1, 2, 1)),
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
    fn lexer_errors_inside_an_array_pass_through() {
        assert_eq!(
            failure("[1, 007]"),
            (SyntaxError::UnexpectedLeadingZero, Span::new(1, 5, 3))
        );
        assert_eq!(
            failure("[\"abc]"),
            (SyntaxError::UnclosedString, Span::new_point(1, 7))
        );
    }

    #[test]
    fn an_error_on_a_later_line_carries_that_line() {
        let diagnostic = Parser::new("[\n  1\n  2\n]")
            .parse_value()
            .err()
            .expect("an error");
        let excerpt = diagnostic.excerpt().expect("has an excerpt");
        assert_eq!(excerpt.snippet(), "  1");
        assert_eq!(excerpt.span(), Span::new_point(2, 4));
    }
}
use crate::error::Diagnostic;
use crate::source::Source;
use crate::span::Span;
use super::cursor::Cursor;
use super::error::Error;
use super::token::Token;
use super::token::TokenKind;

use super::number;
use super::string;

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    text: &'a str,
    finished: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { cursor: Cursor::new(text), text, finished: false }
    }

    fn token(&mut self) -> Result<Token<'a>, Error> {
        let offset = self.cursor.offset();
        let line = self.cursor.line();
        let column = self.cursor.column();

        let kind = match self.cursor.peek() {
            None => TokenKind::Eof,
            Some('"') => TokenKind::String { fragments: string::lex(&mut self.cursor)? },
            Some(c) => self.consume(c)?,
        };

        let text = self.cursor.slice(offset);
        Ok(Token::new(
            kind,
            Span::new(line, column, text.chars().count()),
            text,
        ))
    }

    fn consume(&mut self, c: char) -> Result<TokenKind<'static>, Error> {
        let kind = match c {
            '[' => self.single(TokenKind::LeftBracket),
            ']' => self.single(TokenKind::RightBracket),
            '{' => self.single(TokenKind::LeftBrace),
            '}' => self.single(TokenKind::RightBrace),
            ':' => self.single(TokenKind::Colon),
            ',' => self.single(TokenKind::Comma),
            '.' => self.single(TokenKind::Dot),
            '-' => return number::lex(&mut self.cursor),
            '+' if number::is_digit(self.cursor.peek_at(1)) => return Err(Error::new(
                crate::error::SyntaxError::MalformedNumber,
                Span::new(self.cursor.line(), self.cursor.column(), 1),
            )),
            c if c.is_ascii_digit() => return number::lex(&mut self.cursor),
            ' ' | '\t' => {
                self.cursor.take_while(is_inline_space);
                TokenKind::Whitespace
            },
            '\r' | '\n' => {
                self.cursor.bump(); if c == '\r' && self.cursor.peek() == Some('\n') {
                    self.cursor.bump();
                }
                TokenKind::Newline
            },
            '#' => {
                self.cursor.take_while(|c| !is_newline(c));
                TokenKind::Comment
            },
            c if is_identifier_start(c) => {
                self.cursor.bump();
                self.cursor.take_while(is_identifier_continue);
                TokenKind::Identifier
            },
            _ => self.single(TokenKind::Unknown),
        };
        Ok(kind)
    }

    fn single(&mut self, kind: TokenKind<'static>) -> TokenKind<'static> {
        self.cursor.bump();
        kind
    }

    fn diagnostic(&self, failure: Error) -> Diagnostic {
        let diagnostic = Diagnostic::new(failure.kind);
        match Source::new(self.text).excerpt(failure.span) {
            Some(excerpt) => diagnostic.with_excerpt(excerpt),
            None => diagnostic,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Diagnostic>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        match self.token() {
            Ok(token) => {
                self.finished = token.kind == TokenKind::Eof;
                Some(Ok(token))
            },
            Err(failure) => {
                self.finished = true;
                Some(Err(self.diagnostic(failure)))
            }
        }
    }
}

fn is_inline_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_newline(c: char) -> bool {
    c == '\r' || c == '\n'
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::{*, TokenKind::*};

    fn lex(text: &str) -> Vec<Token<'_>> {
        Lexer::new(text)
            .collect::<Result<_, _>>()
            .expect("lexer should never fail for the current phase")
    }

    fn kinds(text: &str) -> Vec<TokenKind<'_>> {
        lex(text).into_iter().map(|t| t.kind).collect()
    }

    fn texts(text: &str) -> Vec<&str> {
        lex(text).iter().map(|t| t.text).collect()
    }

    #[test]
    fn lexes_a_dotted_path_with_a_dash() {
        match Lexer::new("123.abc: _").collect::<Result<Vec<_>, _>>() {
            Ok(tokens) => {dbg!(tokens);},
            Err(diagnostic) => {println!("{}", diagnostic.report());},
        }

        assert_eq!(kinds("a.b-c"), [Identifier, Dot, Identifier, Eof]);
        assert_eq!(texts("a.b-c"), ["a", ".", "b-c", ""]);
    }

    #[test]
    fn a_digit_starts_a_number_not_an_identifier() {
        let tokens = lex("1abc");
        assert_eq!((&tokens[0].kind, tokens[0].text), (&Integer, "1"));
        assert_eq!((&tokens[1].kind, tokens[1].text), (&Identifier, "abc"));
    }

    #[test]
    fn lexes_all_punctuation() {
        assert_eq!(
            kinds("[]{}:,."),
            [LeftBracket, RightBracket, LeftBrace, RightBrace, Colon, Comma, Dot, Eof]
        );
    }

    #[test]
    fn identifiers_allow_underscore_digits_and_dash() {
        assert_eq!(texts("_a-1_b"), ["_a-1_b", ""]);
    }

    #[test]
    fn true_and_false_are_plain_identifiers() {
        assert_eq!(kinds("true false"), [Identifier, Whitespace, Identifier, Eof]);
        assert_eq!(texts("truex"), ["truex", ""]);
    }

    #[test]
    fn whitespace_is_a_token_between_other_tokens() {
        assert_eq!(kinds("a . b"), [Identifier, Whitespace, Dot, Whitespace, Identifier, Eof]);
    }

    #[test]
    fn spaces_and_tabs_form_one_whitespace_token() {
        assert_eq!(texts("a \t b"), ["a", " \t ", "b", ""]);
    }

    #[test]
    fn lexes_all_three_newline_forms() {
        assert_eq!(texts("a\r\nb\nc\rd"), ["a", "\r\n", "b", "\n", "c", "\r", "d", ""]);
        let newlines = kinds("a\r\nb\nc\rd").iter().filter(|k| **k == Newline).count();
        assert_eq!(newlines, 3);
    }

    #[test]
    fn crlf_has_a_length_of_two_characters() {
        assert_eq!(lex("\r\n")[0].span, crate::Span::new(1, 1, 2));
    }

    #[test]
    fn each_line_restarts_at_column_1() {
        let tokens = lex("a\r\nb\nc\rd");
        assert_eq!(tokens[2].span, crate::Span::new(2, 1, 1)); // b
        assert_eq!(tokens[4].span, crate::Span::new(3, 1, 1)); // c
        assert_eq!(tokens[6].span, crate::Span::new(4, 1, 1)); // d
    }

    #[test]
    fn a_comment_stops_before_the_newline() {
        assert_eq!(kinds("# hi\nx"), [Comment, Newline, Identifier, Eof]);
        assert_eq!(texts("# hi\nx")[0], "# hi");
    }

    #[test]
    fn a_comment_can_follow_code() {
        assert_eq!(kinds("a # c"), [Identifier, Whitespace, Comment, Eof]);
    }

    #[test]
    fn a_hash_inside_a_comment_stays_in_the_comment() {
        assert_eq!(texts("# a # b"), ["# a # b", ""]);
    }

    #[test]
    fn unknown_characters_become_unknown_tokens() {
        assert_eq!(kinds("@"), [Unknown, Eof]);
    }

    #[test]
    fn a_non_ascii_character_is_one_unknown_character() {
        let tokens = lex("éa");
        assert_eq!(tokens[0].kind, Unknown);
        assert_eq!(tokens[0].span, crate::Span::new(1, 1, 1));
        assert_eq!(tokens[1].span, crate::Span::new(1, 2, 1));
    }

    #[test]
    fn a_tab_counts_as_one_column() {
        assert_eq!(lex("\tx")[1].span, crate::Span::new(1, 2, 1));
    }

    #[test]
    fn eof_has_a_zero_length_span_at_the_end() {
        assert_eq!(lex("[x").last().unwrap().span, crate::Span::new_point(1, 3));
        assert_eq!(lex("a\n").last().unwrap().span, crate::Span::new_point(2, 1));
    }

    #[test]
    fn empty_text_gives_only_eof() {
        let tokens = lex("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, Eof);
        assert_eq!(tokens[0].text, "");
        assert_eq!(tokens[0].span, crate::Span::new_point(1, 1));
    }

    #[test]
    fn the_iterator_ends_after_eof() {
        let mut lexer = Lexer::new("a");
        assert!(matches!(lexer.next(), Some(Ok(t)) if t.kind == Identifier));
        assert!(matches!(lexer.next(), Some(Ok(t)) if t.kind == Eof));
        assert!(lexer.next().is_none());
        assert!(lexer.next().is_none());
    }

    #[test]
    fn token_texts_rebuild_the_source() {
        let source = "[a.b]\r\nkey : 1 # c\n\t@é";
        assert_eq!(texts(source).concat(), source);
    }
}
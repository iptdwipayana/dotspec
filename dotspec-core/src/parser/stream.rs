use crate::error::Diagnostic;
use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::span::Span;

pub(super) struct TokenStream<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Token<'a>>,
    failure: Option<Diagnostic>,
    end: Span,
    consumed: usize,
}

impl<'a> TokenStream<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self { 
            lexer: Lexer::new(text),
            end: Span::new(1, 1, 0), 
            peeked: None, 
            failure: None,
            consumed: 0,
        }
    }

    pub(super) fn peek(&mut self) -> Result<&Token<'a>, Diagnostic> {
        self.fill()?;
        Ok(self.peeked.as_ref().expect("fill() store a token"))
    }

    pub(super) fn advance(&mut self) -> Result<Token<'a>, Diagnostic> {
        self.fill()?;
        let token = self.peeked.take().expect("fill() store a token");

        self.consumed += token.text.chars().count();
        if !is_spacing(&token.kind) {
            self.end = end_of(&token);
        }
        if matches!(token.kind, TokenKind::Eof) {
            self.peeked = Some(token.clone());
        }
        Ok(token)
    }

    pub(super) fn skip_inline_space(&mut self) -> Result<(), Diagnostic> {
        while matches!(self.peek()?.kind, TokenKind::Whitespace) {
            self.advance()?;
        }
        Ok(())
    }

    pub(super) fn skip_spacing(&mut self) -> Result<(), Diagnostic> {
        while is_spacing(&self.peek()?.kind) {
            self.advance()?;
        }
        Ok(())
    }

    pub(super) fn consumed(&self) -> usize { self.consumed }
    pub(super) fn end(&self) -> Span { self.end }

    fn fill(&mut self) -> Result<(), Diagnostic> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        if self.peeked.is_some() {
            return Ok(());
        }

        let next = self.lexer
            .next()
            .expect("the lexer yields eof before it ends, and the stream keeps eof");
        match next {
            Ok(token) => {
                self.peeked = Some(token);
                Ok(())
            },
            Err(diagnostic) => {
                self.failure = Some(diagnostic.clone());
                Err(diagnostic)
            }
        }
    }
}

fn is_spacing(kind: &TokenKind<'_>) -> bool {
    matches!(
        kind,
        TokenKind::Whitespace | TokenKind::Newline | TokenKind::Comment
    )
}

fn end_of(token: &Token<'_>) -> Span {
    let mut line = token.span.line;
    let mut column = token.span.column;
    let mut chars = token.text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                line += 1;
                column = 1;
            },
            '\r' if chars.peek() != Some(&'\n') => {
                line += 1;
                column = 1;
            },
            _ => column += 1,
        }
    }
    Span::new_point(line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenKind::Colon;
    use crate::lexer::TokenKind::Comment; 
    use crate::lexer::TokenKind::Eof; 
    use crate::lexer::TokenKind::Identifier; 
    use crate::lexer::TokenKind::Integer; 
    use crate::lexer::TokenKind::Newline; 
    use crate::lexer::TokenKind::Whitespace;

    #[test]
    fn advance_yields_tokens_in_order_and_keeps_whitespace() {
        let mut stream = TokenStream::new("a: 1");
        let kinds: Vec<_> = (0..5)
            .map(|_| stream.advance().expect("valid input").kind)
            .collect();
        assert_eq!(kinds, [Identifier, Colon, Whitespace, Integer, Eof]);
    }

    #[test]
    fn peek_does_not_consume() {
        let mut stream = TokenStream::new("a b");
        assert_eq!(stream.peek().expect("valid input").text, "a");
        assert_eq!(stream.peek().expect("valid input").text, "a");
        assert_eq!(stream.advance().expect("valid input").text, "a");
        assert_eq!(stream.peek().expect("valid input").kind, Whitespace);
    }

    #[test]
    fn eof_stays_in_place_after_it_is_read() {
        let mut stream = TokenStream::new("");
        for _ in 0..3 {
            assert_eq!(stream.advance().expect("valid input").kind, Eof);
        }
        assert_eq!(stream.peek().expect("valid input").kind, Eof);
    }

    #[test]
    fn skip_inline_space_skips_spaces_and_tabs_only() {
        let mut stream = TokenStream::new(" \t x");
        stream.skip_inline_space().expect("valid input");
        assert_eq!(stream.peek().expect("valid input").text, "x");

        let mut newline = TokenStream::new(" \n");
        newline.skip_inline_space().expect("valid input");
        assert_eq!(newline.peek().expect("valid input").kind, Newline);

        let mut comment = TokenStream::new(" # c");
        comment.skip_inline_space().expect("valid input");
        assert_eq!(comment.peek().expect("valid input").kind, Comment);
    }

    #[test]
    fn skip_inline_space_without_whitespace_changes_nothing() {
        let mut stream = TokenStream::new("x");
        stream.skip_inline_space().expect("valid input");
        assert_eq!(stream.peek().expect("valid input").text, "x");
    }

    #[test]
    fn skip_spacing_skips_spaces_newlines_and_comments() {
        let mut stream = TokenStream::new(" \n # c\n\tx");
        stream.skip_spacing().expect("valid input");
        assert_eq!(stream.peek().expect("valid input").text, "x");
    }

    #[test]
    fn skip_spacing_stops_at_the_end_of_input() {
        let mut stream = TokenStream::new("  \n ");
        stream.skip_spacing().expect("valid input");
        assert_eq!(stream.peek().expect("valid input").kind, Eof);
    }

    #[test]
    fn a_lexer_error_surfaces_and_sticks() {
        let mut stream = TokenStream::new("a 007");
        stream.advance().expect("a");
        stream.advance().expect("the space");

        let first = stream.peek().expect_err("007 has a leading zero");
        assert_eq!(stream.peek().expect_err("still failing"), first);
        assert_eq!(stream.advance().expect_err("still failing"), first);
    }

    #[test]
    fn consumed_counts_every_character_of_every_token() {
        let mut stream = TokenStream::new("ab \"x\"");
        assert_eq!(stream.consumed(), 0);
        stream.advance().expect("ab");
        assert_eq!(stream.consumed(), 2);
        stream.advance().expect("the space");
        assert_eq!(stream.consumed(), 3);
        stream.advance().expect("the string");
        assert_eq!(stream.consumed(), 6);
        stream.advance().expect("eof");
        assert_eq!(stream.consumed(), 6);
    }

    #[test]
    fn end_starts_at_the_beginning() {
        assert_eq!(TokenStream::new("a").end(), Span::new_point(1, 1));
    }

    #[test]
    fn end_is_right_after_the_last_token_that_is_not_spacing() {
        let mut stream = TokenStream::new("ab  # c\n  cd");
        stream.advance().expect("ab");
        assert_eq!(stream.end(), Span::new_point(1, 3));

        // Spaces, the comment and the newline do not move it.
        stream.skip_spacing().expect("valid input");
        assert_eq!(stream.end(), Span::new_point(1, 3));

        stream.advance().expect("cd");
        assert_eq!(stream.end(), Span::new_point(2, 5));
    }

    #[test]
    fn end_follows_a_token_that_spans_lines() {
        for text in ["\"\"\"\nabc\n\"\"\" x", "\"\"\"\r\nabc\r\n\"\"\" x"] {
            let mut stream = TokenStream::new(text);
            stream.advance().expect("the string");
            assert_eq!(stream.end(), Span::new_point(3, 4), "{text:?}");
        }
    }

    #[test]
    fn consumed_counts_the_lines_of_a_multiline_token() {
        let mut lf = TokenStream::new("\"\"\"\nabc\n\"\"\"");
        lf.advance().expect("the string");
        assert_eq!(lf.consumed(), 11);

        let mut crlf = TokenStream::new("\"\"\"\r\nabc\r\n\"\"\"");
        crlf.advance().expect("the string");
        assert_eq!(crlf.consumed(), 13);
    }
}
use crate::error::Diagnostic;
use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenKind;

pub(super) struct TokenStream<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Token<'a>>,
    failure: Option<Diagnostic>,
}

impl<'a> TokenStream<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self { lexer: Lexer::new(text), peeked: None, failure: None }
    }

    pub(super) fn peek(&mut self) -> Result<&Token<'a>, Diagnostic> {
        self.fill()?;
        Ok(self.peeked.as_ref().expect("fill() store a token"))
    }

    pub(super) fn advance(&mut self) -> Result<Token<'a>, Diagnostic> {
        self.fill()?;
        let token = self.peeked.take().expect("fill() store a token");
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
    fn a_lexer_error_surfaces_and_sticks() {
        let mut stream = TokenStream::new("a 007");
        stream.advance().expect("a");
        stream.advance().expect("the space");

        let first = stream.peek().expect_err("007 has a leading zero");
        assert_eq!(stream.peek().expect_err("still failing"), first);
        assert_eq!(stream.advance().expect_err("still failing"), first);
    }
}
use std::borrow::Cow;

use crate::span::Span;
use super::cursor::Cursor;
use super::error::Error;
use super::interpolation::Interpolation;
use super::escape;
use super::interpolation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fragment<'a> {
    Escape(char, Span),
    Interpolation(Interpolation<'a>, Span),
    Text(Cow<'a, str>, Span),
}

impl Fragment<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Escape(_, span)
                | Self::Interpolation(_, span)
                | Self::Text(_, span) => *span,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Line,
    Multiline,
}

impl Mode {
    fn quotes(&self) -> usize {
        match self {
            Self::Line => 1,
            Self::Multiline => 3,
        }
    }
}

struct Run {
    offset: usize,
    line: usize,
    column: usize,
}

impl Run {
    fn start(cursor: &Cursor<'_>) -> Self {
        Self {
            offset: cursor.offset(),
            line: cursor.line(),
            column: cursor.column(),
        }
    }

    fn flush<'a>(&self, cursor: &Cursor<'a>, fragments: &mut Vec<Fragment<'a>>, trim_newline: bool) {
        let mut text = cursor.slice(self.offset);
        if trim_newline {
            text = strip_newline(text);
        }
        if text.is_empty() {
            return;
        }
        fragments.push(Fragment::Text(
            normalize_newline(text),
            Span::new(self.line, self.column, text.chars().count())
        ))
    }
}

pub(super) fn lex<'a>(cursor: &mut Cursor<'a>) -> Result<Vec<Fragment<'a>>, Error> {
    scan(
        cursor,
        if cursor.peek_at(1) == Some('"') && cursor.peek_at(2) == Some('"') { Mode::Multiline }
        else { Mode::Line }
    )
}

fn scan<'a>(cursor: &mut Cursor<'a>, mode: Mode) -> Result<Vec<Fragment<'a>>, Error> {
    for _ in 0..mode.quotes() {
        cursor.bump();
    }
    if mode == Mode::Multiline {
        skip_newline(cursor);
    }

    let mut fragments = Vec::new();

    let mut run = Run::start(cursor);
    loop {
        match cursor.peek() {
            None => return Err(unclosed(mode, cursor.line(), cursor.column())),
            Some('\r' | '\n') if mode == Mode::Line => {
                return Err(unclosed(mode, cursor.line(), cursor.column()));
            },
            Some('"') if closes(cursor, mode) => {
                run.flush(cursor, &mut fragments, mode == Mode::Multiline);
                for _ in 0..mode.quotes() {
                    cursor.bump();
                }
                return Ok(fragments);
            },
            Some('\\') => {
                run.flush(cursor, &mut fragments, false);
                fragments.push(escape_fragment(cursor, mode)?);
                run = Run::start(cursor);
            },
            Some('#') if cursor.peek_at(1) == Some('{') => {
                run.flush(cursor, &mut fragments, false);
                fragments.push(interpolation_fragment(cursor)?);
                run = Run::start(cursor);
            },
            Some(_) => {
                cursor.bump();
            }
        }
    }
}

fn closes(cursor: &Cursor<'_>, mode: Mode) -> bool {
    mode == Mode::Line || (cursor.peek_at(1) == Some('"') && cursor.peek_at(2) == Some('"'))
}

fn unclosed(mode: Mode, line: usize, column: usize) -> Error {
    Error::new(match mode {
        Mode::Line => crate::error::SyntaxError::UnclosedString,
        Mode::Multiline => crate::error::SyntaxError::UnclosedMultilineString,
    }, Span::new_point(line, column))
}

fn escape_fragment<'a>(cursor: &mut Cursor<'a>, mode: Mode) -> Result<Fragment<'a>, Error> {
    let line = cursor.line();
    let column = cursor.column();
    let offset = cursor.offset();
    let next = cursor.peek_at(1);

    if next.is_none() || (mode == Mode::Line && matches!(next, Some('\r' | '\n'))) {
        return Err(unclosed(mode, line, column + 1));
    }

    let value = escape::lex(cursor)?;
    Ok(Fragment::Escape(value, Span::new(
        line,
        column,
        cursor.slice(offset).chars().count()
    )))
}

fn interpolation_fragment<'a>(cursor: &mut Cursor<'a>) -> Result<Fragment<'a>, Error> {
    let line = cursor.line();
    let column = cursor.column();
    let offset = cursor.offset();
    let value = interpolation::lex(cursor)?;
    Ok(Fragment::Interpolation(value, Span::new(
        line,
        column,
        cursor.slice(offset).chars().count()
    )))
}

fn skip_newline(cursor: &mut Cursor<'_>) {
    match cursor.peek() {
        Some('\r') => {
            cursor.bump();
            if cursor.peek() == Some('\n') {
                cursor.bump();
            }
        },
        Some('\n') => {
            cursor.bump();
        },
        _ => {},
    }
}

fn strip_newline(text: &str) -> &str {
    text.strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .or_else(|| text.strip_suffix('\r'))
        .unwrap_or(text)
}

fn normalize_newline(text: &str) -> Cow<'_, str> {
    if text.contains('\r') {
        Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lex::Lexer;
    use super::super::token::TokenKind;
    use crate::error::Kind;
    use crate::error::SyntaxError;

    fn line(body: &str) -> String {
        format!("\"{body}\"")
    }

    fn multiline(body: &str) -> String {
        format!("\"\"\"{body}\"\"\"")
    }
}
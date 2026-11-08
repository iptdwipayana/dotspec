use super::cursor::Cursor;
use super::error::Error;

pub(super) fn lex(cursor: &mut Cursor<'_>) -> Result<char, Error> {
    let line = cursor.line();
    let column = cursor.column();
    cursor.bump();

    let value = match cursor.peek() {
        Some('"') => '"',
        Some('\\') => '\\',
        Some('/') => '/',
        Some('b') => '\u{8}',
        Some('f') => '\u{c}',
        Some('n') => '\n',
        Some('t') => '\t',
        Some('r') => '\r',
        Some('u') => {
            cursor.bump();
            return unicode(cursor, line, column);
        },
        _ => {
            return Err(Error::new(
                crate::error::SyntaxError::InvalidEscapeSequence, 
                crate::span::Span::new(line, column, 2)
            ))
        },
    };
    cursor.bump();

    Ok(value)
}

fn unicode(cursor: &mut Cursor<'_>, line: usize, column: usize) -> Result<char, Error> {
    let high = hex4(cursor).map_err(|read| {
        Error::new(crate::error::SyntaxError::MalformedUnicodeEscape, crate::span::Span::new(
            line,
            column,
            2 + read
        ))
    })?;
    let code = match high {
        0xD800..=0xDBFF => {
            let low = low_surrogate_ahead(cursor).ok_or_else(|| unpaired(line, column))?;
            for _ in 0..6 {
                cursor.bump();
            }
            0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)
        },
        0xDC00..=0xDFFF => return Err(unpaired(line, column)),
        _ => high,
    };
    char::from_u32(code).ok_or_else(|| unpaired(line, column))
}

fn hex4(cursor: &mut Cursor<'_>) -> Result<u32, usize> {
    let mut value = 0;
    for read in 0..4 {
        match cursor.peek().and_then(|c| c.to_digit(16)) {
            Some(digit) => {
                value = value * 16 + digit;
                cursor.bump();
            },
            None => return Err(read),
        }
    }
    Ok(value)
}

fn low_surrogate_ahead(cursor: &Cursor<'_>) -> Option<u32> {
    if cursor.peek_at(0)? != '\\' || cursor.peek_at(1)? != 'u' {
        return None;
    }
    let mut value = 0;
    for i in 2..6 {
        value = value * 16 + cursor.peek_at(i)?.to_digit(16)?;
    }
    (0xDC00..=0xDFFF).contains(&value).then_some(value)
}

fn unpaired(line: usize, column: usize) -> Error {
    Error::new(
        crate::error::SyntaxError::InvalidSurrogate,
        crate::span::Span::new(line, column, 6),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(text: &str) -> Option<char> {
        lex(&mut Cursor::new(text)).ok()
    }

    #[test]
    fn decodes_every_simple_escape() {
        let table = [
            (r#"\""#, '"'),
            (r"\\", '\\'),
            (r"\/", '/'),
            (r"\b", '\u{8}'),
            (r"\f", '\u{c}'),
            (r"\n", '\n'),
            (r"\r", '\r'),
            (r"\t", '\t'),
        ];
        for (text, decoded) in table {
            assert_eq!(decode(text), Some(decoded), "Decode Failed: {text}");
        }
    }

    #[test]
    fn decodes_unicode_escape_in_any_hex_case() {
        assert_eq!(decode(r"\u00e9"), Some('é'));
        assert_eq!(decode(r"\u00E9"), Some('é'));
        assert_eq!(decode(r"\u0000"), Some('\0'));
        assert_eq!(decode(r"\uD7FF"), Some('\u{D7FF}'));
        assert_eq!(decode(r"\uE000"), Some('\u{E000}'));
    }

    #[test]
    fn a_surrogate_pair_becomes_one_character() {
        assert_eq!(decode(r"\ud83d\ude00"), Some('😀'));
        assert_eq!(decode(r"\uD83D\uDE00"), Some('😀'));
    }

    #[test]
    fn unpaired_surrogates_are_rejected() {
        for text in [
            r"\ud800",
            r"\udbff",
            r"\udc00",
            r"\udfff",
            r"\ud83d\u0041",
            r"\ud83dx",
            r"\ud83d\ude0",
        ] {
            assert_eq!(decode(text), None, "Surrogate Pair Failed: {text}");
        }
    }

    #[test]
    fn malformed_unicode_and_unknown_escapes_are_rejected() {
        for text in [r"\u", r"\u12", r"\u12zz", r"\q", r"\x", r"\'", "\\", "\\é"] {
            assert_eq!(decode(text), None, "Malformed Decode Failed: {text}");
        }
    }

    #[test]
    fn consumes_exactly_the_escape() {
        let mut cursor = Cursor::new(r"\u00e9x");
        assert_eq!(lex(&mut cursor).ok(), Some('é'));
        assert_eq!(cursor.peek(), Some('x'));

        let mut pair = Cursor::new(r"\ud83d\ude00x");
        assert_eq!(lex(&mut pair).ok(), Some('😀'));
        assert_eq!(pair.peek(), Some('x'));

        let mut simple = Cursor::new(r"\nx");
        assert_eq!(lex(&mut simple).ok(), Some('\n'));
        assert_eq!(simple.peek(), Some('x'));
    }
}
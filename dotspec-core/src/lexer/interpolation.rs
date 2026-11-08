use super::cursor::Cursor;
use super::error::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Interpolation<'a> {
    Env { path: &'a str, default: Option<&'a str> },
    Config { path: &'a str },
}

pub(super) fn lex<'a>(cursor: &mut Cursor<'a>) -> Result<Interpolation<'a>, Error> {
    let line = cursor.line();
    let column = cursor.column();
    let offset = cursor.offset();
    cursor.bump();
    cursor.bump();
    let offset_body = cursor.offset();

    loop {
        match cursor.peek() {
            Some('}') => break,
            Some('\r' | '\n') | None => {
                return Err(Error::new(
                    crate::error::SyntaxError::UnclosedInterpolation,
                    crate::span::Span::new_point(cursor.line(), cursor.column())
                ))
            },
            Some(_) => { cursor.bump(); },
        }
    }

    let body = cursor.slice(offset_body); cursor.bump(); 
    match body.strip_prefix("env.") {
        Some(rest) => env(rest, line, column + 6),
        None if is_config_path(body) => Ok(Interpolation::Config { path: body }),
        None => Err(Error::new(
            crate::error::SyntaxError::InvalidInterpolation,
            crate::span::Span::new(
                line,
                column,
                cursor.slice(offset).chars().count()
            )
        ))
    }
}

fn env<'a>(rest: &'a str, line: usize, column: usize) -> Result<Interpolation<'a>, Error> {
    let (path, default) = match rest.split_once(":-") {
        Some((path, default)) => (path, Some(default)),
        None => (rest, None),
    };

    if is_env_path(path) {
        Ok(Interpolation::Env { path, default })
    } else {
        Err(Error::new(
            crate::error::SyntaxError::MalformedEnvReference,
            crate::span::Span::new(line, column, path.chars().count())
        ))
    }
}

fn is_env_path(path: &str) -> bool {
    let mut chars = path.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase() || c == '_') && chars.all(|c| {
        c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
    })
}

fn is_config_path(path: &str) -> bool {
    path.split('.').all(is_identifier)
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_') && chars.all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn parse(text: &str) -> Option<Interpolation<'_>> {
        lex(&mut Cursor::new(text)).ok()
    }

    fn config(path: &str) -> Option<Interpolation<'_>> {
        Some(Interpolation::Config { path })
    }

    fn env<'a>(path: &'a str, default: Option<&'a str>) -> Option<Interpolation<'a>> {
        Some(Interpolation::Env { path, default })
    }

    #[test]
    fn config_references() {
        assert_eq!(parse("#{a}"), config("a"));
        assert_eq!(parse("#{server.host}"), config("server.host"));
        assert_eq!(parse("#{_.app-name}"), config("_.app-name"));
        assert_eq!(parse("#{app-name}"), config("app-name"));
    }

    #[test]
    fn env_references() {
        assert_eq!(parse("#{env.HOME}"), env("HOME", None));
        assert_eq!(parse("#{env._X9}"), env("_X9", None));
    }

    #[test]
    fn a_bare_env_is_a_config_path() {
        assert_eq!(parse("#{env}"), config("env"));
    }

    #[test]
    fn env_defaults() {
        assert_eq!(
            parse("#{env.LOG_DIR:-/var/log/app}"),
            env("LOG_DIR", Some("/var/log/app"))
        );
        assert_eq!(parse("#{env.X:-}"), env("X", Some("")));
        assert_eq!(parse("#{env.X:-a:-b#c}"), env("X", Some("a:-b#c")));
    }

    #[test]
    fn the_body_ends_at_the_first_closing_brace() {
        assert_eq!(parse("#{env.X:-a}b}"), env("X", Some("a")));
    }

    #[test]
    fn consumes_exactly_the_interpolation() {
        let mut cursor = Cursor::new("#{a.b}x");
        assert_eq!(lex(&mut cursor).ok(), config("a.b"));
        assert_eq!(cursor.peek(), Some('x'));
    }

    #[test]
    fn invalid_bodies_are_rejected() {
        let bad = [
            "#{}",
            "#{a..b}",
            "#{a b}",
            "#{ a.b }",
            "#{1a}",
            "#{a.}",
            "#{.a}",
            "#{env.lower}",
            "#{env.}",
            "#{env.NAME:x}",
            "#{env.A-B}",
            "#{env.1A}",
        ];
        for text in bad {
            assert_eq!(parse(text), None, "{text}");
        }
    }

    #[test]
    fn unclosed_bodies_are_rejected() {
        for text in ["#{a.b", "#{", "#{a\nb}", "#{a\r\nb}", "#{env.X:-a\n}"] {
            assert_eq!(parse(text), None, "{text:?}");
        }
    }
}
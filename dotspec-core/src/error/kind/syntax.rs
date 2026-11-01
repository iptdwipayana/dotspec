use std::borrow::Cow;
use super::traits::ErrorCode;
use super::traits::ErrorDescription;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnclosedSection,
    MissingSectionPath,
    MalformedSectionPath,
    MissingColon,
    MissingValue,
    InvalidValue,
    InvalidStatementStart,
    UnexpectedTrailingContent,
    MalformedNumber,
    UnexpectedLeadingZero,
    FloatOutOfRange,
    IntegerOutOfRange,
    UnclosedString,
    UnclosedMultilineString,
    InvalidEscapeSequence,
    MalformedUnicodeEscape,
    InvalidSurrogate,
    InvalidInterpolation,
    UnclosedInterpolation,
    MalformedEnvReference,
    UnclosedArray,
    MissingArrayComma,
    UnexpectedArrayComma,
    UnclosedObject,
    InvalidObjectKey,
    MissingObjectComma,
    UnexpectedObjectComma,
}

impl ErrorCode for Error {
    fn code(&self) -> &'static str {
        match self {
            Self::UnclosedSection => "SYN001",
            Self::MissingSectionPath => "SYN002",
            Self::MalformedSectionPath => "SYN003",
            Self::MissingColon => "SYN011",
            Self::MissingValue => "SYN012",
            Self::InvalidValue => "SYN013",
            Self::InvalidStatementStart => "SYN021",
            Self::UnexpectedTrailingContent => "SYN022",
            Self::MalformedNumber => "SYN031",
            Self::UnexpectedLeadingZero => "SYN032",
            Self::FloatOutOfRange => "SYN033",
            Self::IntegerOutOfRange => "SYN034",
            Self::UnclosedString => "SYN041",
            Self::UnclosedMultilineString => "SYN042",
            Self::InvalidEscapeSequence => "SYN051",
            Self::MalformedUnicodeEscape => "SYN052",
            Self::InvalidSurrogate => "SYN053",
            Self::InvalidInterpolation => "SYN061",
            Self::UnclosedInterpolation => "SYN062",
            Self::MalformedEnvReference => "SYN063",
            Self::UnclosedArray => "SYN071",
            Self::MissingArrayComma => "SYN072",
            Self::UnexpectedArrayComma => "SYN073",
            Self::UnclosedObject => "SYN081",
            Self::InvalidObjectKey => "SYN082",
            Self::MissingObjectComma => "SYN083",
            Self::UnexpectedObjectComma => "SYN084",
        }
    }
}

impl ErrorDescription for Error {
    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::UnclosedSection => "unclosed section",
            Self::MissingSectionPath => "missing section path",
            Self::MalformedSectionPath => "malformed section path",
            Self::MissingColon => "missing `:` after key",
            Self::MissingValue => "missing value",
            Self::InvalidValue => "invalid value",
            Self::InvalidStatementStart => "invalid statement start",
            Self::UnexpectedTrailingContent => "unexpected content after statement",
            Self::MalformedNumber => "malformed number",
            Self::UnexpectedLeadingZero => "leading zero in number",
            Self::IntegerOutOfRange => "integer out of range",
            Self::FloatOutOfRange => "float out of range",
            Self::UnclosedString => "unclosed string",
            Self::UnclosedMultilineString => "unclosed multiline string",
            Self::InvalidEscapeSequence => "invalid escape sequence",
            Self::MalformedUnicodeEscape => "malformed unicode escape",
            Self::InvalidSurrogate => "unpaired surrogate",
            Self::InvalidInterpolation => "invalid interpolation",
            Self::UnclosedInterpolation => "unclosed interpolation",
            Self::MalformedEnvReference => "malformed environment reference",
            Self::UnclosedArray => "unclosed array",
            Self::MissingArrayComma => "missing comma in array",
            Self::UnexpectedArrayComma => "unexpected comma in array",
            Self::UnclosedObject => "unclosed object",
            Self::InvalidObjectKey => "invalid object key",
            Self::MissingObjectComma => "missing comma in object",
            Self::UnexpectedObjectComma => "unexpected comma in object",
        })
    }

    fn label(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(match self {
            Self::UnclosedSection => "expect a closing `]` on section header",
            Self::MissingSectionPath => "expect a path inside the brackets",
            Self::MalformedSectionPath => "expect identifiers separated by `.`",
            Self::MissingColon => "expect `:` here",
            Self::MissingValue => "expect a value here",
            Self::InvalidValue => "not a valid value",
            Self::InvalidStatementStart => "expect a section or a key",
            Self::UnexpectedTrailingContent => "expect end of line",
            Self::MalformedNumber => "not a valid number",
            Self::UnexpectedLeadingZero => "remove the leading zero",
            Self::IntegerOutOfRange => "does not fit in a 64-bit signed integer",
            Self::FloatOutOfRange => "too large for a 64-bit float",
            Self::UnclosedString => "expect a closing `\"` here",
            Self::UnclosedMultilineString => "expect a closing `\"\"\"`",
            Self::InvalidEscapeSequence => "unknown escape",
            Self::MalformedUnicodeEscape => "expect four hex digits",
            Self::InvalidSurrogate => "surrogate has no matching pair",
            Self::InvalidInterpolation => "expect a config path or `env.NAME`",
            Self::UnclosedInterpolation => "expect a closing `}`",
            Self::MalformedEnvReference => "environment names use A-Z, 0-9 and `_`",
            Self::UnclosedArray => "expect a closing `]`",
            Self::MissingArrayComma => "expect `,` here",
            Self::UnexpectedArrayComma => "remove this comma",
            Self::UnclosedObject => "expect a closing `}`",
            Self::InvalidObjectKey => "expect an identifier",
            Self::MissingObjectComma => "expect `,` here",
            Self::UnexpectedObjectComma => "remove this comma",
        }))
    }

    fn help(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(match self {
            Self::UnclosedSection => "valid section header is `[key.subkey]`",
            Self::MissingSectionPath => "valid section header is `[key.subkey]`",
            Self::MalformedSectionPath => "an identifier starts with a letter or `_`, then letters, digits, `_` or `-`",
            Self::MissingColon => "write an entry as `key: value`",
            Self::MissingValue => "a value is a string, number, boolean, array, or object",
            Self::InvalidValue => "a value is a string, number, boolean, array, or object",
            Self::InvalidStatementStart => "start a line with `[section]` or `key: value`",
            Self::UnexpectedTrailingContent => "put one statement per line",
            Self::MalformedNumber => "numbers look like `42`, `-5`, `2.5` or `1e3`; a leading `+` is not allowed",
            Self::UnexpectedLeadingZero => "write `7` instead of `007`",
            Self::IntegerOutOfRange => "use a smaller value, or write it as a float or a string",
            Self::FloatOutOfRange => "use a smaller exponent",
            Self::UnclosedString => "strings must end on the same line; use `\"\"\"` for multiple lines",
            Self::UnclosedMultilineString => "end the string with `\"\"\"`",
            Self::InvalidEscapeSequence => "valid escapes are `\\\"` `\\\\` `\\/` `\\b` `\\f` `\\n` `\\r` `\\t` and `\\uXXXX`",
            Self::MalformedUnicodeEscape => "write `\\u` followed by four hex digits, like `\\u00e9`",
            Self::InvalidSurrogate => "write both halves together, like `\\ud83d\\ude00`",
            Self::InvalidInterpolation => "use `#{section.key}` or `#{env.NAME:-default}`",
            Self::UnclosedInterpolation => "use `#{section.key}` or `#{env.NAME:-default}`",
            Self::MalformedEnvReference => "use `#{env.NAME}` or `#{env.NAME:-default}`",
            Self::UnclosedArray => "end the array with `]`",
            Self::MissingArrayComma => "separate array items with commas",
            Self::UnexpectedArrayComma => "a comma must follow an item; only one trailing comma is allowed",
            Self::UnclosedObject => "end the object with `}`",
            Self::InvalidObjectKey => "keys start with a letter or `_` and are not quoted",
            Self::MissingObjectComma => "separate object entries with commas",
            Self::UnexpectedObjectComma => "a comma must follow an entry; only one trailing comma is allowed",
        }))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::traits::ErrorCode;
    use super::super::traits::ErrorDescription;

    fn all() -> Vec<Error> {
        vec![
            Error::UnclosedSection,
            Error::MissingSectionPath,
            Error::MalformedSectionPath,
            Error::MissingColon,
            Error::MissingValue,
            Error::InvalidValue,
            Error::InvalidStatementStart,
            Error::UnexpectedTrailingContent,
            Error::MalformedNumber,
            Error::UnexpectedLeadingZero,
            Error::FloatOutOfRange,
            Error::IntegerOutOfRange,
            Error::UnclosedString,
            Error::UnclosedMultilineString,
            Error::InvalidEscapeSequence,
            Error::MalformedUnicodeEscape,
            Error::InvalidSurrogate,
            Error::InvalidInterpolation,
            Error::UnclosedInterpolation,
            Error::MalformedEnvReference,
            Error::UnclosedArray,
            Error::MissingArrayComma,
            Error::UnexpectedArrayComma,
            Error::UnclosedObject,
            Error::InvalidObjectKey,
            Error::MissingObjectComma,
            Error::UnexpectedObjectComma,
        ]
    }

    #[test]
    fn error_codes_are_unique() {
        assert_eq!(
            all().iter().map(|e| e.code()).collect::<std::collections::HashSet<_>>().len(),
            all().len(),
        );
    }

    #[test]
    fn error_codes_are_using_the_correct_prefix() {
        for e in all() {
            let code = e.code();
            assert_eq!(code.len(), 6);
            assert!(code.starts_with("SYN"), "Invalid code prefix: {code}");
            assert!(code[3..].chars().all(|c| c.is_ascii_digit()), "Invalid code number: {code}");
        }
    }

    #[test]
    fn every_variant_has_implemented_description() {
        for e in all() {
            assert!(e.message().len() > 0, "Missing message: {e:?}");
            assert!(e.label().is_some(), "Expected label to return None: {e:?}");
            assert!(e.help().is_some(), "Expected help to return Some: {e:?}");
        }
    }
}
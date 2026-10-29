use std::borrow::Cow;
use super::traits::ErrorCode;
use super::traits::ErrorDescription;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    FileNotFound,
    IsADirectory,
    InvalidExtension,
    FilePermissionDenied,
    InvalidEncoding,
    Other { cause: String },
}

impl ErrorCode for Error {
    fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound => "IO001",
            Self::IsADirectory => "IO002",
            Self::InvalidExtension => "IO003",
            Self::FilePermissionDenied => "IO011",
            Self::InvalidEncoding => "IO021",
            Self::Other { .. } => "IO099",
        }
    }
}

impl ErrorDescription for Error {
    fn message(&self) -> Cow<'static, str> {
        match self {
            Self::FileNotFound => Cow::Borrowed("file not found"),
            Self::IsADirectory => Cow::Borrowed("is a directory"),
            Self::InvalidExtension => Cow::Borrowed("invalid extension"),
            Self::FilePermissionDenied => Cow::Borrowed("file permission denied"),
            Self::InvalidEncoding => Cow::Borrowed("invalid encoding"),
            Self::Other { cause } => Cow::Owned(format!("io error: {}", cause)), 
        }
    }

    fn label(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn help(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::FileNotFound => Some(Cow::Borrowed("make sure the file exists")),
            Self::IsADirectory => Some(Cow::Borrowed("make sure the file is not a directory")),
            Self::InvalidExtension => Some(Cow::Borrowed("save the file as `.spec`")),
            Self::FilePermissionDenied => Some(Cow::Borrowed("make sure the file is readable")),
            Self::InvalidEncoding => Some(Cow::Borrowed("save the file as UTF-8")),
            Self::Other { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Vec<Error> {
        vec![
            Error::FileNotFound,
            Error::IsADirectory,
            Error::InvalidExtension,
            Error::FilePermissionDenied,
            Error::InvalidEncoding,
            Error::Other { cause: String::from("disk error") },
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
            assert_eq!(code.len(), 5);
            assert!(code.starts_with("IO"), "Invalid code prefix: {code}");
            assert!(code[3..].chars().all(|c| c.is_ascii_digit()), "Invalid code number: {code}");
        }
    }

    #[test]
    fn every_variant_has_implemented_description() {
        for e in all().iter().filter(|e| !matches!(e, Error::Other { .. })) {
            assert!(e.message().len() > 0, "Missing message: {e:?}");
            assert!(e.label().is_none(), "Expected label to return None: {e:?}");
            assert!(e.help().is_some(), "Expected help to return Some: {e:?}");
        }
    }

    #[test]
    fn io_error_for_other_includes_the_cause() {
        assert_eq!(
            Error::Other { cause: String::from("disk error") }.message(),
            "io error: disk error",
        );
    }
}
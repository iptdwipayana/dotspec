#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    FileNotFound,
    FilePermissionDenied,
    IsADirectory,

    InvalidEncoding,
    InvalidExtension,

    Other { cause: String }
}

impl super::traits::ErrorCode for Error {
    fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound => "IO000",
            Self::IsADirectory => "IO001",
            Self::InvalidExtension => "IO002",

            Self::FilePermissionDenied => "IO010",

            Self::InvalidEncoding => "IO020",

            Self::Other { .. } => "IO099",
        }
    }
}

impl super::traits::ErrorDescription for Error {
    fn message(&self) -> std::borrow::Cow<'static, str> {
        use std::borrow::Cow;
        match self {
            Self::FileNotFound => Cow::Borrowed("file not found"),
            Self::FilePermissionDenied => Cow::Borrowed("file permission denied"),
            Self::IsADirectory => Cow::Borrowed("is a directory"),

            Self::InvalidEncoding => Cow::Borrowed("invalid encoding"),
            Self::InvalidExtension => Cow::Borrowed("invalid extension"),

            Self::Other { cause } => Cow::Owned(format!("io error: {}", cause)), 
        }
    }

    fn label(&self) -> Option<std::borrow::Cow<'static, str>> {
        None
    }

    fn help(&self) -> Option<std::borrow::Cow<'static, str>> {
        use std::borrow::Cow;
        match self {
            Self::FileNotFound => Some(Cow::Borrowed("make sure the file exists")),
            Self::FilePermissionDenied => Some(Cow::Borrowed("make sure the file is readable")),
            Self::IsADirectory => Some(Cow::Borrowed("make sure the file is not a directory")),

            Self::InvalidEncoding => Some(Cow::Borrowed("save the file as UTF-8")),
            Self::InvalidExtension => Some(Cow::Borrowed("save the file as `.spec`")),

            Self::Other { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::traits::ErrorCode;
    use super::super::traits::ErrorDescription;

    fn all() -> Vec<Error> {
        vec![
            Error::FileNotFound,
            Error::IsADirectory,
            Error::InvalidExtension,
            Error::FilePermissionDenied,
            Error::InvalidEncoding,
            Error::Other { cause: "disk error".into() },
        ]
    }

    #[test]
    fn err_codes_are_unique() {
        assert_eq!(
            all().iter().map(|e| e.code()).collect::<std::collections::HashSet<_>>().len(),
            all().len()
        );
    }

    #[test]
    fn err_codes_use_correct_prefix() {
        for e in all() {
            let code = e.code();
            assert_eq!(code.len(), 5);
            assert!(code.starts_with("IO"), "{code}");
            assert!(code[3..].chars().all(|c| c.is_ascii_digit()), "{code}");
        }
    }

    #[test]
    fn every_variant_has_implemented_description() {
        let mut all = all(); all.retain(|e| !matches!(e, Error::Other { .. }));
        for e in all {
            assert!(!e.message().is_empty(), "{:?}", e);
            assert!(e.label().is_none(), "{:?}", e);
            assert!(e.help().is_some(), "{:?}", e);
        }
    }

    #[test]
    fn err_other_message_includes_the_cause() {
        assert_eq!(Error::Other { cause: "disk error".into() }.message(), "io error: disk error");
    }
}
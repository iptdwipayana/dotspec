use std::borrow::Cow;
use super::traits::ErrorCode;
use super::traits::ErrorDescription;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    DuplicatePath { path: String },
    DuplicateSection { path: String },
    ConflictingPath { path: String },
    DuplicateObjectKey { key: String },
    ReservedName { name: String },
    MissingReference { path: String },
    InvalidReferenceTarget { path: String },
    InvalidReferenceCycle { cycle: Vec<String> },
    MissingEnvVar { name: String },
    InvalidEnvVar { name: String },
}

impl ErrorCode for Error {
    fn code(&self) -> &'static str {
        match self {
            Self::DuplicatePath { .. } => "SEM001",
            Self::DuplicateSection { .. } => "SEM002",
            Self::ConflictingPath { .. } => "SEM003",
            Self::DuplicateObjectKey { .. } => "SEM004",
            Self::ReservedName { .. } => "SEM005",
            Self::MissingReference { .. } => "SEM011",
            Self::InvalidReferenceTarget { .. } => "SEM012",
            Self::InvalidReferenceCycle { .. } => "SEM013",
            Self::MissingEnvVar { .. } => "SEM021",
            Self::InvalidEnvVar { .. } => "SEM022",
        }
    }
}

impl ErrorDescription for Error {
    fn message(&self) -> Cow<'static, str> {
        Cow::Owned(match self {
            Self::DuplicatePath { path } => format!("duplicate path `{path}`"),
            Self::DuplicateSection { path } => format!("duplicate section `{path}`"),
            Self::ConflictingPath { path } => format!("conflicting path `{path}`"),
            Self::DuplicateObjectKey { key } => format!("duplicate object key `{key}`"),
            Self::ReservedName { name } => format!("reserved name `{name}`"),
            Self::MissingReference { path } => format!("missing reference `{path}`"),
            Self::InvalidReferenceTarget { path } => format!("invalid reference target `{path}`"),
            Self::InvalidReferenceCycle { cycle } => format!("invalid reference cycle: {}", cycle.join(" -> ")),
            Self::MissingEnvVar { name } => format!("missing environment variable `{name}`"),
            Self::InvalidEnvVar { name } => format!("invalid environment variable `{name}`"),
        })
    }

    fn label(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(match self {
            Self::DuplicatePath { .. } => "already defined",
            Self::DuplicateSection { .. } => "section already has its own keys",
            Self::ConflictingPath { .. } => "a key and a section share this path",
            Self::DuplicateObjectKey { .. } => "key already used in this object",
            Self::ReservedName { .. } => "reserved name",
            Self::MissingReference { .. } => "no such path",
            Self::InvalidReferenceTarget { .. } => "expect a string, number, or boolean",
            Self::InvalidReferenceCycle { .. } => "part of a cycle",
            Self::MissingEnvVar { .. } => "not set",
            Self::InvalidEnvVar { .. } => "value cannot be used",
        }))
    }

    fn help(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(match self {
            Self::DuplicatePath { .. } => "each path can be defined only once",
            Self::DuplicateSection { .. } => "merge the keys into one section header",
            Self::ConflictingPath { .. } => "rename the key or the section",
            Self::DuplicateObjectKey { .. } => "remove or rename one of the keys",
            Self::ReservedName { .. } => "`_` holds entries written before any section; choose another name",
            Self::MissingReference { .. } => "check the path; entries before any section live under `_`",
            Self::InvalidReferenceTarget { .. } => "interpolation cannot point to arrays, objects or array items",
            Self::InvalidReferenceCycle { .. } => "break the cycle so values do not depend on each other",
            Self::MissingEnvVar { .. } => "set the variable, or add a default: `#{env.NAME:-default}`",
            Self::InvalidEnvVar { .. } => "set the variable to a UTF-8 value",
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Vec<Error> {
        vec![
            Error::DuplicatePath { path: "a.x".into() },
            Error::DuplicateSection { path: "a".into() },
            Error::ConflictingPath { path: "a.x".into() },
            Error::DuplicateObjectKey { key: "a".into() },
            Error::ReservedName { name: "_".into() },
            Error::MissingReference { path: "a.b".into() },
            Error::InvalidReferenceTarget { path: "lists.ports".into() },
            Error::InvalidReferenceCycle { cycle: vec!["a".into(), "b".into(), "a".into()] },
            Error::MissingEnvVar { name: "HOME".into() },
            Error::InvalidEnvVar { name: "HOME".into() },
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
            assert!(code.starts_with("SEM"), "Invalid code prefix: {code}");
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
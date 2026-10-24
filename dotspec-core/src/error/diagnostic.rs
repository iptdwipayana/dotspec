#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    kind: super::ErrorKind,
    excerpt: Option<super::Excerpt>,
    file: Option<std::path::PathBuf>,
}

impl Diagnostic {
    pub fn new(kind: impl Into<super::ErrorKind>) -> Self {
        Self {
            kind: kind.into(),
            excerpt: None,
            file: None
        }
    }

    pub fn with_excerpt(mut self, excerpt: super::Excerpt) -> Self {
        self.excerpt = Some(excerpt);
        self
    }

    pub fn with_file(mut self, file: impl Into<std::path::PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn kind(&self) -> &super::ErrorKind { &self.kind }
    pub fn excerpt(&self) -> Option<&super::Excerpt> { self.excerpt.as_ref() }
    pub fn file(&self) -> Option<&std::path::Path> { self.file.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ErrorKind,
        IoError,
        SemanticError,
        SyntaxError,
    };

    #[test]
    fn new_has_no_file_and_no_excerpt() {
        let diag = Diagnostic::new(SyntaxError::UnclosedSection);
        assert!(diag.file().is_none());
        assert!(diag.excerpt().is_none());
    }

    #[test]
    fn new_accepts_kind() {
        let kind = ErrorKind::Io(IoError::FileNotFound);
        let diag = Diagnostic::new(kind.clone());
        assert_eq!(diag.kind(), &kind);
    }

    #[test]
    fn new_wraps_each_kind() {
        let io = Diagnostic::new(IoError::FileNotFound);
        let sem = Diagnostic::new(SemanticError::DuplicatePath { path: "foo".into() });
        let syn = Diagnostic::new(SyntaxError::UnclosedSection);

        assert!(matches!(io.kind(), ErrorKind::Io(_)));
        assert!(matches!(sem.kind(), ErrorKind::Semantic(_)));
        assert!(matches!(syn.kind(), ErrorKind::Syntax(_)));
    }

    #[test]
    fn with_file_stores_path() {
        let d = Diagnostic::new(IoError::FileNotFound).with_file("foo");
        assert_eq!(d.file(), Some(std::path::Path::new("foo")));
    }

    #[test]
    fn with_excerpt_stores_span_and_snippet() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection).with_excerpt(crate::Excerpt::new(
            "foo",
            crate::Span::new(1, 1, 1),
        ));
        let e = d.excerpt().expect("excerpt was set");
        assert_eq!(e.snippet(), "foo");
        assert_eq!(e.span(), crate::Span::new(1, 1, 1));
    }

    #[test]
    fn method_chaining_works() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection)
            .with_file("config.spec")
            .with_excerpt(crate::Excerpt::new(
                "[foo",
                crate::Span::new(1, 3, 1),
            ));
        let f = d.file();
        let e = d.excerpt().unwrap();

        assert_eq!(f, Some(std::path::Path::new("config.spec")));
        assert_eq!(e.snippet(), "[foo");
        assert_eq!(e.span(), crate::Span::new(1, 3, 1));
    }

    #[test]
    fn later_file_replaces_the_earlier_one() {
        let d = Diagnostic::new(IoError::FileNotFound)
            .with_file("first.spec")
            .with_file("second.spec");
        assert_eq!(d.file(), Some(std::path::Path::new("second.spec")));
    }

    #[test]
    fn later_excerpt_replaces_the_earlier_one() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection)
            .with_excerpt(crate::Excerpt::new("[x", crate::Span::new_point(1, 3)))
            .with_excerpt(crate::Excerpt::new("[y", crate::Span::new_point(2, 4)));
        assert_eq!(d.excerpt().unwrap().span(), crate::Span::new_point(2, 4));
    }
}
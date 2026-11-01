use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use super::excerpt::Excerpt;
use super::kind::Kind;
use super::render::Render;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    kind: Kind,
    excerpt: Option<Excerpt>,
    path: Option<PathBuf>,
}

impl Diagnostic {
    pub fn new(kind: impl Into<Kind>) -> Self {
        Self { kind: kind.into(), excerpt: None, path: None }
    }

    pub fn with_excerpt(mut self, excerpt: Excerpt) -> Self {
        self.excerpt = Some(excerpt);
        self
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn kind(&self) -> &Kind { &self.kind }
    pub fn excerpt(&self) -> Option<&Excerpt> { self.excerpt.as_ref() }
    pub fn path(&self) -> Option<&Path> { self.path.as_deref() }

    pub fn render(&self) -> Render<'_> {
        Render::new(self)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.render(), f)
    }
}

impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::Span;
    use super::super::IoError;
    use super::super::SemanticError;
    use super::super::SyntaxError;

    #[test]
    fn new_has_no_file_and_no_excerpt() {
        let diag = Diagnostic::new(SyntaxError::UnclosedSection);
        assert!(diag.path().is_none());
        assert!(diag.excerpt().is_none());
    }

    #[test]
    fn new_accepts_kind() {
        let kind = Kind::Io(IoError::FileNotFound);
        let diag = Diagnostic::new(kind.clone());
        assert_eq!(diag.kind(), &kind);
    }

    #[test]
    fn new_wraps_each_kind() {
        let io = Diagnostic::new(IoError::FileNotFound);
        let sem = Diagnostic::new(SemanticError::DuplicatePath { path: "foo".into() });
        let syn = Diagnostic::new(SyntaxError::UnclosedSection);

        assert!(matches!(io.kind(), Kind::Io(_)));
        assert!(matches!(sem.kind(), Kind::Semantic(_)));
        assert!(matches!(syn.kind(), Kind::Syntax(_)));
    }

    #[test]
    fn with_file_stores_path() {
        let d = Diagnostic::new(IoError::FileNotFound).with_path("foo");
        assert_eq!(d.path(), Some(Path::new("foo")));
    }

    #[test]
    fn with_excerpt_stores_span_and_snippet() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection).with_excerpt(Excerpt::new(
            "foo",
            Span::new(1, 1, 1),
        ));
        let e = d.excerpt().expect("excerpt was set");
        assert_eq!(e.snippet(), "foo");
        assert_eq!(e.span(), Span::new(1, 1, 1));
    }

    #[test]
    fn method_chaining_works() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection)
            .with_path("config.spec")
            .with_excerpt(Excerpt::new("[foo", Span::new(1, 3, 1)));
        let f = d.path();
        let e = d.excerpt().unwrap();

        assert_eq!(f, Some(Path::new("config.spec")));
        assert_eq!(e.snippet(), "[foo");
        assert_eq!(e.span(), Span::new(1, 3, 1));
    }

    #[test]
    fn later_file_replaces_the_earlier_one() {
        let d = Diagnostic::new(IoError::FileNotFound)
            .with_path("first.spec")
            .with_path("second.spec");
        assert_eq!(d.path(), Some(Path::new("second.spec")));
    }

    #[test]
    fn later_excerpt_replaces_the_earlier_one() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection)
            .with_excerpt(Excerpt::new("[x", Span::new_point(1, 3)))
            .with_excerpt(Excerpt::new("[y", Span::new_point(2, 4)));
        assert_eq!(d.excerpt().unwrap().span(), Span::new_point(2, 4));
    }

    #[test]
    fn render_works() {
        let d = Diagnostic::new(SyntaxError::UnclosedSection) 
            .with_path("path/to/file.spec")
            .with_excerpt(Excerpt::new("[x", Span::new_point(10, 10)));
        assert_eq!(d.to_string(), d.render().to_string());
    }

    #[test]
    fn diagnostic_works_as_dynamic_std_error_object() {
        fn __(_: &dyn std::error::Error) {} __(&Diagnostic::new(SyntaxError::UnclosedSection));
    }
}
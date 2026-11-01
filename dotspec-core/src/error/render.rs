use std::fmt;
use crate::span::Span;
use super::diagnostic::Diagnostic;
use super::kind::ErrorCode;
use super::kind::ErrorDescription;

pub struct Render<'a> {
    diagnostic: &'a Diagnostic,
}

impl<'a> Render<'a> {
    pub(crate) fn new(diagnostic: &'a Diagnostic) -> Self {
        Self { diagnostic }
    }
}

impl fmt::Display for Render<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.diagnostic.kind();

        write!(f, "error[{}](", kind.code())?;
        match self.diagnostic.path() {
            Some(path) => write!(f, "{}", path.display())?,
            None => write!(f, "<input>")?,
        }
        if let Some(excerpt) = self.diagnostic.excerpt() {
            let Span { line, column, .. } = excerpt.span();
            write!(f, ":{}:{}", line, column)?;
        }
        write!(f, "): {}", kind.message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::kind::Kind;
    use super::super::kind::IoError;
    use super::super::kind::SemanticError;
    use super::super::kind::SyntaxError;
    use super::super::excerpt::Excerpt;

    fn diag(kind: impl Into<Kind>) -> Diagnostic {
        Diagnostic::new(kind.into())
    }

    #[test]
    fn io_error_has_no_line_and_column() {
        let d = diag(IoError::FileNotFound)
            .with_path("path/to/file.spec");
        let d_expected = format!(
            "error[{}](path/to/file.spec): file not found",
            d.kind().code(),
        );
        assert_eq!(Render::new(&d).to_string(), d_expected);
    }

    #[test]
    fn semantic_error_with_path() {
        let d = diag(SemanticError::DuplicatePath { path: "foo".into() })
            .with_path("path/to/file.spec")
            .with_excerpt(Excerpt::new("foo", Span::new_point(10, 10)));
        let d_expected = format!(
            "error[{}](path/to/file.spec:10:10): duplicate path `foo`",
            d.kind().code(),
        );
        assert_eq!(Render::new(&d).to_string(), d_expected);
    }

    #[test]
    fn semantic_error_without_path() {
        let d = diag(SemanticError::DuplicatePath { path: "foo".into() })
            .with_excerpt(Excerpt::new("foo", Span::new_point(10, 10)));
        let d_expected = format!(
            "error[{}](<input>:10:10): duplicate path `foo`",
            d.kind().code(),
        );
        assert_eq!(Render::new(&d).to_string(), d_expected);
    }

    #[test]
    fn semantic_err_uses_the_carried_data() {
        let d = diag(SemanticError::ReservedName { name: String::from("_") });
        let d_rendered = Render::new(&d).to_string();
        assert!(
            d_rendered.ends_with("reserved name `_`"), 
            "Expected the rendered diagnostic to end with the carried data, but got: {}", 
            d_rendered
        );
    }

    #[test]
    fn syntax_error_with_path() {
        let d = diag(SyntaxError::UnclosedSection)
            .with_path("path/to/file.spec")
            .with_excerpt(Excerpt::new("[x", Span::new_point(10, 10)));
        let d_expected = format!(
            "error[{}](path/to/file.spec:10:10): unclosed section",
            d.kind().code(),
        );
        assert_eq!(Render::new(&d).to_string(), d_expected);
    }

    #[test]
    fn syntax_error_without_path() {
        let d = diag(SyntaxError::UnclosedSection)
            .with_excerpt(Excerpt::new("[x", Span::new_point(10, 10)));
        let d_expected = format!(
            "error[{}](<input>:10:10): unclosed section",
            d.kind().code(),
        );
        assert_eq!(Render::new(&d).to_string(), d_expected);
    }

    #[test]
    fn render_output_is_a_single_line() {
        assert!(
            !Render::new(
                &diag(SyntaxError::UnclosedSection)
                    .with_path("path/to/file.spec")
                    .with_excerpt(Excerpt::new("[x", Span::new_point(10, 10)))
            ).to_string().contains('\n')
        );
    }
}
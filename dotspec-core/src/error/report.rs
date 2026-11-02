use std::fmt;
use crate::span::Span;
use super::diagnostic::Diagnostic;
use super::kind::ErrorCode;
use super::kind::ErrorDescription;

pub struct Report<'a> {
    diagnostic: &'a Diagnostic,
}

impl<'a> Report<'a> {
    pub(crate) fn new(diagnostic: &'a Diagnostic) -> Self {
        Self { diagnostic }
    }
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.diagnostic;
        let kind = d.kind();
        let excerpt = d.excerpt();

        let width: usize = excerpt.map_or(1, |e| {
            e.span().line.checked_ilog10().map_or(1, |n| n as usize + 1)
        });
        let pad = " ".repeat(width);

        write!(f, "error[{}]: {}", kind.code(), kind.message())?;
        write!(f, "\n{pad}--> ")?;
        match d.path() {
            Some(path) => write!(f, "{}", path.display())?,
            None => write!(f, "<input>")?,
        }
        if let Some(excerpt) = excerpt {
            let span = excerpt.span();
            let (offset, carets) = resolve_carets_layout(excerpt.snippet(), span);
            write!(f, "\n{pad} |")?;
            write!(f, "\n{:>width$} | {}", span.line, excerpt.snippet().replace('\t', &" ".repeat(4)))?;
            write!(f, "\n{pad} | {}{}", " ".repeat(offset), "^".repeat(carets))?;
            if let Some(label) = kind.label() {
                write!(f, " {label}")?;
            }
            if kind.help().is_some() { 
                write!(f, "\n{pad} |")?;
            }
        }
        if let Some(help) = kind.help() {
            write!(f, "\n{pad} = help: {help}")?;
        }
        Ok(())
    }
}

fn resolve_carets_layout(snippet: &str, span: Span) -> (usize, usize) {
    let start = span.column.saturating_sub(1);
    let missing = start.saturating_sub(snippet.chars().count());
    let before: usize = snippet.chars().take(start).map(|c| if c == '\t' { 4 } else { 1 }).sum();
    let under: usize = snippet.chars().skip(start).take(span.length).map(|c| if c == '\t' { 4 } else { 1 }).sum();
    (before + missing, under.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::excerpt::Excerpt;
    use super::super::kind::Kind;
    use super::super::kind::IoError;
    use super::super::kind::SemanticError;
    use super::super::kind::SyntaxError;

    fn diag(kind: impl Into<Kind>) -> Diagnostic {
        Diagnostic::new(kind.into())
    }

    fn report(diagnostic: &Diagnostic) -> String {
        Report::new(diagnostic).to_string()
    }

    fn get_label(diagnostic: &Diagnostic) -> String {
        diagnostic.kind().label().expect("label should be present").to_string()
    }

    fn get_line(diagnostic: &Diagnostic) -> Vec<String> {
        report(diagnostic).lines().map(String::from).collect()
    }

    #[test]
    fn display_with_single_caret() {
        let d = diag(SyntaxError::UnclosedSection)
            .with_path("path/to/file.spec")
            .with_excerpt(Excerpt::new("[x", Span::new_point(1, 3)));
        let d_expected = [
            format!("error[{}]: unclosed section", d.kind().code()),
            " --> path/to/file.spec".to_string(),
            "  |".to_string(),
            "1 | [x".to_string(),
            "  |   ^ expect a closing `]` on section header".to_string(),
            "  |".to_string(),
            "  = help: valid section header is `[key.subkey]`".to_string(),
        ].join("\n");
        assert_eq!(report(&d), d_expected);
    }

    #[test]
    fn display_with_multiple_carets() {
        let d = diag(SyntaxError::MalformedSectionPath)
            .with_path("path/to/file.spec")
            .with_excerpt(Excerpt::new("[123.abc]", Span::new(1, 2, 3)));
        let d_expected = [
            format!("error[{}]: malformed section path", d.kind().code()),
            " --> path/to/file.spec".to_string(),
            "  |".to_string(),
            "1 | [123.abc]".to_string(),
            "  |  ^^^ expect identifiers separated by `.`".to_string(),
            "  |".to_string(),
            "  = help: an identifier starts with a letter or `_`, then letters, digits, `_` or `-`".to_string(),
        ].join("\n");
        assert_eq!(report(&d), d_expected);
    }

    #[test]
    fn display_with_carried_data() {
        let d = diag(SemanticError::DuplicatePath { path: String::from("foo") })
            .with_path("path/to/file.spec")
            .with_excerpt(Excerpt::new("foo: true", Span::new(1, 1, 3)));
        let d_expected = [
            format!("error[{}]: duplicate path `foo`", d.kind().code()),
            " --> path/to/file.spec".to_string(),
            "  |".to_string(),
            "1 | foo: true".to_string(),
            "  | ^^^ already defined".to_string(),
            "  |".to_string(),
            "  = help: each path can be defined only once".to_string(),
        ].join("\n");
        assert_eq!(report(&d), d_expected);
    }

    #[test]
    fn display_with_short_format() {
        let d = diag(IoError::FileNotFound).with_path("path/to/unknown.spec");
        let d_expected = [
            format!("error[{}]: file not found", d.kind().code()),
            " --> path/to/unknown.spec".to_string(),
            "  = help: make sure the file exists".to_string(),
        ].join("\n");
        assert_eq!(report(&d), d_expected);
    }

    #[test]
    fn missing_path_prints_the_default_to_input() {
        assert_eq!(
            get_line(&diag(SyntaxError::UnclosedSection).with_excerpt(Excerpt::new("[x", Span::new_point(1, 3))))[1],
            " --> <input>"
        );
    }

    #[test]
    fn tab_is_replaced_in_the_line_and_in_the_caret_offset() {
        let d = diag(SyntaxError::MissingColon).with_excerpt(Excerpt::new(
            "\tkey 1", Span::new_point(1, 5)
        ));
        let d_lines = get_line(&d);
        assert_eq!(d_lines[3],         "1 |     key 1");
        assert_eq!(d_lines[4], format!("  |        ^ {}", get_label(&d)));
    }

    #[test]
    fn tab_inside_the_span_widens_the_carets() {
        let d = diag(SyntaxError::UnclosedSection).with_excerpt(Excerpt::new(
            "\tx", Span::new(1, 1, 2)
        ));
        let d_lines = get_line(&d);
        assert_eq!(
            d_lines[4],
            format!("  | ^^^^^ {}", get_label(&d))
        );
    }

    #[test]
    fn gutter_widens_from_line_9_to_line_10() {
        let nine = get_line(&diag(SyntaxError::UnclosedSection).with_excerpt(
            Excerpt::new("a", Span::new_point(9, 2))
        ));
        assert_eq!(nine[1],         " --> <input>");
        assert_eq!(nine[2],         "  |");
        assert!(nine[3].starts_with("9 | a"));

        let ten = get_line(&diag(SyntaxError::UnclosedSection).with_excerpt(
            Excerpt::new("a", Span::new_point(10, 2))
        ));
        assert_eq!(ten[1],         "  --> <input>");
        assert_eq!(ten[2],         "   |");
        assert!(ten[3].starts_with("10 | a"));
        assert!(ten[4].starts_with("   |  ^"));
        assert_eq!(ten[5],         "   |");
        assert!(ten[6].starts_with("   = help: "));
    }

    #[test]
    fn zero_length_span_draws_one_caret() {
        let l = get_line(&diag(SyntaxError::UnclosedSection).with_excerpt(
            Excerpt::new("abc", Span::new_point(1, 1))
        ));
        assert!(l[4].starts_with("  | ^ "), "{}", l[4]);
    }

    #[test]
    fn caret_stops_at_the_end_of_the_line() {
        let d = diag(SyntaxError::UnclosedSection).with_excerpt(
            Excerpt::new("[ab", Span::new(1, 2, 10))
        );
        assert_eq!(get_line(&d)[4], format!("  |  ^^ {}", get_label(&d)));
    }

    #[test]
    fn span_past_the_line_end_keeps_its_offset() {
        let l = get_line(&diag(SyntaxError::UnclosedSection).with_excerpt(
            Excerpt::new("ab", Span::new_point(1, 6))
        ));
        assert!(l[4].starts_with("  |      ^"), "{}", l[4]);
    }

    #[test]
    fn has_no_trailing_newline() {
        assert!(!report(&diag(SyntaxError::UnclosedSection).with_excerpt(
            Excerpt::new("[x", Span::new_point(1, 3))
        )).ends_with('\n'));
    }
}
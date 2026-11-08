use super::error::Diagnostic;
use super::error::Excerpt;
use super::error::Kind;
use super::span::Span;

pub struct Source<'a> {
    source: &'a str,
    source_line_offsets: Vec<usize>,
}

impl<'a> Source<'a> {
    pub fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let mut source_line_offsets = vec![0];
        let mut source_line_offset = 0;
        while source_line_offset < bytes.len() {
            match bytes[source_line_offset] {
                b'\r' => {
                    if bytes.get(source_line_offset + 1) == Some(&b'\n') {
                        source_line_offset += 1;
                    }
                    source_line_offsets.push(source_line_offset + 1);
                },
                b'\n' => {
                    source_line_offsets.push(source_line_offset + 1);
                },
                _ => {},
            }
            source_line_offset += 1;
        }
        Self { source, source_line_offsets }
    }

    pub fn line_offsets_count(&self) -> usize {
        self.source_line_offsets.len()
    }

    pub fn line_offset(&self, number: usize) -> Option<&'a str> {
        let index = number.checked_sub(1)?;
        let offset_start = *self.source_line_offsets.get(index)?;
        let offset_end = self.source_line_offsets
            .get(index + 1)
            .copied()
            .unwrap_or(self.source.len());
        Some(self.source[offset_start..offset_end].trim_end_matches(['\r', '\n']))
    }

    pub fn diagnostic(&self, kind: impl Into<Kind>, span: Span) -> Diagnostic {
        let diagnostic = Diagnostic::new(kind);
        match self.excerpt(span) {
            Some(excerpt) => diagnostic.with_excerpt(excerpt),
            None => diagnostic,
        }
    }

    pub fn excerpt(&self, span: Span) -> Option<Excerpt> {
        self.line_offset(span.line).map(|line| Excerpt::new(line, span))
    }
}

#[cfg(test)]
mod tests { 
    use super::*;

    #[test]
    fn splits_on_lf() {
        let source = Source::new("a\nb");
        assert_eq!(source.line_offset(1), Some("a"));
        assert_eq!(source.line_offset(2), Some("b"));
    }

    #[test]
    fn splits_on_crlf_and_lone_cr() {
        let source = Source::new("a\r\nb\rc\nd");
        assert_eq!(source.line_offsets_count(), 4);
        assert_eq!(source.line_offset(1), Some("a"));
        assert_eq!(source.line_offset(2), Some("b"));
        assert_eq!(source.line_offset(3), Some("c"));
        assert_eq!(source.line_offset(4), Some("d"));
    }

    #[test]
    fn line_numbers_outside_the_text_give_none() {
        let source = Source::new("a\nb");
        assert_eq!(source.line_offset(0), None);
        assert_eq!(source.line_offset(3), None);
    }

    #[test]
    fn trailing_newline_adds_an_empty_last_line() {
        let source = Source::new("a\n");
        assert_eq!(source.line_offsets_count(), 2);
        assert_eq!(source.line_offset(2), Some(""));
    }

    #[test]
    fn empty_text_has_one_empty_line() {
        let source = Source::new("");
        assert_eq!(source.line_offsets_count(), 1);
        assert_eq!(source.line_offset(1), Some(""));
    }

    #[test]
    fn blank_lines_are_kept() {
        let source = Source::new("a\n\nb");
        assert_eq!(source.line_offset(2), Some(""));
        assert_eq!(source.line_offset(3), Some("b"));
    }

    #[test]
    fn keeps_tabs_and_non_ascii_text() {
        let source = Source::new("\tname: café");
        assert_eq!(source.line_offset(1), Some("\tname: café"));
    }

    #[test]
    fn excerpt_uses_the_line_of_the_span() {
        let source = Source::new("ok: 1\n[x");
        let span = Span::new_point(2, 3);
        let excerpt = source.excerpt(span).expect("line 2 exists");
        assert_eq!(excerpt.span(), span);
        assert_eq!(excerpt.snippet(), "[x");
    }

    #[test]
    fn excerpt_of_a_missing_line_is_none() {
        let source = Source::new("a");
        assert!(source.excerpt(Span::new_point(5, 1)).is_none());
    }

    #[test]
    fn diagnostic_carries_the_kind_and_the_excerpt() {
        use super::super::error::SyntaxError;

        let source = Source::new("ok: 1\n[x");
        let span = Span::new_point(2, 3);
        let diagnostic = source.diagnostic(SyntaxError::UnclosedSection, span);
        assert_eq!(diagnostic.kind(), &Kind::Syntax(SyntaxError::UnclosedSection));

        let excerpt = diagnostic.excerpt().expect("has an excerpt");
        assert_eq!(excerpt.span(), span);
        assert_eq!(excerpt.snippet(), "[x");
    }

    #[test]
    fn diagnostic_without_a_matching_line_has_no_excerpt() {
        use super::super::error::SyntaxError;

        let source = Source::new("a");
        let diagnostic = source.diagnostic(SyntaxError::MissingValue, Span::new_point(5, 1));
        assert!(diagnostic.excerpt().is_none());
    }
}
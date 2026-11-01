use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Excerpt {
    snippet: String,
    span: Span,
}

impl Excerpt {
    pub fn new(snippet: impl Into<String>, span: Span) -> Self {
        Self { snippet: snippet.into(), span }
    }

    pub fn snippet(&self) -> &str { &self.snippet }
    pub fn span(&self) -> Span { self.span }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_both_str_and_string() {
        let from_str = Excerpt::new("foo", Span::new(1, 1, 1));
        let from_string = Excerpt::new("foo".to_string(), Span::new(1, 1, 1));
        assert_eq!(from_str, from_string);
    }

    #[test]
    fn keeps_span_and_snippet() {
        let excerpt = Excerpt::new("foo", Span::new(1, 1, 1));
        assert_eq!(excerpt.span(), Span::new(1, 1, 1));
        assert_eq!(excerpt.snippet(), "foo");
    }

    #[test]
    fn equal_when_span_and_snippet_match() {
        let a = Excerpt::new("foo", Span::new(1, 1, 1));
        let b = Excerpt::new("foo", Span::new(1, 1, 1));
        assert_eq!(a, b);
    }

    #[test]
    fn different_when_snippet_not_match() {
        let a = Excerpt::new("foo", Span::new(1, 1, 1));
        let b = Excerpt::new("bar", Span::new(1, 1, 1));
        assert_ne!(a, b);
    }
}
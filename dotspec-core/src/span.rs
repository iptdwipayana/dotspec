#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        Self { line, column, length }
    }

    pub fn new_point(line: usize, column: usize) -> Self {
        Self { line, column, length: 0 }
    }

    pub fn is_zero_length(&self) -> bool {
        self.length == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keeps_all_fields() {
        let span = Span::new(3, 5, 4);
        assert_eq!((span.line, span.column, span.length), (3, 5, 4));
    }

    #[test]
    fn new_point_keeps_line_and_column() {
        assert!(Span::new_point(1, 3).is_zero_length());
    }

    #[test]
    fn non_empty_span_is_not_zero_length() {
        assert!(!Span::new(1, 1, 2).is_zero_length());
    }
}
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
    fn new_keeps_line_column_and_length() {
        let Span { 
            line, 
            column, 
            length 
        } = Span::new(3, 4, 5);
        assert_eq!((line, column, length), (3, 4, 5));
    }

    #[test]
    fn new_point_keeps_line_and_column() {
        let Span { 
            line, 
            column, 
            length 
        } = Span::new_point(3, 4);
        assert_eq!((line, column, length), (3, 4, 0));
    }

    #[test]
    fn empty_span_is_zero_length() {
        assert_eq!(Span::new_point(3, 4).is_zero_length(), true);
    }

    #[test]
    fn non_empty_span_is_not_zero_length() {
        assert_eq!(Span::new(3, 4, 5).is_zero_length(), false);
    }
}
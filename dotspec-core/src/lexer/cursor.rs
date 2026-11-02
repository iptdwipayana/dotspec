pub struct Cursor<'a> {
    offset: usize,
    line: usize,
    column: usize,
    text: &'a str,
}

impl<'a> Cursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { offset: 0, line: 1, column: 1, text }
    }

    pub fn offset(&self) -> usize { self.offset }
    pub fn line(&self) -> usize { self.line }
    pub fn column(&self) -> usize { self.column }

    pub fn peek(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
    }

    pub fn bump(&mut self) -> Option<char> {
        let c = self.peek()?; self.offset += c.len_utf8();
        match c {
            '\n' => self.next(),
            '\r' if self.peek() != Some('\n') => self.next(),
            _ => self.column += 1,
        }
        Some(c)
    }

    pub fn slice(&self, offset: usize) -> &'a str {
        &self.text[offset..self.offset]
    }

    pub fn take_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while let Some(c) = self.peek() {
            if !predicate(c) { 
                break;
            }
            self.bump();
        }
    }

    pub fn next(&mut self) {
        self.line += 1;
        self.column = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(cursor: &Cursor) -> (usize, usize) {
        (cursor.line(), cursor.column())
    }

    #[test]
    fn starts_at_line_1_column_1() {
        let cursor = Cursor::new("abc");
        assert_eq!(position(&cursor), (1, 1));
        assert_eq!(cursor.offset(), 0);
    }

    #[test]
    fn bump_moves_one_column() {
        let mut cursor = Cursor::new("ab");
        assert_eq!(cursor.bump(), Some('a'));
        assert_eq!(position(&cursor), (1, 2));
        assert_eq!(cursor.peek(), Some('b'));
    }

    #[test]
    fn bump_at_the_end_gives_none() {
        let mut cursor = Cursor::new("a");
        cursor.bump();
        assert_eq!(cursor.peek(), None);
        assert_eq!(cursor.bump(), None);
        assert_eq!(position(&cursor), (1, 2));
    }

    #[test]
    fn lf_starts_a_new_line() {
        let mut cursor = Cursor::new("a\nb");
        cursor.bump();
        cursor.bump();
        assert_eq!(position(&cursor), (2, 1));
    }

    #[test]
    fn crlf_is_one_line_break() {
        let mut cursor = Cursor::new("a\r\nb");
        cursor.bump(); // a
        cursor.bump(); // \r
        assert_eq!(position(&cursor), (1, 3));
        cursor.bump(); // \n
        assert_eq!(position(&cursor), (2, 1));
    }

    #[test]
    fn lone_cr_starts_a_new_line() {
        let mut cursor = Cursor::new("a\rb");
        cursor.bump();
        cursor.bump();
        assert_eq!(position(&cursor), (2, 1));
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        let mut cursor = Cursor::new("éa");
        cursor.bump();
        assert_eq!(position(&cursor), (1, 2));
        assert_eq!(cursor.offset(), 2);
    }

    #[test]
    fn tab_counts_as_one_column() {
        let mut cursor = Cursor::new("\ta");
        cursor.bump();
        assert_eq!(position(&cursor), (1, 2));
    }

    #[test]
    fn take_while_stops_at_the_first_miss() {
        let mut cursor = Cursor::new("aab");
        cursor.take_while(|c| c == 'a');
        assert_eq!(cursor.peek(), Some('b'));
        assert_eq!(cursor.slice(0), "aa");
    }
}
use super::span::Span;
use super::lexer::Fragment;

#[derive(Clone, Debug, PartialEq)]
pub struct Key<'a> {
    pub name: &'a str,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind<'a> {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    String(Vec<Fragment<'a>>),
    Array(Vec<Expr<'a>>),
    Object(Vec<Entry<'a>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry<'a> {
    pub key: Key<'a>,
    pub value: Expr<'a>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn integer(value: i64, column: usize) -> Expr<'static> {
        Expr { kind: ExprKind::Integer(value), span: Span::new(1, column, 1) }
    }

    fn entry(name: &'static str, value: i64) -> Entry<'static> {
        Entry {
            key: Key { name, span: Span::new(1, 1, 1) },
            value: integer(value, 4),
            span: Span::new(1, 1, 4),
        }
    }

    #[test]
    fn an_entry_keeps_its_key_value_and_spans() {
        let entry = Entry {
            key: Key { name: "debug", span: Span::new(1, 1, 5) },
            value: Expr { kind: ExprKind::Boolean(false), span: Span::new(1, 8, 5) },
            span: Span::new(1, 1, 12),
        };
        assert_eq!(entry.key.name, "debug");
        assert_eq!(entry.value.kind, ExprKind::Boolean(false));
        assert_eq!(entry.span, Span::new(1, 1, 12));
    }

    #[test]
    fn kinds_compare_by_variant_and_value() {
        assert_eq!(ExprKind::Integer(1), ExprKind::Integer(1));
        assert_ne!(ExprKind::Integer(1), ExprKind::Integer(2));
        assert_ne!(ExprKind::Integer(1), ExprKind::Float(1.0));
    }

    #[test]
    fn a_string_keeps_its_fragments() {
        let fragment = Fragment::Escape('\n', Span::new(1, 2, 2));
        let kind = ExprKind::String(vec![fragment.clone()]);
        assert_eq!(kind, ExprKind::String(vec![fragment]));
    }

    #[test]
    fn an_array_keeps_its_items_in_order() {
        let array = ExprKind::Array(vec![integer(1, 2), integer(2, 5)]);
        assert_eq!(array, ExprKind::Array(vec![integer(1, 2), integer(2, 5)]));
        assert_ne!(array, ExprKind::Array(vec![integer(2, 5), integer(1, 2)]));
    }

    #[test]
    fn an_object_keeps_duplicate_keys() {
        let object = ExprKind::Object(vec![entry("a", 1), entry("a", 2)]);
        let ExprKind::Object(entries) = object else {
            panic!("expected an object");
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key.name, entries[1].key.name);
    }

    #[test]
    fn collections_nest() {
        let inner = Expr { kind: ExprKind::Array(vec![integer(1, 3)]), span: Span::new(1, 2, 3) };
        let outer = ExprKind::Array(vec![inner]);
        assert!(matches!(outer, ExprKind::Array(ref items) if items.len() == 1));
    }
}
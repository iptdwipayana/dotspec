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
}
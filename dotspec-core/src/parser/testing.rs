use crate::ast::Expr;
use crate::ast::ExprKind;
use crate::lexer::Fragment;
use crate::error::Diagnostic;
use crate::error::Kind;
use crate::span::Span;

pub(super) fn show(expr: &Expr<'_>) -> String {
    match &expr.kind {
        ExprKind::Boolean(value) => value.to_string(),
        ExprKind::Integer(value) => value.to_string(),
        ExprKind::Float(value) => format!("{value:?}"),
        ExprKind::String(fragments) => {
            let mut text = String::new();
            for fragment in fragments {
                match fragment {
                    Fragment::Text(value, _) => text.push_str(value),
                    Fragment::Escape(value, _) => text.push(*value),
                    Fragment::Interpolation(..) => text.push_str("#{}"),
                }
            }
            format!("{text:?}")
        },
        ExprKind::Array(items) => {
            let items: Vec<_> = items.iter().map(show).collect();
            format!("[{}]", items.join(", "))
        },
        ExprKind::Object(entries) => {
            let entries: Vec<_> = entries
                .iter()
                .map(|entry| format!("{}: {}", entry.key.name, show(&entry.value)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        },
    }
}

pub(super) fn syntax_error(diagnostic: &Diagnostic) -> (crate::error::SyntaxError, Span) {
    let Kind::Syntax(error) = diagnostic.kind().clone() else {
        panic!("expected a syntax error, got: {diagnostic:?}");
    };
    let span = diagnostic
        .excerpt()
        .expect("a parser error has an excerpt")
        .span();
    (error, span)
}
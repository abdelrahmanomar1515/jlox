use crate::token::Token;

pub enum Expr<'a> {
    Binary {
        left: &'a Expr<'a>,
        operator: Token,
        right: &'a Expr<'a>,
    },
    Unary {
        operator: Token,
        right: &'a Expr<'a>,
    },
    Grouping {
        expr: &'a Expr<'a>,
    },
    Literal {
        value: Token,
    },
}

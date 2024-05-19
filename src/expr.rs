use crate::token::Token;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    // TODO: Split to different literal types
    Literal {
        value: Token,
    },
    Variable {
        name: Token,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    LogicOr {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    LogicAnd {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}

pub trait Visitor {
    type Out;
    fn visit_literal(&mut self, value: &Token) -> Self::Out;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Self::Out;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, args: &[Expr]) -> Self::Out;
    fn visit_grouping(&mut self, expr: &Expr) -> Self::Out;
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Self::Out;
    fn visit_variable(&mut self, name: &Token) -> Self::Out;
    fn visit_assignment(&mut self, name: &Token, value: &Expr) -> Self::Out;
    fn visit_logic_or(&mut self, left: &Expr, right: &Expr) -> Self::Out;
    fn visit_logic_and(&mut self, left: &Expr, right: &Expr) -> Self::Out;
}

impl Expr {
    pub fn accept<V>(&self, visitor: &mut V) -> V::Out
    where
        V: Visitor,
    {
        match self {
            Expr::Literal { ref value } => visitor.visit_literal(value),
            Expr::Unary {
                ref right,
                ref operator,
            } => visitor.visit_unary(operator, right),
            Expr::Binary {
                ref left,
                ref right,
                ref operator,
            } => visitor.visit_binary(left, operator, right),
            Expr::Call {
                ref callee,
                ref paren,
                ref args,
            } => visitor.visit_call(callee, paren, args),
            Expr::Grouping { ref expr } => visitor.visit_grouping(expr),
            Expr::Variable { ref name } => visitor.visit_variable(name),
            Expr::Assignment {
                ref name,
                ref value,
            } => visitor.visit_assignment(name, value),
            Expr::LogicOr {
                ref left,
                ref right,
                ..
            } => visitor.visit_logic_or(left, right),
            Expr::LogicAnd {
                ref left,
                ref right,
                ..
            } => visitor.visit_logic_and(left, right),
        }
    }
}

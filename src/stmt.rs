use crate::{expr::Expr, token::Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    Expression {
        expr: Box<Expr>,
    },
    Print {
        expr: Box<Expr>,
    },
    VariableDeclaration {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
}

pub trait Visitor {
    type Out;
    fn visit_expression(&mut self, expr: &Expr) -> Self::Out;
    fn visit_print(&mut self, expr: &Expr) -> Self::Out;
    fn visit_variable_declaration(&mut self, name: &Token, initializer: Option<&Expr>)
        -> Self::Out;
}

impl Stmt {
    pub fn accept<V>(&self, visitor: &mut V) -> V::Out
    where
        V: Visitor,
    {
        match self {
            Stmt::Expression { ref expr } => visitor.visit_expression(expr),
            Stmt::Print { ref expr } => visitor.visit_print(expr),
            Stmt::VariableDeclaration {
                ref name,
                ref initializer,
            } => visitor.visit_variable_declaration(name, initializer.as_deref()),
        }
    }
}

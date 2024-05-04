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
    Block {
        stmts: Vec<Stmt>,
    },
}

pub trait Visitor {
    type Out;
    fn visit_expression(&mut self, expr: &Expr) -> Self::Out;
    fn visit_print(&mut self, expr: &Expr) -> Self::Out;
    fn visit_variable_declaration(&mut self, name: &Token, initializer: Option<&Expr>)
        -> Self::Out;
    fn visit_block(&mut self, stmts: &[Stmt]) -> Self::Out;
}

impl Stmt {
    pub fn accept<V>(&self, visitor: &mut V) -> V::Out
    where
        V: Visitor,
    {
        match self {
            Stmt::Expression { expr } => visitor.visit_expression(expr),
            Stmt::Print { expr } => visitor.visit_print(expr),
            Stmt::VariableDeclaration { name, initializer } => {
                visitor.visit_variable_declaration(name, initializer.as_deref())
            }
            Stmt::Block { stmts } => visitor.visit_block(stmts.as_slice()),
        }
    }
}

use crate::expr::Expr;

#[derive(Clone, Debug)]
pub enum Stmt {
    Expression { expr: Box<Expr> },
    Print { expr: Box<Expr> },
}

pub trait Visitor {
    type Out;
    fn visit_expression(&mut self, expr: &Expr) -> Self::Out;
    fn visit_print(&mut self, expr: &Expr) -> Self::Out;
}

impl Stmt {
    pub fn accept<V>(&self, visitor: &mut V) -> V::Out
    where
        V: Visitor,
    {
        match self {
            Stmt::Expression { ref expr } => visitor.visit_expression(expr),
            Stmt::Print { ref expr } => visitor.visit_print(expr),
        }
    }
}

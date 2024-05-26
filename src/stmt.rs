use crate::{expr::Expr, token::Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Expression {
        expr: Box<Expr>,
    },
    FunctionDeclaration(FunctionDeclaration),
    Print {
        expr: Box<Expr>,
    },
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    },
    VariableDeclaration {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

pub trait Visitor {
    type Out;
    fn visit_expression(&mut self, expr: &Expr) -> Self::Out;
    fn visit_print(&mut self, expr: &Expr) -> Self::Out;
    fn visit_function_declaration(
        &mut self,
        function_declaration: &FunctionDeclaration,
    ) -> Self::Out;
    fn visit_return(&mut self, keyword: &Token, value: Option<&Expr>) -> Self::Out;
    fn visit_variable_declaration(&mut self, name: &Token, initializer: Option<&Expr>)
        -> Self::Out;
    fn visit_block(&mut self, stmts: &[Stmt]) -> Self::Out;
    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Self::Out;
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> Self::Out;
}

impl Stmt {
    pub fn accept<V>(&self, visitor: &mut V) -> V::Out
    where
        V: Visitor,
    {
        match self {
            Stmt::Expression { expr } => visitor.visit_expression(expr),
            Stmt::Print { expr } => visitor.visit_print(expr),
            Stmt::FunctionDeclaration(function) => visitor.visit_function_declaration(function),
            Stmt::While { condition, body } => visitor.visit_while(condition, body),
            Stmt::Return { keyword, value } => visitor.visit_return(keyword, value.as_deref()),
            Stmt::VariableDeclaration { name, initializer } => {
                visitor.visit_variable_declaration(name, initializer.as_deref())
            }
            Stmt::Block { stmts } => visitor.visit_block(stmts.as_slice()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if(condition, then_branch, else_branch.as_deref()),
        }
    }
}

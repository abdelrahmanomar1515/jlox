use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use crate::{Error, Result};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        while !self.at_end() {
            statements.push(self.parse_declaration_statement()?);
        }

        Ok(statements)
    }

    fn parse_declaration_statement(&mut self) -> Result<Stmt> {
        if let TokenType::Var = self.peek().token_type {
            self.advance();
            return self.parse_variable_declaration();
        }
        self.parse_statement()
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if let TokenType::Print = self.peek().token_type {
            self.advance();
            return self.parse_print_statement();
        }
        if let TokenType::LeftBrace = self.peek().token_type {
            self.advance();
            return self.parse_block();
        }
        self.parse_expression_statement()
    }

    fn parse_block(&mut self) -> Result<Stmt> {
        let mut stmts = vec![];
        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.at_end() {
            stmts.push(self.parse_declaration_statement()?);
        }
        match self.peek().token_type {
            TokenType::RightBrace => {
                self.advance();
            }
            _ => {
                return Err(self.error("Expected } after block"));
            }
        }
        Ok(Stmt::Block { stmts })
    }

    fn parse_print_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
            }
            _ => {
                return Err(self.error("Expected ; after print statement"));
            }
        }
        Ok(Stmt::Print {
            expr: Box::new(expr),
        })
    }

    fn parse_variable_declaration(&mut self) -> Result<Stmt> {
        let name = match self.peek().token_type {
            TokenType::Identifier => self.advance(),
            _ => {
                return Err(self.error("Expected variable name"));
            }
        };
        let initializer = match self.peek().token_type {
            TokenType::Equal => {
                self.advance();
                let initializer = Some(Box::new(self.parse_expression()?));
                if let TokenType::Semicolon = self.peek().token_type {
                    self.advance();
                } else {
                    return Err(self.error("Expected ';' after expression"));
                }
                initializer
            }

            TokenType::Semicolon => {
                self.advance();
                None
            }
            _ => {
                return Err(self.error("Expected ; after variable declaration"));
            }
        };
        Ok(Stmt::VariableDeclaration { name, initializer })
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
            }
            _ => {
                return Err(self.error("Expected ; after expression statement"));
            }
        }
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_equality()?;

        if let TokenType::Equal = self.peek().token_type {
            let _equal = self.advance();
            let value = self.parse_assignment()?;
            if let Expr::Variable { ref name } = &expr {
                return Ok(Expr::Assignment {
                    name: name.clone(),
                    value: Box::new(value),
                });
            }
            return Err(self.error("Invalid assignment target"));
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;

        while let TokenType::BangEqual | TokenType::EqualEqual = self.peek().token_type {
            let operator = self.advance();
            let right = self.parse_comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;

        while let TokenType::Greater
        | TokenType::GreaterEqual
        | TokenType::Less
        | TokenType::LessEqual = self.peek().token_type
        {
            self.advance();
            let operator = self.previous();
            let right = self.parse_term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;

        while let TokenType::Minus | TokenType::Plus = self.peek().token_type {
            self.advance();
            let operator = self.previous();
            let right = self.parse_factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        while let operator @ Token {
            token_type: TokenType::Slash | TokenType::Star,
            ..
        } = self.peek()
        {
            self.advance();
            let right = self.parse_unary()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if let TokenType::Bang | TokenType::Minus = self.peek().token_type {
            self.advance();
            let operator = self.previous();
            let right = self.parse_unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek().token_type {
            TokenType::True | TokenType::False | TokenType::Number(..) | TokenType::String(..) => {
                let value = self.advance();
                Ok(Expr::Literal { value })
            }
            TokenType::Identifier => {
                let name = self.advance();
                Ok(Expr::Variable { name })
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                if let TokenType::RightParen = self.peek().token_type {
                    self.advance();
                } else {
                    return Err(self.error("Expected ')' after expression"));
                }
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                })
            }
            token_type => Err(self.error(&format!("Expected expression found: {:?}", token_type))),
        }
    }

    fn advance(&mut self) -> Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn error(&self, msg: &str) -> Error {
        Error::ParseError {
            line: self.previous().line,
            msg: msg.into(),
        }
    }
}

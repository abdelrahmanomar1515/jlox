use crate::stmt::Stmt;
use crate::token::TokenType;
use crate::{expr::Expr, token::Token};
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
            let statement = self.parse_statement();
            statements.push(statement?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if let TokenType::Print = self.peek().token_type {
            self.advance();
            return self.parse_print_statement();
        }
        println!("parsing expression statement");
        self.parse_expression_statement()
    }

    fn parse_print_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
            }
            _ => {
                return Err(Error::ParseError(
                    "Expected ; after print statement".to_string(),
                ))
            }
        }
        Ok(Stmt::Print {
            expr: Box::new(expr),
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
            }
            _ => {
                return Err(Error::ParseError(
                    "Expected ; after expression statement".to_string(),
                ))
            }
        }
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;

        while let TokenType::BangEqual | TokenType::EqualEqual = self.peek().token_type {
            self.advance();
            let operator = self.previous();
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
                self.advance();
                let token = self.previous();
                Ok(Expr::Literal { value: token })
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                if let TokenType::RightParen = self.peek().token_type {
                    self.advance();
                } else {
                    return Err("Expected ')' after expression.".into());
                }
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                })
            }
            token => Err(Error::ParseError(format!(
                "Expected expression found: {:?} ",
                token
            ))),
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
}

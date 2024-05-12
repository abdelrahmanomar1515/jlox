use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use crate::{Error, Result};

macro_rules! match_next {
    ($self: ident, $p:pat) => {
        if let $p = $self.peek().token_type {
            $self.advance();
            true
        } else {
            false
        }
    };
}

macro_rules! consume_next {
    ($self: ident, $p:pat, $e: expr) => {
        match $self.peek().token_type {
            $p => $self.advance(),
            _ => {
                return Err($self.error($e));
            }
        }
    };
}

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
        if match_next!(self, TokenType::Var) {
            return self.parse_variable_declaration();
        }
        self.parse_statement()
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if match_next!(self, TokenType::For) {
            return self.parse_for_statement();
        }
        if match_next!(self, TokenType::If) {
            return self.parse_if_statement();
        }
        if match_next!(self, TokenType::While) {
            return self.parse_while_loop();
        }
        if match_next!(self, TokenType::Print) {
            return self.parse_print_statement();
        }
        if match_next!(self, TokenType::LeftBrace) {
            return self.parse_block();
        }
        self.parse_expression_statement()
    }

    fn parse_block(&mut self) -> Result<Stmt> {
        let mut stmts = vec![];
        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.at_end() {
            stmts.push(self.parse_declaration_statement()?);
        }
        consume_next!(self, TokenType::RightBrace, "Expect } after block");
        Ok(Stmt::Block { stmts })
    }

    fn parse_for_statement(&mut self) -> Result<Stmt> {
        consume_next!(self, TokenType::LeftParen, "Expect '(' after 'for'.");
        let initializer = match self.peek().token_type {
            TokenType::Var => {
                self.advance();
                Some(self.parse_variable_declaration()?)
            }
            TokenType::Semicolon => {
                self.advance();
                None
            }
            _ => Some(self.parse_expression_statement()?),
        };

        let condition = match self.peek().token_type {
            TokenType::Semicolon => Expr::Literal {
                value: Token::new(TokenType::True, "true".to_string(), self.peek().line),
            },
            _ => self.parse_expression()?,
        };
        consume_next!(
            self,
            TokenType::Semicolon,
            "Expect ';' after loop condition."
        );

        let increment = match self.peek().token_type {
            TokenType::RightParen => None,
            _ => Some(self.parse_expression()?),
        };
        consume_next!(self, TokenType::RightParen, "Expect ')' after for clauses.");
        let mut body = self.parse_statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                stmts: vec![
                    body,
                    Stmt::Expression {
                        expr: Box::new(increment),
                    },
                ],
            }
        }

        body = Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                stmts: vec![initializer, body],
            }
        }

        Ok(body)
    }

    fn parse_if_statement(&mut self) -> Result<Stmt> {
        consume_next!(self, TokenType::LeftParen, "Expect '(' after 'if'.");
        let condition = self.parse_expression()?;
        consume_next!(
            self,
            TokenType::RightParen,
            "Expect ')' after if condition."
        );

        let then_branch = self.parse_statement()?;
        let mut else_branch = None;

        if match_next!(self, TokenType::Else) {
            else_branch = Some(self.parse_statement()?);
        }
        Ok(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn parse_while_loop(&mut self) -> Result<Stmt> {
        consume_next!(self, TokenType::LeftParen, "Expect '(' after 'if'.");
        let condition = self.parse_expression()?;
        consume_next!(
            self,
            TokenType::RightParen,
            "Expect ')' after if condition."
        );
        let body = self.parse_statement()?;
        Ok(Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_print_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        consume_next!(self, TokenType::Semicolon, "Expect ; after print statement");
        Ok(Stmt::Print {
            expr: Box::new(expr),
        })
    }

    fn parse_variable_declaration(&mut self) -> Result<Stmt> {
        let name = consume_next!(self, TokenType::Identifier, "Expect variable name");
        let initializer = match self.peek().token_type {
            TokenType::Equal => {
                self.advance();
                let initializer = Some(Box::new(self.parse_expression()?));
                consume_next!(self, TokenType::Semicolon, "Expect ';' after expression");
                initializer
            }

            TokenType::Semicolon => {
                self.advance();
                None
            }
            _ => {
                return Err(self.error("Expect ; after variable declaration"));
            }
        };
        Ok(Stmt::VariableDeclaration { name, initializer })
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        consume_next!(
            self,
            TokenType::Semicolon,
            "Expect ; after expression statement"
        );
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_logic_or()?;

        if match_next!(self, TokenType::Equal) {
            let _equal = self.previous();
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

    fn parse_logic_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logic_and()?;
        while match_next!(self, TokenType::Or) {
            let operator = self.previous();
            let right = self.parse_logic_and()?;
            expr = Expr::LogicOr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_logic_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        while match_next!(self, TokenType::And) {
            let operator = self.previous();
            let right = self.parse_equality()?;
            expr = Expr::LogicAnd {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;

        while match_next!(self, TokenType::BangEqual | TokenType::EqualEqual) {
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

        while match_next!(
            self,
            TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual
        ) {
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

        while match_next!(self, TokenType::Minus | TokenType::Plus) {
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

        while match_next!(self, TokenType::Slash | TokenType::Star) {
            let operator = self.previous();
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
        if match_next!(self, TokenType::Bang | TokenType::Minus) {
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
            TokenType::True
            | TokenType::False
            | TokenType::Number(..)
            | TokenType::String(..)
            | TokenType::Nil => {
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
                consume_next!(self, TokenType::RightParen, "Expect ')' after expression");
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                })
            }
            token_type => Err(self.error(&format!("Expect expression found: {:?}", token_type))),
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

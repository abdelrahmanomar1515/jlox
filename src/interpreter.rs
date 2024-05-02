use crate::Result;
use crate::{
    expr::{self, Expr},
    stmt::{self, Stmt},
    token::{Token, TokenType},
    Error,
};
use derive_more::Display;
use std::collections::HashMap;

#[derive(Default)]
pub struct Interpreter {
    env: HashMap<String, Value>,
}

impl Interpreter {
    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            stmt.accept(self)?
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr.accept(self)
    }

    pub fn is_truthy(&self, value: &Value) -> bool {
        match *value {
            Value::String(_) => true,
            Value::Number(_) => true,
            Value::Boolean(v) => v,
            Value::Nil => false,
        }
    }
}

impl expr::Visitor for Interpreter {
    type Out = Result<Value>;

    fn visit_literal(&mut self, value: &Token) -> Self::Out {
        match &value.token_type {
            TokenType::Nil => Ok(Value::Nil),
            TokenType::False => Ok(Value::Boolean(false)),
            TokenType::True => Ok(Value::Boolean(true)),
            TokenType::Number(n) => Ok(Value::Number(*n)),
            TokenType::String(s) => Ok(Value::String(s.clone())),
            _ => Err(Error::runtime(value, "Unknown literal type")),
        }
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Self::Out {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match right {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Err(Error::runtime(operator, "Operand must be a number")),
            },
            TokenType::Bang => Ok(Value::Boolean(self.is_truthy(&right))),
            _ => Err(Error::runtime(operator, "Unknown unary operator")),
        }
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Self::Out {
        self.evaluate(expr)
    }

    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Self::Out {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),
                _ => Err(Error::runtime(
                    operator,
                    "Operands must be numbers or strings",
                )),
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::Greater => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::Less => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::LessEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
                _ => Err(Error::runtime(operator, "Operands must be numbers")),
            },
            TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
            _ => Err(Error::runtime(operator, "Unknown binary operator")),
        }
    }

    fn visit_variable(&mut self, name: &Token) -> Self::Out {
        let value = self.env.get(&name.text).ok_or(Error::RuntimeError {
            line: name.line,
            msg: format!("Undefined variable: {}", name.text),
        })?;
        Ok(value.clone())
    }

    fn visit_assignment(&mut self, name: &Token, value: &Expr) -> Self::Out {
        let value = self.evaluate(value)?;
        if self.env.contains_key(&name.text) {
            self.env.insert(name.text.clone(), value.clone());
            return Ok(value);
        }
        Err(Error::RuntimeError {
            line: name.line,
            msg: format!("Undefined variable {}", name.text),
        })
    }
}

impl stmt::Visitor for Interpreter {
    type Out = Result<()>;

    fn visit_expression(&mut self, expr: &Expr) -> Self::Out {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_print(&mut self, expr: &Expr) -> Self::Out {
        let value = self.evaluate(expr)?;
        println!("{value}");
        Ok(())
    }

    fn visit_variable_declaration(
        &mut self,
        name: &Token,
        initializer: Option<&Expr>,
    ) -> Self::Out {
        let value = match initializer {
            Some(initializer) => self.evaluate(initializer)?,
            None => Value::Nil,
        };
        self.env.insert(name.text.clone(), value);
        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            Value::Number(num) => write!(f, "{}", num),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "null"),
        }
    }
}

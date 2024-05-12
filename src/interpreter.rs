use crate::Result;
use crate::{
    expr::{self, Expr},
    stmt::{self, Stmt},
    token::{Token, TokenType},
    Error,
};
use derive_more::Display;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Default::default(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            self.execute(&stmt)?
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn evaluate_block(&mut self, stmts: &[Stmt], env: Environment) -> Result<()> {
        let previous = Rc::clone(&self.env);
        self.env = Rc::new(RefCell::new(env));
        let result = stmts.iter().try_for_each(|stmt| self.execute(stmt));
        self.env = previous;
        result
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr.accept(self)
    }

    fn is_truthy(&self, value: &Value) -> bool {
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
        self.env.borrow().get(name)
    }

    fn visit_assignment(&mut self, name: &Token, value: &Expr) -> Self::Out {
        let value = self.evaluate(value)?;
        self.env.borrow_mut().assign(name, &value)
    }

    fn visit_logic_or(&mut self, left: &Expr, right: &Expr) -> Self::Out {
        let value = self.evaluate(left)?;
        if self.is_truthy(&value) {
            Ok(value)
        } else {
            self.evaluate(right)
        }
    }

    fn visit_logic_and(&mut self, left: &Expr, right: &Expr) -> Self::Out {
        let value = self.evaluate(left)?;
        if self.is_truthy(&value) {
            self.evaluate(right)
        } else {
            Ok(value)
        }
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
        RefCell::borrow_mut(&self.env).define(name, value);
        Ok(())
    }

    fn visit_block(&mut self, stmts: &[Stmt]) -> Self::Out {
        let environment = Environment::new(Some(Rc::clone(&self.env)));
        self.evaluate_block(stmts, environment)?;
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

type Env = Rc<RefCell<Environment>>;

#[derive(Default)]
struct Environment {
    enclosing: Option<Env>,
    store: HashMap<String, Value>,
}

impl Environment {
    fn new(enclosing: Option<Env>) -> Self {
        Self {
            enclosing,
            store: HashMap::new(),
        }
    }

    fn define(&mut self, name: &Token, value: Value) {
        self.store.insert(name.text.clone(), value);
    }

    fn assign(&mut self, name: &Token, value: &Value) -> Result<Value> {
        if self.store.contains_key(&name.text) {
            self.store.insert(name.text.clone(), value.clone());
            return Ok(value.clone());
        }
        if let Some(ref enclosing) = self.enclosing {
            let mut enclosing = RefCell::borrow_mut(enclosing);
            return enclosing.assign(name, value);
        }

        Err(Error::runtime(
            name,
            format!("Undefined variable {}", name.text).as_str(),
        ))
    }

    fn get(&self, name: &Token) -> Result<Value> {
        let value = self.store.get(&name.text);
        if let Some(value) = value {
            return Ok(value.clone());
        } else if let Some(ref enclosing) = self.enclosing {
            return RefCell::borrow(enclosing).get(name);
        }
        Err(Error::runtime(
            name,
            format!("Undefined variable: {}", name.text).as_str(),
        ))
    }
}

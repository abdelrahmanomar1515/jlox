use crate::stmt::FunctionDeclaration;
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
use std::time::SystemTime;

trait Callable {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value], paren: &Token) -> Result<Value>;
    fn arity(&self) -> usize;
}

#[derive(PartialEq, Debug, Clone)]
pub struct NativeFunction {
    arity: usize,
    name: String,
    function: fn(&mut Interpreter, &[Value]) -> Value,
}

impl Callable for NativeFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value], _paren: &Token) -> Result<Value> {
        Ok((self.function)(interpreter, args))
    }

    fn arity(&self) -> usize {
        self.arity
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    declaration: FunctionDeclaration,
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value], paren: &Token) -> Result<Value> {
        if args.len() != self.arity() {
            return Err(Error::runtime(
                paren,
                &format!(
                    "Expected {} arguments but got {} arguments",
                    self.arity(),
                    args.len(),
                ),
            ));
        };

        let mut environment = Environment::new(Some(interpreter.env.clone()));
        self.declaration
            .params
            .iter()
            .enumerate()
            .for_each(|(i, param)| environment.define(param, args[i].clone()));

        interpreter.execute_block(&self.declaration.body, environment)?;

        Ok(Value::Nil)
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Function(Function),
    NativeFunction(NativeFunction),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            Value::Number(num) => write!(f, "{}", num),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Function(Function {
                declaration: FunctionDeclaration { name, .. },
                ..
            }) => write!(f, "<function {}>", name.text),
            Value::NativeFunction(NativeFunction { name, .. }) => {
                write!(f, "<native function {}>", name)
            }
            Value::Nil => write!(f, "null"),
        }
    }
}

pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::default();
        env.store.insert(
            "clock".to_string(),
            Value::NativeFunction(NativeFunction {
                arity: 0,
                name: "clock".to_string(),
                function: |_interpreter, _args| {
                    Value::Number(
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .expect("Clock may have gone backwards")
                            .as_millis() as f64
                            / 1000.0,
                    )
                },
            }),
        );
        Self {
            env: Rc::new(RefCell::new(env)),
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

    fn execute_block(&mut self, stmts: &[Stmt], env: Environment) -> Result<()> {
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
            Value::Function(_) => false,
            Value::NativeFunction(_) => false,
            Value::Nil => false,
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
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

    fn visit_call(&mut self, callee: &Expr, paren: &Token, args: &[Expr]) -> Self::Out {
        let callee = self.evaluate(callee)?;
        let args = args
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<Vec<_>>>()?;
        let callable: Box<dyn Callable> = match callee {
            Value::NativeFunction(f) => Box::new(f),
            Value::Function(f) => Box::new(f),
            _ => {
                return Err(Error::runtime(paren, "Can only call functions and classes"));
            }
        };
        if args.len() != callable.arity() {
            return Err(Error::runtime(
                paren,
                &format!(
                    "Expected {} arguments but got {} arguments",
                    callable.arity(),
                    args.len(),
                ),
            ));
        };
        callable.call(self, &args, paren)
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

    fn visit_function_declaration(
        &mut self,
        function_declaration: &FunctionDeclaration,
    ) -> Self::Out {
        let function = Function {
            declaration: function_declaration.clone(),
        };
        self.env.borrow_mut().define(
            &function_declaration.name,
            Value::Function(function).clone(),
        );

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
        self.execute_block(stmts, environment)?;
        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Self::Out {
        let condition = &self.evaluate(condition)?;
        if self.is_truthy(condition) {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> Self::Out {
        loop {
            let condition_result = &self.evaluate(condition)?;
            if !self.is_truthy(condition_result) {
                break;
            }
            self.execute(body)?;
        }
        Ok(())
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

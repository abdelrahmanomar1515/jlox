use derive_more::From;
use token::Token;

pub mod expr;
pub mod interpreter;
pub mod parser;
// pub mod print;
pub mod scanner;
pub mod stmt;
pub mod token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    ParseError {
        line: usize,
        msg: String,
    },

    FromUtf8Error(std::string::FromUtf8Error),
    IO(std::io::Error),

    RuntimeError {
        line: usize,
        msg: String,
    },
}

impl Error {
    pub fn runtime(token: &Token, message: &str) -> Error {
        Error::RuntimeError {
            line: token.line,
            msg: message.to_string(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn custom(value: impl std::fmt::Display) -> Self {
        Self::Custom(value.to_string())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8Error(value)
    }
}

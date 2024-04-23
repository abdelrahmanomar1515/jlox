use jlox::Result;
use std::{
    env::args,
    fs,
    io::{self, BufRead, Write},
};

fn main() -> jlox::Result<()> {
    let args: Vec<String> = args().collect();
    match args.len().cmp(&2) {
        std::cmp::Ordering::Greater => Err("Too many arguments")?,
        std::cmp::Ordering::Equal => run_file(&args[1]),
        std::cmp::Ordering::Less => run_prompt(),
    }
}

fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush()?;
    for line in stdin.lock().lines() {
        print!("> ");
        io::stdout().flush()?;
        run(&line?)?
    }
    Ok(())
}

fn run_file(path: &str) -> Result<()> {
    let source = fs::read(path)?;
    run(&String::from_utf8(source)?)
}

fn run(_source: &str) -> Result<()> {
    // eprintln!("{}", file_bytes);
    // let scanner = Scanner::new(source);
    // let tokens = scanner.scan_tokens();
    // tokens.for_each(|token| println!("{}", token));
    Ok(())
}

enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

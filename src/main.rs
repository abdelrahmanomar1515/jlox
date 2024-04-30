use jlox::print::Printer;
use jlox::{parser::Parser, scanner::Scanner, Result};
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
        run(line?)?
    }
    Ok(())
}

fn run_file(path: &str) -> Result<()> {
    let source = fs::read(path)?;
    run(String::from_utf8(source)?)
}

fn run(source: String) -> Result<()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let mut printer = Printer;
    println!("{}", printer.print(&parser.parse()?));
    Ok(())
}

use core::f64;
use std::collections::HashMap;

#[derive(Debug, Clone)]
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
    String(String),
    Number(f64),

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

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    text: String,
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, text: String, line: usize) -> Self {
        Self {
            token_type,
            text,
            line,
        }
    }
}

#[derive(Default)]
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let keywords = HashMap::from([
            ("and".to_string(), TokenType::And),
            ("class".to_string(), TokenType::Class),
            ("else".to_string(), TokenType::Else),
            ("false".to_string(), TokenType::False),
            ("for".to_string(), TokenType::For),
            ("fun".to_string(), TokenType::Fun),
            ("if".to_string(), TokenType::If),
            ("nil".to_string(), TokenType::Nil),
            ("or".to_string(), TokenType::Or),
            ("print".to_string(), TokenType::Print),
            ("return".to_string(), TokenType::Return),
            ("super".to_string(), TokenType::Super),
            ("this".to_string(), TokenType::This),
            ("true".to_string(), TokenType::True),
            ("var".to_string(), TokenType::Var),
            ("while".to_string(), TokenType::While),
        ]);

        Self {
            source: source.chars().collect(),
            keywords,
            ..Default::default()
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token()
        }
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            text: "".to_string(),
            line: self.line,
        });
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let next_is_equal = self.match_char('=');
                if next_is_equal {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '=' => {
                let next_is_equal = self.match_char('=');
                if next_is_equal {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '>' => {
                let next_is_equal = self.match_char('=');
                if next_is_equal {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }
            '<' => {
                let next_is_equal = self.match_char('=');
                if next_is_equal {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '"' => self.string(),
            c if c.is_ascii_digit() => self.number(),
            c if c.is_alphabetic() => self.identifier(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            _ => {
                // TODO: add error logging
                unimplemented!()
            }
        };
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        };
        self.source[self.current + 1]
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }
        if self.at_end() {
            // TODO: Better error handling
            eprintln!("Unterminated string");
            return;
        }

        // The closing "
        self.advance();

        let literal = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String(literal.iter().collect()));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num = &self.source[self.start..self.current];
        self.add_token(TokenType::Number(
            num.iter()
                .collect::<String>()
                .parse()
                .expect("Failed to parse string to float"),
        ));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token_type = self
            .keywords
            .get(&text.iter().collect::<String>())
            .unwrap_or(&TokenType::Identifier)
            .to_owned();
        self.add_token(token_type);
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];
        let token = Token::new(token_type, text.iter().collect(), self.line);
        self.tokens.push(token);
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

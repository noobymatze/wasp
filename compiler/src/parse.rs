use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};

pub type Line = usize;

pub type Col = usize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    BadChar(Line, Col, char),
    Number(Line, Col, String),
    BadEndOfInput(Line, Col),
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub line: Line,
    pub col: Col,
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Deserialize)]
pub struct Region {
    pub start: Position,
    pub end: Position,
}

impl Region {
    fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Region {
            start: Position {
                line: start_line,
                col: start_col,
            },
            end: Position {
                line: end_line,
                col: end_col,
            },
        }
    }
}

impl Serialize for Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.start.line)?;
        seq.serialize_element(&self.start.col)?;
        seq.serialize_element(&self.end.line)?;
        seq.serialize_element(&self.end.col)?;
        seq.end()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Token {
    LParen,
    RParen,
    Symbol(String),
    Number(f64),
    Eof,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Module {
    pub filename: Option<String>,
    pub expressions: Vec<Expr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Expr {
    Number { region: Region, value: f64 },
    Symbol { region: Region, value: String },
    List { region: Region, expressions: Vec<Expr> },
}

// PARSING

pub fn parse(filename: Option<String>, input: &str) -> Result<Module, Error> {
    let mut parser = Parser::new(lexer(input));
    parser.module(filename)
}

struct Parser<T: Iterator<Item = LexResult>> {
    input: T,
    token0: Option<(Region, Token)>,
    errors: Vec<Error>,
}

impl<T: Iterator<Item = LexResult>> Parser<T> {
    fn new(input: T) -> Self {
        Parser {
            token0: None,
            errors: vec![],
            input,
        }
    }

    fn module(&mut self, filename: Option<String>) -> Result<Module, Error> {
        let mut expressions = vec![];
        loop {
            match self.advance() {
                None => {
                    return Ok(Module {
                        filename,
                        expressions,
                    })
                }
                Some(token) => match self.expr(token) {
                    Err(error) => return Err(error),
                    Ok(expr) => expressions.push(expr),
                },
            }
        }
    }

    fn expr(&mut self, token: (Region, Token)) -> Result<Expr, Error> {
        match token {
            (region, Token::Number(value)) => Ok(Expr::Number { region, value }),
            (region, Token::Symbol(value)) => Ok(Expr::Symbol { region, value }),
            (region, Token::LParen) => {
                let mut expressions = vec![];
                loop {
                    match self.advance() {
                        None => return Err(Error::BadEndOfInput(0, 0)),
                        Some((end_region, Token::RParen)) => {
                            let region = Region::new(
                                region.start.line,
                                region.start.col,
                                end_region.end.line,
                                end_region.end.col,
                            );

                            return Ok(Expr::List { region, expressions });
                        }
                        Some(token) => expressions.push(self.expr(token)?),
                    }
                }
            }
            (_, Token::Eof) | (_, Token::RParen) => Err(Error::BadEndOfInput(0, 0)),
        }
    }

    fn advance(&mut self) -> Option<(Region, Token)> {
        match self.token0.take() {
            None => self.next(),
            Some(value) => Some(value),
        }
    }

    fn next(&mut self) -> Option<(Region, Token)> {
        loop {
            match self.input.next() {
                None => return None,
                Some(Ok(result)) => return Some(result),
                Some(Err(error)) => self.errors.push(error),
            }
        }
    }
}

// LEXING

type LexResult = Result<(Region, Token), Error>;

struct Lexer<T: Iterator<Item = char>> {
    input: T,
    pending: Vec<(Region, Token)>,
    char0: Option<char>,
    line: usize,
    start_line: usize,
    col: usize,
    start_col: usize,
}

pub fn lexer(input: &str) -> impl Iterator<Item = LexResult> + '_ {
    Lexer {
        start_line: 1,
        start_col: 1,
        line: 1,
        col: 1,
        pending: vec![],
        input: input.chars(),
        char0: None,
    }
}

impl<T: Iterator<Item = char>> Lexer<T> {
    fn consume(&mut self) -> LexResult {
        while self.pending.is_empty() {
            self.consume_next()?;
        }

        Ok(self.pending.remove(0))
    }

    /// Consume the next [`Token`] or [`Error`].
    fn consume_next(&mut self) -> Result<(), Error> {
        self.start_line = self.line;
        self.start_col = self.col;
        match self.advance() {
            None => self.emit(Token::Eof),
            Some(c) => self.consume_token(c)?,
        }

        Ok(())
    }

    fn consume_token(&mut self, c: char) -> Result<(), Error> {
        match c {
            '\n' => {
                self.line += 1;
                self.col = 1;
            }
            '(' => self.emit(Token::LParen),
            ')' => self.emit(Token::RParen),
            c if c.is_whitespace() => {
                // Don't need to do anything here.
            }
            c if c.is_ascii_digit() => self.consume_number(c)?,
            c if c.is_symbol_start() => self.consume_symbol(c)?,
            c => return Err(Error::BadChar(self.line, self.col - 1, c)),
        }

        Ok(())
    }

    fn consume_number(&mut self, start: char) -> Result<(), Error> {
        let mut result = String::from(start);
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            if let Some(c) = self.advance() {
                result.push(c);
            }
        }

        if self.peek() == Some('.') {
            self.advance();
            result.push('.');
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                if let Some(c) = self.advance() {
                    result.push(c);
                }
            }
        }

        let number = result
            .parse::<f64>()
            .map_err(|err| Error::Number(self.line, self.col - 1, format!("{}", err)))?;
        self.emit(Token::Number(number));
        Ok(())
    }

    /// Consume an identifier. An identifier in Zen starts with
    /// an alphabetic letter.
    fn consume_symbol(&mut self, start: char) -> Result<(), Error> {
        let mut result = String::from(start);
        while let Some(c) = self.peek() {
            if c.is_symbol() {
                self.advance();
                result.push(c);
            } else {
                break;
            }
        }

        match result.as_str() {
            // "true" => self.emit(Token::Boolean(true)),
            // "false" => self.emit(Token::Boolean(false)),
            _ => self.emit(Token::Symbol(result)),
        }

        Ok(())
    }

    /// Pushes a new token into the [pending] list of tokens.
    ///
    /// ## Example
    ///
    /// ```
    /// let mut lexer = Lexer::new("test".chars());
    /// assert!(lexer.pending.is_empty());
    /// lexer.emit(Token::Data);
    /// assert!(!lexer.pending.is_empty());
    /// ```
    fn emit(&mut self, token: Token) {
        let region = Region::new(
            self.start_line,
            self.start_col,
            self.line,
            self.col - 1, // -1 because the col is always a character further.
        );
        self.pending.push((region, token));
    }

    /// Returns the next character in the input, without advancing
    /// the input.
    ///
    /// This allows to look ahead to decide what to do, based on
    /// the character.
    ///
    /// # Example
    ///
    /// ```rust
    /// let lexer = Lexer::new("test".chars());
    /// assert_eq!(lexer.peek(), Some('t'));
    /// ```
    fn peek(&mut self) -> Option<char> {
        if self.char0.is_none() {
            self.char0 = self.input.next();
        }

        self.char0
    }

    /// Returns the next character in the input.
    ///
    /// If [peek](peek) has been called before, [advance](advance)
    /// will return the peeked character instead of advancing.
    ///
    /// # Example
    ///
    /// ```rust
    /// let lexer = Lexer::new("test".chars());
    /// assert_eq!(lexer.advance(), Some('t'));
    /// ```
    ///
    /// ```rust
    /// let lexer = Lexer::new("test".chars());
    /// assert_eq!(lexer.peek(), Some('t'));
    /// assert_eq!(lexer.advance(), Some('t'));
    /// ```
    fn advance(&mut self) -> Option<char> {
        self.col += 1;
        match self.char0 {
            None => self.input.next(),
            Some(c) => {
                self.char0 = None;
                Some(c)
            }
        }
    }
}

impl<T: Iterator<Item = char>> Iterator for Lexer<T> {
    type Item = LexResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.consume() {
            Ok((_, Token::Eof)) => None,
            Ok(token) => Some(Ok(token)),
            Err(err) => Some(Err(err)),
        }
    }
}

trait SymbolExt {
    fn is_symbol_start(&self) -> bool;
    fn is_symbol(&self) -> bool;
    fn is_misc(&self) -> bool;
}

impl SymbolExt for char {
    fn is_misc(&self) -> bool {
        *self == '*'
            || *self == '.'
            || *self == '!'
            || *self == '-'
            || *self == '_'
            || *self == '?'
            || *self == '$'
            || *self == '%'
            || *self == '&'
            || *self == '='
            || *self == '<'
            || *self == '>'
            || *self == '/'
            || *self == ':'
            || *self == '#'
            || *self == '+'
    }

    fn is_symbol_start(&self) -> bool {
        self.is_alphabetic() || self.is_misc()
    }

    fn is_symbol(&self) -> bool {
        self.is_alphanumeric() || self.is_misc()
    }
}

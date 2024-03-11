use serde::{Deserialize, Serialize};

pub type Line = usize;

pub type Col = usize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    BadChar(Line, Col, char),
    Number(Line, Col, String),
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub line: Line,
    pub col: Col,
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
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
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
//#[serde(tag = "type")] <- this does not seem to work with wasm
pub enum Token {
    LParen,
    RParen,
    Symbol(String),
    Number(f64),
    Eof,
}

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
            c if c.is_digit(10) => self.consume_number(c)?,
            c if c.is_symbol_start() => self.consume_symbol(c)?,
            c => return Err(Error::BadChar(self.line, self.col - 1, c)),
        }

        Ok(())
    }

    fn consume_number(&mut self, start: char) -> Result<(), Error> {
        let mut result = String::from(start);
        while matches!(self.peek(), Some(c) if c.is_digit(10)) {
            if let Some(c) = self.advance() {
                result.push(c);
            }
        }

        if self.peek() == Some('.') {
            self.advance();
            result.push('.');
            while matches!(self.peek(), Some(c) if c.is_digit(10)) {
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

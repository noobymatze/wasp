pub mod error;
pub mod lexer;
mod token;

use crate::parse::lexer::{lexer, LexResult};
use crate::parse::token::Token;
use crate::reporting::Region;
use error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Module {
    pub filename: Option<String>,
    pub expressions: Vec<Expr>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Expr {
    Number {
        region: Region,
        value: f64,
    },
    Symbol {
        region: Region,
        value: String,
    },
    List {
        region: Region,
        expressions: Vec<Expr>,
    },
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

                            return Ok(Expr::List {
                                region,
                                expressions,
                            });
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

impl From<(Region, f64)> for Expr {
    fn from((region, value): (Region, f64)) -> Self {
        Expr::Number { region, value }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use crate::parse::Expr;
    use crate::reporting::Region;
    use pretty_assertions::assert_eq;
    use std::vec;

    #[test]
    fn parse_test() {
        let actual = parse(None, "(def foo 5)").unwrap().expressions;
        let expected: Vec<Expr> = vec![list(
            (1, 1, 11),
            vec![
                sym((1, 2, 4), "def"),
                sym((1, 6, 8), "foo"),
                num((1, 10, 10), 5.0),
            ],
        )];

        assert_eq!(expected, actual)
    }

    fn list<R: Into<Region>>(region: R, expressions: Vec<Expr>) -> Expr {
        Expr::List {
            region: region.into(),
            expressions,
        }
    }

    fn num<R: Into<Region>>(region: R, value: f64) -> Expr {
        Expr::Number {
            region: region.into(),
            value,
        }
    }

    fn sym<R: Into<Region>>(region: R, value: &str) -> Expr {
        Expr::Symbol {
            region: region.into(),
            value: value.to_string(),
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Token {
    LParen,
    RParen,
    Symbol(String),
    Number(f64),
    Eof,
}

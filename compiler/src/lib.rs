use parse::error;
use crate::parse::Module;

pub mod parse;
pub mod reporting;

pub fn parse(filename: Option<String>, input: &str) -> Result<Module, error::Error> {
    parse::parse(filename, input)
}

#[cfg(test)]
mod tests {}

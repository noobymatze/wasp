use crate::parse::Module;

pub mod parse;

pub fn parse(filename: Option<String>, input: &str) -> Result<Module, parse::Error> {
    parse::parse(filename, input)
}

#[cfg(test)]
mod tests {}

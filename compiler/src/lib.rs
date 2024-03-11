use crate::parse::{lexer, Token};

pub mod parse;

pub fn parse(input: &str) -> Result< Vec<Token>, Vec<parse::Error>> {
    let mut errors = vec![];
    let mut tokens = vec![];
    for result in lexer(input) {
        match result {
            Ok((_, token)) => tokens.push(token),
            Err(error) => errors.push(error),
        }
    }

    if !errors.is_empty() {
        return Err(errors)
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {

}

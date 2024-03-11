use std::{fs, io};
use std::path::PathBuf;
use clap::Parser;
use compiler::parse::Token;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Run the given file.
    Run {
        /// File to be run, the default is main.
        file: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { file } => {
            let file = file.unwrap_or_else(|| "main.edn".into());
            let result = fs::read_to_string(file)?;
            let result = compiler::parse(result.as_str());
            match result {
                Ok(tokens) => {}
                Err(errors) => {
                    for err in errors {
                        println!("{:?}", err);
                    }
                }
            }

        }
    }

    Ok(())
}

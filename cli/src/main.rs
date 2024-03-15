use clap::Parser;
use std::path::PathBuf;
use std::{fs, io};

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

    /// Compile the given file.
    Compile {
        /// File to be run, the default is main.
        file: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Parse(compiler::parse::error::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

impl From<compiler::parse::error::Error> for Error {
    fn from(value: compiler::parse::error::Error) -> Self {
        Error::Parse(value)
    }
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { file } => {
            let file = file.unwrap_or_else(|| "main.edn".into());
            let result = fs::read_to_string(&file)?;
            let filename = file.to_str().map(|x| x.to_string());
            let result = compiler::parse(filename, result.as_str());
            match result {
                Ok(module) => {
                    let module = serde_json::to_string(&module)?;
                    println!("{}", module)
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
        Command::Compile { file } => {
            let file = file.unwrap_or_else(|| "main.edn".into());
            let result = fs::read_to_string(&file)?;
            let filename = file.to_str().map(|x| x.to_string());
            let mut result = compiler::compile(filename, result.as_str())?;
            fs::write("program.wasm", result)?;
        }
    }

    Ok(())
}

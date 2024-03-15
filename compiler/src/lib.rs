use crate::parse::{Expr, Module};
use parse::error;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, TypeSection,
    ValType,
};

pub mod compile;
pub mod parse;
pub mod reporting;

pub fn parse(filename: Option<String>, input: &str) -> Result<Module, error::Error> {
    parse::parse(filename, input)
}

pub fn compile(filename: Option<String>, input: &str) -> Result<Vec<u8>, error::Error> {
    compile::compile(filename, input)
}

#[cfg(test)]
mod tests {}

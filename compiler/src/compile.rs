use crate::parse;
use crate::parse::{Expr, Module};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, TypeSection,
    ValType,
};

pub fn compile(filename: Option<String>, input: &str) -> Result<Vec<u8>, parse::error::Error> {
    let module = parse::parse(filename, input)?;
    Ok(run(module))
}

struct WasmModule {
    types: TypeSection,
    functions: FunctionSection,
    exports: ExportSection,
    code: CodeSection,
    type_idx: u32,
}

fn run(module: Module) -> Vec<u8> {
    let mut wasm_module = WasmModule {
        types: TypeSection::new(),
        functions: FunctionSection::new(),
        exports: ExportSection::new(),
        code: CodeSection::new(),
        type_idx: 0,
    };

    for expr in &module.expressions {
        compile_expr(expr, &mut wasm_module);
    }

    let mut module = wasm_encoder::Module::new();

    module
        .section(&wasm_module.types)
        .section(&wasm_module.functions)
        .section(&wasm_module.exports)
        .section(&wasm_module.code);

    module.finish()
}

fn compile_expr(expr: &Expr, wasm_module: &mut WasmModule) {
    match expr {
        Expr::Number { .. } => {}
        Expr::Symbol { .. } => {}
        Expr::List { expressions, .. } => {
            let mut iter = expressions.iter();
            match iter.next() {
                Some(Expr::Symbol { value, .. }) => {
                    if value == "defn" {
                        compile_defn(wasm_module, wasm_module.type_idx, &mut iter);
                        wasm_module.type_idx += 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn compile_defn<'a>(
    wasm_module: &mut WasmModule,
    idx: u32,
    iter: &mut impl Iterator<Item = &'a Expr>,
) {
    wasm_module.types.function(vec![], vec![ValType::F64]);
    wasm_module.functions.function(idx);

    match iter.next() {
        Some(Expr::Symbol { value, .. }) => {
            wasm_module
                .exports
                .export(value.as_str(), ExportKind::Func, idx);
        }
        _ => {} // TODO: Wrong form, missing
    }

    iter.next(); // params for now are ignored
    let mut func = Function::new(vec![]);
    match iter.next() {
        Some(Expr::List { expressions, .. }) => {
            for expr in expressions.iter().rev() {
                match expr {
                    Expr::Number { value, .. } => {
                        func.instruction(&Instruction::F64Const(*value));
                    }
                    Expr::Symbol { value, .. } => {
                        func.instruction(
                            &compile_symbol_to_instruction(&value).expect("Do be defined"),
                        );
                    }
                    Expr::List { .. } => {}
                }
            }
            func.instruction(&Instruction::End);
        }
        _ => {}
    }

    wasm_module.code.function(&func);
}

fn compile_symbol_to_instruction(symbol: &String) -> Option<Instruction> {
    match symbol.as_str() {
        "+" => Some(Instruction::F64Add),
        "-" => Some(Instruction::F64Sub),
        "*" => Some(Instruction::F64Mul),
        "/" => Some(Instruction::F64Div),
        _ => None,
    }
}

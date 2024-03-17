use crate::parse;
use crate::parse::{Expr, Module};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, TypeSection,
    ValType,
};

pub fn compile(filename: Option<String>, input: &str) -> Result<Vec<u8>, parse::error::Error> {
    let module = parse::parse(filename, input)?;
    Ok(codegen(module))
}

struct WasmModule {
    types: TypeSection,
    functions: FunctionSection,
    exports: ExportSection,
    code: CodeSection,
    type_idx: u32,
}

impl WasmModule {
    fn get_and_increment_type_idx(&mut self) -> u32 {
        let cur = self.type_idx;
        self.type_idx += 1;
        cur
    }
}

fn codegen(module: Module) -> Vec<u8> {
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
        Expr::List { expressions, .. } => match expressions.as_slice() {
            [Expr::Symbol { value, .. }, Expr::Symbol { value: name, .. }, Expr::List {
                expressions: params,
                ..
            }, body]
                if value == "defn" =>
            {
                compile_defn(wasm_module, name, params, body);
            }
            _ => unimplemented!("Unknown form!"),
        },
    }
}

fn compile_defn(wasm_module: &mut WasmModule, name: &str, _params: &[Expr], body: &Expr) {
    let idx = wasm_module.get_and_increment_type_idx();
    wasm_module.types.function(vec![], vec![ValType::F64]);
    wasm_module.functions.function(idx);
    wasm_module.exports.export(name, ExportKind::Func, idx);

    let mut func = Function::new(vec![]);
    for instr in compile_instructions(&body) {
        func.instruction(&instr);
    }
    func.instruction(&Instruction::End);
    wasm_module.code.function(&func);
}

fn compile_instructions(expr: &Expr) -> Vec<Instruction> {
    let mut instructions = vec![];
    match expr {
        Expr::List { expressions, .. } => match expressions.as_slice() {
            [Expr::Symbol { value, .. }, args @ ..] => {
                let mut instrs = compile_expr_with_args(value, args);
                instructions.append(&mut instrs);
            }
            _ => {}
        },
        Expr::Number { value, .. } => instructions.push(Instruction::F64Const(*value)),
        Expr::Symbol { .. } => {}
    }

    instructions
}

fn compile_expr_with_args<'a>(symbol: &'a str, args: &'a [Expr]) -> Vec<Instruction<'a>> {
    match symbol {
        "+" => compile_bin_op(Instruction::F64Add, args),
        "-" => compile_bin_op(Instruction::F64Sub, args),
        "*" => compile_bin_op(Instruction::F64Mul, args),
        "/" => compile_bin_op(Instruction::F64Div, args),
        "<" => compile_bin_op(Instruction::F64Lt, args),
        "<=" => compile_bin_op(Instruction::F64Le, args),
        ">" => compile_bin_op(Instruction::F64Gt, args),
        ">=" => compile_bin_op(Instruction::F64Ge, args),
        _ => vec![],
    }
}

fn compile_bin_op<'a>(op: Instruction<'a>, args: &'a [Expr]) -> Vec<Instruction<'a>> {
    // (+ 3 5 6 7) -> (+ (+ (+ 3 5) 6) 7)
    // const 3
    // const 5
    // add
    // const 6
    // add
    // const 7
    // add
    match args {
        [head, rest @ ..] => {
            let mut instructions = vec![];
            instructions.append(&mut compile_instructions(&head));
            for expr in rest {
                instructions.append(&mut compile_instructions(&expr));
                instructions.push(op.clone());
            }

            instructions
        }
        _ => panic!("That's bad man."),
    }
}

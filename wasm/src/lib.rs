use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn main(a: usize, b: usize) -> usize {
    compiler::add(a, b)
}

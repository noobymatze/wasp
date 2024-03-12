use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn main(value: String) -> JsValue {
    let result = compiler::parse(None, value.as_str());
    let result = serde_wasm_bindgen::to_value(&result);
    match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Error serializing result: {:?}", error);
            serde_wasm_bindgen::to_value(&"An error occurred during serialization")
                .expect("Error handling failed")
        }
    }
}

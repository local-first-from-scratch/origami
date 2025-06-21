mod store;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: String) -> String {
    format!("Hello, {name}")
}

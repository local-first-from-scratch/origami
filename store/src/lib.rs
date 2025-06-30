mod op;
mod storage;
mod store;
mod timestamp;

#[cfg(test)]
mod test_helpers;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: String) -> String {
    format!("Hello, {name}")
}

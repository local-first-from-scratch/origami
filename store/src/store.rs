use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Store {}

#[wasm_bindgen]
impl Store {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}

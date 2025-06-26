use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct Store {}

#[wasm_bindgen]
impl Store {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}

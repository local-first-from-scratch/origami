mod utils;
mod timestamp;

#[cfg(test)]
mod test_helpers;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    set_panic_hook();

    alert("Hello, store!");
}

mod document;
mod operation;
mod timestamp;
mod utils;

#[cfg(test)]
mod test_helpers;

use document::Document;
use utils::set_panic_hook;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn dry_run() {
    set_panic_hook();

    let mut doc: Document<()> = Document::new();

    log(&format!("{doc:#?}"));

    {
        let root = doc.root();
        log(&format!("root: {root:#?}"))
    }

    {
        let new_root = doc.make_map(Uuid::from_u128(0));
        log(&format!("{new_root:#?}"));
        log(&format!("{doc:#?}"));
    }

    {
        match doc.root() {
            Some(document::Object::Map(map)) => {
                log(&format!("got a map after insertion, as expected: {map:#?}"));
            }
            None => log("no map after insertion, contrary to expectations"),
        }
    }
}

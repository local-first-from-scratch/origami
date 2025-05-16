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

#[wasm_bindgen]
pub fn subscribe(document_id: js_sys::JsString, cb: &js_sys::Function) -> u64 {
    // TODO: these are just roughed in for now
    log(&format!("subscribing to {document_id} with {cb:#?}"));

    log(&format!(
        "{:#?}",
        cb.call1(&JsValue::null(), &JsValue::from(1))
    ));

    0
}

#[wasm_bindgen]
pub fn unsubscribe(subscription_id: u64) {
    // TODO: these are just roughed in for now
    log(&format!("dropped subscription {subscription_id}"))
}

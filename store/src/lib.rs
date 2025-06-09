mod document;
mod hub;
mod timestamp;
mod utils;

#[cfg(test)]
mod test_helpers;

use document::{AssignKey, Document};
use std::collections::BTreeSet;
use utils::set_panic_hook;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn dry_run() -> Result<JsValue, serde_wasm_bindgen::Error> {
    set_panic_hook();

    let mut doc: Document = Document::default();
    let node = Uuid::from_u128(0);

    log(&format!("{doc:#?}"));

    {
        let map_id = doc.make_map("test".into(), node);
        log(&format!("new map: {map_id:#?}"));
        log(&format!("doc is now: {doc:#?}"));

        let world_id = doc.make_val("World".into(), "test".into(), node);
        doc.assign(
            map_id,
            AssignKey::MapKey("hello".into()),
            world_id,
            BTreeSet::new(),
            node,
        );

        let list_id = doc.make_list("test".into(), node);
        doc.assign(
            map_id,
            AssignKey::MapKey("list".into()),
            list_id,
            BTreeSet::new(),
            node,
        );

        let item_1 = doc.insert_after(list_id, node);
        doc.assign(
            list_id,
            AssignKey::InsertAfter(item_1),
            world_id,
            BTreeSet::new(),
            node,
        );

        let item_2 = doc.insert_after(item_1, node);
        doc.assign(
            list_id,
            AssignKey::InsertAfter(item_2),
            world_id,
            BTreeSet::new(),
            node,
        );

        let item_3 = doc.insert_after(item_2, node);
        doc.assign(
            list_id,
            AssignKey::InsertAfter(item_3),
            world_id,
            BTreeSet::new(),
            node,
        );

        let to_remove_id = doc.assign(
            map_id,
            AssignKey::MapKey("mistake".into()),
            world_id,
            BTreeSet::new(),
            node,
        );
        doc.remove(
            map_id,
            AssignKey::MapKey("mistake".into()),
            BTreeSet::from([to_remove_id]),
            node,
        );
    }

    log(&format!("new doc: {doc:#?}"));

    let v = doc.as_value();
    log(&format!("{v:#?}"));
    serde_wasm_bindgen::to_value(&doc.as_patch())
}

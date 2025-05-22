mod document;
mod fugue;
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
pub fn dry_run() {
    set_panic_hook();

    let mut doc: Document<&str> = Document::new();
    let node = Uuid::from_u128(0);

    log(&format!("{doc:#?}"));

    {
        let root = doc.root();
        log(&format!("first root: {root:#?}"))
    }

    {
        let map_id = doc.make_map(node);
        log(&format!("new map: {map_id:#?}"));
        log(&format!("doc is now: {doc:#?}"));

        let world_id = doc.make_val("World", node);
        doc.assign(
            map_id,
            AssignKey::MapKey("hello".into()),
            world_id,
            BTreeSet::new(),
            node,
        );

        let list_id = doc.make_list(node);
        doc.assign(
            map_id,
            AssignKey::MapKey("list".into()),
            world_id,
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

    log(&format!("new root: {:#?}", doc.root()));
    log(&format!("new doc: {doc:#?}"));
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

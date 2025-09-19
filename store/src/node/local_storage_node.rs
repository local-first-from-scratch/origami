use crate::{
    clock::{Clock, js_date::JsDate},
    hlc::Hlc,
};

use super::Node;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setItem, js_namespace = localStorage)]
    pub fn set_item(key: &str, value: JsValue);

    #[wasm_bindgen(js_name = getItem, js_namespace = localStorage)]
    pub fn get_item(key: &str) -> JsValue;
}

pub struct LocalStorageNode;

impl Node for LocalStorageNode {
    fn node_id(&self) -> u16 {
        match serde_wasm_bindgen::from_value::<Option<u16>>(get_item("node_id")) {
            Ok(Some(id)) => id,
            Ok(None) | Err(_) => {
                // we just replace the node ID on error. Naive but probably
                // fine; this should be pretty ephemeral and should only be used
                // for tiebreaks in HLCs.
                let new = (js_sys::Math::random() * u16::MAX as f64) as u16;

                // for now, it's fine if we just thrash if we can't store.
                let _ = serde_wasm_bindgen::to_value(&new).map(|v| set_item("node_id", v));

                new
            }
        }
    }

    fn clock(&self) -> crate::hlc::Hlc {
        match serde_wasm_bindgen::from_value::<Option<Hlc>>(get_item("clock")) {
            Ok(Some(clock)) => clock,
            Ok(None) | Err(_) => Hlc::new_at(JsDate::unix_timestamp(), 0, self.node_id()),
        }
    }

    fn receive_clock(&mut self, new: crate::hlc::Hlc) {
        if new > self.clock() {
            set_item("clock", serde_wasm_bindgen::to_value(&new).unwrap());
        }
    }
}

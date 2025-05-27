use std::collections::BTreeMap;

use uuid::Uuid;
use wasm_bindgen::JsValue;

#[derive(Debug, Default)]
pub struct Subscriptions {
    subscriptions: BTreeMap<Uuid, BTreeMap<usize, js_sys::Function>>,
    next_id: usize,
}

impl Subscriptions {
    pub fn new() -> Self {
        Subscriptions {
            subscriptions: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn subscribe(&mut self, doc: &Uuid, cb: js_sys::Function) -> usize {
        let id = self.next_id;
        let entry = self.subscriptions.entry(*doc).or_default();
        entry.insert(id, cb);

        self.next_id += 1;

        id
    }

    pub fn unsubscribe(&mut self, sub: usize) {
        for cbs in self.subscriptions.values_mut() {
            cbs.remove(&sub);
        }
    }

    pub fn notify(&self, doc: &Uuid) -> Result<(), Error> {
        if let Some(cbs) = self.subscriptions.get(doc) {
            for cb in cbs.values() {
                cb.call0(&JsValue::NULL).map_err(Error::Callback)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not notify callback: {0:?}")]
    Callback(JsValue),
}

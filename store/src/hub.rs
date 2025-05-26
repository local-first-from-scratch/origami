use crate::document::{Document, Value};
use js_sys::JsString;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct Hub {
    documents: BTreeMap<String, Arc<RwLock<Document>>>,
}

#[wasm_bindgen]
impl Hub {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            documents: BTreeMap::new(),
        }
    }

    pub fn create(&mut self) -> Handle {
        let doc_id = Uuid::new_v4();
        let doc = Arc::new(RwLock::new(Document::new()));

        self.documents.insert(doc_id.to_string(), Arc::clone(&doc));

        Handle { doc }
    }

    pub fn lookup(&self, document_id: js_sys::JsString) -> Handle {
        let id: String = document_id.into();
        let doc = self.documents.get(&id).unwrap();

        Handle {
            doc: Arc::clone(doc),
        }
    }

    pub fn subscribe(&mut self, document_id: js_sys::JsString, cb: &js_sys::Function) -> u64 {
        0
    }

    pub fn unsubscribe(&mut self, subscription_id: u64) {}
}

#[wasm_bindgen]
pub struct Handle {
    doc: Arc<RwLock<Document>>,
}

#[wasm_bindgen]
impl Handle {
    /// Get the current value. In cases where there is more than one current
    /// value, this will give you an arbitrary (but consistent) result.
    pub fn current(&self) -> Result<JsValue, JsString> {
        self.doc
            .read()
            .map(|doc| doc.as_value().into())
            .map_err(|e| e.to_string().into())
    }

    pub fn set(&self, key: JsString, value: JsValue) {
        let mut doc = self.doc.write().expect("a non-poisoned lock");
        let root = *doc.root().expect("an existing doc root");

        let me = Uuid::nil();

        let val_id = doc.make_val(
            value
                .try_into()
                .expect("a safe value to store in a document"),
            me,
        );
        doc.assign(
            root, // Use the stored root value directly
            crate::document::AssignKey::MapKey(key.into()),
            val_id,
            BTreeSet::new(),
            me,
        );
    }
}

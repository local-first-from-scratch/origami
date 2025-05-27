use crate::document::{Document, ValueError};
use js_sys::JsString;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, PoisonError, RwLock};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct Hub {
    actor: Arc<Uuid>,
    documents: BTreeMap<String, Arc<RwLock<Document>>>,
}

#[wasm_bindgen]
impl Hub {
    #[wasm_bindgen(constructor)]
    pub fn new(existing_id: Option<JsString>) -> Result<Self, Error> {
        let actor = match existing_id {
            Some(existing) => Uuid::try_parse(&Into::<String>::into(existing))?,
            None => Uuid::new_v4(),
        };

        Ok(Self {
            actor: Arc::new(actor),
            documents: BTreeMap::new(),
        })
    }

    pub fn actor_id(&self) -> JsString {
        self.actor.to_string().into()
    }

    pub fn create(&mut self, root_kind: RootKind) -> Handle {
        let doc_id = Uuid::new_v4();

        let mut doc = Document::new();
        match root_kind {
            RootKind::Map => doc.make_map(*self.actor),
            RootKind::List => doc.make_list(*self.actor),
        };

        let doc = Arc::new(RwLock::new(doc));

        self.documents.insert(doc_id.to_string(), Arc::clone(&doc));

        Handle {
            actor: Arc::clone(&self.actor),
            doc,
        }
    }

    pub fn lookup(&self, document_id: js_sys::JsString) -> Handle {
        let id: String = document_id.into();
        let doc = self.documents.get(&id).unwrap();

        Handle {
            actor: Arc::clone(&self.actor),
            doc: Arc::clone(doc),
        }
    }

    pub fn subscribe(&mut self, document_id: js_sys::JsString, cb: &js_sys::Function) -> u64 {
        0
    }

    pub fn unsubscribe(&mut self, subscription_id: u64) {}
}

#[wasm_bindgen]
pub enum RootKind {
    Map,
    List,
}

#[wasm_bindgen]
pub struct Handle {
    actor: Arc<Uuid>,
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

    pub fn set(&self, key: JsString, value: JsValue) -> Result<(), Error> {
        let mut doc = self.doc.write()?;
        let root = *doc.root().ok_or(Error::MissingRoot)?;

        let val_id = doc.make_val(value.try_into()?, *self.actor);

        doc.assign(
            root, // Use the stored root value directly
            crate::document::AssignKey::MapKey(key.into()),
            val_id,
            BTreeSet::new(),
            *self.actor,
        );

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error with UUID: {0}")]
    BadUuid(#[from] uuid::Error),
    #[error("Lock was poisoned")]
    PoisonError,
    #[error("Missing document root")]
    MissingRoot,
    #[error("Could not convert value: {0}")]
    ValueConversion(#[from] ValueError),
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        value.to_string().into()
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonError
    }
}

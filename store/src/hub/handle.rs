use super::subscriptions::{self, Subscriptions};
use crate::document::{Document, ValueError};
use js_sys::JsString;
use std::sync::{Arc, PoisonError, RwLock};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Handle {
    actor: Arc<Uuid>,
    doc: Arc<RwLock<Document>>,
    doc_id: Uuid,
    subscriptions: Arc<RwLock<Subscriptions<js_sys::Function>>>,
}

impl Handle {
    pub fn new(
        actor: Arc<Uuid>,
        doc: Arc<RwLock<Document>>,
        doc_id: Uuid,
        subscriptions: Arc<RwLock<Subscriptions<js_sys::Function>>>,
    ) -> Self {
        Self {
            actor,
            doc,
            doc_id,
            subscriptions,
        }
    }
}

#[wasm_bindgen]
impl Handle {
    /// Get the document ID
    #[wasm_bindgen(getter, js_name = documentId)]
    pub fn document_id(&self) -> JsString {
        self.doc_id.to_string().into()
    }

    /// Get the current value. In cases where there is more than one current
    /// value, this will give you an arbitrary (but consistent) result.
    pub fn current(&self) -> Result<JsValue, JsString> {
        let doc = self.doc.read().map_err(|e| e.to_string())?;

        serde_wasm_bindgen::to_value(&doc.as_value()).map_err(|e| e.to_string().into())
    }

    /// Assign a value in a map at the current location.
    pub fn set(&self, key: JsString, value: JsValue) -> Result<(), Error> {
        // Make the write
        {
            let mut doc = self.doc.write()?;
            let root = *doc.root().ok_or(Error::MissingRoot)?;

            let val_id = doc.make_val(value.try_into()?, *self.actor);

            let key = crate::document::AssignKey::MapKey(key.into());

            let current = doc.current_assigns(&root, &key);

            doc.assign(root, key, val_id, current, *self.actor);
        }

        // Notify subscribers
        {
            let subs = self.subscriptions.read()?;
            subs.notify(&self.doc_id)?;
        }

        Ok(())
    }

    /// Subscribe to changes at the current document/location. This will give
    /// you a subscription ID. When you're done listening, call
    /// `Hub.unsubscribe` with that ID to clean up.
    pub fn subscribe(&mut self, cb: js_sys::Function) -> Result<usize, Error> {
        let mut subs = self.subscriptions.write()?;
        Ok(subs.subscribe(&self.doc_id, cb))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lock was poisoned")]
    LockWasPoisoned,
    #[error("Missing document root")]
    MissingRoot,
    #[error("Could not convert value: {0}")]
    ValueConversion(#[from] ValueError),
    #[error("Error in subscription: {0}")]
    Notify(#[from] subscriptions::Error),
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        value.to_string().into()
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::LockWasPoisoned
    }
}

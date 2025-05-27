use super::subscriptions::{self, Subscriptions};
use crate::document::{Document, ValueError};
use js_sys::JsString;
use std::collections::BTreeSet;
use std::sync::{Arc, PoisonError, RwLock};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Handle {
    actor: Arc<Uuid>,
    doc: Arc<RwLock<Document>>,
    doc_id: Uuid,
    subscriptions: Arc<RwLock<Subscriptions>>,
}

impl Handle {
    pub fn new(
        actor: Arc<Uuid>,
        doc: Arc<RwLock<Document>>,
        doc_id: Uuid,
        subscriptions: Arc<RwLock<Subscriptions>>,
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
    /// Get the current value. In cases where there is more than one current
    /// value, this will give you an arbitrary (but consistent) result.
    pub fn current(&self) -> Result<JsValue, JsString> {
        self.doc
            .read()
            .map(|doc| doc.as_value().into())
            .map_err(|e| e.to_string().into())
    }

    pub fn set(&self, key: JsString, value: JsValue) -> Result<(), Error> {
        // Make the write
        {
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
        }

        // Notify subscribers
        {
            let subs = self.subscriptions.read()?;
            subs.notify(&self.doc_id)?;
        }

        Ok(())
    }

    pub fn subscribe(&mut self, cb: js_sys::Function) -> Result<usize, Error> {
        let mut subs = self.subscriptions.write()?;
        Ok(subs.subscribe(&self.doc_id, cb))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error with UUID: {0}")]
    BadUuid(#[from] uuid::Error),
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

mod handle;
mod reader;
mod subscriptions;

use crate::document::Document;
use handle::Handle;
use js_sys::JsString;
use std::collections::BTreeMap;
use std::sync::{Arc, PoisonError, RwLock};
use subscriptions::Subscriptions;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct Hub {
    actor: Arc<Uuid>,
    documents: BTreeMap<Uuid, Arc<RwLock<Document>>>,
    subscriptions: Arc<RwLock<Subscriptions<js_sys::Function>>>,
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
            subscriptions: Arc::default(),
        })
    }

    #[wasm_bindgen(getter, js_name = actorId)]
    pub fn actor_id(&self) -> JsString {
        self.actor.to_string().into()
    }

    pub fn create(&mut self, root_kind: RootKind, schema: String) -> Handle {
        let doc_id = Uuid::new_v4();

        let mut doc = Document::default();
        match root_kind {
            RootKind::Map => doc.make_map(schema, *self.actor),
            RootKind::List => doc.make_list(schema, *self.actor),
        };

        let doc = Arc::new(RwLock::new(doc));

        self.documents.insert(doc_id, Arc::clone(&doc));

        Handle::new(
            Arc::clone(&self.actor),
            doc,
            doc_id,
            Arc::clone(&self.subscriptions),
        )
    }

    pub fn lookup(&self, document_id: js_sys::JsString) -> Result<Handle, Error> {
        let owned_document_id: String = document_id.into();
        let doc_id: Uuid = owned_document_id.try_into()?;
        let doc = self.documents.get(&doc_id).unwrap();

        Ok(Handle::new(
            Arc::clone(&self.actor),
            Arc::clone(doc),
            doc_id,
            Arc::clone(&self.subscriptions),
        ))
    }

    pub fn unsubscribe(&mut self, subscription_id: usize) -> Result<(), Error> {
        let mut subs = self.subscriptions.write()?;
        subs.unsubscribe(subscription_id);

        Ok(())
    }
}

#[wasm_bindgen]
pub enum RootKind {
    Map,
    List,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error with UUID: {0}")]
    BadUuid(#[from] uuid::Error),
    #[error("Lock was poisoned")]
    LockWasPoisoned,
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

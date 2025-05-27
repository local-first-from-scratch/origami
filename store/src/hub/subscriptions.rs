use std::collections::BTreeMap;

use uuid::Uuid;
use wasm_bindgen::JsValue;

#[derive(Debug, Default)]
pub struct Subscriptions<Callback> {
    subscriptions: BTreeMap<Uuid, BTreeMap<usize, Callback>>,
    next_id: usize,
}

impl<Callback> Subscriptions<Callback> {
    pub fn new() -> Self {
        Subscriptions {
            subscriptions: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn subscribe(&mut self, doc: &Uuid, cb: Callback) -> usize {
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
}

impl Subscriptions<js_sys::Function> {
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

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_assumptions() {
        let subs: Subscriptions<usize> = Subscriptions::default();
        assert!(subs.subscriptions.is_empty());
        assert_eq!(subs.next_id, 0);
    }

    #[test]
    fn subscribe() {
        let mut subs = Subscriptions::new();
        let doc_id = Uuid::nil();

        let sub_id = subs.subscribe(&doc_id, 1000);

        assert_eq!(sub_id, 0);
        assert_eq!(subs.next_id, 1);

        assert!(subs.subscriptions.contains_key(&doc_id));
        assert_eq!(
            subs.subscriptions.get(&doc_id),
            Some(&BTreeMap::from([(0, 1000)]))
        );
    }

    #[test]
    fn unsubscribe() {
        let mut subs = Subscriptions::new();
        let doc_id = Uuid::nil();

        let sub_id = subs.subscribe(&doc_id, 1000);
        subs.unsubscribe(sub_id);

        assert_eq!(subs.subscriptions.get(&doc_id), Some(&BTreeMap::new()));
    }

    #[test]
    fn multiple_documents() {
        let mut subs = Subscriptions::new();
        let doc_id1 = Uuid::nil();
        let doc_id2 = Uuid::max();

        let sub_id1 = subs.subscribe(&doc_id1, 1000);
        let sub_id2 = subs.subscribe(&doc_id2, 1001);

        assert_eq!(sub_id1, 0);
        assert_eq!(sub_id2, 1);
        assert_eq!(
            subs.subscriptions,
            BTreeMap::from([
                (doc_id1, BTreeMap::from([(sub_id1, 1000)])),
                (doc_id2, BTreeMap::from([(sub_id2, 1001)])),
            ])
        );
    }

    #[test]
    fn multiple_subscriptions_same_document() {
        let mut subs = Subscriptions::new();
        let doc_id = Uuid::nil();

        let sub_id1 = subs.subscribe(&doc_id, 1000);
        let sub_id2 = subs.subscribe(&doc_id, 1001);

        assert_eq!(subs.subscriptions.len(), 1);
        assert_eq!(
            subs.subscriptions.get(&doc_id),
            Some(&BTreeMap::from([(sub_id1, 1000), (sub_id2, 1001)]))
        );

        subs.unsubscribe(sub_id1);
        assert_eq!(subs.subscriptions.get(&doc_id).unwrap().get(&sub_id1), None);
    }
}

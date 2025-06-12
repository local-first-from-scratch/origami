#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyOrIndex {
    Key(String),
    Index(usize),
    LastIndex,
}

impl From<&String> for KeyOrIndex {
    fn from(v: &String) -> Self {
        Self::Key(v.clone())
    }
}

impl From<&str> for KeyOrIndex {
    fn from(v: &str) -> Self {
        Self::Key(v.to_string())
    }
}

impl From<usize> for KeyOrIndex {
    fn from(v: usize) -> Self {
        Self::Index(v)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(Vec<KeyOrIndex>);

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Path {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn with_next_segment(&self, segment: KeyOrIndex) -> Self {
        let mut path = self.clone();
        path.0.push(segment);
        path
    }

    pub fn all_but_last(&self) -> impl Iterator<Item = &KeyOrIndex> {
        self.0.iter().take(self.0.len().max(1) - 1)
    }

    pub fn last(&self) -> Option<&KeyOrIndex> {
        self.0.last()
    }
}

impl<const N: usize> From<[KeyOrIndex; N]> for Path {
    fn from(v: [KeyOrIndex; N]) -> Self {
        Self(v.to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_path_is_root() {
        let path = Path::from([]);
        assert!(path.is_root());
    }

    #[test]
    fn non_empty_path_is_not_root() {
        let path = Path::from(["foo".into()]);
        assert!(!path.is_root());
    }

    #[test]
    fn with_next_segment_appends_segment() {
        let path = Path::from(["foo".into()]);
        let new_path = path.with_next_segment("bar".into());
        assert_eq!(new_path.0, vec!["foo".into(), "bar".into()]);
    }

    #[test]
    fn all_but_last_is_empty_for_empty_array() {
        let path = Path::from([]);
        assert_eq!(path.all_but_last().count(), 0);
    }

    #[test]
    fn all_but_last_is_empty_for_single_element_array() {
        let path = Path::from(["foo".into()]);
        assert_eq!(path.all_but_last().count(), 0);
    }

    #[test]
    fn all_but_last_gives_prefix_to_longer_list() {
        let path = Path::from([KeyOrIndex::LastIndex, "foo".into()]);

        assert_eq!(
            path.all_but_last().cloned().collect::<Vec<KeyOrIndex>>(),
            Vec::from([KeyOrIndex::LastIndex])
        );
    }

    #[test]
    fn last_is_none_for_empty_array() {
        let path = Path::from([]);
        assert_eq!(path.last(), None);
    }

    #[test]
    fn last_gets_last_element() {
        let path = Path::from(["foo".into()]);
        assert_eq!(path.last(), Some(&KeyOrIndex::Key("foo".into())));
    }
}

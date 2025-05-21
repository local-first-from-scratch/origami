pub struct Fugue<V> {
    root: Option<Node<V>>,
}

impl<V> Fugue<V> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, index: usize, value: V) {
        match &mut self.root {
            None => self.root = Some(Node::new(value)),
            Some(root) => root.insert(index, value),
        }
    }

    pub fn get(&self, index: usize) -> Option<&V> {
        self.root.as_ref().and_then(|r| r.get(index))
    }
}

#[derive(Debug)]
struct Node<V> {
    left: Vec<Node<V>>,
    here: V,
    right: Vec<Node<V>>,
}

impl<V> Node<V> {
    fn new(v: V) -> Self {
        Self {
            left: Vec::new(),
            here: v,
            right: Vec::new(),
        }
    }

    fn insert(&mut self, index: usize, value: V) {
        todo!()
    }

    fn iter(&self) -> FugueIterator<V> {
        FugueIterator::new(self)
    }

    fn get(&self, index: usize) -> Option<&V> {
        self.iter().nth(index)
    }
}

pub struct FugueIterator<'fugue, V> {
    stack: Vec<&'fugue Node<V>>,
}

impl<'fugue, V> FugueIterator<'fugue, V> {
    fn new(root: &'fugue Node<V>) -> Self {
        let mut iter = Self { stack: Vec::new() };
        iter.push_lefts(root);

        iter
    }

    fn push_lefts(&mut self, node: &'fugue Node<V>) {
        self.stack.push(node);

        while let Some(node) = self.stack.last() {
            if node.left.is_empty() {
                return;
            }

            for child in node.left.iter().rev() {
                self.stack.push(child);
            }
        }
    }

    fn push_rights(&mut self, node: &'fugue Node<V>) {
        for child in node.right.iter().rev() {
            self.push_lefts(child)
        }
    }
}

impl<'fugue, V> Iterator for FugueIterator<'fugue, V> {
    type Item = &'fugue V;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            self.push_rights(node);

            &node.here
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iteration_iterates_correctly() {
        // encode 1 through 7 in the binary tree. We don't care about balancing
        // for Fugue, but we care about it here so that we're sure to hit all
        // the cases.
        let node_1 = Node::new(1);
        let node_3 = Node::new(3);
        let node_123 = Node {
            left: vec![node_1],
            here: 2,
            right: vec![node_3],
        };

        let node_5 = Node::new(5);
        let node_7 = Node::new(7);
        let node_567 = Node {
            left: vec![node_5],
            here: 6,
            right: vec![node_7],
        };

        let node_1234567 = Node {
            left: vec![node_123],
            here: 4,
            right: vec![node_567],
        };

        let numbers: Vec<usize> = node_1234567.iter().copied().collect();

        assert_eq!(vec![1, 2, 3, 4, 5, 6, 7], numbers);
    }
}

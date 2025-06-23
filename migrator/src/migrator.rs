use crate::lens::Lens;
use crate::migration::Migration;
use petgraph::{Directed, Graph, algo::astar, graph::NodeIndex};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Migrator {
    root_node_id: NodeIndex,
    node_ids: HashMap<String, NodeIndex>,
    graph: Graph<(), Vec<Lens>, Directed>,
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrator {
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let root_node_id = graph.add_node(());

        Migrator {
            root_node_id,
            node_ids: HashMap::from([("ROOT".to_string(), root_node_id)]),
            graph,
        }
    }

    fn node_id(&mut self, name: &str) -> NodeIndex {
        match self.node_ids.get(name) {
            Some(id) => *id,
            None => {
                let id = self.graph.add_node(());
                self.node_ids.insert(name.to_string(), id);
                id
            }
        }
    }

    pub fn add_migration(&mut self, migration: Migration) {
        let Migration { id, base, ops } = migration;

        // Generate our node IDs so we can add edges
        let node_id = self.node_id(&id);
        let base_id = match base {
            None => self.root_node_id,
            Some(base) => self.node_id(&base),
        };

        // Keep track of the name so we can navigate to/from it later
        self.node_ids.insert(id, node_id);

        // Add the paths to base and back, both forward and reverse
        self.graph.add_edge(
            node_id,
            base_id,
            ops.iter().rev().map(|lens| lens.reversed()).collect(),
        );
        self.graph.add_edge(base_id, node_id, ops);
    }

    pub fn migration_path(&self, from: Option<&str>, to: &str) -> Option<Vec<&Lens>> {
        let source_node_id = match from {
            Some(from) => *self.node_ids.get(from)?,
            None => self.root_node_id,
        };
        let dest_node_id = *self.node_ids.get(to)?;

        astar(
            &self.graph,
            source_node_id,
            |n| n == dest_node_id,
            |_| 1,
            |_| 0,
        )
        .map(|(_, path)| {
            let mut out = Vec::with_capacity(path.len());

            for (src, dest) in path.iter().zip(path.iter().skip(1)) {
                if let Some(ops) = self
                    .graph
                    .find_edge(*src, *dest)
                    .and_then(|edge| self.graph.edge_weight(edge))
                {
                    out.extend(ops)
                }
            }

            out
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! lens {
        ($patch:tt) => {
            serde_json::from_value::<Lens>(serde_json::json!($patch)).unwrap()
        };
    }

    #[test]
    fn migration_path() {
        let mut migrator = Migrator::new();

        let id_a = "a";
        let lens_a = lens!({"add": {
            "name": "a",
            "type": "string",
        }});
        let migration_a = Migration {
            id: id_a.to_string(),
            base: None,
            ops: vec![lens_a.clone()],
        };
        migrator.add_migration(migration_a.clone());

        let id_b = "b";
        let lens_b = lens!({"rename": {
            "from": "a",
            "to": "b"
        }});
        let migration_b = Migration {
            id: id_b.to_string(),
            base: Some(id_a.to_string()),
            ops: vec![lens_b.clone()],
        };
        migrator.add_migration(migration_b.clone());

        let id_c = "c";
        let lens_c = lens!({"rename": {
            "from": "b",
            "to": "c"
        }});
        let migration_c = Migration {
            id: id_c.to_string(),
            base: Some(id_b.to_string()),
            ops: vec![lens_c.clone()],
        };
        migrator.add_migration(migration_c.clone());

        assert_eq!(
            Some(vec![&lens_a, &lens_b, &lens_c]),
            migrator.migration_path(None, id_c)
        );

        assert_eq!(
            Some(vec![&lens_c.reversed(), &lens_b.reversed()]),
            migrator.migration_path(Some(id_c), id_a)
        );
    }
}

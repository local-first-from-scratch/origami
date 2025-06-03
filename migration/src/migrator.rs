use petgraph::{Graph, Undirected, algo::astar, graph::NodeIndex};

use crate::migration::Migration;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Migrator {
    migrations: HashMap<NodeIndex, Migration>,
    root_node_id: NodeIndex,
    node_ids: HashMap<String, NodeIndex>,
    graph: Graph<String, (), Undirected>,
}

const ROOT_ID: &str = "ROOT";

impl Migrator {
    pub fn new() -> Self {
        let mut graph = Graph::new_undirected();
        let root_node_id = graph.add_node(ROOT_ID.to_string());

        let migrations = HashMap::from([(
            root_node_id,
            Migration {
                base: None,
                ops: vec![],
            },
        )]);

        Migrator {
            migrations,
            root_node_id,
            node_ids: HashMap::new(),
            graph,
        }
    }

    pub fn add_migration(&mut self, name: String, migration: Migration) {
        let node_id = self.graph.add_node(name.clone());

        self.node_ids.insert(name, node_id);

        let base_id = match &migration.base {
            None => self.root_node_id,
            Some(base) => match self.node_ids.get(base) {
                Some(id) => *id,
                None => {
                    let id = self.graph.add_node(base.clone());
                    self.node_ids.insert(base.clone(), id);
                    id
                }
            },
        };

        self.graph.add_edge(base_id, node_id, ());

        self.migrations.insert(node_id, migration);
    }

    pub fn migration_path(&self, from: Option<&str>, to: &str) -> Option<Vec<&Migration>> {
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
            path.iter()
                // We skip the first migration because we assume we're already
                // at the state given by `from`.
                .skip(1)
                .map(|&node_id| self.migrations.get(&node_id).unwrap())
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn migration_path() {
        let mut migrator = Migrator::new();

        let name_a = "a";
        let migration_a = Migration {
            base: None,
            ops: vec![],
        };
        migrator.add_migration(name_a.to_string(), migration_a.clone());

        let name_b = "b";
        let migration_b = Migration {
            base: Some(name_a.to_string()),
            ops: vec![],
        };
        migrator.add_migration(name_b.to_string(), migration_b.clone());

        let name_c = "c";
        let migration_c = Migration {
            base: Some(name_b.to_string()),
            ops: vec![],
        };
        migrator.add_migration(name_c.to_string(), migration_c.clone());

        let path = migrator.migration_path(None, name_c);
        println!("{:#?}", path);

        assert_eq!(path, Some(vec![&migration_a, &migration_b, &migration_c]));
    }
}

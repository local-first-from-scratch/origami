use petgraph::{Graph, Undirected, algo::astar, graph::NodeIndex};

use crate::migration::Migration;
use std::collections::HashMap;

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

    pub fn add_migration(&mut self, name: String, migration: Migration) -> Result<(), AddError> {
        let node_id = self.graph.add_node(name.clone());

        self.node_ids.insert(name, node_id);

        let base_id = match &migration.base {
            Some(base) => *self
                .node_ids
                .get(base)
                .ok_or_else(|| AddError::MissingBase(base.clone()))?,
            None => self.root_node_id,
        };

        self.graph.add_edge(base_id, node_id, ());

        self.migrations.insert(node_id, migration);

        Ok(())
    }

    fn migration_path(&mut self, from: &str, to: &str) -> Option<Vec<&Migration>> {
        let source_node_id = *self.node_ids.get(from)?;
        let dest_node_id = *self.node_ids.get(to)?;

        astar(
            &self.graph,
            source_node_id,
            |g| g == dest_node_id,
            |_| 1,
            |_| 0,
        )
        .map(|(_, path)| {
            path.iter()
                .map(|&node_id| self.migrations.get(&node_id).unwrap())
                .collect()
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddError {
    #[error("Base migration not found: {0}")]
    MissingBase(String),
}

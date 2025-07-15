use crate::hlc::Hlc;

use super::Node;

pub struct MemoryNode {
    id: u16,
    clock: Hlc,
}

impl MemoryNode {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            clock: Hlc::new_at(0, 0, id),
        }
    }
}

impl Node for MemoryNode {
    fn node_id(&self) -> u16 {
        self.id
    }

    fn clock(&self) -> Hlc {
        self.clock
    }

    fn receive_clock(&mut self, clock: Hlc) {
        if self.clock > clock {
            self.clock = clock
        }
    }
}

pub mod local_storage_node;
pub mod memory_node;

use crate::hlc::Hlc;

/// Small pieces of node-specific information that shouldn't be shared. Ideally
/// should be pretty ephemeral as well! If, e.g. localStorage chooses to boot
/// this data out it should be OK to recover or regenerate it from other
/// sources.
pub trait Node {
    /// Get the current node ID
    fn node_id(&self) -> u16;

    /// Get the highest cached clock
    fn clock(&self) -> Hlc;

    /// Receive a clock, storing it if it's higher than the one we have. If you
    /// get a clock that's higher than the one you have already, you're expected
    /// to retrieve it (with the node ID reset to yours) in the next call tock
    /// `clock`.
    fn receive_clock(&mut self, clock: Hlc);
}

//! Breakpoint manager — set, remove, and handle software breakpoints (INT3).

use crate::{BreakpointId, DebugError};
use std::collections::HashMap;

/// Manages software breakpoints for a debug session.
#[derive(Debug, Default)]
pub struct BreakpointManager {
    breakpoints: HashMap<BreakpointId, Breakpoint>,
    next_id: BreakpointId,
}

/// A software breakpoint (INT3 / 0xCC).
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: BreakpointId,
    pub address: u64,
    pub original_byte: u8,
    pub state: BreakpointState,
    pub hit_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakpointState {
    Enabled,
    Disabled,
    Pending,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_id: 1,
        }
    }

    /// Set a breakpoint at the given address.
    pub fn set(&mut self, address: u64, original_byte: u8) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint {
            id,
            address,
            original_byte,
            state: BreakpointState::Enabled,
            hit_count: 0,
        };
        self.breakpoints.insert(id, bp);
        id
    }

    /// Remove a breakpoint by ID.
    pub fn remove(&mut self, id: BreakpointId) -> Result<Breakpoint, DebugError> {
        self.breakpoints
            .remove(&id)
            .ok_or(DebugError::BreakpointNotFound { id })
    }

    /// Get a breakpoint by ID.
    pub fn get(&self, id: BreakpointId) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Get a mutable breakpoint by ID.
    pub fn get_mut(&mut self, id: BreakpointId) -> Option<&mut Breakpoint> {
        self.breakpoints.get_mut(&id)
    }

    /// Find a breakpoint by address.
    pub fn find_by_address(&self, address: u64) -> Option<&Breakpoint> {
        self.breakpoints.values().find(|bp| bp.address == address)
    }

    /// List all breakpoints.
    pub fn list(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// List addresses of all active (enabled) breakpoints.
    pub fn list_active(&self) -> std::collections::HashSet<u64> {
        self.breakpoints
            .values()
            .filter(|bp| bp.state == BreakpointState::Enabled)
            .map(|bp| bp.address)
            .collect()
    }

    /// Get a map of active breakpoint addresses to their original bytes.
    pub fn active_original_bytes(&self) -> std::collections::HashMap<u64, u8> {
        self.breakpoints
            .values()
            .filter(|bp| bp.state == BreakpointState::Enabled)
            .map(|bp| (bp.address, bp.original_byte))
            .collect()
    }

    /// Handle a breakpoint hit: increment counter and return the breakpoint.
    pub fn hit(&mut self, id: BreakpointId) -> Result<&Breakpoint, DebugError> {
        let bp = self
            .breakpoints
            .get_mut(&id)
            .ok_or(DebugError::BreakpointNotFound { id })?;
        bp.hit_count += 1;
        Ok(bp)
    }
}

//! Tracepoint Attach Point
//!
//! Static kernel tracepoints for observability.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use super::{AttachError, AttachId, AttachPoint, AttachResult, AttachType};
use crate::bytecode::program::BpfProgram;
use crate::profile::{ActiveProfile, PhysicalProfile};

/// Tracepoint attach point.
pub struct TracepointAttach<P: PhysicalProfile = ActiveProfile> {
    /// Tracepoint category (e.g., "syscalls", "sched")
    category: String,
    /// Tracepoint name (e.g., "sys_enter_write")
    name: String,
    /// Attached program IDs
    attached: Vec<AttachId>,
    /// Next ID counter
    next_id: u32,
    /// Profile marker (using fn pointer for Send + Sync)
    _profile: PhantomData<fn() -> P>,
}

impl<P: PhysicalProfile> TracepointAttach<P> {
    /// Create a new tracepoint attach point.
    pub fn new(category: &str, name: &str) -> AttachResult<Self> {
        if category.is_empty() || name.is_empty() {
            return Err(AttachError::InvalidTarget(alloc::format!(
                "{}:{}", category, name
            )));
        }

        Ok(Self {
            category: category.into(),
            name: name.into(),
            attached: Vec::new(),
            next_id: 1,
            _profile: PhantomData,
        })
    }

    /// Get the category.
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Get the tracepoint name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<P: PhysicalProfile> AttachPoint<P> for TracepointAttach<P> {
    fn attach_type(&self) -> AttachType {
        AttachType::Tracepoint
    }

    fn target(&self) -> &str {
        // Return category:name format
        // Note: this allocates, consider caching if performance matters
        &self.name
    }

    fn attach(&mut self, _program: &BpfProgram<P>) -> AttachResult<AttachId> {
        let id = AttachId(self.next_id);
        self.next_id += 1;
        self.attached.push(id);
        Ok(id)
    }

    fn detach(&mut self, id: AttachId) -> AttachResult<()> {
        if let Some(idx) = self.attached.iter().position(|&i| i == id) {
            self.attached.remove(idx);
            Ok(())
        } else {
            Err(AttachError::ResourceNotFound)
        }
    }

    fn is_attached(&self, id: AttachId) -> bool {
        self.attached.contains(&id)
    }

    fn attached_ids(&self) -> Vec<AttachId> {
        self.attached.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tracepoint() {
        let tp = TracepointAttach::<ActiveProfile>::new("syscalls", "sys_enter_write").unwrap();
        assert_eq!(tp.category(), "syscalls");
        assert_eq!(tp.name(), "sys_enter_write");
    }
}

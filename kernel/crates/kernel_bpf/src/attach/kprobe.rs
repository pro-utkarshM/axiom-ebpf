//! Kprobe Attach Point
//!
//! Kernel probes allow attaching BPF programs to kernel function entry/exit.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use super::{AttachError, AttachId, AttachPoint, AttachResult, AttachType};
use crate::bytecode::program::BpfProgram;
use crate::profile::{ActiveProfile, PhysicalProfile};

/// Type of kernel probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KprobeType {
    /// Function entry probe
    Entry,
    /// Function return probe
    Return,
}

/// Kprobe attach point.
pub struct KprobeAttach<P: PhysicalProfile = ActiveProfile> {
    /// Target function name
    function: String,
    /// Probe type (entry or return)
    probe_type: KprobeType,
    /// Attached program IDs
    attached: Vec<AttachId>,
    /// Next ID counter
    next_id: u32,
    /// Profile marker (using fn pointer for Send + Sync)
    _profile: PhantomData<fn() -> P>,
}

impl<P: PhysicalProfile> KprobeAttach<P> {
    /// Create a new kprobe attach point.
    pub fn new(function: &str, probe_type: KprobeType) -> AttachResult<Self> {
        if function.is_empty() {
            return Err(AttachError::InvalidTarget(function.into()));
        }

        Ok(Self {
            function: function.into(),
            probe_type,
            attached: Vec::new(),
            next_id: 1,
            _profile: PhantomData,
        })
    }

    /// Get the target function name.
    pub fn function(&self) -> &str {
        &self.function
    }

    /// Get the probe type.
    pub fn probe_type(&self) -> KprobeType {
        self.probe_type
    }
}

impl<P: PhysicalProfile> AttachPoint<P> for KprobeAttach<P> {
    fn attach_type(&self) -> AttachType {
        match self.probe_type {
            KprobeType::Entry => AttachType::Kprobe,
            KprobeType::Return => AttachType::Kretprobe,
        }
    }

    fn target(&self) -> &str {
        &self.function
    }

    fn attach(&mut self, _program: &BpfProgram<P>) -> AttachResult<AttachId> {
        let id = AttachId(self.next_id);
        self.next_id += 1;
        self.attached.push(id);

        // In a real implementation, this would:
        // 1. Register the kprobe with the kernel
        // 2. Associate the BPF program with the probe

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
    fn create_kprobe() {
        let kprobe = KprobeAttach::<ActiveProfile>::new("sys_write", KprobeType::Entry).unwrap();
        assert_eq!(kprobe.function(), "sys_write");
        assert_eq!(kprobe.probe_type(), KprobeType::Entry);
        assert_eq!(kprobe.attach_type(), AttachType::Kprobe);
    }

    #[test]
    fn create_kretprobe() {
        let kprobe = KprobeAttach::<ActiveProfile>::new("sys_read", KprobeType::Return).unwrap();
        assert_eq!(kprobe.attach_type(), AttachType::Kretprobe);
    }

    #[test]
    fn invalid_function_name() {
        let result = KprobeAttach::<ActiveProfile>::new("", KprobeType::Entry);
        assert!(matches!(result, Err(AttachError::InvalidTarget(_))));
    }
}

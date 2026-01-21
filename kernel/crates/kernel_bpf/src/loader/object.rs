//! Loaded BPF Object
//!
//! Represents a loaded BPF object file with programs and maps.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::bytecode::insn::BpfInsn;
use crate::bytecode::program::BpfProgType;
use crate::maps::MapDef;
use crate::profile::{ActiveProfile, PhysicalProfile};

/// A loaded BPF program.
#[derive(Debug, Clone)]
pub struct LoadedProgram<P: PhysicalProfile = ActiveProfile> {
    /// Program name (from section name)
    name: String,
    /// Program type
    prog_type: BpfProgType,
    /// Program instructions
    insns: Vec<BpfInsn>,
    /// Profile marker
    _profile: PhantomData<P>,
}

impl<P: PhysicalProfile> LoadedProgram<P> {
    /// Create a new loaded program.
    pub fn new(name: String, prog_type: BpfProgType, insns: Vec<BpfInsn>) -> Self {
        Self {
            name,
            prog_type,
            insns,
            _profile: PhantomData,
        }
    }

    /// Get the program name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the program type.
    pub fn prog_type(&self) -> BpfProgType {
        self.prog_type
    }

    /// Get the instructions.
    pub fn insns(&self) -> &[BpfInsn] {
        &self.insns
    }

    /// Get the instruction count.
    pub fn insn_count(&self) -> usize {
        self.insns.len()
    }

    /// Take ownership of the instructions.
    pub fn into_insns(self) -> Vec<BpfInsn> {
        self.insns
    }
}

/// A loaded map definition.
#[derive(Debug, Clone)]
pub struct LoadedMap {
    /// Map name
    pub name: String,
    /// Map definition
    pub def: MapDef,
}

impl LoadedMap {
    /// Get the map name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the map definition.
    pub fn def(&self) -> &MapDef {
        &self.def
    }
}

/// A loaded BPF object file.
#[derive(Debug)]
pub struct BpfObject<P: PhysicalProfile = ActiveProfile> {
    /// Loaded programs
    programs: Vec<LoadedProgram<P>>,
    /// Loaded maps
    maps: Vec<LoadedMap>,
    /// License string
    license: Option<String>,
}

impl<P: PhysicalProfile> BpfObject<P> {
    /// Create a new BPF object.
    pub fn new(
        programs: Vec<LoadedProgram<P>>,
        maps: Vec<LoadedMap>,
        license: Option<String>,
    ) -> Self {
        Self {
            programs,
            maps,
            license,
        }
    }

    /// Get all programs.
    pub fn programs(&self) -> &[LoadedProgram<P>] {
        &self.programs
    }

    /// Get a program by name.
    pub fn program(&self, name: &str) -> Option<&LoadedProgram<P>> {
        self.programs.iter().find(|p| p.name == name)
    }

    /// Get all maps.
    pub fn maps(&self) -> &[LoadedMap] {
        &self.maps
    }

    /// Get a map by name.
    pub fn map(&self, name: &str) -> Option<&LoadedMap> {
        self.maps.iter().find(|m| m.name == name)
    }

    /// Get the license string.
    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    /// Get number of programs.
    pub fn program_count(&self) -> usize {
        self.programs.len()
    }

    /// Get number of maps.
    pub fn map_count(&self) -> usize {
        self.maps.len()
    }

    /// Take ownership of a program by name.
    pub fn take_program(&mut self, name: &str) -> Option<LoadedProgram<P>> {
        if let Some(idx) = self.programs.iter().position(|p| p.name == name) {
            Some(self.programs.remove(idx))
        } else {
            None
        }
    }

    /// Iterator over program names.
    pub fn program_names(&self) -> impl Iterator<Item = &str> {
        self.programs.iter().map(|p| p.name.as_str())
    }

    /// Iterator over map names.
    pub fn map_names(&self) -> impl Iterator<Item = &str> {
        self.maps.iter().map(|m| m.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn loaded_program_basics() {
        let insns = vec![BpfInsn::mov64_imm(0, 42), BpfInsn::exit()];
        let prog = LoadedProgram::<ActiveProfile>::new(
            "test_prog".into(),
            BpfProgType::SocketFilter,
            insns.clone(),
        );

        assert_eq!(prog.name(), "test_prog");
        assert_eq!(prog.prog_type(), BpfProgType::SocketFilter);
        assert_eq!(prog.insn_count(), 2);
        assert_eq!(prog.insns(), &insns);
    }

    #[test]
    fn bpf_object_basics() {
        let programs = vec![LoadedProgram::<ActiveProfile>::new(
            "prog1".into(),
            BpfProgType::SocketFilter,
            vec![],
        )];

        let maps = vec![LoadedMap {
            name: "map1".into(),
            def: MapDef::new(crate::maps::MapType::Array, 4, 8, 100),
        }];

        let obj = BpfObject::new(programs, maps, Some("GPL".into()));

        assert_eq!(obj.program_count(), 1);
        assert_eq!(obj.map_count(), 1);
        assert_eq!(obj.license(), Some("GPL"));

        assert!(obj.program("prog1").is_some());
        assert!(obj.program("nonexistent").is_none());
        assert!(obj.map("map1").is_some());
    }
}

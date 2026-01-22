//! BPF Relocation Handler
//!
//! Handles relocations for map references and other symbols in BPF programs.

extern crate alloc;

use alloc::vec::Vec;

use super::elf::ElfParser;
use super::error::{LoadError, LoadResult};
use super::object::LoadedMap;
use crate::bytecode::insn::BpfInsn;

// BPF relocation types
const R_BPF_64_64: u32 = 1;
const R_BPF_64_ABS64: u32 = 2;
const R_BPF_64_ABS32: u32 = 3;
const R_BPF_64_32: u32 = 10;

/// BPF instruction relocation handler.
pub struct Relocator<'a> {
    /// Map definitions for resolving map references
    maps: &'a [LoadedMap],
}

impl<'a> Relocator<'a> {
    /// Create a new relocator.
    pub fn new(maps: &'a [LoadedMap]) -> Self {
        Self { maps }
    }

    /// Apply relocations to instructions.
    pub fn relocate(
        &mut self,
        section_name: &str,
        mut insns: Vec<BpfInsn>,
        parser: &ElfParser,
    ) -> LoadResult<Vec<BpfInsn>> {
        // Find the section index for this program
        let sections = parser.sections()?;
        let section_idx = sections
            .iter()
            .position(|s| {
                parser
                    .section_name(s)
                    .map(|n| n == section_name)
                    .unwrap_or(false)
            })
            .ok_or(LoadError::InvalidRelocation)?;

        // Get relocations for this section
        let relocs = parser.relocations(section_idx)?;
        if relocs.is_empty() {
            return Ok(insns);
        }

        // Get symbol table
        let symbols = parser.symbols()?;

        // Apply each relocation
        for reloc in relocs {
            let insn_idx = (reloc.offset / 8) as usize;
            if insn_idx >= insns.len() {
                return Err(LoadError::InvalidRelocation);
            }

            // Get symbol
            if reloc.sym_idx as usize >= symbols.len() {
                return Err(LoadError::UndefinedSymbol);
            }
            let sym = &symbols[reloc.sym_idx as usize];
            let sym_name = parser.symbol_name(sym)?;

            // Apply relocation based on type
            match reloc.rel_type {
                R_BPF_64_64 => {
                    // Map reference - 64-bit load immediate
                    self.relocate_map_ref(&mut insns, insn_idx, &sym_name)?;
                }
                R_BPF_64_32 => {
                    // Helper function call
                    self.relocate_call(&mut insns, insn_idx, &sym_name)?;
                }
                R_BPF_64_ABS64 | R_BPF_64_ABS32 => {
                    // Absolute references - typically for data
                    // These are handled differently based on context
                }
                _ => {
                    // Unknown relocation type - ignore for now
                }
            }
        }

        Ok(insns)
    }

    /// Relocate a map reference.
    fn relocate_map_ref(
        &self,
        insns: &mut [BpfInsn],
        insn_idx: usize,
        sym_name: &str,
    ) -> LoadResult<()> {
        // Find map by name
        let map_idx = self
            .maps
            .iter()
            .position(|m| m.name == sym_name)
            .ok_or(LoadError::UndefinedSymbol)?;

        // Update instruction with map index
        // BPF uses ld_imm64 for map references
        let insn = &mut insns[insn_idx];

        // src_reg = BPF_PSEUDO_MAP_FD (1) indicates map reference
        // imm contains the map index
        // regs format: dst (low 4 bits) | src (high 4 bits)
        insn.regs = (insn.regs & 0x0f) | (1 << 4); // Set src to BPF_PSEUDO_MAP_FD
        insn.imm = map_idx as i32;

        // If this is a wide instruction, update the second half too
        if insn_idx + 1 < insns.len() && insns[insn_idx].is_wide() {
            insns[insn_idx + 1].imm = 0;
        }

        Ok(())
    }

    /// Relocate a function call.
    fn relocate_call(
        &self,
        insns: &mut [BpfInsn],
        insn_idx: usize,
        sym_name: &str,
    ) -> LoadResult<()> {
        // Check if this is a helper function call
        if let Some(helper_id) = Self::helper_name_to_id(sym_name) {
            insns[insn_idx].imm = helper_id;
        }
        // Otherwise, it's a BPF-to-BPF call which needs different handling
        // (not implemented in streaming verifier)

        Ok(())
    }

    /// Convert helper function name to ID.
    fn helper_name_to_id(name: &str) -> Option<i32> {
        // Common BPF helper functions
        match name {
            "bpf_map_lookup_elem" => Some(1),
            "bpf_map_update_elem" => Some(2),
            "bpf_map_delete_elem" => Some(3),
            "bpf_probe_read" => Some(4),
            "bpf_ktime_get_ns" => Some(5),
            "bpf_trace_printk" => Some(6),
            "bpf_get_prandom_u32" => Some(7),
            "bpf_get_smp_processor_id" => Some(8),
            "bpf_skb_store_bytes" => Some(9),
            "bpf_l3_csum_replace" => Some(10),
            "bpf_l4_csum_replace" => Some(11),
            "bpf_tail_call" => Some(12),
            "bpf_clone_redirect" => Some(13),
            "bpf_get_current_pid_tgid" => Some(14),
            "bpf_get_current_uid_gid" => Some(15),
            "bpf_get_current_comm" => Some(16),
            "bpf_get_cgroup_classid" => Some(17),
            "bpf_skb_vlan_push" => Some(18),
            "bpf_skb_vlan_pop" => Some(19),
            "bpf_skb_get_tunnel_key" => Some(20),
            "bpf_skb_set_tunnel_key" => Some(21),
            "bpf_perf_event_read" => Some(22),
            "bpf_redirect" => Some(23),
            "bpf_get_route_realm" => Some(24),
            "bpf_perf_event_output" => Some(25),
            "bpf_skb_load_bytes" => Some(26),
            "bpf_get_stackid" => Some(27),
            "bpf_csum_diff" => Some(28),
            "bpf_skb_get_tunnel_opt" => Some(29),
            "bpf_skb_set_tunnel_opt" => Some(30),
            "bpf_skb_change_proto" => Some(31),
            "bpf_skb_change_type" => Some(32),
            "bpf_skb_under_cgroup" => Some(33),
            "bpf_get_hash_recalc" => Some(34),
            "bpf_get_current_task" => Some(35),
            "bpf_probe_write_user" => Some(36),
            "bpf_current_task_under_cgroup" => Some(37),
            "bpf_skb_change_tail" => Some(38),
            "bpf_skb_pull_data" => Some(39),
            "bpf_csum_update" => Some(40),
            "bpf_set_hash_invalid" => Some(41),
            "bpf_get_numa_node_id" => Some(42),
            "bpf_skb_change_head" => Some(43),
            "bpf_xdp_adjust_head" => Some(44),
            "bpf_probe_read_str" => Some(45),
            "bpf_get_socket_cookie" => Some(46),
            "bpf_get_socket_uid" => Some(47),
            "bpf_set_hash" => Some(48),
            "bpf_setsockopt" => Some(49),
            "bpf_skb_adjust_room" => Some(50),
            // Ring buffer helpers
            "bpf_ringbuf_output" => Some(130),
            "bpf_ringbuf_reserve" => Some(131),
            "bpf_ringbuf_submit" => Some(132),
            "bpf_ringbuf_discard" => Some(133),
            "bpf_ringbuf_query" => Some(134),
            // rkBPF robotics-specific helpers
            "bpf_motor_emergency_stop" => Some(200),
            "bpf_timeseries_push" => Some(201),
            "bpf_sensor_last_timestamp" => Some(202),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_name_mapping() {
        assert_eq!(Relocator::helper_name_to_id("bpf_map_lookup_elem"), Some(1));
        assert_eq!(Relocator::helper_name_to_id("bpf_ktime_get_ns"), Some(5));
        assert_eq!(
            Relocator::helper_name_to_id("bpf_ringbuf_output"),
            Some(130)
        );
        assert_eq!(
            Relocator::helper_name_to_id("bpf_motor_emergency_stop"),
            Some(200)
        );
        assert_eq!(Relocator::helper_name_to_id("unknown_helper"), None);
    }
}

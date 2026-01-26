use alloc::vec::Vec;
use kernel_abi::{SYS_BPF, BPF_PROG_LOAD, BPF_MAP_CREATE, BpfAttr};
use crate::BPF_MANAGER;
use kernel_bpf::bytecode::insn::BpfInsn;

pub fn sys_bpf(cmd: usize, attr_ptr: usize, _size: usize) -> isize {
    // Basic permissions check would go here
    
    let cmd_u32 = cmd as u32;
    
    match cmd_u32 {
        BPF_PROG_LOAD => {
            log::info!("sys_bpf: PROG_LOAD");
            
            // Safety: We assume the pointer is valid for this MVP.
            // In a real kernel we would use copy_from_user and validate ranges.
            if attr_ptr == 0 {
                return -1; // EFAULT
            }
            
            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };
            
            let insn_cnt = attr.insn_cnt as usize;
            let insns_ptr = attr.insns as *const BpfInsn;
            
            if insns_ptr.is_null() || insn_cnt == 0 || insn_cnt > 4096 {
                log::error!("sys_bpf: invalid instructions (ptr={:p}, cnt={})", insns_ptr, insn_cnt);
                return -1; // EINVAL
            }
            
            log::info!("sys_bpf: loading {} instructions", insn_cnt);
            
            // Copy instructions from userspace
            let mut insns = Vec::with_capacity(insn_cnt);
            for i in 0..insn_cnt {
                unsafe {
                    insns.push(*insns_ptr.add(i));
                }
            }
            
            // Load into manager
            if let Some(manager) = BPF_MANAGER.get() {
                 match manager.lock().load_raw_program(insns) {
                     Ok(id) => {
                         log::info!("sys_bpf: program loaded with id {}", id);
                         id as isize
                     },
                     Err(e) => {
                         log::error!("sys_bpf: failed to load program: {}", e);
                         -1
                     }
                 }
            } else {
                log::error!("sys_bpf: BPF_MANAGER not initialized");
                -1
            }
        }
        BPF_MAP_CREATE => {
            log::info!("sys_bpf: MAP_CREATE");
            -1
        }
        _ => {
            log::warn!("sys_bpf: Unknown command {}", cmd);
            -1
        }
    }
}

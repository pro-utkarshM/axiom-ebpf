use alloc::vec::Vec;

use kernel_abi::{
    BPF_MAP_CREATE, BPF_MAP_DELETE_ELEM, BPF_MAP_LOOKUP_ELEM, BPF_MAP_UPDATE_ELEM, BPF_PROG_ATTACH,
    BPF_PROG_LOAD, BpfAttr,
};
use kernel_bpf::bytecode::insn::BpfInsn;

use crate::BPF_MANAGER;

pub fn sys_bpf(cmd: usize, attr_ptr: usize, _size: usize) -> isize {
    // Basic permissions check would go here

    let cmd_u32 = cmd as u32;

    match cmd_u32 {
        BPF_MAP_CREATE => {
            log::info!("sys_bpf: MAP_CREATE");
            if attr_ptr == 0 {
                return -1; // EFAULT
            }
            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };

            // For MAP_CREATE, fields are:
            // prog_type -> map_type
            // insn_cnt -> key_size
            // insns (low u32) -> value_size
            // insns (high u32) -> max_entries
            let map_type = attr.prog_type;
            let key_size = attr.insn_cnt;
            let value_size = (attr.insns & 0xFFFFFFFF) as u32;
            let max_entries = ((attr.insns >> 32) & 0xFFFFFFFF) as u32;

            if let Some(manager) = BPF_MANAGER.get() {
                match manager
                    .lock()
                    .create_map(map_type, key_size, value_size, max_entries)
                {
                    Ok(map_id) => map_id as isize,
                    Err(e) => {
                        log::error!("sys_bpf: MAP_CREATE failed: {}", e);
                        -1
                    }
                }
            } else {
                -1
            }
        }
        BPF_MAP_LOOKUP_ELEM => {
            log::debug!("sys_bpf: MAP_LOOKUP_ELEM");
            if attr_ptr == 0 {
                return -1;
            }
            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };

            let map_id = attr.map_fd;
            let key_ptr = attr.key as *const u8;
            let value_ptr = attr.value as *mut u8;

            if key_ptr.is_null() || value_ptr.is_null() {
                return -1;
            }

            if let Some(manager) = BPF_MANAGER.get() {
                let mgr = manager.lock();
                // Get key size from map (for now, assume 4 bytes)
                let key_size = 4usize; // TODO: get from map def
                let key = unsafe { core::slice::from_raw_parts(key_ptr, key_size) };

                if let Some(value) = mgr.map_lookup(map_id, key) {
                    // Copy value to user buffer
                    unsafe {
                        core::ptr::copy_nonoverlapping(value.as_ptr(), value_ptr, value.len());
                    }
                    0
                } else {
                    -2 // ENOENT
                }
            } else {
                -1
            }
        }
        BPF_MAP_UPDATE_ELEM => {
            log::debug!("sys_bpf: MAP_UPDATE_ELEM");
            if attr_ptr == 0 {
                return -1;
            }
            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };

            let map_id = attr.map_fd;
            let key_ptr = attr.key as *const u8;
            let value_ptr = attr.value as *const u8;
            let flags = attr.flags;

            if key_ptr.is_null() || value_ptr.is_null() {
                return -1;
            }

            if let Some(manager) = BPF_MANAGER.get() {
                let mgr = manager.lock();
                // For now, assume fixed sizes (TODO: get from map def)
                let key_size = 4usize;
                let value_size = 8usize;
                let key = unsafe { core::slice::from_raw_parts(key_ptr, key_size) };
                let value = unsafe { core::slice::from_raw_parts(value_ptr, value_size) };

                match mgr.map_update(map_id, key, value, flags) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("sys_bpf: MAP_UPDATE failed: {}", e);
                        -1
                    }
                }
            } else {
                -1
            }
        }
        BPF_MAP_DELETE_ELEM => {
            log::debug!("sys_bpf: MAP_DELETE_ELEM");
            if attr_ptr == 0 {
                return -1;
            }
            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };

            let map_id = attr.map_fd;
            let key_ptr = attr.key as *const u8;

            if key_ptr.is_null() {
                return -1;
            }

            if let Some(manager) = BPF_MANAGER.get() {
                let mgr = manager.lock();
                let key_size = 4usize;
                let key = unsafe { core::slice::from_raw_parts(key_ptr, key_size) };

                match mgr.map_delete(map_id, key) {
                    Ok(_) => 0,
                    Err(_) => -2, // ENOENT
                }
            } else {
                -1
            }
        }
        BPF_PROG_ATTACH => {
            log::info!("sys_bpf: PROG_ATTACH");
            if attr_ptr == 0 {
                return -1;
            }
            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };

            let attach_type = attr.attach_btf_id;
            let prog_id = attr.attach_prog_fd;

            if let Some(manager) = BPF_MANAGER.get() {
                match manager.lock().attach(attach_type, prog_id) {
                    Ok(_) => {
                        log::info!("sys_bpf: attached prog {} to type {}", prog_id, attach_type);
                        0
                    }
                    Err(e) => {
                        log::error!("sys_bpf: attach failed: {}", e);
                        -1
                    }
                }
            } else {
                -1
            }
        }
        BPF_PROG_LOAD => {
            log::info!("sys_bpf: PROG_LOAD");

            if attr_ptr == 0 {
                return -1;
            }

            let attr = unsafe { &*(attr_ptr as *const BpfAttr) };

            let insn_cnt = attr.insn_cnt as usize;
            let insns_ptr = attr.insns as *const BpfInsn;

            if insns_ptr.is_null() || insn_cnt == 0 || insn_cnt > 4096 {
                log::error!(
                    "sys_bpf: invalid instructions (ptr={:p}, cnt={})",
                    insns_ptr,
                    insn_cnt
                );
                return -1;
            }

            log::info!("sys_bpf: loading {} instructions", insn_cnt);

            let mut insns = Vec::with_capacity(insn_cnt);
            for i in 0..insn_cnt {
                unsafe {
                    insns.push(*insns_ptr.add(i));
                }
            }

            if let Some(manager) = BPF_MANAGER.get() {
                match manager.lock().load_raw_program(insns) {
                    Ok(id) => {
                        log::info!("sys_bpf: program loaded with id {}", id);
                        id as isize
                    }
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
        _ => {
            log::warn!("sys_bpf: Unknown command {}", cmd);
            -1
        }
    }
}

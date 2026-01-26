use bitflags::bitflags;

bitflags! {
    pub struct BpfMapTags: u32 {
        const UNSPEC       = 0;
        const HASH         = 1;
        const ARRAY        = 2;
        const PROG_ARRAY   = 3;
        const PERF_EVENT_ARRAY = 4;
        const PER_CPU_HASH = 5;
        const PER_CPU_ARRAY = 6;
        const STACK_TRACE  = 7;
        const CGROUP_ARRAY = 8;
        const LRU_HASH     = 9;
        const LRU_PER_CPU_HASH = 10;
        const LPM_TRIE     = 11;
        const ARRAY_OF_MAPS = 12;
        const HASH_OF_MAPS = 13;
        const DEVMAP       = 14;
        const SOCKMAP      = 15;
        const CPUMAP       = 16;
        const XSKMAP       = 17;
        const SOCKHASH     = 18;
        const CGROUP_STORAGE = 19;
        const REUSEPORT_SOCKARRAY = 20;
        const PERCPU_CGROUP_STORAGE = 21;
        const QUEUE        = 22;
        const STACK        = 23;
        const SK_STORAGE   = 24;
        const DEVMAP_HASH  = 25;
        const STRUCT_OPS   = 26;
        const RINGBUF      = 27;
        const INODE_STORAGE = 28;
    }
}

pub const BPF_MAP_CREATE: u32 = 0;
pub const BPF_MAP_LOOKUP_ELEM: u32 = 1;
pub const BPF_MAP_UPDATE_ELEM: u32 = 2;
pub const BPF_MAP_DELETE_ELEM: u32 = 3;
pub const BPF_MAP_GET_NEXT_KEY: u32 = 4;
pub const BPF_PROG_LOAD: u32 = 5;
pub const BPF_OBJ_PIN: u32 = 6;
pub const BPF_OBJ_GET: u32 = 7;
pub const BPF_PROG_ATTACH: u32 = 8;
pub const BPF_PROG_DETACH: u32 = 9;
pub const BPF_PROG_TEST_RUN: u32 = 10;
pub const BPF_PROG_GET_NEXT_ID: u32 = 11;
pub const BPF_MAP_GET_NEXT_ID: u32 = 12;
pub const BPF_PROG_GET_FD_BY_ID: u32 = 13;
pub const BPF_MAP_GET_FD_BY_ID: u32 = 14;
pub const BPF_OBJ_GET_INFO_BY_FD: u32 = 15;
pub const BPF_PROG_QUERY: u32 = 16;
pub const BPF_RAW_TRACEPOINT_OPEN: u32 = 17;
pub const BPF_BTF_LOAD: u32 = 18;
pub const BPF_BTF_GET_FD_BY_ID: u32 = 19;
pub const BPF_TASK_FD_QUERY: u32 = 20;
pub const BPF_MAP_LOOKUP_AND_DELETE_ELEM: u32 = 21;
pub const BPF_MAP_FREEZE: u32 = 22;
pub const BPF_BTF_GET_NEXT_ID: u32 = 23;
pub const BPF_MAP_LOOKUP_BATCH: u32 = 24;
pub const BPF_MAP_LOOKUP_AND_DELETE_BATCH: u32 = 25;
pub const BPF_MAP_UPDATE_BATCH: u32 = 26;
pub const BPF_MAP_DELETE_BATCH: u32 = 27;
pub const BPF_LINK_CREATE: u32 = 28;
pub const BPF_LINK_UPDATE: u32 = 29;
pub const BPF_LINK_GET_FD_BY_ID: u32 = 30;
pub const BPF_LINK_GET_NEXT_ID: u32 = 31;
pub const BPF_ENABLE_STATS: u32 = 32;
pub const BPF_ITER_CREATE: u32 = 33;
pub const BPF_LINK_DETACH: u32 = 34;
pub const BPF_PROG_BIND_MAP: u32 = 35;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BpfAttr {
    // This is a simplified union/struct for BPF syscall attributes.
    // In Linux it is a huge union. We represent common fields.
    pub test_run_id: u32,
    pub flags: u32,
    pub data_in: u64,
    pub data_out: u64,
    pub data_size_in: u32,
    pub data_size_out: u32,
    // Add more fields as needed for PROG_LOAD etc.
}

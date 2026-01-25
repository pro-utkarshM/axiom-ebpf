# Axiom Implementation Roadmap

## Current Priority: BPF Integration

The kernel boots. The BPF subsystem is complete. Now we connect them.

```
BEFORE (current state)
──────────────────────
Kernel runs → init prints "hello" → done

AFTER (goal)
────────────
Kernel runs → BPF manager initialized → programs loadable
           → attach to timer/syscalls/GPIO → BPF executes on events
           → userspace can load programs via syscall
```

---

## Phase 3: BPF Integration

### Step 1: BPF Manager

**Location:** `kernel/src/bpf/mod.rs` (new)

**Goal:** Kernel component that manages BPF programs.

```rust
// kernel/src/bpf/mod.rs

use kernel_bpf::{
    verifier::Verifier,
    execution::Interpreter,
    maps::{ArrayMap, HashMap, RingBufMap},
    loader::ElfLoader,
};

pub struct BpfManager {
    /// Loaded programs (verified, ready to execute)
    programs: Vec<LoadedProgram>,

    /// Active maps
    maps: Vec<Box<dyn BpfMap>>,

    /// Attached programs (program_id → attach_point)
    attachments: Vec<Attachment>,
}

pub struct LoadedProgram {
    pub id: u32,
    pub name: String,
    pub instructions: Vec<BpfInsn>,
    pub verified: bool,
}

pub struct Attachment {
    pub program_id: u32,
    pub attach_type: AttachType,
    pub enabled: bool,
}

impl BpfManager {
    pub fn new() -> Self { ... }

    /// Load program from ELF bytes, verify, store
    pub fn load_program(&mut self, elf_bytes: &[u8]) -> Result<u32, BpfError> {
        let program = ElfLoader::load(elf_bytes)?;
        Verifier::verify(&program)?;
        let id = self.programs.len() as u32;
        self.programs.push(LoadedProgram {
            id,
            name: program.name,
            instructions: program.instructions,
            verified: true,
        });
        Ok(id)
    }

    /// Attach program to event source
    pub fn attach(&mut self, program_id: u32, attach_type: AttachType) -> Result<(), BpfError> {
        // Validate program exists
        // Register with appropriate subsystem
        // Store attachment
    }

    /// Execute program with context
    pub fn execute(&self, program_id: u32, ctx: &BpfContext) -> Result<u64, BpfError> {
        let program = self.programs.get(program_id as usize)?;
        Interpreter::run(&program.instructions, ctx)
    }
}
```

**Integration point:** Initialize in `kernel/src/lib.rs`:

```rust
// In kernel::init()
pub fn init(boot_info: &BootInfo) {
    // ... existing init ...

    // Initialize BPF subsystem
    log::info!("Initializing BPF subsystem");
    let bpf_manager = BpfManager::new();
    BPF_MANAGER.call_once(|| Mutex::new(bpf_manager));

    // ... continue to load init ...
}

static BPF_MANAGER: OnceCell<Mutex<BpfManager>> = OnceCell::new();
```

---

### Step 2: bpf() Syscall

**Location:** `kernel/crates/kernel_abi/src/syscall.rs`

Add syscall number:
```rust
pub const SYS_BPF: usize = 50;
```

**Location:** `kernel/src/syscall/bpf.rs` (new)

```rust
// BPF syscall commands
pub const BPF_PROG_LOAD: u32 = 0;
pub const BPF_MAP_CREATE: u32 = 1;
pub const BPF_PROG_ATTACH: u32 = 2;
pub const BPF_PROG_DETACH: u32 = 3;
pub const BPF_MAP_LOOKUP_ELEM: u32 = 4;
pub const BPF_MAP_UPDATE_ELEM: u32 = 5;
pub const BPF_MAP_DELETE_ELEM: u32 = 6;

#[repr(C)]
pub struct BpfAttrProgLoad {
    pub prog_type: u32,
    pub insn_cnt: u32,
    pub insns: *const u8,
    pub license: *const u8,
}

#[repr(C)]
pub struct BpfAttrMapCreate {
    pub map_type: u32,
    pub key_size: u32,
    pub value_size: u32,
    pub max_entries: u32,
}

pub fn sys_bpf(cmd: u32, attr: *const u8, size: usize) -> Result<i64, Errno> {
    let bpf_manager = BPF_MANAGER.get().ok_or(Errno::ENODEV)?;
    let mut manager = bpf_manager.lock();

    match cmd {
        BPF_PROG_LOAD => {
            // Copy attr from userspace
            // Load and verify program
            // Return program ID
        }
        BPF_MAP_CREATE => {
            // Create map
            // Return map ID
        }
        BPF_PROG_ATTACH => {
            // Attach program to event source
        }
        _ => Err(Errno::EINVAL),
    }
}
```

**Wire into syscall dispatch:** `kernel/src/syscall/mod.rs`

```rust
match syscall_num {
    // ... existing syscalls ...
    SYS_BPF => sys_bpf(arg0 as u32, arg1 as *const u8, arg2),
    _ => Err(Errno::ENOSYS),
}
```

---

### Step 3: Timer Attach Point

**Goal:** Execute BPF program on every timer tick.

**Location:** Modify existing timer interrupt handler.

For x86_64 (`kernel/src/arch/x86_64.rs` or timer code):

```rust
// In timer interrupt handler
fn timer_interrupt_handler() {
    // Existing tick handling...

    // Execute attached BPF programs
    if let Some(bpf_manager) = BPF_MANAGER.get() {
        let manager = bpf_manager.lock();
        for attachment in manager.get_timer_attachments() {
            let ctx = BpfContext {
                timestamp: get_kernel_time_ns(),
                // ... other context
            };
            let _ = manager.execute(attachment.program_id, &ctx);
        }
    }
}
```

For AArch64 (similar pattern in ARM timer handler).

---

### Step 4: Syscall Tracing Attach Point

**Goal:** Execute BPF program on syscall entry/exit.

**Location:** `kernel/src/syscall/mod.rs`

```rust
fn handle_syscall(num: usize, args: [usize; 6]) -> Result<i64, Errno> {
    // BPF: syscall entry
    run_bpf_syscall_enter(num, &args);

    // Dispatch to actual handler
    let result = match num {
        SYS_EXIT => sys_exit(args[0] as i32),
        // ... etc
    };

    // BPF: syscall exit
    run_bpf_syscall_exit(num, &result);

    result
}

fn run_bpf_syscall_enter(syscall_num: usize, args: &[usize; 6]) {
    if let Some(bpf_manager) = BPF_MANAGER.get() {
        let manager = bpf_manager.lock();
        for attachment in manager.get_syscall_enter_attachments() {
            let ctx = SyscallContext {
                syscall_num: syscall_num as u64,
                arg0: args[0] as u64,
                arg1: args[1] as u64,
                // ...
            };
            let _ = manager.execute(attachment.program_id, &ctx.as_bpf_context());
        }
    }
}
```

---

### Step 5: Helper Implementation

**Location:** `kernel/src/bpf/helpers.rs` (new)

```rust
/// Get current kernel time in nanoseconds
pub fn bpf_ktime_get_ns() -> u64 {
    // Read from HPET or ARM timer
    crate::time::get_kernel_time_ns()
}

/// Look up element in map
pub fn bpf_map_lookup_elem(map_id: u32, key: *const u8) -> *const u8 {
    if let Some(bpf_manager) = BPF_MANAGER.get() {
        let manager = bpf_manager.lock();
        if let Some(map) = manager.maps.get(map_id as usize) {
            return map.lookup(key);
        }
    }
    core::ptr::null()
}

/// Output to ring buffer
pub fn bpf_ringbuf_output(map_id: u32, data: *const u8, size: u32) -> i32 {
    if let Some(bpf_manager) = BPF_MANAGER.get() {
        let manager = bpf_manager.lock();
        if let Some(map) = manager.maps.get(map_id as usize) {
            if let Some(ringbuf) = map.as_ringbuf() {
                return ringbuf.output(data, size);
            }
        }
    }
    -1
}

/// Print to serial console (debug)
pub fn bpf_trace_printk(fmt: *const u8, _fmt_size: u32) -> i32 {
    // Safety: validate fmt is in valid memory
    let s = unsafe { core::ffi::CStr::from_ptr(fmt as *const i8) };
    if let Ok(msg) = s.to_str() {
        log::info!("[BPF] {}", msg);
        return 0;
    }
    -1
}
```

**Wire into interpreter:** Modify `kernel_bpf/src/execution/interpreter.rs` to call these helpers:

```rust
fn dispatch_helper(helper_id: u32, args: [u64; 5]) -> u64 {
    match helper_id {
        1 => bpf_ktime_get_ns(),
        2 => bpf_map_lookup_elem(args[0] as u32, args[1] as *const u8) as u64,
        3 => bpf_map_update_elem(...) as u64,
        6 => bpf_ringbuf_output(args[0] as u32, args[1] as *const u8, args[2] as u32) as u64,
        7 => bpf_trace_printk(args[0] as *const u8, args[1] as u32) as u64,
        _ => 0,
    }
}
```

---

### Step 6: Userspace BPF Loader

**Location:** `userspace/minilib/src/lib.rs`

Add bpf() syscall wrapper:

```rust
pub fn bpf(cmd: u32, attr: *const u8, size: usize) -> i64 {
    syscall3(SYS_BPF, cmd as usize, attr as usize, size)
}
```

**Location:** `userspace/bpf_loader/` (new, simple test program)

```rust
#![no_std]
#![no_main]

use minilib::{bpf, write, exit};

// Minimal BPF program bytecode (hardcoded for testing)
// This program just returns 0
static PROG: [u8; 16] = [
    0xb7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov r0, 0
    0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // exit
];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let attr = BpfAttrProgLoad {
        prog_type: 0,
        insn_cnt: 2,
        insns: PROG.as_ptr(),
        license: b"GPL\0".as_ptr(),
    };

    let result = bpf(BPF_PROG_LOAD, &attr as *const _ as *const u8, core::mem::size_of_val(&attr));

    if result >= 0 {
        write(1, b"BPF program loaded!\n");
    } else {
        write(1, b"BPF load failed\n");
    }

    exit(0);
}
```

---

## Phase 4: Hardware Attach (RPi5)

### GPIO Driver

**Location:** `kernel/src/driver/gpio/` (new)

```rust
pub struct Rpi5Gpio {
    base: usize, // MMIO base address
}

impl Rpi5Gpio {
    pub fn new(dtb: &DeviceTree) -> Self {
        // Parse GPIO base from DTB
    }

    pub fn configure_input(&self, pin: u32) {
        // Set pin as input
    }

    pub fn configure_output(&self, pin: u32) {
        // Set pin as output
    }

    pub fn set(&self, pin: u32, value: bool) {
        // Write to pin
    }

    pub fn get(&self, pin: u32) -> bool {
        // Read pin
    }

    pub fn enable_edge_interrupt(&self, pin: u32, edge: Edge) {
        // Configure interrupt on edge
    }
}
```

### GPIO Interrupt Handler

```rust
fn gpio_interrupt_handler(pin: u32, edge: Edge) {
    // Execute attached BPF programs
    if let Some(bpf_manager) = BPF_MANAGER.get() {
        let manager = bpf_manager.lock();
        for attachment in manager.get_gpio_attachments(pin) {
            let ctx = GpioContext {
                pin,
                edge,
                timestamp: get_kernel_time_ns(),
            };
            let _ = manager.execute(attachment.program_id, &ctx.as_bpf_context());
        }
    }
}
```

### GPIO Helper

```rust
/// Set GPIO pin value from BPF program
pub fn bpf_gpio_set(chip: u32, line: u32, value: u32) -> i32 {
    if let Some(gpio) = get_gpio_driver() {
        gpio.set(line, value != 0);
        return 0;
    }
    -1
}
```

---

## Testing Strategy

### Unit Tests

```bash
# Test BPF library in isolation
cargo test -p kernel_bpf

# Test with embedded profile
cargo test -p kernel_bpf --features embedded-profile

# Test with cloud profile
cargo test -p kernel_bpf --features cloud-profile
```

### QEMU Integration

```bash
# Build and run kernel
cargo run --release

# Expected output:
# [kernel] Booting Axiom...
# [kernel] Physical memory initialized
# [kernel] Virtual memory initialized
# [kernel] BPF subsystem initialized    <- NEW
# [kernel] Loading /bin/init
# [init] hello from init!
```

### BPF Smoke Test

Once syscall is implemented:

```bash
# Build kernel with bpf_loader in filesystem
cargo run --release

# Expected:
# [init] BPF program loaded!
# or
# [BPF] Hello from BPF!  (if using trace_printk)
```

### RPi5 Hardware Test

```bash
# Build for AArch64
cargo build --target aarch64-unknown-none --release

# Create SD card image
./scripts/make_rpi5_image.sh

# Boot on RPi5
# Connect button to GPIO pin
# Connect LED to another GPIO pin

# Expected:
# Button press → BPF executes → LED toggles
```

---

## Milestones

### Milestone 1: BPF Runs in Kernel (Week 1-2)
- [ ] BpfManager integrated into kernel
- [ ] Hardcoded BPF program executes during init
- [ ] Output visible on serial console

### Milestone 2: Syscall Works (Week 2-3)
- [ ] bpf() syscall implemented
- [ ] Can load program from userspace
- [ ] Program executes successfully

### Milestone 3: Attach Points Work (Week 3-4)
- [ ] Timer attach point working
- [ ] Syscall tracing working
- [ ] BPF runs on events

### Milestone 4: RPi5 Demo (Week 5-6)
- [ ] Kernel boots on RPi5
- [ ] GPIO driver working
- [ ] Button → BPF → LED demo

### Milestone 5: Full Demo (Week 7-8)
- [ ] Multiple example programs
- [ ] Safety interlock demo
- [ ] Performance benchmarks
- [ ] Video demo for proposal

---

## File Changes Summary

### New Files
```
kernel/src/bpf/
├── mod.rs          # BpfManager
├── helpers.rs      # Helper implementations
└── context.rs      # BPF context types

kernel/src/syscall/bpf.rs    # bpf() syscall handler

kernel/src/driver/gpio/
├── mod.rs          # GPIO abstraction
└── rpi5.rs         # RPi5 GPIO driver

userspace/bpf_loader/        # Test program
```

### Modified Files
```
kernel/src/lib.rs            # Add BPF init
kernel/src/syscall/mod.rs    # Add SYS_BPF dispatch
kernel/crates/kernel_abi/src/syscall.rs  # Add SYS_BPF constant
kernel/crates/kernel_bpf/src/execution/interpreter.rs  # Wire real helpers
```

---

## Dependencies

The kernel_bpf crate needs to be usable from kernel context:

```toml
# kernel/Cargo.toml
[dependencies]
kernel_bpf = { path = "crates/kernel_bpf", default-features = false, features = ["embedded-profile"] }
```

Ensure kernel_bpf is no_std compatible (it should already be).

# Axiom Task Tracking

## Implementation Status Overview

Axiom is a **complete operating system kernel** with BPF as a first-class primitive. The kernel boots on real hardware (x86_64, AArch64/RPi5, RISC-V). The BPF subsystem is fully implemented as a library. The next milestone is **integration** - wiring the BPF subsystem into the running kernel.

| Layer | Status | Description |
|-------|--------|-------------|
| Kernel Core | ✅ Complete | Boot, memory, processes, VFS, syscalls |
| BPF Subsystem | ✅ Complete | Verifier, interpreter, JIT, maps, signing |
| BPF Integration | ❌ Not Started | Connect BPF to kernel events |
| Hardware Attach | ❌ Not Started | GPIO, PWM, timer hooks on RPi5 |
| Example Programs | ❌ Not Started | Demo .bpf programs |

---

## Current Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        IMPLEMENTED                           │
│                                                              │
│  Kernel Core              BPF Subsystem (library)           │
│  ────────────             ───────────────────────           │
│  ✅ Limine boot           ✅ Streaming verifier             │
│  ✅ Physical memory       ✅ Interpreter                    │
│  ✅ Virtual memory        ✅ x86_64 JIT                     │
│  ✅ Process/tasks         ⚠️ ARM64 JIT (partial)            │
│  ✅ Scheduler             ✅ Maps (array, hash, ring, ts)   │
│  ✅ VFS + Ext2            ✅ ELF loader                     │
│  ✅ DevFS                 ✅ Ed25519 signing                │
│  ✅ Syscalls (8/41)       ✅ Attach abstractions            │
│  ✅ ELF loader                                              │
│                                                              │
│  Architectures            Userspace                         │
│  ─────────────            ─────────                         │
│  ✅ x86_64 (full)         ✅ init (minimal)                 │
│  ✅ AArch64 (full)        ✅ minilib (syscalls)             │
│  ✅ RPi5 platform         ✅ rk-cli (tooling)               │
│  ⚠️ RISC-V (boot)         ✅ rk-bridge (events)             │
└──────────────────────────────────────────────────────────────┘
                              │
                              │ GAP: Integration
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                      NOT YET CONNECTED                       │
│                                                              │
│  • No bpf() syscall                                         │
│  • BPF library not called from kernel                       │
│  • Attach points are abstractions, not hooked to hardware   │
│  • No way to load BPF programs at runtime                   │
└──────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Kernel Core ✅ COMPLETE

### Boot & Architecture
- [x] Limine bootloader integration
- [x] x86_64 boot sequence (GDT, IDT, ACPI, APIC, HPET)
- [x] AArch64 boot sequence (exception vectors, GIC, DTB)
- [x] AArch64 RPi5 platform support
- [x] RISC-V boot sequence (basic)

### Memory Management
- [x] Physical frame allocator (sparse regions, multi-size pages)
- [x] Virtual memory manager (address space tracking)
- [x] Kernel heap (linked list allocator)
- [x] Memory API traits (MemoryApi, Allocation)

### Process & Scheduling
- [x] Process abstraction (ProcessId, ProcessTree)
- [x] Task management (TaskId, state machine)
- [x] Context switching (CR3, register save/restore)
- [x] Round-robin scheduler
- [x] Per-CPU state

### Filesystem
- [x] VFS abstraction layer
- [x] Ext2 filesystem (read)
- [x] DevFS (/dev)
- [x] Path resolution

### Syscalls
- [x] Syscall dispatch (x86_64)
- [x] SYS_EXIT
- [x] SYS_READ
- [x] SYS_WRITE
- [x] SYS_OPEN
- [x] SYS_GETCWD
- [x] SYS_MMAP
- [x] SYS_FCNTL
- [x] SYS_STAT/FSTAT

### Drivers
- [x] Serial console (UART)
- [x] VirtIO block device
- [x] PCI enumeration

---

## Phase 2: BPF Subsystem ✅ COMPLETE (as library)

### Bytecode
- [x] Instruction encoding/decoding (`kernel_bpf/src/bytecode/`)
- [x] Opcode classes (ALU64, ALU32, JMP, LDX, STX, etc.)
- [x] Register file (R0-R10)
- [x] Program representation with profile constraints

### Verifier
- [x] Streaming verifier (`kernel_bpf/src/verifier/streaming.rs`)
- [x] O(registers × basic_block_depth) memory complexity
- [x] CFG analysis and reachability
- [x] Register type tracking (11 types)
- [x] Helper function validation
- [x] Profile-aware constraints

### Execution
- [x] Interpreter (`kernel_bpf/src/execution/interpreter.rs`)
  - All ALU operations
  - All jump conditions
  - Memory load/store
  - Helper dispatch
- [x] x86_64 JIT (`kernel_bpf/src/execution/jit/`)
  - Full instruction encoding
  - Register allocation
  - Prologue/epilogue
- [x] ARM64 JIT (`kernel_bpf/src/execution/jit_aarch64.rs`)
  - Structure complete
  - Register mapping
  - ⚠️ Instruction emission partial

### Maps
- [x] Array map - O(1) lookup
- [x] Hash map - linear probing
- [x] Ring buffer - lock-free SPMC
- [x] Time-series map - circular buffer
- [x] Static pool - embedded profile allocator

### Loader
- [x] ELF64 parser (no libbpf)
- [x] Section extraction
- [x] Relocation handling
- [x] Map definition parsing

### Signing
- [x] SHA3-256 hashing
- [x] Ed25519 signatures
- [x] Signed program format
- [x] TrustedKey management

### Attach Abstractions
- [x] AttachPoint trait
- [x] Kprobe abstraction
- [x] Tracepoint abstraction
- [x] GPIO abstraction
- [x] PWM abstraction
- [x] IIO abstraction
- ⚠️ All are framework only - not connected to kernel

### Scheduler
- [x] ThroughputPolicy (cloud)
- [x] DeadlinePolicy (embedded, EDF)
- [x] Program queue management

---

## Phase 3: BPF Integration ❌ CURRENT PRIORITY

### BPF Manager (kernel component)
- [ ] `BpfManager` struct in kernel
  - Holds loaded programs
  - Manages program lifecycle
  - Tracks attached programs
- [ ] Integration with kernel initialization

### bpf() Syscall
- [ ] Add SYS_BPF to kernel_abi
- [ ] Syscall handler in kernel_syscall
- [ ] Commands:
  - [ ] BPF_PROG_LOAD
  - [ ] BPF_MAP_CREATE
  - [ ] BPF_PROG_ATTACH
  - [ ] BPF_PROG_DETACH
  - [ ] BPF_MAP_LOOKUP
  - [ ] BPF_MAP_UPDATE

### Attach Point Implementation
- [ ] Timer interrupt hook
  - [ ] Hook into HPET/ARM timer
  - [ ] Execute BPF on each tick
  - [ ] Pass timer context to program
- [ ] Syscall tracing
  - [ ] Hook syscall entry/exit
  - [ ] Pass syscall args to BPF
- [ ] Function tracing (kprobe-like)
  - [ ] Hook arbitrary kernel functions
  - [ ] Requires some form of instrumentation

### Helper Implementation
- [ ] bpf_ktime_get_ns() - read kernel time
- [ ] bpf_map_lookup_elem() - map lookup
- [ ] bpf_map_update_elem() - map update
- [ ] bpf_map_delete_elem() - map delete
- [ ] bpf_ringbuf_output() - event output
- [ ] bpf_trace_printk() - debug output to serial

### Userspace Integration
- [ ] Update minilib with bpf() syscall wrapper
- [ ] Simple BPF loader program
- [ ] Test loading program from userspace

---

## Phase 4: Hardware Attach (RPi5) ❌ NOT STARTED

### GPIO
- [ ] RPi5 GPIO driver in kernel
- [ ] Edge detection interrupt handling
- [ ] GPIO attach point implementation
- [ ] BPF execution on GPIO event

### PWM
- [ ] RPi5 PWM driver in kernel
- [ ] PWM state change observation
- [ ] PWM attach point implementation
- [ ] BPF execution on PWM change

### Timer (high-resolution)
- [ ] ARM timer configuration
- [ ] Configurable tick rate
- [ ] BPF execution with timing data

### Demo: GPIO → BPF → LED
- [ ] Button press detected by GPIO interrupt
- [ ] BPF program executes
- [ ] BPF program toggles LED via helper
- [ ] End-to-end demo on real RPi5

---

## Phase 5: Validation & Demos ❌ NOT STARTED

### Example BPF Programs
- [ ] `hello.bpf.c` - minimal program, prints to serial
- [ ] `counter.bpf.c` - counts events using map
- [ ] `syscall_trace.bpf.c` - traces syscall entry/exit
- [ ] `gpio_toggle.bpf.c` - toggles LED on button press
- [ ] `safety_interlock.bpf.c` - emergency stop demo

### Performance Benchmarks
- [ ] Kernel memory footprint
- [ ] Boot time
- [ ] BPF verification time
- [ ] BPF execution overhead
- [ ] Interrupt latency

### Demo Scenarios
- [ ] **Runtime Behavior Change**: Load new scheduling policy live
- [ ] **Production Debugging**: Attach trace to running kernel
- [ ] **Safety Interlock**: Kernel-enforced emergency stop

### Documentation
- [ ] Getting started guide
- [ ] BPF program writing guide
- [ ] Architecture documentation
- [ ] API reference

---

## File Index

### Kernel Core
| Path | Description |
|------|-------------|
| `kernel/src/main.rs` | Entry points (x86_64, aarch64, riscv64) |
| `kernel/src/lib.rs` | Kernel initialization |
| `kernel/src/arch/` | Architecture-specific code |
| `kernel/src/mcore/` | Process, task, scheduler |
| `kernel/src/mem/` | Memory management glue |
| `kernel/src/file/` | VFS, Ext2, DevFS |
| `kernel/src/syscall/` | Syscall handlers |
| `kernel/src/driver/` | VirtIO, PCI |

### Kernel Crates
| Path | Description |
|------|-------------|
| `kernel/crates/kernel_bpf/` | BPF subsystem |
| `kernel/crates/kernel_abi/` | Syscall numbers, errno |
| `kernel/crates/kernel_physical_memory/` | Frame allocator |
| `kernel/crates/kernel_virtual_memory/` | Address space |
| `kernel/crates/kernel_vfs/` | VFS abstraction |
| `kernel/crates/kernel_syscall/` | Syscall utilities |
| `kernel/crates/kernel_elfloader/` | ELF loading |
| `kernel/crates/kernel_device/` | Device abstraction |
| `kernel/crates/kernel_devfs/` | DevFS implementation |
| `kernel/crates/kernel_pci/` | PCI enumeration |
| `kernel/crates/kernel_memapi/` | Memory API traits |

### BPF Subsystem
| Path | Description |
|------|-------------|
| `kernel/crates/kernel_bpf/src/verifier/` | Streaming verifier |
| `kernel/crates/kernel_bpf/src/execution/` | Interpreter + JIT |
| `kernel/crates/kernel_bpf/src/maps/` | Map implementations |
| `kernel/crates/kernel_bpf/src/loader/` | ELF loader |
| `kernel/crates/kernel_bpf/src/attach/` | Attach abstractions |
| `kernel/crates/kernel_bpf/src/signing/` | Cryptographic signing |
| `kernel/crates/kernel_bpf/src/profile/` | Cloud/embedded profiles |
| `kernel/crates/kernel_bpf/src/scheduler/` | BPF scheduler |

### Userspace
| Path | Description |
|------|-------------|
| `userspace/init/` | Root process |
| `userspace/minilib/` | Syscall wrappers |
| `userspace/rk_cli/` | Deployment CLI |
| `userspace/rk_bridge/` | Event consumer |

### Documentation
| Path | Description |
|------|-------------|
| `docs/proposal.md` | Full project proposal |
| `docs/tasks.md` | This file |
| `docs/implementation.md` | Implementation details |
| `docs/howto.md` | Usage guide |

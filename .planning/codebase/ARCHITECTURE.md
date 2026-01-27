# Architecture

**Analysis Date:** 2026-01-27

## Pattern Overview

**Overall:** Layered Monolithic Kernel with Modular Runtime Subsystems

**Key Characteristics:**
- Bare-metal operating system kernel for runtime programmability
- BPF as first-class kernel primitive (not bolted on)
- Multi-architecture support via trait abstraction
- Compile-time profile selection (cloud vs embedded)
- POSIX-like syscall interface

## Layers

```
┌─────────────────────────────────────────────────┐
│           ENTRY POINT (Bootloader)              │
│           kernel_main() for each arch           │
├─────────────────────────────────────────────────┤
│         USERSPACE INIT & PROGRAMS               │
├─────────────────────────────────────────────────┤
│              SYSCALL DISPATCHER                 │
│    dispatch_syscall() → routes to handlers      │
├─────────────────────────────────────────────────┤
│         KERNEL SERVICE LAYER                    │
│  BPF Manager  │ Process Manager  │ File System  │
│  Memory Mgmt  │ Device Drivers   │ VFS          │
├─────────────────────────────────────────────────┤
│          ARCHITECTURE-SPECIFIC LAYER            │
│  x86_64 (complete) │ AArch64 (complete) │ RISC-V│
├─────────────────────────────────────────────────┤
│          HARDWARE (QEMU/Real Hardware)          │
└─────────────────────────────────────────────────┘
```

**Hardware Abstraction** (`kernel/src/arch/`)
- Purpose: CPU-specific initialization, context switching, interrupts
- Contains: Per-architecture boot, paging, exception handlers
- Depends on: Hardware only
- Used by: All kernel layers

**Memory Management** (`kernel/src/mem/`)
- Purpose: Physical/virtual memory management, heap allocation
- Contains: `phys.rs`, `virt.rs`, `heap.rs`, `memapi.rs`
- Depends on: Architecture layer for paging
- Used by: Process management, file system

**Process Management** (`kernel/src/mcore/mtask/`)
- Purpose: Tasks, processes, scheduling
- Contains: Process abstraction, task state machine, scheduler
- Depends on: Memory management, architecture
- Used by: Syscall dispatcher

**File System** (`kernel/src/file/`)
- Purpose: VFS abstraction, Ext2, DevFS
- Contains: `ext2.rs`, `devfs.rs`, VFS adapter
- Depends on: Block device drivers
- Used by: Syscall handlers

**BPF Subsystem** (`kernel/crates/kernel_bpf/`)
- Purpose: Verified bytecode execution engine
- Contains: Verifier, interpreter, JIT, maps, loader
- Depends on: Memory management
- Used by: BPF syscall handler

**Driver Layer** (`kernel/src/driver/`)
- Purpose: VirtIO block/GPU, PCI, device management
- Contains: Block device abstraction, VirtIO implementation
- Depends on: Architecture I/O
- Used by: File system

## Data Flow

**Syscall Request Flow:**
```
Userspace Program
        ↓
   [Interrupt 0x80]
        ↓
   kernel/src/arch/idt.rs (syscall_handler)
        ↓
   kernel/src/syscall/mod.rs (dispatch_syscall)
        ↓
   Specific Syscall Handler:
   ├─ SYS_BPF → kernel/src/syscall/bpf.rs
   ├─ SYS_READ → kernel_syscall crate
   ├─ SYS_WRITE → kernel_syscall crate
   └─ SYS_MMAP → kernel_syscall crate
        ↓
   Return to userspace via interrupt frame
```

**BPF Program Lifecycle:**
```
1. LOAD (sys_bpf, BPF_PROG_LOAD)
   → kernel/src/syscall/bpf.rs: parse instructions
   → BpfManager::load_raw_program()
   → kernel_bpf verifier validates
   → Store in programs vec, return program ID

2. ATTACH (sys_bpf, BPF_PROG_ATTACH)
   → BpfManager::attach(attach_type, prog_id)
   → Map attach_type → program IDs

3. EXECUTE
   → Triggered by syscall entry (attach_type=2)
   → Or timer interrupt (attach_type=1)
   → BpfManager::execute_hooks()
   → Execute via Interpreter<ActiveProfile>
   → Return result
```

**State Management:**
- File-based: All persistent state in filesystem
- No persistent in-memory state across reboots
- Each syscall is independent

## Key Abstractions

**Architecture Trait** (`kernel/src/arch/traits.rs`)
- Purpose: Platform-independent kernel operations
- Examples: `Aarch64`, `x86_64` implementations
- Pattern: Trait abstraction for architecture-specific code

**BpfManager** (`kernel/src/bpf/mod.rs`)
- Purpose: Central hub for all BPF operations
- Examples: Program loading, map creation, hook execution
- Pattern: Global singleton via `OnceCell`

**Profile System** (`kernel/crates/kernel_bpf/src/profile/`)
- Purpose: Compile-time resource selection
- Examples: `CloudProfile`, `EmbeddedProfile`
- Pattern: Generic type parameter `P: PhysicalProfile`

**Execution Context** (`kernel/src/mcore/context.rs`)
- Purpose: Per-CPU execution state
- Examples: Current GDT/IDT, LAPIC, task pointer
- Pattern: Thread-local storage equivalent

**VFS** (`kernel/src/file/mod.rs`)
- Purpose: Virtual filesystem abstraction
- Examples: Mount points, file operations
- Pattern: Global `RwLock<Vfs>` singleton

## Entry Points

| Purpose | Path | Function |
|---------|------|----------|
| Build Orchestrator | `src/main.rs` | Runner, invokes cargo, boots QEMU |
| Kernel Entry (x86_64) | `kernel/src/main.rs` | `kernel_main()` |
| Kernel Entry (ARM64) | `kernel/src/main.rs` | `kernel_main()` |
| Kernel Init | `kernel/src/lib.rs` | `init()` |
| Syscall Dispatcher | `kernel/src/syscall/mod.rs` | `dispatch_syscall()` |
| BPF Syscall | `kernel/src/syscall/bpf.rs` | `sys_bpf()` |
| Interrupt Handlers | `kernel/src/arch/idt.rs` | `create_idt()` |
| Userspace Init | `userspace/init/src/main.rs` | `_start()` |

## Error Handling

**Strategy:** Throw errors, catch at boundaries

**Patterns:**
- Result types for fallible operations
- `expect()` for initialization failures (kernel panic acceptable)
- Error codes returned to userspace via syscall return value

## Cross-Cutting Concerns

**Logging:**
- `log` crate abstraction - `kernel/src/log.rs`
- Serial console output for debug

**Validation:**
- BPF verifier for bytecode safety - `kernel/crates/kernel_bpf/src/verifier/`
- Syscall boundary validation in handlers

**Authentication:**
- Not applicable (single-user bare-metal kernel)

---

*Architecture analysis: 2026-01-27*
*Update when major patterns change*

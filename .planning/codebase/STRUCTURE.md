# Codebase Structure

**Analysis Date:** 2026-01-27

## Directory Layout

```
axiom-ebpf/
├── src/                    # Build orchestrator (runner)
├── kernel/                 # Kernel build target
│   ├── src/               # Main kernel source
│   ├── crates/            # Subsystem crates (kernel_*)
│   ├── demos/             # Demo programs
│   └── platform/          # Platform configs (RPi5)
├── userspace/             # Userspace programs
├── examples/              # Example programs
├── docs/                  # Documentation
├── scripts/               # Build/deploy scripts
├── .planning/             # Project planning (GSD)
├── .github/               # CI/CD workflows
├── Cargo.toml             # Root workspace config
├── build.rs               # ISO/disk image creation
├── rust-toolchain.toml    # Rust version config
└── limine.conf            # Bootloader config
```

## Directory Purposes

**src/**
- Purpose: Build orchestrator that invokes cargo and boots QEMU
- Contains: `main.rs` (CLI runner), `lib.rs` (kernel library)
- Key files: `main.rs` - Entry point for `cargo run`
- Subdirectories: None

**kernel/src/**
- Purpose: Main kernel source code
- Contains: Architecture, memory, process, filesystem, drivers, BPF integration
- Key files: `main.rs` (kernel entry), `lib.rs` (init)
- Subdirectories:
  - `arch/` - Architecture-specific code (x86_64, aarch64, riscv64)
  - `mem/` - Memory management
  - `mcore/` - Process/scheduler (mtask/)
  - `file/` - Filesystem (ext2, devfs)
  - `bpf/` - BPF manager (kernel-side)
  - `syscall/` - Syscall dispatcher
  - `driver/` - Device drivers (virtio, pci)

**kernel/crates/**
- Purpose: Testable subsystem crates
- Contains: 11 crates with `kernel_` prefix
- Key crates:
  - `kernel_bpf/` - BPF subsystem (verifier, execution, maps)
  - `kernel_abi/` - Syscall ABI definitions
  - `kernel_vfs/` - VFS abstraction
  - `kernel_syscall/` - Syscall implementations
  - `kernel_elfloader/` - ELF binary loader

**kernel/crates/kernel_bpf/**
- Purpose: Core BPF subsystem (self-contained)
- Contains: Bytecode, verifier, execution, maps, loader, scheduler
- Key files:
  - `src/lib.rs` - Module root, profile config
  - `src/bytecode/` - BPF instruction set
  - `src/verifier/` - Static safety verification
  - `src/execution/` - Interpreter and JIT
  - `src/maps/` - Map implementations
  - `benches/` - Criterion benchmarks

**userspace/**
- Purpose: Userspace programs
- Contains: Init process, CLI tools, syscall wrappers
- Key subdirectories:
  - `init/` - PID 1 root process
  - `rk_cli/` - BPF deployment CLI
  - `rk_bridge/` - Ring buffer to ROS2 bridge
  - `minilib/` - Syscall wrappers
  - `bpf_loader/` - BPF program loader

**examples/**
- Purpose: Example programs
- Contains: BPF example programs
- Key files: `bpf/hello.bpf.c`, `bpf/README.md`

**docs/**
- Purpose: Project documentation
- Contains: Proposal, implementation, platform docs
- Key files: `proposal.md`, `implementation.md`, `tasks.md`, `howto.md`

**.github/**
- Purpose: CI/CD configuration
- Contains: Workflows, Claude guidance
- Key files: `workflows/build.yml`, `workflows/bpf-profiles.yml`, `CLAUDE.md`

## Key File Locations

**Entry Points:**
- `src/main.rs` - Build orchestrator entry
- `kernel/src/main.rs` - Kernel entry (`kernel_main`)
- `userspace/init/src/main.rs` - Init process entry

**Configuration:**
- `Cargo.toml` - Workspace configuration
- `kernel/Cargo.toml` - Kernel dependencies
- `rust-toolchain.toml` - Rust nightly + components
- `limine.conf` - Bootloader configuration
- `.rustfmt.toml` - Code formatting

**Core Logic:**
- `kernel/src/syscall/mod.rs` - Syscall dispatcher
- `kernel/src/syscall/bpf.rs` - BPF syscall handler
- `kernel/src/bpf/mod.rs` - BPF manager
- `kernel/crates/kernel_bpf/src/verifier/` - BPF verifier
- `kernel/crates/kernel_bpf/src/execution/` - BPF execution

**Testing:**
- `kernel/crates/*/src/*.rs` - Unit tests in `#[cfg(test)]` modules
- `kernel/crates/kernel_bpf/benches/` - Criterion benchmarks
- `kernel/crates/kernel_bpf/tests/` - Integration tests

**Documentation:**
- `README.md` - Project overview
- `docs/proposal.md` - Vision and roadmap
- `.github/CLAUDE.md` - Claude Code guidance

## Naming Conventions

**Files:**
- `snake_case.rs` - Rust source files
- `mod.rs` - Module entry points
- `lib.rs` - Crate roots
- `main.rs` - Binary entry points

**Directories:**
- `snake_case` - All directories
- `kernel_` prefix - Subsystem crates
- Plural for collections: `crates/`, `demos/`, `examples/`

**Special Patterns:**
- `*_new.rs` - Alternative implementations
- `*.ld` - Linker scripts
- `*.toml` - Configuration files

## Where to Add New Code

**New BPF Feature:**
- Primary code: `kernel/crates/kernel_bpf/src/`
- Tests: Same file in `#[cfg(test)]` module
- Benchmarks: `kernel/crates/kernel_bpf/benches/`

**New Syscall:**
- Definition: `kernel/crates/kernel_abi/src/syscall.rs`
- Handler: `kernel/crates/kernel_syscall/src/`
- BPF-specific: `kernel/src/syscall/bpf.rs`

**New Architecture:**
- Implementation: `kernel/src/arch/{arch_name}/`
- Entry: `kernel/src/main_{arch}.rs`
- Platform config: `kernel/platform/{platform}/`

**New Driver:**
- Implementation: `kernel/src/driver/`
- VirtIO: `kernel/src/driver/virtio/`

**Utilities:**
- Shared helpers: `kernel/crates/kernel_*/src/`
- Userspace: `userspace/minilib/`

## Special Directories

**.planning/**
- Purpose: GSD project planning documents
- Source: Generated by `/gsd:map-codebase`
- Committed: Yes

**target/**
- Purpose: Build artifacts
- Source: Cargo build output
- Committed: No (in .gitignore)

**kernel/platform/rpi5/**
- Purpose: Raspberry Pi 5 platform configuration
- Contains: Boot config, device tree, memory map
- Committed: Yes

---

*Structure analysis: 2026-01-27*
*Update when directory structure changes*

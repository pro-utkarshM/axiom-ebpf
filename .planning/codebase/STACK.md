# Technology Stack

**Analysis Date:** 2026-01-27

## Languages

**Primary:**
- Rust (Nightly, Edition 2024) - All kernel and userspace code

**Secondary:**
- C - BPF example programs (`examples/bpf/hello.bpf.c`)

## Runtime

**Environment:**
- Bare-metal kernel (no_std) - No OS runtime
- Target architectures: x86_64, aarch64 (Raspberry Pi 5), riscv64 (WIP)
- Requires Rust nightly toolchain

**Package Manager:**
- Cargo with Rust Workspace
- Lockfile: `Cargo.lock` present (pinned dependencies)

**Toolchain Configuration:**
- `rust-toolchain.toml` - Nightly channel with components: rustfmt, clippy, llvm-tools-preview, rust-src, miri

## Frameworks

**Core:**
- None (vanilla bare-metal Rust kernel)

**Testing:**
- Rust built-in test framework - Unit tests via `#[test]`
- Criterion 0.5 - Benchmarking (`kernel/crates/kernel_bpf/benches/`)
- Miri - Undefined behavior detection

**Build/Dev:**
- Cargo - Build system
- Custom `build.rs` - ISO/disk image creation
- Limine v9.x - Bootloader
- OVMF 0.2.3 - UEFI firmware for QEMU (x86_64)

## Key Dependencies

**Critical:**
- `spin = "0.10"` - Spinlock synchronization primitive - `Cargo.toml`
- `x86_64 = "0.15"` - x86_64 architecture crate - `Cargo.toml`
- `aarch64-cpu = "9.4"` - ARM64 CPU utilities - `Cargo.toml`
- `limine = "0.5"` - Bootloader protocol - `Cargo.toml`
- `linked_list_allocator = "0.10"` - Memory allocator - `Cargo.toml`

**Infrastructure:**
- `virtio-drivers = "0.12"` - VirtIO device drivers - `Cargo.toml`
- `acpi = "5.2"` - ACPI table parsing (x86_64) - `Cargo.toml`
- `ext2 = "0.4"` - Ext2 filesystem - `Cargo.toml`
- `zerocopy = "0.9.0-alpha.0"` - Safe zero-copy memory layouts - `Cargo.toml`
- `volatile = "0.6"` - Safe volatile memory access - `Cargo.toml`

**Development:**
- `clap = "4.5"` - CLI argument parsing (runner) - `Cargo.toml`
- `addr2line = "0.25"` - Debug symbol resolution - `Cargo.toml`

## Configuration

**Environment:**
- No environment variables required
- Configuration via Cargo features and linker scripts

**Build:**
- `Cargo.toml` - Workspace configuration
- `kernel/Cargo.toml` - Kernel build config
- `rust-toolchain.toml` - Rust version
- `limine.conf` - Bootloader configuration
- `kernel/linker-x86_64.ld`, `kernel/linker-aarch64.ld` - Linker scripts

**Feature Flags:**
- `cloud-profile` - Cloud/server deployment (elastic memory, JIT, throughput scheduling)
- `embedded-profile` - Embedded/IoT deployment (static 64KB pool, no JIT, deadline scheduling)

## Platform Requirements

**Development:**
- Linux/macOS (any platform with Rust nightly toolchain)
- QEMU for testing (automatically invoked)
- No Docker required

**Production:**
- x86_64: UEFI-compatible systems, VirtIO devices
- aarch64: Raspberry Pi 5 with custom bootloader
- riscv64: RISC-V 64-bit systems (implementation incomplete)

---

*Stack analysis: 2026-01-27*
*Update after major dependency changes*

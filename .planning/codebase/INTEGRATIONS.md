# External Integrations

**Analysis Date:** 2026-01-27

## APIs & External Services

**Payment Processing:**
- Not applicable (bare-metal kernel)

**Email/SMS:**
- Not applicable

**External APIs:**
- Not applicable - Bare-metal kernel with no external service dependencies

## Data Storage

**Databases:**
- Not applicable (no database clients)

**File Storage:**
- Ext2 filesystem - Primary filesystem implementation - `kernel/src/file/ext2.rs`
- DevFS - Device filesystem - `kernel/src/file/devfs.rs`
- VFS abstraction - `kernel/crates/kernel_vfs/`

**Caching:**
- None (all in-memory, no persistent cache)

## Authentication & Identity

**Auth Provider:**
- Not applicable

**OAuth Integrations:**
- None

## Monitoring & Observability

**Error Tracking:**
- Serial console output - `kernel/src/serial.rs`
- Backtrace support - `kernel/src/backtrace.rs`

**Analytics:**
- Not applicable

**Logs:**
- Kernel log via `log` crate - `kernel/src/log.rs`
- Serial console output to UART

## CI/CD & Deployment

**Hosting:**
- Bare-metal deployment or QEMU emulation
- ISO image generation via `build.rs`

**CI Pipeline:**
- GitHub Actions - `.github/workflows/build.yml`, `.github/workflows/bpf-profiles.yml`
- Jobs: lint, test, miri, build
- Schedule: On push + twice daily (5am and 5pm UTC)

## Environment Configuration

**Development:**
- Required: Rust nightly toolchain
- Optional: QEMU for testing
- No secrets/env vars required

**Staging:**
- Not applicable (bare-metal)

**Production:**
- Boot via Limine bootloader
- ISO image deployment

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## BPF Subsystem Integration

**Core eBPF Components:**
- Verifier: Streaming verification (50KB peak memory) - `kernel/crates/kernel_bpf/src/verifier/`
- Interpreter: Complete BPF instruction interpreter - `kernel/crates/kernel_bpf/src/execution/interpreter.rs`
- JIT Compilers:
  - x86_64 JIT (full) - `kernel/crates/kernel_bpf/src/execution/jit/`
  - ARM64 JIT (partial) - `kernel/crates/kernel_bpf/src/execution/jit_aarch64.rs`

**BPF Map Types:**
1. Array Map - `kernel/crates/kernel_bpf/src/maps/array.rs`
2. Hash Map - `kernel/crates/kernel_bpf/src/maps/hash.rs`
3. Ring Buffer - `kernel/crates/kernel_bpf/src/maps/ringbuf.rs`
4. Time Series Map - `kernel/crates/kernel_bpf/src/maps/timeseries.rs`
5. Static Pool (embedded) - `kernel/crates/kernel_bpf/src/maps/static_pool.rs`

**BPF Syscall Interface:**
- `sys_bpf` handler - `kernel/src/syscall/bpf.rs`
- Operations: BPF_MAP_CREATE, BPF_MAP_LOOKUP_ELEM, BPF_MAP_UPDATE_ELEM, BPF_MAP_DELETE_ELEM, BPF_PROG_LOAD, BPF_PROG_ATTACH

**BPF Helper Functions:**
- `bpf_ktime_get_ns` - Get kernel time - `kernel/src/bpf/helpers.rs`
- `bpf_trace_printk` - Print to kernel logs - `kernel/src/bpf/helpers.rs`
- `bpf_map_lookup_elem` - Map lookup - `kernel/src/bpf/helpers.rs`
- `bpf_map_update_elem` - Map update - `kernel/src/bpf/helpers.rs`
- `bpf_map_delete_elem` - Map deletion - `kernel/src/bpf/helpers.rs`

**Attach Points:**
- Timer events (attach_type=1) - `kernel/src/syscall/mod.rs`
- Syscall entry (attach_type=2) - `kernel/src/syscall/mod.rs`
- Planned: GPIO, PWM, IIO, Kprobe, Tracepoint - `kernel/crates/kernel_bpf/src/attach/`

## Userspace Tools Integration

**rk-bridge:**
- eBPF Ring Buffer to ROS2 Bridge - `userspace/rk_bridge/`
- Dependencies: tokio, serde, libc, clap
- Optional ROS2 integration via `ros2` feature

**rk-cli:**
- BPF deployment & management CLI - `userspace/rk_cli/`
- Dependencies: clap, ring (crypto), sha3, walkdir

**Init Process:**
- PID 1 init - `userspace/init/`
- Starts BPF subsystem and loads initial programs

## Profile-Based Deployment

**Cloud Profile:**
- Memory: Elastic heap allocation
- Stack: 512 KB
- Instructions: 1,000,000 soft limit
- JIT: Available
- Build: `cargo build --no-default-features --features cloud-profile -p kernel_bpf`

**Embedded Profile (Default):**
- Memory: Static 64KB pool
- Stack: 8 KB
- Instructions: 100,000 hard limit
- JIT: Erased at compile time
- Build: `cargo build --no-default-features --features embedded-profile -p kernel_bpf`

---

*Integration audit: 2026-01-27*
*Update when adding/removing external services*

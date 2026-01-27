# Codebase Concerns

**Analysis Date:** 2026-01-27

## Tech Debt

**Hardcoded BPF Map Sizes:**
- Issue: All BPF maps assumed to have 4-byte keys and 8-byte values
- Files: `kernel/src/syscall/bpf.rs` (lines 67, 101-103)
- Why: Rapid implementation without proper map definition parsing
- Impact: Breaks any map with different sizing, buffer overflow risk
- Fix approach: Extract key/value sizes from BpfAttr structure properly

**Unsafe Pointer Casts Without Validation:**
- Issue: User-provided pointers cast directly to kernel structures
- Files: `kernel/src/syscall/bpf.rs` (lines 22, 54, 88, 123, 150, 177)
- Why: Initial implementation prioritized functionality
- Impact: Security risk - could read/write arbitrary kernel memory
- Fix approach: Add address space, alignment, and bounds validation

**JIT Stack Size Hardcoding:**
- Issue: ARM64 JIT uses fixed 512-byte stack for all programs
- File: `kernel/crates/kernel_bpf/src/execution/jit_aarch64.rs` (line 634)
- Why: Simplified implementation
- Impact: Stack overflow for programs with deep call stacks
- Fix approach: Compute stack size from program analysis

**Edition 2024 in Cargo.toml:**
- Issue: Edition "2024" specified but doesn't exist (current is 2021)
- Files: `Cargo.toml`, `kernel/Cargo.toml`
- Why: Likely typo or forward-looking
- Impact: Build failure on standard toolchains
- Fix approach: Change to `edition = "2021"`

## Known Bugs

**VFS Node Reuse Not Implemented:**
- Symptoms: Repeated file opens create new VfsNodes
- Trigger: Any file access pattern
- Files: `kernel/crates/kernel_vfs/src/vfs/mod.rs` (line 89)
- Workaround: None (performance degradation only)
- Root cause: FIXME comment indicates known gap

**Mount Point Validation Missing:**
- Symptoms: Can mount filesystem at non-directory paths
- Trigger: `vfs.mount("/file", fs)`
- File: `kernel/crates/kernel_vfs/src/vfs/mod.rs` (line 57)
- Workaround: None
- Root cause: TODO comment indicates unimplemented check

## Security Considerations

**Syscall Pointer Validation:**
- Risk: User-provided pointers passed to unsafe blocks without validation
- Files: `kernel/src/syscall/bpf.rs` (multiple locations)
- Current mitigation: Basic null check only (`if attr_ptr == 0`)
- Recommendations:
  - Add address space verification (user vs kernel)
  - Add alignment validation
  - Add bounds checking on data lengths

**Missing Safety Comments:**
- Risk: 70+ files with unsafe blocks lack SAFETY documentation
- Files: Throughout `kernel/src/`, especially `syscall/`, `arch/`
- Current mitigation: None
- Recommendations: Add SAFETY comments documenting invariants

## Performance Bottlenecks

**Linear Memory Search:**
- Problem: Physical frame allocator does linear scan
- File: `kernel/crates/kernel_physical_memory/src/lib.rs` (lines 73-80)
- Measurement: O(n) per allocation where n = number of regions
- Cause: Simple implementation without free list
- Improvement path: Add buddy allocator or bitmap-based tracking

**BTreeMap for VFS Paths:**
- Problem: Mount point lookup uses BTreeMap
- File: `kernel/crates/kernel_vfs/src/vfs/mod.rs` (line 23)
- Measurement: O(log n) lookups on every file operation
- Cause: Simple implementation
- Improvement path: Trie-based mount point tracking

## Fragile Areas

**Page Fault Handler:**
- File: `kernel/src/arch/idt.rs` (lines 240-290)
- Why fragile: 5 TODOs in critical exception handling
- Common failures: Missing lazy allocation, nested page faults
- Safe modification: Add extensive testing before changes
- Test coverage: No automated tests (bare-metal kernel)

**BPF Syscall Handler:**
- File: `kernel/src/syscall/bpf.rs`
- Why fragile: Multiple unsafe blocks, hardcoded assumptions
- Common failures: Invalid pointer access, size mismatches
- Safe modification: Add input validation layer
- Test coverage: No unit tests currently

## Scaling Limits

**Static Memory Pool (Embedded Profile):**
- Current capacity: 64KB fixed pool
- Limit: ~100-200 BPF programs depending on size
- Symptoms at limit: Allocation failures
- Scaling path: Increase pool size or switch to cloud profile

**BPF Instruction Limits:**
- Cloud: 1,000,000 instructions (soft)
- Embedded: 100,000 instructions (hard)
- Symptoms at limit: Verification failure
- Scaling path: Profile selection at compile time

## Dependencies at Risk

**Alpha/RC Dependencies:**
- `zerocopy = "0.9.0-alpha.0"` - Alpha version
- `sha3 = "0.11.0-rc.3"` - Release candidate
- Risk: API changes, bugs, security issues
- Migration plan: Update to stable versions when released

**Git Dependencies:**
- `mkfs-ext2 = { git = ... }` - Not versioned
- `mkfs-filesystem = { git = ... }` - Not versioned
- Risk: Breaking changes without warning
- Migration plan: Pin to specific commits or use releases

## Missing Critical Features

**BTF Parsing:**
- Problem: Binary Type Format support not implemented
- File: `kernel/crates/kernel_bpf/src/loader/mod.rs` (line 152)
- Current workaround: Manual type definitions
- Blocks: Rich debugging, CO-RE (Compile Once Run Everywhere)
- Implementation complexity: Medium-High

**Demand Paging (aarch64):**
- Problem: Page fault handling incomplete
- File: `kernel/src/arch/aarch64/exceptions.rs` (line 178)
- Current workaround: All memory pre-allocated
- Blocks: Efficient memory usage
- Implementation complexity: Medium

## Test Coverage Gaps

**BPF Syscall Handler:**
- What's not tested: `kernel/src/syscall/bpf.rs`
- Risk: Security vulnerabilities, data corruption
- Priority: High
- Difficulty: Requires kernel testing framework

**Unsafe Pointer Operations:**
- What's not tested: Pointer validation in syscall handlers
- Risk: Memory safety violations
- Priority: High
- Difficulty: Need to test invalid inputs safely

**RISC-V Platform:**
- What's not tested: `kernel/src/arch/riscv64/`
- Risk: Runtime failures on RISC-V hardware
- Priority: Medium
- Difficulty: Requires RISC-V emulator/hardware

## Platform Implementation Gaps

**RISC-V (Incomplete):**
- `kernel/src/main_riscv.rs` - Only prints TODO messages
- `kernel/src/arch/riscv64/interrupts.rs` - PLIC handling not implemented
- `kernel/src/arch/riscv64/paging.rs` - Kernel page tables not set up
- Impact: RISC-V builds but doesn't function

**AArch64 (Partial):**
- `kernel/src/arch/aarch64/exceptions.rs` - Demand paging, COW not implemented
- Impact: Full memory pre-allocation required

---

*Concerns audit: 2026-01-27*
*Update as issues are fixed or new ones discovered*

# Testing Patterns

**Analysis Date:** 2026-01-27

## Test Framework

**Runner:**
- Rust built-in test framework
- No separate config file needed

**Assertion Library:**
- Rust built-in: `assert!`, `assert_eq!`, `assert_ne!`
- Pattern matching: `assert!(matches!(...))`

**Run Commands:**
```bash
cargo test                              # Run all tests
cargo test -p kernel_bpf                # Single crate
cargo test --release                    # Release mode
cargo miri test -p <crate>              # Miri undefined behavior check
cargo bench -p kernel_bpf               # Run benchmarks
```

## Test File Organization

**Location:**
- Co-located with source in `#[cfg(test)]` modules
- No separate `tests/` directories (except `kernel/crates/kernel_bpf/tests/`)

**Naming:**
- Unit tests: Same file, `mod tests` at end
- Integration tests: `kernel/crates/kernel_bpf/tests/`
- Benchmarks: `kernel/crates/kernel_bpf/benches/`

**Structure:**
```
kernel/crates/kernel_bpf/
├── src/
│   ├── attach/
│   │   ├── gpio.rs          # contains #[cfg(test)] mod tests
│   │   └── kprobe.rs         # contains #[cfg(test)] mod tests
│   ├── bytecode/
│   │   └── insn.rs           # contains #[cfg(test)] mod tests
│   └── ...
├── tests/
│   └── semantic_consistency.rs
└── benches/
    ├── interpreter.rs
    ├── verifier.rs
    └── maps.rs
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_gpio_attach() {
        let gpio = GpioAttach::<ActiveProfile>::new(
            "gpiochip0", 17, GpioEdge::Rising
        ).unwrap();

        assert_eq!(gpio.chip(), "gpiochip0");
        assert_eq!(gpio.line(), 17);
        assert_eq!(gpio.edge(), GpioEdge::Rising);
    }

    #[test]
    fn invalid_function_name() {
        let result = KprobeAttach::<ActiveProfile>::new("", KprobeType::Entry);
        assert!(matches!(result, Err(AttachError::InvalidTarget(_))));
    }
}
```

**Patterns:**
- One test per behavior
- Descriptive test names
- Arrange/Act/Assert structure (implicit)

## Mocking

**Framework:**
- No mocking framework used
- Manual test doubles where needed

**Patterns:**
- Profile-specific testing via feature flags
- Generic type parameters for dependency injection

**What to Mock:**
- External hardware interactions (via abstractions)

**What NOT to Mock:**
- Pure functions
- Internal logic

## Fixtures and Factories

**Test Data:**
```rust
// Factory pattern in test
fn create_test_program() -> BpfProgram<ActiveProfile> {
    ProgramBuilder::<ActiveProfile>::new(BpfProgType::SocketFilter)
        .insn(BpfInsn::mov64_imm(0, 0))
        .insn(BpfInsn::exit())
        .build()
        .expect("valid program")
}
```

**Location:**
- Factory functions in test modules
- No shared fixtures directory

## Coverage

**Requirements:**
- No enforced coverage target
- Focus on critical paths (verifier, execution)

**Configuration:**
- No coverage tool configured
- Miri for undefined behavior detection

**View Coverage:**
```bash
# Not configured - use cargo-tarpaulin if needed
cargo install cargo-tarpaulin
cargo tarpaulin -p kernel_bpf
```

## Test Types

**Unit Tests:**
- Scope: Single function/struct in isolation
- Location: `#[cfg(test)] mod tests` in source files
- Examples: `kernel/crates/kernel_bpf/src/bytecode/insn.rs`

**Integration Tests:**
- Scope: Multiple modules together
- Location: `kernel/crates/kernel_bpf/tests/`
- Examples: `semantic_consistency.rs` (profile consistency)

**Benchmarks:**
- Framework: Criterion 0.5
- Location: `kernel/crates/kernel_bpf/benches/`
- Examples: `interpreter.rs`, `verifier.rs`, `maps.rs`

**Profile-Specific Tests:**
```bash
# Cloud profile
cargo test -p kernel_bpf --no-default-features --features cloud-profile

# Embedded profile (default)
cargo test -p kernel_bpf --no-default-features --features embedded-profile
```

## Common Patterns

**Async Testing:**
- Not applicable (no async in kernel_bpf)

**Error Testing:**
```rust
#[test]
fn invalid_function_name() {
    let result = KprobeAttach::<ActiveProfile>::new("", KprobeType::Entry);
    assert!(matches!(result, Err(AttachError::InvalidTarget(_))));
}
```

**Snapshot Testing:**
- Not used

**Benchmark Pattern:**
```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter/arithmetic");

    let program = create_test_program();
    let interp = Interpreter::<ActiveProfile>::new();
    let ctx = BpfContext::empty();

    group.bench_function("simple_math", |b| {
        b.iter(|| interp.execute(black_box(&program), black_box(&ctx)))
    });

    group.finish();
}

criterion_group!(benches, bench_arithmetic);
criterion_main!(benches);
```

## CI Pipeline

**Jobs (`.github/workflows/build.yml`):**

1. **Lint:**
   - `cargo fmt -- --check`
   - `cargo clippy --workspace --lib -- -D clippy::all`

2. **Test:**
   - Matrix: debug and release modes
   - `cargo test` and `cargo test --release`

3. **Miri (per-crate):**
   - `cargo miri test -p <package>`
   - Excludes kernel_bpf (has dedicated job)

4. **Miri kernel_bpf:**
   - Profile matrix: cloud-profile, embedded-profile
   - `cargo miri test -p kernel_bpf --no-default-features --features <profile>`

5. **Build:**
   - `cargo build --release`
   - Artifacts: `muffin.iso`

**Schedule:** On push + twice daily (0 5,17 * * *)

**Local Validation:**
```bash
cargo fmt -- --check
cargo clippy --workspace --lib -- -D clippy::all
cargo build
cargo test
cargo miri setup
cargo miri test -p kernel_bpf
cargo build --release
```

## Test Gaps

**Known Gaps:**
- BPF syscall handler (`kernel/src/syscall/bpf.rs`) - No unit tests
- Unsafe pointer operations - Limited testing
- Full BPF lifecycle integration - Manual testing only
- JIT compiler correctness - Limited coverage

**Priority Areas:**
- Verifier (critical for safety)
- Map operations (data integrity)
- Syscall boundary validation

---

*Testing analysis: 2026-01-27*
*Update when test patterns change*

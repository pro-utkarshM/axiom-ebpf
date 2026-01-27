# Coding Conventions

**Analysis Date:** 2026-01-27

## Naming Patterns

**Files:**
- `snake_case.rs` for all Rust files
- `mod.rs` for module entry points
- `lib.rs` for crate roots
- `*.test.rs` pattern not used (tests in same file)

**Functions:**
- `snake_case` for all functions
- Constructor pattern: `new()`, `from_*()`
- Getters: Simple method names without prefix (`chip()`, `line()`, `edge()`)
- Predicates: `is_*()` pattern (`is_rising()`, `is_falling()`, `is_writable()`)

**Variables:**
- `snake_case` for all variables
- Register variables: `dst`, `src`, `regs`, `value`
- Counters: `next_id`, `index`
- Constants: `SCREAMING_SNAKE_CASE` (`KERNEL_BINARY`, `BOOTABLE_ISO`)

**Types:**
- `PascalCase` for structs: `BpfInsn`, `RegisterFile`, `BpfProgram`
- `PascalCase` for enums: `OpcodeClass`, `SourceType`, `AluOp`
- `PascalCase` for traits: `PhysicalProfile`, `AttachPoint`, `BpfExecutor`
- No `I` prefix for interfaces/traits

## Code Style

**Formatting:**
- Rustfmt with `.rustfmt.toml`
- `imports_granularity = "Module"`
- `group_imports = "StdExternalCrate"`
- 4-space indentation (Rust standard)

**Linting:**
- Clippy with all warnings as errors: `-D clippy::all`
- Run: `cargo clippy --workspace --lib -- -D clippy::all`
- CI enforces: Format check mandatory

**Attributes:**
- `#[repr(C)]` for FFI structs
- `#[inline]` on hot-path functions
- `#[must_use]` on constructors returning values
- `#[allow(dead_code)]` for intentional unused code

## Import Organization

**Order:**
1. `extern crate` declarations
2. Standard library (`core::`, `alloc::`)
3. External crates
4. Internal modules (`super::`, `crate::`)

**Grouping:**
- Blank line between groups
- `use` statements at top of file
- Re-exports via `pub use`

**Path Aliases:**
- None defined (use relative paths)

## Error Handling

**Patterns:**
- `Result<T, E>` for fallible operations
- `expect()` for initialization failures (acceptable kernel panic)
- Error codes returned to userspace

**Error Types:**
- Custom error enums per module (`VerifyError`, `AttachError`, `OpenError`)
- `thiserror` for error derivation in some crates

**Unsafe:**
- Required `// SAFETY:` comment for all unsafe blocks (not consistently followed)
- Prefer safe Rust; justify all unsafe blocks

## Logging

**Framework:**
- `log` crate abstraction - `kernel/src/log.rs`
- Serial console output

**Patterns:**
- `log::info!`, `log::debug!`, `log::error!` macros
- Debug output to serial console

## Comments

**When to Comment:**
- Module-level `//!` documentation
- Public API `///` documentation
- Explain why, not what
- Safety comments for unsafe blocks

**JSDoc/TSDoc:**
- Not applicable (Rust uses `///` doc comments)

**TODO Comments:**
- Format: `// TODO: description`
- Also: `// FIXME: description`
- 51 TODOs found in codebase

## Function Design

**Size:**
- Keep functions focused
- Extract helpers for complex logic

**Parameters:**
- Generic bounds: `<P: PhysicalProfile>`
- Reference parameters preferred
- Use `AsRef<T>` for path-like parameters

**Return Values:**
- `Result<T, E>` for fallible operations
- `Option<T>` for nullable returns
- Return early for guard clauses

## Module Design

**Exports:**
- Named exports preferred
- Re-export public API from `mod.rs`
- `pub use` for submodule re-exports

**Visibility:**
- `pub` for public API
- `pub(crate)` for crate-internal
- Private by default

**Crate Organization:**
- `kernel_` prefix for subsystem crates
- Each crate independently testable
- Workspace dependencies via `workspace = true`

## Test Patterns

**Location:**
- Co-located `#[cfg(test)] mod tests { ... }` at end of file

**Naming:**
- `snake_case` test function names
- Descriptive names: `create_gpio_attach`, `invalid_function_name`

**Assertions:**
- `assert_eq!()` for equality
- `assert!()` for boolean
- `assert!(matches!())` for pattern matching

---

*Convention analysis: 2026-01-27*
*Update when patterns change*

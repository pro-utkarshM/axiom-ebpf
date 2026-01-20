# rkBPF Implementation Roadmap

## Immediate Priority: Complete Phase 1

### 1. Streaming Verifier (Core Innovation)

**Location:** `kernel/crates/kernel_bpf/src/verifier/`

**Design Goals:**
- Implement O(registers × basic_block_depth) algorithm
- Single forward pass, maintain only current basic block state
- Target: Verify 1000-insn program in <50KB memory

**Implementation Details:**
- Track register states per basic block only
- Discard state after block completion
- Handle jumps by recording entry states at branch targets
- Profile-aware: embedded profile uses minimal state tracking

---

### 2. Ring Buffer Map

**Location:** `kernel/crates/kernel_bpf/src/maps/ringbuf.rs`

**Design Goals:**
- Lock-free ring buffer for event streaming
- Profile-aware sizing (cloud: elastic, embedded: fixed)
- Implement `bpf_ringbuf_output()` helper

**Implementation Details:**
- Producer-consumer model with atomic operations
- Memory-mapped interface for userspace consumption
- Configurable buffer size based on profile
- Support for variable-length records

---

### 3. Hash Map

**Location:** `kernel/crates/kernel_bpf/src/maps/hash.rs`

**Design Goals:**
- Key-value storage with profile-aware allocation
- Support for BPF_MAP_TYPE_HASH

**Implementation Details:**
- Open addressing with linear probing (embedded)
- Chained hashing option (cloud profile)
- Pre-allocated buckets for deterministic memory
- Support for per-CPU variants

---

### 4. libbpf-free Loader (~50KB)

**Location:** `kernel/crates/kernel_bpf/src/loader/`

**Design Goals:**
- Minimal ELF parser for BPF objects
- No external dependencies
- Program and map relocation

**Implementation Details:**
- Parse ELF headers (minimal subset)
- Extract .text, .maps, .rodata sections
- Apply relocations for map references
- Validate program against profile constraints

---

### 5. ARM64 JIT Compiler

**Location:** `kernel/crates/kernel_bpf/src/execution/jit/aarch64.rs`

**Design Goals:**
- Target Jetson Nano / RPi platforms
- Cloud profile only (erased from embedded)

**Implementation Details:**
- BPF to AArch64 instruction mapping
- Register allocation (BPF R0-R10 to ARM64 registers)
- Prologue/epilogue generation
- Helper call trampolines

---

## Phase 2: Robotics Integration

### 6. Attach Point Abstraction (HIL-style)

**Location:** `kernel/crates/kernel_bpf/src/attach/`

**Design Goals:**
- Trait-based hardware interface (inspired by Tock OS)
- ImuSensor, MotorController, GpioLine traits

**Implementation Details:**
```rust
pub trait AttachPoint {
    fn attach(&self, prog: &BpfProgram) -> Result<AttachHandle>;
    fn detach(&self, handle: AttachHandle) -> Result<()>;
}

pub trait ImuSensor: AttachPoint {
    fn on_sample(&self) -> AttachType;
}

pub trait GpioLine: AttachPoint {
    fn on_edge(&self, edge: Edge) -> AttachType;
}
```

---

### 7. IIO/GPIO/PWM Integration

**Design Goals:**
- Linux Industrial I/O subsystem hooks
- GPIO event tracing
- PWM observation for motor commands

**Implementation Details:**
- Hook into IIO buffer completion events
- Trace GPIO edge interrupts
- Capture PWM configuration changes
- Expose sensor data to BPF programs

---

### 8. ROS2 Bridge

**Location:** `userspace/rk_bridge/`

**Design Goals:**
- Daemon to forward ring buffer events to ROS topics
- `rk-cli` for loading/attaching programs

**Implementation Details:**
- Poll ring buffer for new events
- Convert events to ROS2 message format
- Publish to configurable topics
- CLI commands: `load`, `attach`, `detach`, `list`

---

## Verification Strategy

### Unit Tests
```bash
cargo test -p kernel_bpf --features <profile>
```

### Miri (Undefined Behavior Detection)
```bash
cargo miri test -p kernel_bpf
```

### QEMU Integration
```bash
cargo run  # Boots kernel with BPF subsystem
```

### Benchmarks
- Memory usage per component
- Verification time vs instruction count
- Execution overhead (interpreter vs JIT)

---

## File Structure

```
kernel/crates/kernel_bpf/src/
├── attach/           # Phase 2: Attach points
│   ├── mod.rs
│   ├── iio.rs
│   ├── gpio.rs
│   └── pwm.rs
├── bytecode/         # DONE: Instruction parsing
├── execution/
│   ├── interpreter.rs  # DONE
│   └── jit/
│       ├── mod.rs
│       ├── x86_64.rs   # Stub
│       └── aarch64.rs  # TODO
├── loader/           # TODO: ELF parsing
│   ├── mod.rs
│   ├── elf.rs
│   └── reloc.rs
├── maps/
│   ├── array.rs      # DONE
│   ├── hash.rs       # TODO
│   └── ringbuf.rs    # TODO
├── profile/          # DONE
├── scheduler/        # DONE
└── verifier/         # PARTIAL (streaming TODO)

userspace/rk_bridge/  # TODO: ROS2 bridge daemon
```

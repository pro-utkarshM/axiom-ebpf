# rkBPF Task Tracking

## Phase 1: Core Runtime (Proposal Weeks 1-6)

### Completed

- [x] **Profile System** - `kernel_bpf/src/profile/`
- [x] **Bytecode Module** - `kernel_bpf/src/bytecode/`
- [x] **Interpreter** - `kernel_bpf/src/execution/interpreter.rs`
- [x] **Array Map** - `kernel_bpf/src/maps/array.rs`
- [x] **Scheduler** - `kernel_bpf/src/scheduler/`
- [x] **Streaming Verifier** - `kernel_bpf/src/verifier/streaming.rs` - O(registers × basic_block_depth) algorithm
- [x] **Ring Buffer Map** - `kernel_bpf/src/maps/ringbuf.rs` - Lock-free kernel-to-userspace streaming
- [x] **Hash Map** - `kernel_bpf/src/maps/hash.rs` - O(1) lookup with linear probing
- [x] **libbpf-free Loader** - `kernel_bpf/src/loader/` - Minimal ELF64 parser (~50KB)
- [x] **ARM64 JIT Compiler** - `kernel_bpf/src/execution/jit_aarch64.rs`

### In Progress

- [ ] **Verifier Framework** - `kernel_bpf/src/verifier/` (core framework, needs helper integration)
- [ ] **JIT Compiler** - `kernel_bpf/src/execution/jit/` (x86 stub only)

**Phase 1 Progress: ~90%**

---

## Phase 2: Robotics Integration (Weeks 7-10)

### Completed

- [x] **Attach Point Abstraction** - `kernel_bpf/src/attach/mod.rs`
- [x] **IIO Subsystem Attach Points** - `kernel_bpf/src/attach/iio.rs`
- [x] **GPIO Event Hooks** - `kernel_bpf/src/attach/gpio.rs`
- [x] **PWM Observation Points** - `kernel_bpf/src/attach/pwm.rs`
- [x] **Kprobe/Tracepoint Attach** - `kernel_bpf/src/attach/kprobe.rs`, `tracepoint.rs`

### Pending

- [ ] **ros2_tracing Bridge** - `userspace/rk_bridge/`
- [ ] **Time-Series Map Type**

**Phase 2 Progress: ~70%**

---

## Phase 3-4: Production & Ecosystem

### In Progress

- [ ] **Documentation** - partial

### Pending

- [ ] **Program Signing**
- [ ] **Deployment Tooling**
- [ ] **Benchmarks**

**Phase 3-4 Progress: ~10%**

---

## Priority Queue

1. ~~Streaming Verifier (core innovation)~~ DONE
2. ~~Ring Buffer Map~~ DONE
3. ~~Hash Map~~ DONE
4. ~~libbpf-free Loader~~ DONE
5. ~~ARM64 JIT Compiler~~ DONE
6. ~~Attach Point Abstraction~~ DONE
7. ~~IIO/GPIO/PWM Integration~~ DONE
8. ROS2 Bridge
9. Time-Series Map
10. x86_64 JIT Compiler
11. Helper function integration

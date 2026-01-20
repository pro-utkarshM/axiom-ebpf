# rkBPF Task Tracking

## Phase 1: Core Runtime (Proposal Weeks 1-6)

### Completed

- [x] **Profile System** - `kernel_bpf/src/profile/`
- [x] **Bytecode Module** - `kernel_bpf/src/bytecode/`
- [x] **Interpreter** - `kernel_bpf/src/execution/interpreter.rs`
- [x] **Array Map** - `kernel_bpf/src/maps/array.rs`
- [x] **Scheduler** - `kernel_bpf/src/scheduler/`

### In Progress

- [ ] **Verifier Framework** - `kernel_bpf/src/verifier/` (partial, not streaming)
- [ ] **JIT Compiler** - `kernel_bpf/src/execution/jit/` (x86 stub only)

### Pending

- [ ] **Streaming Verifier** - O(registers × basic_block_depth) algorithm
- [ ] **Hash Map** - `kernel_bpf/src/maps/hash.rs`
- [ ] **Ring Buffer** - `kernel_bpf/src/maps/ringbuf.rs`
- [ ] **libbpf-free Loader** - `kernel_bpf/src/loader/`
- [ ] **ARM64 JIT Compiler** - `kernel_bpf/src/execution/jit/aarch64.rs`

**Phase 1 Progress: ~50%**

---

## Phase 2: Robotics Integration (Weeks 7-10)

### Pending

- [ ] **Attach Point Abstraction** - `kernel_bpf/src/attach/`
- [ ] **IIO Subsystem Attach Points**
- [ ] **GPIO Event Hooks**
- [ ] **PWM Observation Points**
- [ ] **ros2_tracing Bridge** - `userspace/rk_bridge/`
- [ ] **Time-Series Map Type**

**Phase 2 Progress: 0%**

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

1. Streaming Verifier (core innovation)
2. Ring Buffer Map
3. Hash Map
4. libbpf-free Loader
5. ARM64 JIT Compiler
6. Attach Point Abstraction
7. IIO/GPIO/PWM Integration
8. ROS2 Bridge

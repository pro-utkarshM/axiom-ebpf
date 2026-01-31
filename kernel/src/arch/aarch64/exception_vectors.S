// exception_vectors.S - AArch64 Exception Vector Table
//
// ARM64 exception vectors are organized as:
//   - 4 exception types: Synchronous, IRQ, FIQ, SError
//   - 4 execution contexts: current EL SP0, current EL SPx, lower EL 64-bit, lower EL 32-bit
//   - Total: 16 entries, each 128 bytes (0x80)
//   - Table must be 2KB aligned
//
// This provides trampolines to Rust exception handlers.

.section .text
.global exception_vector_base
.balign 2048

// Macro to handle invalid exceptions
.macro invalid_exception kind, source
    .balign 0x80
    // Save caller-saved registers
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x29, [sp, #-16]!
    stp     x30, xzr, [sp, #-16]!

    // Call Rust handler with kind and source
    mov     x0, #\kind
    mov     x1, #\source
    bl      handle_invalid_exception

    // Should never return, but if it does, hang
1:  wfi
    b       1b
.endm

exception_vector_base:

// =============================================================================
// Current EL with SP0 (not typically used in kernel)
// =============================================================================
invalid_exception 0, 0    // Synchronous
invalid_exception 1, 0    // IRQ
invalid_exception 2, 0    // FIQ
invalid_exception 3, 0    // SError

// =============================================================================
// Current EL with SPx (kernel mode exceptions)
// =============================================================================
.balign 0x80
curr_el_spx_sync:
    // Save caller-saved registers
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x29, [sp, #-16]!
    stp     x30, xzr, [sp, #-16]!

    // Call Rust handler
    bl      handle_sync_exception

    // Restore registers
    ldp     x30, xzr, [sp], #16
    ldp     x18, x29, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x8, x9, [sp], #16
    ldp     x6, x7, [sp], #16
    ldp     x4, x5, [sp], #16
    ldp     x2, x3, [sp], #16
    ldp     x0, x1, [sp], #16
    eret

.balign 0x80
curr_el_spx_irq:
    // Save caller-saved registers
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x29, [sp, #-16]!
    stp     x30, xzr, [sp, #-16]!

    // Call Rust handler
    bl      handle_irq

    // Restore registers
    ldp     x30, xzr, [sp], #16
    ldp     x18, x29, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x8, x9, [sp], #16
    ldp     x6, x7, [sp], #16
    ldp     x4, x5, [sp], #16
    ldp     x2, x3, [sp], #16
    ldp     x0, x1, [sp], #16
    eret

invalid_exception 2, 1    // FIQ
invalid_exception 3, 1    // SError

// =============================================================================
// Lower EL using AArch64 (user space exceptions)
// =============================================================================
.balign 0x80
lower_el_aarch64_sync:
    // Save caller-saved registers
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x29, [sp, #-16]!
    stp     x30, xzr, [sp, #-16]!

    // Call Rust handler
    bl      handle_sync_exception

    // Restore registers
    ldp     x30, xzr, [sp], #16
    ldp     x18, x29, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x8, x9, [sp], #16
    ldp     x6, x7, [sp], #16
    ldp     x4, x5, [sp], #16
    ldp     x2, x3, [sp], #16
    ldp     x0, x1, [sp], #16
    eret

.balign 0x80
lower_el_aarch64_irq:
    // Save caller-saved registers
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x29, [sp, #-16]!
    stp     x30, xzr, [sp, #-16]!

    // Call Rust handler
    bl      handle_irq

    // Restore registers
    ldp     x30, xzr, [sp], #16
    ldp     x18, x29, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x8, x9, [sp], #16
    ldp     x6, x7, [sp], #16
    ldp     x4, x5, [sp], #16
    ldp     x2, x3, [sp], #16
    ldp     x0, x1, [sp], #16
    eret

invalid_exception 2, 2    // FIQ
invalid_exception 3, 2    // SError

// =============================================================================
// Lower EL using AArch32 (not used - we don't support 32-bit user space)
// =============================================================================
invalid_exception 0, 3    // Synchronous
invalid_exception 1, 3    // IRQ
invalid_exception 2, 3    // FIQ
invalid_exception 3, 3    // SError

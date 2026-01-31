#!/bin/bash
set -e

# Build the kernel for QEMU virt
cargo build --target aarch64-unknown-none --features virt -p kernel

# Run in QEMU
timeout 5s qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a57 \
    -nographic \
    -kernel target/aarch64-unknown-none/debug/kernel \
    -d guest_errors,unimp \
    -semihosting

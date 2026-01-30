#![no_std]
#![no_main]

use core::panic::PanicInfo;
use log::{error, info};

#[cfg(target_arch = "riscv64")]
use riscv::asm::wfi;

// SAFETY: This is the kernel entry point for RISC-V, exported with a specific name
// so the bootloader can find it. The bootloader guarantees the hardware is in a
// valid state upon entry.
#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    info!("axiom-ebpf RISC-V Kernel");
    info!("=======================");

    // Initialize basic kernel subsystems
    kernel::log::init();

    info!("Log system initialized");
    info!("Memory management: TODO");
    info!("Interrupt handling: TODO");
    info!("Device drivers: TODO");

    info!("Kernel initialization complete");
    info!("Entering idle loop...");

    loop {
        // SAFETY: wfi (wait for interrupt) is safe to execute in the idle loop.
        unsafe { wfi() };
    }
}

#[panic_handler]
#[cfg(not(test))]
fn rust_panic(info: &PanicInfo) -> ! {
    error!("KERNEL PANIC!");
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{}:{}",
            location.file(),
            location.line(),
            location.column(),
        );
    }
    error!("{}", info.message());
    
    loop {
        #[cfg(target_arch = "riscv64")]
        // SAFETY: wfi (wait for interrupt) is safe to execute in the panic loop
        // to save power while halting the CPU.
        unsafe { wfi(); }
    }
}

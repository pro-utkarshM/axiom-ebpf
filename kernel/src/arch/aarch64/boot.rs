/// Boot information passed from bootloader
pub struct BootInfo {
    pub dtb_addr: usize,
}

static mut BOOT_INFO: BootInfo = BootInfo { dtb_addr: 0 };

/// Initialize boot information
///
/// # Safety
/// The caller must ensure that `dtb_addr` is a valid physical address.
pub unsafe fn init_boot_info(dtb_addr: usize) {
    // SAFETY: We are writing to the static BOOT_INFO. This is safe because:
    // 1. We are in early boot (single core)
    // 2. interrupts are disabled
    // 3. This function is only called once from _start
    unsafe {
        BOOT_INFO.dtb_addr = dtb_addr;
    }
}

/// Get boot information
#[allow(clippy::deref_addrof)]
pub fn boot_info() -> &'static BootInfo {
    // SAFETY: BOOT_INFO is initialized in _start before any other code runs.
    // It is effectively read-only after initialization.
    unsafe { &*(&raw const BOOT_INFO) }
}

/// Early boot initialization (called from assembly)
///
/// # Safety
/// This function is the kernel entry point and expects to be called with MMU disabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start(dtb_addr: usize) -> ! {
    // Initialize boot info
    // SAFETY: This is the first thing we do. dtb_addr is passed in x0 from the bootloader.
    unsafe {
        init_boot_info(dtb_addr);
    }

    // BSS is already cleared by assembly, but we define the symbols
    // for reference
    unsafe extern "C" {
        static __bss_start: u8;
        static __bss_end: u8;
    }

    // Initialize platform-specific hardware (UART, etc.)
    #[cfg(feature = "rpi5")]
    super::platform::rpi5::init();

    // Parse device tree to get memory information
    // SAFETY: dtb_addr is guaranteed to be a valid physical address by the bootloader protocol.
    if let Err(e) = unsafe { super::dtb::parse(dtb_addr) } {
        // Log error but continue - we can fall back to hardcoded values
        log::warn!("Failed to parse DTB: {}", e);
    }

    // Jump to kernel main
    unsafe extern "Rust" {
        fn kernel_main() -> !;
    }

    // SAFETY: We have initialized the minimal environment required for the kernel main.
    // This function never returns.
    unsafe { kernel_main() }
}

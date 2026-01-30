use x86_64::instructions::port::Port;

pub fn shutdown() -> ! {
    let mut port = Port::new(0xf4);
    // SAFETY: We are writing to the QEMU/KVM debug exit port to shut down the system.
    // This is the standard way to trigger a shutdown in QEMU.
    unsafe {
        port.write(0x00_u32);
    }
    loop {
        x86_64::instructions::hlt();
    }
}

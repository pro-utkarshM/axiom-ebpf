#![no_std]
#![no_main]

use minilib::{exit, write};

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let bytes = b"hello from init!\n";
    write(1, bytes);
    
    use minilib::bpf;
    use kernel_abi::BpfAttr;

    write(1, b"Loading dynamic BPF program...\n");

    #[repr(C)]
    struct BpfInsn {
        code: u8,
        dst_src: u8,
        off: i16,
        imm: i32,
    }
    
    let insns = [
        BpfInsn { code: 0xb7, dst_src: 0x00, off: 0, imm: 42 }, // r0 = 42
        BpfInsn { code: 0x95, dst_src: 0x00, off: 0, imm: 0 },  // exit
    ];
    
    let mut attr = BpfAttr::default();
    attr.prog_type = 1; 
    attr.insn_cnt = 2;
    attr.insns = insns.as_ptr() as u64;
    
    let attr_ptr = &attr as *const BpfAttr as *const u8;
    
    let res = bpf(5, attr_ptr, core::mem::size_of::<BpfAttr>() as i32);

    if res >= 0 {
         write(1, b"BPF program loaded successfully!\n");
    } else {
         write(1, b"Failed to load BPF program\n");
    }

    exit(0);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    loop {}
}

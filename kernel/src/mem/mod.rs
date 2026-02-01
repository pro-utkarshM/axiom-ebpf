use log::info;

#[cfg(target_arch = "x86_64")]
use crate::limine::MEMORY_MAP_REQUEST;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use crate::mem::address_space::AddressSpace;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use crate::mem::heap::Heap;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod address_space;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod heap;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod memapi;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod phys;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod virt;

#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_panics_doc)]
pub fn init() {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .expect("should have a memory map response");

    let usable_physical_memory = phys::init_stage1(response.entries());

    address_space::init();

    let address_space = AddressSpace::kernel();

    heap::init(address_space, usable_physical_memory);

    virt::init();

    phys::init_stage2();

    heap::init_stage2();

    info!("memory initialized, {Heap:x?}");
}

#[cfg(target_arch = "aarch64")]
#[allow(clippy::missing_panics_doc)]
pub fn init() {
    use crate::arch::aarch64::mm;
    use crate::arch::aarch64::phys;

    info!("Starting memory initialization...");
    mm::init();

    // Stage 1 phys alloc already initialized in mm::init()
    let usable_physical_memory = phys::total_memory();
    info!("Usable physical memory: {} MB", usable_physical_memory / 1024 / 1024);

    address_space::init();
    info!("Address space initialized");

    let address_space = AddressSpace::kernel();

    heap::init(address_space, usable_physical_memory);
    info!("Heap initialized");

    virt::init();
    info!("Virtual memory initialized");

    info!("memory initialized via arch::mm::init");
    info!("heap info: {Heap:x?}");

    // Initialize stage 2 physical allocator (requires heap)
    phys::init_stage2();
    info!("Physical memory stage 2 initialized");

    // We need to call heap::init_stage2 if we want dynamic resizing.
    heap::init_stage2();
    info!("Heap stage 2 initialized");
}

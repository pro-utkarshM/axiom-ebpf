mod block;
#[cfg(target_arch = "x86_64")]
mod gpu;
mod hal;
#[cfg(all(target_arch = "aarch64", feature = "virt"))]
pub mod mmio;

//! Platform-specific code for AArch64 boards
//!
//! Each platform (Raspberry Pi 5, QEMU virt, etc.) has its own module
//! with board-specific drivers and initialization.

#[cfg(feature = "rpi5")]
pub mod rpi5;

#[cfg(all(feature = "rpi5", not(feature = "virt")))]
pub use rpi5::*;

// If both are enabled, prefer rpi5 for top-level exports to avoid conflicts,
// but ensure the virt module is still available for explicit usage.
// Actually, if we want to support multi-platform binary, we shouldn't glob re-export at all.
// But for now, let's stick to compile-time selection.

#[cfg(feature = "virt")]
pub mod virt;

#[cfg(all(feature = "virt", not(feature = "rpi5")))]
pub use virt::*;

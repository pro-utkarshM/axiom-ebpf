# Proposal: Porting Muffin OS to Raspberry Pi 5 with GPIO Support (Revised)

**Author:** utkarsh  
**Date:** December 22, 2025  
**Version:** 2.0

---

## 1. Introduction

This document outlines a revised, detailed plan for porting the **Muffin OS** to the **Raspberry Pi 5**. The primary goal is to achieve a successful bare-metal boot and implement a basic GPIO driver. This updated proposal addresses the critical architectural differences of the Pi 5, specifically the introduction of the **RP1 I/O controller (southbridge)**, which manages peripherals over a PCIe bus.

This is a significant departure from previous Raspberry Pi models and requires a different approach to driver development. This proposal provides a more accurate and technically sound checklist for a developer experienced in systems programming.

---

## 2. Project Goals and Scope

### Primary Objectives

1.  **PCIe Initialization:** Successfully initialize the PCIe bus on the BCM2712 to communicate with the RP1 chip.
2.  **Bare-Metal Boot:** Boot a modified Muffin OS kernel on the Pi 5, executing in AArch64 EL1.
3.  **GPIO Driver:** Develop a GPIO driver that accesses the RP1 controller through the PCIe bus.
4.  **LED Control:** Create a kernel application to blink an LED using the new driver.
5.  **Serial Console:** Implement a minimal UART driver for debugging via the RP1.

### Out of Scope

-   Full POSIX compliance on ARM.
-   Advanced interrupt handling (beyond basic timer/GPIO).
-   Filesystem or block device support.
-   Multi-core support.
-   Userspace program execution.

---

## 3. Hardware and Software Requirements

(No changes from Version 1.0)

| Category      | Item                                                                 | Purpose                                                    |
|---------------|----------------------------------------------------------------------|------------------------------------------------------------|
| **Hardware**  | Raspberry Pi 5 (any memory variant)                                  | Target device for the OS port.                             |
|               | MicroSD Card (16GB or larger)                                        | Storage for the bootloader and kernel image.               |
|               | USB-C Power Supply (5V/5A recommended)                               | Powering the Raspberry Pi 5.                             |
|               | LED and a resistor (e.g., 330Ω)                                      | Hardware for the GPIO test.                                |
|               | Breadboard and jumper wires                                          | Connecting the LED to the GPIO pins.                       |
|               | USB-to-TTL Serial Cable (e.g., PL2303, CP2102)                        | Essential for viewing kernel debug output.                 |
| **Software**  | Rust Nightly toolchain                                               | Muffin OS is built on nightly Rust features.               |
|               | `aarch64-none-elf` cross-compilation target                          | To build the kernel for the ARM64 architecture.            |
|               | QEMU (system-aarch64)                                                | Optional, for early-stage emulation and testing.           |
|               | Raspberry Pi firmware files (`bootcode.bin`, `start.elf`)            | Required for the Pi's boot process.                        |

---

## 4. The Raspberry Pi 5 Architecture: BCM2712 and the RP1 Southbridge

The most critical architectural change in the Raspberry Pi 5 is the move to a two-chip solution, which fundamentally alters how peripherals are accessed in a bare-metal environment.

-   **BCM2712 (AP):** The main Application Processor, a 16nm Broadcom SoC, contains the quad-core ARM Cortex-A76 CPU.
-   **RP1 (I/O Controller):** A separate chip, designed in-house by Raspberry Pi, that acts as a **southbridge**. It connects to the BCM2712 via a **PCIe 2.0 x4 bus** [1].

**All traditional peripherals—including GPIO, UART, SPI, and I2C—are located on the RP1 chip, not on the BCM2712.** This means they cannot be accessed by simply writing to a memory-mapped address on the main SoC. Instead, the kernel must first initialize the PCIe bus, discover the RP1, and then communicate with its peripherals through the PCIe address space.

Access to the RP1's peripherals is managed through **Base Address Registers (BARs)** exposed over the PCIe bus. The RP1 datasheet indicates that **BAR1** maps the peripheral region [1].

---

## 5. Implementation Checklist (Revised)

This revised checklist accounts for the RP1 architecture.

### Phase 1: Environment Setup

(No changes from Version 1.0 - setup remains the same)

### Phase 2: Kernel Modifications for RP1 Architecture

1.  **Create a Platform Module:**
    -   Inside `kernel/src/arch/aarch64`, create a `platform/rpi5` module to hold all Pi 5-specific code.

2.  **Implement a Basic PCIe Driver:**
    -   **Goal:** Enumerate the PCIe bus and find the RP1 controller.
    -   **Action:** This is the most complex new step. You will need to:
        1.  Research the BCM2712's PCIe controller registers.
        2.  Write code to scan the PCIe bus for devices.
        3.  Identify the RP1 by its Vendor and Device ID.
        4.  Read the RP1's BARs to find the base address for its peripheral memory space.
    -   This step is a prerequisite for all other drivers.

3.  **Implement a UART Driver (via RP1):**
    -   **Goal:** Get `printk!` working for debug output.
    -   **Action:**
        1.  Using the peripheral base address from the PCIe BAR, calculate the virtual address for the UART peripheral.
        2.  The RP1 datasheet specifies the offset for `uart0` is `0x40030000` within its local address space [1]. Your driver will access it at `BAR1_VIRTUAL_ADDRESS + 0x40030000`.
        3.  Implement functions to initialize the UART and write characters to the correct registers at this new address.
        4.  Hook this driver into a global `WRITER`.

4.  **Adapt the Boot Process:**
    -   (No changes from Version 1.0 - the kernel entry point and DTB handling remain the same).

5.  **Implement a GPIO Driver (via RP1):**
    -   **Goal:** Control a GPIO pin.
    -   **Action:**
        1.  Create a `gpio.rs` file in the platform module.
        2.  Using the peripheral base address from the PCIe BAR, define the base address for the GPIO controller.
        3.  Implement functions to set a pin's function and state by writing to the correct registers relative to the BAR address.

6.  **Develop the Main Kernel Logic:**
    -   (Largely the same as Version 1.0, but now calls the new RP1-aware drivers).

### Phase 3: Build and Deployment

(No changes from Version 1.0 - the process of creating and deploying `kernel8.img` is the same).

### Phase 4: Testing

(No changes from Version 1.0 - the hardware setup and verification method are the same).

---

## 6. Risks and Mitigation (Revised)

| Risk                               | Mitigation                                                                                                                              |
|------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| **PCIe Initialization Complexity** | **(New)** This is the highest risk. The process is complex and requires deep understanding of the PCIe protocol. Rely heavily on the Linux kernel source for the Pi 5, U-Boot source, and community reverse-engineering efforts [2]. Start with simple bus enumeration before attempting full configuration. |
| **Incomplete RP1 Documentation**   | The official RP1 datasheet is marked as a draft [1]. Details may be missing or incorrect. Supplement with community-driven reverse-engineering efforts and be prepared for trial-and-error. |
| **MMU and Caching Issues**         | Bare-metal MMU setup is complex. Initially, run with the MMU disabled or use a simple identity mapping for the PCIe address space. |

---

## 7. Conclusion

The discovery of the RP1 southbridge architecture significantly increases the complexity of porting Muffin OS to the Raspberry Pi 5. The original proposal, while structurally sound, was based on an incorrect architectural assumption. This revised proposal provides a more accurate and realistic path forward.

Success now hinges on the challenging first step of initializing the PCIe bus and communicating with the RP1. By tackling this head-on, the project can proceed on a solid foundation, leading to a successful boot and the implementation of the desired GPIO functionality.

---

## 8. References (Revised)

[1] Raspberry Pi. (2023). *RP1 Peripherals Datasheet*. [https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf](https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf)

[2] G33KatWork. *RP1-Reverse-Engineering*. GitHub Repository. [https://github.com/G33KatWork/RP1-Reverse-Engineering](https://github.com/G33KatWork/RP1-Reverse-Engineering)

Engineering)


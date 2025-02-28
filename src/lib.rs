/*
 * DOS Emulator Library
 * ===================
 *
 * This library provides the core functionality for the DOS emulator,
 * including system components and emulation logic.
 *
 * Components:
 * ----------
 * - Memory Management (RAM/ROM)
 * - CPU Emulation
 * - BIOS Implementation
 * - Disk System
 * - DMA Controller
 * - Serial Interface
 */

pub mod bios;
pub mod cpu;
pub mod disk;
pub mod dma;
pub mod memory;
pub mod rom;
pub mod serial;

// Re-export commonly used types
pub use bios::{init_bios_data_area, init_bios_interrupts};
pub use cpu::Cpu;
pub use disk::DiskImage;
pub use memory::SystemMemory;
pub use serial::Serial; 
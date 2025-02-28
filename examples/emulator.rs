/*
 * DOS Emulator Example
 * ===================
 *
 * This example demonstrates how to use the DOS emulator library to create
 * a complete emulation environment.
 */

use dos_emu::{
    init_bios_data_area, init_bios_interrupts,
    Cpu, DiskImage, Serial, SystemMemory,
    disk::PARTITION_TABLE_OFFSET,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create disk image
    let drive_c_path = PathBuf::from("drive_c");
    let disk = DiskImage::new(&drive_c_path)?;

    // Verify MBR has boot code - fail if not present
    let mbr_sector = match disk.read_sector(0) {
        Some(data) => data,
        None => {
            eprintln!("Failed to read MBR");
            std::process::exit(1);
        }
    };
    if mbr_sector.len() != 512 {
        eprintln!("Invalid MBR sector size");
        std::process::exit(1);
    }

    // Check for valid boot code (at least some non-zero bytes in boot code area)
    let has_boot_code = mbr_sector[0..PARTITION_TABLE_OFFSET]
        .iter()
        .any(|&byte| byte != 0);

    if !has_boot_code {
        eprintln!("MBR boot code not present");
        std::process::exit(1);
    }

    // Create memory with ROM and RAM
    let memory = SystemMemory::new(1024 * 1024); // 1MB RAM

    // Initialize CPU with memory and serial port
    let mut cpu = Cpu::new(Box::new(memory), Serial::new(), disk);

    // Initialize BIOS
    init_bios_interrupts(&mut cpu);
    init_bios_data_area(&mut cpu);

    // Verify ROM is valid before starting
    if !cpu.has_valid_rom() {
        return Err("No valid ROM loaded".into());
    }

    // Set initial state to start at BIOS reset vector
    cpu.regs.cs = 0xF000;
    cpu.regs.ip = 0xFFF0;
    cpu.regs.ds = 0x0000;
    cpu.regs.es = 0x0000;
    cpu.regs.ss = 0x0000;
    cpu.regs.sp = 0x7C00;

    // Run CPU
    loop {
        if cpu.is_halted() {
            println!("CPU halted normally");
            break;
        }

        if let Err(e) = cpu.execute_instruction() {
            println!("CPU error: {}", e);
            break;
        }
    }

    Ok(())
} 
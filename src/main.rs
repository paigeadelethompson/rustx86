/*
 * DOS Emulator Main Entry Point
 * ============================
 * 
 * This is the main entry point for the DOS emulator. It handles system initialization
 * and CPU execution while delegating all disk operations to the disk module.
 * 
 * Architecture:
 * ------------
 * 1. System Components:
 *    - Memory (1MB RAM)
 *    - BIOS ROM
 *    - Serial Interface
 *    - Disk System
 *    - CPU
 * 
 * 2. Initialization Flow:
 *    - Initialize memory and peripherals
 *    - Set up disk system with drive_c path
 *    - Load BIOS ROM
 *    - Initialize CPU with all components
 *    - Set up BIOS interrupts and data area
 * 
 * 3. Disk Handling:
 *    All disk operations are handled by the disk module, including:
 *    - MBR loading and execution
 *    - Partition table management
 *    - Boot sector loading
 *    - FAT filesystem middleware
 * 
 * 4. Execution:
 *    The main loop runs the CPU until either:
 *    - The CPU halts normally
 *    - An error occurs
 *    - Maximum cycle count is reached
 * 
 * This design separates core emulation from disk handling, allowing the disk
 * module to manage all filesystem interactions independently.
 */

mod cpu;
mod memory;
mod rom;
mod disk;
mod serial;
mod dma;
mod bios;

use std::path::PathBuf;
use cpu::CPU;
use memory::Memory;
use serial::Serial;
use disk::DiskImage;
use rom::BiosRom;
use std::io::{self, Read, Write};
use std::time::Duration;

fn main() -> Result<(), String> {
    // println!("Starting DOS emulator...");
    
    // Initialize components
    let mut memory = Memory::new(1024 * 1024); // 1MB RAM
    let serial = Serial::new();
    
    // Load BIOS ROM first
    let bios_rom = BiosRom::new();
    memory.load_rom(bios_rom);
    
    // Initialize disk with drive_c path
    let drive_c_path = PathBuf::from("drive_c");
    let mut disk = DiskImage::new(&drive_c_path)
        .map_err(|e| format!("Failed to initialize disk: {}", e))?;

    // Load MBR (sector 0) into memory at 0x7C00
    let mbr = disk.read_sector(0)
        .ok_or("Failed to read MBR")?;
    memory.load_boot_sector(mbr);

    // Initialize CPU with memory that already has ROM loaded
    let mut cpu = CPU::new(memory, serial, disk);

    // Initialize BIOS interrupts and data area
    bios::init_bios_interrupts(&mut cpu);
    bios::init_bios_data_area(&mut cpu);

    // Now reset the CPU to start at F000:FFF0
    cpu.reset();

    // println!("\n=== Starting CPU Execution ===");
    // println!("Initial CS:IP = F000:FFF0");

    // Set up stdin for non-blocking reads
    let mut stdin = io::stdin();
    let _ = termios::Termios::from_fd(0).map(|mut t| {
        t.c_lflag &= !(termios::ICANON | termios::ECHO);
        t.c_cc[termios::VMIN] = 0;
        t.c_cc[termios::VTIME] = 0;
        let _ = termios::tcsetattr(0, termios::TCSANOW, &t);
    });

    // Run CPU for fixed number of cycles
    let max_cycles = 0;
    let mut cycles = 0;
    
    loop {
        // Check for input
        let mut buf = [0; 1];
        if stdin.read_exact(&mut buf).is_ok() {
            cpu.serial.add_input(buf[0]);
        }

        // Process any pending output
        while let Some(byte) = cpu.serial.get_output() {
            print!("{}", byte as char);
            let _ = std::io::stdout().flush().unwrap();
        }

        match cpu.run() {
            Ok(_) => {
                if cpu.is_halted() {
                    break;
                }
            }
            Err(e) => {
                println!("\nCPU error: {}", e);
                break;
            }
        }
        
        if max_cycles != 0 {
            cycles += 1;
        }

        if cycles > max_cycles {
            println!("\nReached maximum cycles ({})", max_cycles);
            break;
        }

        // Small delay to prevent CPU from consuming too many resources
        std::thread::sleep(Duration::from_micros(100));
    }
    
    Ok(())
} 
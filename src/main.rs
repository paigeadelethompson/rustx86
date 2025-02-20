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

fn main() -> Result<(), String> {
    println!("Starting DOS emulator...");
    
    // Initialize components
    let mut memory = Memory::new(1024 * 1024); // 1MB RAM
    let serial = Serial::new();
    
    // Initialize disk with drive_c path
    let drive_c_path = PathBuf::from("drive_c");
    let disk = DiskImage::new(&drive_c_path)
        .map_err(|e| format!("Failed to initialize disk: {}", e))?;

    // Load BIOS ROM
    let bios_rom = BiosRom::new();
    memory.load_rom(bios_rom.as_slice());

    // Initialize CPU
    let mut cpu = CPU::new(memory, serial, disk);
    
    // Initialize BIOS interrupts and data area
    bios::init_bios_interrupts(&mut cpu);
    bios::init_bios_data_area(&mut cpu);

    println!("Initial CPU state - CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);

    // Run CPU until halted or max cycles reached
    let max_cycles = -1; // -1 for infinite execution
    let mut cycles = 0;
    
    loop {
        let cs = cpu.regs.cs;
        let ip = cpu.regs.ip;
        let physical_addr = ((cs as u32) << 4) + (ip as u32);
        
        if max_cycles > 0 {
            println!("\nCycle {}: CS:IP={:04X}:{:04X}, Physical={:05X}", 
                     cycles, cs, ip, physical_addr);
        } else {
            println!("\nCS:IP={:04X}:{:04X}, Physical={:05X}", 
                     cs, ip, physical_addr);
        }
        
        match cpu.run() {
            Ok(_) => {
                // Log register state
                println!("AX={:04X} BX={:04X} CX={:04X} DX={:04X}", 
                         cpu.regs.ax, cpu.regs.bx, cpu.regs.cx, cpu.regs.dx);
                println!("SI={:04X} DI={:04X} BP={:04X} SP={:04X}", 
                         cpu.regs.si, cpu.regs.di, cpu.regs.bp, cpu.regs.sp);
                println!("Flags: {:?}", cpu.regs.flags);
                
                if cpu.is_halted() {
                    if max_cycles > 0 {
                        println!("\nCPU halted normally after {} cycles", cycles);
                    } else {
                        println!("\nCPU halted normally");
                    }
                    break;
                }
            }
            Err(e) => {
                if max_cycles > 0 {
                    println!("\nCPU error after {} cycles: {}", cycles, e);
                } else {
                    println!("\nCPU error: {}", e);
                }
                break;
            }
        }
        
        if max_cycles > 0 {
            cycles += 1;
            if cycles >= max_cycles {
                println!("\nReached maximum cycles ({}). Final state:", max_cycles);
                println!("CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);
                println!("Physical address = {:05X}", ((cpu.regs.cs as u32) << 4) + (cpu.regs.ip as u32));
                break;
            }
        }
    }
    
    Ok(())
} 
// IMPORTANT: This is a HEADLESS emulator that uses serial port for TTY output.
// DO NOT implement video/graphics functionality. All output goes through serial port.

use crate::cpu::CPU;
use crate::disk::DiskImage;
use chrono::{Datelike, Timelike};
use std::io::{self, Write};

const SERIAL_PORT: u16 = 0x3F8; // COM1 port

pub fn init_bios_interrupts(cpu: &mut CPU) {
    // Initialize interrupt vector table at 0x0000
    
    // INT 14h - Serial Services (Primary TTY output)
    set_interrupt_vector(cpu, 0x14, bios_seg(), serial_services_offset());
    
    // INT 13h - Disk Services
    set_interrupt_vector(cpu, 0x13, bios_seg(), disk_services_offset());
    
    // INT 16h - Keyboard Services
    set_interrupt_vector(cpu, 0x16, bios_seg(), keyboard_services_offset());
}

fn set_interrupt_vector(cpu: &mut CPU, int_num: u8, segment: u16, offset: u16) {
    let addr = (int_num as u32) * 4;
    cpu.memory.write_word(addr, offset);
    cpu.memory.write_word(addr + 2, segment);
}

fn bios_seg() -> u16 {
    0xF000
}

fn video_services_offset() -> u16 {
    0xE300  // Point to our video interrupt handler in ROM
}

fn serial_services_offset() -> u16 {
    0xE000  // Point to our serial interrupt handler in ROM
}

fn keyboard_services_offset() -> u16 {
    0x0100 // We'll implement this later
}

fn disk_services_offset() -> u16 {
    0x0200 // We'll implement this later
}

pub fn handle_bios_interrupt(cpu: &mut CPU, int_num: u8) -> Result<(), String> {
    match int_num {
        0x10 => handle_video_interrupt(cpu),
        0x14 => handle_serial_interrupt(cpu),
        0x13 => handle_disk_interrupt(cpu),
        0x16 => handle_keyboard_interrupt(cpu),
        0x11 => { cpu.int11_equipment_list()?; Ok(()) },    // Get Equipment List
        0x12 => { cpu.int12_memory_size()?; Ok(()) },       // Get Memory Size
        0x15 => { cpu.int15_system_services()?; Ok(()) },   // System Services
        0x1A => { cpu.int1a_time_services()?; Ok(()) },     // Time Services
        _ => Err(format!("Unhandled BIOS interrupt: {:02X}", int_num)),
    }
}

fn handle_video_interrupt(cpu: &mut CPU) -> Result<(), String> {
    match cpu.regs.get_ah() {
        0x0E => {
            // Redirect TTY output to serial port
            let char = cpu.regs.get_al();
            print!("{}", char as char);
            std::io::stdout().flush().unwrap();
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_serial_interrupt(cpu: &mut CPU) -> Result<(), String> {
    match cpu.regs.get_ah() {
        0x00 => {
            // Initialize port
            println!("Serial INT 14h: Initialize port");
            // For now, just return success
            cpu.regs.set_ah(0);
            Ok(())
        }
        0x01 => {
            // Send character
            let char = cpu.regs.get_al();
            println!("Serial INT 14h: Send character '{}' (0x{:02X})", char as char, char);
            print!("{}", char as char);
            std::io::stdout().flush().unwrap();
            cpu.regs.set_ah(0); // Success
            Ok(())
        }
        _ => {
            println!("Serial INT 14h: Unhandled function AH={:02X}", cpu.regs.get_ah());
            Ok(())
        }
    }
}

fn handle_keyboard_interrupt(cpu: &mut CPU) -> Result<(), String> {
    match cpu.regs.get_ah() {
        0x00 => {
            // Read character
            cpu.regs.set_al(0); // Return null for now
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_disk_interrupt(cpu: &mut CPU) -> Result<(), String> {
    match cpu.regs.get_ah() {
        0x00 => {
            // Reset disk system
            cpu.regs.set_ah(0); // Success
            Ok(())
        }
        _ => {
            cpu.regs.set_ah(1); // Error
            Ok(())
        }
    }
}

impl CPU {
    fn int11_equipment_list(&mut self) -> Result<(), String> {
        self.regs.ax = BIOS_EQUIPMENT_LIST;
        Ok(())
    }

    fn int12_memory_size(&mut self) -> Result<(), String> {
        self.regs.ax = BIOS_MEMORY_SIZE;
        Ok(())
    }

    fn int15_system_services(&mut self) -> Result<(), String> {
        match self.regs.ax >> 8 {
            0x87 => {
                // Block move
                // Often used by DOS for extended memory operations
                // For now, return error as we don't support extended memory
                self.regs.flags.set_carry(true);
            }
            0x88 => {
                // Get extended memory size
                // Return 0 as we don't support extended memory yet
                self.regs.ax = 0;
            }
            0x89 => {
                // Switch to protected mode
                // Return error as we don't support protected mode yet
                self.regs.flags.set_carry(true);
            }
            _ => {
                // Unsupported function
                self.regs.flags.set_carry(true);
            }
        }
        Ok(())
    }

    fn int1a_time_services(&mut self) -> Result<(), String> {
        match self.regs.get_ah() {
            0x00 => {
                // Read system clock counter
                let ticks = (self.cycles / 54945) as u32; // ~18.2 Hz tick rate
                self.regs.set_al(0); // Midnight flag
                self.regs.set_cx((ticks >> 16) as u16);
                self.regs.set_dx(ticks as u16);
                self.regs.flags.set_carry(false);
                Ok(())
            }
            0x02 => {
                let now = chrono::Local::now();
                // Set hours and minutes
                self.regs.set_ch(now.hour() as u8);
                self.regs.set_cl(now.minute() as u8);
                // Set month and day
                self.regs.set_dh(now.month() as u8);
                self.regs.set_dl(now.day() as u8);
                self.regs.flags.set_carry(false);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn int16_keyboard_services(&mut self) -> Result<(), String> {
        match self.regs.ax >> 8 {
            0x00 => {
                // Read keyboard character
                // For now, always return a carriage return
                self.regs.ax = 0x0D00;
            }
            0x01 => {
                // Check for keyboard character
                // For now, always indicate no character available
                self.regs.flags.set_zero(true);
            }
            0x02 => {
                // Get keyboard shift status
                self.regs.ax = (self.regs.ax & 0xFF00) | 0x00;
            }
            _ => {
                self.regs.flags.set_carry(true);
            }
        }
        Ok(())
    }
}

// Add these constants for BIOS services
const BIOS_EQUIPMENT_LIST: u16 = 0b0000_0010_0000_0011; // Base memory, serial port, no display
const BIOS_MEMORY_SIZE: u16 = 640; // 640K conventional memory 

pub fn init_bios_data_area(cpu: &mut CPU) {
    // BIOS data area starts at 0x0400
    
    // Equipment list (serial ports only, no display)
    cpu.memory.write_word(0x0410, 0x0001);
    
    // Base memory size (640KB)
    cpu.memory.write_word(0x0413, 640);
    
    // COM port addresses
    cpu.memory.write_word(0x0400, 0x3F8); // COM1
    cpu.memory.write_word(0x0402, 0x2F8); // COM2
    cpu.memory.write_word(0x0404, 0x3E8); // COM3
    cpu.memory.write_word(0x0406, 0x2E8); // COM4
}

fn print_debug(cpu: &mut CPU, msg: &str) {
    // Print debug messages to stdout instead of serial port
    println!("{}", msg);
}

// Add error code constants
const ERR_SUCCESS: u8 = 0x00;
const ERR_INVALID_COMMAND: u8 = 0x01;
const ERR_ADDRESS_MARK: u8 = 0x02;
const ERR_WRITE_PROTECT: u8 = 0x03;
const ERR_SECTOR_NOT_FOUND: u8 = 0x04;
const ERR_RESET_FAILED: u8 = 0x05;
const ERR_DISK_CHANGED: u8 = 0x06;
const ERR_DRIVE_PARAMETER: u8 = 0x07;
const ERR_DMA_OVERRUN: u8 = 0x08;
const ERR_DMA_BOUNDARY: u8 = 0x09;
const ERR_BAD_SECTOR: u8 = 0x0A;
const ERR_BAD_TRACK: u8 = 0x0B;
const ERR_MEDIA_TYPE: u8 = 0x0C;
const ERR_INVALID_SECTORS: u8 = 0x0D;
const ERR_INVALID_DRIVE: u8 = 0x80;

fn check_dma_boundary(addr: u32, size: u32) -> bool {
    // Check if transfer crosses 64K boundary
    let end_addr = addr + size - 1;
    (addr & 0xFFFF0000) == (end_addr & 0xFFFF0000)
}

fn perform_dma_transfer(cpu: &mut CPU, channel: usize, buffer: &mut [u8], addr: u32, size: usize) -> bool {
    print_debug(cpu, &format!("DMA transfer: channel={}, addr={:08X}, size={:04X}\n", 
        channel, addr, size));

    if !check_dma_boundary(addr, size as u32) {
        print_debug(cpu, "DMA Error: Transfer would cross 64K boundary\n");
        return false;
    }

    if addr + (size as u32) > 0x100000 {
        print_debug(cpu, "DMA Error: Transfer beyond 1MB boundary\n");
        return false;
    }

    // Simulate DMA transfer by copying memory
    let src_slice = unsafe {
        std::slice::from_raw_parts(addr as *const u8, size)
    };
    buffer.copy_from_slice(src_slice);
    true
}

fn handle_time_interrupt(cpu: &mut CPU) -> Result<(), String> {
    match cpu.regs.get_ah() {
        0x00 => {
            // Get system time
            let now = chrono::Local::now();
            cpu.regs.set_al(0); // Midnight flag
            cpu.regs.set_cx((now.hour() << 8 | now.minute()) as u16);
            cpu.regs.set_dx((now.second() << 8) as u16);
            Ok(())
        }
        0x02 => {
            // Get real-time clock time
            let now = chrono::Local::now();
            cpu.regs.set_cx((now.hour() << 8 | now.minute()) as u16);
            cpu.regs.set_dx((now.second() << 8) as u16);
            Ok(())
        }
        0x04 => {
            // Get real-time clock date
            let now = chrono::Local::now();
            cpu.regs.set_cx(now.year() as u16);
            cpu.regs.set_dx(((now.month() as u16) << 8 | now.day() as u16));
            Ok(())
        }
        _ => Err(format!("Unhandled time interrupt function: {:02X}", cpu.regs.get_ah())),
    }
} 
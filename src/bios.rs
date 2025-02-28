// IMPORTANT: This is a HEADLESS emulator that uses serial port for TTY output.
// DO NOT implement video/graphics functionality. All output goes through serial port.

use crate::cpu::CPU;
use crate::disk::DiskImage;
use chrono::{Datelike, Timelike};
use std::io::{self, Write};
use crate::io::Error;

const SERIAL_PORT: u16 = 0x3F8; // COM1 port

pub fn init_bios_interrupts(cpu: &mut CPU) {
    // Initialize interrupt vector table at 0x0000
    
    // INT 10h - Video Services
    set_interrupt_vector(cpu, 0x10, bios_seg(), video_services_offset());
    
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
            let port = cpu.regs.get_dx() as u32;  // Convert to u32
            let divisor = 0x0C;  // 9600 baud
            
            // Set DLAB
            cpu.memory.write_byte(port + 3, 0x80);
            // Set divisor
            cpu.memory.write_byte(port, divisor as u8);
            cpu.memory.write_byte(port + 1, (divisor >> 8) as u8);
            // 8N1, clear DLAB
            cpu.memory.write_byte(port + 3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            cpu.memory.write_byte(port + 2, 0xC7);
            // IRQs enabled, RTS/DSR set
            cpu.memory.write_byte(port + 4, 0x0B);
            
            cpu.regs.set_ah(0);  // Success
            Ok(())
        }
        0x01 => {
            // Send character
            let char = cpu.regs.get_al() & 0xFF;  // Only use low byte
            cpu.serial.write_byte(0x3F8, char);
            cpu.regs.set_ah(0);  // Success
            Ok(())
        }
        0x02 => {
            // Read character
            let char = cpu.serial.read_byte(0x3F8);
            cpu.regs.set_al(char);
            cpu.regs.set_ah(0); // Success
            Ok(())
        }
        0x03 => {
            // Get port status
            let status = if cpu.serial.has_data() { 0x01 } else { 0x00 };
            cpu.regs.set_ah(status);
            Ok(())
        }
        _ => {
            Err(format!("Unhandled serial interrupt function: {:#04X}", cpu.regs.get_ah()))
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

pub fn handle_disk_interrupt(cpu: &mut CPU) -> Result<(), String> {
    let function = cpu.regs.get_ah();
    let drive = cpu.regs.get_dl();

    match function {
        0x00 => {
            // Reset Disk System
            cpu.regs.flags.set_carry(false); // Success
            Ok(())
        }
        0x02 => {
            // Read Sectors
            let sector = cpu.regs.get_cl() & 0x3F;
            let cylinder = ((cpu.regs.get_cl() & 0xC0) << 2) | cpu.regs.get_ch();
            let head = cpu.regs.get_dh();
            let count = cpu.regs.get_al();
            let buffer_segment = cpu.regs.get_es();
            let buffer_offset = cpu.regs.get_bx();

            // Calculate LBA = (cylinder * heads_per_cylinder + head) * sectors_per_track + (sector - 1)
            let heads_per_cylinder = 16;
            let sectors_per_track = 63;
            let lba = ((cylinder as u32 * heads_per_cylinder as u32 + head as u32) 
                      * sectors_per_track as u32 + (sector - 1) as u32) as u32;

            let mut success = true;
            for i in 0..count {
                if let Some(sector_data) = cpu.disk.read_sector(lba + i as u32) {
                    // Process sector data
                    let dest_addr = cpu.get_physical_address(buffer_segment, buffer_offset.wrapping_add((i as u16) * 512));
                    for (j, &byte) in sector_data.iter().enumerate() {
                        cpu.memory.write_byte(dest_addr + j as u32, byte);
                    }
                } else {
                    success = false;
                    break;
                }
            }

            if success {
                cpu.regs.set_al(count);
                cpu.regs.flags.set_carry(false); // Success
            } else {
                cpu.regs.set_ah(0x01); // Error
                cpu.regs.flags.set_carry(true); // Error
            }
            Ok(())
        }
        0xC0 => {
            // Get Drive Parameters
            if drive == 0x80 {
                // Hard disk parameters
                let max_cylinder = 1023;  // 0-1023 (1024 cylinders)
                let max_head = 15;        // 0-15 (16 heads)
                let max_sector = 63;      // 1-63 (63 sectors)
                let num_drives = 1;       // Number of hard drives

                // Return values
                cpu.regs.set_ah(0);           // Status (success)
                cpu.regs.set_al(0);           // Reserved (0)
                cpu.regs.set_bl(4);           // Drive type (4 = Fixed disk)
                cpu.regs.set_ch(max_cylinder as u8); // Low 8 bits of max cylinder number
                cpu.regs.set_cl(((max_cylinder >> 8) << 6) as u8 | max_sector); // High 2 bits of cylinder in bits 6-7, max sector in bits 0-5
                cpu.regs.set_dh(max_head);    // Maximum head number
                cpu.regs.set_dl(num_drives);  // Number of drives
                cpu.regs.flags.set_carry(false); // Success
            } else {
                cpu.regs.set_ah(0x01);        // Invalid drive
                cpu.regs.flags.set_carry(true); // Error
            }
            Ok(())
        }
        _ => {
            cpu.regs.flags.set_carry(true); // Error
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

pub fn print_debug(_cpu: &mut CPU, _msg: &str) {
    // Debug printing disabled
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
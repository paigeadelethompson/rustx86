mod registers;
mod flags;
mod instructions;
mod addressing;
mod execute;

pub use registers::Registers;
pub use flags::Flags;

use crate::memory::Memory;
use crate::serial::Serial;
use crate::disk::Disk;

pub struct CPU {
    pub regs: Registers,
    pub memory: Memory,
    pub serial: Serial,
    pub disk: Disk,
    pub cycles: u64,
    halted: bool,
}

impl CPU {
    pub fn new(memory: Memory, serial: Serial, disk: Disk) -> Self {
        CPU {
            memory,
            serial,
            disk,
            regs: Registers::new(),
            cycles: 0,
            halted: false,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }
        
        let cs_ip = ((self.regs.cs as u32) << 4) + (self.regs.ip as u32);
        let opcode = self.fetch_byte()?;
        println!("Executing at CS:IP={:04X}:{:04X} (Physical={:05X}), Opcode={:02X}", 
                 self.regs.cs, self.regs.ip.wrapping_sub(1), cs_ip, opcode);
        self.execute_instruction(opcode)?;
        Ok(())
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    fn fetch_byte(&mut self) -> Result<u8, String> {
        let addr = ((self.regs.cs as u32) << 4) + (self.regs.ip as u32);
        let byte = self.memory.read_byte(addr);
        println!("Fetched byte {:#04X} from {:#08X}", byte, addr);
        self.regs.ip = self.regs.ip.wrapping_add(1);
        Ok(byte)
    }

    fn fetch_word(&mut self) -> Result<u16, String> {
        let low = self.fetch_byte()? as u16;
        let high = self.fetch_byte()? as u16;
        Ok((high << 8) | low)
    }

    fn add_cycles(&mut self, _count: u64) {
        // Implementation of add_cycles method
    }

    fn sync_timing(&mut self) {
        // Implementation of sync_timing method
    }

    fn get_reg8(&mut self, reg: u8) -> Result<u8, String> {
        match reg {
            0 => Ok(self.regs.ax as u8),  // AL
            1 => Ok((self.regs.ax >> 8) as u8),  // AH
            2 => Ok(self.regs.bx as u8),  // BL
            3 => Ok((self.regs.bx >> 8) as u8),  // BH
            4 => Ok(self.regs.cx as u8),  // CL
            5 => Ok((self.regs.cx >> 8) as u8),  // CH
            6 => Ok(self.regs.dx as u8),  // DL
            7 => Ok((self.regs.dx >> 8) as u8),  // DH
            _ => Err(format!("Invalid 8-bit register: {}", reg)),
        }
    }

    fn set_reg8(&mut self, reg: u8, value: u8) -> Result<(), String> {
        match reg {
            0 => { // AL
                self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);
                Ok(())
            }
            1 => { // AH
                self.regs.ax = (self.regs.ax & 0x00FF) | ((value as u16) << 8);
                Ok(())
            }
            2 => { // BL
                self.regs.bx = (self.regs.bx & 0xFF00) | (value as u16);
                Ok(())
            }
            3 => { // BH
                self.regs.bx = (self.regs.bx & 0x00FF) | ((value as u16) << 8);
                Ok(())
            }
            4 => { // CL
                self.regs.cx = (self.regs.cx & 0xFF00) | (value as u16);
                Ok(())
            }
            5 => { // CH
                self.regs.cx = (self.regs.cx & 0x00FF) | ((value as u16) << 8);
                Ok(())
            }
            6 => { // DL
                self.regs.dx = (self.regs.dx & 0xFF00) | (value as u16);
                Ok(())
            }
            7 => { // DH
                self.regs.dx = (self.regs.dx & 0x00FF) | ((value as u16) << 8);
                Ok(())
            }
            _ => Err(format!("Invalid 8-bit register: {}", reg)),
        }
    }
} 
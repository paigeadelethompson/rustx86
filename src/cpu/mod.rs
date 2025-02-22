mod registers;
mod flags;
mod instructions;
mod addressing;
mod execute;

pub use registers::Registers;
pub use flags::Flags;

use crate::memory::Memory;
use crate::serial::Serial;
use crate::disk::DiskImage;

pub struct CPU {
    pub regs: Registers,
    pub memory: Memory,
    pub serial: Serial,
    pub disk: DiskImage,
    pub cycles: u64,
    halted: bool,
}

impl CPU {
    pub fn new(memory: Memory, serial: Serial, disk: DiskImage) -> Self {
        let mut cpu = CPU {
            memory,
            serial,
            disk,
            regs: Registers::new(),
            cycles: 0,
            halted: false,
        };
        cpu
    }

    pub fn reset(&mut self) {
        // Set initial CPU state
        self.regs.cs = 0xF000;  // Start at BIOS ROM segment
        self.regs.ip = 0xFFF0;  // BIOS reset vector
        self.regs.flags.value = 0x0002;  // Only reserved bit is set
        
        // Initialize segment registers
        self.regs.ds = 0;
        self.regs.es = 0;
        self.regs.ss = 0;
        
        // Initialize general purpose registers
        self.regs.ax = 0;
        self.regs.bx = 0;
        self.regs.cx = 0;
        self.regs.dx = 0;
        self.regs.si = 0;
        self.regs.di = 0;
        self.regs.bp = 0;
        self.regs.sp = 0;
        
        self.cycles = 0;
        self.halted = false;
    }

    pub fn run(&mut self) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }
        
        let cs_ip = ((self.regs.cs as u32) << 4) + (self.regs.ip as u32);
        let opcode = self.fetch_byte()?;
        // println!("\nExecuting at CS:IP={:04X}:{:04X} (Physical={:05X}), Opcode={:02X}", 
        //          self.regs.cs, self.regs.ip.wrapping_sub(1), cs_ip, opcode);
        // println!("AX={:04X} BX={:04X} CX={:04X} DX={:04X}", 
        //          self.regs.ax, self.regs.bx, self.regs.cx, self.regs.dx);
        // println!("SI={:04X} DI={:04X} BP={:04X} SP={:04X}", 
        //          self.regs.si, self.regs.di, self.regs.bp, self.regs.sp);
        // println!("Flags: {:?}", self.regs.flags);
        
        self.execute_instruction(opcode)?;
        Ok(())
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    fn fetch_byte(&mut self) -> Result<u8, String> {
        let physical_addr = ((self.regs.cs as u32) << 4) + (self.regs.ip as u32);
        let byte = self.memory.read_byte(physical_addr);
        // println!("Fetched byte {:#04X} from CS:IP={:04X}:{:04X} (Physical={:05X})", 
        //          byte, self.regs.cs, self.regs.ip, physical_addr);
        self.regs.ip = self.regs.ip.wrapping_add(1);
        Ok(byte)
    }

    fn fetch_word(&mut self) -> Result<u16, String> {
        let low = self.fetch_byte()? as u16;
        let high = self.fetch_byte()? as u16;
        let word = (high << 8) | low;
        // println!("Fetched word {:#06X}", word);
        Ok(word)
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
} 
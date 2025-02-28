pub mod flags;
pub mod registers;
pub mod instructions;
pub mod execute;

use std::fmt;
use crate::memory::Memory;
use crate::memory::SystemMemory;
use crate::serial::Serial;
use crate::disk::{DiskImage, PARTITION_TABLE_OFFSET};
pub use flags::Flags;
pub use registers::Registers;

pub struct CPU {
    pub regs: Registers,
    pub memory: Box<dyn Memory>,
    pub serial: Serial,
    pub disk: DiskImage,
    pub halted: bool,
    pub cycles: u64,
    pub segment_override: Option<SegmentRegister>,
    pub has_valid_mbr: bool,
    pub has_valid_boot_sector: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentRegister {
    CS,
    DS,
    ES,
    SS,
}

impl CPU {
    pub fn new(memory: Box<dyn Memory>, serial: Serial, disk: DiskImage) -> Self {
        // Check if disk has valid MBR boot code
        let mbr = match disk.read_sector(0) {
            Some(data) => data,
            None => vec![0; 512],
        };
        let has_valid_mbr = !mbr[..PARTITION_TABLE_OFFSET].iter().all(|&byte| byte == 0);

        // Check if disk has valid boot sector at LBA 63
        let boot = match disk.read_sector(63) {
            Some(data) => data,
            None => vec![0; 512],
        };
        let boot_valid = boot.len() == 512 && 
            boot[510] == 0x55 && boot[511] == 0xAA; // Must have valid boot signature

        CPU {
            memory,
            regs: Registers::new(),
            serial,
            disk,
            halted: false,
            cycles: 0,
            segment_override: None,
            has_valid_mbr,
            has_valid_boot_sector: boot_valid,
        }
    }

    pub fn has_valid_rom(&self) -> bool {
        // Check if memory is SystemMemory and has valid ROM code
        if let Some(sys_mem) = self.memory.as_any().downcast_ref::<SystemMemory>() {
            sys_mem.has_valid_rom()
        } else {
            false
        }
    }

    pub fn has_valid_mbr(&self) -> bool {
        self.has_valid_mbr
    }

    pub fn has_valid_boot_sector(&self) -> bool {
        self.has_valid_boot_sector
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.halted = false;
        self.cycles = 0;
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn run(&mut self) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }
        
        self.execute_instruction()
    }

    pub fn fetch_byte(&mut self) -> Result<u8, String> {
        let addr = self.get_physical_address(self.regs.cs, self.regs.ip);
        let byte = self.memory.read_byte(addr);
        self.regs.ip = self.regs.ip.wrapping_add(1);
        Ok(byte)
    }

    pub fn fetch_word(&mut self) -> Result<u16, String> {
        let addr = self.get_physical_address(self.regs.cs, self.regs.ip);
        let word = self.memory.read_word(addr);
        self.regs.ip = self.regs.ip.wrapping_add(2);
        Ok(word)
    }

    pub fn get_physical_address(&self, segment: u16, offset: u16) -> u32 {
        ((segment as u32) << 4) + (offset as u32)
    }

    // Helper functions used by instructions
    pub(crate) fn get_rm8(&mut self, modrm: u8) -> Result<u8, String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        if mod_bits == 3 {
            // Register operand
            Ok(self.regs.get_reg8(rm))
        } else {
            // Memory operand
            let addr = self.get_rm_addr(modrm)?;
            let segment = match rm {
                2 | 3 | 6 if mod_bits == 0 && rm == 6 => self.regs.ds, // Special case for direct address
                2 | 3 | 6 => self.regs.ss, // BP-based addressing uses SS
                _ => self.regs.ds, // Other cases use DS
            };
            Ok(self.memory.read_byte(self.get_physical_address(segment, addr as u16)))
        }
    }

    pub(crate) fn write_rm8(&mut self, modrm: u8, value: u8) -> Result<(), String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        if mod_bits == 3 {
            // Register operand
            self.regs.set_reg8(rm, value)?;
        } else {
            // Memory operand
            let addr = self.get_rm_addr(modrm)?;
            let segment = match rm {
                2 | 3 | 6 if mod_bits == 0 && rm == 6 => self.regs.ds, // Special case for direct address
                2 | 3 | 6 => self.regs.ss, // BP-based addressing uses SS
                _ => self.regs.ds, // Other cases use DS
            };
            self.memory.write_byte(self.get_physical_address(segment, addr as u16), value);
        }
        Ok(())
    }

    pub(crate) fn get_rm16(&mut self, modrm: u8) -> Result<u16, String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        if mod_bits == 3 {
            // Register operand
            Ok(self.regs.get_reg16(rm))
        } else {
            // Memory operand
            let addr = self.get_rm_addr(modrm)?;
            let segment = match rm {
                2 | 3 | 6 if mod_bits == 0 && rm == 6 => self.regs.ds, // Special case for direct address
                2 | 3 | 6 => self.regs.ss, // BP-based addressing uses SS
                _ => self.regs.ds, // Other cases use DS
            };
            Ok(self.memory.read_word(self.get_physical_address(segment, addr as u16)))
        }
    }

    pub(crate) fn write_rm16(&mut self, modrm: u8, value: u16) -> Result<(), String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        if mod_bits == 3 {
            // Register operand
            self.regs.set_reg16(rm, value)?;
        } else {
            // Memory operand
            let addr = self.get_rm_addr(modrm)?;
            let segment = match rm {
                2 | 3 | 6 if mod_bits == 0 && rm == 6 => self.regs.ds, // Special case for direct address
                2 | 3 | 6 => self.regs.ss, // BP-based addressing uses SS
                _ => self.regs.ds, // Other cases use DS
            };
            self.memory.write_word(self.get_physical_address(segment, addr as u16), value);
        }
        Ok(())
    }

    pub(crate) fn update_flags_sub(&mut self, a: u8, b: u8, result: u8, carry: bool) {
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(
            ((a ^ b) & (a ^ result) & 0x80) != 0
        );
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
    }

    pub(crate) fn update_flags_sub16(&mut self, a: u16, b: u16, result: u16, carry: bool) {
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(
            ((a ^ b) & (a ^ result) & 0x8000) != 0
        );
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
    }

    pub(crate) fn update_flags_inc16(&mut self, result: u16) {
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(result == 0x8000);
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
    }

    pub(crate) fn update_flags_dec16(&mut self, result: u16) {
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(result == 0x7FFF);
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
    }

    pub(crate) fn get_rm_addr(&mut self, modrm: u8) -> Result<u32, String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        let addr = match (mod_bits, rm) {
            (0, 6) => self.fetch_word()? as u32,
            (0, _) => self.get_rm_addr_mode0(rm)?,
            (1, _) => {
                let disp = self.fetch_byte()? as i8;
                self.get_rm_addr_mode0(rm)?.wrapping_add(disp as u32)
            },
            (2, _) => {
                let disp = self.fetch_word()? as i16;
                self.get_rm_addr_mode0(rm)?.wrapping_add(disp as u32)
            },
            _ => return Err("Invalid ModR/M addressing mode".to_string()),
        };

        Ok(addr)
    }

    fn get_rm_addr_mode0(&self, rm: u8) -> Result<u32, String> {
        let addr = match rm {
            0 => self.regs.bx.wrapping_add(self.regs.si) as u32,
            1 => self.regs.bx.wrapping_add(self.regs.di) as u32,
            2 => self.regs.bp.wrapping_add(self.regs.si) as u32,
            3 => self.regs.bp.wrapping_add(self.regs.di) as u32,
            4 => self.regs.si as u32,
            5 => self.regs.di as u32,
            6 => self.regs.bp as u32,
            7 => self.regs.bx as u32,
            _ => return Err("Invalid R/M value".to_string()),
        };
        Ok(addr)
    }

    pub(crate) fn update_flags_arithmetic(&mut self, op1: u8, op2: u8, result: u8, is_sub: bool) {
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result as i8) < 0);
        if is_sub {
            self.regs.flags.set_carry(op1 < op2);
            self.regs.flags.set_overflow(
                ((op1 ^ op2) & (op1 ^ result) & 0x80) != 0
            );
        } else {
            self.regs.flags.set_carry(result < op1);
            self.regs.flags.set_overflow(
                ((op1 ^ result) & (op2 ^ result) & 0x80) != 0
            );
        }
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
    }

    pub(crate) fn update_flags_arithmetic_16(&mut self, op1: u16, op2: u16, result: u16, is_sub: bool) {
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result as i16) < 0);
        if is_sub {
            self.regs.flags.set_carry(op1 < op2);
            self.regs.flags.set_overflow(
                ((op1 ^ op2) & (op1 ^ result) & 0x8000) != 0
            );
        } else {
            self.regs.flags.set_carry(result < op1);
            self.regs.flags.set_overflow(
                ((op1 ^ result) & (op2 ^ result) & 0x8000) != 0
            );
        }
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
    }

    pub(crate) fn push(&mut self, value: u16) -> Result<(), String> {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        let addr = self.get_physical_address(self.regs.ss, self.regs.sp);
        self.memory.write_word(addr, value);
        Ok(())
    }

    pub(crate) fn pop(&mut self) -> Result<u16, String> {
        let addr = self.get_physical_address(self.regs.ss, self.regs.sp);
        let value = self.memory.read_word(addr);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        Ok(value)
    }

    pub(crate) fn read_word(&mut self, addr: u32) -> Result<u16, String> {
        Ok(self.memory.read_word(addr))
    }

    pub(crate) fn write_word(&mut self, addr: u32, value: u16) -> Result<(), String> {
        self.memory.write_word(addr, value);
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }

        self.execute_instruction()?;
        self.cycles += 1;
        Ok(())
    }

    pub(crate) fn update_flags_inc(&mut self, operand: u16, result: u16) {
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
        self.regs.flags.set_overflow(operand == 0x7FFF);
        self.regs.flags.set_adjust((operand & 0xF) == 0xF);
    }

    pub fn set_segment_override(&mut self, segment: SegmentRegister) {
        self.segment_override = Some(segment);
    }

    pub fn clear_segment_override(&mut self) {
        self.segment_override = None;
    }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CPU {{ regs: {:?}, halted: {} }}", self.regs, self.halted)
    }
} 
use std::any::Any;
use super::Memory;

pub struct RamMemory {
    memory: Vec<u8>
}

impl RamMemory {
    pub fn new(size: usize) -> Self {
        RamMemory {
            memory: vec![0; size]
        }
    }
}

impl Memory for RamMemory {
    fn read_byte(&self, addr: u32) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, addr: u32, value: u8) {
        self.memory[addr as usize] = value;
    }

    fn read_word(&self, addr: u32) -> u16 {
        let low = self.read_byte(addr) as u16;
        let high = self.read_byte(addr + 1) as u16;
        (high << 8) | low
    }

    fn write_word(&mut self, addr: u32, value: u16) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    fn has_valid_rom(&self) -> bool {
        // RamMemory doesn't have ROM, so always return false
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 
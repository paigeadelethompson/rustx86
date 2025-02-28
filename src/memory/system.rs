use super::Memory;
use crate::rom::BiosRom;
use std::any::Any;

pub struct SystemMemory {
    ram: Vec<u8>,
    bios_rom: BiosRom,
}

impl SystemMemory {
    pub fn new(ram_size: usize) -> Self {
        let mut system = SystemMemory {
            ram: vec![0; ram_size],
            bios_rom: BiosRom::new(),
        };

        // Verify ROM code after initialization
        system.bios_rom.verify_rom_code();
        system
    }

    pub fn has_valid_rom(&self) -> bool {
        self.bios_rom.has_valid_code()
    }
}

impl Memory for SystemMemory {
    fn read_byte(&self, addr: u32) -> u8 {
        if (0xF0000..=0xFFFFF).contains(&addr) {
            // BIOS ROM area (64KB)
            self.bios_rom.read_byte((addr - 0xF0000) as usize)
        } else if (addr as usize) < self.ram.len() {
            // RAM area
            self.ram[addr as usize]
        } else {
            // Invalid memory address
            0
        }
    }

    fn write_byte(&mut self, addr: u32, value: u8) {
        if (0xF0000..=0xFFFFF).contains(&addr) {
            // BIOS ROM area - writes are ignored
        } else if (addr as usize) < self.ram.len() {
            // RAM area
            self.ram[addr as usize] = value;
        }
    }

    fn has_valid_rom(&self) -> bool {
        self.bios_rom.has_valid_code()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

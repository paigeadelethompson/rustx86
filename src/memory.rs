use crate::rom::BiosRom;

// Memory constants
const CONVENTIONAL_MEMORY_SIZE: usize = 640 * 1024;  // 640K conventional memory
const EXTENDED_MEMORY_START: usize = 1024 * 1024;    // 1MB
const HIGH_MEMORY_START: usize = 0xA0000;           // 640K
const HIGH_MEMORY_END: usize = 0x100000;            // 1MB

pub struct Memory {
    ram: Vec<u8>,
    rom: BiosRom,
    a20_enabled: bool,
    smram_enabled: bool,
}

impl Memory {
    pub fn new(ram_size: usize) -> Self {
        Memory {
            ram: vec![0; ram_size],
            rom: BiosRom::new(),  // This will be replaced by load_rom
            a20_enabled: false,
            smram_enabled: false,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let physical_addr = self.translate_address(addr);
        // println!("Memory read at {:#06X} (translated from {:#06X})", physical_addr, addr);
        
        // Check for ROM access (F0000-FFFFF)
        if physical_addr >= 0xF0000 && physical_addr <= 0xFFFFF {
            let rom_offset = physical_addr - 0xF0000;
            let value = self.rom.read_byte(rom_offset as usize);
            // println!("ROM read: physical={:#06X} offset={:#05X} value={:#04X}", 
            //          physical_addr, rom_offset, value);
            return value;
        }

        // Handle RAM access
        let value = if physical_addr < self.ram.len() as u32 {
            self.ram[physical_addr as usize]
        } else {
            // println!("Warning: Reading from unmapped memory at {:#06X}", physical_addr);
            0xFF  // Return 0xFF for unmapped memory instead of 0
        };
        // println!("RAM read: addr={:#06X} value={:#04X}", physical_addr, value);
        value
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let physical_addr = self.translate_address(addr);
        // println!("Memory write at {:#06X} value={:#04X}", physical_addr, value);
        
        // Prevent writes to ROM
        if physical_addr >= 0xF0000 && physical_addr <= 0xFFFFF {
            // println!("Warning: Attempted write to ROM at {:#06X}", physical_addr);
            return;
        }

        if physical_addr < self.ram.len() as u32 {
            self.ram[physical_addr as usize] = value;
            // println!("RAM write: addr={:#06X} value={:#04X}", physical_addr, value);
        } else {
            // println!("Warning: Writing to unmapped memory at {:#06X}", physical_addr);
        }
    }

    pub fn read_word(&self, addr: u32) -> u16 {
        let low = self.read_byte(addr) as u16;
        let high = self.read_byte(addr + 1) as u16;
        (high << 8) | low
    }

    pub fn write_word(&mut self, addr: u32, value: u16) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    fn translate_address(&self, addr: u32) -> u32 {
        // For ROM access (F0000-FFFFF), return the physical address directly
        if addr >= 0xF0000 && addr <= 0xFFFFF {
            // println!("ROM access at {:#06X}", addr);
            return addr;
        }

        // For other addresses, handle A20 line
        let physical_addr = if !self.a20_enabled {
            // A20 line disabled - mask bit 20
            addr & 0xFFFEFFFF
        } else {
            addr
        };

        // Check if the translated address falls into ROM range
        if physical_addr >= 0xF0000 && physical_addr <= 0xFFFFF {
            // println!("Translated address {:#06X} falls into ROM range", physical_addr);
            physical_addr
        } else {
            physical_addr
        }
    }

    pub fn enable_a20(&mut self, enabled: bool) {
        self.a20_enabled = enabled;
    }

    pub fn is_a20_enabled(&self) -> bool {
        self.a20_enabled
    }

    pub fn enable_smram(&mut self, enabled: bool) {
        self.smram_enabled = enabled;
    }

    pub fn read_dword(&self, addr: u32) -> u32 {
        let low = self.read_word(addr) as u32;
        let high = self.read_word(addr + 2) as u32;
        (high << 16) | low
    }

    pub fn write_dword(&mut self, addr: u32, value: u32) {
        self.write_word(addr, (value & 0xFFFF) as u16);
        self.write_word(addr + 2, (value >> 16) as u16);
    }

    // Helper function to check if an address is valid
    pub fn is_address_valid(&self, addr: u32) -> bool {
        let addr = self.translate_address(addr);
        addr < self.ram.len() as u32
    }

    // Helper function to check if an address range is valid
    pub fn is_range_valid(&self, start: u32, size: usize) -> bool {
        let start = self.translate_address(start);
        start + size as u32 <= self.ram.len() as u32
    }

    pub fn load_rom(&mut self, rom: BiosRom) {
        self.rom = rom;
    }

    pub fn load_boot_sector(&mut self, data: &[u8]) {
        // Load boot sector at 0000:7C00
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(0x7C00 + i as u32, byte);
        }
    }
} 
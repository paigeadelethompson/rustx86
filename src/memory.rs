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
        let addr = self.translate_address(addr) as usize;
        
        match addr {
            0..=0x9FFFF => {
                // Conventional memory (0-640K)
                self.ram[addr]
            }
            0xA0000..=0xEFFFF => {
                // Upper Memory Area (UMA)
                self.ram[addr]
            }
            0xF0000..=0xFFFFF => {
                // ROM BIOS area (F000:0000 to F000:FFFF)
                self.rom.read_byte(addr - 0xF0000)
            }
            _ => 0,
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = self.translate_address(addr) as usize;
        
        match addr {
            0..=0x9FFFF => {
                // Conventional memory (0-640K)
                self.ram[addr] = value;
            }
            0xA0000..=0xEFFFF => {
                // Upper Memory Area (UMA)
                self.ram[addr] = value;
            }
            0xF0000..=0xFFFFF => {
                // ROM BIOS area - writes are ignored
            }
            _ => {}
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
        if !self.a20_enabled {
            // A20 line disabled - wrap addresses
            addr & 0xFFEFFFFF
        } else {
            addr
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

    pub fn load_rom(&mut self, data: &[u8]) {
        // Create a new ROM with the provided data
        self.rom = BiosRom::from_data(data.to_vec());
    }

    pub fn load_boot_sector(&mut self, data: &[u8]) {
        // Load boot sector at 0000:7C00
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(0x7C00 + i as u32, byte);
        }
    }
} 
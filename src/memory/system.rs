use super::Memory;
use crate::rom::BiosRom;
use std::any::Any;
use crate::memory::ram::RamMemory;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_memory_initialization() {
        let system = SystemMemory::new(1024);
        
        // Initial state should be zero
        for addr in 0..1024 {
            assert_eq!(system.read_byte(addr), 0);
        }
    }

    #[test]
    fn test_system_memory_byte_operations() {
        let mut system = SystemMemory::new(1024);
        
        // Test byte write/read in RAM area
        system.write_byte(0x100, 0xAA);
        assert_eq!(system.read_byte(0x100), 0xAA);
        
        // Test byte write/read in ROM area (should be ignored)
        system.write_byte(0xF0000, 0xBB);
        assert_ne!(system.read_byte(0xF0000), 0xBB);
    }

    #[test]
    fn test_system_memory_word_operations() {
        let mut system = SystemMemory::new(1024);
        
        // Test word write/read
        system.write_byte(0x200, 0xCD);
        system.write_byte(0x201, 0xAB);
        assert_eq!(system.read_byte(0x200), 0xCD); // Low byte
        assert_eq!(system.read_byte(0x201), 0xAB); // High byte
    }

    #[test]
    fn test_system_memory_boundaries() {
        let mut system = SystemMemory::new(1024);
        
        // Test writing beyond RAM size
        system.write_byte(1024, 0x42); // Should be ignored
        assert_eq!(system.read_byte(1024), 0); // Should return 0
        
        // Test ROM area access
        system.write_byte(0xF0000, 0x42); // Should be ignored
        assert!(system.read_byte(0xF0000) != 0x42); // ROM data should be preserved
    }

    #[test]
    fn test_system_memory_rom_validation() {
        let system = SystemMemory::new(1024);
        assert!(system.has_valid_rom());
    }
}

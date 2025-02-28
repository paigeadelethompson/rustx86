use super::Memory;
use std::any::Any;

pub struct RamMemory {
    memory: Vec<u8>,
}

impl RamMemory {
    #[allow(dead_code)]
    pub fn new(size: usize) -> Self {
        RamMemory {
            memory: vec![0; size],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_initialization() {
        let ram = RamMemory::new(1024);
        assert_eq!(ram.memory.len(), 1024);
        // Memory should be initialized to zero
        for i in 0..1024 {
            assert_eq!(ram.memory[i], 0);
        }
    }

    #[test]
    fn test_ram_byte_operations() {
        let mut ram = RamMemory::new(1024);
        
        // Test single byte operations
        ram.write_byte(0x100, 0xAA);
        assert_eq!(ram.read_byte(0x100), 0xAA);
        
        // Test multiple byte operations
        ram.write_byte(0x200, 0x12);
        ram.write_byte(0x201, 0x34);
        assert_eq!(ram.read_byte(0x200), 0x12);
        assert_eq!(ram.read_byte(0x201), 0x34);
    }

    #[test]
    fn test_ram_word_operations() {
        let mut ram = RamMemory::new(1024);
        
        // Test word write/read
        ram.write_word(0x300, 0xABCD);
        assert_eq!(ram.read_word(0x300), 0xABCD);
        
        // Verify byte ordering (little-endian)
        assert_eq!(ram.read_byte(0x300), 0xCD); // Low byte
        assert_eq!(ram.read_byte(0x301), 0xAB); // High byte
    }

    #[test]
    fn test_ram_boundary_operations() {
        let mut ram = RamMemory::new(1024);
        
        // Test operations at start of memory
        ram.write_word(0, 0x1234);
        assert_eq!(ram.read_word(0), 0x1234);
        
        // Test operations at end of memory
        ram.write_word(1022, 0x5678);
        assert_eq!(ram.read_word(1022), 0x5678);
    }

    #[test]
    fn test_ram_no_rom() {
        let ram = RamMemory::new(1024);
        assert!(!ram.has_valid_rom());
    }
}

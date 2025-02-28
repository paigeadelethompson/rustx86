use std::any::Any;

pub trait Memory: Any {
    fn read_byte(&self, addr: u32) -> u8;
    fn write_byte(&mut self, addr: u32, value: u8);
    fn has_valid_rom(&self) -> bool;

    fn read_word(&self, addr: u32) -> u16 {
        let low = self.read_byte(addr) as u16;
        let high = self.read_byte(addr + 1) as u16;
        (high << 8) | low
    }

    fn write_word(&mut self, addr: u32, value: u16) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    fn as_any(&self) -> &dyn Any;
}

pub mod ram;
pub mod system;

pub use system::SystemMemory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ram::RamMemory;

    struct TestMemory {
        data: Vec<u8>
    }

    impl Memory for TestMemory {
        fn read_byte(&self, addr: u32) -> u8 {
            self.data[addr as usize]
        }

        fn write_byte(&mut self, addr: u32, value: u8) {
            self.data[addr as usize] = value;
        }

        fn has_valid_rom(&self) -> bool {
            false
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_memory_word_operations() {
        let mut mem = TestMemory { data: vec![0; 1024] };
        
        // Test write_word
        mem.write_word(0x100, 0x1234);
        assert_eq!(mem.data[0x100], 0x34); // Low byte
        assert_eq!(mem.data[0x101], 0x12); // High byte

        // Test read_word
        assert_eq!(mem.read_word(0x100), 0x1234);
    }

    #[test]
    fn test_memory_byte_operations() {
        let mut mem = TestMemory { data: vec![0; 1024] };
        
        // Test write_byte
        mem.write_byte(0x200, 0xAB);
        assert_eq!(mem.data[0x200], 0xAB);

        // Test read_byte
        assert_eq!(mem.read_byte(0x200), 0xAB);
    }

    #[test]
    fn test_memory_sequential_operations() {
        let mut mem = TestMemory { data: vec![0; 1024] };
        
        // Write sequential bytes
        mem.write_byte(0x300, 0x12);
        mem.write_byte(0x301, 0x34);
        mem.write_byte(0x302, 0x56);
        mem.write_byte(0x303, 0x78);

        // Read as words
        assert_eq!(mem.read_word(0x300), 0x3412);
        assert_eq!(mem.read_word(0x302), 0x7856);
    }
}

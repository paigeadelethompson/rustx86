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

pub mod system;
pub mod ram;

pub use system::SystemMemory;
pub use ram::RamMemory; 
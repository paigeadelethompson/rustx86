use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn in_al_imm8(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        let value = self.io_read_byte(port as u16)?;
        self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);
        Ok(())
    }

    pub(crate) fn in_ax_imm8(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        let value = self.io_read_word(port as u16)?;
        self.regs.ax = value;
        Ok(())
    }

    pub(crate) fn in_al_dx(&mut self) -> Result<(), String> {
        let value = self.io_read_byte(self.regs.dx)?;
        self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);
        Ok(())
    }

    pub(crate) fn in_ax_dx(&mut self) -> Result<(), String> {
        let value = self.io_read_word(self.regs.dx)?;
        self.regs.ax = value;
        Ok(())
    }

    pub(crate) fn out_imm8_al(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        let value = self.regs.ax as u8;
        self.io_write_byte(port as u16, value)?;
        Ok(())
    }

    pub(crate) fn out_imm8_ax(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        self.io_write_word(port as u16, self.regs.ax)?;
        Ok(())
    }

    pub(crate) fn out_dx_al(&mut self) -> Result<(), String> {
        let value = self.regs.ax as u8;
        self.io_write_byte(self.regs.dx, value)?;
        Ok(())
    }

    pub(crate) fn out_dx_ax(&mut self) -> Result<(), String> {
        self.io_write_word(self.regs.dx, self.regs.ax)?;
        Ok(())
    }

    // Helper functions
    pub(crate) fn io_read_byte(&mut self, _port: u16) -> Result<u8, String> {
        Ok(0)
    }

    pub(crate) fn io_read_word(&mut self, _port: u16) -> Result<u16, String> {
        Ok(0)
    }

    pub(crate) fn io_write_byte(&mut self, _port: u16, _value: u8) -> Result<(), String> {
        Ok(())
    }

    pub(crate) fn io_write_word(&mut self, _port: u16, _value: u16) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use crate::disk::disk_image::DiskImage;
    use std::path::Path;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024));  // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(Path::new("drive_c/")).expect("Failed to create disk image");
        Cpu::new(memory, serial, disk)
    }

    #[test]
    fn test_in_al_imm8() {
        let mut cpu = setup_cpu();
        cpu.memory.write_byte(0, 0x42);  // Port number
        assert!(cpu.in_al_imm8().is_ok());
        assert_eq!(cpu.regs.get_al(), 0);  // Default implementation returns 0
    }

    #[test]
    fn test_in_ax_imm8() {
        let mut cpu = setup_cpu();
        cpu.memory.write_byte(0, 0x42);  // Port number
        assert!(cpu.in_ax_imm8().is_ok());
        assert_eq!(cpu.regs.ax, 0);  // Default implementation returns 0
    }

    #[test]
    fn test_in_al_dx() {
        let mut cpu = setup_cpu();
        cpu.regs.dx = 0x42;  // Port number in DX
        assert!(cpu.in_al_dx().is_ok());
        assert_eq!(cpu.regs.get_al(), 0);  // Default implementation returns 0
    }

    #[test]
    fn test_in_ax_dx() {
        let mut cpu = setup_cpu();
        cpu.regs.dx = 0x42;  // Port number in DX
        assert!(cpu.in_ax_dx().is_ok());
        assert_eq!(cpu.regs.ax, 0);  // Default implementation returns 0
    }

    #[test]
    fn test_out_imm8_al() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;  // AL = 0x34
        cpu.memory.write_byte(0, 0x42);  // Port number
        assert!(cpu.out_imm8_al().is_ok());
    }

    #[test]
    fn test_out_imm8_ax() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        cpu.memory.write_byte(0, 0x42);  // Port number
        assert!(cpu.out_imm8_ax().is_ok());
    }

    #[test]
    fn test_out_dx_al() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;  // AL = 0x34
        cpu.regs.dx = 0x42;    // Port number in DX
        assert!(cpu.out_dx_al().is_ok());
    }

    #[test]
    fn test_out_dx_ax() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        cpu.regs.dx = 0x42;  // Port number in DX
        assert!(cpu.out_dx_ax().is_ok());
    }

    #[test]
    fn test_io_read_byte() {
        let mut cpu = setup_cpu();
        assert_eq!(cpu.io_read_byte(0x42).unwrap(), 0);  // Default implementation returns 0
    }

    #[test]
    fn test_io_read_word() {
        let mut cpu = setup_cpu();
        assert_eq!(cpu.io_read_word(0x42).unwrap(), 0);  // Default implementation returns 0
    }

    #[test]
    fn test_io_write_byte() {
        let mut cpu = setup_cpu();
        assert!(cpu.io_write_byte(0x42, 0x34).is_ok());  // Default implementation just returns Ok
    }

    #[test]
    fn test_io_write_word() {
        let mut cpu = setup_cpu();
        assert!(cpu.io_write_word(0x42, 0x1234).is_ok());  // Default implementation just returns Ok
    }
}

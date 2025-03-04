use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn cli(&mut self) -> Result<(), String> {
        self.regs.flags.set_interrupt(false);
        Ok(())
    }

    pub(crate) fn sti(&mut self) -> Result<(), String> {
        self.regs.flags.set_interrupt(true);
        Ok(())
    }

    pub(crate) fn clc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(false);
        Ok(())
    }

    pub(crate) fn stc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(true);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn cmc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(!self.regs.flags.get_carry());
        Ok(())
    }

    pub(crate) fn cld(&mut self) -> Result<(), String> {
        self.regs.flags.set_direction(false);
        Ok(())
    }

    pub(crate) fn std(&mut self) -> Result<(), String> {
        self.regs.flags.set_direction(true);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn sahf(&mut self) -> Result<(), String> {
        let ah = (self.regs.ax >> 8) as u8;
        self.regs.flags.set_from_byte(ah);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn lahf(&mut self) -> Result<(), String> {
        let flags = self.regs.flags.as_byte();
        self.regs.ax = (self.regs.ax & 0x00FF) | ((flags as u16) << 8);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn pushf(&mut self) -> Result<(), String> {
        let flags = self.regs.flags.as_u16();
        self.push_word(flags)
    }

    #[allow(dead_code)]
    pub(crate) fn popf(&mut self) -> Result<(), String> {
        let flags = self.pop_word()?;
        println!("POPF: Popped flags value: 0x{:04X}", flags);
        println!(
            "POPF: Previous flags value: 0x{:04X}",
            self.regs.flags.as_u16()
        );
        self.regs.flags.set_from_u16(flags);
        println!("POPF: New flags value: 0x{:04X}", self.regs.flags.as_u16());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::disk_image::DiskImage;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use std::path::PathBuf;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024)); // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(&PathBuf::from("drive_c/")).expect("Failed to create disk image");
        Cpu::new(memory, serial, disk)
    }

    #[test]
    fn test_cli() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_interrupt(true);
        assert!(cpu.cli().is_ok());
        assert!(!cpu.regs.flags.get_interrupt());
    }

    #[test]
    fn test_sti() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_interrupt(false);
        assert!(cpu.sti().is_ok());
        assert!(cpu.regs.flags.get_interrupt());
    }

    #[test]
    fn test_clc() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_carry(true);
        assert!(cpu.clc().is_ok());
        assert!(!cpu.regs.flags.get_carry());
    }

    #[test]
    fn test_stc() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_carry(false);
        assert!(cpu.stc().is_ok());
        assert!(cpu.regs.flags.get_carry());
    }

    #[test]
    fn test_cmc() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_carry(false);
        assert!(cpu.cmc().is_ok());
        assert!(cpu.regs.flags.get_carry());

        assert!(cpu.cmc().is_ok());
        assert!(!cpu.regs.flags.get_carry());
    }

    #[test]
    fn test_cld() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_direction(true);
        assert!(cpu.cld().is_ok());
        assert!(!cpu.regs.flags.get_direction());
    }

    #[test]
    fn test_std() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_direction(false);
        assert!(cpu.std().is_ok());
        assert!(cpu.regs.flags.get_direction());
    }

    #[test]
    fn test_sahf() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x5500; // Set AH to 0x55
        assert!(cpu.sahf().is_ok());
        // Check that flags were set from AH
        assert_eq!(cpu.regs.flags.as_byte() & 0xD5, 0x55 & 0xD5);
    }

    #[test]
    fn test_lahf() {
        let mut cpu = setup_cpu();
        // Set some flags
        cpu.regs.flags.set_carry(true);
        cpu.regs.flags.set_zero(true);
        cpu.regs.flags.set_sign(true);
        assert!(cpu.lahf().is_ok());
        // Check that AH contains the flags
        assert_eq!(cpu.regs.ax & 0xFF00, (cpu.regs.flags.as_byte() as u16) << 8);
    }

    #[test]
    fn test_pushf() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x2000;
        let flags_value = cpu.regs.flags.as_u16();
        assert!(cpu.pushf().is_ok());
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE),
            flags_value
        );
        assert_eq!(cpu.regs.sp, 0x1FFE);
    }

    #[test]
    fn test_popf() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        let flags_value = 0x0202; // Example flags value
        println!(
            "Test: Writing flags 0x{:04X} to memory at SS:SP ({}:{})",
            flags_value, cpu.regs.ss, cpu.regs.sp
        );
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, flags_value);

        // Verify the write worked
        let read_back = cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE);
        println!("Test: Read back value from memory: 0x{:04X}", read_back);

        assert!(cpu.popf().is_ok());

        // The actual flags value will include the reserved bits (1 and 3) set to 1
        let expected_flags = flags_value | 0x000A; // Add reserved bits
        assert_eq!(
            cpu.regs.flags.as_u16(),
            expected_flags,
            "Flags not set correctly. Expected 0x{:04X}, got 0x{:04X}",
            expected_flags,
            cpu.regs.flags.as_u16()
        );
        assert_eq!(
            cpu.regs.sp, 0x2000,
            "Stack pointer not incremented correctly. Expected 0x2000, got 0x{:04X}",
            cpu.regs.sp
        );
    }
}

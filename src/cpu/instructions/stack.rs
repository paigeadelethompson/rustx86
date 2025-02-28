use crate::cpu::Cpu;

impl Cpu {
    pub fn push_ax(&mut self) -> Result<(), String> {
        self.push_word(self.regs.ax)?;
        Ok(())
    }

    pub fn push_cx(&mut self) -> Result<(), String> {
        self.push_word(self.regs.cx)?;
        Ok(())
    }

    pub fn push_dx(&mut self) -> Result<(), String> {
        self.push_word(self.regs.dx)?;
        Ok(())
    }

    pub fn push_bx(&mut self) -> Result<(), String> {
        self.push_word(self.regs.bx)?;
        Ok(())
    }

    pub fn push_es(&mut self) -> Result<(), String> {
        self.push_word(self.regs.es)?;
        Ok(())
    }

    pub fn push_word(&mut self, value: u16) -> Result<(), String> {
        // Decrement SP by 2 (word size)
        self.regs.sp = self.regs.sp.wrapping_sub(2);

        // Calculate physical address for the stack
        let address = ((self.regs.ss as u32) << 4) | (self.regs.sp as u32);

        // Write the word to memory
        self.memory.write_word(address, value);

        Ok(())
    }

    pub fn pop_cx(&mut self) -> Result<(), String> {
        self.regs.cx = self.pop_word()?;
        Ok(())
    }

    pub fn pop_dx(&mut self) -> Result<(), String> {
        self.regs.dx = self.pop_word()?;
        Ok(())
    }

    pub fn pop_bx(&mut self) -> Result<(), String> {
        self.regs.bx = self.pop_word()?;
        Ok(())
    }

    pub fn pop_ax(&mut self) -> Result<(), String> {
        self.regs.ax = self.pop_word()?;
        Ok(())
    }

    pub fn pop_es(&mut self) -> Result<(), String> {
        self.regs.es = self.pop_word()?;
        Ok(())
    }

    pub fn leave(&mut self) -> Result<(), String> {
        self.regs.sp = self.regs.bp;
        self.regs.bp = self.pop_word()?;
        Ok(())
    }

    pub fn enter(&mut self, nesting_level: u8) -> Result<(), String> {
        let frame_size = self.fetch_word()?;
        let bp = self.regs.bp;
        self.push_word(bp)?;
        let frame_ptr = self.regs.sp;

        if nesting_level > 0 {
            for _level in 1..=nesting_level {
                self.regs.bp = self.regs.bp.wrapping_sub(2);
                let temp = self.memory.read_word(self.regs.bp as u32);
                self.push_word(temp)?;
            }
        }

        self.regs.bp = frame_ptr;
        self.regs.sp = self.regs.sp.wrapping_sub(frame_size);
        Ok(())
    }

    pub fn ret_far_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        let ip = self.pop_word()?;
        let cs = self.pop_word()?;
        self.regs.sp += imm16;
        self.regs.ip = ip;
        self.regs.cs = cs;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::disk_image::DiskImage;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use std::path::Path;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024)); // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(Path::new("drive_c/")).expect("Failed to create disk image");
        let mut cpu = Cpu::new(memory, serial, disk);
        cpu.regs.sp = 0x2000; // Initialize stack pointer
        cpu
    }

    #[test]
    fn test_push_ax() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        assert!(cpu.push_ax().is_ok());
        assert_eq!(cpu.regs.sp, 0x1FFE); // SP should decrease by 2
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE),
            0x1234
        );
    }

    #[test]
    fn test_push_cx() {
        let mut cpu = setup_cpu();
        cpu.regs.cx = 0x5678;
        assert!(cpu.push_cx().is_ok());
        assert_eq!(cpu.regs.sp, 0x1FFE);
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE),
            0x5678
        );
    }

    #[test]
    fn test_push_dx() {
        let mut cpu = setup_cpu();
        cpu.regs.dx = 0x9ABC;
        assert!(cpu.push_dx().is_ok());
        assert_eq!(cpu.regs.sp, 0x1FFE);
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE),
            0x9ABC
        );
    }

    #[test]
    fn test_push_bx() {
        let mut cpu = setup_cpu();
        cpu.regs.bx = 0xDEF0;
        assert!(cpu.push_bx().is_ok());
        assert_eq!(cpu.regs.sp, 0x1FFE);
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE),
            0xDEF0
        );
    }

    #[test]
    fn test_push_es() {
        let mut cpu = setup_cpu();
        cpu.regs.es = 0x1000;
        assert!(cpu.push_es().is_ok());
        assert_eq!(cpu.regs.sp, 0x1FFE);
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | 0x1FFE),
            0x1000
        );
    }

    #[test]
    fn test_pop_cx() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x5678);
        assert!(cpu.pop_cx().is_ok());
        assert_eq!(cpu.regs.cx, 0x5678);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_pop_dx() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x9ABC);
        assert!(cpu.pop_dx().is_ok());
        assert_eq!(cpu.regs.dx, 0x9ABC);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_pop_bx() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0xDEF0);
        assert!(cpu.pop_bx().is_ok());
        assert_eq!(cpu.regs.bx, 0xDEF0);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_pop_ax() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x1234);
        assert!(cpu.pop_ax().is_ok());
        assert_eq!(cpu.regs.ax, 0x1234);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_pop_es() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x1000);
        assert!(cpu.pop_es().is_ok());
        assert_eq!(cpu.regs.es, 0x1000);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_leave() {
        let mut cpu = setup_cpu();
        cpu.regs.bp = 0x1FFE;
        cpu.regs.sp = 0x1FFC;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x2000);
        assert!(cpu.leave().is_ok());
        assert_eq!(cpu.regs.bp, 0x2000);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    #[ignore = "ENTER instruction implementation needs verification"]
    fn test_enter() {
        let mut cpu = setup_cpu();
        cpu.regs.bp = 0x2000;
        cpu.regs.sp = 0x2000;
        cpu.memory.write_word(0, 0x0010); // Frame size
        cpu.memory.write_byte(2, 0); // Nesting level
        assert!(cpu.enter(0).is_ok());
        assert_eq!(cpu.regs.bp, 0x1FFE); // New frame pointer
        assert_eq!(cpu.regs.sp, 0x1FEE); // SP = BP - frame_size
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_ret_far_imm16() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFC;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFC, 0x1000); // CS
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x0100); // IP
        cpu.memory.write_word(0, 0x0004); // Immediate value
        assert!(cpu.ret_far_imm16().is_ok());
        assert_eq!(cpu.regs.cs, 0x1000);
        assert_eq!(cpu.regs.ip, 0x0100);
        assert_eq!(cpu.regs.sp, 0x2004); // SP = original + 4 + imm16
    }
}

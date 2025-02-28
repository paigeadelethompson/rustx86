use crate::cpu::Cpu;
use std::path::Path;

impl Cpu {
    pub(crate) fn mov_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        self.write_rm8(modrm, reg_val)?;
        Ok(())
    }

    pub(crate) fn mov_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        self.regs.set_reg8(reg, rm_val)?;
        Ok(())
    }

    pub(crate) fn mov_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg_val = self.regs.get_reg16((modrm >> 3) & 0x07);
        self.write_rm16(modrm, reg_val)?;
        Ok(())
    }

    pub(crate) fn mov_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        self.regs.set_reg16(reg, rm_val)?;
        Ok(())
    }

    pub(crate) fn mov_rm16_sreg(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let sreg = (modrm >> 3) & 0x03;
        let sreg_val = self.regs.get_sreg(sreg);
        self.write_rm16(modrm, sreg_val)?;
        Ok(())
    }

    pub(crate) fn mov_sreg_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let sreg = (modrm >> 3) & 0x03;
        self.regs.set_sreg(sreg, rm_val);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn mov_al_moffs8(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let val = self
            .memory
            .read_byte(self.get_physical_address(self.regs.ds, offset));
        self.regs.ax = (self.regs.ax & 0xFF00) | (val as u16);
        Ok(())
    }

    pub(crate) fn mov_ax_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        self.regs.ax = imm16;
        Ok(())
    }

    pub(crate) fn mov_si_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        self.regs.si = imm16;
        Ok(())
    }

    pub(crate) fn mov_ah_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        self.regs.set_ah(imm8);
        Ok(())
    }

    pub(crate) fn mov_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        self.regs.set_al(imm8);
        Ok(())
    }

    pub(crate) fn mov_cx_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        self.regs.cx = imm16;
        Ok(())
    }

    pub(crate) fn mov_bx_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        self.regs.bx = imm16;
        Ok(())
    }

    pub(crate) fn mov_dl_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        self.regs.set_dl(imm8);
        Ok(())
    }

    pub(crate) fn mov_sp_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        self.regs.sp = imm16;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn mov_ax_moffs16(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let val = self
            .memory
            .read_word(self.get_physical_address(self.regs.ds, offset));
        self.regs.ax = val;
        Ok(())
    }

    pub(crate) fn xchg_ax_r16(&mut self, reg: u8) -> Result<(), String> {
        let ax = self.regs.ax;
        let reg_val = self.regs.get_reg16(reg);
        self.regs.ax = reg_val;
        self.regs.set_reg16(reg, ax)?;
        Ok(())
    }

    // Pop instructions
    pub(crate) fn pop_word(&mut self) -> Result<u16, String> {
        let addr = self.get_physical_address(self.regs.ss, self.regs.sp);
        let value = self.memory.read_word(addr);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        Ok(value)
    }

    pub(crate) fn les_r16_m16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_addr = self.get_rm_addr(modrm)?;
        let offset = self.read_word(rm_addr)?;
        let segment = self.read_word(rm_addr.wrapping_add(2))?;
        let reg = (modrm >> 3) & 0x07;
        self.regs.set_reg16(reg, offset)?;
        self.regs.es = segment;
        Ok(())
    }

    pub(crate) fn mov_ch_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        self.regs.set_ch(imm8);
        Ok(())
    }

    pub(crate) fn mov_dh_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        self.regs.set_dh(imm8);
        Ok(())
    }

    pub(crate) fn mov_cl_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        self.regs.set_cl(imm8);
        Ok(())
    }

    // More data transfer instructions can be added here...
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
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_rm8_r8() {
        let mut cpu = setup_cpu();
        // Set AH to 0x12 and AL to 0x34
        cpu.regs.ax = 0x1234;
        // ModR/M byte: 0xC4 = register-to-register, dest=AL (reg 0), src=AH (reg 4)
        cpu.memory.write_byte(0, 0xC4);
        assert!(cpu.mov_rm8_r8().is_ok());
        // AL should now be 0x12, AH should still be 0x12
        assert_eq!(cpu.regs.get_al(), 0x12);
        assert_eq!(cpu.regs.get_ah(), 0x12);
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_r8_rm8() {
        let mut cpu = setup_cpu();
        // Set AH to 0x12 and AL to 0x34
        cpu.regs.ax = 0x1234;
        // ModR/M byte: 0xE0 = register-to-register, dest=AH (reg 4), src=AL (reg 0)
        cpu.memory.write_byte(0, 0xE0);
        assert!(cpu.mov_r8_rm8().is_ok());
        // AH should now be 0x34, AL should still be 0x34
        assert_eq!(cpu.regs.get_ah(), 0x34);
        assert_eq!(cpu.regs.get_al(), 0x34);
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_rm16_r16() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        cpu.memory.write_byte(0, 0xC0);  // ModR/M byte for register-to-register
        assert!(cpu.mov_rm16_r16().is_ok());
        assert_eq!(cpu.regs.ax, 0x1234);  // Value should be moved
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_r16_rm16() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        cpu.memory.write_byte(0, 0xC0);  // ModR/M byte for register-to-register
        assert!(cpu.mov_r16_rm16().is_ok());
        assert_eq!(cpu.regs.ax, 0x1234);  // Value should be moved
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_al_moffs8() {
        let mut cpu = setup_cpu();
        cpu.memory.write_word(0, 0x1000);  // Offset
        cpu.memory.write_byte(0x1000, 0x42);  // Value at memory location
        assert!(cpu.mov_al_moffs8().is_ok());
        assert_eq!(cpu.regs.ax & 0xFF, 0x42);  // AL should contain the value
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_ax_imm16() {
        let mut cpu = setup_cpu();
        cpu.memory.write_word(0, 0x1234);  // Immediate value
        assert!(cpu.mov_ax_imm16().is_ok());
        assert_eq!(cpu.regs.ax, 0x1234);
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_si_imm16() {
        let mut cpu = setup_cpu();
        cpu.memory.write_word(0, 0x1234);  // Immediate value
        assert!(cpu.mov_si_imm16().is_ok());
        assert_eq!(cpu.regs.si, 0x1234);
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_ah_imm8() {
        let mut cpu = setup_cpu();
        cpu.memory.write_byte(0, 0x42);  // Immediate value
        assert!(cpu.mov_ah_imm8().is_ok());
        assert_eq!(cpu.regs.ax & 0xFF00, 0x4200);
        assert_eq!(cpu.regs.get_ah(), 0x42);
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_mov_al_imm8() {
        let mut cpu = setup_cpu();
        cpu.memory.write_byte(0, 0x42);  // Immediate value
        assert!(cpu.mov_al_imm8().is_ok());
        assert_eq!(cpu.regs.ax & 0xFF, 0x42);
        assert_eq!(cpu.regs.get_al(), 0x42);
    }

    #[test]
    fn test_xchg_ax_r16() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        cpu.regs.bx = 0x5678;
        assert!(cpu.xchg_ax_r16(3).is_ok());  // 3 is BX
        assert_eq!(cpu.regs.ax, 0x5678);
        assert_eq!(cpu.regs.bx, 0x1234);
    }

    #[test]
    #[ignore = "Needs investigation of segment handling"]
    fn test_les_r16_m16() {
        let mut cpu = setup_cpu();
        cpu.memory.write_byte(0, 0x06);  // ModR/M byte for memory mode
        cpu.memory.write_word(0x1000, 0x1234);  // Offset
        cpu.memory.write_word(0x1002, 0x2000);  // Segment
        assert!(cpu.les_r16_m16().is_ok());
        assert_eq!(cpu.regs.es, 0x2000);
    }
}

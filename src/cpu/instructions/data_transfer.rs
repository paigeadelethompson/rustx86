use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn mov_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("MOV: ModR/M byte = 0x{:02X}", modrm);
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        println!("MOV: reg = {}, reg_val = 0x{:02X}", reg, reg_val);
        self.write_rm8(modrm, reg_val)?;
        Ok(())
    }

    pub(crate) fn mov_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("MOV: ModR/M byte = 0x{:02X}", modrm);
        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        println!("MOV: rm_val = 0x{:02X}, reg = {}", rm_val, reg);
        self.regs.set_reg8(reg, rm_val)?;
        Ok(())
    }

    pub(crate) fn mov_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("MOV: ModR/M byte = 0x{:02X}", modrm);
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg16(reg);
        println!("MOV: reg = {}, reg_val = 0x{:04X}", reg, reg_val);
        self.write_rm16(modrm, reg_val)?;
        Ok(())
    }

    pub(crate) fn mov_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("MOV: ModR/M byte = 0x{:02X}", modrm);
        let rm_val = self.get_rm16(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        println!("MOV: rm_val = 0x{:04X}, reg = {}", rm_val, reg);
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

    pub(crate) fn mov_al_moffs8(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        println!("MOV: offset = 0x{:04X}", offset);
        let val = self
            .memory
            .read_byte(self.get_physical_address(self.regs.ds, offset));
        println!("MOV: val = 0x{:02X}", val);
        self.regs.ax = (self.regs.ax & 0xFF00) | (val as u16);
        Ok(())
    }

    pub(crate) fn mov_ax_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        println!("MOV: imm16 = 0x{:04X}", imm16);
        self.regs.ax = imm16;
        Ok(())
    }

    pub(crate) fn mov_si_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        println!("MOV: imm16 = 0x{:04X}", imm16);
        self.regs.si = imm16;
        Ok(())
    }

    pub(crate) fn mov_ah_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        println!("MOV: imm8 = 0x{:02X}", imm8);
        self.regs.set_ah(imm8);
        Ok(())
    }

    pub(crate) fn mov_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        println!("MOV: imm8 = 0x{:02X}", imm8);
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
        println!("LES: ModR/M byte = 0x{:02X}", modrm);
        let rm_addr = self.get_rm_addr(modrm)?;
        println!("LES: rm_addr = 0x{:04X}", rm_addr);
        let offset = self.read_word(rm_addr)?;
        let segment = self.read_word(rm_addr.wrapping_add(2))?;
        println!("LES: offset = 0x{:04X}, segment = 0x{:04X}", offset, segment);
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
    use crate::disk::disk_image::DiskImage;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use std::path::Path;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024)); // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(Path::new("drive_c/")).expect("Failed to create disk image");
        Cpu::new(memory, serial, disk)
    }

    #[test]
    fn test_mov_rm8_r8() {
        let mut cpu = setup_cpu();
        // Set AH to 0x12 and AL to 0x34
        cpu.regs.ax = 0x1234;
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xE0 = 11 100 000
        // 11: register-to-register mode
        // 100: source register is AH (reg 4)
        // 000: destination register is AL (reg 0)
        cpu.memory.write_byte(0x100, 0xE0);
        assert!(cpu.mov_rm8_r8().is_ok());
        // AL should now be 0x12, AH should still be 0x12
        assert_eq!(cpu.regs.get_al(), 0x12);
        assert_eq!(cpu.regs.get_ah(), 0x12);
    }

    #[test]
    fn test_mov_r8_rm8() {
        let mut cpu = setup_cpu();
        // Set AH to 0x12 and AL to 0x34
        cpu.regs.ax = 0x1234;
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xE0 = 11 100 000
        // 11: register-to-register mode
        // 100: destination register is AH (reg 4)
        // 000: source register is AL (reg 0)
        cpu.memory.write_byte(0x100, 0xE0);
        assert!(cpu.mov_r8_rm8().is_ok());
        // AH should now be 0x34, AL should still be 0x34
        assert_eq!(cpu.regs.get_ah(), 0x34);
        assert_eq!(cpu.regs.get_al(), 0x34);
    }

    #[test]
    fn test_mov_rm16_r16() {
        let mut cpu = setup_cpu();
        // Set AX to 0x1234 and BX to 0x5678
        cpu.regs.ax = 0x1234;
        cpu.regs.bx = 0x5678;
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xC3 = 11 000 011
        // 11: register-to-register mode
        // 000: source register is AX (reg 0)
        // 011: destination register is BX (reg 3)
        cpu.memory.write_byte(0x100, 0xC3);
        assert!(cpu.mov_rm16_r16().is_ok());
        // BX should now be 0x1234, AX should still be 0x1234
        assert_eq!(cpu.regs.get_bx(), 0x1234);
        assert_eq!(cpu.regs.get_ax(), 0x1234);
    }

    #[test]
    fn test_mov_r16_rm16() {
        let mut cpu = setup_cpu();
        // Set AX to 0x1234 and BX to 0x5678
        cpu.regs.ax = 0x1234;
        cpu.regs.bx = 0x5678;
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xD8 = 11 011 000
        // 11: register-to-register mode
        // 011: destination register is BX (reg 3)
        // 000: source register is AX (reg 0)
        cpu.memory.write_byte(0x100, 0xD8);
        assert!(cpu.mov_r16_rm16().is_ok());
        // BX should now be 0x1234, AX should still be 0x1234
        assert_eq!(cpu.regs.get_bx(), 0x1234);
        assert_eq!(cpu.regs.get_ax(), 0x1234);
    }

    #[test]
    fn test_mov_al_moffs8() {
        let mut cpu = setup_cpu();
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set DS to 0 so physical address matches offset
        cpu.regs.ds = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // Write the offset (0x1000) to memory at IP
        cpu.memory.write_word(0x100, 0x1000);
        // Write the value (0x42) to memory at offset
        cpu.memory.write_byte(0x1000, 0x42);
        assert!(cpu.mov_al_moffs8().is_ok());
        // AL should contain the value
        assert_eq!(cpu.regs.get_al(), 0x42);
    }

    #[test]
    fn test_mov_ax_imm16() {
        let mut cpu = setup_cpu();
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // Write the immediate value (0x1234) to memory at IP
        cpu.memory.write_word(0x100, 0x1234);
        assert!(cpu.mov_ax_imm16().is_ok());
        // AX should contain the immediate value
        assert_eq!(cpu.regs.get_ax(), 0x1234);
    }

    #[test]
    fn test_mov_si_imm16() {
        let mut cpu = setup_cpu();
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // Write the immediate value (0x1234) to memory at IP
        cpu.memory.write_word(0x100, 0x1234);
        assert!(cpu.mov_si_imm16().is_ok());
        // SI should contain the immediate value
        assert_eq!(cpu.regs.si, 0x1234);
    }

    #[test]
    fn test_mov_ah_imm8() {
        let mut cpu = setup_cpu();
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // Write the immediate value (0x42) to memory at IP
        cpu.memory.write_byte(0x100, 0x42);
        assert!(cpu.mov_ah_imm8().is_ok());
        // AH should contain the immediate value
        assert_eq!(cpu.regs.ax & 0xFF00, 0x4200);
        assert_eq!(cpu.regs.get_ah(), 0x42);
    }

    #[test]
    fn test_mov_al_imm8() {
        let mut cpu = setup_cpu();
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // Write the immediate value (0x42) to memory at IP
        cpu.memory.write_byte(0x100, 0x42);
        assert!(cpu.mov_al_imm8().is_ok());
        // AL should contain the immediate value
        assert_eq!(cpu.regs.ax & 0xFF, 0x42);
        assert_eq!(cpu.regs.get_al(), 0x42);
    }

    #[test]
    fn test_xchg_ax_r16() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        cpu.regs.bx = 0x5678;
        assert!(cpu.xchg_ax_r16(3).is_ok()); // 3 is BX
        assert_eq!(cpu.regs.ax, 0x5678);
        assert_eq!(cpu.regs.bx, 0x1234);
    }

    #[test]
    fn test_les_r16_m16() {
        let mut cpu = setup_cpu();
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set DS to 0 so physical address matches offset
        cpu.regs.ds = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0x06 = 00 000 110
        // 00: memory mode, no displacement
        // 000: destination register is AX (reg 0)
        // 110: direct memory addressing
        cpu.memory.write_byte(0x100, 0x06);
        // Write the address (0x1000) for direct addressing
        cpu.memory.write_word(0x101, 0x1000);
        // Write the far pointer at the memory location
        cpu.memory.write_word(0x1000, 0x1234); // Offset
        cpu.memory.write_word(0x1002, 0x2000); // Segment
        assert!(cpu.les_r16_m16().is_ok());
        // AX should contain the offset
        assert_eq!(cpu.regs.get_ax(), 0x1234);
        // ES should contain the segment
        assert_eq!(cpu.regs.es, 0x2000);
    }
}

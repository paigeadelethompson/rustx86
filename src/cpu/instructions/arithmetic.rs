use crate::cpu::Cpu;

impl Cpu {
    pub fn add_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("ModR/M byte: 0x{:02X}", modrm);
        let rm_val = self.get_rm8(modrm)?;
        println!("rm_val (destination): 0x{:02X}", rm_val);
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        println!("reg_val (source): 0x{:02X}", reg_val);
        let (result, carry) = rm_val.overflowing_add(reg_val);
        println!("result: 0x{:02X}", result);
        self.write_rm8(modrm, result)?;
        self.update_flags_add(rm_val, reg_val, result, carry);
        Ok(())
    }

    pub fn add_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("ModR/M byte: 0x{:02X}", modrm);
        let rm_val = self.get_rm16(modrm)?;
        println!("rm_val (destination): 0x{:04X}", rm_val);
        let reg_val = self.regs.get_reg16((modrm >> 3) & 0x07);
        println!("reg_val (source): 0x{:04X}", reg_val);
        let (result, carry) = rm_val.overflowing_add(reg_val);
        println!("result: 0x{:04X}", result);
        self.write_rm16(modrm, result)?;
        self.update_flags_add16(rm_val, reg_val, result, carry);
        Ok(())
    }

    pub fn add_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        println!("Immediate value: 0x{:02X}", imm8);
        let al = self.regs.get_al();
        println!("AL value: 0x{:02X}", al);
        let (result, carry) = al.overflowing_add(imm8);
        println!("Result: 0x{:02X}", result);
        self.regs.set_al(result);
        self.update_flags_add(al, imm8, result, carry);
        Ok(())
    }

    pub fn add_ax_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        let ax = self.regs.ax;
        let (result, carry) = ax.overflowing_add(imm16);
        self.regs.ax = result;
        self.update_flags_add16(ax, imm16, result, carry);
        Ok(())
    }

    pub fn adc_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let dest = self.get_rm8(modrm)?;
        let src = self.regs.get_reg8((modrm >> 3) & 0x7);
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        println!("ADC: dest={:02X}, src={:02X}, carry={}", dest, src, carry);
        let (result, carry1) = dest.overflowing_add(src);
        let (result, carry2) = result.overflowing_add(carry);
        println!(
            "ADC: result={:02X}, carry1={}, carry2={}",
            result, carry1, carry2
        );
        self.write_rm8(modrm, result)?;
        self.regs.flags.set_carry(carry1 || carry2);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result as i8) < 0);
        Ok(())
    }

    pub fn adc_al_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let al = self.regs.ax as u8;
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        let (temp, carry1) = rm_val.overflowing_add(carry);
        let (result, carry2) = al.overflowing_add(temp);
        self.regs.ax = (self.regs.ax & 0xFF00) | (result as u16);
        self.update_flags_add(al, rm_val, result, carry1 || carry2);
        Ok(())
    }

    pub fn add_ax_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg16(reg);
        let (result, carry) = reg_val.overflowing_add(rm_val);
        self.regs.set_reg16(reg, result)?;
        self.update_flags_add16(reg_val, rm_val, result, carry);
        Ok(())
    }

    pub fn add_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        let (result, carry) = reg_val.overflowing_add(rm_val);
        self.regs.set_reg8(reg, result)?;
        self.update_flags_add(reg_val, rm_val, result, carry);
        Ok(())
    }

    pub fn cmp_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.get_reg8(0); // AL is register 0
        let (result, carry) = al.overflowing_sub(imm8);
        self.update_flags_sub(al, imm8, result, carry);
        Ok(())
    }

    pub fn cmp_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg16(reg);
        let (result, carry) = reg_val.overflowing_sub(rm_val);
        self.update_flags_sub16(reg_val, rm_val, result, carry);
        Ok(())
    }

    pub fn cmp_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let (result, carry) = rm_val.overflowing_sub(reg_val);
        // CMP is like SUB but doesn't store the result
        self.update_flags_sub(rm_val, reg_val, result, carry);
        Ok(())
    }

    // Helper functions for flag updates
    pub fn update_flags_add(&mut self, a: u8, b: u8, result: u8, carry: bool) {
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs
            .flags
            .set_overflow(((a ^ result) & (b ^ result) & 0x80) != 0);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
    }

    pub fn update_flags_add16(&mut self, _a: u16, _b: u16, result: u16, carry: bool) {
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        let result_i16 = result as i16;
        let overflow = !(-0x8000..=0x7FFF).contains(&result_i16);
        self.regs.flags.set_overflow(overflow);
        self.regs
            .flags
            .set_parity((result as u8).count_ones() % 2 == 0);
    }

    // INC/DEC instructions
    pub fn inc_ax(&mut self) -> Result<(), String> {
        let (result, _) = self.regs.ax.overflowing_add(1);
        self.update_flags_inc16(result);
        self.regs.ax = result;
        Ok(())
    }

    pub fn inc_cx(&mut self) -> Result<(), String> {
        let (result, _) = self.regs.cx.overflowing_add(1);
        self.update_flags_inc16(result);
        self.regs.cx = result;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn dec_bx(&mut self) -> Result<(), String> {
        let (result, _) = self.regs.bx.overflowing_sub(1);
        self.update_flags_dec16(result);
        self.regs.bx = result;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn imul_r16_rm16_imm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let imm = self.fetch_word()?;
        println!("IMUL: rm_val={:04X}, imm={:04X}", rm_val, imm);
        let result = (rm_val as i16 as i32) * (imm as i16 as i32);
        println!("IMUL: result={:08X}", result);
        let reg = (modrm >> 3) & 0x07;
        self.regs.set_reg16(reg, result as u16)?;
        // Set flags based on overflow
        let overflow = !(-0x8000..=0x7FFF).contains(&(result as i16));
        self.regs.flags.set_carry(overflow);
        self.regs.flags.set_overflow(overflow);
        self.regs.flags.set_sign((result as i16) < 0);
        self.regs.flags.set_zero(result == 0);
        Ok(())
    }

    pub fn inc_si(&mut self) -> Result<(), String> {
        let si = self.regs.si;
        let (result, _) = si.overflowing_add(1);
        self.regs.si = result;
        self.update_flags_inc(si, result);
        Ok(())
    }

    pub fn inc_bp(&mut self) -> Result<(), String> {
        let (result, _) = self.regs.bp.overflowing_add(1);
        self.update_flags_inc(self.regs.bp, result);
        self.regs.bp = result;
        Ok(())
    }

    pub fn inc_sp(&mut self) -> Result<(), String> {
        let sp = self.regs.sp;
        let (result, _) = sp.overflowing_add(1);
        self.regs.sp = result;
        self.update_flags_inc(sp, result);
        Ok(())
    }

    pub fn dec_di(&mut self) -> Result<(), String> {
        let (result, _) = self.regs.di.overflowing_sub(1);
        self.update_flags_dec16(result);
        self.regs.di = result;
        Ok(())
    }

    pub fn dec_si(&mut self) -> Result<(), String> {
        let (result, _) = self.regs.si.overflowing_sub(1);
        self.update_flags_dec16(result);
        self.regs.si = result;
        Ok(())
    }

    pub fn salc(&mut self) -> Result<(), String> {
        // Set AL to 0xFF if carry flag is set, 0x00 if carry flag is clear
        self.regs.set_al(if self.regs.flags.get_carry() {
            0xFF
        } else {
            0x00
        });
        Ok(())
    }

    pub fn sbb_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.get_al();
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        let (result1, overflow1) = al.overflowing_sub(imm8);
        let (result, overflow2) = result1.overflowing_sub(carry);
        self.update_flags_sub(al, imm8, result, overflow1 || overflow2);
        self.regs.set_al(result);
        Ok(())
    }

    pub fn aam(&mut self) -> Result<(), String> {
        let al = self.regs.get_al();
        let divisor = self.fetch_byte()?;
        if divisor == 0 {
            return Err("Division by zero in AAM".to_string());
        }
        let ah = al / divisor;
        let al_new = al % divisor;
        self.regs.set_ah(ah);
        self.regs.set_al(al_new);

        // Update flags based on the result in AL
        self.regs.flags.set_sign((al_new & 0x80) != 0);
        self.regs.flags.set_zero(al_new == 0);
        self.regs.flags.set_parity(al_new.count_ones() % 2 == 0);
        // Carry and overflow are undefined by the AAM instruction

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn sub_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm = (modrm & 0x38) >> 3;
        let reg = modrm & 0x07;
        let rm_val = self.regs.get_reg16(rm);
        let reg_val = self.regs.get_reg16(reg);
        let result = rm_val.wrapping_sub(reg_val);
        self.regs.set_reg16(rm, result)?;
        self.regs.flags.update_flags_sub16(rm_val, reg_val, result);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn mul_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        let rm_val = if mod_bits == 3 {
            self.regs.get_reg16(rm)
        } else {
            let addr = self.get_rm_addr(modrm)?;
            self.memory.read_word(addr)
        };

        let ax = self.regs.get_ax();
        let result = (ax as u32) * (rm_val as u32);

        // Store lower 16 bits in AX, upper 16 bits in DX
        self.regs.set_ax((result & 0xFFFF) as u16);
        self.regs.set_dx((result >> 16) as u16);

        // Set carry and overflow flags if the upper 16 bits are non-zero
        let high_word = (result >> 16) as u16;
        self.regs.flags.set_carry(high_word != 0);
        self.regs.flags.set_overflow(high_word != 0);

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn div_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        let rm_val = if mod_bits == 3 {
            self.regs.get_reg16(rm)
        } else {
            let addr = self.get_rm_addr(modrm)?;
            self.memory.read_word(addr)
        };

        // Check for division by zero
        if rm_val == 0 {
            return Err("Division by zero".to_string());
        }

        // Get 32-bit dividend from DX:AX
        let dividend = ((self.regs.get_dx() as u32) << 16) | (self.regs.get_ax() as u32);
        let divisor = rm_val as u32;

        // Check for division overflow
        let quotient = dividend / divisor;
        let remainder = dividend % divisor;

        if quotient > 0xFFFF {
            return Err("Division overflow".to_string());
        }

        // Store results
        self.regs.set_ax(quotient as u16);
        self.regs.set_dx(remainder as u16);

        Ok(())
    }

    // More arithmetic instructions can be added here...
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
    fn test_add_rm8_r8() {
        let mut cpu = setup_cpu();
        // Set AL to 5 and AH to 5
        cpu.regs.set_al(5);
        cpu.regs.set_ah(5);
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xE0 = 11 100 000
        // 11: register-to-register mode
        // 100: source register is AH (reg 4)
        // 000: destination register is AL (reg 0)
        cpu.memory.write_byte(0x100, 0xE0);
        assert!(cpu.add_rm8_r8().is_ok());
        // Result should be AL = AL + AH = 5 + 5 = 10
        assert_eq!(cpu.regs.get_al(), 10);
        // Check flags
        assert!(!cpu.regs.flags.get_carry()); // No carry expected
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero
        assert!(!cpu.regs.flags.get_sign()); // Result is positive
    }

    #[test]
    fn test_add_rm16_r16() {
        let mut cpu = setup_cpu();
        // Set AX to 0x0505
        cpu.regs.set_ax(0x0505);
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xC0 = 11 000 000
        // 11: register-to-register mode
        // 000: source register is AX (reg 0)
        // 000: destination register is AX (reg 0)
        cpu.memory.write_byte(0x100, 0xC0);
        assert!(cpu.add_rm16_r16().is_ok());
        // Result should be 0x0505 + 0x0505 = 0x0A0A
        assert_eq!(cpu.regs.get_ax(), 0x0A0A);
        // Check flags
        assert!(!cpu.regs.flags.get_carry()); // No carry expected
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero
        assert!(!cpu.regs.flags.get_sign()); // Result is positive
    }

    #[test]
    fn test_add_al_imm8() {
        let mut cpu = setup_cpu();
        // Set AL to 5
        cpu.regs.set_al(5);
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // Write immediate value 3 to memory
        cpu.memory.write_byte(0x100, 3);
        // Execute the add_al_imm8 instruction directly
        assert!(cpu.add_al_imm8().is_ok());
        // Result should be 5 + 3 = 8
        assert_eq!(cpu.regs.get_al(), 8);
        // Check flags
        assert!(!cpu.regs.flags.get_carry()); // No carry expected
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero
        assert!(!cpu.regs.flags.get_sign()); // Result is positive
    }

    #[test]
    fn test_adc_r8_rm8() {
        let mut cpu = setup_cpu();
        // Set AL to 5 and AH to 5
        cpu.regs.set_al(5);
        cpu.regs.set_ah(5);
        // Set carry flag
        cpu.regs.flags.set_carry(true);
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xE0 = 11 100 000
        // 11: register-to-register mode
        // 100: source register is AH (reg 4)
        // 000: destination register is AL (reg 0)
        cpu.memory.write_byte(0x100, 0xE0);
        assert!(cpu.adc_r8_rm8().is_ok());
        // Result should be AL = AL + AH + carry = 5 + 5 + 1 = 11 (0x0B)
        assert_eq!(cpu.regs.get_al(), 0x0B);
        // Check flags
        assert!(!cpu.regs.flags.get_carry()); // No carry out
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero
        assert!(!cpu.regs.flags.get_sign()); // Result is positive
    }

    #[test]
    fn test_cmp_al_imm8() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x0005; // AL = 5
        cpu.memory.write_byte(0, 0x03); // Compare with 3
        assert!(cpu.cmp_al_imm8().is_ok());
        assert!(cpu.regs.flags.get_carry() == false); // 5 > 3, no borrow needed
        assert!(cpu.regs.flags.get_zero() == false); // Result is not zero
    }

    #[test]
    fn test_inc_ax() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x1234;
        assert!(cpu.inc_ax().is_ok());
        assert_eq!(cpu.regs.ax, 0x1235);
    }

    #[test]
    fn test_dec_bx() {
        let mut cpu = setup_cpu();
        cpu.regs.bx = 0x1234;
        assert!(cpu.dec_bx().is_ok());
        assert_eq!(cpu.regs.bx, 0x1233);
    }

    #[test]
    fn test_imul_r16_rm16_imm16() {
        let mut cpu = setup_cpu();
        // Set AX to 2
        cpu.regs.set_ax(2);
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Set IP to a known location
        cpu.regs.ip = 0x100;
        // ModR/M byte: 0xC0 = register-to-register, dest=AX (reg 0), src=AX (reg 0)
        cpu.memory.write_byte(0x100, 0xC0);
        // Immediate value 3
        cpu.memory.write_word(0x101, 3);
        assert!(cpu.imul_r16_rm16_imm16().is_ok());
        // Result should be 2 * 3 = 6
        assert_eq!(cpu.regs.get_ax(), 6);
        // Check flags
        assert!(!cpu.regs.flags.get_carry()); // No overflow
        assert!(!cpu.regs.flags.get_overflow()); // No overflow
        assert!(!cpu.regs.flags.get_sign()); // Result is positive
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero

        // Test negative numbers
        cpu.regs.set_ax(0xFFFE); // -2 in two's complement
                                 // Reset IP to start of instruction
        cpu.regs.ip = 0x100;
        cpu.memory.write_byte(0x100, 0xC0);
        cpu.memory.write_word(0x101, 3);
        assert!(cpu.imul_r16_rm16_imm16().is_ok());
        // Result should be -2 * 3 = -6 (0xFFFA in two's complement)
        assert_eq!(cpu.regs.get_ax(), 0xFFFA);
        // Check flags
        assert!(!cpu.regs.flags.get_carry()); // No overflow
        assert!(!cpu.regs.flags.get_overflow()); // No overflow
        assert!(cpu.regs.flags.get_sign()); // Result is negative
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero
    }

    #[test]
    fn test_salc() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_carry(true);
        assert!(cpu.salc().is_ok());
        assert_eq!(cpu.regs.ax & 0xFF, 0xFF); // AL should be set to 0xFF when carry is set
    }

    #[test]
    fn test_aam() {
        let mut cpu = setup_cpu();
        // Set AL to 28 (decimal)
        cpu.regs.set_al(28);
        // Set CS to 0 so physical address matches IP
        cpu.regs.cs = 0;
        // Write divisor (base 10) to memory at IP
        cpu.regs.ip = 0x100; // Set IP to a known location
        cpu.memory.write_byte(0x100, 10);
        assert!(cpu.aam().is_ok());
        // After AAM: AH = 28 / 10 = 2, AL = 28 % 10 = 8
        assert_eq!(cpu.regs.get_ah(), 2);
        assert_eq!(cpu.regs.get_al(), 8);
        // Check flags
        assert!(!cpu.regs.flags.get_sign()); // Result is positive
        assert!(!cpu.regs.flags.get_zero()); // Result is not zero
        assert!(!cpu.regs.flags.get_parity()); // 8 has odd parity (one bit set)

        // Test with AL = 99 (max valid BCD value)
        cpu.regs.set_al(99);
        cpu.regs.cs = 0; // Ensure CS is still 0
        cpu.regs.ip = 0x200; // Set IP to a new location
        cpu.memory.write_byte(0x200, 10);
        assert!(cpu.aam().is_ok());
        // After AAM: AH = 99 / 10 = 9, AL = 99 % 10 = 9
        assert_eq!(cpu.regs.get_ah(), 9);
        assert_eq!(cpu.regs.get_al(), 9);

        // Test division by zero
        cpu.regs.set_al(1);
        cpu.regs.cs = 0; // Ensure CS is still 0
        cpu.regs.ip = 0x300; // Set IP to a new location
        cpu.memory.write_byte(0x300, 0);
        assert!(cpu.aam().is_err());
    }
}

use crate::cpu::Cpu;

impl Cpu {
    pub fn add_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let (result, carry) = rm_val.overflowing_add(reg_val);
        self.write_rm8(modrm, result)?;
        self.update_flags_add(rm_val, reg_val, result, carry);
        Ok(())
    }

    pub fn add_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm = (modrm & 0x38) >> 3;
        let reg = modrm & 0x07;
        let rm_val = self.regs.get_reg16(rm);
        let reg_val = self.regs.get_reg16(reg);
        let result = rm_val.wrapping_add(reg_val);
        self.regs.set_reg16(rm, result)?;
        self.regs.flags.update_flags_add16(rm_val, reg_val, result);
        Ok(())
    }

    pub fn add_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.ax as u8;
        let (result, carry) = al.overflowing_add(imm8);
        self.regs.ax = (self.regs.ax & 0xFF00) | (result as u16);
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
        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        let (temp, carry1) = rm_val.overflowing_add(carry);
        let (result, carry2) = reg_val.overflowing_add(temp);
        self.regs.set_reg8(reg, result)?;
        self.update_flags_add(reg_val, rm_val, result, carry1 || carry2);
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
            .set_overflow(((a ^ !b) & (a ^ result) & 0x80) != 0);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
    }

    pub fn update_flags_add16(&mut self, a: u16, b: u16, result: u16, carry: bool) {
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs
            .flags
            .set_overflow(((a ^ !b) & (a ^ result) & 0x8000) != 0);
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
        let imm16 = self.fetch_word()?;
        let reg = (modrm >> 3) & 0x07;

        // Perform signed multiplication
        let result = (rm_val as i16 as i32) * (imm16 as i16 as i32);

        // Store lower 16 bits in destination register
        self.regs.set_reg16(reg, result as u16)?;

        // Set flags
        let truncated = result as u16;
        let sign_extended = truncated as i16 as i32;

        // Set CF and OF if the result was truncated
        let overflow = sign_extended != result;
        self.regs.flags.set_carry(overflow);
        self.regs.flags.set_overflow(overflow);

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
        let divisor = self.fetch_byte()?;
        if divisor == 0 {
            return Err("Division by zero in AAM".to_string());
        }
        let al = self.regs.get_al();
        let ah = al / divisor;
        let al_new = al % divisor;
        self.regs.set_ah(ah);
        self.regs.set_al(al_new);
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

use crate::cpu::CPU;

impl CPU {
    pub(crate) fn and_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val & reg_val;
        self.write_rm8(modrm, result)?;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn or_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val | reg_val;
        self.write_rm8(modrm, result)?;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val ^ reg_val;
        self.write_rm8(modrm, result)?;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        let result = rm_val ^ reg_val;
        self.regs.set_reg8(reg, result)?;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn test_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val & reg_val;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn test_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.get_al();
        let result = al & imm8;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_al_imm8(&mut self) -> Result<(), String> {
        let imm = self.fetch_byte()?;
        let al = self.regs.get_al();
        let result = al ^ imm;
        self.regs.set_al(result);
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg_val = self.regs.get_reg16((modrm >> 3) & 0x07);
        let result = rm_val ^ reg_val;
        self.write_rm16(modrm, result)?;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg16(reg);
        let result = rm_val ^ reg_val;
        self.regs.set_reg16(reg, result)?;
        
        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity((result as u8).count_ones() % 2 == 0);
        Ok(())
    }

    // More logic instructions can be added here...
} 
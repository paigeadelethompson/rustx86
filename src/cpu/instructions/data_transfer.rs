use crate::cpu::CPU;

impl CPU {
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

    pub(crate) fn mov_al_moffs8(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let val = self.memory.read_byte(self.get_physical_address(self.regs.ds, offset));
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

    pub(crate) fn mov_ax_moffs16(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let val = self.memory.read_word(self.get_physical_address(self.regs.ds, offset));
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
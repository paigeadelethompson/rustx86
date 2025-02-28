use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn jmp_near(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let _old_ip = self.regs.ip;
        self.regs.ip = self.regs.ip.wrapping_add(offset);
        Ok(())
    }

    pub(crate) fn jmp_far(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let segment = self.fetch_word()?;
        self.regs.cs = segment;
        self.regs.ip = offset;
        Ok(())
    }

    pub fn jmp_short(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let _old_ip = self.regs.ip;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub(crate) fn call_near(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()? as i16;
        self.push_word(self.regs.ip)?;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn call_far(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let segment = self.fetch_word()?;
        let old_cs = self.regs.cs;
        let _old_ip = self.regs.ip;
        self.push_word(old_cs)?;
        self.push_word(self.regs.ip)?;
        self.regs.ip = offset;
        self.regs.cs = segment;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn ret_near(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        self.regs.ip = ip;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn ret_far(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        let cs = self.pop_word()?;
        self.regs.ip = ip;
        self.regs.cs = cs;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn ret_near_imm16(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        let imm16 = self.fetch_word()?;
        self.regs.ip = ip;
        self.regs.sp = self.regs.sp.wrapping_add(imm16);
        Ok(())
    }

    pub fn jcxz(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.cx == 0 {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn loop_cx(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let _old_cx = self.regs.cx;
        self.regs.cx = self.regs.cx.wrapping_sub(1);
        if self.regs.cx != 0 {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jnz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jo_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_overflow() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jno_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_overflow() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_carry() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jnb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_carry() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_carry() || self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jnbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_carry() && !self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }
}

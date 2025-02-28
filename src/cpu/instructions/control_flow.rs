use crate::cpu::CPU;

impl CPU {
    pub fn jmp_near(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()? as i16;
        let old_ip = self.regs.ip;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub fn jmp_far(&mut self) -> Result<(), String> {
        let ip = self.fetch_word()?;
        let cs = self.fetch_word()?;
        self.regs.cs = cs;
        self.regs.ip = ip;
        Ok(())
    }

    pub fn jmp_short(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let old_ip = self.regs.ip;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub fn call_near(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()? as i16;
        let return_addr = self.regs.ip;
        self.push_word(return_addr)?;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub fn call_far(&mut self) -> Result<(), String> {
        let ip = self.fetch_word()?;
        let cs = self.fetch_word()?;
        let old_cs = self.regs.cs;
        let old_ip = self.regs.ip;
        self.push_word(old_cs)?;
        self.push_word(old_ip)?;
        self.regs.cs = cs;
        self.regs.ip = ip;
        Ok(())
    }

    pub fn ret_near(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        self.regs.ip = ip;
        Ok(())
    }

    pub fn ret_far(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        let cs = self.pop_word()?;
        self.regs.ip = ip;
        self.regs.cs = cs;
        Ok(())
    }

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
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn loop_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let old_cx = self.regs.cx;
        self.regs.cx = self.regs.cx.wrapping_sub(1);
        if self.regs.cx != 0 {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.zero_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jnz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.zero_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jo_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.overflow_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jno_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.overflow_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.carry_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jnb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.carry_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.carry_flag() || self.regs.flags.zero_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub fn jnbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.carry_flag() && !self.regs.flags.zero_flag() {
            let old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }
} 
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

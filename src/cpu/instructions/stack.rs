use crate::cpu::CPU;
use crate::cpu::fmt::Error;
use crate::memory::Memory;

impl CPU {
    pub(crate) fn push_ax(&mut self) -> Result<(), String> {
        self.push_word(self.regs.ax)?;
        Ok(())
    }

    pub(crate) fn push_cx(&mut self) -> Result<(), String> {
        self.push_word(self.regs.cx)?;
        Ok(())
    }

    pub(crate) fn push_dx(&mut self) -> Result<(), String> {
        self.push_word(self.regs.dx)?;
        Ok(())
    }

    pub(crate) fn push_bx(&mut self) -> Result<(), String> {
        self.push_word(self.regs.bx)?;
        Ok(())
    }

    pub(crate) fn push_es(&mut self) -> Result<(), String> {
        self.push_word(self.regs.es)?;
        Ok(())
    }

    pub(crate) fn push_word(&mut self, value: u16) -> Result<(), String> {
        // Decrement SP by 2 (word size)
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        
        // Calculate physical address for the stack
        let address = (self.regs.ss as u32) << 4 | (self.regs.sp as u32);
        
        // Write the word to memory
        self.memory.write_word(address, value);
        
        Ok(())
    }

    pub(crate) fn pop_cx(&mut self) -> Result<(), String> {
        self.regs.cx = self.pop_word()?;
        Ok(())
    }

    pub(crate) fn pop_dx(&mut self) -> Result<(), String> {
        self.regs.dx = self.pop_word()?;
        Ok(())
    }

    pub(crate) fn pop_bx(&mut self) -> Result<(), String> {
        self.regs.bx = self.pop_word()?;
        Ok(())
    }

    pub(crate) fn pop_ax(&mut self) -> Result<(), String> {
        self.regs.ax = self.pop_word()?;
        Ok(())
    }

    pub(crate) fn pop_es(&mut self) -> Result<(), String> {
        self.regs.es = self.pop_word()?;
        Ok(())
    }

    pub fn leave(&mut self) -> Result<(), String> {
        self.regs.sp = self.regs.bp;
        self.regs.bp = self.pop_word()?;
        Ok(())
    }

    pub fn enter(&mut self) -> Result<(), String> {
        let frame_size = self.fetch_word()?;
        let nesting_level = self.fetch_byte()? & 0x1F;
        
        // Save BP
        self.push_word(self.regs.bp)?;
        
        let frame_temp = self.regs.sp;
        
        // Copy previous stack frame pointers if nesting level > 0
        if nesting_level > 0 {
            for level in 1..=nesting_level {
                self.regs.bp -= 2;
                let addr = ((self.regs.ss as u32) << 4) + self.regs.bp as u32;
                let temp = self.read_word(addr)?;
                self.push_word(temp)?;
            }
            // Push the temporary frame pointer value
            self.push_word(frame_temp)?;
        }
        
        self.regs.bp = frame_temp;
        self.regs.sp -= frame_size;
        
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
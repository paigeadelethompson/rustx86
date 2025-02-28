use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn cli(&mut self) -> Result<(), String> {
        self.regs.flags.set_interrupt(false);
        Ok(())
    }

    pub(crate) fn sti(&mut self) -> Result<(), String> {
        self.regs.flags.set_interrupt(true);
        Ok(())
    }

    pub(crate) fn clc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(false);
        Ok(())
    }

    pub(crate) fn stc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(true);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn cmc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(!self.regs.flags.get_carry());
        Ok(())
    }

    pub(crate) fn cld(&mut self) -> Result<(), String> {
        self.regs.flags.set_direction(false);
        Ok(())
    }

    pub(crate) fn std(&mut self) -> Result<(), String> {
        self.regs.flags.set_direction(true);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn sahf(&mut self) -> Result<(), String> {
        let ah = (self.regs.ax >> 8) as u8;
        self.regs.flags.set_from_byte(ah);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn lahf(&mut self) -> Result<(), String> {
        let flags = self.regs.flags.as_byte();
        self.regs.ax = (self.regs.ax & 0x00FF) | ((flags as u16) << 8);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn pushf(&mut self) -> Result<(), String> {
        let flags = self.regs.flags.as_u16();
        self.push_word(flags)
    }

    #[allow(dead_code)]
    pub(crate) fn popf(&mut self) -> Result<(), String> {
        let flags = self.pop_word()?;
        self.regs.flags.set_from_u16(flags);
        Ok(())
    }
}

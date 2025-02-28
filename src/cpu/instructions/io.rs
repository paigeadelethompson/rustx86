use crate::cpu::CPU;

impl CPU {
    pub(crate) fn in_al_imm8(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        let value = self.io_read_byte(port as u16)?;
        self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);
        Ok(())
    }

    pub(crate) fn in_ax_imm8(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        let value = self.io_read_word(port as u16)?;
        self.regs.ax = value;
        Ok(())
    }

    pub(crate) fn in_al_dx(&mut self) -> Result<(), String> {
        let value = self.io_read_byte(self.regs.dx)?;
        self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);
        Ok(())
    }

    pub(crate) fn in_ax_dx(&mut self) -> Result<(), String> {
        let value = self.io_read_word(self.regs.dx)?;
        self.regs.ax = value;
        Ok(())
    }

    pub(crate) fn out_imm8_al(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        let value = self.regs.ax as u8;
        self.io_write_byte(port as u16, value)?;
        Ok(())
    }

    pub(crate) fn out_imm8_ax(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()?;
        self.io_write_word(port as u16, self.regs.ax)?;
        Ok(())
    }

    pub(crate) fn out_dx_al(&mut self) -> Result<(), String> {
        let value = self.regs.ax as u8;
        self.io_write_byte(self.regs.dx, value)?;
        Ok(())
    }

    pub(crate) fn out_dx_ax(&mut self) -> Result<(), String> {
        self.io_write_word(self.regs.dx, self.regs.ax)?;
        Ok(())
    }

    // Helper functions
    pub(crate) fn io_read_byte(&mut self, _port: u16) -> Result<u8, String> {
        Ok(0)
    }

    pub(crate) fn io_read_word(&mut self, _port: u16) -> Result<u16, String> {
        Ok(0)
    }

    pub(crate) fn io_write_byte(&mut self, _port: u16, _value: u8) -> Result<(), String> {
        Ok(())
    }

    pub(crate) fn io_write_word(&mut self, _port: u16, _value: u16) -> Result<(), String> {
        Ok(())
    }
} 
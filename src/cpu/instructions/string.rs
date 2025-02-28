use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn movsb(&mut self) -> Result<(), String> {
        let src_addr = self.get_physical_address(self.regs.ds, self.regs.si);
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let value = self.memory.read_byte(src_addr);
        self.memory.write_byte(dst_addr, value);

        if !self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_add(1);
            self.regs.di = self.regs.di.wrapping_add(1);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(1);
            self.regs.di = self.regs.di.wrapping_sub(1);
        }
        Ok(())
    }

    pub(crate) fn movsw(&mut self) -> Result<(), String> {
        let src_addr = self.get_physical_address(self.regs.ds, self.regs.si);
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let value = self.memory.read_word(src_addr);
        self.memory.write_word(dst_addr, value);

        if !self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_add(2);
            self.regs.di = self.regs.di.wrapping_add(2);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(2);
            self.regs.di = self.regs.di.wrapping_sub(2);
        }
        Ok(())
    }

    pub(crate) fn lodsb(&mut self) -> Result<(), String> {
        let src_addr = self.get_physical_address(self.regs.ds, self.regs.si);
        let value = self.memory.read_byte(src_addr);
        self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);

        if !self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_add(1);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(1);
        }
        Ok(())
    }

    pub(crate) fn lodsw(&mut self) -> Result<(), String> {
        let src_addr = self.get_physical_address(self.regs.ds, self.regs.si);
        let value = self.memory.read_word(src_addr);
        self.regs.ax = value;

        if !self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_add(2);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(2);
        }
        Ok(())
    }

    pub(crate) fn stosb(&mut self) -> Result<(), String> {
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let value = self.regs.ax as u8;
        self.memory.write_byte(dst_addr, value);

        if !self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
        }
        Ok(())
    }

    pub(crate) fn stosw(&mut self) -> Result<(), String> {
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        self.memory.write_word(dst_addr, self.regs.ax);

        if !self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_add(2);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(2);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn cmpsb(&mut self) -> Result<(), String> {
        let src_addr = self.get_physical_address(self.regs.ds, self.regs.si);
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let src_val = self.memory.read_byte(src_addr);
        let dst_val = self.memory.read_byte(dst_addr);

        let (result, carry) = dst_val.overflowing_sub(src_val);
        self.update_flags_sub(dst_val, src_val, result, carry);

        if !self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_add(1);
            self.regs.di = self.regs.di.wrapping_add(1);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(1);
            self.regs.di = self.regs.di.wrapping_sub(1);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn cmpsw(&mut self) -> Result<(), String> {
        let src_addr = self.get_physical_address(self.regs.ds, self.regs.si);
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let src_val = self.memory.read_word(src_addr);
        let dst_val = self.memory.read_word(dst_addr);

        let (result, carry) = dst_val.overflowing_sub(src_val);
        self.update_flags_sub16(dst_val, src_val, result, carry);

        if !self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_add(2);
            self.regs.di = self.regs.di.wrapping_add(2);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(2);
            self.regs.di = self.regs.di.wrapping_sub(2);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn scasb(&mut self) -> Result<(), String> {
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let dst_val = self.memory.read_byte(dst_addr);
        let al = self.regs.ax as u8;

        let (result, carry) = al.overflowing_sub(dst_val);
        self.update_flags_sub(al, dst_val, result, carry);

        if !self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn scasw(&mut self) -> Result<(), String> {
        let dst_addr = self.get_physical_address(self.regs.es, self.regs.di);
        let dst_val = self.memory.read_word(dst_addr);

        let (result, carry) = self.regs.ax.overflowing_sub(dst_val);
        self.update_flags_sub16(self.regs.ax, dst_val, result, carry);

        if !self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_add(2);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(2);
        }
        Ok(())
    }
}

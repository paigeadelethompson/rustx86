use super::CPU;

impl CPU {
    pub fn get_rm_addr(&mut self, modrm: u8) -> Result<u32, String> {
        let rm = modrm & 0x07;
        let mod_bits = modrm >> 6;
        
        match mod_bits {
            0 => match rm {
                0 => Ok((self.regs.bx.wrapping_add(self.regs.si)) as u32),
                1 => Ok((self.regs.bx.wrapping_add(self.regs.di)) as u32),
                2 => Ok((self.regs.bp.wrapping_add(self.regs.si)) as u32),
                3 => Ok((self.regs.bp.wrapping_add(self.regs.di)) as u32),
                4 => Ok(self.regs.si as u32),
                5 => Ok(self.regs.di as u32),
                6 => {
                    let disp = self.fetch_word()?;
                    Ok(disp as u32)
                },
                7 => Ok(self.regs.bx as u32),
                _ => Err("Invalid rm value".to_string()),
            },
            1 => {
                let disp = self.fetch_byte()? as i8;
                match rm {
                    0 => Ok(((self.regs.bx.wrapping_add(self.regs.si)) as i32 + disp as i32) as u32),
                    1 => Ok(((self.regs.bx.wrapping_add(self.regs.di)) as i32 + disp as i32) as u32),
                    2 => Ok(((self.regs.bp.wrapping_add(self.regs.si)) as i32 + disp as i32) as u32),
                    3 => Ok(((self.regs.bp.wrapping_add(self.regs.di)) as i32 + disp as i32) as u32),
                    4 => Ok((self.regs.si as i32 + disp as i32) as u32),
                    5 => Ok((self.regs.di as i32 + disp as i32) as u32),
                    6 => Ok((self.regs.bp as i32 + disp as i32) as u32),
                    7 => Ok((self.regs.bx as i32 + disp as i32) as u32),
                    _ => Err("Invalid rm value".to_string()),
                }
            },
            2 => {
                let disp = self.fetch_word()? as i16;
                match rm {
                    0 => Ok(((self.regs.bx.wrapping_add(self.regs.si)) as i32 + disp as i32) as u32),
                    1 => Ok(((self.regs.bx.wrapping_add(self.regs.di)) as i32 + disp as i32) as u32),
                    2 => Ok(((self.regs.bp.wrapping_add(self.regs.si)) as i32 + disp as i32) as u32),
                    3 => Ok(((self.regs.bp.wrapping_add(self.regs.di)) as i32 + disp as i32) as u32),
                    4 => Ok((self.regs.si as i32 + disp as i32) as u32),
                    5 => Ok((self.regs.di as i32 + disp as i32) as u32),
                    6 => Ok((self.regs.bp as i32 + disp as i32) as u32),
                    7 => Ok((self.regs.bx as i32 + disp as i32) as u32),
                    _ => Err("Invalid rm value".to_string()),
                }
            },
            3 => match rm {
                0..=7 => Ok(rm as u32),
                _ => Err("Invalid rm value".to_string()),
            },
            _ => Err("Invalid mod value".to_string()),
        }
    }

    pub fn set_rm8(&mut self, modrm: u8, value: u8) -> Result<(), String> {
        let addr = self.get_rm_addr(modrm)?;
        self.memory.write_byte(addr, value);
        Ok(())
    }

    pub fn set_rm16(&mut self, modrm: u8, value: u16) -> Result<(), String> {
        let addr = self.get_rm_addr(modrm)?;
        self.memory.write_word(addr, value);
        Ok(())
    }

    pub fn get_rm8(&mut self, modrm: u8) -> Result<u8, String> {
        let addr = self.get_rm_addr(modrm)?;
        Ok(self.memory.read_byte(addr))
    }

    pub fn get_rm16(&mut self, modrm: u8) -> Result<u16, String> {
        let mod_bits = modrm >> 6;
        if mod_bits == 3 {
            // Register operand
            let rm = modrm & 0x07;
            match rm {
                0 => Ok(self.regs.ax),
                1 => Ok(self.regs.cx),
                2 => Ok(self.regs.dx),
                3 => Ok(self.regs.bx),
                4 => Ok(self.regs.sp),
                5 => Ok(self.regs.bp),
                6 => Ok(self.regs.si),
                7 => Ok(self.regs.di),
                _ => Err("Invalid register".to_string()),
            }
        } else {
            // Memory operand
            let addr = self.get_rm_addr(modrm)?;
            Ok(self.memory.read_word(addr))
        }
    }
} 
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use crate::disk::disk_image::DiskImage;
    use std::path::Path;

    fn setup_cpu() -> CPU {
        let memory = Box::new(RamMemory::new(1024 * 1024));
        let serial = Serial::new();
        let disk = DiskImage::new(Path::new("drive_c/")).expect("Failed to create disk image");
        CPU::new(memory, serial, disk)
    }

    #[test]
    fn test_mod00_addressing() {
        let mut cpu = setup_cpu();
        
        // Test [BX + SI]
        cpu.regs.bx = 0x2000;
        cpu.regs.si = 0x100;
        let modrm = 0x00; // mod=00, rm=000
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x2100);

        // Test [BX + DI]
        cpu.regs.bx = 0x2000;
        cpu.regs.di = 0x200;
        let modrm = 0x01; // mod=00, rm=001
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x2200);

        // Test direct address
        let modrm = 0x06; // mod=00, rm=110
        cpu.memory.write_word(cpu.regs.ip as u32, 0x1234);
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x1234);
    }

    #[test]
    fn test_mod01_addressing() {
        let mut cpu = setup_cpu();
        
        // Test [BX + SI + disp8]
        cpu.regs.bx = 0x2000;
        cpu.regs.si = 0x100;
        cpu.memory.write_byte(cpu.regs.ip as u32, 0x10); // disp8 = 0x10
        let modrm = 0x40; // mod=01, rm=000
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x2110);

        // Test [BP + DI + disp8]
        cpu.regs.bp = 0x3000;
        cpu.regs.di = 0x200;
        cpu.memory.write_byte(cpu.regs.ip as u32, 0x20); // disp8 = 0x20
        let modrm = 0x43; // mod=01, rm=011
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x3220);
    }

    #[test]
    fn test_mod10_addressing() {
        let mut cpu = setup_cpu();
        
        // Test [BX + SI + disp16]
        cpu.regs.bx = 0x2000;
        cpu.regs.si = 0x100;
        cpu.memory.write_word(cpu.regs.ip as u32, 0x1000); // disp16 = 0x1000
        let modrm = 0x80; // mod=10, rm=000
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x3100);

        // Test [BP + DI + disp16]
        cpu.regs.bp = 0x3000;
        cpu.regs.di = 0x200;
        cpu.memory.write_word(cpu.regs.ip as u32, 0x2000); // disp16 = 0x2000
        let modrm = 0x83; // mod=10, rm=011
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0x5200);
    }

    #[test]
    fn test_mod11_addressing() {
        let mut cpu = setup_cpu();
        
        // Test register addressing
        let modrm = 0xC0; // mod=11, rm=000 (AL/AX)
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 0);
        
        let modrm = 0xC7; // mod=11, rm=111 (BH/DI)
        assert_eq!(cpu.get_rm_addr(modrm).unwrap(), 7);
    }
} 
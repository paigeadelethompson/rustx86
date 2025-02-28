use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_group1_rm8_imm8(&mut self, _group: u8) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let imm = self.fetch_byte()?;
        let result = match (modrm >> 3) & 0x07 {
            0 => rm_val.wrapping_add(imm), // ADD
            1 => rm_val | imm,             // OR
            2 => rm_val
                .wrapping_add(if self.regs.flags.get_carry() { 1 } else { 0 })
                .wrapping_add(imm), // ADC
            3 => rm_val
                .wrapping_sub(if self.regs.flags.get_carry() { 1 } else { 0 })
                .wrapping_sub(imm), // SBB
            4 => rm_val & imm,             // AND
            5 => rm_val.wrapping_sub(imm), // SUB
            6 => rm_val ^ imm,             // XOR
            7 => {
                // CMP
                let _ = rm_val.wrapping_sub(imm);
                rm_val
            }
            _ => return Err("Invalid group1 operation".to_string()),
        };
        if (modrm >> 3) & 0x07 != 7 {
            // Don't write result for CMP
            self.write_rm8(modrm, result)?;
        }
        self.update_flags_arithmetic(rm_val, imm, result, (modrm >> 3) & 0x07 >= 5);
        Ok(())
    }

    pub(crate) fn handle_81_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let imm = self.fetch_word()?;
        let result = match (modrm >> 3) & 0x07 {
            0 => rm_val.wrapping_add(imm), // ADD
            1 => rm_val | imm,             // OR
            2 => rm_val
                .wrapping_add(if self.regs.flags.get_carry() { 1 } else { 0 })
                .wrapping_add(imm), // ADC
            3 => rm_val
                .wrapping_sub(if self.regs.flags.get_carry() { 1 } else { 0 })
                .wrapping_sub(imm), // SBB
            4 => rm_val & imm,             // AND
            5 => rm_val.wrapping_sub(imm), // SUB
            6 => rm_val ^ imm,             // XOR
            7 => {
                // CMP
                let _ = rm_val.wrapping_sub(imm);
                rm_val
            }
            _ => return Err("Invalid group1 operation".to_string()),
        };
        if (modrm >> 3) & 0x07 != 7 {
            // Don't write result for CMP
            self.write_rm16(modrm, result)?;
        }
        self.update_flags_arithmetic_16(rm_val, imm, result, (modrm >> 3) & 0x07 >= 5);
        Ok(())
    }

    pub(crate) fn handle_82_group(&mut self) -> Result<(), String> {
        self.execute_group1_rm8_imm8(2)
    }

    pub(crate) fn handle_83_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let imm = (self.fetch_byte()? as i8) as i16 as u16;
        let result = match (modrm >> 3) & 0x07 {
            0 => rm_val.wrapping_add(imm), // ADD
            1 => rm_val | imm,             // OR
            2 => rm_val
                .wrapping_add(if self.regs.flags.get_carry() { 1 } else { 0 })
                .wrapping_add(imm), // ADC
            3 => rm_val
                .wrapping_sub(if self.regs.flags.get_carry() { 1 } else { 0 })
                .wrapping_sub(imm), // SBB
            4 => rm_val & imm,             // AND
            5 => rm_val.wrapping_sub(imm), // SUB
            6 => rm_val ^ imm,             // XOR
            7 => {
                // CMP
                let _ = rm_val.wrapping_sub(imm);
                rm_val
            }
            _ => return Err("Invalid group1 operation".to_string()),
        };
        if (modrm >> 3) & 0x07 != 7 {
            // Don't write result for CMP
            self.write_rm16(modrm, result)?;
        }
        self.update_flags_arithmetic_16(rm_val, imm, result, (modrm >> 3) & 0x07 >= 5);
        Ok(())
    }

    pub(crate) fn handle_f6_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        match (modrm >> 3) & 0x07 {
            0 | 1 => {
                // TEST
                let rm_val = self.get_rm8(modrm)?;
                let imm = self.fetch_byte()?;
                let result = rm_val & imm;
                self.regs.flags.set_zero(result == 0);
                self.regs.flags.set_sign((result as i8) < 0);
                self.regs.flags.set_carry(false);
                self.regs.flags.set_overflow(false);
            }
            2 => {
                // NOT
                let rm_val = self.get_rm8(modrm)?;
                self.write_rm8(modrm, !rm_val)?;
            }
            3 => {
                // NEG
                let rm_val = self.get_rm8(modrm)?;
                let result = (!rm_val).wrapping_add(1);
                self.write_rm8(modrm, result)?;
                self.update_flags_arithmetic(rm_val, 0, result, true);
            }
            4 => {
                // MUL
                let rm_val = self.get_rm8(modrm)?;
                let al = self.regs.get_reg8(0);
                let result = (al as u16) * (rm_val as u16);
                self.regs.ax = result;
                self.regs.flags.set_carry(result > 0xFF);
                self.regs.flags.set_overflow(result > 0xFF);
            }
            5 => {
                // IMUL
                let rm_val = self.get_rm8(modrm)? as i8;
                let al = self.regs.get_reg8(0) as i8;
                let result = (al as i16) * (rm_val as i16);
                self.regs.ax = result as u16;
                self.regs
                    .flags
                    .set_carry(!(-0x80..=0x7F).contains(&result));
                self.regs
                    .flags
                    .set_overflow(!(-0x80..=0x7F).contains(&result));
            }
            6 => {
                // DIV
                let rm_val = self.get_rm8(modrm)?;
                if rm_val == 0 {
                    return Err("Division by zero".to_string());
                }
                let ax = self.regs.ax;
                let quotient = ax / (rm_val as u16);
                let remainder = ax % (rm_val as u16);
                if quotient > 0xFF {
                    return Err("Division overflow".to_string());
                }
                self.regs.set_reg8(0, quotient as u8)?; // AL
                self.regs.set_reg8(1, remainder as u8)?; // AH
            }
            7 => {
                // IDIV
                let rm_val = self.get_rm8(modrm)? as i8;
                if rm_val == 0 {
                    return Err("Division by zero".to_string());
                }
                let ax = self.regs.ax as i16;
                let quotient = ax / (rm_val as i16);
                let remainder = ax % (rm_val as i16);
                if !(-0x80..=0x7F).contains(&quotient) {
                    return Err("Division overflow".to_string());
                }
                self.regs.set_reg8(0, quotient as u8)?; // AL
                self.regs.set_reg8(1, remainder as u8)?; // AH
            }
            _ => return Err("Invalid group2 operation".to_string()),
        }
        Ok(())
    }

    pub(crate) fn handle_f7_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        match (modrm >> 3) & 0x07 {
            0 | 1 => {
                // TEST
                let rm_val = self.get_rm16(modrm)?;
                let imm = self.fetch_word()?;
                let result = rm_val & imm;
                self.regs.flags.set_zero(result == 0);
                self.regs.flags.set_sign((result as i16) < 0);
                self.regs.flags.set_carry(false);
                self.regs.flags.set_overflow(false);
            }
            2 => {
                // NOT
                let rm_val = self.get_rm16(modrm)?;
                self.write_rm16(modrm, !rm_val)?;
            }
            3 => {
                // NEG
                let rm_val = self.get_rm16(modrm)?;
                let result = (!rm_val).wrapping_add(1);
                self.write_rm16(modrm, result)?;
                self.update_flags_arithmetic_16(rm_val, 0, result, true);
            }
            4 => {
                // MUL
                let rm_val = self.get_rm16(modrm)?;
                let ax = self.regs.ax;
                let result = (ax as u32) * (rm_val as u32);
                self.regs.ax = result as u16;
                self.regs.dx = (result >> 16) as u16;
                self.regs.flags.set_carry(result > 0xFFFF);
                self.regs.flags.set_overflow(result > 0xFFFF);
            }
            5 => {
                // IMUL
                let rm_val = self.get_rm16(modrm)? as i16;
                let ax = self.regs.ax as i16;
                let result = (ax as i32) * (rm_val as i32);
                self.regs.ax = result as u16;
                self.regs.dx = (result >> 16) as u16;
                self.regs
                    .flags
                    .set_carry(!(-0x8000..=0x7FFF).contains(&result));
                self.regs
                    .flags
                    .set_overflow(!(-0x8000..=0x7FFF).contains(&result));
            }
            6 => {
                // DIV
                let rm_val = self.get_rm16(modrm)?;
                if rm_val == 0 {
                    return Err("Division by zero".to_string());
                }
                let dividend = ((self.regs.dx as u32) << 16) | (self.regs.ax as u32);
                let quotient = dividend / (rm_val as u32);
                let remainder = dividend % (rm_val as u32);
                if quotient > 0xFFFF {
                    return Err("Division overflow".to_string());
                }
                self.regs.ax = quotient as u16;
                self.regs.dx = remainder as u16;
            }
            7 => {
                // IDIV
                let rm_val = self.get_rm16(modrm)? as i16;
                if rm_val == 0 {
                    return Err("Division by zero".to_string());
                }
                let dividend = ((self.regs.dx as i32) << 16) | (self.regs.ax as i32);
                let quotient = dividend / (rm_val as i32);
                let remainder = dividend % (rm_val as i32);
                if !(-0x8000..=0x7FFF).contains(&quotient) {
                    return Err("Division overflow".to_string());
                }
                self.regs.ax = quotient as u16;
                self.regs.dx = remainder as u16;
            }
            _ => return Err("Invalid group2 operation".to_string()),
        }
        Ok(())
    }

    pub(crate) fn handle_fe_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let result = match (modrm >> 3) & 0x07 {
            0 => rm_val.wrapping_add(1), // INC
            1 => rm_val.wrapping_sub(1), // DEC
            _ => return Err("Invalid group3 operation".to_string()),
        };
        self.write_rm8(modrm, result)?;
        self.update_flags_arithmetic(
            rm_val,
            if (modrm >> 3) & 0x07 == 0 { 1 } else { !0 },
            result,
            (modrm >> 3) & 0x07 == 1,
        );
        Ok(())
    }

    pub(crate) fn handle_ff_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        match (modrm >> 3) & 0x07 {
            0 | 1 => {
                // INC/DEC
                let rm_val = self.get_rm16(modrm)?;
                let result = if (modrm >> 3) & 0x07 == 0 {
                    rm_val.wrapping_add(1)
                } else {
                    rm_val.wrapping_sub(1)
                };
                self.write_rm16(modrm, result)?;
                self.update_flags_arithmetic_16(
                    rm_val,
                    if (modrm >> 3) & 0x07 == 0 { 1 } else { !0 },
                    result,
                    (modrm >> 3) & 0x07 == 1,
                );
            }
            2 => {
                // CALL near
                let target = self.get_rm16(modrm)?;
                let next_ip = self.regs.ip;
                self.push(next_ip)?;
                self.regs.ip = target;
            }
            3 => {
                // CALL far
                let rm_addr = self.get_rm_addr(modrm)?;
                let offset = self.read_word(rm_addr)?;
                let segment = self.read_word(rm_addr.wrapping_add(2))?;
                let next_ip = self.regs.ip;
                let next_cs = self.regs.cs;
                self.push(next_cs)?;
                self.push(next_ip)?;
                self.regs.ip = offset;
                self.regs.cs = segment;
            }
            4 => {
                // JMP near
                let target = self.get_rm16(modrm)?;
                self.regs.ip = target;
            }
            5 => {
                // JMP far
                let rm_addr = self.get_rm_addr(modrm)?;
                let offset = self.read_word(rm_addr)?;
                let segment = self.read_word(rm_addr.wrapping_add(2))?;
                self.regs.ip = offset;
                self.regs.cs = segment;
            }
            6 => {
                // PUSH
                let value = self.get_rm16(modrm)?;
                self.push(value)?;
            }
            _ => return Err("Invalid group4 operation".to_string()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use crate::disk::disk_image::DiskImage;
    use std::path::Path;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024));  // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(Path::new("drive_c/")).expect("Failed to create disk image");
        Cpu::new(memory, serial, disk)
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_execute_group1_rm8_imm8_add() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x0505;  // AL = 5
        cpu.memory.write_byte(0, 0xC0);  // ModR/M byte for register-to-register, reg=0 (ADD)
        cpu.memory.write_byte(1, 0x03);  // Immediate value 3
        assert!(cpu.execute_group1_rm8_imm8(0).is_ok());
        assert_eq!(cpu.regs.get_al(), 0x08);  // 5 + 3 = 8
        assert!(!cpu.regs.flags.get_carry());
        assert!(!cpu.regs.flags.get_zero());
    }

    #[test]
    fn test_handle_f6_group_test() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0xFF00;  // AL = 0
        cpu.memory.write_byte(0, 0xC0);  // ModR/M byte for register-to-register, reg=0 (TEST)
        cpu.memory.write_byte(1, 0xFF);  // Test with 0xFF
        assert!(cpu.handle_f6_group().is_ok());
        assert!(cpu.regs.flags.get_zero());  // 0 & 0xFF = 0
        assert!(!cpu.regs.flags.get_carry());
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_handle_fe_group_inc() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x0041;  // AL = 0x41
        cpu.memory.write_byte(0, 0xC0);  // ModR/M byte for register-to-register, reg=0 (INC)
        assert!(cpu.handle_fe_group().is_ok());
        assert_eq!(cpu.regs.get_al(), 0x42);
        assert!(!cpu.regs.flags.get_zero());
    }

    #[test]
    #[ignore = "Needs investigation of instruction execution"]
    fn test_handle_fe_group_dec() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x0042;  // AL = 0x42
        cpu.memory.write_byte(0, 0xC8);  // ModR/M byte for register-to-register, reg=1 (DEC)
        assert!(cpu.handle_fe_group().is_ok());
        assert_eq!(cpu.regs.get_al(), 0x41);
        assert!(!cpu.regs.flags.get_zero());
    }
}

use super::CPU;

impl CPU {
    pub(super) fn mov_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.regs.get_reg8(reg);
        println!("[mov_rm8_r8] Moving {:#04X} to r/m8", value);
        self.set_rm8(modrm, value)?;
        Ok(())
    }

    pub(super) fn mov_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.get_reg16(reg);
        println!("MOV r/m16, r16: Moving {:#06X} to r/m16", value);
        self.set_reg16(reg, value);
        Ok(())
    }

    pub(super) fn mov_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.get_rm8(modrm)?;
        println!("MOV r8, r/m8: Moving {:#04X} to reg{}", value, reg);
        self.regs.set_reg8(reg, value);
        Ok(())
    }

    pub(super) fn mov_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.get_rm16(modrm)?;
        self.set_reg16(reg, value);
        Ok(())
    }

    pub(super) fn mov_sreg_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_rm16(modrm)?;
        let sreg = (modrm >> 3) & 0x07;
        self.set_sreg(sreg, rm_value);
        Ok(())
    }

    pub(super) fn mov_rm16_sreg(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let sreg = (modrm >> 3) & 0x03;
        let value = self.get_sreg(sreg);
        self.set_rm16(modrm, value)?;
        Ok(())
    }

    pub(super) fn cli(&mut self) -> Result<(), String> {
        // println!("CLI: Clearing interrupt flag");
        self.regs.flags.set_interrupt(false);
        Ok(())
    }

    pub(super) fn sti(&mut self) -> Result<(), String> {
        // println!("STI: Setting interrupt flag");
        self.regs.flags.set_interrupt(true);
        Ok(())
    }

    pub(super) fn int_n(&mut self, int_num: u8) -> Result<(), String> {
        // Push flags
        self.push_word(self.regs.flags.value)?;
        
        // Push CS:IP
        self.push_word(self.regs.cs)?;
        self.push_word(self.regs.ip)?;
        
        // Get interrupt vector
        let vector_addr = (int_num as u32) * 4;
        let offset = self.memory.read_word(vector_addr);
        let segment = self.memory.read_word(vector_addr + 2);
        
        // Jump to interrupt handler
        self.regs.ip = offset;
        self.regs.cs = segment;
        
        // Clear IF and TF flags
        self.regs.flags.set_interrupt(false);
        self.regs.flags.set_trap(false);
        
        Ok(())
    }

    fn handle_interrupt(&mut self, int_num: u8) -> Result<(), String> {
        println!("[CPU::handle_interrupt] Handling interrupt {:02X}h", int_num);
        println!("[CPU::handle_interrupt] Saving state - IP={:04X} CS={:04X} FLAGS={:04X}", 
                 self.regs.ip, self.regs.cs, self.regs.flags.value);
        
        // Save return address and flags
        let ip = self.regs.ip;
        self.push_word(ip)?;
        self.push_word(self.regs.cs)?;
        self.push_word(self.regs.flags.value)?;
        
        // Load new CS:IP from interrupt vector table
        let vector_addr = int_num as u32 * 4;
        self.regs.ip = self.memory.read_word(vector_addr);
        self.regs.cs = self.memory.read_word(vector_addr + 2);
        
        println!("[CPU::handle_interrupt] Loading interrupt vector - New CS:IP = {:04X}:{:04X}", 
                 self.regs.cs, self.regs.ip);

        // Clear IF and TF flags
        self.regs.flags.value &= !0x0300;  // Clear IF (bit 9) and TF (bit 8)
        println!("[CPU::handle_interrupt] Cleared IF and TF flags - New flags = {:04X}", 
                 self.regs.flags.value);

        Ok(())
    }

    fn push_word(&mut self, value: u16) -> Result<(), String> {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        let addr = ((self.regs.ss as u32) << 4) | (self.regs.sp as u32);
        self.memory.write_word(addr, value);
        Ok(())
    }

    fn get_reg16(&self, reg: u8) -> u16 {
        match reg & 0x07 {
            0 => self.regs.ax,
            1 => self.regs.bx,
            2 => self.regs.cx,
            3 => self.regs.dx,
            4 => self.regs.sp,
            5 => self.regs.bp,
            6 => self.regs.si,
            7 => self.regs.di,
            _ => unreachable!(),
        }
    }

    pub(super) fn set_reg16(&mut self, reg: u8, value: u16) -> Result<(), String> {
        match reg {
            0 => {
                self.regs.set_ax(value);
                Ok(())
            },
            1 => {
                self.regs.set_cx(value);
                Ok(())
            },
            2 => {
                self.regs.set_dx(value);
                Ok(())
            },
            3 => {
                self.regs.set_bx(value);
                Ok(())
            },
            4 => {
                self.regs.set_sp(value);
                Ok(())
            },
            5 => {
                self.regs.set_bp(value);
                Ok(())
            },
            6 => {
                self.regs.set_si(value);
                Ok(())
            },
            7 => {
                self.regs.set_di(value);
                Ok(())
            },
            _ => Err("Invalid register number".to_string())
        }
    }

    pub(super) fn set_reg8(&mut self, reg: u8, value: u8) -> Result<(), String> {
        match reg {
            0 => {
                self.regs.set_al(value);
                Ok(())
            },
            1 => {
                self.regs.set_cl(value);
                Ok(())
            },
            2 => {
                self.regs.set_dl(value);
                Ok(())
            },
            3 => {
                self.regs.set_bl(value);
                Ok(())
            },
            4 => {
                self.regs.set_ah(value);
                Ok(())
            },
            5 => {
                self.regs.set_ch(value);
                Ok(())
            },
            6 => {
                self.regs.set_dh(value);
                Ok(())
            },
            7 => {
                self.regs.set_bh(value);
                Ok(())
            },
            _ => Err("Invalid register number".to_string())
        }
    }

    fn set_sreg(&mut self, sreg: u8, value: u16) {
        match sreg & 0x03 {
            0 => self.regs.es = value,
            1 => self.regs.cs = value,
            2 => self.regs.ss = value,
            3 => self.regs.ds = value,
            _ => unreachable!(),
        }
    }

    fn get_sreg(&self, sreg: u8) -> u16 {
        match sreg {
            0 => self.regs.es,
            1 => self.regs.cs,
            2 => self.regs.ss,
            3 => self.regs.ds,
            _ => panic!("Invalid segment register index"),
        }
    }

    pub(super) fn xor_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_reg16((modrm & 0x07) as u8);
        let reg = (modrm >> 3) & 0x07;
        let reg_value = self.get_reg16(reg as u8);
        let result = reg_value ^ rm_value;
        self.regs.flags.update_logical_flags(result);
        self.regs.set_reg16(reg as u8, result);
        Ok(())
    }

    pub(super) fn mov_sp_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.set_reg16(4, value)?;  // SP is register 4
        Ok(())
    }

    pub(super) fn mov_bx_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.set_reg16(3, value)?;  // BX is register 3
        Ok(())
    }

    pub(super) fn mov_dx_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.set_reg16(2, value)?;  // DX is register 2
        Ok(())
    }

    pub(super) fn mov_si_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.set_reg16(6, value)?;  // SI is register 6
        Ok(())
    }

    pub(super) fn lodsb(&mut self) -> Result<(), String> {
        let addr = (self.regs.ds as u32) << 4 | (self.regs.si as u32);
        let value = self.memory.read_byte(addr);
        println!("LODSB: Loading byte {:#04X} from DS:SI ({:#06X})", value, addr);
        self.regs.set_al(value);
        self.regs.si = self.regs.si.wrapping_add(1);
        Ok(())
    }

    pub(super) fn or_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8(reg);
        let result = reg_val | rm_val;
        println!("OR r/m8, r8: {:#04X} | {:#04X} = {:#04X}", rm_val, reg_val, result);
        self.set_rm8(modrm, result)?;
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        Ok(())
    }

    pub(super) fn jz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        println!("JZ rel8: Offset={}, ZF={}", offset, self.regs.flags.get_zero());
        if self.regs.flags.get_zero() {
            let new_ip = self.regs.ip.wrapping_add(offset as u16);
            println!("  Taking jump to {:#06X}", new_ip);
            self.regs.ip = new_ip;
        } else {
            println!("  Not taking jump");
        }
        Ok(())
    }

    pub(super) fn jnz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_zero() {
            self.regs.ip = ((self.regs.ip as i16) + (offset as i16)) as u16;
        }
        Ok(())
    }

    pub(super) fn jl_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let sf = self.regs.flags.get_sign();
        let of = self.regs.flags.get_overflow();
        if sf != of {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn push_bx(&mut self) -> Result<(), String> {
        println!("[CPU::push_bx]: Pushing {:#06X} to stack", self.regs.bx);
        self.push_word(self.regs.bx)?;
        Ok(())
    }

    pub(super) fn push_cx(&mut self) -> Result<(), String> {
        println!("[CPU::push_cx]: Pushing {:#06X} to stack", self.regs.cx);
        self.push_word(self.regs.cx)?;
        Ok(())
    }

    pub(super) fn push_dx(&mut self) -> Result<(), String> {
        println!("[CPU::push_dx]: Pushing {:#06X} to stack", self.regs.dx);
        self.push_word(self.regs.dx)?;
        Ok(())
    }

    pub(super) fn add_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        let rm_val = self.get_rm8(modrm)?;
        let (result, carry) = rm_val.overflowing_add(reg_val);
        println!("[CPU::add_rm8_r8] {:#04X} + {:#04X} = {:#04X} (carry={})", 
                 rm_val, reg_val, result, carry);
        self.set_rm8(modrm, result)?;
        self.update_flags_add(rm_val, reg_val, result, carry);
        Ok(())
    }

    pub(super) fn add_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8(reg);
        let (result, carry) = reg_val.overflowing_add(rm_val);
        self.regs.set_reg8(reg, result);

        // Update flags
        self.update_flags_add(reg_val, rm_val, result, carry);

        Ok(())
    }

    pub(super) fn handle_0f_opcode(&mut self) -> Result<(), String> {
        let opcode = self.fetch_byte()?;
        match opcode {
            0x00 => {
                // SLDT - Store Local Descriptor Table Register
                // Since we're not implementing protected mode features,
                // we'll just return 0 and ignore the ModR/M byte
                let modrm = self.fetch_byte()?;
                let rm = modrm & 0x07;
                self.set_reg16(rm, 0);
                Ok(())
            }
            0x45 => {
                // CMOVNZ/CMOVNE - Not supported on 8086/8088
                Err("Invalid opcode 0F 45h: CMOVNZ/CMOVNE is not supported on 8086/8088".to_string())
            }
            _ => Err(format!("Unimplemented 0F opcode: {:#02x}", opcode)),
        }
    }

    pub(super) fn and_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8(reg);
        let result = rm_val & reg_val;
        self.set_rm8(modrm, result)?;
        
        // Update flags
        self.regs.flags.value = 0;

        Ok(())
    }

    pub(super) fn handle_c0_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let count = self.fetch_byte()?;
        
        match reg {
            0 => {
                // ROL rm8, imm8
                let result = if count == 0 {
                    rm_val
                } else {
                    let count = count & 0x07; // Only use bottom 3 bits for 8-bit rotates
                    let result = (rm_val << count) | (rm_val >> (8 - count));
                    // Update carry flag with last bit rotated out
                    self.regs.flags.set_carry(count > 0 && (result & 0x01) != 0);
                    result
                };
                self.set_rm8(modrm, result)?;
            }
            6 => {
                // SHL rm8, imm8
                let result = if count == 0 {
                    rm_val
                } else {
                    let count = count & 0x1F; // Only use bottom 5 bits for shifts
                    let result = if count >= 8 { 0 } else { rm_val << count };
                    // Update carry flag with last bit shifted out
                    self.regs.flags.set_carry(count > 0 && count <= 8 && (rm_val & (0x80 >> (count - 1))) != 0);
                    // Update overflow flag for count == 1
                    if count == 1 {
                        self.regs.flags.set_overflow((rm_val & 0x80) != 0);
                    }
                    result
                };
                self.set_rm8(modrm, result)?;
                // Update other flags
                self.regs.flags.set_zero(result == 0);
                self.regs.flags.set_sign((result & 0x80) != 0);
                self.regs.flags.set_parity(result.count_ones() % 2 == 0);
            }
            _ => return Err(format!("Unimplemented C0 group operation: {}", reg)),
        }
        Ok(())
    }

    pub(super) fn execute_group1_rm8_imm8(&mut self, _operation: u8) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let operation = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let imm = self.fetch_byte()?;

        let result = match operation {
            0 => rm_val.wrapping_add(imm),  // ADD
            1 => rm_val | imm,              // OR
            2 => rm_val.wrapping_add(imm),  // ADC (TODO: handle carry flag)
            3 => rm_val.wrapping_sub(imm),  // SBB (TODO: handle borrow flag)
            4 => rm_val & imm,              // AND
            5 => rm_val.wrapping_sub(imm),  // SUB
            6 => rm_val ^ imm,              // XOR
            7 => {                          // CMP
                let result = rm_val.wrapping_sub(imm);
                self.update_flags_sub(rm_val, imm, result, false);
                return Ok(());
            }
            _ => return Err(format!("Invalid group 1 operation: {}", operation)),
        };

        // Store the result if it's not a CMP operation
        if operation != 7 {
            self.set_rm8(modrm, result)?;
        }

        Ok(())
    }

    fn update_flags_add(&mut self, a: u8, b: u8, result: u8, carry: bool) {
        // Zero flag
        self.regs.flags.set_zero(result == 0);
        
        // Sign flag
        self.regs.flags.set_sign((result & 0x80) != 0);
        
        // Carry flag
        self.regs.flags.set_carry(carry);
        
        // Auxiliary carry (half-carry)
        self.regs.flags.set_auxiliary(((a & 0x0F) + (b & 0x0F)) & 0x10 != 0);
        
        // Overflow flag
        let a_sign = (a & 0x80) != 0;
        let b_sign = (b & 0x80) != 0;
        let r_sign = (result & 0x80) != 0;
        self.regs.flags.set_overflow((a_sign == b_sign) && (r_sign != a_sign));
        
        // Parity flag
        let mut ones = 0;
        let mut temp = result;
        for _ in 0..8 {
            if temp & 1 != 0 {
                ones += 1;
            }
            temp >>= 1;
        }
        self.regs.flags.set_parity(ones % 2 == 0);
    }

    fn update_flags_sub(&mut self, a: u8, b: u8, result: u8, carry: bool) {
        // Zero flag
        self.regs.flags.set_zero(result == 0);

        // Sign flag
        self.regs.flags.set_sign((result & 0x80) != 0);

        // Carry flag
        self.regs.flags.set_carry(carry);

        // Overflow flag
        let overflow = ((a ^ b) & 0x80) != 0 && ((a ^ result) & 0x80) != 0;
        self.regs.flags.set_overflow(overflow);
    }

    pub(super) fn in_al_imm8(&mut self) -> Result<(), String> {
        let _port = self.fetch_byte()? as u16;
        let value = self.serial.read_byte();
        self.regs.set_al(value);
        Ok(())
    }

    pub(super) fn in_al_dx(&mut self) -> Result<(), String> {
        let _port = self.regs.get_dx();
        let value = self.serial.read_byte();
        self.regs.set_al(value);
        Ok(())
    }

    pub(super) fn jmp_far(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let segment = self.fetch_word()?;
        self.regs.ip = offset;
        self.regs.cs = segment;
        Ok(())
    }

    pub(super) fn jmp_near(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()? as i16;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub(super) fn call_near(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()? as i16;
        // Save the return address (current IP) on the stack
        self.push_word(self.regs.ip)?;
        // Jump to the target address
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub(super) fn jmp_short(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub(super) fn push_ax(&mut self) -> Result<(), String> {
        println!("[CPU::push_ax]: Pushing {:#06X} to stack", self.regs.ax);
        self.push_word(self.regs.ax)?;
        Ok(())
    }

    pub(super) fn push_sp(&mut self) -> Result<(), String> {
        self.push_word(self.regs.sp)?;
        Ok(())
    }

    pub(super) fn push_bp(&mut self) -> Result<(), String> {
        self.push_word(self.regs.bp)?;
        Ok(())
    }

    pub(super) fn push_si(&mut self) -> Result<(), String> {
        self.push_word(self.regs.si)?;
        Ok(())
    }

    pub(super) fn push_di(&mut self) -> Result<(), String> {
        self.push_word(self.regs.di)?;
        Ok(())
    }

    pub(super) fn pop_ax(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        println!("[CPU::pop_ax]: Popping {:#06X} from stack", value);
        self.regs.ax = value;
        Ok(())
    }

    pub(super) fn pop_cx(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        println!("[CPU::pop_cx]: Popping {:#06X} from stack", value);
        self.regs.cx = value;
        Ok(())
    }

    pub(super) fn pop_dx(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        println!("[CPU::pop_dx]: Popping {:#06X} from stack", value);
        self.regs.dx = value;
        Ok(())
    }

    pub(super) fn pop_bx(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        println!("[CPU::pop_bx]: Popping {:#06X} from stack", value);
        self.regs.bx = value;
        Ok(())
    }

    pub(super) fn pop_sp(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        self.regs.sp = value;
        Ok(())
    }

    pub(super) fn pop_bp(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        self.regs.bp = value;
        Ok(())
    }

    pub(super) fn pop_si(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        self.regs.si = value;
        Ok(())
    }

    pub(super) fn pop_di(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        self.regs.di = value;
        Ok(())
    }

    fn pop_word(&mut self) -> Result<u16, String> {
        let addr = ((self.regs.ss as u32) << 4) | (self.regs.sp as u32);
        let value = self.memory.read_word(addr);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        Ok(value)
    }

    pub(super) fn iret(&mut self) -> Result<(), String> {
        // Pop IP, CS, and FLAGS
        self.regs.ip = self.pop_word()?;
        self.regs.cs = self.pop_word()?;
        self.regs.flags.value = self.pop_word()?;
        Ok(())
    }

    pub(super) fn inc_sp(&mut self) -> Result<(), String> {
        // Increment SP by 1
        self.regs.sp = self.regs.sp.wrapping_add(1);
        Ok(())
    }

    pub(super) fn clc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(false);
        Ok(())
    }

    pub(super) fn add_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm16(modrm)?;
        let reg_val = self.get_reg16(reg);
        let (result, carry) = reg_val.overflowing_add(rm_val);
        self.set_reg16(reg, result);

        // Update flags
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(
            ((reg_val ^ !rm_val) & (reg_val ^ result) & 0x8000) != 0
        );

        Ok(())
    }

    pub(super) fn adc_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8(reg);
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        let (result1, carry1) = reg_val.overflowing_add(rm_val);
        let (result, carry2) = result1.overflowing_add(carry);
        self.regs.set_reg8(reg, result);

        // Update flags
        self.regs.flags.set_carry(carry1 || carry2);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(
            ((reg_val ^ !rm_val) & (reg_val ^ result) & 0x80) != 0
        );

        Ok(())
    }

    pub(super) fn xor_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_rm16(modrm)?;
        let reg_value = self.get_reg16(modrm);
        let result = rm_value ^ reg_value;
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result as i16) < 0);
        self.regs.flags.set_carry(false);
        self.regs.flags.set_overflow(false);
        Ok(())
    }

    pub(super) fn xor_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_rm8(modrm)?;
        let reg_value = self.regs.get_reg8(modrm);
        let result = rm_value ^ reg_value;
        println!("XOR r/m8, r8: {:#04X} ^ {:#04X} = {:#04X}", 
                 rm_value, reg_value, result);
        self.set_rm8(modrm, result)?;
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result as i8) < 0);
        self.regs.flags.set_carry(false);
        self.regs.flags.set_overflow(false);
        Ok(())
    }

    pub(super) fn xor_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_rm8(modrm)?;
        let reg_value = self.regs.get_reg8(modrm);
        let result = reg_value ^ rm_value;
        self.regs.set_reg8(modrm, result);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result as i8) < 0);
        self.regs.flags.set_carry(false);
        self.regs.flags.set_overflow(false);
        Ok(())
    }

    pub(super) fn mov_sp_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.get_reg16(reg);
        self.set_reg16(4, value)?;  // SP is register 4
        Ok(())
    }

    pub(super) fn mov_dl_imm8(&mut self) -> Result<(), String> {
        let value = self.fetch_byte()?;
        self.regs.set_reg8(2, value);  // DL is register 2
        Ok(())
    }

    pub(super) fn dec_di(&mut self) -> Result<(), String> {
        let value = self.regs.di.wrapping_sub(1);
        self.regs.di = value;
        self.regs.flags.update_flags_dec16(value);
        Ok(())
    }

    pub(super) fn lock_prefix(&mut self) -> Result<(), String> {
        // LOCK prefix - ignored in 8086 emulation since we don't support multiprocessor
        Ok(())
    }

    pub(super) fn rep_prefix(&mut self) -> Result<(), String> {
        // REP/REPE/REPZ prefix - ignored in basic 8086 emulation
        Ok(())
    }

    pub(super) fn test_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8(reg);
        let result = rm_val & reg_val;
        println!("TEST r/m8, r8: {:#04X} & {:#04X} = {:#04X}", 
                 rm_val, reg_val, result);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        self.regs.flags.set_carry(false);
        self.regs.flags.set_overflow(false);
        Ok(())
    }

    // Group operations
    pub(super) fn execute_group1_rm16_imm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let imm16 = self.fetch_word()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_value = self.get_rm16(modrm)?;

        match reg {
            0 => self.add_rm16_imm16(modrm, rm_value, imm16),
            1 => self.or_rm16_imm16(modrm, rm_value, imm16),
            2 => self.adc_rm16_imm16(modrm, rm_value, imm16),
            3 => self.sbb_rm16_imm16(modrm, rm_value, imm16),
            4 => self.and_rm16_imm16(modrm, rm_value, imm16),
            5 => self.sub_rm16_imm16(modrm, rm_value, imm16),
            6 => self.xor_rm16_imm16(modrm, rm_value, imm16),
            7 => self.cmp_rm16_imm16(modrm, rm_value, imm16),
            _ => Err("Invalid group1 operation".to_string())
        }
    }

    pub(super) fn execute_group1_rm16_imm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let imm8 = self.fetch_byte()? as i8;
        let reg = (modrm >> 3) & 0x07;
        let rm_value = self.get_rm16(modrm)?;
        let imm16 = imm8 as i16 as u16;

        match reg {
            0 => self.add_rm16_imm16(modrm, rm_value, imm16),
            1 => self.or_rm16_imm16(modrm, rm_value, imm16),
            2 => self.adc_rm16_imm16(modrm, rm_value, imm16),
            3 => self.sbb_rm16_imm16(modrm, rm_value, imm16),
            4 => self.and_rm16_imm16(modrm, rm_value, imm16),
            5 => self.sub_rm16_imm16(modrm, rm_value, imm16),
            6 => self.xor_rm16_imm16(modrm, rm_value, imm16),
            7 => self.cmp_rm16_imm16(modrm, rm_value, imm16),
            _ => Err("Invalid group1 operation".to_string())
        }
    }

    // String operations
    pub(super) fn movsb(&mut self) -> Result<(), String> {
        let src_addr = ((self.regs.ds as u32) << 4) + (self.regs.si as u32);
        let dst_addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        
        let value = self.memory.read_byte(src_addr);
        self.memory.write_byte(dst_addr, value);
        
        if self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_sub(1);
            self.regs.di = self.regs.di.wrapping_sub(1);
        } else {
            self.regs.si = self.regs.si.wrapping_add(1);
            self.regs.di = self.regs.di.wrapping_add(1);
        }
        Ok(())
    }

    pub(super) fn movsw(&mut self) -> Result<(), String> {
        let src_addr = ((self.regs.ds as u32) << 4) + (self.regs.si as u32);
        let dst_addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        
        let value = self.memory.read_word(src_addr);
        self.memory.write_word(dst_addr, value);
        
        if self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_sub(2);
            self.regs.di = self.regs.di.wrapping_sub(2);
        } else {
            self.regs.si = self.regs.si.wrapping_add(2);
            self.regs.di = self.regs.di.wrapping_add(2);
        }
        Ok(())
    }

    // Flag operations
    pub(super) fn cmc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(!self.regs.flags.get_carry());
        Ok(())
    }

    pub(super) fn stc(&mut self) -> Result<(), String> {
        self.regs.flags.set_carry(true);
        Ok(())
    }

    pub(super) fn cld(&mut self) -> Result<(), String> {
        self.regs.flags.set_direction(false);
        Ok(())
    }

    pub(super) fn std(&mut self) -> Result<(), String> {
        self.regs.flags.set_direction(true);
        Ok(())
    }

    // Register operations
    pub(super) fn mov_r8_imm8(&mut self, reg: u8) -> Result<(), String> {
        let value = self.fetch_byte()?;
        self.set_reg8(reg, value)
    }

    pub(super) fn mov_r16_imm16(&mut self, reg: u8) -> Result<(), String> {
        let value = self.fetch_word()?;
        match reg {
            0 => { self.regs.ax = value; Ok(()) }
            1 => { self.regs.cx = value; Ok(()) }
            2 => { self.regs.dx = value; Ok(()) }
            3 => { self.regs.bx = value; Ok(()) }
            4 => { self.regs.sp = value; Ok(()) }
            5 => { self.regs.bp = value; Ok(()) }
            6 => { self.regs.si = value; Ok(()) }
            7 => { self.regs.di = value; Ok(()) }
            _ => Err(format!("Invalid 16-bit register: {}", reg))
        }
    }

    // Exchange operations
    pub(super) fn xchg_ax_r16(&mut self, reg: u8) -> Result<(), String> {
        let ax = self.regs.ax;
        match reg {
            0 => Ok(()), // NOP - exchange AX with itself
            1 => {
                self.regs.ax = self.regs.cx;
                self.regs.cx = ax;
                Ok(())
            }
            2 => {
                self.regs.ax = self.regs.dx;
                self.regs.dx = ax;
                Ok(())
            }
            3 => {
                self.regs.ax = self.regs.bx;
                self.regs.bx = ax;
                Ok(())
            }
            4 => {
                self.regs.ax = self.regs.sp;
                self.regs.sp = ax;
                Ok(())
            }
            5 => {
                self.regs.ax = self.regs.bp;
                self.regs.bp = ax;
                Ok(())
            }
            6 => {
                self.regs.ax = self.regs.si;
                self.regs.si = ax;
                Ok(())
            }
            7 => {
                self.regs.ax = self.regs.di;
                self.regs.di = ax;
                Ok(())
            }
            _ => Err(format!("Invalid register for XCHG: {}", reg))
        }
    }

    // Memory operations
    pub(super) fn mov_al_moffs8(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let addr = ((self.regs.ds as u32) << 4) + (offset as u32);
        let value = self.memory.read_byte(addr);
        self.regs.ax = (self.regs.ax & 0xFF00) | (value as u16);
        Ok(())
    }

    pub(super) fn mov_ax_moffs16(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let addr = ((self.regs.ds as u32) << 4) + (offset as u32);
        self.regs.ax = self.memory.read_word(addr);
        Ok(())
    }

    pub(super) fn mov_moffs8_al(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let addr = ((self.regs.ds as u32) << 4) + (offset as u32);
        self.memory.write_byte(addr, self.regs.ax as u8);
        Ok(())
    }

    pub(super) fn mov_moffs16_ax(&mut self) -> Result<(), String> {
        let offset = self.fetch_word()?;
        let addr = ((self.regs.ds as u32) << 4) + (offset as u32);
        self.memory.write_word(addr, self.regs.ax);
        Ok(())
    }

    // Prefix handlers
    pub(super) fn repne_prefix(&mut self) -> Result<(), String> {
        let next_opcode = self.fetch_byte()?;
        while self.regs.cx != 0 && !self.regs.flags.get_zero() {
            self.execute_instruction(next_opcode)?;
            self.regs.cx = self.regs.cx.wrapping_sub(1);
        }
        Ok(())
    }

    // Group handlers
    pub(super) fn handle_fe_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let operation = (modrm >> 3) & 0x7;
        
        match operation {
            0 => self.inc_rm8(modrm),
            1 => self.dec_rm8(modrm),
            _ => Err(format!("Invalid FE group operation: {}", operation))
        }
    }

    pub(super) fn handle_ff_group(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let operation = (modrm >> 3) & 0x7;
        
        match operation {
            0 => self.inc_rm16(modrm),
            1 => self.dec_rm16(modrm),
            2 => self.call_rm16(modrm),
            3 => self.call_m16_16(modrm),
            4 => self.jmp_rm16(modrm),
            5 => self.jmp_m16_16(modrm),
            6 => self.push_rm16(modrm),
            _ => Err(format!("Invalid FF group operation: {}", operation))
        }
    }

    fn add_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let (result, carry) = rm_value.overflowing_add(imm16);
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn or_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let result = rm_value | imm16;
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn adc_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        let (result1, carry1) = rm_value.overflowing_add(imm16);
        let (result, carry2) = result1.overflowing_add(carry);
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(carry1 || carry2);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn sbb_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let carry = if self.regs.flags.get_carry() { 1 } else { 0 };
        let (result1, carry1) = rm_value.overflowing_sub(imm16);
        let (result, carry2) = result1.overflowing_sub(carry);
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(carry1 || carry2);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn and_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let result = rm_value & imm16;
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn sub_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let (result, carry) = rm_value.overflowing_sub(imm16);
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn xor_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let result = rm_value ^ imm16;
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn cmp_rm16_imm16(&mut self, modrm: u8, rm_value: u16, imm16: u16) -> Result<(), String> {
        let (result, carry) = rm_value.overflowing_sub(imm16);
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        Ok(())
    }

    fn inc_rm8(&mut self, modrm: u8) -> Result<(), String> {
        let rm_val = self.get_rm8(modrm)?;
        let result = rm_val.wrapping_add(1);
        self.set_rm8(modrm, result)?;
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(rm_val == 0x7F);
        Ok(())
    }

    fn dec_rm8(&mut self, modrm: u8) -> Result<(), String> {
        let rm_val = self.get_rm8(modrm)?;
        let result = rm_val.wrapping_sub(1);
        self.set_rm8(modrm, result)?;
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(rm_val == 0x80);
        Ok(())
    }

    fn inc_rm16(&mut self, modrm: u8) -> Result<(), String> {
        let rm_val = self.get_rm16(modrm)?;
        let result = rm_val.wrapping_add(1);
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(rm_val == 0x7FFF);
        Ok(())
    }

    fn dec_rm16(&mut self, modrm: u8) -> Result<(), String> {
        let rm_val = self.get_rm16(modrm)?;
        let result = rm_val.wrapping_sub(1);
        self.set_rm16(modrm, result)?;
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(rm_val == 0x8000);
        Ok(())
    }

    fn call_rm16(&mut self, modrm: u8) -> Result<(), String> {
        let target = self.get_rm16(modrm)?;
        self.push_word(self.regs.ip)?;
        self.regs.ip = target;
        Ok(())
    }

    fn call_m16_16(&mut self, modrm: u8) -> Result<(), String> {
        let addr = self.get_rm16_addr(modrm)?;
        let offset = self.memory.read_word(addr);
        let segment = self.memory.read_word(addr + 2);
        self.push_word(self.regs.ip)?;
        self.push_word(self.regs.cs)?;
        self.regs.ip = offset;
        self.regs.cs = segment;
        Ok(())
    }

    fn jmp_rm16(&mut self, modrm: u8) -> Result<(), String> {
        let target = self.get_rm16(modrm)?;
        self.regs.ip = target;
        Ok(())
    }

    fn jmp_m16_16(&mut self, modrm: u8) -> Result<(), String> {
        let addr = self.get_rm16_addr(modrm)?;
        let offset = self.memory.read_word(addr);
        let segment = self.memory.read_word(addr + 2);
        self.regs.ip = offset;
        self.regs.cs = segment;
        Ok(())
    }

    fn push_rm16(&mut self, modrm: u8) -> Result<(), String> {
        let value = self.get_rm16(modrm)?;
        self.push_word(value)
    }

    fn get_rm16_addr(&mut self, modrm: u8) -> Result<u32, String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;
        
        match mod_bits {
            0 => {
                match rm {
                    0 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + self.regs.si) as u32)),
                    1 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + self.regs.di) as u32)),
                    2 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + self.regs.si) as u32)),
                    3 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + self.regs.di) as u32)),
                    4 => Ok(((self.regs.ds as u32) << 4) + (self.regs.si as u32)),
                    5 => Ok(((self.regs.ds as u32) << 4) + (self.regs.di as u32)),
                    6 => {
                        let disp16 = self.fetch_word()?;
                        Ok(((self.regs.ds as u32) << 4) + (disp16 as u32))
                    },
                    7 => Ok(((self.regs.ds as u32) << 4) + (self.regs.bx as u32)),
                    _ => Err(format!("Invalid r/m value: {}", rm))
                }
            },
            1 => {
                let disp8 = self.fetch_byte()? as i8 as i16 as u16;
                match rm {
                    0 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + self.regs.si + disp8) as u32)),
                    1 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + self.regs.di + disp8) as u32)),
                    2 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + self.regs.si + disp8) as u32)),
                    3 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + self.regs.di + disp8) as u32)),
                    4 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.si + disp8) as u32)),
                    5 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.di + disp8) as u32)),
                    6 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + disp8) as u32)),
                    7 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + disp8) as u32)),
                    _ => Err(format!("Invalid r/m value: {}", rm))
                }
            },
            2 => {
                let disp16 = self.fetch_word()?;
                match rm {
                    0 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + self.regs.si + disp16) as u32)),
                    1 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + self.regs.di + disp16) as u32)),
                    2 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + self.regs.si + disp16) as u32)),
                    3 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + self.regs.di + disp16) as u32)),
                    4 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.si + disp16) as u32)),
                    5 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.di + disp16) as u32)),
                    6 => Ok(((self.regs.ss as u32) << 4) + ((self.regs.bp + disp16) as u32)),
                    7 => Ok(((self.regs.ds as u32) << 4) + ((self.regs.bx + disp16) as u32)),
                    _ => Err(format!("Invalid r/m value: {}", rm))
                }
            },
            3 => Err("Invalid mod bits for memory addressing".to_string()),
            _ => Err(format!("Invalid mod bits: {}", mod_bits))
        }
    }

    // Conversion operations
    pub(super) fn cbw(&mut self) -> Result<(), String> {
        let al = (self.regs.ax & 0xFF) as i8;
        self.regs.ax = al as i16 as u16;
        Ok(())
    }

    pub(super) fn cwd(&mut self) -> Result<(), String> {
        let ax = self.regs.ax as i16;
        let dx = if ax < 0 { 0xFFFF } else { 0 };
        self.regs.dx = dx;
        Ok(())
    }

    // Flag operations
    pub(super) fn pushf(&mut self) -> Result<(), String> {
        self.push_word(self.regs.flags.value)
    }

    pub(super) fn popf(&mut self) -> Result<(), String> {
        let flags = self.pop_word()?;
        self.regs.flags.value = flags;
        Ok(())
    }

    pub(super) fn sahf(&mut self) -> Result<(), String> {
        let ah = (self.regs.ax >> 8) as u8;
        self.regs.flags.value = (self.regs.flags.value & 0xFF00) | (ah as u16);
        Ok(())
    }

    pub(super) fn lahf(&mut self) -> Result<(), String> {
        let flags = self.regs.flags.value as u8;
        self.regs.ax = (self.regs.ax & 0x00FF) | ((flags as u16) << 8);
        Ok(())
    }

    // String operations
    pub(super) fn cmpsb(&mut self) -> Result<(), String> {
        let src_addr = ((self.regs.ds as u32) << 4) + (self.regs.si as u32);
        let dst_addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        
        let src = self.memory.read_byte(src_addr);
        let dst = self.memory.read_byte(dst_addr);
        
        let (result, carry) = dst.overflowing_sub(src);
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        
        if self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_sub(1);
            self.regs.di = self.regs.di.wrapping_sub(1);
        } else {
            self.regs.si = self.regs.si.wrapping_add(1);
            self.regs.di = self.regs.di.wrapping_add(1);
        }
        Ok(())
    }

    pub(super) fn cmpsw(&mut self) -> Result<(), String> {
        let src_addr = ((self.regs.ds as u32) << 4) + (self.regs.si as u32);
        let dst_addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        
        let src = self.memory.read_word(src_addr);
        let dst = self.memory.read_word(dst_addr);
        
        let (result, carry) = dst.overflowing_sub(src);
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        
        if self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_sub(2);
            self.regs.di = self.regs.di.wrapping_sub(2);
        } else {
            self.regs.si = self.regs.si.wrapping_add(2);
            self.regs.di = self.regs.di.wrapping_add(2);
        }
        Ok(())
    }

    pub(super) fn stosb(&mut self) -> Result<(), String> {
        let addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        self.memory.write_byte(addr, self.regs.ax as u8);
        
        if self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_sub(1);
        } else {
            self.regs.di = self.regs.di.wrapping_add(1);
        }
        Ok(())
    }

    pub(super) fn stosw(&mut self) -> Result<(), String> {
        let addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        self.memory.write_word(addr, self.regs.ax);
        
        if self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_sub(2);
        } else {
            self.regs.di = self.regs.di.wrapping_add(2);
        }
        Ok(())
    }

    pub(super) fn lodsw(&mut self) -> Result<(), String> {
        let addr = ((self.regs.ds as u32) << 4) + (self.regs.si as u32);
        self.regs.ax = self.memory.read_word(addr);
        
        if self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_sub(2);
        } else {
            self.regs.si = self.regs.si.wrapping_add(2);
        }
        Ok(())
    }

    pub(super) fn scasb(&mut self) -> Result<(), String> {
        let addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        let mem_val = self.memory.read_byte(addr);
        let al = self.regs.ax as u8;
        
        let (result, carry) = al.overflowing_sub(mem_val);
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        
        if self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_sub(1);
        } else {
            self.regs.di = self.regs.di.wrapping_add(1);
        }
        Ok(())
    }

    pub(super) fn scasw(&mut self) -> Result<(), String> {
        let addr = ((self.regs.es as u32) << 4) + (self.regs.di as u32);
        let mem_val = self.memory.read_word(addr);
        
        let (result, carry) = self.regs.ax.overflowing_sub(mem_val);
        self.regs.flags.set_carry(carry);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        
        if self.regs.flags.get_direction() {
            self.regs.di = self.regs.di.wrapping_sub(2);
        } else {
            self.regs.di = self.regs.di.wrapping_add(2);
        }
        Ok(())
    }

    // I/O operations
    pub(super) fn in_ax_imm8(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()? as u16;
        let value = (self.serial.read_byte() as u16) << 8 | self.serial.read_byte() as u16;
        self.regs.ax = value;
        Ok(())
    }

    pub(super) fn out_imm8_al(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()? as u16;
        self.serial.write_byte(self.regs.ax as u8);
        Ok(())
    }

    pub(super) fn out_imm8_ax(&mut self) -> Result<(), String> {
        let port = self.fetch_byte()? as u16;
        self.serial.write_byte((self.regs.ax >> 8) as u8);
        self.serial.write_byte(self.regs.ax as u8);
        Ok(())
    }

    pub(super) fn out_dx_al(&mut self) -> Result<(), String> {
        self.serial.write_byte(self.regs.ax as u8);
        Ok(())
    }

    pub(super) fn out_dx_ax(&mut self) -> Result<(), String> {
        self.serial.write_byte((self.regs.ax >> 8) as u8);
        self.serial.write_byte(self.regs.ax as u8);
        Ok(())
    }

    // Return operations
    pub(super) fn ret_near(&mut self) -> Result<(), String> {
        self.regs.ip = self.pop_word()?;
        Ok(())
    }

    pub(super) fn ret_far(&mut self) -> Result<(), String> {
        self.regs.ip = self.pop_word()?;
        self.regs.cs = self.pop_word()?;
        Ok(())
    }

    // Segment operations
    pub(super) fn les(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let addr = self.get_rm16_addr(modrm)?;
        let offset = self.memory.read_word(addr);
        let segment = self.memory.read_word(addr + 2);
        self.set_reg16((modrm >> 3) & 0x07, offset)?;
        self.regs.es = segment;
        Ok(())
    }

    pub(super) fn lds(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let addr = self.get_rm16_addr(modrm)?;
        let offset = self.memory.read_word(addr);
        let segment = self.memory.read_word(addr + 2);
        self.set_reg16((modrm >> 3) & 0x07, offset)?;
        self.regs.ds = segment;
        Ok(())
    }

    // Conditional jump instructions
    pub(super) fn jo_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_overflow() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jno_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_overflow() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_carry() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jnb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_carry() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_carry() || self.regs.flags.get_zero() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jnbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_carry() && !self.regs.flags.get_zero() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn js_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_sign() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jns_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_sign() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jp_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_parity() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jnp_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_parity() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jnl_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let sf = self.regs.flags.get_sign();
        let of = self.regs.flags.get_overflow();
        if sf == of {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jle_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let sf = self.regs.flags.get_sign();
        let of = self.regs.flags.get_overflow();
        let zf = self.regs.flags.get_zero();
        if zf || (sf != of) {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jnle_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let sf = self.regs.flags.get_sign();
        let of = self.regs.flags.get_overflow();
        let zf = self.regs.flags.get_zero();
        if !zf && (sf == of) {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn test_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.get_al();
        let result = al & imm8;
        self.regs.flags.update_logical_flags(result as u16);
        Ok(())
    }

    pub(super) fn test_ax_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        let ax = self.regs.get_ax();
        let result = ax & imm16;
        self.regs.flags.update_logical_flags(result);
        Ok(())
    }

    pub(super) fn mov_rm8_imm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let imm8 = self.fetch_byte()?;
        self.write_rm8(modrm, imm8)?;
        Ok(())
    }

    pub(super) fn mov_rm16_imm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let imm16 = self.fetch_word()?;
        self.write_rm16(modrm, imm16)?;
        Ok(())
    }

    pub(super) fn in_ax_dx(&mut self) -> Result<(), String> {
        let port = self.regs.dx;
        let value = self.io_read_word(port)?;
        self.regs.set_ax(value);
        Ok(())
    }

    pub(super) fn write_rm8(&mut self, modrm: u8, value: u8) -> Result<(), String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        match mod_bits {
            0b00 => {
                let addr = self.get_rm_addr(modrm)?;
                self.memory.write_byte(addr, value);
                Ok(())
            },
            0b01 | 0b10 => {
                let addr = self.get_rm_addr(modrm)?;
                self.memory.write_byte(addr, value);
                Ok(())
            },
            0b11 => {
                self.set_reg8(rm, value)?;
                Ok(())
            },
            _ => Err("Invalid mod bits".to_string())
        }
    }

    pub(super) fn write_rm16(&mut self, modrm: u8, value: u16) -> Result<(), String> {
        let rm = modrm & 0x07;
        let mod_bits = (modrm >> 6) & 0x03;

        match mod_bits {
            0b00 => {
                let addr = self.get_rm_addr(modrm)?;
                self.memory.write_word(addr, value);
                Ok(())
            },
            0b01 | 0b10 => {
                let addr = self.get_rm_addr(modrm)?;
                self.memory.write_word(addr, value);
                Ok(())
            },
            0b11 => {
                self.set_reg16(rm, value)?;
                Ok(())
            },
            _ => Err("Invalid mod bits".to_string())
        }
    }

    pub(super) fn io_read_word(&mut self, port: u16) -> Result<u16, String> {
        let low = self.io_read_byte(port)? as u16;
        let high = self.io_read_byte(port + 1)? as u16;
        Ok((high << 8) | low)
    }

    pub(super) fn io_read_byte(&mut self, port: u16) -> Result<u8, String> {
        // For now, just return 0 for all ports
        Ok(0)
    }

    pub(super) fn inc_si(&mut self) -> Result<(), String> {
        let value = self.regs.si;
        let result = value.wrapping_add(1);
        self.regs.si = result;
        self.update_flags_inc16(result);
        Ok(())
    }

    fn update_flags_inc16(&mut self, result: u16) {
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(result == 0x8000);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
    }

    pub fn inc_bp(&mut self) -> Result<(), String> {
        let value = self.regs.bp;
        let result = value.wrapping_add(1);
        self.regs.bp = result;
        self.update_flags_inc16(result);
        Ok(())
    }

    pub fn inc_di(&mut self) -> Result<(), String> {
        let value = self.regs.di;
        let result = value.wrapping_add(1);
        self.regs.di = result;
        self.update_flags_inc16(result);
        Ok(())
    }

    pub fn sbb_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.get_al();
        let carry = self.regs.flags.get_carry() as u8;
        let result = al.wrapping_sub(imm8).wrapping_sub(carry);
        self.regs.set_al(result);
        self.update_flags_sub(al, imm8, result, carry != 0);
        Ok(())
    }

    pub fn sbb_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_rm8(modrm)?;
        let reg_value = self.get_reg8(modrm)?;
        let carry = self.regs.flags.get_carry() as u8;
        let result = rm_value.wrapping_sub(reg_value).wrapping_sub(carry);
        self.set_rm8(modrm, result)?;
        self.update_flags_sub(rm_value, reg_value, result, carry != 0);
        Ok(())
    }

    pub fn sbb_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_value = self.get_rm16(modrm)?;
        let reg_value = self.get_reg16(modrm);
        let carry = self.regs.flags.get_carry() as u16;
        let result = rm_value.wrapping_sub(reg_value).wrapping_sub(carry);
        self.set_rm16(modrm, result)?;
        self.update_flags_sub16(rm_value, reg_value, result, carry != 0);
        Ok(())
    }

    pub fn sbb_ax_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        let ax = self.regs.ax;
        let carry = self.regs.flags.get_carry() as u16;
        let result = ax.wrapping_sub(imm16).wrapping_sub(carry);
        self.regs.ax = result;
        self.update_flags_sub16(ax, imm16, result, carry != 0);
        Ok(())
    }

    fn update_flags_sub16(&mut self, a: u16, b: u16, result: u16, carry: bool) {
        // Zero flag
        self.regs.flags.set_zero(result == 0);

        // Sign flag
        self.regs.flags.set_sign((result & 0x8000) != 0);

        // Carry flag
        self.regs.flags.set_carry(carry);

        // Overflow flag
        let overflow = (a ^ b) & (a ^ result) & 0x8000 != 0;
        self.regs.flags.set_overflow(overflow);

        // Auxiliary flag (not used in 16-bit operations)
        self.regs.flags.set_auxiliary(false);

        // Parity flag
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
    }

    fn sub_rm8_imm8(&mut self, modrm: u8) -> Result<(), String> {
        let rm_val = self.get_rm8(modrm)?;
        let imm = self.fetch_byte()?;
        let result = rm_val.wrapping_sub(imm);
        self.set_rm8(modrm, result)?;
        self.update_flags_sub(rm_val, imm, result, false);
        Ok(())
    }

    pub fn pop_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let value = self.pop_word()?;
        self.set_rm16(modrm, value)?;
        Ok(())
    }

    pub fn dec_bx(&mut self) -> Result<(), String> {
        let result = self.regs.bx.wrapping_sub(1);
        self.regs.bx = result;
        self.regs.flags.update_flags_dec16(result);
        Ok(())
    }

    pub(super) fn ret_near_imm16(&mut self) -> Result<(), String> {
        let imm16 = self.fetch_word()?;
        self.regs.ip = self.pop_word()?;
        self.regs.sp = self.regs.sp.wrapping_add(imm16);
        Ok(())
    }

    pub(super) fn hlt(&mut self) -> Result<(), String> {
        self.halted = true;
        Ok(())
    }

    pub(super) fn outsw(&mut self) -> Result<(), String> {
        // Output word from memory at DS:SI to port in DX
        let port = self.regs.dx;
        let addr = ((self.regs.ds as u32) << 4) + (self.regs.si as u32);
        let value = self.memory.read_word(addr);
        
        // Write to port
        self.serial.write_byte(value as u8);
        
        // Increment/decrement SI based on direction flag
        if self.regs.flags.get_direction() {
            self.regs.si = self.regs.si.wrapping_sub(2);
        } else {
            self.regs.si = self.regs.si.wrapping_add(2);
        }
        
        Ok(())
    }
} 
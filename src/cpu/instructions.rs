use super::CPU;

impl CPU {
    pub(super) fn mov_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.regs.get_reg8(reg);
        self.set_rm8(modrm, value)?;
        Ok(())
    }

    pub(super) fn mov_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.get_reg16(reg);
        self.set_reg16(reg, value);
        Ok(())
    }

    pub(super) fn mov_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let value = self.get_rm8(modrm)?;
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
        self.regs.flags.set_interrupt(false);
        Ok(())
    }

    pub(super) fn sti(&mut self) -> Result<(), String> {
        self.regs.flags.set_interrupt(true);
        Ok(())
    }

    fn handle_interrupt(&mut self, int_num: u8) -> Result<(), String> {
        // Save return address and flags
        let ip = self.regs.ip;
        self.push_word(ip)?;
        self.push_word(self.regs.cs)?;
        self.push_word(self.regs.flags.value)?;
        
        // Load new CS:IP from interrupt vector table
        self.regs.ip = self.memory.read_word(int_num as u32 * 4);
        self.regs.cs = self.memory.read_word(int_num as u32 * 4 + 2);

        // Clear IF and TF flags
        self.regs.flags.value &= !0x0300;  // Clear IF (bit 9) and TF (bit 8)

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

    fn set_reg16(&mut self, reg: u8, value: u16) {
        match reg & 0x07 {
            0 => self.regs.ax = value,
            1 => self.regs.bx = value,
            2 => self.regs.cx = value,
            3 => self.regs.dx = value,
            4 => self.regs.sp = value,
            5 => self.regs.bp = value,
            6 => self.regs.si = value,
            7 => self.regs.di = value,
            _ => unreachable!(),
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

    fn xor_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm = modrm & 0x07;
        let value = self.get_reg16(rm);
        let reg_value = self.get_reg16(reg);
        let result = reg_value ^ value;
        self.set_reg16(reg, result);
        self.regs.flags.value = 0;
        Ok(())
    }

    fn mov_sp_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.regs.sp = value;
        Ok(())
    }

    pub(super) fn mov_ah_imm8(&mut self) -> Result<(), String> {
        let value = self.fetch_byte()?;
        self.regs.set_ah(value);
        Ok(())
    }

    pub(super) fn mov_bx_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.regs.bx = value;
        Ok(())
    }

    pub(super) fn mov_dx_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.regs.dx = value;
        Ok(())
    }

    pub(super) fn mov_si_imm16(&mut self) -> Result<(), String> {
        let value = self.fetch_word()?;
        self.regs.si = value;
        Ok(())
    }

    pub(super) fn lodsb(&mut self) -> Result<(), String> {
        let addr = (self.regs.ds as u32) << 4 | (self.regs.si as u32);
        let value = self.memory.read_byte(addr);
        println!("LODSB: Reading byte 0x{:02X} from DS:SI = {:04X}:{:04X} (Physical = {:05X})", 
                 value, self.regs.ds, self.regs.si, addr);
        println!("LODSB: Before set_al: AX = {:04X}, AL = {:02X}, AH = {:02X}", 
                 self.regs.ax, self.regs.get_al(), self.regs.get_ah());
        self.regs.set_al(value);
        println!("LODSB: After set_al: AX = {:04X}, AL = {:02X}, AH = {:02X}", 
                 self.regs.ax, self.regs.get_al(), self.regs.get_ah());
        self.regs.si = self.regs.si.wrapping_add(1);
        Ok(())
    }

    pub(super) fn or_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8(reg);
        let result = reg_val | rm_val;
        self.set_rm8(modrm, result)?;

        // Update flags
        self.regs.flags.set_carry(false);  // OR always clears carry
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);  // OR always clears overflow

        Ok(())
    }

    pub(super) fn jz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_zero() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(super) fn jnz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_zero() {
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
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

    pub(super) fn int_n(&mut self) -> Result<(), String> {
        let int_num = self.fetch_byte()?;
        self.handle_interrupt(int_num)?;
        Ok(())
    }

    pub(super) fn push_bx(&mut self) -> Result<(), String> {
        let value = self.regs.bx;
        self.push_word(value)?;
        Ok(())
    }

    pub(super) fn push_cx(&mut self) -> Result<(), String> {
        let value = self.regs.cx;
        self.push_word(value)?;
        Ok(())
    }

    pub(super) fn push_dx(&mut self) -> Result<(), String> {
        let value = self.regs.dx;
        self.push_word(value)?;
        Ok(())
    }

    pub(super) fn add_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        let rm_val = self.get_rm8(modrm)?;
        let (result, carry) = rm_val.overflowing_add(reg_val);
        self.set_rm8(modrm, result)?;

        // Update flags
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
                self.update_flags_sub(rm_val, imm, result);
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

    fn update_flags_sub(&mut self, a: u8, b: u8, result: u8) {
        // Zero flag
        self.regs.flags.set_zero(result == 0);

        // Sign flag
        self.regs.flags.set_sign((result & 0x80) != 0);

        // Carry flag
        self.regs.flags.set_carry((a as u16) < (b as u16));

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
        println!("Far JMP to {:04X}:{:04X}", segment, offset);
        self.regs.cs = segment;
        self.regs.ip = offset;
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
        let value = self.regs.ax;
        self.push_word(value)?;
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
        self.regs.ax = value;
        Ok(())
    }

    pub(super) fn pop_cx(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        self.regs.cx = value;
        Ok(())
    }

    pub(super) fn pop_dx(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
        self.regs.dx = value;
        Ok(())
    }

    pub(super) fn pop_bx(&mut self) -> Result<(), String> {
        let value = self.pop_word()?;
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
        // Pop FLAGS, CS, and IP from the stack in reverse order of pushing
        let flags = self.pop_word()?;
        let cs = self.pop_word()?;
        let ip = self.pop_word()?;
        
        // Update registers and flags
        self.regs.flags.value = flags;
        self.regs.cs = cs;
        self.regs.ip = ip;
        
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
        let reg_value = self.regs.get_reg16((modrm >> 3) & 0x07);
        self.regs.sp = reg_value;
        Ok(())
    }

    pub(super) fn mov_dl_imm8(&mut self) -> Result<(), String> {
        let value = self.fetch_byte()?;
        self.regs.set_dl(value);
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
        
        // Set flags
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        self.regs.flags.set_carry(false);
        self.regs.flags.set_overflow(false);
        
        Ok(())
    }
} 
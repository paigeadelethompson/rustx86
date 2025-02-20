use super::CPU;

impl CPU {
    pub fn execute_instruction(&mut self, opcode: u8) -> Result<(), String> {
        match opcode {
            0x00 => self.add_rm8_r8()?,
            0x02 => self.add_r8_rm8()?,
            0x03 => self.add_r16_rm16()?,
            0x08 => self.or_rm8_r8()?,
            0x0F => self.handle_0f_opcode()?,
            0x10 => self.adc_r8_rm8()?,
            0x20 => self.and_rm8_r8()?,
            0x30 => self.xor_rm8_r8()?,
            0x31 => self.xor_rm16_r16()?,
            0x32 => self.xor_r8_rm8()?,
            0x44 => self.inc_sp()?,
            0x4F => self.dec_di()?,
            0x50 => self.push_ax()?,
            0x51 => self.push_cx()?,
            0x52 => self.push_dx()?,
            0x53 => self.push_bx()?,
            0x54 => self.push_sp()?,
            0x55 => self.push_bp()?,
            0x56 => self.push_si()?,
            0x57 => self.push_di()?,
            0x58 => self.pop_ax()?,
            0x59 => self.pop_cx()?,
            0x5A => self.pop_dx()?,
            0x5B => self.pop_bx()?,
            0x5C => self.pop_sp()?,
            0x5D => self.pop_bp()?,
            0x5E => self.pop_si()?,
            0x5F => self.pop_di()?,
            0x6F => self.outsw()?,
            0x74 => self.jz_rel8()?,
            0x75 => self.jnz_rel8()?,
            0x7C => self.jl_rel8()?,
            0x80 => self.execute_group1_rm8_imm8(0)?,
            0x84 => self.test_rm8_r8()?,
            0x88 => self.mov_rm8_r8()?,
            0x89 => self.mov_rm16_r16()?,
            0x8A => self.mov_r8_rm8()?,
            0x8B => self.mov_r16_rm16()?,
            0x8C => self.mov_rm16_sreg()?,
            0x8E => self.mov_sreg_rm16()?,
            0xAC => self.lodsb()?,
            0xB2 => self.mov_dl_imm8()?,
            0xB4 => self.mov_ah_imm8()?,
            0xBA => self.mov_dx_imm16()?,
            0xBB => self.mov_bx_imm16()?,
            0xBC => self.mov_sp_r16()?,
            0xBE => self.mov_si_imm16()?,
            0xC0 => self.handle_c0_group()?,
            0xCD => self.int_n()?,
            0xCF => self.iret()?,
            0xEA => self.jmp_far()?,
            0xE4 => self.in_al_imm8()?,
            0xE8 => self.call_near()?,
            0xE9 => self.jmp_near()?,
            0xEB => self.jmp_short()?,
            0xF0 => self.lock_prefix()?,
            0xF3 => self.rep_prefix()?,
            0xF4 => self.hlt()?,
            0xF8 => self.clc()?,
            0xFA => self.cli()?,
            0xFB => self.sti()?,
            0x90 => {
                // NOP - No operation
                return Ok(());
            }
            _ => return Err(format!("Unimplemented opcode: {:#04x}", opcode)),
        }
        Ok(())
    }

    fn hlt(&mut self) -> Result<(), String> {
        self.halted = true;
        Ok(())
    }

    fn outsw(&mut self) -> Result<(), String> {
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
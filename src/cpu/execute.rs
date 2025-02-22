use super::CPU;

impl CPU {
    pub fn execute_instruction(&mut self, opcode: u8) -> Result<(), String> {
        // println!("\nExecuting opcode {:#04X} at CS:IP={:04X}:{:04X}", 
        //          opcode, self.regs.cs, self.regs.ip);
        match opcode {
            0x00 => Ok(self.add_rm8_r8()?),
            0x02 => Ok(self.add_r8_rm8()?),
            0x03 => Ok(self.add_r16_rm16()?),
            0x08 => Ok(self.or_rm8_r8()?),
            0x0F => Ok(self.handle_0f_opcode()?),
            0x10 => Ok(self.adc_r8_rm8()?),
            0x20 => Ok(self.and_rm8_r8()?),
            0x30 => Ok(self.xor_rm8_r8()?),
            0x31 => Ok(self.xor_rm16_r16()?),
            0x32 => Ok(self.xor_r8_rm8()?),
            0x33 => Ok(self.xor_r16_rm16()?),
            0x44 => Ok(self.inc_sp()?),
            0x45 => Ok(self.inc_bp()?),
            0x46 => Ok(self.inc_si()?),
            0x47 => Ok(self.inc_di()?),
            0x4F => Ok(self.dec_di()?),
            0x50 => Ok(self.push_ax()?),
            0x51 => Ok(self.push_cx()?),
            0x52 => Ok(self.push_dx()?),
            0x53 => Ok(self.push_bx()?),
            0x54 => Ok(self.push_sp()?),
            0x55 => Ok(self.push_bp()?),
            0x56 => Ok(self.push_si()?),
            0x57 => Ok(self.push_di()?),
            0x58 => Ok(self.pop_ax()?),
            0x59 => Ok(self.pop_cx()?),
            0x5A => Ok(self.pop_dx()?),
            0x5B => Ok(self.pop_bx()?),
            0x5C => Ok(self.pop_sp()?),
            0x5D => Ok(self.pop_bp()?),
            0x5E => Ok(self.pop_si()?),
            0x5F => Ok(self.pop_di()?),
            0x6F => Ok(self.outsw()?),
            0x70 => Ok(self.jo_rel8()?),   // Jump if overflow
            0x71 => Ok(self.jno_rel8()?),  // Jump if not overflow
            0x72 => Ok(self.jb_rel8()?),   // Jump if below/carry
            0x73 => Ok(self.jnb_rel8()?),  // Jump if not below/carry
            0x74 => Ok(self.jz_rel8()?),
            0x75 => Ok(self.jnz_rel8()?),
            0x76 => Ok(self.jbe_rel8()?),  // Jump if below or equal
            0x77 => Ok(self.jnbe_rel8()?), // Jump if not below or equal
            0x78 => Ok(self.js_rel8()?),   // Jump if sign
            0x79 => Ok(self.jns_rel8()?),  // Jump if not sign
            0x7A => Ok(self.jp_rel8()?),   // Jump if parity
            0x7B => Ok(self.jnp_rel8()?),  // Jump if not parity
            0x7C => Ok(self.jl_rel8()?),
            0x7D => Ok(self.jnl_rel8()?),  // Jump if not less
            0x7E => Ok(self.jle_rel8()?),  // Jump if less or equal
            0x7F => Ok(self.jnle_rel8()?), // Jump if not less or equal
            0x80 => Ok(self.execute_group1_rm8_imm8(0)?),
            0x81 => Ok(self.execute_group1_rm16_imm16()?),
            0x83 => Ok(self.execute_group1_rm16_imm8()?),
            0x84 => Ok(self.test_rm8_r8()?),
            0x88 => Ok(self.mov_rm8_r8()?),
            0x89 => Ok(self.mov_rm16_r16()?),
            0x8A => Ok(self.mov_r8_rm8()?),
            0x8B => Ok(self.mov_r16_rm16()?),
            0x8C => Ok(self.mov_rm16_sreg()?),
            0x8E => Ok(self.mov_sreg_rm16()?),
            0x8F => Ok(self.pop_rm16()?),
            0x90..=0x97 => Ok(self.xchg_ax_r16(opcode - 0x90)?),
            0x98 => Ok(self.cbw()?),
            0x99 => Ok(self.cwd()?),
            0x9C => Ok(self.pushf()?),
            0x9D => Ok(self.popf()?),
            0x9E => Ok(self.sahf()?),
            0x9F => Ok(self.lahf()?),
            0xA0 => Ok(self.mov_al_moffs8()?),
            0xA1 => Ok(self.mov_ax_moffs16()?),
            0xA2 => Ok(self.mov_moffs8_al()?),
            0xA3 => Ok(self.mov_moffs16_ax()?),
            0xA4 => Ok(self.movsb()?),
            0xA5 => Ok(self.movsw()?),
            0xA6 => Ok(self.cmpsb()?),
            0xA7 => Ok(self.cmpsw()?),
            0xA8 => self.test_al_imm8(),
            0xA9 => self.test_ax_imm16(),
            0xAA => Ok(self.stosb()?),
            0xAB => Ok(self.stosw()?),
            0xAC => Ok(self.lodsb()?),
            0xAD => Ok(self.lodsw()?),
            0xAE => Ok(self.scasb()?),
            0xAF => Ok(self.scasw()?),
            0xB0..=0xB7 => Ok(self.mov_r8_imm8(opcode - 0xB0)?),
            0xB8..=0xBF => Ok(self.mov_r16_imm16(opcode - 0xB8)?),
            0xC0 => Ok(self.handle_c0_group()?),
            0xC2 => Ok(self.ret_near_imm16()?),
            0xC3 => Ok(self.ret_near()?),
            0xC4 => Ok(self.les()?),
            0xC5 => Ok(self.lds()?),
            0xC6 => self.mov_rm8_imm8(),
            0xC7 => self.mov_rm16_imm16(),
            0xCB => Ok(self.ret_far()?),
            0xCD => {
                let int_num = self.fetch_byte()?;
                Ok(self.int_n(int_num)?)
            },
            0xCF => Ok(self.iret()?),
            0xE4 => Ok(self.in_al_imm8()?),
            0xE5 => Ok(self.in_ax_imm8()?),
            0xE6 => Ok(self.out_imm8_al()?),
            0xE7 => Ok(self.out_imm8_ax()?),
            0xE8 => Ok(self.call_near()?),
            0xE9 => Ok(self.jmp_near()?),
            0xEA => {
                // println!("\n!!! EXECUTING JMP FAR !!!");
                Ok(self.jmp_far()?)
            },
            0xEB => Ok(self.jmp_short()?),
            0xEC => Ok(self.in_al_dx()?),
            0xED => Ok(self.in_ax_dx()?),
            0xEE => Ok(self.out_dx_al()?),
            0xEF => Ok(self.out_dx_ax()?),
            0xF0 => Ok(self.lock_prefix()?),
            0xF2 => Ok(self.repne_prefix()?),
            0xF3 => Ok(self.rep_prefix()?),
            0xF4 => {
                println!("[CPU::execute_instruction]: HLT: Halting CPU execution");
                Ok(self.hlt()?)
            },
            0xF5 => Ok(self.cmc()?),
            0xF8 => Ok(self.clc()?),
            0xF9 => Ok(self.stc()?),
            0xFA => Ok(self.cli()?),
            0xFB => Ok(self.sti()?),
            0xFC => Ok(self.cld()?),
            0xFD => Ok(self.std()?),
            0xFE => Ok(self.handle_fe_group()?),
            0xFF => Ok(self.handle_ff_group()?),
            0x1A => Ok(self.sbb_rm8_r8()?),
            0x1B => Ok(self.sbb_rm16_r16()?),
            0x1C => Ok(self.sbb_al_imm8()?),
            0x1D => Ok(self.sbb_ax_imm16()?),
            0x4B => Ok(self.dec_bx()?),
            _ => {
                println!("\n[CPU::execute_instruction]: *** CRITICAL ERROR ***");
                println!("[CPU::execute_instruction]: Unimplemented opcode {:#04X} at CS:IP={:04X}:{:04X}", 
                         opcode, self.regs.cs, self.regs.ip);
                println!("[CPU::execute_instruction]: CPU HALTED - Illegal instruction");
                self.halted = true;
                Err(format!("HALT: Illegal opcode {:#04X}", opcode))
            }
        }
    }
}
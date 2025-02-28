use super::CPU;
use crate::cpu::SegmentRegister;

// All instruction implementations should go in their respective modules under instructions/
// This file should only contain the instruction dispatch logic (execute_instruction)

impl CPU {    
    pub fn execute_instruction(&mut self) -> Result<(), String> {
        // First check ROM validity
        if !self.memory.has_valid_rom() {
            return Err("Cannot execute: BIOS ROM is corrupted or invalid".to_string());
        }

        // Check if we have valid MBR boot code
        if !self.has_valid_mbr {
            return Err("Cannot execute: No valid MBR boot code".to_string());
        }

        // Check if we have valid boot sector
        if !self.has_valid_boot_sector {
            return Err("Cannot execute: No valid boot sector at LBA 63".to_string());
        }

        let cs_ip = ((self.regs.cs as u32) << 4) + (self.regs.ip as u32);
        let opcode = self.fetch_byte()?;

        match opcode {
            // Data Transfer Instructions
            0x06 => Ok(self.push_es()?),
            0x07 => Ok(self.pop_es()?),
            0x88 => Ok(self.mov_rm8_r8()?),
            0x89 => Ok(self.mov_rm16_r16()?),
            0x8A => Ok(self.mov_r8_rm8()?),
            0x8B => Ok(self.mov_r16_rm16()?),
            0x8C => Ok(self.mov_rm16_sreg()?),
            0x8E => Ok(self.mov_sreg_rm16()?),
            0xB0 => Ok(self.mov_al_imm8()?),
            0xB1 => Ok(self.mov_cl_imm8()?),
            0xB4 => Ok(self.mov_ah_imm8()?),
            0xB5 => Ok(self.mov_ch_imm8()?),
            0xB6 => Ok(self.mov_dh_imm8()?),
            0xB2 => Ok(self.mov_dl_imm8()?),
            0xB8 => Ok(self.mov_ax_imm16()?),
            0xB9 => Ok(self.mov_cx_imm16()?),
            0xBB => Ok(self.mov_bx_imm16()?),
            0xBC => Ok(self.mov_sp_imm16()?),
            0xBE => Ok(self.mov_si_imm16()?),
            0xC4 => Ok(self.les_r16_m16()?),
            0x44 => Ok(self.inc_sp()?),
            0x45 => Ok(self.inc_bp()?),
            0x46 => Ok(self.inc_si()?),
            0x41 => Ok(self.inc_cx()?),
            0x4E => Ok(self.dec_si()?),
            0x4F => Ok(self.dec_di()?),

            // Interrupt Instructions
            0xCD => {
                let interrupt_number = self.fetch_byte()?;
                Ok(self.int(interrupt_number)?)
            },

            // Arithmetic Instructions
            0x00 => Ok(self.add_rm8_r8()?),
            0x01 => Ok(self.add_rm16_r16()?),
            0x02 => Ok(self.add_r8_rm8()?),
            0x03 => Ok(self.add_ax_rm16()?),
            0x04 => Ok(self.add_al_imm8()?),
            0x05 => Ok(self.add_ax_imm16()?),
            0x10 => Ok(self.adc_r8_rm8()?),
            0x12 => Ok(self.adc_al_rm8()?),
            0x08 => Ok(self.or_rm8_r8()?),
            0x38 => Ok(self.cmp_rm8_r8()?),
            0x40 => Ok(self.inc_ax()?),
            0x3C => Ok(self.cmp_al_imm8()?),
            0x3B => Ok(self.cmp_r16_rm16()?),
            0x36 => {
                self.set_segment_override(SegmentRegister::SS);
                self.execute_instruction()?;
                self.clear_segment_override();
                Ok(())
            },
            0x1C => Ok(self.sbb_al_imm8()?),

            // Logic Instructions
            0x20 => Ok(self.and_rm8_r8()?),
            0x30 => Ok(self.xor_rm8_r8()?),
            0x31 => Ok(self.xor_rm16_r16()?),
            0x32 => Ok(self.xor_r8_rm8()?),
            0x33 => Ok(self.xor_r16_rm16()?),

            // Control Flow Instructions
            0xE8 => Ok(self.call_near()?),
            0xE9 => Ok(self.jmp_near()?),
            0xEA => Ok(self.jmp_far()?),
            0xEB => Ok(self.jmp_short()?),
            0xE3 => Ok(self.jcxz()?),
            0xE2 => Ok(self.loop_rel8()?),
            0x70 => Ok(self.jo_rel8()?),
            0x71 => Ok(self.jno_rel8()?),
            0x72 => Ok(self.jb_rel8()?),
            0x73 => Ok(self.jnb_rel8()?),
            0x74 => Ok(self.jz_rel8()?),
            0x75 => Ok(self.jnz_rel8()?),
            0x76 => Ok(self.jbe_rel8()?),
            0x77 => Ok(self.jnbe_rel8()?),

            // String Instructions
            0xA4 => Ok(self.movsb()?),
            0xA5 => Ok(self.movsw()?),
            0xAC => Ok(self.lodsb()?),
            0xAD => Ok(self.lodsw()?),
            0xAA => Ok(self.stosb()?),
            0xAB => Ok(self.stosw()?),

            // Flag Instructions
            0xF8 => Ok(self.clc()?),
            0xF9 => Ok(self.stc()?),
            0xFA => Ok(self.cli()?),
            0xFB => Ok(self.sti()?),
            0xFC => Ok(self.cld()?),
            0xFD => Ok(self.std()?),

            // I/O Instructions
            0xE4 => Ok(self.in_al_imm8()?),
            0xE5 => Ok(self.in_ax_imm8()?),
            0xE6 => Ok(self.out_imm8_al()?),
            0xE7 => Ok(self.out_imm8_ax()?),
            0xEC => Ok(self.in_al_dx()?),
            0xED => Ok(self.in_ax_dx()?),
            0xEE => Ok(self.out_dx_al()?),
            0xEF => Ok(self.out_dx_ax()?),

            // Group Instructions
            0x80 => Ok(self.execute_group1_rm8_imm8(0)?),
            0x81 => Ok(self.handle_81_group()?),
            0x82 => Ok(self.handle_82_group()?),
            0x83 => Ok(self.handle_83_group()?),
            0xF6 => Ok(self.handle_f6_group()?),
            0xF7 => Ok(self.handle_f7_group()?),
            0xFE => Ok(self.handle_fe_group()?),
            0xFF => Ok(self.handle_ff_group()?),

            // Other Instructions
            0x90..=0x97 => Ok(self.xchg_ax_r16(opcode - 0x90)?),
            0xF4 => {
                self.halted = true;
                Ok(())
            },
            // 0xF1 => Ok(self.int1()?),

            // Stack Instructions
            0x50 => Ok(self.push_ax()?),
            0x51 => Ok(self.push_cx()?),
            0x52 => Ok(self.push_dx()?),
            0x53 => Ok(self.push_bx()?),
            0x59 => Ok(self.pop_cx()?),
            0x5A => Ok(self.pop_dx()?),
            0x5B => Ok(self.pop_bx()?),
            0x58 => Ok(self.pop_ax()?),
            0xCF => Ok(self.iret()?),

            // New instructions
            0xD6 => Ok(self.salc()?),
            0xD4 => Ok(self.aam()?),

            // Stack Instructions
            0xC8 => Ok(self.enter()?),
            0xC9 => Ok(self.leave()?),
            0xCA => Ok(self.ret_far_imm16()?),

            // Prefix Instructions
            0xF0 => {
                self.execute_instruction()
            },

            _ => {
                self.halted = true;
                Err(format!("Illegal opcode {:#04X}", opcode))
            }
        }
    }
}
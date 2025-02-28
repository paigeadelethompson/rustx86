use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn jmp_near(&mut self) -> Result<(), String> {
        let old_ip = self.regs.ip;
        let offset = self.fetch_word()? as i16;
        println!("JMP_NEAR: old_ip=0x{:04X}, offset=0x{:04X}", old_ip, offset);

        // The offset should be added to the IP of the next instruction
        // At this point, IP points to the next instruction (after fetch_word)
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        println!("JMP_NEAR: new_ip=0x{:04X}", self.regs.ip);

        Ok(())
    }

    pub(crate) fn jmp_far(&mut self) -> Result<(), String> {
        let old_cs = self.regs.cs;
        let old_ip = self.regs.ip;
        let offset = self.fetch_word()?;
        let segment = self.fetch_word()?;
        self.regs.cs = segment;
        self.regs.ip = offset;
        println!(
            "JMP_FAR: old_cs:ip=0x{:04X}:0x{:04X}, new_cs:ip=0x{:04X}:0x{:04X}",
            old_cs, old_ip, segment, offset
        );
        Ok(())
    }

    pub fn jmp_short(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        let _old_ip = self.regs.ip;
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        Ok(())
    }

    pub(crate) fn call_near(&mut self) -> Result<(), String> {
        let old_ip = self.regs.ip;
        let offset = self.fetch_word()? as i16;

        // Push the IP of the next instruction (after the offset word)
        self.push_word(self.regs.ip)?;

        // Add the offset to the current IP
        self.regs.ip = self.regs.ip.wrapping_add(offset as u16);

        println!(
            "CALL_NEAR: old_ip=0x{:04X}, offset=0x{:04X}, return_ip=0x{:04X}, new_ip=0x{:04X}",
            old_ip,
            offset,
            old_ip.wrapping_add(2),
            self.regs.ip
        );

        Ok(())
    }

    #[allow(dead_code)]
    pub fn call_far(&mut self) -> Result<(), String> {
        let old_cs = self.regs.cs;
        let old_ip = self.regs.ip;
        let offset = self.fetch_word()?;
        let segment = self.fetch_word()?;

        // The return IP should be after both words (offset and segment)
        let return_ip = old_ip.wrapping_add(4);

        // First push the return CS, then the return IP
        self.push_word(old_cs)?;
        self.push_word(return_ip)?;

        // Update CS:IP to the new target
        self.regs.cs = segment;
        self.regs.ip = offset;

        println!("CALL_FAR: old_cs:ip=0x{:04X}:0x{:04X}, return_ip=0x{:04X}, new_cs:ip=0x{:04X}:0x{:04X}",
                 old_cs, old_ip, return_ip, segment, offset);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn ret_near(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        self.regs.ip = ip;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn ret_far(&mut self) -> Result<(), String> {
        let old_cs = self.regs.cs;
        let old_ip = self.regs.ip;

        // Pop IP first, then CS (reverse order of push in call_far)
        let ip = self.pop_word()?;
        let cs = self.pop_word()?;

        // Update CS:IP
        self.regs.ip = ip;
        self.regs.cs = cs;

        println!(
            "RET_FAR: old_cs:ip=0x{:04X}:0x{:04X}, new_cs:ip=0x{:04X}:0x{:04X}",
            old_cs, old_ip, cs, ip
        );

        Ok(())
    }

    #[allow(dead_code)]
    pub fn ret_near_imm16(&mut self) -> Result<(), String> {
        let ip = self.pop_word()?;
        let imm16 = self.fetch_word()?;
        self.regs.ip = ip;
        self.regs.sp = self.regs.sp.wrapping_add(imm16);
        Ok(())
    }

    pub fn jcxz(&mut self) -> Result<(), String> {
        let old_ip = self.regs.ip;
        let offset = self.fetch_byte()? as i8;

        // After fetching the byte, IP points to next instruction
        let next_ip = self.regs.ip;

        println!(
            "JCXZ: old_ip=0x{:04X}, cx=0x{:04X}, offset=0x{:02X}",
            old_ip, self.regs.cx, offset as u8
        );

        if self.regs.cx == 0 {
            // Jump relative to next instruction
            self.regs.ip = next_ip.wrapping_add(offset as u16);
            println!("JCXZ: Jumping to 0x{:04X}", self.regs.ip);
        } else {
            println!("JCXZ: Not jumping, CX != 0");
        }

        Ok(())
    }

    pub(crate) fn loop_cx(&mut self) -> Result<(), String> {
        let old_ip = self.regs.ip;
        let offset = self.fetch_byte()? as i8;
        let next_ip = self.regs.ip; // IP after fetching the offset

        let old_cx = self.regs.cx;
        self.regs.cx = self.regs.cx.wrapping_sub(1);

        println!(
            "LOOP: old_ip=0x{:04X}, old_cx=0x{:04X}, new_cx=0x{:04X}, offset=0x{:02X}",
            old_ip, old_cx, self.regs.cx, offset as u8
        );

        if self.regs.cx != 0 {
            // Jump relative to next instruction
            self.regs.ip = next_ip.wrapping_add(offset as u16);
            println!("LOOP: Jumping to 0x{:04X}", self.regs.ip);
        } else {
            println!("LOOP: Not jumping, CX = 0");
        }

        Ok(())
    }

    pub(crate) fn jz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jnz_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jo_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_overflow() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jno_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_overflow() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_carry() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jnb_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_carry() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if self.regs.flags.get_carry() || self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }

    pub(crate) fn jnbe_rel8(&mut self) -> Result<(), String> {
        let offset = self.fetch_byte()? as i8;
        if !self.regs.flags.get_carry() && !self.regs.flags.get_zero() {
            let _old_ip = self.regs.ip;
            self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::disk_image::DiskImage;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use std::path::Path;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024)); // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(Path::new("drive_c/")).expect("Failed to create disk image");
        Cpu::new(memory, serial, disk)
    }

    #[test]
    fn test_jmp_near() {
        let mut cpu = setup_cpu();
        cpu.regs.cs = 0x100; // Set CS to 0x100 so CS:IP points to physical address 0x1000
        cpu.regs.ip = 0x0000; // Set IP to 0x0000
        cpu.memory.write_word(0x1000, 0x0100); // Jump offset of 0x0100
        assert!(cpu.jmp_near().is_ok());
        assert_eq!(cpu.regs.ip, 0x0102); // 0x0000 + 2 + 0x0100
    }

    #[test]
    fn test_jmp_far() {
        let mut cpu = setup_cpu();
        cpu.regs.cs = 0x100; // Set CS to 0x100 so CS:IP points to physical address 0x1000
        cpu.regs.ip = 0x0000; // Set IP to 0x0000

        // Write the far jump target at physical address 0x1000
        cpu.memory.write_word(0x1000, 0x3000); // New IP
        cpu.memory.write_word(0x1002, 0x4000); // New CS

        assert!(cpu.jmp_far().is_ok());
        assert_eq!(cpu.regs.ip, 0x3000); // Should jump to new IP
        assert_eq!(cpu.regs.cs, 0x4000); // Should update CS
    }

    #[test]
    fn test_jmp_short() {
        let mut cpu = setup_cpu();
        cpu.regs.cs = 0x100; // Set CS to 0x100 so CS:IP points to physical address 0x1000
        cpu.regs.ip = 0x0000; // Set IP to 0x0000

        // Write the short jump offset at physical address 0x1000
        cpu.memory.write_byte(0x1000, 0x10); // Jump offset of 16

        assert!(cpu.jmp_short().is_ok());
        assert_eq!(cpu.regs.ip, 0x0011); // 0x0000 + 1 + 0x10
    }

    #[test]
    fn test_call_near() {
        let mut cpu = setup_cpu();
        cpu.regs.cs = 0x100; // Set CS to 0x100 so CS:IP points to physical address 0x1000
        cpu.regs.ip = 0x0000; // Set IP to 0x0000
        cpu.regs.ss = 0x200; // Set SS to 0x200 so SS:SP points to physical address 0x2000
        cpu.regs.sp = 0x0000; // Set SP to 0x0000

        // Write the near call offset at physical address 0x1000
        cpu.memory.write_word(0x1000, 0x0100); // Call offset of 0x0100

        assert!(cpu.call_near().is_ok());

        // Check that old IP was pushed to SS:SP
        let stack_addr = ((cpu.regs.ss as u32) << 4) | (cpu.regs.sp as u32);
        assert_eq!(
            cpu.memory.read_word(stack_addr),
            0x0002, // Old IP is after the offset word
            "Expected return IP 0x0002 at stack address 0x{:05X}, got 0x{:04X}",
            stack_addr,
            cpu.memory.read_word(stack_addr)
        );

        // Check new IP
        assert_eq!(
            cpu.regs.ip, 0x0102,
            "Expected new IP 0x0102, got 0x{:04X}",
            cpu.regs.ip
        ); // 0x0000 + 2 + 0x0100

        // Check that SP was decremented
        assert_eq!(
            cpu.regs.sp, 0xFFFE,
            "Expected SP 0xFFFE, got 0x{:04X}",
            cpu.regs.sp
        ); // 0x0000 - 2
    }

    #[test]
    fn test_call_far() {
        let mut cpu = setup_cpu();

        // Set up segment registers
        cpu.regs.cs = 0x2000; // Code segment
        cpu.regs.ss = 0x3000; // Stack segment
        cpu.regs.ip = 0x1000; // Instruction pointer
        cpu.regs.sp = 0x2000; // Stack pointer

        // Calculate physical addresses
        let code_addr = (cpu.regs.cs as u32) << 4 | (cpu.regs.ip as u32);
        println!("Writing new IP:CS at physical address 0x{:05X}", code_addr);

        // Write the far call target
        cpu.memory.write_word(code_addr, 0x3000); // New IP
        cpu.memory.write_word(code_addr + 2, 0x4000); // New CS

        assert!(cpu.call_far().is_ok());

        // After pushing CS and IP, SP should be at 0x1FFC
        // The stack grows downward, so:
        // CS is at SS:0x1FFE (physical: SS<<4 | 0x1FFE)
        // IP is at SS:0x1FFC (physical: SS<<4 | 0x1FFC)
        let stack_cs_addr = (cpu.regs.ss as u32) << 4 | 0x1FFE;
        let stack_ip_addr = (cpu.regs.ss as u32) << 4 | 0x1FFC;

        println!(
            "Reading return CS:IP from stack at physical addresses 0x{:05X} and 0x{:05X}",
            stack_cs_addr, stack_ip_addr
        );

        // Check that old CS and IP were pushed correctly
        assert_eq!(
            cpu.memory.read_word(stack_cs_addr),
            0x2000, // Old CS
            "Return CS on stack incorrect"
        );
        assert_eq!(
            cpu.memory.read_word(stack_ip_addr),
            0x1004, // Return IP should be after both words
            "Return IP on stack incorrect"
        );

        // Check that CS:IP was updated correctly
        assert_eq!(cpu.regs.ip, 0x3000, "New IP incorrect");
        assert_eq!(cpu.regs.cs, 0x4000, "New CS incorrect");

        // Check that SP was decremented correctly
        assert_eq!(
            cpu.regs.sp, 0x1FFC,
            "Stack pointer not decremented correctly"
        );
    }

    #[test]
    fn test_ret_near() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFE;
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x1234);
        assert!(cpu.ret_near().is_ok());
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_ret_far() {
        let mut cpu = setup_cpu();

        // Set up segment registers
        cpu.regs.cs = 0x2000; // Current code segment
        cpu.regs.ss = 0x3000; // Stack segment
        cpu.regs.ip = 0x1000; // Current instruction pointer
        cpu.regs.sp = 0x1FFC; // Stack pointer

        // Calculate physical addresses for stack
        let stack_cs_addr = (cpu.regs.ss as u32) << 4 | 0x1FFE;
        let stack_ip_addr = (cpu.regs.ss as u32) << 4 | 0x1FFC;
        println!(
            "Writing return CS:IP to stack at physical addresses 0x{:05X} and 0x{:05X}",
            stack_cs_addr, stack_ip_addr
        );

        // Write the return CS:IP values to the stack
        cpu.memory.write_word(stack_ip_addr, 0x5678); // Return IP
        cpu.memory.write_word(stack_cs_addr, 0x1234); // Return CS

        assert!(cpu.ret_far().is_ok());

        // Check that CS:IP was updated correctly
        assert_eq!(cpu.regs.ip, 0x5678, "New IP incorrect");
        assert_eq!(cpu.regs.cs, 0x1234, "New CS incorrect");

        // Check that SP was incremented correctly
        assert_eq!(
            cpu.regs.sp, 0x2000,
            "Stack pointer not incremented correctly"
        );
    }

    #[test]
    fn test_jcxz() {
        let mut cpu = setup_cpu();

        // Set up segment registers
        cpu.regs.cs = 0x2000; // Code segment
        cpu.regs.ip = 0x1000; // Instruction pointer
        cpu.regs.cx = 0; // CX = 0, should jump

        // Calculate physical address for instruction
        let code_addr = (cpu.regs.cs as u32) << 4 | (cpu.regs.ip as u32);
        println!(
            "Writing jump offset at physical address 0x{:05X}",
            code_addr
        );

        // Write the jump offset
        cpu.memory.write_byte(code_addr, 0x10); // Jump offset of 16

        // Test jumping when CX = 0
        assert!(cpu.jcxz().is_ok());
        assert_eq!(cpu.regs.ip, 0x1011, "IP incorrect after jump when CX = 0"); // IP + 1 + offset

        // Test not jumping when CX != 0
        cpu.regs.ip = 0x1000; // Reset IP
        cpu.regs.cx = 1; // Set CX to non-zero
        assert!(cpu.jcxz().is_ok());
        assert_eq!(
            cpu.regs.ip, 0x1001,
            "IP incorrect after no jump when CX != 0"
        ); // IP + 1 (no jump)
    }

    #[test]
    fn test_loop_cx() {
        let mut cpu = setup_cpu();

        // Set up segment registers
        cpu.regs.cs = 0x2000; // Code segment
        cpu.regs.ip = 0x1000; // Instruction pointer
        cpu.regs.cx = 2; // Set CX to 2 for two iterations

        // Calculate physical address for instruction
        let code_addr = (cpu.regs.cs as u32) << 4 | (cpu.regs.ip as u32);
        println!(
            "Writing loop offset at physical address 0x{:05X}",
            code_addr
        );

        // Write the loop offset
        cpu.memory.write_byte(code_addr, 0x10); // Jump offset of 16

        // First iteration (CX = 2 -> 1)
        assert!(cpu.loop_cx().is_ok());
        assert_eq!(cpu.regs.cx, 1, "CX should be decremented to 1");
        assert_eq!(cpu.regs.ip, 0x1011, "Should jump when CX becomes 1"); // IP + 1 + offset

        // Reset IP to simulate loop start
        cpu.regs.ip = 0x1000;
        cpu.memory.write_byte(code_addr, 0x10); // Write offset again

        // Second iteration (CX = 1 -> 0)
        assert!(cpu.loop_cx().is_ok());
        assert_eq!(cpu.regs.cx, 0, "CX should be decremented to 0");
        assert_eq!(cpu.regs.ip, 0x1001, "Should not jump when CX becomes 0"); // IP + 1 (no jump)

        // Reset IP to simulate loop start
        cpu.regs.ip = 0x1000;
        cpu.memory.write_byte(code_addr, 0x10); // Write offset again

        // Third iteration (CX = 0 -> 0xFFFF)
        assert!(cpu.loop_cx().is_ok());
        assert_eq!(cpu.regs.cx, 0xFFFF, "CX should underflow to 0xFFFF");
        assert_eq!(cpu.regs.ip, 0x1011, "Should jump when CX becomes 0xFFFF"); // IP + 1 + offset
    }
}

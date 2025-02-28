use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn and_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val & reg_val;
        self.write_rm8(modrm, result)?;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn or_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val | reg_val;
        self.write_rm8(modrm, result)?;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        println!("XOR_RM8_R8: ModR/M byte = {:#04x}", modrm);

        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);

        println!(
            "XOR_RM8_R8: rm_val = {:#04x}, reg_val = {:#04x}",
            rm_val, reg_val
        );

        let result = rm_val ^ reg_val;
        println!("XOR_RM8_R8: result = {:#04x}", result);

        self.write_rm8(modrm, result)?;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);

        println!(
            "XOR_RM8_R8: Flags - ZF={}, SF={}, PF={}",
            self.regs.flags.get_zero(),
            self.regs.flags.get_sign(),
            self.regs.flags.get_parity()
        );

        Ok(())
    }

    pub(crate) fn xor_r8_rm8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg8(reg);
        let result = rm_val ^ reg_val;
        self.regs.set_reg8(reg, result)?;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn test_rm8_r8(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm8(modrm)?;
        let reg_val = self.regs.get_reg8((modrm >> 3) & 0x07);
        let result = rm_val & reg_val;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn test_al_imm8(&mut self) -> Result<(), String> {
        let imm8 = self.fetch_byte()?;
        let al = self.regs.get_al();
        let result = al & imm8;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn xor_al_imm8(&mut self) -> Result<(), String> {
        let imm = self.fetch_byte()?;
        let al = self.regs.get_al();
        let result = al ^ imm;
        self.regs.set_al(result);

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x80) != 0);
        self.regs.flags.set_overflow(false);
        self.regs.flags.set_parity(result.count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_rm16_r16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg_val = self.regs.get_reg16((modrm >> 3) & 0x07);
        let result = rm_val ^ reg_val;
        self.write_rm16(modrm, result)?;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(false);
        self.regs
            .flags
            .set_parity((result as u8).count_ones() % 2 == 0);
        Ok(())
    }

    pub(crate) fn xor_r16_rm16(&mut self) -> Result<(), String> {
        let modrm = self.fetch_byte()?;
        let rm_val = self.get_rm16(modrm)?;
        let reg = (modrm >> 3) & 0x07;
        let reg_val = self.regs.get_reg16(reg);
        let result = rm_val ^ reg_val;
        self.regs.set_reg16(reg, result)?;

        // Update flags
        self.regs.flags.set_carry(false);
        self.regs.flags.set_zero(result == 0);
        self.regs.flags.set_sign((result & 0x8000) != 0);
        self.regs.flags.set_overflow(false);
        self.regs
            .flags
            .set_parity((result as u8).count_ones() % 2 == 0);
        Ok(())
    }

    // More logic instructions can be added here...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::disk_image::DiskImage;
    use crate::memory::ram::RamMemory;
    use crate::serial::Serial;
    use std::path::PathBuf;

    fn setup_cpu() -> Cpu {
        let memory = Box::new(RamMemory::new(1024 * 1024)); // 1MB RAM
        let serial = Serial::new();
        let disk = DiskImage::new(&PathBuf::from("drive_c/")).expect("Failed to create disk image");
        Cpu::new(memory, serial, disk)
    }

    #[test]
    fn test_and_rm8_r8() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x0F0F; // AL = 0x0F
        cpu.memory.write_byte(0, 0xC0); // ModR/M byte for register-to-register
        assert!(cpu.and_rm8_r8().is_ok());
        assert_eq!(cpu.regs.ax & 0xFF, 0x0F); // Result should be 0x0F & 0x0F = 0x0F
        assert!(!cpu.regs.flags.get_carry());
        assert!(!cpu.regs.flags.get_overflow());
    }

    #[test]
    fn test_or_rm8_r8() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x0F0F; // AL = 0x0F
        cpu.memory.write_byte(0, 0xC0); // ModR/M byte for register-to-register
        assert!(cpu.or_rm8_r8().is_ok());
        assert_eq!(cpu.regs.ax & 0xFF, 0x0F); // Result should be 0x0F | 0x0F = 0x0F
        assert!(!cpu.regs.flags.get_carry());
        assert!(!cpu.regs.flags.get_overflow());
    }

    #[test]
    fn test_xor_rm8_r8() {
        let mut cpu = setup_cpu();

        // Set up initial register values
        cpu.regs.ax = 0x0F0F; // AL = 0x0F
        cpu.regs.cs = 0; // Set code segment to 0
        cpu.regs.ip = 0x0100; // Set instruction pointer

        // ModR/M byte 0xC0 means:
        // - mod = 11 (register-to-register)
        // - reg = 000 (AL)
        // - r/m = 000 (AL)
        let modrm = 0xC0;
        cpu.memory.write_byte(0x0100, modrm);
        println!(
            "Test setup: Writing ModR/M byte {:#04x} at IP {:#06x}",
            modrm, cpu.regs.ip
        );

        // Execute XOR instruction
        assert!(cpu.xor_rm8_r8().is_ok());

        // Check result (0x0F ^ 0x0F = 0x00)
        let result = cpu.regs.ax & 0xFF;
        println!("Test result: AL = {:#04x}", result);
        assert_eq!(result, 0x00, "XOR result should be 0x00");

        // Check flags
        assert!(!cpu.regs.flags.get_carry(), "CF should be clear");
        assert!(!cpu.regs.flags.get_overflow(), "OF should be clear");
        assert!(cpu.regs.flags.get_zero(), "ZF should be set");
        assert!(!cpu.regs.flags.get_sign(), "SF should be clear");
        assert!(
            cpu.regs.flags.get_parity(),
            "PF should be set for even number of 1s"
        );

        // Check that IP was incremented
        assert_eq!(cpu.regs.ip, 0x0101, "IP should be incremented by 1");
    }

    #[test]
    fn test_xor_r8_rm8() {
        let mut cpu = setup_cpu();

        // Set up initial register values
        cpu.regs.ax = 0x0F0F; // AL = 0x0F
        cpu.regs.cs = 0; // Set code segment to 0
        cpu.regs.ip = 0x0100; // Set instruction pointer

        // ModR/M byte 0xC0 means:
        // - mod = 11 (register-to-register)
        // - reg = 000 (AL)
        // - r/m = 000 (AL)
        let modrm = 0xC0;
        cpu.memory.write_byte(0x0100, modrm);
        println!(
            "Test setup: Writing ModR/M byte {:#04x} at IP {:#06x}",
            modrm, cpu.regs.ip
        );

        // Execute XOR instruction
        assert!(cpu.xor_r8_rm8().is_ok());

        // Check result (0x0F ^ 0x0F = 0x00)
        let result = cpu.regs.ax & 0xFF;
        println!("Test result: AL = {:#04x}", result);
        assert_eq!(result, 0x00, "XOR result should be 0x00");

        // Check flags
        assert!(!cpu.regs.flags.get_carry(), "CF should be clear");
        assert!(!cpu.regs.flags.get_overflow(), "OF should be clear");
        assert!(cpu.regs.flags.get_zero(), "ZF should be set");
        assert!(!cpu.regs.flags.get_sign(), "SF should be clear");
        assert!(
            cpu.regs.flags.get_parity(),
            "PF should be set for even number of 1s"
        );

        // Check that IP was incremented
        assert_eq!(cpu.regs.ip, 0x0101, "IP should be incremented by 1");
    }

    #[test]
    fn test_test_rm8_r8() {
        let mut cpu = setup_cpu();

        // Set up initial register values
        cpu.regs.ax = 0x0F0F; // AL = 0x0F
        cpu.regs.cs = 0; // Set code segment to 0
        cpu.regs.ip = 0x0100; // Set instruction pointer

        // ModR/M byte 0xC0 means:
        // - mod = 11 (register-to-register)
        // - reg = 000 (AL)
        // - r/m = 000 (AL)
        let modrm = 0xC0;
        cpu.memory.write_byte(0x0100, modrm);
        println!(
            "Test setup: Writing ModR/M byte {:#04x} at IP {:#06x}",
            modrm, cpu.regs.ip
        );

        // Execute TEST instruction
        assert!(cpu.test_rm8_r8().is_ok());

        // Check result (0x0F & 0x0F = 0x0F)
        let result = cpu.regs.ax & 0xFF;
        println!("Test result: AL = {:#04x} (unchanged)", result);
        assert_eq!(result, 0x0F, "AL should be unchanged");

        // Check flags
        assert!(!cpu.regs.flags.get_carry(), "CF should be clear");
        assert!(!cpu.regs.flags.get_overflow(), "OF should be clear");
        assert!(
            !cpu.regs.flags.get_zero(),
            "ZF should be clear (result is non-zero)"
        );
        assert!(
            !cpu.regs.flags.get_sign(),
            "SF should be clear (result is positive)"
        );
        assert!(
            cpu.regs.flags.get_parity(),
            "PF should be set (even number of 1s)"
        );

        // Check that IP was incremented
        assert_eq!(cpu.regs.ip, 0x0101, "IP should be incremented by 1");
    }

    #[test]
    fn test_test_al_imm8() {
        let mut cpu = setup_cpu();
        cpu.regs.ax = 0x00FF; // AL = 0xFF
        cpu.memory.write_byte(0, 0x00); // Test with 0
        assert!(cpu.test_al_imm8().is_ok());
        assert!(!cpu.regs.flags.get_carry());
        assert!(!cpu.regs.flags.get_overflow());
        assert!(cpu.regs.flags.get_zero()); // 0xFF & 0x00 = 0
    }

    #[test]
    fn test_xor_al_imm8() {
        let mut cpu = setup_cpu();

        // Set up initial register values
        cpu.regs.ax = 0x00FF; // AL = 0xFF
        cpu.regs.cs = 0; // Set code segment to 0
        cpu.regs.ip = 0x0100; // Set instruction pointer

        // Write immediate value 0xFF
        cpu.memory.write_byte(0x0100, 0xFF);
        println!(
            "Test setup: Writing immediate value 0xFF at IP {:#06x}",
            cpu.regs.ip
        );

        // Execute XOR instruction
        assert!(cpu.xor_al_imm8().is_ok());

        // Check result (0xFF ^ 0xFF = 0x00)
        let result = cpu.regs.ax & 0xFF;
        println!("Test result: AL = {:#04x}", result);
        assert_eq!(result, 0x00, "XOR result should be 0x00");

        // Check flags
        assert!(!cpu.regs.flags.get_carry(), "CF should be clear");
        assert!(!cpu.regs.flags.get_overflow(), "OF should be clear");
        assert!(cpu.regs.flags.get_zero(), "ZF should be set");
        assert!(!cpu.regs.flags.get_sign(), "SF should be clear");
        assert!(
            cpu.regs.flags.get_parity(),
            "PF should be set for even number of 1s"
        );

        // Check that IP was incremented
        assert_eq!(cpu.regs.ip, 0x0101, "IP should be incremented by 1");
    }

    #[test]
    fn test_xor_rm16_r16() {
        let mut cpu = setup_cpu();

        // Set up initial register values
        cpu.regs.ax = 0xFFFF;
        cpu.regs.cs = 0; // Set code segment to 0
        cpu.regs.ip = 0x0100; // Set instruction pointer

        // ModR/M byte 0xC0 means:
        // - mod = 11 (register-to-register)
        // - reg = 000 (AX)
        // - r/m = 000 (AX)
        let modrm = 0xC0;
        cpu.memory.write_byte(0x0100, modrm);
        println!(
            "Test setup: Writing ModR/M byte {:#04x} at IP {:#06x}",
            modrm, cpu.regs.ip
        );

        // Execute XOR instruction
        assert!(cpu.xor_rm16_r16().is_ok());

        // Check result (0xFFFF ^ 0xFFFF = 0x0000)
        let result = cpu.regs.ax;
        println!("Test result: AX = {:#06x}", result);
        assert_eq!(result, 0x0000, "XOR result should be 0x0000");

        // Check flags
        assert!(!cpu.regs.flags.get_carry(), "CF should be clear");
        assert!(!cpu.regs.flags.get_overflow(), "OF should be clear");
        assert!(cpu.regs.flags.get_zero(), "ZF should be set");
        assert!(!cpu.regs.flags.get_sign(), "SF should be clear");
        assert!(
            cpu.regs.flags.get_parity(),
            "PF should be set for even number of 1s"
        );

        // Check that IP was incremented
        assert_eq!(cpu.regs.ip, 0x0101, "IP should be incremented by 1");
    }

    #[test]
    fn test_xor_r16_rm16() {
        let mut cpu = setup_cpu();

        // Set up initial register values
        cpu.regs.ax = 0xFFFF;
        cpu.regs.cs = 0; // Set code segment to 0
        cpu.regs.ip = 0x0100; // Set instruction pointer

        // ModR/M byte 0xC0 means:
        // - mod = 11 (register-to-register)
        // - reg = 000 (AX)
        // - r/m = 000 (AX)
        let modrm = 0xC0;
        cpu.memory.write_byte(0x0100, modrm);
        println!(
            "Test setup: Writing ModR/M byte {:#04x} at IP {:#06x}",
            modrm, cpu.regs.ip
        );

        // Execute XOR instruction
        assert!(cpu.xor_r16_rm16().is_ok());

        // Check result (0xFFFF ^ 0xFFFF = 0x0000)
        let result = cpu.regs.ax;
        println!("Test result: AX = {:#06x}", result);
        assert_eq!(result, 0x0000, "XOR result should be 0x0000");

        // Check flags
        assert!(!cpu.regs.flags.get_carry(), "CF should be clear");
        assert!(!cpu.regs.flags.get_overflow(), "OF should be clear");
        assert!(cpu.regs.flags.get_zero(), "ZF should be set");
        assert!(!cpu.regs.flags.get_sign(), "SF should be clear");
        assert!(
            cpu.regs.flags.get_parity(),
            "PF should be set for even number of 1s"
        );

        // Check that IP was incremented
        assert_eq!(cpu.regs.ip, 0x0101, "IP should be incremented by 1");
    }
}

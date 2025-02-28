use crate::bios::handle_bios_interrupt;
use crate::cpu::Cpu;

impl Cpu {
    pub(crate) fn int(&mut self, interrupt_number: u8) -> Result<(), String> {
        // Save flags and CS:IP
        let flags = self.regs.flags.as_word();
        let cs = self.regs.cs;
        let ip = self.regs.ip;

        // Push flags and return address
        self.push_word(flags)?;
        self.push_word(cs)?;
        self.push_word(ip)?;

        // Clear IF and TF
        self.regs.flags.set_interrupt(false);
        self.regs.flags.set_trap(false);

        // Get interrupt vector
        let vector_addr = (interrupt_number as u32) * 4;
        let new_ip = self.memory.read_word(vector_addr);
        let new_cs = self.memory.read_word(vector_addr + 2);

        // Handle BIOS interrupts
        if new_cs == 0xF000 {
            handle_bios_interrupt(self, interrupt_number)?;
        }

        // Jump to interrupt handler
        self.regs.cs = new_cs;
        self.regs.ip = new_ip;

        Ok(())
    }

    pub(crate) fn iret(&mut self) -> Result<(), String> {
        // Pop IP, CS, and FLAGS
        let new_ip = self.pop_word()?;
        let new_cs = self.pop_word()?;
        let flags = self.pop_word()?;

        println!(
            "IRET: Popped IP=0x{:04X}, CS=0x{:04X}, FLAGS=0x{:04X}",
            new_ip, new_cs, flags
        );

        // Set flags, ensuring reserved bits 1 and 3 are set
        let flags_with_reserved = flags | 0x000A; // Set bits 1 and 3
        println!(
            "IRET: Setting flags with reserved bits: 0x{:04X}",
            flags_with_reserved
        );

        self.regs.ip = new_ip;
        self.regs.cs = new_cs;
        self.regs.flags.set_from_word(flags_with_reserved);

        println!(
            "IRET: Final flags value: 0x{:04X}",
            self.regs.flags.as_word()
        );

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
        let mut cpu = Cpu::new(memory, serial, disk);
        cpu.regs.sp = 0x2000; // Initialize stack pointer
        cpu
    }

    #[test]
    fn test_int_basic() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_interrupt(true);
        cpu.regs.flags.set_trap(true);
        cpu.regs.cs = 0x1000;
        cpu.regs.ip = 0x2000;

        // Set up interrupt vector
        let interrupt_num = 0x21;
        let vector_addr = (interrupt_num as u32) * 4;
        cpu.memory.write_word(vector_addr, 0x3000); // IP
        cpu.memory.write_word(vector_addr + 2, 0x4000); // CS

        assert!(cpu.int(interrupt_num).is_ok());

        // Check that flags and return address were pushed
        let sp = cpu.regs.sp as u32;
        assert_eq!(cpu.memory.read_word((cpu.regs.ss as u32) << 4 | sp), 0x2000); // IP
        assert_eq!(
            cpu.memory.read_word((cpu.regs.ss as u32) << 4 | (sp + 2)),
            0x1000
        ); // CS

        // Check that flags were cleared
        assert!(!cpu.regs.flags.get_interrupt());
        assert!(!cpu.regs.flags.get_trap());

        // Check that we jumped to the interrupt handler
        assert_eq!(cpu.regs.cs, 0x4000);
        assert_eq!(cpu.regs.ip, 0x3000);
    }

    #[test]
    fn test_iret() {
        let mut cpu = setup_cpu();
        cpu.regs.sp = 0x1FFA;

        // Push test values onto stack in correct order (IP, CS, FLAGS)
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFE, 0x0202); // Flags with IF set
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFC, 0x2000); // CS
        cpu.memory
            .write_word((cpu.regs.ss as u32) << 4 | 0x1FFA, 0x3000); // IP

        assert!(cpu.iret().is_ok());

        // Check that values were popped correctly
        assert_eq!(cpu.regs.ip, 0x3000);
        assert_eq!(cpu.regs.cs, 0x2000);
        assert_eq!(cpu.regs.flags.as_u16() & 0x0202, 0x0202); // Check IF and reserved bit
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn test_int_bios() {
        let mut cpu = setup_cpu();
        cpu.regs.flags.set_interrupt(true);
        cpu.regs.cs = 0x1000;
        cpu.regs.ip = 0x2000;

        // Set up BIOS interrupt vector
        let interrupt_num = 0x10; // Video services
        let vector_addr = (interrupt_num as u32) * 4;
        cpu.memory.write_word(vector_addr, 0x0000); // IP
        cpu.memory.write_word(vector_addr + 2, 0xF000); // CS (BIOS segment)

        assert!(cpu.int(interrupt_num).is_ok());

        // Check that flags were cleared
        assert!(!cpu.regs.flags.get_interrupt());
    }
}

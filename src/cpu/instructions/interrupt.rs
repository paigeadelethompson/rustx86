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

        self.regs.ip = new_ip;
        self.regs.cs = new_cs;
        self.regs.flags.set_from_word(flags);

        Ok(())
    }
}

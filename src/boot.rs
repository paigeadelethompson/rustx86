pub struct BootSector {
    data: [u8; 512]
}

impl BootSector {
    pub fn new() -> Self {
        // Create an empty boot sector
        let mut data = [0; 512];
        
        // Boot code that prints "Hello from DOS!" using serial port
        let code = [
            0xFA,                   // CLI - Disable interrupts during setup
            0xB4, 0x00,            // MOV AH, 0x00    ; Initialize serial port
            0xB2, 0x00,            // MOV DL, 0x00    ; COM1
            0xCD, 0x14,            // INT 0x14        ; Call BIOS serial service
            0xBE, 0x20, 0x7C,      // MOV SI, message ; Load message address
            // print_loop:
            0xAC,                  // LODSB           ; Load next character
            0x08, 0xC0,           // OR AL, AL       ; Test for end of string
            0x74, 0x04,           // JZ done         ; If zero, we're done
            0xB4, 0x01,           // MOV AH, 0x01    ; Write character function
            0xCD, 0x14,           // INT 0x14        ; Call BIOS serial service
            0xEB, 0xF5,           // JMP print_loop  ; Repeat
            // done:
            0xF4,                 // HLT             ; Stop execution
            0x90, 0x90,          // NOP, NOP        ; Padding
            // message: (at offset 0x20)
            b'H', b'e', b'l', b'l', b'o', b' ',
            b'f', b'r', b'o', b'm', b' ',
            b'D', b'O', b'S', b'!', 0x0D, 0x0A, 0
        ];
        
        // Copy boot code
        data[..code.len()].copy_from_slice(&code);
        
        // Fill the rest with NOPs until the boot signature
        for i in code.len()..510 {
            data[i] = 0x90; // NOP
        }
        
        // Boot signature
        data[510] = 0x55;
        data[511] = 0xAA;
        
        BootSector { data }
    }
    
    pub fn load(&self, memory: &mut crate::memory::Memory) {
        // Load boot sector at 0x7C00
        for (i, &byte) in self.data.iter().enumerate() {
            memory.write_byte(0x7C00 + i as u32, byte);
        }
        
        // Verify the load
        for i in 0..self.data.len() {
            let loaded = memory.read_byte(0x7C00 + i as u32);
            if loaded != self.data[i] {
                println!("Mismatch at offset {:#04x}: expected {:#02x}, got {:#02x}", 
                    i, self.data[i], loaded);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn test_boot_sector_creation() {
        let boot = BootSector::new();
        assert_eq!(boot.data[0], 0xFA); // CLI
        assert_eq!(boot.data[1], 0xB4); // MOV AH, 0x00
        assert_eq!(boot.data[2], 0x00);
        assert_eq!(boot.data[510], 0x55); // Boot signature
        assert_eq!(boot.data[511], 0xAA);
    }

    #[test]
    fn test_boot_sector_loading() {
        let boot = BootSector::new();
        let mut memory = Memory::new(1024 * 1024); // 1MB
        boot.load(&mut memory);
        
        assert_eq!(memory.read_byte(0x7C00), 0xFA); // CLI
        assert_eq!(memory.read_byte(0x7C01), 0xB4); // MOV AH, 0x00
        assert_eq!(memory.read_byte(0x7C02), 0x00);
        assert_eq!(memory.read_byte(0x7C00 + 510), 0x55);
        assert_eq!(memory.read_byte(0x7C00 + 511), 0xAA);
    }
} 
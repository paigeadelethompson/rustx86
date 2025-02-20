pub struct BiosRom {
    data: Vec<u8>,
}

impl BiosRom {
    pub fn new() -> Self {
        // Initialize with basic BIOS code
        let mut data = vec![0; 0x10000]; // 64KB ROM
        
        // Add serial interrupt handler at F000:E000
        let serial_handler = [
            0x50,             // PUSH AX
            0x53,             // PUSH BX
            0x51,             // PUSH CX
            0x52,             // PUSH DX
            0x80, 0xFC, 0x01, // CMP AH, 0x01 (write character)
            0x75, 0x09,       // JNE skip_write
            // Handle serial write
            0xB2, 0x00,       // MOV DL, 0 (COM1)
            0xE6, 0xF8,       // OUT 0xF8, AL (write to COM1)
            0xB4, 0x00,       // MOV AH, 0 (success)
            0xEB, 0x03,       // JMP done
            // skip_write:
            0x90, 0x90, 0x90, // NOP padding
            // done:
            0x5A,             // POP DX
            0x59,             // POP CX
            0x5B,             // POP BX
            0x58,             // POP AX
            0xCF,             // IRET
        ];
        
        // Copy serial handler to ROM
        let serial_offset = 0xE000;
        data[serial_offset..serial_offset + serial_handler.len()]
            .copy_from_slice(&serial_handler);
        
        // Basic BIOS initialization code at F000:FFF0 (offset 0xFFF0)
        let reset_vector = 0xFFF0;
        
        // Far JMP to F000:E05B
        data[reset_vector] = 0xEA;     // Far JMP
        data[reset_vector + 1] = 0x5B; // IP = E05B
        data[reset_vector + 2] = 0xE0;
        data[reset_vector + 3] = 0x00; // CS = F000
        data[reset_vector + 4] = 0xF0;

        // BIOS entry point at F000:E05B (offset 0xE05B)
        let entry_point = 0xE05B;
        
        // Basic initialization sequence
        let init_code = [
            0xFA,             // CLI - Disable interrupts
            0x31, 0xC0,      // XOR AX, AX
            0x8E, 0xD8,      // MOV DS, AX
            0x8E, 0xC0,      // MOV ES, AX
            0x8E, 0xD0,      // MOV SS, AX
            0xBC, 0x00, 0x7C, // MOV SP, 0x7C00
            0xFB,             // STI - Enable interrupts
            0xEA,             // Far JMP to 0000:7C00 (boot sector)
            0x00, 0x7C,      // IP = 7C00
            0x00, 0x00,      // CS = 0000
        ];

        data[entry_point..entry_point + init_code.len()].copy_from_slice(&init_code);

        BiosRom { data }
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn from_data(mut data: Vec<u8>) -> Self {
        // Ensure the data is 64KB
        if data.len() < 0x10000 {
            data.resize(0x10000, 0);
        }

        // Add serial interrupt handler at F000:E000
        let serial_handler = [
            0x50,             // PUSH AX
            0x53,             // PUSH BX
            0x51,             // PUSH CX
            0x52,             // PUSH DX
            0x80, 0xFC, 0x01, // CMP AH, 0x01 (write character)
            0x75, 0x09,       // JNE skip_write
            // Handle serial write
            0xB2, 0x00,       // MOV DL, 0 (COM1)
            0xE6, 0xF8,       // OUT 0xF8, AL (write to COM1)
            0xB4, 0x00,       // MOV AH, 0 (success)
            0xEB, 0x03,       // JMP done
            // skip_write:
            0x90, 0x90, 0x90, // NOP padding
            // done:
            0x5A,             // POP DX
            0x59,             // POP CX
            0x5B,             // POP BX
            0x58,             // POP AX
            0xCF,             // IRET
        ];
        
        // Copy serial handler to ROM
        let serial_offset = 0xE000;
        data[serial_offset..serial_offset + serial_handler.len()]
            .copy_from_slice(&serial_handler);
        
        // Basic BIOS initialization code at F000:FFF0 (offset 0xFFF0)
        let reset_vector = 0xFFF0;
        
        // Far JMP to F000:E05B
        data[reset_vector] = 0xEA;     // Far JMP
        data[reset_vector + 1] = 0x5B; // IP = E05B
        data[reset_vector + 2] = 0xE0;
        data[reset_vector + 3] = 0x00; // CS = F000
        data[reset_vector + 4] = 0xF0;

        // BIOS entry point at F000:E05B (offset 0xE05B)
        let entry_point = 0xE05B;
        
        // Basic initialization sequence
        let init_code = [
            0xFA,             // CLI - Disable interrupts
            0x31, 0xC0,      // XOR AX, AX
            0x8E, 0xD8,      // MOV DS, AX
            0x8E, 0xC0,      // MOV ES, AX
            0x8E, 0xD0,      // MOV SS, AX
            0xBC, 0x00, 0x7C, // MOV SP, 0x7C00
            0xFB,             // STI - Enable interrupts
            0xEA,             // Far JMP to 0000:7C00 (boot sector)
            0x00, 0x7C,      // IP = 7C00
            0x00, 0x00,      // CS = 0000
        ];

        data[entry_point..entry_point + init_code.len()].copy_from_slice(&init_code);

        BiosRom { data }
    }
} 
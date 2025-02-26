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
            0x80, 0xFC, 0x00, // CMP AH, 0x00 (initialize port)
            0x75, 0x0F,       // JNE not_init
            // Handle initialization
            0xB4, 0x00,       // MOV AH, 0 (success)
            0xB0, 0x03,       // MOV AL, 0x03 (port initialized)
            0x5A,             // POP DX
            0x59,             // POP CX
            0x5B,             // POP BX
            0x58,             // POP AX
            0xCF,             // IRET
            // not_init:
            0x80, 0xFC, 0x01, // CMP AH, 0x01 (write character)
            0x75, 0x0F,       // JNE skip_write
            // Handle serial write
            0xBA, 0xF8, 0x03, // MOV DX, 0x3F8 (COM1)
            0xEE,             // OUT DX, AL
            0xB4, 0x00,       // MOV AH, 0 (success)
            0x5A,             // POP DX
            0x59,             // POP CX
            0x5B,             // POP BX
            0x58,             // POP AX
            0xCF,             // IRET
            // skip_write:
            0xB4, 0x01,       // MOV AH, 1 (error - unsupported function)
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
        
        // BIOS entry point at F000:E05B
        let entry_point = 0xE05B;
        
        // Basic initialization sequence
        let init_code = [
            0xFA,             // CLI - Disable interrupts
            0x31, 0xC0,      // XOR AX, AX
            0x8E, 0xD8,      // MOV DS, AX  - Set DS = 0
            0x8E, 0xC0,      // MOV ES, AX  - Set ES = 0
            0x8E, 0xD0,      // MOV SS, AX  - Set SS = 0
            0xBC, 0x00, 0x7C, // MOV SP, 0x7C00 - Set up stack just below boot sector
            0xFB,             // STI - Enable interrupts
            0xEA,             // Far JMP to 0000:7C00 (boot sector)
            0x00, 0x7C,      // IP = 7C00
            0x00, 0x00,      // CS = 0000
        ];

        // Copy initialization code to ROM at the entry point offset
        data[entry_point..entry_point + init_code.len()].copy_from_slice(&init_code);

        // Reset vector at F000:FFF0
        let reset_vector = 0xFFF0;
        data[reset_vector] = 0xEA;     // Far JMP
        data[reset_vector + 1] = 0x5B; // IP = E05B
        data[reset_vector + 2] = 0xE0;
        data[reset_vector + 3] = 0x00; // CS = F000
        data[reset_vector + 4] = 0xF0;

        BiosRom { data }
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        let value = self.data[offset];
        // println!("ROM read: offset={:#05X} value={:#04X}", offset, value);
        value
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
            0x80, 0xFC, 0x00, // CMP AH, 0x00 (initialize port)
            0x75, 0x0F,       // JNE not_init
            // Handle initialization
            0xB4, 0x00,       // MOV AH, 0 (success)
            0xB0, 0x03,       // MOV AL, 0x03 (port initialized)
            0x5A,             // POP DX
            0x59,             // POP CX
            0x5B,             // POP BX
            0x58,             // POP AX
            0xCF,             // IRET
            // not_init:
            0x80, 0xFC, 0x01, // CMP AH, 0x01 (write character)
            0x75, 0x0F,       // JNE skip_write
            // Handle serial write
            0xBA, 0xF8, 0x03, // MOV DX, 0x3F8 (COM1)
            0xEE,             // OUT DX, AL
            0xB4, 0x00,       // MOV AH, 0 (success)
            0x5A,             // POP DX
            0x59,             // POP CX
            0x5B,             // POP BX
            0x58,             // POP AX
            0xCF,             // IRET
            // skip_write:
            0xB4, 0x01,       // MOV AH, 1 (error - unsupported function)
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
        
        // BIOS entry point at F000:E05B
        let entry_point = 0xE05B;
        
        // Basic initialization sequence
        let init_code = [
            0xFA,             // CLI - Disable interrupts
            0x31, 0xC0,      // XOR AX, AX
            0x8E, 0xD8,      // MOV DS, AX  - Set DS = 0
            0x8E, 0xC0,      // MOV ES, AX  - Set ES = 0
            0x8E, 0xD0,      // MOV SS, AX  - Set SS = 0
            0xBC, 0x00, 0x7C, // MOV SP, 0x7C00 - Set up stack just below boot sector
            0xFB,             // STI - Enable interrupts
            0xEA,             // Far JMP to 0000:7C00 (boot sector)
            0x00, 0x7C,      // IP = 7C00
            0x00, 0x00,      // CS = 0000
        ];

        // Copy initialization code to ROM at the entry point offset
        data[entry_point..entry_point + init_code.len()].copy_from_slice(&init_code);

        // Reset vector at F000:FFF0
        let reset_vector = 0xFFF0;
        data[reset_vector] = 0xEA;     // Far JMP
        data[reset_vector + 1] = 0x5B; // IP = E05B
        data[reset_vector + 2] = 0xE0;
        data[reset_vector + 3] = 0x00; // CS = F000
        data[reset_vector + 4] = 0xF0;

        BiosRom { data }
    }
} 
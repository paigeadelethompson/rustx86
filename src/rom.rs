pub struct BiosRom {
    data: Vec<u8>,
    has_valid_code: bool,
}

impl BiosRom {
    pub fn new() -> Self {
        // Initialize with basic BIOS code
        let mut data = vec![0; 0x10000]; // 64KB ROM

        // Add serial interrupt handler at F000:E000
        let serial_handler = [
            0x50, // PUSH AX
            0x53, // PUSH BX
            0x51, // PUSH CX
            0x52, // PUSH DX
            0x80, 0xFC, 0x00, // CMP AH, 0x00 (initialize port)
            0x75, 0x0F, // JNE not_init
            // Handle initialization
            0xB4, 0x00, // MOV AH, 0 (success)
            0xB0, 0x03, // MOV AL, 0x03 (port initialized)
            0x5A, // POP DX
            0x59, // POP CX
            0x5B, // POP BX
            0x58, // POP AX
            0xCF, // IRET
            // not_init:
            0x80, 0xFC, 0x01, // CMP AH, 0x01 (write character)
            0x75, 0x0F, // JNE skip_write
            // Handle serial write
            0xBA, 0xF8, 0x03, // MOV DX, 0x3F8 (COM1)
            0xEE, // OUT DX, AL
            0xB4, 0x00, // MOV AH, 0 (success)
            0x5A, // POP DX
            0x59, // POP CX
            0x5B, // POP BX
            0x58, // POP AX
            0xCF, // IRET
            // skip_write:
            0xB4, 0x01, // MOV AH, 1 (error - unsupported function)
            0x5A, // POP DX
            0x59, // POP CX
            0x5B, // POP BX
            0x58, // POP AX
            0xCF, // IRET
        ];

        // Copy serial handler to ROM
        let serial_offset = 0xE000;
        data[serial_offset..serial_offset + serial_handler.len()].copy_from_slice(&serial_handler);

        // Reset vector at F000:FFF0
        let reset_vector = 0xFFF0;
        data[reset_vector] = 0xEA; // Far JMP
        data[reset_vector + 1] = 0x5B; // IP = E05B
        data[reset_vector + 2] = 0xE0;
        data[reset_vector + 3] = 0x00; // CS = F000
        data[reset_vector + 4] = 0xF0;

        // BIOS entry point at F000:E05B
        let entry_point = 0xE05B;
        data[entry_point] = 0xF4; // HLT - This will halt the CPU when executed

        BiosRom {
            data,
            has_valid_code: true, // Code is valid - we have reset vector and HLT
        }
    }

    pub fn verify_rom_code(&mut self) -> bool {
        if !self.data.is_empty() {
            self.has_valid_code = true;
            true
        } else {
            false
        }
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        if offset < self.data.len() {
            self.data[offset]
        } else {
            0
        }
    }

    #[allow(dead_code)]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn has_valid_code(&self) -> bool {
        self.has_valid_code
    }

    #[allow(dead_code)]
    pub fn set_valid_code(&mut self, valid: bool) {
        self.has_valid_code = valid;
    }

    #[allow(dead_code)]
    pub fn from_data(mut data: Vec<u8>) -> Self {
        // Ensure the data is 64KB
        if data.len() < 0x10000 {
            data.resize(0x10000, 0);
        }

        // Add serial interrupt handler at F000:E000
        let serial_handler = [
            0x50, // PUSH AX
            0x53, // PUSH BX
            0x51, // PUSH CX
            0x52, // PUSH DX
            0x80, 0xFC, 0x00, // CMP AH, 0x00 (initialize port)
            0x75, 0x0F, // JNE not_init
            // Handle initialization
            0xB4, 0x00, // MOV AH, 0 (success)
            0xB0, 0x03, // MOV AL, 0x03 (port initialized)
            0x5A, // POP DX
            0x59, // POP CX
            0x5B, // POP BX
            0x58, // POP AX
            0xCF, // IRET
            // not_init:
            0x80, 0xFC, 0x01, // CMP AH, 0x01 (write character)
            0x75, 0x0F, // JNE skip_write
            // Handle serial write
            0xBA, 0xF8, 0x03, // MOV DX, 0x3F8 (COM1)
            0xEE, // OUT DX, AL
            0xB4, 0x00, // MOV AH, 0 (success)
            0x5A, // POP DX
            0x59, // POP CX
            0x5B, // POP BX
            0x58, // POP AX
            0xCF, // IRET
            // skip_write:
            0xB4, 0x01, // MOV AH, 1 (error - unsupported function)
            0x5A, // POP DX
            0x59, // POP CX
            0x5B, // POP BX
            0x58, // POP AX
            0xCF, // IRET
        ];

        // Copy serial handler to ROM
        let serial_offset = 0xE000;
        data[serial_offset..serial_offset + serial_handler.len()].copy_from_slice(&serial_handler);

        // BIOS entry point at F000:E05B
        let entry_point = 0xE05B;

        // Basic initialization sequence
        let init_code = [
            0xEA, // Far JMP
            0x5B, // IP = E05B
            0xE0, //
            0x00, // CS = F000
            0xF0, //
            0xf4, // HLT
        ];

        // Copy initialization code to ROM at the entry point offset
        data[entry_point..entry_point + init_code.len()].copy_from_slice(&init_code);

        BiosRom {
            data,
            has_valid_code: true, // When loading from data, we assume it's valid
        }
    }
}

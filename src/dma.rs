pub struct DMAChannel {
    current_address: u16,
    current_word_count: u16,
    base_address: u16,
    base_word_count: u16,
    mode: u8,
    page: u8,
    mask: bool,
}

impl DMAChannel {
    fn new() -> Self {
        DMAChannel {
            current_address: 0,
            current_word_count: 0,
            base_address: 0,
            base_word_count: 0,
            mode: 0,
            page: 0,
            mask: true,
        }
    }
}

pub struct DMAController {
    channels: [DMAChannel; 4],
    command: u8,
    status: u8,
    request: u8,
    mask: u8,
}

impl DMAController {
    pub fn new() -> Self {
        DMAController {
            channels: [
                DMAChannel::new(),
                DMAChannel::new(),
                DMAChannel::new(),
                DMAChannel::new(),
            ],
            command: 0,
            status: 0,
            request: 0,
            mask: 0xFF, // All channels masked initially
        }
    }

    pub fn read_port(&self, port: u16) -> u8 {
        match port {
            0x00..=0x07 => self.read_channel_register(port),
            0x08 => self.status,
            0x09 => 0, // Not implemented: Request register read
            0x0A => 0, // Not implemented: Single mask register read
            0x0B => 0, // Not implemented: Mode register read
            0x0C => 0, // Clear byte pointer flip-flop
            0x0D => 0, // Not implemented: Master clear read
            0x0E => 0, // Not implemented: Clear mask register read
            0x0F => 0, // Not implemented: Write all mask register read
            _ => 0,
        }
    }

    pub fn write_port(&mut self, port: u16, value: u8) {
        match port {
            0x00..=0x07 => self.write_channel_register(port, value),
            0x08 => self.command = value,
            0x09 => self.request = value,
            0x0A => self.set_mask_bit(value),
            0x0B => self.set_mode(value),
            0x0C => (), // Clear byte pointer flip-flop
            0x0D => self.master_clear(),
            0x0E => self.clear_mask_register(),
            0x0F => self.mask = value,
            0x80..=0x8F => self.write_page_register(port - 0x80, value),
            _ => (),
        }
    }

    fn read_channel_register(&self, port: u16) -> u8 {
        let channel = (port >> 1) as usize;
        if channel >= 4 {
            return 0;
        }

        match port & 1 {
            0 => (self.channels[channel].current_address & 0xFF) as u8,
            1 => ((self.channels[channel].current_address >> 8) & 0xFF) as u8,
            _ => unreachable!(),
        }
    }

    fn write_channel_register(&mut self, port: u16, value: u8) {
        let channel = (port >> 1) as usize;
        if channel >= 4 {
            return;
        }

        match port & 1 {
            0 => {
                self.channels[channel].base_address = 
                    (self.channels[channel].base_address & 0xFF00) | (value as u16);
                self.channels[channel].current_address = self.channels[channel].base_address;
            }
            1 => {
                self.channels[channel].base_address = 
                    (self.channels[channel].base_address & 0x00FF) | ((value as u16) << 8);
                self.channels[channel].current_address = self.channels[channel].base_address;
            }
            _ => unreachable!(),
        }
    }

    fn set_mask_bit(&mut self, value: u8) {
        let channel = value & 0x03;
        let mask = (value & 0x04) != 0;
        self.channels[channel as usize].mask = mask;
    }

    fn set_mode(&mut self, value: u8) {
        let channel = value & 0x03;
        self.channels[channel as usize].mode = value;
    }

    fn master_clear(&mut self) {
        self.command = 0;
        self.status = 0;
        self.request = 0;
        self.mask = 0xFF;
        for channel in &mut self.channels {
            channel.mask = true;
            channel.mode = 0;
        }
    }

    fn clear_mask_register(&mut self) {
        self.mask = 0;
        for channel in &mut self.channels {
            channel.mask = false;
        }
    }

    fn write_page_register(&mut self, page_reg: u16, value: u8) {
        // Map page register to channel
        let channel = match page_reg {
            0 => Some(0), // Channel 0
            1 => Some(1), // Channel 1
            2 => Some(2), // Channel 2
            3 => Some(3), // Channel 3
            7 => Some(0), // Channel 4 (not implemented)
            _ => None,
        };

        if let Some(ch) = channel {
            self.channels[ch].page = value;
        }
    }

    pub fn transfer(&mut self, channel: usize, memory: &mut [u8], io_buffer: &[u8]) -> bool {
        if channel >= 4 || self.channels[channel].mask {
            return false;
        }

        let ch = &mut self.channels[channel];
        let mode = ch.mode & 0x0C;
        let addr = ((ch.page as u32) << 16) | (ch.current_address as u32);

        match mode {
            0x04 => { // Single transfer mode
                if io_buffer.len() > 0 {
                    memory[addr as usize] = io_buffer[0];
                    ch.current_address = ch.current_address.wrapping_add(1);
                    ch.current_word_count = ch.current_word_count.wrapping_sub(1);
                }
            }
            0x08 => { // Block transfer mode
                let len = io_buffer.len().min(ch.current_word_count as usize);
                for i in 0..len {
                    memory[(addr as usize) + i] = io_buffer[i];
                }
                ch.current_address = ch.current_address.wrapping_add(len as u16);
                ch.current_word_count = ch.current_word_count.wrapping_sub(len as u16);
            }
            _ => return false,
        }

        true
    }
} 
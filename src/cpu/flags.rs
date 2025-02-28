use std::ops::{BitAnd, BitOr, Not};

#[derive(Debug, Clone)]
pub struct Flags {
    carry: bool,     // CF (bit 0)
    parity: bool,    // PF (bit 2)
    adjust: bool,    // AF (bit 4)
    zero: bool,      // ZF (bit 6)
    sign: bool,      // SF (bit 7)
    trap: bool,      // TF (bit 8)
    interrupt: bool, // IF (bit 9)
    direction: bool, // DF (bit 10)
    overflow: bool,  // OF (bit 11)
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            carry: false,
            parity: false,
            adjust: false,
            zero: false,
            sign: false,
            trap: false,
            interrupt: false,
            direction: false,
            overflow: false,
        }
    }

    #[allow(dead_code)]
    pub fn from_word(word: u16) -> Flags {
        Flags {
            carry: (word & 0x0001) != 0,
            parity: (word & 0x0004) != 0,
            adjust: (word & 0x0010) != 0,
            zero: (word & 0x0040) != 0,
            sign: (word & 0x0080) != 0,
            trap: (word & 0x0100) != 0,
            interrupt: (word & 0x0200) != 0,
            direction: (word & 0x0400) != 0,
            overflow: (word & 0x0800) != 0,
        }
    }

    pub fn get_carry(&self) -> bool {
        self.carry
    }
    #[allow(dead_code)]
    pub fn get_parity(&self) -> bool {
        self.parity
    }
    #[allow(dead_code)]
    pub fn get_adjust(&self) -> bool {
        self.adjust
    }
    pub fn get_zero(&self) -> bool {
        self.zero
    }
    #[allow(dead_code)]
    pub fn get_sign(&self) -> bool {
        self.sign
    }
    #[allow(dead_code)]
    pub fn get_trap(&self) -> bool {
        self.trap
    }
    #[allow(dead_code)]
    pub fn get_interrupt(&self) -> bool {
        self.interrupt
    }
    pub fn get_direction(&self) -> bool {
        self.direction
    }
    pub fn get_overflow(&self) -> bool {
        self.overflow
    }

    pub fn set_carry(&mut self, value: bool) {
        self.carry = value;
    }
    pub fn set_parity(&mut self, value: bool) {
        self.parity = value;
    }
    pub fn set_adjust(&mut self, value: bool) {
        self.adjust = value;
    }
    pub fn set_zero(&mut self, value: bool) {
        self.zero = value;
    }
    pub fn set_sign(&mut self, value: bool) {
        self.sign = value;
    }
    pub fn set_trap(&mut self, value: bool) {
        self.trap = value;
    }
    pub fn set_interrupt(&mut self, value: bool) {
        self.interrupt = value;
    }
    pub fn set_direction(&mut self, value: bool) {
        self.direction = value;
    }
    pub fn set_overflow(&mut self, value: bool) {
        self.overflow = value;
    }

    #[allow(dead_code)]
    pub fn as_byte(&self) -> u8 {
        let mut result = 0u8;
        if self.carry {
            result |= 0x01;
        }
        if self.parity {
            result |= 0x04;
        }
        if self.adjust {
            result |= 0x10;
        }
        if self.zero {
            result |= 0x40;
        }
        if self.sign {
            result |= 0x80;
        }
        result
    }

    pub fn set_from_byte(&mut self, value: u8) {
        self.carry = (value & 0x01) != 0;
        self.parity = (value & 0x04) != 0;
        self.adjust = (value & 0x10) != 0;
        self.zero = (value & 0x40) != 0;
        self.sign = (value & 0x80) != 0;
    }

    pub fn as_u16(&self) -> u16 {
        let mut result = 0u16;
        if self.carry {
            result |= 0x0001;
        }
        if self.parity {
            result |= 0x0004;
        }
        if self.adjust {
            result |= 0x0010;
        }
        if self.zero {
            result |= 0x0040;
        }
        if self.sign {
            result |= 0x0080;
        }
        if self.trap {
            result |= 0x0100;
        }
        if self.interrupt {
            result |= 0x0200;
        }
        if self.direction {
            result |= 0x0400;
        }
        if self.overflow {
            result |= 0x0800;
        }
        result
    }

    pub fn set_from_u16(&mut self, value: u16) {
        self.carry = (value & 0x0001) != 0;
        self.parity = (value & 0x0004) != 0;
        self.adjust = (value & 0x0010) != 0;
        self.zero = (value & 0x0040) != 0;
        self.sign = (value & 0x0080) != 0;
        self.trap = (value & 0x0100) != 0;
        self.interrupt = (value & 0x0200) != 0;
        self.direction = (value & 0x0400) != 0;
        self.overflow = (value & 0x0800) != 0;
    }

    #[allow(dead_code)]
    pub fn update_flags_dec16(&mut self, result: u16) {
        // Zero flag
        self.set_zero(result == 0);

        // Sign flag
        self.set_sign((result & 0x8000) != 0);

        // Overflow flag (set if decrementing 0x8000)
        self.set_overflow(result == 0x7FFF);

        // Auxiliary flag (set if decrementing a value with low nibble of 0)
        self.set_adjust((result & 0x0F) == 0x0F);
    }

    #[allow(dead_code)]
    pub fn update_logical_flags(&mut self, result: u16) {
        self.set_zero(result == 0);
        self.set_sign((result & 0x8000) != 0);
        self.set_carry(false);
        self.set_overflow(false);
        // Parity is set if the number of 1 bits in the least significant byte is even
        let lsb = result as u8;
        let mut count = 0;
        for i in 0..8 {
            if (lsb & (1 << i)) != 0 {
                count += 1;
            }
        }
        self.set_parity(count % 2 == 0);
    }

    pub fn as_word(&self) -> u16 {
        let mut word = 0u16;
        if self.carry {
            word |= 0x0001;
        }
        if self.parity {
            word |= 0x0004;
        }
        if self.adjust {
            word |= 0x0010;
        }
        if self.zero {
            word |= 0x0040;
        }
        if self.sign {
            word |= 0x0080;
        }
        if self.trap {
            word |= 0x0100;
        }
        if self.interrupt {
            word |= 0x0200;
        }
        if self.direction {
            word |= 0x0400;
        }
        if self.overflow {
            word |= 0x0800;
        }
        word
    }

    pub fn set_from_word(&mut self, word: u16) {
        self.carry = (word & 0x0001) != 0;
        self.parity = (word & 0x0004) != 0;
        self.adjust = (word & 0x0010) != 0;
        self.zero = (word & 0x0040) != 0;
        self.sign = (word & 0x0080) != 0;
        self.trap = (word & 0x0100) != 0;
        self.interrupt = (word & 0x0200) != 0;
        self.direction = (word & 0x0400) != 0;
        self.overflow = (word & 0x0800) != 0;
    }

    #[allow(dead_code)]
    pub fn zero_flag(&self) -> bool {
        self.zero
    }

    #[allow(dead_code)]
    pub fn carry_flag(&self) -> bool {
        self.carry
    }

    #[allow(dead_code)]
    pub fn overflow_flag(&self) -> bool {
        self.overflow
    }

    #[allow(dead_code)]
    pub fn set_word(&mut self, word: u16) {
        self.set_from_word(word);
    }

    #[allow(dead_code)]
    pub fn set_interrupt_flag(&mut self, value: bool) {
        self.set_interrupt(value);
    }

    #[allow(dead_code)]
    pub fn set_trap_flag(&mut self, value: bool) {
        self.set_trap(value);
    }

    #[allow(dead_code)]
    pub fn update_zero_and_sign_flags_16(&mut self, value: u16) {
        self.set_zero(value == 0);
        self.set_sign((value & 0x8000) != 0);
    }

    pub fn update_flags_add16(&mut self, a: u16, b: u16, result: u16) {
        self.set_carry(result < a);
        self.set_zero(result == 0);
        self.set_sign((result & 0x8000) != 0);
        self.set_overflow(((a ^ b) & 0x8000) == 0 && ((a ^ result) & 0x8000) != 0);
        self.set_parity(result.count_ones() % 2 == 0);
    }

    pub fn update_flags_sub16(&mut self, a: u16, b: u16, result: u16) {
        self.set_carry(a < b);
        self.set_zero(result == 0);
        self.set_sign((result & 0x8000) != 0);
        self.set_overflow(((a ^ b) & 0x8000) != 0 && ((a ^ result) & 0x8000) != 0);
        self.set_parity(result.count_ones() % 2 == 0);
    }
}

impl BitOr<u16> for Flags {
    type Output = Flags;
    fn bitor(self, rhs: u16) -> Flags {
        let mut result = self;
        result.set_from_u16(result.as_u16() | rhs);
        result
    }
}

impl BitAnd<u16> for Flags {
    type Output = Flags;
    fn bitand(self, rhs: u16) -> Flags {
        let mut result = self;
        result.set_from_u16(result.as_u16() & rhs);
        result
    }
}

impl Not for Flags {
    type Output = Flags;
    fn not(self) -> Flags {
        let mut result = self;
        result.set_from_u16(!result.as_u16());
        result
    }
}

use std::ops::{BitOr, BitAnd, Not};

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub value: u16,
}

impl Flags {
    pub fn new() -> Self {
        Flags { value: 0 }
    }

    pub fn get_carry(&self) -> bool {
        (self.value & 0x0001) != 0
    }

    pub fn set_carry(&mut self, value: bool) {
        if value {
            self.value |= 0x0001;
        } else {
            self.value &= !0x0001;
        }
    }

    pub fn get_parity(&self) -> bool {
        (self.value & 0x0004) != 0
    }

    pub fn set_parity(&mut self, value: bool) {
        if value {
            self.value |= 0x0004;
        } else {
            self.value &= !0x0004;
        }
    }

    pub fn get_auxiliary(&self) -> bool {
        (self.value & 0x0010) != 0
    }

    pub fn set_auxiliary(&mut self, value: bool) {
        if value {
            self.value |= 0x0010;
        } else {
            self.value &= !0x0010;
        }
    }

    pub fn get_zero(&self) -> bool {
        (self.value & 0x0040) != 0
    }

    pub fn set_zero(&mut self, value: bool) {
        if value {
            self.value |= 0x0040;
        } else {
            self.value &= !0x0040;
        }
    }

    pub fn get_sign(&self) -> bool {
        (self.value & 0x0080) != 0
    }

    pub fn set_sign(&mut self, value: bool) {
        if value {
            self.value |= 0x0080;
        } else {
            self.value &= !0x0080;
        }
    }

    pub fn get_trap(&self) -> bool {
        (self.value & 0x0100) != 0
    }

    pub fn set_trap(&mut self, value: bool) {
        if value {
            self.value |= 0x0100;
        } else {
            self.value &= !0x0100;
        }
    }

    pub fn get_interrupt(&self) -> bool {
        (self.value & 0x0200) != 0
    }

    pub fn set_interrupt(&mut self, value: bool) {
        if value {
            self.value |= 0x0200;
        } else {
            self.value &= !0x0200;
        }
    }

    pub fn get_direction(&self) -> bool {
        (self.value & 0x0400) != 0
    }

    pub fn set_direction(&mut self, value: bool) {
        if value {
            self.value |= 0x0400;
        } else {
            self.value &= !0x0400;
        }
    }

    pub fn get_overflow(&self) -> bool {
        (self.value & 0x0800) != 0
    }

    pub fn set_overflow(&mut self, value: bool) {
        if value {
            self.value |= 0x0800;
        } else {
            self.value &= !0x0800;
        }
    }

    pub fn update_flags_dec16(&mut self, result: u16) {
        // Zero flag
        self.set_zero(result == 0);
        
        // Sign flag
        self.set_sign((result & 0x8000) != 0);
        
        // Overflow flag (set if decrementing 0x8000)
        self.set_overflow(result == 0x7FFF);
        
        // Auxiliary flag (set if decrementing a value with low nibble of 0)
        self.set_auxiliary((result & 0x0F) == 0x0F);
    }
}

impl BitOr<u16> for Flags {
    type Output = Flags;

    fn bitor(self, rhs: u16) -> Self::Output {
        Flags { value: self.value | rhs }
    }
}

impl BitAnd<u16> for Flags {
    type Output = Flags;

    fn bitand(self, rhs: u16) -> Self::Output {
        Flags { value: self.value & rhs }
    }
}

impl Not for Flags {
    type Output = Flags;

    fn not(self) -> Self::Output {
        Flags { value: !self.value }
    }
} 
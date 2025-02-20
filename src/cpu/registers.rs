use super::flags::Flags;

#[derive(Debug, Clone)]
pub struct Registers {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub si: u16,
    pub di: u16,
    pub bp: u16,
    pub sp: u16,
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub ip: u16,
    pub flags: Flags,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            si: 0,
            di: 0,
            bp: 0,
            sp: 0,
            cs: 0xF000,
            ds: 0,
            es: 0,
            ss: 0,
            ip: 0xFFF0,
            flags: Flags::new(),
        }
    }

    pub fn get_bx(&self) -> u16 {
        self.bx
    }

    pub fn get_si(&self) -> u16 {
        self.si
    }

    pub fn get_di(&self) -> u16 {
        self.di
    }

    pub fn get_bp(&self) -> u16 {
        self.bp
    }

    pub fn get_al(&self) -> u8 {
        (self.ax & 0xFF) as u8
    }

    pub fn get_ah(&self) -> u8 {
        ((self.ax >> 8) & 0xFF) as u8
    }

    pub fn set_al(&mut self, value: u8) {
        self.ax = (self.ax & 0xFF00) | (value as u16);
        println!("set_al: Setting AL to 0x{:02X}, AX is now 0x{:04X}", value, self.ax);
    }

    pub fn set_ah(&mut self, value: u8) {
        self.ax = (self.ax & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_cl(&self) -> u8 {
        (self.cx & 0xFF) as u8
    }

    pub fn get_ch(&self) -> u8 {
        ((self.cx >> 8) & 0xFF) as u8
    }

    pub fn set_cl(&mut self, value: u8) {
        self.cx = (self.cx & 0xFF00) | (value as u16);
    }

    pub fn set_ch(&mut self, value: u8) {
        self.cx = (self.cx & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_dl(&self) -> u8 {
        (self.dx & 0xFF) as u8
    }

    pub fn get_dh(&self) -> u8 {
        ((self.dx >> 8) & 0xFF) as u8
    }

    pub fn set_dl(&mut self, value: u8) {
        self.dx = (self.dx & 0xFF00) | (value as u16);
    }

    pub fn set_dh(&mut self, value: u8) {
        self.dx = (self.dx & 0x00FF) | ((value as u16) << 8);
    }

    pub fn set_cx(&mut self, value: u16) {
        self.cx = value;
    }

    pub fn set_dx(&mut self, value: u16) {
        self.dx = value;
    }

    pub fn get_reg8(&self, reg: u8) -> u8 {
        match reg {
            0 => (self.ax & 0xFF) as u8,         // al
            1 => (self.cx & 0xFF) as u8,         // cl
            2 => (self.dx & 0xFF) as u8,         // dl
            3 => (self.bx & 0xFF) as u8,         // bl
            4 => (self.ax >> 8) as u8,           // ah
            5 => (self.cx >> 8) as u8,           // ch
            6 => (self.dx >> 8) as u8,           // dh
            7 => (self.bx >> 8) as u8,           // bh
            _ => panic!("Invalid register index"),
        }
    }

    pub fn set_reg8(&mut self, reg: u8, value: u8) {
        match reg {
            0 => self.ax = (self.ax & 0xFF00) | (value as u16),  // al
            1 => self.cx = (self.cx & 0xFF00) | (value as u16),  // cl
            2 => self.dx = (self.dx & 0xFF00) | (value as u16),  // dl
            3 => self.bx = (self.bx & 0xFF00) | (value as u16),  // bl
            4 => self.ax = (self.ax & 0x00FF) | ((value as u16) << 8),  // ah
            5 => self.cx = (self.cx & 0x00FF) | ((value as u16) << 8),  // ch
            6 => self.dx = (self.dx & 0x00FF) | ((value as u16) << 8),  // dh
            7 => self.bx = (self.bx & 0x00FF) | ((value as u16) << 8),  // bh
            _ => panic!("Invalid register index"),
        }
    }

    pub fn get_reg8_low(&self, reg: u8) -> u8 {
        self.get_reg8(reg & 0x3)  // Bottom 2 bits select AL, CL, DL, BL
    }

    pub fn set_reg8_low(&mut self, reg: u8, value: u8) {
        self.set_reg8(reg & 0x3, value)  // Bottom 2 bits select AL, CL, DL, BL
    }

    pub fn get_reg8_high(&self, reg: u8) -> u8 {
        self.get_reg8((reg & 0x3) | 0x4)  // Bottom 2 bits select AH, CH, DH, BH
    }

    pub fn set_reg8_high(&mut self, reg: u8, value: u8) {
        self.set_reg8((reg & 0x3) | 0x4, value)  // Bottom 2 bits select AH, CH, DH, BH
    }

    pub fn get_dx(&self) -> u16 {
        self.dx
    }

    pub fn get_reg16(&self, reg: u8) -> u16 {
        match reg {
            0 => self.ax,
            1 => self.cx,
            2 => self.dx,
            3 => self.bx,
            4 => self.sp,
            5 => self.bp,
            6 => self.si,
            7 => self.di,
            _ => panic!("Invalid register index"),
        }
    }
} 
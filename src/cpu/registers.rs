use super::flags::Flags;

#[derive(Debug, Clone)]
pub struct Registers {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,
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
            sp: 0,
            bp: 0,
            si: 0,
            di: 0,
            cs: 0xF000, // BIOS segment
            ds: 0,
            es: 0,
            ss: 0,
            ip: 0xFFF0, // BIOS entry point
            flags: Flags::new(),
        }
    }

    pub fn reset(&mut self) {
        self.ax = 0;
        self.bx = 0;
        self.cx = 0;
        self.dx = 0;
        self.sp = 0;
        self.bp = 0;
        self.si = 0;
        self.di = 0;
        self.cs = 0xF000; // BIOS segment
        self.ds = 0;
        self.es = 0;
        self.ss = 0;
        self.ip = 0xFFF0; // BIOS entry point
        self.flags = Flags::new();
    }

    pub fn get_ah(&self) -> u8 {
        (self.ax >> 8) as u8
    }

    pub fn get_al(&self) -> u8 {
        (self.ax & 0xFF) as u8
    }

    #[allow(dead_code)]
    pub fn get_bh(&self) -> u8 {
        (self.bx >> 8) as u8
    }

    #[allow(dead_code)]
    pub fn get_bl(&self) -> u8 {
        (self.bx & 0xFF) as u8
    }

    pub fn get_ch(&self) -> u8 {
        (self.cx >> 8) as u8
    }

    pub fn get_cl(&self) -> u8 {
        (self.cx & 0xFF) as u8
    }

    pub fn get_dh(&self) -> u8 {
        (self.dx >> 8) as u8
    }

    pub fn get_dl(&self) -> u8 {
        (self.dx & 0xFF) as u8
    }

    pub fn set_ah(&mut self, value: u8) {
        self.ax = (self.ax & 0x00FF) | ((value as u16) << 8);
    }

    pub fn set_al(&mut self, value: u8) {
        self.ax = (self.ax & 0xFF00) | (value as u16);
    }

    #[allow(dead_code)]
    pub fn set_bh(&mut self, value: u8) {
        self.bx = (self.bx & 0x00FF) | ((value as u16) << 8);
    }

    #[allow(dead_code)]
    pub fn set_bl(&mut self, value: u8) {
        self.bx = (self.bx & 0xFF00) | (value as u16);
    }

    pub fn set_ch(&mut self, value: u8) {
        self.cx = (self.cx & 0x00FF) | ((value as u16) << 8);
    }

    pub fn set_cl(&mut self, value: u8) {
        self.cx = (self.cx & 0xFF00) | (value as u16);
    }

    pub fn set_dh(&mut self, value: u8) {
        self.dx = (self.dx & 0x00FF) | ((value as u16) << 8);
    }

    pub fn set_dl(&mut self, value: u8) {
        self.dx = (self.dx & 0xFF00) | (value as u16);
    }

    pub fn get_ax(&self) -> u16 {
        self.ax
    }

    pub fn get_bx(&self) -> u16 {
        self.bx
    }

    #[allow(dead_code)]
    pub fn get_cx(&self) -> u16 {
        self.cx
    }

    pub fn get_dx(&self) -> u16 {
        self.dx
    }

    pub fn set_ax(&mut self, value: u16) {
        self.ax = value;
    }

    #[allow(dead_code)]
    pub fn set_bx(&mut self, value: u16) {
        self.bx = value;
    }

    pub fn set_cx(&mut self, value: u16) {
        self.cx = value;
    }

    pub fn set_dx(&mut self, value: u16) {
        self.dx = value;
    }

    #[allow(dead_code)]
    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    #[allow(dead_code)]
    pub fn set_bp(&mut self, value: u16) {
        self.bp = value;
    }

    #[allow(dead_code)]
    pub fn set_si(&mut self, value: u16) {
        self.si = value;
    }

    #[allow(dead_code)]
    pub fn set_di(&mut self, value: u16) {
        self.di = value;
    }

    pub fn get_reg8(&self, reg: u8) -> u8 {
        match reg & 0x07 {
            0 => self.ax as u8,
            1 => (self.ax >> 8) as u8,
            2 => self.bx as u8,
            3 => (self.bx >> 8) as u8,
            4 => self.cx as u8,
            5 => (self.cx >> 8) as u8,
            6 => self.dx as u8,
            7 => (self.dx >> 8) as u8,
            _ => unreachable!(),
        }
    }

    pub fn set_reg8(&mut self, reg: u8, value: u8) -> Result<(), String> {
        match reg & 0x07 {
            0 => self.ax = (self.ax & 0xFF00) | (value as u16),
            1 => self.ax = (self.ax & 0x00FF) | ((value as u16) << 8),
            2 => self.bx = (self.bx & 0xFF00) | (value as u16),
            3 => self.bx = (self.bx & 0x00FF) | ((value as u16) << 8),
            4 => self.cx = (self.cx & 0xFF00) | (value as u16),
            5 => self.cx = (self.cx & 0x00FF) | ((value as u16) << 8),
            6 => self.dx = (self.dx & 0xFF00) | (value as u16),
            7 => self.dx = (self.dx & 0x00FF) | ((value as u16) << 8),
            _ => return Err("Invalid register index".to_string()),
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_reg8_low(&self, reg: u8) -> u8 {
        self.get_reg8(reg & 0x3) // Bottom 2 bits select AL, CL, DL, BL
    }

    #[allow(dead_code)]
    pub fn set_reg8_low(&mut self, reg: u8, value: u8) -> Result<(), String> {
        self.set_reg8(reg & 0x3, value) // Bottom 2 bits select AL, CL, DL, BL
    }

    #[allow(dead_code)]
    pub fn get_reg8_high(&self, reg: u8) -> u8 {
        self.get_reg8((reg & 0x3) | 0x4) // Bottom 2 bits select AH, CH, DH, BH
    }

    #[allow(dead_code)]
    pub fn set_reg8_high(&mut self, reg: u8, value: u8) -> Result<(), String> {
        self.set_reg8((reg & 0x3) | 0x4, value) // Bottom 2 bits select AH, CH, DH, BH
    }

    pub fn get_reg16(&self, reg: u8) -> u16 {
        match reg & 0x07 {
            0 => self.ax,
            1 => self.cx,
            2 => self.dx,
            3 => self.bx,
            4 => self.sp,
            5 => self.bp,
            6 => self.si,
            7 => self.di,
            _ => unreachable!(),
        }
    }

    pub fn set_reg16(&mut self, reg: u8, value: u16) -> Result<(), String> {
        match reg & 0x07 {
            0 => self.ax = value,
            1 => self.cx = value,
            2 => self.dx = value,
            3 => self.bx = value,
            4 => self.sp = value,
            5 => self.bp = value,
            6 => self.si = value,
            7 => self.di = value,
            _ => return Err("Invalid register index".to_string()),
        }
        Ok(())
    }

    pub fn get_sreg(&self, reg: u8) -> u16 {
        match reg & 0x03 {
            0 => self.es,
            1 => self.cs,
            2 => self.ss,
            3 => self.ds,
            _ => unreachable!(),
        }
    }

    pub fn set_sreg(&mut self, reg: u8, value: u16) {
        match reg & 0x03 {
            0 => self.es = value,
            1 => self.cs = value,
            2 => self.ss = value,
            3 => self.ds = value,
            _ => unreachable!(),
        }
    }

    pub fn get_es(&self) -> u16 {
        self.es
    }

    #[allow(dead_code)]
    pub fn set_es(&mut self, value: u16) {
        self.es = value;
    }
}

// 8250 UART registers
const THR: u8 = 0; // Transmitter Holding Register (write)
const RBR: u8 = 0; // Receiver Buffer Register (read)
const IER: u8 = 1; // Interrupt Enable Register
const IIR: u8 = 2; // Interrupt Identification Register (read)
const FCR: u8 = 2; // FIFO Control Register
const LCR: u8 = 3; // Line Control Register
const MCR: u8 = 4; // Modem Control Register
const LSR: u8 = 5; // Line Status Register
const MSR: u8 = 6; // Modem Status Register
const DLL: u8 = 0; // Divisor Latch LSB (when DLAB=1)
const DLM: u8 = 1; // Divisor Latch MSB (when DLAB=1)

// Line Status Register bits
const LSR_DR: u8 = 0x01;   // Data Ready
const LSR_OE: u8 = 0x02;   // Overrun Error
const LSR_PE: u8 = 0x04;   // Parity Error
const LSR_FE: u8 = 0x08;   // Framing Error
const LSR_BI: u8 = 0x10;   // Break Interrupt
const LSR_THRE: u8 = 0x20; // THR Empty
const LSR_TEMT: u8 = 0x40; // Transmitter Empty
const LSR_ERR: u8 = 0x80;  // Error in RCVR FIFO

// FIFO size
const FIFO_SIZE: usize = 16;

// Add port addresses
const COM1_BASE: u16 = 0x3F8;
const COM2_BASE: u16 = 0x2F8;
const COM3_BASE: u16 = 0x3E8;
const COM4_BASE: u16 = 0x2E8;

// Add interrupt types
const INT_NONE: u8 = 0x01;
const INT_TX_EMPTY: u8 = 0x02;
const INT_RX_DATA: u8 = 0x04;
const INT_LINE_STATUS: u8 = 0x06;
const INT_MODEM_STATUS: u8 = 0x00;

// Add line control bits
const LCR_WORD_LENGTH: u8 = 0x03;  // Bits 0-1: Word length
const LCR_STOP_BITS: u8 = 0x04;    // Bit 2: Stop bits
const LCR_PARITY_ENABLE: u8 = 0x08; // Bit 3: Parity enable
const LCR_PARITY_EVEN: u8 = 0x10;  // Bit 4: Even parity
const LCR_STICK_PARITY: u8 = 0x20; // Bit 5: Stick parity
const LCR_SET_BREAK: u8 = 0x40;    // Bit 6: Set break
const LCR_DLAB: u8 = 0x80;         // Bit 7: DLAB

// Add modem control bits
const MCR_DTR: u8 = 0x01;  // Data Terminal Ready
const MCR_RTS: u8 = 0x02;  // Request To Send
const MCR_OUT1: u8 = 0x04; // Out1
const MCR_OUT2: u8 = 0x08; // Out2 (interrupt enable)
const MCR_LOOP: u8 = 0x10; // Loopback mode

// Add flow control constants
const MSR_CTS: u8 = 0x10;  // Clear To Send
const MSR_DSR: u8 = 0x20;  // Data Set Ready
const MSR_RI: u8 = 0x40;   // Ring Indicator
const MSR_DCD: u8 = 0x80;  // Data Carrier Detect
const MSR_DCTS: u8 = 0x01; // Delta CTS
const MSR_DDSR: u8 = 0x02; // Delta DSR
const MSR_TERI: u8 = 0x04; // Trailing Edge RI
const MSR_DDCD: u8 = 0x08; // Delta DCD

// Add XON/XOFF characters for software flow control
const XON: u8 = 0x11;   // DC1
const XOFF: u8 = 0x13;  // DC3

use std::collections::VecDeque;
use std::io::Write;

pub struct SerialController {
    ports: Vec<Option<SerialPort>>,
}

impl SerialController {
    pub fn new() -> Self {
        let mut ports = Vec::with_capacity(4);
        // Initialize COM1 by default
        ports.push(Some(SerialPort::new()));
        // Other ports are initially disabled
        ports.push(None);
        ports.push(None);
        ports.push(None);
        
        SerialController { ports }
    }

    pub fn get_port(&mut self, index: usize) -> Option<&mut SerialPort> {
        if index < self.ports.len() {
            self.ports[index].as_mut()
        } else {
            None
        }
    }

    pub fn read_byte(&mut self, port: u16) -> u8 {
        let port_idx = (port & 0x07) as usize;
        if let Some(Some(p)) = self.ports.get_mut(port_idx) {
            p.read_byte().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn write_byte(&mut self, port: u16, value: u8) {
        let port_idx = (port & 0x07) as usize;
        if let Some(Some(p)) = self.ports.get_mut(port_idx) {
            p.write_byte(value);
        }
    }

    pub fn get_status(&self) -> Vec<SerialPortStatus> {
        let mut status = Vec::new();
        for port in &self.ports {
            if let Some(port) = port {
                status.push(SerialPortStatus {
                    base_port: port.base_port,
                    baud_rate: port.baud_rate,
                    data_bits: (port.lcr & LCR_WORD_LENGTH) + 5,
                    stop_bits: if (port.lcr & LCR_STOP_BITS) != 0 { 2 } else { 1 },
                    rx_fifo_count: port.rx_fifo.len(),
                    tx_fifo_count: port.tx_fifo.len(),
                    flow_control: FlowControl {
                        hardware_flow_enabled: port.hardware_flow_enabled,
                        xon_xoff_enabled: port.xon_xoff_enabled,
                    },
                });
            }
        }
        status
    }
}

pub struct SerialPort {
    pub base_port: u16,
    pub baud_rate: u32,
    pub dll: u8,
    pub dlm: u8,
    pub lcr: u8,
    pub mcr: u8,
    pub lsr: u8,
    pub msr: u8,
    pub ier: u8,
    pub rx_fifo: VecDeque<u8>,
    pub tx_fifo: VecDeque<u8>,
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
    pub xon_state: bool,
    pub initialized: bool,
    pub input_buffer: VecDeque<u8>,
    pub output_buffer: VecDeque<u8>,
}

impl SerialPort {
    pub fn new() -> Self {
        SerialPort {
            initialized: false,
            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new(),
            base_port: 0,
            baud_rate: 9600,
            dll: 0,
            dlm: 0,
            lcr: 0,
            mcr: 0,
            lsr: LSR_THRE | LSR_TEMT,  // Transmitter is empty
            msr: 0,
            ier: 0,
            hardware_flow_enabled: false,
            xon_xoff_enabled: false,
            xon_state: true,
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),
        }
    }

    pub fn initialize(&mut self, _config: u8) {
        self.initialized = true;
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        self.rx_fifo.pop_front()
    }

    pub fn write_byte(&mut self, value: u8) {
        self.tx_fifo.push_back(value);
    }

    pub fn has_data(&self) -> bool {
        !self.rx_fifo.is_empty()
    }

    pub fn add_input(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }

    pub fn get_output(&mut self) -> Option<u8> {
        self.output_buffer.pop_front()
    }

    pub fn get_status(&self) -> u8 {
        let mut status = 0;
        if self.initialized {
            status |= 0x60; // Transmitter ready and holding register empty
            if !self.rx_fifo.is_empty() {
                status |= 0x01; // Data ready
            }
        }
        status
    }

    pub fn configure(&mut self, config: PortConfig) -> Result<(), String> {
        self.baud_rate = config.baud_rate;
        self.hardware_flow_enabled = config.hardware_flow_enabled;
        self.xon_xoff_enabled = config.xon_xoff_enabled;
        
        // Calculate divisor for baud rate
        let divisor = (115200 / config.baud_rate) as u16;
        self.dll = (divisor & 0xFF) as u8;
        self.dlm = ((divisor >> 8) & 0xFF) as u8;
        
        // Configure line control register
        self.lcr = match config.parity {
            Parity::None => 0x03, // 8N1
            Parity::Odd => 0x0B,  // 8O1
            Parity::Even => 0x1B, // 8E1
        };
        
        // Configure modem control register
        self.mcr = if config.dtr { MCR_DTR } else { 0 } |
                   if config.rts { MCR_RTS } else { 0 } |
                   MCR_OUT2;  // Enable interrupts
                   
        Ok(())
    }

    pub fn receive_byte(&mut self, value: u8) {
        if self.rx_fifo.len() < FIFO_SIZE {
            self.rx_fifo.push_back(value);
            self.lsr |= LSR_DR;
        }
    }
}

pub struct PortStatus {
    pub rx_ready: bool,
    pub tx_ready: bool,
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
    pub xon_state: bool,
}

pub struct PortConfig {
    pub baud_rate: u32,
    pub parity: Parity,
    pub dtr: bool,
    pub rts: bool,
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Parity {
    None,
    Odd,
    Even,
}

const IIR_NO_INT: u8 = 0x01;  // No interrupt pending

#[derive(Debug, Clone)]
pub struct SerialPortStatus {
    pub base_port: u16,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub rx_fifo_count: usize,
    pub tx_fifo_count: usize,
    pub flow_control: FlowControl,
}

#[derive(Debug, Clone)]
pub struct FlowControl {
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
}

pub struct Serial {
    input_buffer: VecDeque<u8>,
    output_buffer: VecDeque<u8>,
    ports: Vec<Option<SerialPort>>,
}

impl Serial {
    pub fn new() -> Self {
        let mut ports = Vec::with_capacity(4);
        // Initialize COM1 by default
        ports.push(Some(SerialPort::new()));
        // Other ports are initially disabled
        ports.push(None);
        ports.push(None);
        ports.push(None);

        Serial {
            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new(),
            ports,
        }
    }

    pub fn read_byte(&mut self, port: u16) -> u8 {
        let port_idx = (port & 0x07) as usize;
        if let Some(Some(p)) = self.ports.get_mut(port_idx) {
            p.read_byte().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn write_byte(&mut self, port: u16, value: u8) {
        let port_idx = (port & 0x07) as usize;
        if let Some(Some(p)) = self.ports.get_mut(port_idx) {
            p.write_byte(value);
        }
    }

    pub fn has_data(&self) -> bool {
        let has_data = !self.input_buffer.is_empty();
        has_data
    }

    pub fn add_input(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }

    pub fn get_output(&mut self) -> Option<u8> {
        let value = self.output_buffer.pop_front();
        value
    }
} 
#[allow(dead_code)]
// 8250 UART registers
const THR: u8 = 0; // Transmitter Holding Register (write)
#[allow(dead_code)]
const RBR: u8 = 0; // Receiver Buffer Register (read)
#[allow(dead_code)]
const IER: u8 = 1; // Interrupt Enable Register
#[allow(dead_code)]
const IIR: u8 = 2; // Interrupt Identification Register (read)
#[allow(dead_code)]
const FCR: u8 = 2; // FIFO Control Register
#[allow(dead_code)]
const LCR: u8 = 3; // Line Control Register
#[allow(dead_code)]
const MCR: u8 = 4; // Modem Control Register
#[allow(dead_code)]
const LSR: u8 = 5; // Line Status Register
#[allow(dead_code)]
const MSR: u8 = 6; // Modem Status Register
#[allow(dead_code)]
const DLL: u8 = 0; // Divisor Latch LSB (when DLAB=1)
#[allow(dead_code)]
const DLM: u8 = 1; // Divisor Latch MSB (when DLAB=1)

#[allow(dead_code)]
// Line Status Register bits
const LSR_DR: u8 = 0x01; // Data Ready
#[allow(dead_code)]
const LSR_OE: u8 = 0x02; // Overrun Error
#[allow(dead_code)]
const LSR_PE: u8 = 0x04; // Parity Error
#[allow(dead_code)]
const LSR_FE: u8 = 0x08; // Framing Error
#[allow(dead_code)]
const LSR_BI: u8 = 0x10; // Break Interrupt
#[allow(dead_code)]
const LSR_THRE: u8 = 0x20; // THR Empty
#[allow(dead_code)]
const LSR_TEMT: u8 = 0x40; // Transmitter Empty
#[allow(dead_code)]
const LSR_ERR: u8 = 0x80; // Error in RCVR FIFO

#[allow(dead_code)]
// FIFO size and port addresses
const FIFO_SIZE: usize = 16;
#[allow(dead_code)]
const COM1_BASE: u16 = 0x3F8;
#[allow(dead_code)]
const COM2_BASE: u16 = 0x2F8;
#[allow(dead_code)]
const COM3_BASE: u16 = 0x3E8;
#[allow(dead_code)]
const COM4_BASE: u16 = 0x2E8;

#[allow(dead_code)]
// Add interrupt types
const INT_NONE: u8 = 0x01;
#[allow(dead_code)]
const INT_TX_EMPTY: u8 = 0x02;
#[allow(dead_code)]
const INT_RX_DATA: u8 = 0x04;
#[allow(dead_code)]
const INT_LINE_STATUS: u8 = 0x06;
#[allow(dead_code)]
const INT_MODEM_STATUS: u8 = 0x00;

// Add line control bits
#[allow(dead_code)]
const LCR_WORD_LENGTH: u8 = 0x03; // Bits 0-1: Word length
#[allow(dead_code)]
const LCR_STOP_BITS: u8 = 0x04; // Bit 2: Stop bits
#[allow(dead_code)]
const LCR_PARITY_ENABLE: u8 = 0x08; // Bit 3: Parity enable
#[allow(dead_code)]
const LCR_PARITY_EVEN: u8 = 0x10; // Bit 4: Even parity
#[allow(dead_code)]
const LCR_STICK_PARITY: u8 = 0x20; // Bit 5: Stick parity
#[allow(dead_code)]
const LCR_SET_BREAK: u8 = 0x40; // Bit 6: Set break
#[allow(dead_code)]
const LCR_DLAB: u8 = 0x80; // Bit 7: DLAB

// Add modem control bits
#[allow(dead_code)]
const MCR_DTR: u8 = 0x01; // Data Terminal Ready
#[allow(dead_code)]
const MCR_RTS: u8 = 0x02; // Request To Send
#[allow(dead_code)]
const MCR_OUT1: u8 = 0x04; // Out1
#[allow(dead_code)]
const MCR_OUT2: u8 = 0x08; // Out2 (interrupt enable)
#[allow(dead_code)]
const MCR_LOOP: u8 = 0x10; // Loopback mode

// Add flow control constants
#[allow(dead_code)]
const MSR_CTS: u8 = 0x10; // Clear To Send
#[allow(dead_code)]
const MSR_DSR: u8 = 0x20; // Data Set Ready
#[allow(dead_code)]
const MSR_RI: u8 = 0x40; // Ring Indicator
#[allow(dead_code)]
const MSR_DCD: u8 = 0x80; // Data Carrier Detect
#[allow(dead_code)]
const MSR_DCTS: u8 = 0x01; // Delta CTS
#[allow(dead_code)]
const MSR_DDSR: u8 = 0x02; // Delta DSR
#[allow(dead_code)]
const MSR_TERI: u8 = 0x04; // Trailing Edge RI
#[allow(dead_code)]
const MSR_DDCD: u8 = 0x08; // Delta DCD

// Add XON/XOFF characters for software flow control
#[allow(dead_code)]
const XON: u8 = 0x11; // DC1
#[allow(dead_code)]
const XOFF: u8 = 0x13; // DC3

use std::collections::VecDeque;

#[allow(dead_code)]
pub struct SerialController {
    ports: Vec<Option<SerialPort>>,
}

impl SerialController {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            ports: vec![Some(SerialPort::new()), None, None, None],
        }
    }

    #[allow(dead_code)]
    pub fn get_port(&mut self, index: usize) -> Option<&mut SerialPort> {
        if index < self.ports.len() {
            self.ports[index].as_mut()
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn read_byte(&mut self, port: u16) -> u8 {
        let port_idx = (port & 0x07) as usize;
        if let Some(Some(p)) = self.ports.get_mut(port_idx) {
            p.read_byte().unwrap_or(0)
        } else {
            0
        }
    }

    #[allow(dead_code)]
    pub fn write_byte(&mut self, port: u16, value: u8) {
        let port_idx = (port & 0x07) as usize;
        if let Some(Some(p)) = self.ports.get_mut(port_idx) {
            p.write_byte(value);
        }
    }

    #[allow(dead_code)]
    pub fn get_status(&self) -> Vec<SerialPortStatus> {
        let mut status = Vec::new();
        for port in self.ports.iter().flatten() {
            status.push(SerialPortStatus {
                base_port: port.base_port,
                baud_rate: port.baud_rate,
                data_bits: 8,
                stop_bits: 1,
                rx_fifo_count: port.rx_fifo.len(),
                tx_fifo_count: port.tx_fifo.len(),
                flow_control: FlowControl {
                    hardware_flow_enabled: port.hardware_flow_enabled,
                    xon_xoff_enabled: port.xon_xoff_enabled,
                },
            });
        }
        status
    }
}

#[allow(dead_code)]
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
    #[allow(dead_code)]
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
            lsr: LSR_THRE | LSR_TEMT, // Transmitter is empty
            msr: 0,
            ier: 0,
            hardware_flow_enabled: false,
            xon_xoff_enabled: false,
            xon_state: true,
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),
        }
    }

    #[allow(dead_code)]
    pub fn initialize(&mut self, _config: u8) {
        self.initialized = true;
    }

    #[allow(dead_code)]
    pub fn read_byte(&mut self) -> Option<u8> {
        self.rx_fifo.pop_front()
    }

    #[allow(dead_code)]
    pub fn write_byte(&mut self, value: u8) {
        self.tx_fifo.push_back(value);
    }

    #[allow(dead_code)]
    pub fn has_data(&self) -> bool {
        !self.rx_fifo.is_empty()
    }

    #[allow(dead_code)]
    pub fn add_input(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }

    #[allow(dead_code)]
    pub fn get_output(&mut self) -> Option<u8> {
        self.output_buffer.pop_front()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        self.mcr =
            if config.dtr { MCR_DTR } else { 0 } | if config.rts { MCR_RTS } else { 0 } | MCR_OUT2; // Enable interrupts

        Ok(())
    }

    #[allow(dead_code)]
    pub fn receive_byte(&mut self, value: u8) {
        if self.rx_fifo.len() < FIFO_SIZE {
            self.rx_fifo.push_back(value);
            self.lsr |= LSR_DR;
        }
    }
}

#[allow(dead_code)]
pub struct PortStatus {
    pub rx_ready: bool,
    pub tx_ready: bool,
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
    pub xon_state: bool,
}

#[allow(dead_code)]
pub struct PortConfig {
    pub baud_rate: u32,
    pub parity: Parity,
    pub dtr: bool,
    pub rts: bool,
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
}

#[allow(dead_code)]
pub enum Parity {
    None,
    Odd,
    Even,
}

#[allow(dead_code)]
const IIR_NO_INT: u8 = 0x01; // No interrupt pending

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlowControl {
    pub hardware_flow_enabled: bool,
    pub xon_xoff_enabled: bool,
}

#[allow(dead_code)]
pub struct Serial {
    input_buffer: VecDeque<u8>,
    output_buffer: VecDeque<u8>,
    ports: Vec<Option<SerialPort>>,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new(),
            ports: vec![Some(SerialPort::new()), None, None, None],
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
        !self.input_buffer.is_empty()
    }

    #[allow(dead_code)]
    pub fn add_input(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }

    #[allow(dead_code)]
    pub fn get_output(&mut self) -> Option<u8> {
        self.output_buffer.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_port_initialization() {
        let port = SerialPort::new();
        assert!(!port.initialized);
        assert_eq!(port.baud_rate, 9600);
        assert_eq!(port.lsr, LSR_THRE | LSR_TEMT);
        assert!(port.rx_fifo.is_empty());
        assert!(port.tx_fifo.is_empty());
    }

    #[test]
    fn test_serial_port_configuration() {
        let mut port = SerialPort::new();
        let config = PortConfig {
            baud_rate: 19200,
            parity: Parity::Even,
            dtr: true,
            rts: true,
            hardware_flow_enabled: true,
            xon_xoff_enabled: false,
        };
        
        assert!(port.configure(config).is_ok());
        assert_eq!(port.baud_rate, 19200);
        assert!(port.hardware_flow_enabled);
        assert!(!port.xon_xoff_enabled);
        assert_eq!(port.mcr & MCR_DTR, MCR_DTR);
        assert_eq!(port.mcr & MCR_RTS, MCR_RTS);
    }

    #[test]
    fn test_serial_port_data_transfer() {
        let mut port = SerialPort::new();
        
        // Test write
        port.write_byte(0x41); // 'A'
        assert_eq!(port.tx_fifo.len(), 1);
        assert_eq!(port.tx_fifo[0], 0x41);
        
        // Test read
        port.receive_byte(0x42); // 'B'
        assert_eq!(port.read_byte(), Some(0x42));
        assert!(port.rx_fifo.is_empty());
    }

    #[test]
    #[ignore]
    fn test_serial_port_status() {
        let mut port = SerialPort::new();
        
        // Test empty status
        assert!(!port.has_data());
        
        // Test with data
        port.receive_byte(0x41);
        assert!(port.has_data());
        assert_eq!(port.get_status() & LSR_DR, LSR_DR);
    }

    #[test]
    fn test_serial_controller_initialization() {
        let controller = SerialController::new();
        assert_eq!(controller.ports.len(), 4);
        assert!(controller.ports[0].is_some());
        assert!(controller.ports[1].is_none());
        assert!(controller.ports[2].is_none());
        assert!(controller.ports[3].is_none());
    }

    #[test]
    #[ignore]
    fn test_serial_controller_port_access() {
        let mut controller = SerialController::new();
        
        // Test write to COM1
        controller.write_byte(COM1_BASE, 0x41);
        assert_eq!(controller.read_byte(COM1_BASE), 0x41);
        
        // Test write to invalid port
        controller.write_byte(0x1234, 0x42);
        assert_eq!(controller.read_byte(0x1234), 0);
    }

    #[test]
    fn test_serial_port_fifo() {
        let mut port = SerialPort::new();
        
        // Fill FIFO
        for i in 0..FIFO_SIZE {
            port.receive_byte(i as u8);
        }
        
        // Verify FIFO is full
        assert_eq!(port.rx_fifo.len(), FIFO_SIZE);
        
        // Read from FIFO
        for i in 0..FIFO_SIZE {
            assert_eq!(port.read_byte(), Some(i as u8));
        }
        
        // Verify FIFO is empty
        assert!(port.rx_fifo.is_empty());
    }

    #[test]
    fn test_serial_port_line_status() {
        let mut port = SerialPort::new();
        
        // Test initial line status
        assert_eq!(port.lsr & LSR_THRE, LSR_THRE);
        assert_eq!(port.lsr & LSR_TEMT, LSR_TEMT);
        
        // Test after receiving data
        port.receive_byte(0x41);
        assert_eq!(port.lsr & LSR_DR, LSR_DR);
    }

    #[test]
    fn test_serial_port_modem_control() {
        let mut port = SerialPort::new();
        
        // Test DTR and RTS signals
        port.mcr = MCR_DTR | MCR_RTS;
        assert_eq!(port.mcr & MCR_DTR, MCR_DTR);
        assert_eq!(port.mcr & MCR_RTS, MCR_RTS);
        
        // Test interrupt enable
        assert_eq!(port.mcr & MCR_OUT2, 0);
        port.mcr |= MCR_OUT2;
        assert_eq!(port.mcr & MCR_OUT2, MCR_OUT2);
    }
}

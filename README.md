# DOS Emulator in Rust

A lightweight x86 DOS emulator implemented in Rust that emulates core PC functionality including BIOS, CPU, memory, and disk operations.

## Features

- 8086/8088 CPU emulation
- BIOS interrupt handling
- Serial port I/O
- Disk operations (MBR, FAT16)
- 1MB memory management
- Basic DMA support

## Architecture

### Core Components

1. **CPU**
   - Complete 8086 instruction set
   - Register management
   - Flag operations
   - Interrupt handling

2. **Memory**
   - 1MB address space
   - ROM (64KB)
   - Conventional memory
   - BIOS data area

3. **BIOS**
   - Interrupt handlers
   - Hardware initialization
   - Basic I/O services

4. **Disk System**
   - MBR handling
   - FAT16 filesystem support
   - Disk I/O operations

5. **Serial Interface**
   - COM port emulation
   - TTY output support

## Building

```bash
# Clone the repository
git clone https://github.com/yourusername/dos-emu
cd dos-emu

# Build the project
cargo build --release
```

## Usage

1. Create the disk image and directory structure:
```bash
cargo run --bin extract -- extract-all freedos
```
This will create the `drive_c` directory with a properly configured FreeDOS disk image.

2. Run the emulator:
```bash
cargo run --bin dos_emu
```

## Debugging

### CPU State Debugging

1. Enable debug output in `main.rs`:
```rust
// Add debug prints
if cpu.is_halted() {
    println!("CPU State at halt:");
    println!("CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);
    println!("Flags: {:?}", cpu.regs.flags);
    println!("Last instruction at: {:05X}", ((cpu.regs.cs as u32) << 4) + cpu.regs.ip as u32);
}
```

2. Memory dump utility:
```rust
fn dump_memory(cpu: &CPU, start: u32, length: u32) {
    for i in (start..start+length).step_by(16) {
        print!("{:05X}:", i);
        for j in 0..16 {
            if i+j < start+length {
                print!(" {:02X}", cpu.memory.read_byte(i+j));
            }
        }
        println!();
    }
}
```

### Common Debug Points

1. BIOS Entry (F000:FFF0)
2. MBR Load (0000:7C00)
3. Boot Sector (Partition start + 7C00)
4. Interrupt Handlers

## Troubleshooting

### Common Issues

1. **"MBR boot code not present" Error**
   - Verify disk image has valid boot code at offset 0
   - Check partition table at offset 446
   - Ensure boot signature (55 AA) at offset 510

2. **CPU Not Halting**
   - Check reset vector points to correct entry point
   - Verify HLT instruction (0xF4) is present
   - Debug CPU state and instruction pointer

3. **Memory Access Violations**
   - Verify segment:offset calculations
   - Check stack pointer initialization
   - Monitor memory access patterns

4. **Disk Read Failures**
   - Validate disk image format
   - Check LBA calculations
   - Verify partition boundaries

### Solutions

1. **Reset Vector Issues**
```rust
// Correct reset vector setup
data[0xFFF0] = 0xEA;     // Far JMP
data[0xFFF1] = 0x5B;     // IP = E05B
data[0xFFF2] = 0xE0;
data[0xFFF3] = 0x00;     // CS = F000
data[0xFFF4] = 0xF0;
```

2. **Memory Access Fix**
```rust
// Safe memory access
pub fn read_byte(&self, addr: u32) -> u8 {
    if addr >= 0x100000 {
        return 0;  // Beyond 1MB
    }
    // ... rest of implementation
}
```

## Extended Technical Documentation

### CPU Implementation

1. **Instruction Execution Cycle**
```rust
loop {
    1. Fetch instruction
    2. Decode opcode
    3. Execute operation
    4. Update flags
    5. Check interrupts
    6. Check halt state
}
```

2. **Memory Segmentation**
```
Physical Address = Segment * 16 + Offset
Max Address = FFFF:FFFF = 10FFEF (1MB + 64KB - 16)
```

3. **Interrupt Handling**
```
Vector Table: 0000:0000 - 0000:03FF
Each Entry: 4 bytes (IP:CS)
Hardware Interrupts: 0x00-0x1F
Software Interrupts: 0x20-0xFF
```

### BIOS Services

1. **Video (INT 10h)**
   - AH=0E: TTY Output
   - AH=13: String Output

2. **Disk (INT 13h)**
   - AH=02: Read Sectors
   - AH=03: Write Sectors
   - AH=08: Get Drive Parameters

3. **Serial (INT 14h)**
   - AH=00: Initialize Port
   - AH=01: Send Character
   - AH=02: Receive Character

### FAT16 Implementation

1. **Filesystem Structure**
```
Boot Sector
FAT1
FAT2
Root Directory
Data Area
```

2. **Directory Entry Format**
```
Offset  Size    Description
0x00    8       Filename
0x08    3       Extension
0x0B    1       Attributes
0x0C    10      Reserved
0x16    2       Time
0x18    2       Date
0x1A    2       Starting Cluster
0x1C    4       File Size
```

## Development Roadmap

### Phase 1 - Current
- [x] Basic CPU emulation
- [x] Memory management
- [x] BIOS interrupts
- [x] Disk operations
- [x] Serial I/O

### Phase 2 - In Progress
- [ ] Extended instruction set
- [ ] Better error handling
- [ ] Improved debugging tools
- [ ] Configuration system
- [ ] Test suite

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. **Code Style**
   - Follow Rust style guidelines
   - Use meaningful variable names
   - Comment complex algorithms
   - Include unit tests

2. **Pull Request Process**
   - Fork the repository
   - Create a feature branch
   - Add tests for new features
   - Update documentation
   - Submit PR with description

3. **Testing**
   - Unit tests for new code
   - Integration tests for features
   - Regression testing
   - Performance benchmarks

## License

This project is licensed under the MIT License - see the LICENSE file for details.

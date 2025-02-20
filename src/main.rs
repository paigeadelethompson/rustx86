mod cpu;
mod memory;
mod rom;
mod disk;
mod serial;
mod dma;
mod bios;
mod boot;

use cpu::CPU;
use memory::Memory;
use boot::BootSector;
use serial::Serial;
use disk::Disk;
use rom::BiosRom;

fn main() -> Result<(), String> {
    println!("Starting emulator...");
    
    // Initialize components
    let mut memory = Memory::new(1024 * 1024); // 1MB RAM
    let boot_sector = BootSector::new();
    let serial = Serial::new();
    let disk = Disk::new();

    // Load BIOS ROM
    let bios_rom = rom::BiosRom::new();
    memory.load_rom(bios_rom.as_slice());

    // Create a boot sector that displays a message
    let mut boot_sector_data = vec![0u8; 512];
    
    // Boot sector code:
    // 0000:7C00  31 C0       XOR  AX, AX        ; Zero AX
    // 0000:7C02  8E D8       MOV  DS, AX        ; Set DS to 0
    // 0000:7C04  BE 15 7C    MOV  SI, msg       ; Point SI to message
    // 0000:7C07  B4 01       MOV  AH, 01h       ; Serial output function
    // 0000:7C09  B2 00       MOV  DL, 0         ; COM1
    // 0000:7C0B  AC          LODSB              ; Load character from [SI] to AL
    // 0000:7C0C  84 C0       TEST AL, AL        ; Check if character is 0
    // 0000:7C0E  74 06       JZ   done          ; If zero, we're done
    // 0000:7C10  CD 14       INT  14h           ; Call serial BIOS
    // 0000:7C12  EB F7       JMP  short loop    ; Repeat for next character
    // 0000:7C14  F4          HLT               ; Stop execution

    // Write the boot sector code
    boot_sector_data[0] = 0x31;  // XOR AX, AX
    boot_sector_data[1] = 0xC0;
    boot_sector_data[2] = 0x8E;  // MOV DS, AX
    boot_sector_data[3] = 0xD8;
    boot_sector_data[4] = 0xBE;  // MOV SI, msg
    boot_sector_data[5] = 0x15;  // offset of msg
    boot_sector_data[6] = 0x7C;  // segment of msg
    boot_sector_data[7] = 0xB4;  // MOV AH, 01h
    boot_sector_data[8] = 0x01;
    boot_sector_data[9] = 0xB2;  // MOV DL, 0
    boot_sector_data[10] = 0x00;
    boot_sector_data[11] = 0xAC;  // LODSB
    boot_sector_data[12] = 0x84;  // TEST AL, AL
    boot_sector_data[13] = 0xC0;
    boot_sector_data[14] = 0x74;  // JZ done
    boot_sector_data[15] = 0x04;  // Jump forward 4 bytes to HLT
    boot_sector_data[16] = 0xCD;  // INT 14h
    boot_sector_data[17] = 0x14;
    boot_sector_data[18] = 0xEB;  // JMP short loop
    boot_sector_data[19] = 0xF7;  // -9 bytes back to LODSB
    boot_sector_data[20] = 0xF4;  // HLT

    // Write the message
    let msg = b"Hello from boot sector!\r\n\0";
    println!("Message bytes: {:?}", msg);
    for (i, &byte) in msg.iter().enumerate() {
        boot_sector_data[21 + i] = byte;
        println!("Writing byte 0x{:02X} at offset {}", byte, 21 + i);
    }

    // Boot signature
    boot_sector_data[510] = 0x55;
    boot_sector_data[511] = 0xAA;

    // Load the boot sector
    memory.load_boot_sector(&boot_sector_data);

    // Initialize CPU and load BIOS
    let mut cpu = CPU::new(memory, serial, disk);
    println!("Initial CPU state - CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);

    // Initialize BIOS interrupts and data area
    bios::init_bios_interrupts(&mut cpu);
    bios::init_bios_data_area(&mut cpu);

    // Run CPU until we reach the boot sector or max cycles
    let mut cycles = 0;
    let max_init_cycles = 1000;
    
    while cycles < max_init_cycles {
        let cs = cpu.regs.cs;
        let ip = cpu.regs.ip;
        let physical_addr = ((cs as u32) << 4) + (ip as u32);
        
        println!("\nCycle {}: CS:IP={:04X}:{:04X}, Physical={:05X}", 
                 cycles, cs, ip, physical_addr);
        
        if let Err(e) = cpu.run() {
            println!("CPU error: {}", e);
            break;
        }
        
        // Log register changes
        println!("After instruction - CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);
        println!("AX={:04X} BX={:04X} CX={:04X} DX={:04X}", 
                 cpu.regs.ax, cpu.regs.bx, cpu.regs.cx, cpu.regs.dx);
        println!("SI={:04X} DI={:04X} BP={:04X} SP={:04X}", 
                 cpu.regs.si, cpu.regs.di, cpu.regs.bp, cpu.regs.sp);
        println!("Flags: {:?}", cpu.regs.flags);
        
        if cpu.is_halted() {
            println!("CPU halted at CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);
            break;
        }
        
        cycles += 1;
    }

    if cycles >= max_init_cycles {
        println!("\nReached maximum initialization cycles ({}) without reaching boot sector.", max_init_cycles);
        println!("Last known state:");
        println!("CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);
        println!("Physical address = {:05X}", ((cpu.regs.cs as u32) << 4) + (cpu.regs.ip as u32));
        return Ok(());
    }

    // Now load the boot sector
    println!("\nLoading boot sector...");
    boot_sector.load(&mut cpu.memory);
    
    // Run CPU until halted or max cycles reached
    println!("\nExecuting boot sector code...");
    let max_cycles = 1000; // Limit boot sector execution as well
    let mut cycles = 0;
    
    while cycles < max_cycles {
        match cpu.run() {
            Ok(_) => {
                cycles += 1;
                if cpu.is_halted() {
                    println!("\nCPU halted normally after {} cycles", cycles);
                    break;
                }
            }
            Err(e) => {
                println!("\nError during boot sector execution after {} cycles: {}", cycles, e);
                break;
            }
        }
    }

    if cycles >= max_cycles {
        println!("\nReached maximum boot sector cycles ({}). Final state:", max_cycles);
        println!("CS:IP = {:04X}:{:04X}", cpu.regs.cs, cpu.regs.ip);
        println!("Physical address = {:05X}", ((cpu.regs.cs as u32) << 4) + (cpu.regs.ip as u32));
    }
    
    Ok(())
} 
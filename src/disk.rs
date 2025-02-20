/*
 * Disk Emulation Implementation
 * ============================
 * 
 * This implementation provides a virtual disk that maps between DOS filesystem operations
 * and the host filesystem. The key components are:
 * 
 * Directory Structure:
 * -------------------
 * drive_c/
 *   |- mbr.bin         # Master Boot Record with partition table
 *   |- boot_sector.bin # FAT16 boot sector for C: partition
 *   |- fs/             # Contains actual files as they appear in FAT16
 *      |- IO.SYS
 *      |- MSDOS.SYS
 *      |- etc...
 * 
 * Key Concepts:
 * ------------
 * 1. Disk Layout:
 *    - MBR (Sector 0)
 *      * Boot code (first 446 bytes)
 *      * Partition table (64 bytes)
 *      * Boot signature
 *    - Primary Partition
 *      * Boot Sector
 *      * FAT1
 *      * FAT2
 *      * Root Directory
 *      * Data Area
 * 
 * 2. MBR Structure (Sector 0):
 *    - Offset  Size    Description
 *    - 0x000   446    Boot code (loads partition boot sector)
 *    - 0x1BE   64     Partition table (4 entries × 16 bytes)
 *    - 0x1FE   2      Boot signature (0x55, 0xAA)
 *    
 *    Partition Entry (16 bytes):
 *    - 0x00    1      Boot indicator (0x80 = bootable)
 *    - 0x01    1      Starting head
 *    - 0x02    1      Starting sector (bits 0-5), cylinder (bits 6-7)
 *    - 0x03    1      Starting cylinder (bits 0-7)
 *    - 0x04    1      System ID (0x06 = FAT16)
 *    - 0x05    1      Ending head
 *    - 0x06    1      Ending sector (bits 0-5), cylinder (bits 6-7)
 *    - 0x07    1      Ending cylinder (bits 0-7)
 *    - 0x08    4      Starting sector (LBA)
 *    - 0x0C    4      Total sectors
 * 
 * 3. FAT16 Boot Sector Structure:
 *    - Offset  Size    Description
 *    - 0x000   3      Jump instruction
 *    - 0x003   8      OEM Name
 *    - 0x00B   25     BIOS Parameter Block (BPB):
 *      * 0x00B   2      Bytes per sector (usually 512)
 *      * 0x00D   1      Sectors per cluster
 *      * 0x00E   2      Reserved sectors
 *      * 0x010   1      Number of FATs
 *      * 0x011   2      Root directory entries
 *      * 0x013   2      Total sectors (if < 65535)
 *      * 0x015   1      Media descriptor
 *      * 0x016   2      Sectors per FAT
 *      * 0x018   2      Sectors per track
 *      * 0x01A   2      Number of heads
 *      * 0x01C   4      Hidden sectors
 *      * 0x020   4      Total sectors (if >= 65535)
 *    - 0x024   ...    Extended BPB and boot code
 *    - 0x1FE   2      Boot signature (0x55, 0xAA)
 * 
 * 4. Loading Sequence:
 *    BIOS -> MBR -> FAT16 Boot Sector -> DOS System Files
 *    
 *    The BIOS loads and executes the MBR, which then:
 *    1. Scans the partition table
 *    2. Finds the active partition
 *    3. Loads that partition's boot sector
 *    4. Transfers control to the boot sector
 *    
 *    The FAT16 boot sector then:
 *    1. Sets up the environment
 *    2. Locates the DOS system files using the BPB
 *    3. Loads IO.SYS/MSDOS.SYS
 *    4. Transfers control to DOS
 * 
 * 5. Disk Operations:
 *    - FDISK /MBR: Updates sector 0 (MBR)
 *    - FORMAT C:: Updates partition boot sector and clears filesystem
 * 
 * 2. Virtual FAT:
 *    - FAT table is maintained in memory
 *    - Rebuilt from actual files in fs/ directory on startup
 *    - Maps clusters to file locations
 * 
 * 3. Sector Operations:
 *    - read_sector() maps sector numbers to appropriate regions
 *    - write_sector() handles writes to different regions:
 *      * MBR writes may come from FDISK /MBR
 *      * Boot sector writes may come from FORMAT C:
 *      * FAT writes update the in-memory FAT table
 *      * Directory writes may create/delete files
 *      * Data writes go to the appropriate file
 * 
 * 4. File Operations:
 *    - Files in fs/ are real files on the host system
 *    - Directory entries map between DOS names and host files
 *    - File operations (create/delete) directly translate to file create/delete operations on the host
 * 
 * 5. Format Handling:
 *    - FORMAT C: detected when boot sector is rewritten
 *    - FDISK /MBR detected when MBR is rewritten
 *    - Only FORMAT C: clears fs/ directory
 *    - FDISK /MBR only updates partition table
 * 
 * 6. BPB (BIOS Parameter Block):
 *    - Stored in boot sector
 *    - Defines disk geometry and filesystem parameters
 *    - Used to calculate sector mappings
 * 
 * This implementation allows DOS to work with files normally while
 * maintaining them as regular files on the host system. The FAT
 * filesystem structure is virtualized, with real files being stored
 * in the fs/ directory with their DOS names.
 */

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

const SECTOR_SIZE: usize = 512;

#[derive(Debug)]
struct DiskGeometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

impl Default for DiskGeometry {
    fn default() -> Self {
        Self {
            cylinders: 1024,
            heads: 16,
            sectors: 63,
        }
    }
}

#[derive(Debug, Clone)]
struct DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: u8,
    reserved: [u8; 10],
    time: u16,
    date: u16,
    start_cluster: u16,
    file_size: u32,
}

impl DirEntry {
    fn new() -> Self {
        Self {
            name: [0x20; 8],      // Space padded
            ext: [0x20; 3],       // Space padded
            attr: 0,
            reserved: [0; 10],
            time: 0,
            date: 0,
            start_cluster: 0,
            file_size: 0,
        }
    }

    fn from_host_file(path: &Path) -> io::Result<Self> {
        let metadata = path.metadata()?;
        let filename = path.file_name().unwrap().to_string_lossy();
        let mut entry = Self::new();

        // Split filename into name and extension
        let (name, ext) = match filename.rsplit_once('.') {
            Some((n, e)) => (n, e),
            None => (filename.as_ref(), ""),
        };

        // Copy name and extension (space padded)
        entry.name[..name.len().min(8)].copy_from_slice(&name.as_bytes()[..name.len().min(8)]);
        entry.ext[..ext.len().min(3)].copy_from_slice(&ext.as_bytes()[..ext.len().min(3)]);

        // Set attributes
        entry.attr = if metadata.is_dir() { 0x10 } else { 0x20 };
        entry.file_size = metadata.len() as u32;

        Ok(entry)
    }
}

#[derive(Debug)]
struct BiosParameterBlock {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    number_of_fats: u8,
    root_entries: u16,
    total_sectors: u16,
    media_descriptor: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    number_of_heads: u16,
    hidden_sectors: u32,
    large_sectors: u32,
}

impl BiosParameterBlock {
    fn from_boot_sector(data: &[u8]) -> Self {
        BiosParameterBlock {
            bytes_per_sector: u16::from_le_bytes([data[11], data[12]]),
            sectors_per_cluster: data[13],
            reserved_sectors: u16::from_le_bytes([data[14], data[15]]),
            number_of_fats: data[16],
            root_entries: u16::from_le_bytes([data[17], data[18]]),
            total_sectors: u16::from_le_bytes([data[19], data[20]]),
            media_descriptor: data[21],
            sectors_per_fat: u16::from_le_bytes([data[22], data[23]]),
            sectors_per_track: u16::from_le_bytes([data[24], data[25]]),
            number_of_heads: u16::from_le_bytes([data[26], data[27]]),
            hidden_sectors: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
            large_sectors: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),
        }
    }

    fn default() -> Self {
        BiosParameterBlock {
            bytes_per_sector: 512,
            sectors_per_cluster: 4,
            reserved_sectors: 1,
            number_of_fats: 2,
            root_entries: 224,
            total_sectors: 2880,
            media_descriptor: 0xF0,
            sectors_per_fat: 9,
            sectors_per_track: 18,
            number_of_heads: 2,
            hidden_sectors: 0,
            large_sectors: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PartitionEntry {
    bootable: bool,
    start_head: u8,
    start_sector: u8,
    start_cylinder: u16,
    system_id: u8,
    end_head: u8,
    end_sector: u8,
    end_cylinder: u16,
    start_lba: u32,
    total_sectors: u32,
}

impl PartitionEntry {
    fn new() -> Self {
        PartitionEntry {
            bootable: false,
            start_head: 0,
            start_sector: 1,
            start_cylinder: 0,
            system_id: 0x06,  // FAT16
            end_head: 15,
            end_sector: 63,
            end_cylinder: 1023,
            start_lba: 63,    // Standard offset for first partition
            total_sectors: 65536, // 32MB partition
        }
    }

    fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0] = if self.bootable { 0x80 } else { 0x00 };
        bytes[1] = self.start_head;
        // Start cylinder bits 8-9 go into bits 6-7 of start_sector
        let start_cyl_high = ((self.start_cylinder >> 8) & 0x03) as u8;
        bytes[2] = (self.start_sector & 0x3F) | (start_cyl_high << 6);
        bytes[3] = (self.start_cylinder & 0xFF) as u8;
        bytes[4] = self.system_id;
        bytes[5] = self.end_head;
        // End cylinder bits 8-9 go into bits 6-7 of end_sector
        let end_cyl_high = ((self.end_cylinder >> 8) & 0x03) as u8;
        bytes[6] = (self.end_sector & 0x3F) | (end_cyl_high << 6);
        bytes[7] = (self.end_cylinder & 0xFF) as u8;
        bytes[8..12].copy_from_slice(&self.start_lba.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.total_sectors.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        PartitionEntry {
            bootable: bytes[0] == 0x80,
            start_head: bytes[1],
            start_sector: bytes[2] & 0x3F,
            start_cylinder: ((bytes[2] as u16 & 0xC0) << 2) as u16 | bytes[3] as u16,
            system_id: bytes[4],
            end_head: bytes[5],
            end_sector: bytes[6] & 0x3F,
            end_cylinder: ((bytes[6] as u16 & 0xC0) << 2) as u16 | bytes[7] as u16,
            start_lba: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            total_sectors: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        }
    }
}

pub struct DiskImage {
    geometry: DiskGeometry,
    mbr: [u8; SECTOR_SIZE],
    boot_sector: [u8; SECTOR_SIZE],
    fat_table: Vec<u16>,
    root_directory: Vec<DirEntry>,
    fs_path: PathBuf,
    cluster_map: HashMap<u32, (PathBuf, u64)>,
    write_protected: bool,
    bpb: BiosParameterBlock,
    fat_cache: Vec<u8>,
    root_dir_cache: Vec<u8>,
    data_sector_cache: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum DiskRegion {
    BootSector,
    FAT1,
    FAT2,
    RootDirectory,
    Data,
}

impl DiskImage {
    pub fn new(drive_c_path: &Path) -> io::Result<Self> {
        let fs_path = drive_c_path.join("fs");
        let mbr_path = drive_c_path.join("mbr.bin");
        let boot_sector_path = drive_c_path.join("boot_sector.bin");

        // Create fs directory if it doesn't exist
        fs::create_dir_all(&fs_path)?;

        // Load or create MBR
        let mbr = if mbr_path.exists() {
            let mut file = File::open(&mbr_path)?;
            let mut data = [0u8; SECTOR_SIZE];
            file.read_exact(&mut data)?;
            data
        } else {
            let data = Self::create_default_mbr();
            let mut file = File::create(&mbr_path)?;
            file.write_all(&data)?;
            data
        };

        // Load or create boot sector
        let boot_sector = if boot_sector_path.exists() {
            let mut file = File::open(&boot_sector_path)?;
            let mut data = [0u8; SECTOR_SIZE];
            file.read_exact(&mut data)?;
            data
        } else {
            let data = Self::create_default_boot_sector();
            let mut file = File::create(&boot_sector_path)?;
            file.write_all(&data)?;
            data
        };

        let bpb = BiosParameterBlock::from_boot_sector(&boot_sector);

        let mut disk = DiskImage {
            geometry: DiskGeometry::default(),
            mbr,
            boot_sector,
            fat_table: Vec::new(),
            root_directory: Vec::new(),
            fs_path,
            cluster_map: HashMap::new(),
            write_protected: false,
            bpb,
            fat_cache: Vec::new(),
            root_dir_cache: Vec::new(),
            data_sector_cache: vec![0; SECTOR_SIZE],
        };

        // Build FAT structures from fs/ contents
        disk.rebuild_fat_from_fs()?;
        Ok(disk)
    }

    fn create_default_mbr() -> [u8; SECTOR_SIZE] {
        let mut mbr = [0u8; SECTOR_SIZE];
        
        // Real MS-DOS MBR code that searches for active partition and loads it
        let boot_code = [
            0xFA,             // CLI - Disable interrupts
            0x33, 0xC0,      // XOR AX, AX
            0x8E, 0xD0,      // MOV SS, AX
            0xBC, 0x00, 0x7C, // MOV SP, 0x7C00
            0x8E, 0xD8,      // MOV DS, AX
            0x8E, 0xC0,      // MOV ES, AX
            
            // Read partition table entries
            0xBE, 0xBE, 0x7C, // MOV SI, 0x7CBE (partition table offset)
            0xBF, 0xBE, 0x07, // MOV DI, 0x7BE  (another pointer to partition table)
            0xB9, 0x04, 0x00, // MOV CX, 4      (4 partition entries)
            
            // Search for active partition
            0x56,             // PUSH SI
            0x8A, 0x04,      // MOV AL, [SI]    (get boot indicator)
            0x3C, 0x80,      // CMP AL, 0x80    (is it active/bootable?)
            0x74, 0x12,      // JE found_active (if yes, jump to load it)
            0x46,            // INC SI
            0x46,            // INC SI
            0x46,            // INC SI
            0x46,            // INC SI
            0xE2, 0xF4,      // LOOP search_loop
            
            // No active partition found - print error and halt
            0xB4, 0x0E,      // MOV AH, 0x0E    (BIOS teletype output)
            0xB0, 0x4E,      // MOV AL, 'N'     (No)
            0xCD, 0x10,      // INT 0x10        (BIOS video service)
            0xB0, 0x6F,      // MOV AL, 'o'
            0xCD, 0x10,      // INT 0x10
            0xEB, 0xFE,      // JMP $ (hang)
            
            // Found active partition - load its boot sector
            0x5E,            // POP SI
            0x8B, 0x44, 0x08,// MOV AX, [SI+8]  (LBA start sector)
            0x8B, 0x4C, 0x0A,// MOV CX, [SI+10] (sector count)
            0xB4, 0x02,      // MOV AH, 0x02    (BIOS read sectors)
            0xB0, 0x01,      // MOV AL, 1       (sectors to read)
            0xBB, 0x00, 0x7C,// MOV BX, 0x7C00  (destination address)
            0x8A, 0x74, 0x01,// MOV DH, [SI+1]  (head)
            0x8A, 0x54, 0x02,// MOV DL, [SI+2]  (sector)
            0xCD, 0x13,      // INT 0x13        (BIOS disk service)
            0x73, 0x0A,      // JNC success     (if carry clear, jump to success)
            
            // Read error - print 'E' and halt
            0xB4, 0x0E,      // MOV AH, 0x0E
            0xB0, 0x45,      // MOV AL, 'E'
            0xCD, 0x10,      // INT 0x10
            0xEB, 0xFE,      // JMP $ (hang)
            
            // Success - jump to loaded boot sector
            0xEA, 0x00, 0x7C, 0x00, 0x00  // JMP 0000:7C00
        ];
        
        // Copy boot code
        mbr[..boot_code.len()].copy_from_slice(&boot_code);
        
        // Create primary partition entry
        let partition = PartitionEntry::new();
        let partition_bytes = partition.to_bytes();
        
        // Copy partition entry to partition table
        mbr[446..462].copy_from_slice(&partition_bytes);
        
        // Boot signature
        mbr[510] = 0x55;
        mbr[511] = 0xAA;
        
        mbr
    }

    fn create_default_boot_sector() -> [u8; SECTOR_SIZE] {
        let mut boot = [0u8; SECTOR_SIZE];
        let bpb = BiosParameterBlock::default();
        
        // Jump instruction and OEM name
        boot[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
        boot[3..11].copy_from_slice(b"MSWIN4.1");

        // BPB values
        boot[11..13].copy_from_slice(&bpb.bytes_per_sector.to_le_bytes());
        boot[13] = bpb.sectors_per_cluster;
        boot[14..16].copy_from_slice(&bpb.reserved_sectors.to_le_bytes());
        boot[16] = bpb.number_of_fats;
        boot[17..19].copy_from_slice(&bpb.root_entries.to_le_bytes());
        boot[19..21].copy_from_slice(&bpb.total_sectors.to_le_bytes());
        boot[21] = bpb.media_descriptor;
        boot[22..24].copy_from_slice(&bpb.sectors_per_fat.to_le_bytes());
        boot[24..26].copy_from_slice(&bpb.sectors_per_track.to_le_bytes());
        boot[26..28].copy_from_slice(&bpb.number_of_heads.to_le_bytes());
        boot[28..32].copy_from_slice(&bpb.hidden_sectors.to_le_bytes());
        boot[32..36].copy_from_slice(&bpb.large_sectors.to_le_bytes());

        // Boot signature
        boot[510] = 0x55;
        boot[511] = 0xAA;
        
        boot
    }

    fn rebuild_fat_from_fs(&mut self) -> io::Result<()> {
        self.fat_table.clear();
        self.root_directory.clear();
        self.cluster_map.clear();

        let mut next_cluster = 2; // Clusters 0 and 1 are reserved

        // Walk the fs/ directory
        for entry in fs::read_dir(&self.fs_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dir_entry = DirEntry::from_host_file(&path)?;
                
                // Allocate clusters for the file
                let size = dir_entry.file_size;
                let clusters_needed = ((size + (SECTOR_SIZE as u32 * self.bpb.sectors_per_cluster as u32 - 1))
                    / (SECTOR_SIZE as u32 * self.bpb.sectors_per_cluster as u32)) as u16;
                
                // Update directory entry
                let mut entry = dir_entry;
                entry.start_cluster = next_cluster as u16;
                self.root_directory.push(entry);

                // Update FAT chain
                for i in 0..clusters_needed {
                    let current_cluster = next_cluster + i as u32;
                    let next = if i == clusters_needed - 1 { 0xFFFF } else { (current_cluster + 1) as u16 };
                    
                    // Extend FAT if needed
                    if current_cluster as usize >= self.fat_table.len() {
                        self.fat_table.resize(current_cluster as usize + 1, 0);
                    }
                    self.fat_table[current_cluster as usize] = next;

                    // Map cluster to file location
                    self.cluster_map.insert(
                        current_cluster,
                        (path.clone(), i as u64 * (SECTOR_SIZE as u64 * self.bpb.sectors_per_cluster as u64))
                    );
                }

                next_cluster += clusters_needed as u32;
            }
        }

        Ok(())
    }

    pub fn read_sector(&mut self, sector: u32) -> Option<&[u8]> {
        match self.sector_to_region(sector) {
            DiskRegion::BootSector => Some(&self.boot_sector),
            DiskRegion::FAT1 | DiskRegion::FAT2 => {
                let offset = self.fat_offset(sector);
                self.fat_cache = self.fat_bytes();
                Some(&self.fat_cache[offset..offset + SECTOR_SIZE])
            }
            DiskRegion::RootDirectory => {
                let offset = self.root_dir_offset(sector);
                self.root_dir_cache = self.root_dir_bytes();
                Some(&self.root_dir_cache[offset..offset + SECTOR_SIZE])
            }
            DiskRegion::Data => self.read_data_sector(sector),
        }
    }

    pub fn write_sector(&mut self, sector: u32, data: &[u8]) -> io::Result<()> {
        if self.write_protected {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Disk is write protected"));
        }

        // Handle MBR writes (sector 0)
        if sector == 0 {
            if self.detect_fdisk_mbr(data) {
                // FDISK /MBR detected - update MBR but don't clear filesystem
                self.mbr.copy_from_slice(data);
                let mut file = File::create(self.fs_path.parent().unwrap().join("mbr.bin"))?;
                file.write_all(data)?;
            }
            return Ok(());
        }

        // Get partition info
        let partition = PartitionEntry::from_bytes(&self.mbr[446..462]);
        
        // Convert absolute sector to partition-relative sector
        let partition_sector = sector.checked_sub(partition.start_lba);
        
        match partition_sector {
            Some(0) => {
                // Writing to partition boot sector
                if self.detect_format_operation(data) {
                    // FORMAT C: detected
                    self.boot_sector.copy_from_slice(data);
                    self.bpb = BiosParameterBlock::from_boot_sector(data);
                    
                    // Save boot sector
                    let mut file = File::create(self.fs_path.parent().unwrap().join("boot_sector.bin"))?;
                    file.write_all(data)?;

                    // Clear filesystem
                    self.handle_format()?;
                }
            }
            Some(rel_sector) => {
                // Handle writes to FAT, root directory, or data area
                match self.sector_to_region(rel_sector) {
                    DiskRegion::FAT1 | DiskRegion::FAT2 => self.update_fat(rel_sector, data)?,
                    DiskRegion::RootDirectory => self.update_root_directory(rel_sector, data)?,
                    DiskRegion::Data => self.write_data_sector(rel_sector, data)?,
                    _ => (),
                }
            }
            None => {
                // Sector is outside partition bounds
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Sector outside partition bounds"));
            }
        }

        Ok(())
    }

    fn detect_format_operation(&self, boot_sector: &[u8]) -> bool {
        // Check if this is a new boot sector different from our current one
        // but only if it's writing to the partition boot sector, not the MBR
        boot_sector != &self.boot_sector[..]
    }

    fn detect_fdisk_mbr(&self, mbr: &[u8]) -> bool {
        // Check if this is a new MBR different from our current one
        mbr != &self.mbr[..]
    }

    fn handle_format(&mut self) -> io::Result<()> {
        // Clear out drive_c/fs/
        if self.fs_path.exists() {
            fs::remove_dir_all(&self.fs_path)?;
            fs::create_dir(&self.fs_path)?;
        }

        // Clear our in-memory structures
        self.fat_table.clear();
        self.root_directory.clear();
        self.cluster_map.clear();

        Ok(())
    }

    fn sector_to_region(&self, sector: u32) -> DiskRegion {
        let fat_start = self.bpb.reserved_sectors as u32;
        let fat_size = self.bpb.sectors_per_fat as u32;
        let root_dir_start = fat_start + (self.bpb.number_of_fats as u32 * fat_size);
        let root_dir_sectors = ((self.bpb.root_entries as u32 * 32) + 
            (self.bpb.bytes_per_sector as u32 - 1)) / self.bpb.bytes_per_sector as u32;
        let data_start = root_dir_start + root_dir_sectors;

        match sector {
            0 => DiskRegion::BootSector,
            s if s >= fat_start && s < fat_start + fat_size => DiskRegion::FAT1,
            s if s >= fat_start + fat_size && s < root_dir_start => DiskRegion::FAT2,
            s if s >= root_dir_start && s < data_start => DiskRegion::RootDirectory,
            _ => DiskRegion::Data,
        }
    }

    fn fat_offset(&self, sector: u32) -> usize {
        let fat_start = if self.sector_to_region(sector) == DiskRegion::FAT1 {
            self.bpb.reserved_sectors as u32
        } else {
            self.bpb.reserved_sectors as u32 + self.bpb.sectors_per_fat as u32
        };
        ((sector - fat_start) * self.bpb.bytes_per_sector as u32) as usize
    }

    fn root_dir_offset(&self, sector: u32) -> usize {
        let root_dir_start = self.bpb.reserved_sectors as u32 + 
            (self.bpb.number_of_fats as u32 * self.bpb.sectors_per_fat as u32);
        ((sector - root_dir_start) * self.bpb.bytes_per_sector as u32) as usize
    }

    fn fat_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.fat_table.len() * 2);
        for &entry in &self.fat_table {
            bytes.extend_from_slice(&entry.to_le_bytes());
        }
        bytes
    }

    fn root_dir_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.bpb.root_entries as usize * 32);
        for entry in &self.root_directory {
            bytes.extend_from_slice(&entry.name);
            bytes.extend_from_slice(&entry.ext);
            bytes.push(entry.attr);
            bytes.extend_from_slice(&entry.reserved);
            bytes.extend_from_slice(&entry.time.to_le_bytes());
            bytes.extend_from_slice(&entry.date.to_le_bytes());
            bytes.extend_from_slice(&entry.start_cluster.to_le_bytes());
            bytes.extend_from_slice(&entry.file_size.to_le_bytes());
        }
        // Pad to full size
        bytes.resize(self.bpb.root_entries as usize * 32, 0);
        bytes
    }

    fn read_data_sector(&mut self, sector: u32) -> Option<&[u8]> {
        let cluster = self.sector_to_cluster(sector);
        if let Some((file_path, offset)) = self.cluster_map.get(&cluster) {
            let sector_offset = self.sector_offset_in_cluster(sector);
            let file_offset = offset + (sector_offset * SECTOR_SIZE as u64);
            
            if let Ok(mut file) = File::open(file_path) {
                if file.seek(SeekFrom::Start(file_offset)).is_ok() {
                    self.data_sector_cache.fill(0);
                    if file.read_exact(&mut self.data_sector_cache).is_ok() {
                        return Some(&self.data_sector_cache);
                    }
                }
            }
        }
        Some(&[0; SECTOR_SIZE])
    }

    fn write_data_sector(&mut self, sector: u32, data: &[u8]) -> io::Result<()> {
        let cluster = self.sector_to_cluster(sector);
        if let Some((file_path, offset)) = self.cluster_map.get(&cluster).cloned() {
            let sector_offset = self.sector_offset_in_cluster(sector);
            let file_offset = offset + (sector_offset * SECTOR_SIZE as u64);
            
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(file_path)?;
            
            file.seek(SeekFrom::Start(file_offset))?;
            file.write_all(data)?;
        }
        Ok(())
    }

    fn update_fat(&mut self, sector: u32, data: &[u8]) -> io::Result<()> {
        let offset = self.fat_offset(sector);
        let entries = data.chunks(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();

        // Update FAT table
        for (i, &entry) in entries.iter().enumerate() {
            let fat_idx = offset / 2 + i;
            if fat_idx >= self.fat_table.len() {
                self.fat_table.resize(fat_idx + 1, 0);
            }
            self.fat_table[fat_idx] = entry;
        }

        // If this is FAT1, mirror to FAT2
        if self.sector_to_region(sector) == DiskRegion::FAT1 {
            let fat2_sector = sector + self.bpb.sectors_per_fat as u32;
            self.write_sector(fat2_sector, data)?;
        }

        Ok(())
    }

    fn update_root_directory(&mut self, sector: u32, data: &[u8]) -> io::Result<()> {
        let offset = self.root_dir_offset(sector);
        let entries = data.chunks(32)
            .map(|chunk| self.parse_dir_entry(chunk))
            .collect::<Vec<_>>();

        for (i, entry) in entries.iter().enumerate() {
            let dir_idx = offset / 32 + i;
            if dir_idx >= self.root_directory.len() {
                self.root_directory.resize(dir_idx + 1, DirEntry::new());
            }

            // Clone the old entry before handling file operation
            let old_entry = self.root_directory[dir_idx].clone();
            self.handle_file_operation(&old_entry, entry)?;
            self.root_directory[dir_idx] = entry.clone();
        }

        Ok(())
    }

    fn handle_file_operation(&mut self, old: &DirEntry, new: &DirEntry) -> io::Result<()> {
        let new_name = self.format_file_name(new);
        let old_name = self.format_file_name(old);
        let path = self.fs_path.join(&new_name);

        if new.name[0] == 0xE5 {
            // File deletion
            if old.name[0] != 0 && old.name[0] != 0xE5 {
                let old_path = self.fs_path.join(old_name);
                if old_path.exists() {
                    fs::remove_file(old_path)?;
                }
            }
        } else if old.name[0] == 0 && new.name[0] != 0 {
            // New file creation
            File::create(&path)?;
        }

        Ok(())
    }

    fn format_file_name(&self, entry: &DirEntry) -> String {
        let name = String::from_utf8_lossy(&entry.name).trim_end().to_string();
        let ext = String::from_utf8_lossy(&entry.ext).trim_end().to_string();
        if ext.is_empty() {
            name
        } else {
            format!("{}.{}", name, ext)
        }
    }

    fn parse_dir_entry(&self, data: &[u8]) -> DirEntry {
        let mut entry = DirEntry::new();
        entry.name.copy_from_slice(&data[0..8]);
        entry.ext.copy_from_slice(&data[8..11]);
        entry.attr = data[11];
        entry.reserved.copy_from_slice(&data[12..22]);
        entry.time = u16::from_le_bytes([data[22], data[23]]);
        entry.date = u16::from_le_bytes([data[24], data[25]]);
        entry.start_cluster = u16::from_le_bytes([data[26], data[27]]);
        entry.file_size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        entry
    }

    fn sector_to_cluster(&self, sector: u32) -> u32 {
        let root_dir_start = self.bpb.reserved_sectors as u32 + 
            (self.bpb.number_of_fats as u32 * self.bpb.sectors_per_fat as u32);
        let root_dir_sectors = ((self.bpb.root_entries as u32 * 32) + 
            (self.bpb.bytes_per_sector as u32 - 1)) / self.bpb.bytes_per_sector as u32;
        let data_start = root_dir_start + root_dir_sectors;
        
        ((sector - data_start) / self.bpb.sectors_per_cluster as u32) + 2
    }

    fn sector_offset_in_cluster(&self, sector: u32) -> u64 {
        (sector % self.bpb.sectors_per_cluster as u32) as u64
    }

    pub fn set_write_protected(&mut self, protected: bool) {
        self.write_protected = protected;
    }

    pub fn is_write_protected(&self) -> bool {
        self.write_protected
    }
} 
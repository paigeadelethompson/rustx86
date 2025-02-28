use std::io;
use std::path::Path;
use super::{
    SECTOR_SIZE, Mbr,
    FAT16_SECTORS_PER_CLUSTER, FAT16_RESERVED_SECTORS, FAT16_NUMBER_OF_FATS,
    FAT16_ROOT_ENTRIES, FAT16_TOTAL_SECTORS, FAT16_SECTORS_PER_FAT,
    FAT16_MEDIA_DESCRIPTOR, BYTES_PER_SECTOR, SECTORS_PER_TRACK, HEADS_PER_CYLINDER,
    PARTITION_TABLE_OFFSET, PARTITION_ENTRY_SIZE,
    FAT16_SYSTEM_ID
};

// Define sector start constants
const FAT1_START: u32 = 63; // First FAT starts after boot sector
const FAT2_START: u32 = FAT1_START + FAT16_SECTORS_PER_FAT as u32;
const ROOT_DIR_START: u32 = FAT2_START + FAT16_SECTORS_PER_FAT as u32;
const DATA_START: u32 = ROOT_DIR_START + ((FAT16_ROOT_ENTRIES * 32 + SECTOR_SIZE as u16 - 1) / SECTOR_SIZE as u16) as u32;

#[derive(Debug, Clone)]
pub struct BootSector {
    data: [u8; SECTOR_SIZE]
}

impl BootSector {
    pub fn new() -> Self {
        let mut data = [0u8; SECTOR_SIZE];
        // Boot signature bytes at specific offsets:
        data[SECTOR_SIZE - 2] = 0x55;  // Offset 510 (0x1FE)
        data[SECTOR_SIZE - 1] = 0xAA;  // Offset 511 (0x1FF)
        data[SECTOR_SIZE - 3] = 0x4;   // Offset 509 (0x1FD)
        BootSector { data }
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

pub struct DiskImage {
    pub geometry: DiskGeometry,
    pub mbr: Mbr,
    pub boot_sector: [u8; SECTOR_SIZE],
    pub fat_table: Vec<u8>,
    pub root_directory: Vec<u8>,
    pub data_sectors: Vec<u8>,
    pub write_protected: bool,
    pub bpb: BiosParameterBlock,
    pub mbr_cache: Vec<u8>,
    pub fat_cache: Vec<u8>,
    pub root_dir_cache: Vec<u8>,
    pub data_sector_cache: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct DiskGeometry {
    pub cylinders: u16,
    pub heads: u8,
    pub sectors: u8,
}

pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub number_of_fats: u8,
    pub root_entries: u16,
    pub total_sectors: u32,
    pub media_descriptor: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub number_of_heads: u16,
    pub hidden_sectors: u16,
    pub large_sectors: u32,
}

#[derive(PartialEq)]
pub enum DiskRegion {
    BootSector,
    FAT1,
    FAT2,
    RootDirectory,
    Data,
}

impl DiskImage {
    pub fn new(drive_c_path: &Path) -> io::Result<Self> {
        let geometry = DiskGeometry::default();
        
        // Create MBR with bootable FAT16 partition starting at sector 63
        let boot_code = vec![
            0x33, 0xC0,             // xor ax, ax
            0x8E, 0xD0,             // mov ss, ax
            0xBC, 0x00, 0x7C,       // mov sp, 0x7C00
            0x8E, 0xD8,             // mov ds, ax
            0x8E, 0xC0,             // mov es, ax
            0xBE, 0x00, 0x7C,       // mov si, 0x7C00
            0xBF, 0x00, 0x06,       // mov di, 0x0600
            0xB9, 0x00, 0x02,       // mov cx, 512
            0xFC,                   // cld
            0xF3, 0xA4,             // rep movsb
            0xEA, 0x00, 0x06, 0x00, 0x00  // jmp 0:0x0600
        ];

        let mbr = Mbr::create_bootable_fat16_mbr(boot_code).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Create boot sector for the FAT16 partition
        let boot_sector = BootSector::new();
        let bpb = BiosParameterBlock::new(
            FAT16_SECTORS_PER_CLUSTER,
            FAT16_RESERVED_SECTORS,
            FAT16_NUMBER_OF_FATS,
            FAT16_ROOT_ENTRIES,
            FAT16_TOTAL_SECTORS as u16,
            FAT16_MEDIA_DESCRIPTOR,
            FAT16_SECTORS_PER_FAT
        );

        // Calculate sizes based on BPB
        let fat_size = bpb.sectors_per_fat as usize * SECTOR_SIZE;
        let root_dir_size = (bpb.root_entries as usize * 32 + SECTOR_SIZE - 1) / SECTOR_SIZE * SECTOR_SIZE;
        let data_sectors_size = FAT16_TOTAL_SECTORS as usize * SECTOR_SIZE;

        // Initialize FAT table
        let mut fat_table = vec![0; fat_size];
        fat_table[0] = FAT16_MEDIA_DESCRIPTOR; // Media descriptor
        fat_table[1] = 0xFF; // FAT ID
        fat_table[2] = 0xFF; // End of chain marker

        // Initialize root directory
        let root_directory = vec![0; root_dir_size];

        // Initialize data sectors
        let data_sectors = vec![0; data_sectors_size];

        Ok(DiskImage {
            geometry,
            mbr,
            boot_sector: boot_sector.as_bytes().try_into().unwrap(),
            fat_table,
            root_directory,
            data_sectors,
            write_protected: false,
            bpb,
            mbr_cache: Vec::new(),
            fat_cache: Vec::new(),
            root_dir_cache: Vec::new(),
            data_sector_cache: Vec::new(),
        })
    }

    pub fn read_sector(&self, lba: u32) -> Option<Vec<u8>> {
        let mut sector = vec![0; SECTOR_SIZE];

        match lba {
            0 => sector.copy_from_slice(&self.mbr.to_bytes()),
            63 => sector.copy_from_slice(&self.boot_sector),
            _ => {
                let region = self.sector_to_region(lba);
                match region {
                    DiskRegion::BootSector => {
                        sector.copy_from_slice(&self.mbr.to_bytes());
                    }
                    DiskRegion::FAT1 => {
                        let offset = (lba - FAT1_START) as usize * SECTOR_SIZE;
                        sector.copy_from_slice(&self.fat_table[offset..offset + SECTOR_SIZE]);
                    }
                    DiskRegion::FAT2 => {
                        let offset = (lba - FAT2_START) as usize * SECTOR_SIZE;
                        sector.copy_from_slice(&self.fat_table[offset..offset + SECTOR_SIZE]);
                    }
                    DiskRegion::RootDirectory => {
                        let offset = (lba - ROOT_DIR_START) as usize * SECTOR_SIZE;
                        sector.copy_from_slice(&self.root_directory[offset..offset + SECTOR_SIZE]);
                    }
                    DiskRegion::Data => {
                        let offset = (lba - DATA_START) as usize * SECTOR_SIZE;
                        sector.copy_from_slice(&self.data_sectors[offset..offset + SECTOR_SIZE]);
                    }
                }
            }
        }

        Some(sector)
    }

    fn sector_to_region(&self, sector: u32) -> DiskRegion {
        if sector == 0 {
            DiskRegion::BootSector
        } else if sector >= FAT1_START && sector < FAT2_START {
            DiskRegion::FAT1
        } else if sector >= FAT2_START && sector < ROOT_DIR_START {
            DiskRegion::FAT2
        } else if sector >= ROOT_DIR_START && sector < DATA_START {
            DiskRegion::RootDirectory
        } else {
            DiskRegion::Data
        }
    }

    fn fat_offset(&self, sector: u32) -> usize {
        let partition = &self.mbr.partitions[0];
        let rel_sector = sector.checked_sub(partition.start_lba)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Sector outside partition bounds"))
            .unwrap_or(0);

        let fat_start = if self.sector_to_region(sector) == DiskRegion::FAT1 {
            self.bpb.reserved_sectors as u32
        } else {
            self.bpb.reserved_sectors as u32 + self.bpb.sectors_per_fat as u32
        };

        // Check if rel_sector is less than fat_start to avoid underflow
        if rel_sector < fat_start {
            0
        } else {
            ((rel_sector - fat_start) * self.bpb.bytes_per_sector as u32) as usize
        }
    }

    fn root_dir_offset(&self, sector: u32) -> usize {
        let partition = &self.mbr.partitions[0];
        let rel_sector = sector.checked_sub(partition.start_lba)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Sector outside partition bounds"))
            .unwrap_or(0);

        let root_dir_start = self.bpb.reserved_sectors as u32 + 
            (self.bpb.number_of_fats as u32 * self.bpb.sectors_per_fat as u32);

        if rel_sector < root_dir_start {
            0
        } else {
            ((rel_sector - root_dir_start) * self.bpb.bytes_per_sector as u32) as usize
        }
    }

    fn read_data_sector(&mut self, sector: u32) -> Option<&[u8]> {
        let offset = (sector as usize) * SECTOR_SIZE;
        if offset + SECTOR_SIZE <= self.data_sectors.len() {
            self.data_sector_cache.clear();
            self.data_sector_cache.extend_from_slice(&self.data_sectors[offset..offset + SECTOR_SIZE]);
            Some(&self.data_sector_cache)
        } else {
            None
        }
    }
}

impl BiosParameterBlock {
    pub fn new(
        sectors_per_cluster: u8,
        reserved_sectors: u16,
        number_of_fats: u8,
        root_entries: u16,
        total_sectors: u16,
        media_descriptor: u8,
        sectors_per_fat: u16
    ) -> Self {
        BiosParameterBlock {
            bytes_per_sector: BYTES_PER_SECTOR,
            sectors_per_cluster,
            reserved_sectors,
            number_of_fats,
            root_entries,
            total_sectors: total_sectors as u32,
            media_descriptor,
            sectors_per_fat,
            sectors_per_track: SECTORS_PER_TRACK,
            number_of_heads: HEADS_PER_CYLINDER,
            hidden_sectors: 0,
            large_sectors: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; 27] {
        let mut bytes = [0u8; 27];
        bytes[0..2].copy_from_slice(&self.bytes_per_sector.to_le_bytes());
        bytes[2] = self.sectors_per_cluster;
        bytes[3..5].copy_from_slice(&self.reserved_sectors.to_le_bytes());
        bytes[5] = self.number_of_fats;
        bytes[6..8].copy_from_slice(&self.root_entries.to_le_bytes());
        bytes[8..10].copy_from_slice(&(self.total_sectors as u16).to_le_bytes());
        bytes[10] = self.media_descriptor;
        bytes[11..13].copy_from_slice(&self.sectors_per_fat.to_le_bytes());
        bytes[13..15].copy_from_slice(&self.sectors_per_track.to_le_bytes());
        bytes[15..17].copy_from_slice(&self.number_of_heads.to_le_bytes());
        bytes[17..19].copy_from_slice(&self.hidden_sectors.to_le_bytes());
        bytes[21..25].copy_from_slice(&self.large_sectors.to_le_bytes());
        bytes
    }
} 
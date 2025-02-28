use super::{
    Mbr, BYTES_PER_SECTOR, FAT16_MEDIA_DESCRIPTOR, FAT16_NUMBER_OF_FATS, FAT16_RESERVED_SECTORS,
    FAT16_ROOT_ENTRIES, FAT16_SECTORS_PER_CLUSTER, FAT16_SECTORS_PER_FAT, FAT16_TOTAL_SECTORS,
    HEADS_PER_CYLINDER, SECTORS_PER_TRACK, SECTOR_SIZE,
};
use std::io;
use std::path::Path;

// Define sector start constants
const BOOT_SECTOR: u32 = 63; // Boot sector is at sector 63
const FAT1_START: u32 = 64; // First FAT starts after boot sector
const FAT2_START: u32 = FAT1_START + FAT16_SECTORS_PER_FAT as u32;
const ROOT_DIR_START: u32 = FAT2_START + FAT16_SECTORS_PER_FAT as u32;
const DATA_START: u32 =
    ROOT_DIR_START + (FAT16_ROOT_ENTRIES * 32).div_ceil(SECTOR_SIZE as u16) as u32;

#[derive(Debug, Clone)]
pub struct BootSector {
    data: [u8; SECTOR_SIZE],
}

impl Default for BootSector {
    fn default() -> Self {
        Self::new()
    }
}

impl BootSector {
    pub fn new() -> Self {
        let mut data = [0u8; SECTOR_SIZE];
        // Boot signature bytes at specific offsets:
        data[SECTOR_SIZE - 2] = 0x55; // Offset 510 (0x1FE)
        data[SECTOR_SIZE - 1] = 0xAA; // Offset 511 (0x1FF)
        data[SECTOR_SIZE - 3] = 0x4; // Offset 509 (0x1FD)
        BootSector { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug)]
pub struct DiskGeometry {
    #[allow(dead_code)]
    pub cylinders: u16,
    #[allow(dead_code)]
    pub heads: u8,
    #[allow(dead_code)]
    pub sectors: u8,
    pub _bytes_per_sector: u16,
}

impl Default for DiskGeometry {
    fn default() -> Self {
        DiskGeometry {
            cylinders: 80,
            heads: 2,
            sectors: 18,
            _bytes_per_sector: 512,
        }
    }
}

#[derive(Debug)]
pub struct BiosParameterBlock {
    pub _bytes_per_sector: u16,
    pub _sectors_per_cluster: u8,
    pub _reserved_sectors: u16,
    pub _num_fats: u8,
    pub _root_entries: u16,
    pub _total_sectors: u16,
    pub _media_descriptor: u8,
    pub _sectors_per_fat: u16,
    pub _sectors_per_track: u16,
    pub _num_heads: u16,
    pub _hidden_sectors: u32,
    pub _large_sectors: u32,
}

#[derive(Debug, PartialEq)]
pub enum DiskRegion {
    BootSector,
    FAT1,
    FAT2,
    RootDirectory,
    Data,
}

#[derive(Debug)]
pub struct DiskImage {
    #[allow(dead_code)]
    pub geometry: DiskGeometry,
    pub mbr: Mbr,
    pub boot_sector: [u8; SECTOR_SIZE],
    pub fat_table: Vec<u8>,
    pub root_directory: Vec<u8>,
    pub data_sectors: Vec<u8>,
    #[allow(dead_code)]
    pub write_protected: bool,
    pub _bpb: BiosParameterBlock,
    #[allow(dead_code)]
    pub mbr_cache: Vec<u8>,
    #[allow(dead_code)]
    pub fat_cache: Vec<u8>,
    #[allow(dead_code)]
    pub root_dir_cache: Vec<u8>,
    #[allow(dead_code)]
    pub data_sector_cache: Vec<u8>,
}

impl DiskImage {
    pub fn new(_drive_c_path: &Path) -> io::Result<Self> {
        let geometry = DiskGeometry::default();

        // Create MBR with bootable FAT16 partition starting at sector 63
        let boot_code = vec![
            0x33, 0xC0, // xor ax, ax
            0x8E, 0xD0, // mov ss, ax
            0xBC, 0x00, 0x7C, // mov sp, 0x7C00
            0x8E, 0xD8, // mov ds, ax
            0x8E, 0xC0, // mov es, ax
            0xBE, 0x00, 0x7C, // mov si, 0x7C00
            0xBF, 0x00, 0x06, // mov di, 0x0600
            0xB9, 0x00, 0x02, // mov cx, 512
            0xFC, // cld
            0xF3, 0xA4, // rep movsb
            0xEA, 0x00, 0x06, 0x00, 0x00, // jmp 0:0x0600
        ];

        let mbr = Mbr::create_bootable_fat16_mbr(boot_code)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Create boot sector for the FAT16 partition
        let boot_sector = BootSector::new();
        let bpb = BiosParameterBlock::new(
            FAT16_SECTORS_PER_CLUSTER,
            FAT16_RESERVED_SECTORS,
            FAT16_NUMBER_OF_FATS,
            FAT16_ROOT_ENTRIES,
            FAT16_TOTAL_SECTORS as u16,
            FAT16_MEDIA_DESCRIPTOR,
            FAT16_SECTORS_PER_FAT,
        );

        // Calculate sizes based on BPB
        let fat_size = bpb._sectors_per_fat as usize * SECTOR_SIZE;
        let root_dir_size = (bpb._root_entries as usize * 32).div_ceil(SECTOR_SIZE) * SECTOR_SIZE;
        let data_sectors_size = FAT16_TOTAL_SECTORS as usize * SECTOR_SIZE;

        // Initialize FAT table with media descriptor and FAT ID
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
            _bpb: bpb,
            mbr_cache: Vec::new(),
            fat_cache: Vec::new(),
            root_dir_cache: Vec::new(),
            data_sector_cache: Vec::new(),
        })
    }

    pub fn read_sector(&self, lba: u32) -> Option<Vec<u8>> {
        let mut sector = vec![0; SECTOR_SIZE];
        println!("Reading sector {}", lba);
        println!("FAT table first bytes: {:?}", &self.fat_table[..4]);

        // For sectors beyond disk size, return a zeroed sector
        if lba >= FAT16_TOTAL_SECTORS {
            return Some(sector);
        }

        match lba {
            0 => {
                println!("Reading MBR");
                sector.copy_from_slice(&self.mbr.to_bytes());
            }
            _ => {
                let region = self.sector_to_region(lba);
                println!("Region for sector {}: {:?}", lba, region);
                match region {
                    DiskRegion::BootSector => {
                        println!("Reading boot sector region");
                        sector.copy_from_slice(&self.boot_sector);
                    }
                    DiskRegion::FAT1 => {
                        let offset = (lba - FAT1_START) as usize * SECTOR_SIZE;
                        if offset + SECTOR_SIZE <= self.fat_table.len() {
                            println!("Reading FAT1 at offset {}", offset);
                            sector.copy_from_slice(&self.fat_table[offset..offset + SECTOR_SIZE]);
                        }
                    }
                    DiskRegion::FAT2 => {
                        let offset = (lba - FAT2_START) as usize * SECTOR_SIZE;
                        if offset + SECTOR_SIZE <= self.fat_table.len() {
                            println!("Reading FAT2 at offset {}", offset);
                            sector.copy_from_slice(&self.fat_table[offset..offset + SECTOR_SIZE]);
                        }
                    }
                    DiskRegion::RootDirectory => {
                        let offset = (lba - ROOT_DIR_START) as usize * SECTOR_SIZE;
                        if offset + SECTOR_SIZE <= self.root_directory.len() {
                            println!("Reading root directory at offset {}", offset);
                            sector.copy_from_slice(&self.root_directory[offset..offset + SECTOR_SIZE]);
                        }
                    }
                    DiskRegion::Data => {
                        let offset = (lba - DATA_START) as usize * SECTOR_SIZE;
                        if offset + SECTOR_SIZE <= self.data_sectors.len() {
                            println!("Reading data at offset {}", offset);
                            sector.copy_from_slice(&self.data_sectors[offset..offset + SECTOR_SIZE]);
                        }
                    }
                }
            }
        }

        println!("First bytes of sector: {:?}", &sector[..4]);
        Some(sector)
    }

    fn sector_to_region(&self, sector: u32) -> DiskRegion {
        if sector == 0 || sector == BOOT_SECTOR {
            DiskRegion::BootSector
        } else if (FAT1_START..FAT2_START).contains(&sector) {
            DiskRegion::FAT1
        } else if (FAT2_START..ROOT_DIR_START).contains(&sector) {
            DiskRegion::FAT2
        } else if (ROOT_DIR_START..DATA_START).contains(&sector) {
            DiskRegion::RootDirectory
        } else {
            DiskRegion::Data
        }
    }

    #[allow(dead_code)]
    fn fat_offset(&self, sector: u32) -> usize {
        let partition = &self.mbr.partitions[0];
        let rel_sector = sector
            .checked_sub(partition.start_lba)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Sector outside partition bounds",
                )
            })
            .unwrap_or(0);

        let fat_start = if self.sector_to_region(sector) == DiskRegion::FAT1 {
            self._bpb._reserved_sectors as u32
        } else {
            self._bpb._reserved_sectors as u32 + self._bpb._sectors_per_fat as u32
        };

        // Check if rel_sector is less than fat_start to avoid underflow
        if rel_sector < fat_start {
            0
        } else {
            ((rel_sector - fat_start) * self._bpb._bytes_per_sector as u32) as usize
        }
    }

    #[allow(dead_code)]
    fn root_dir_offset(&self, sector: u32) -> usize {
        let partition = &self.mbr.partitions[0];
        let rel_sector = sector
            .checked_sub(partition.start_lba)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Sector outside partition bounds",
                )
            })
            .unwrap_or(0);

        let root_dir_start = self._bpb._reserved_sectors as u32
            + (self._bpb._num_fats as u32 * self._bpb._sectors_per_fat as u32);

        if rel_sector < root_dir_start {
            0
        } else {
            ((rel_sector - root_dir_start) * self._bpb._bytes_per_sector as u32) as usize
        }
    }

    #[allow(dead_code)]
    fn read_data_sector(&mut self, sector: u32) -> Option<&[u8]> {
        let offset = (sector as usize) * SECTOR_SIZE;
        if offset + SECTOR_SIZE <= self.data_sectors.len() {
            self.data_sector_cache.clear();
            self.data_sector_cache
                .extend_from_slice(&self.data_sectors[offset..offset + SECTOR_SIZE]);
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
        sectors_per_fat: u16,
    ) -> Self {
        BiosParameterBlock {
            _bytes_per_sector: BYTES_PER_SECTOR,
            _sectors_per_cluster: sectors_per_cluster,
            _reserved_sectors: reserved_sectors,
            _num_fats: number_of_fats,
            _root_entries: root_entries,
            _total_sectors: total_sectors,
            _media_descriptor: media_descriptor,
            _sectors_per_fat: sectors_per_fat,
            _sectors_per_track: SECTORS_PER_TRACK,
            _num_heads: HEADS_PER_CYLINDER,
            _hidden_sectors: 0,
            _large_sectors: 0,
        }
    }

    #[allow(dead_code)]
    pub fn into_bytes(self) -> [u8; 27] {
        let mut bytes = [0u8; 27];
        bytes[0..2].copy_from_slice(&self._bytes_per_sector.to_le_bytes());
        bytes[2] = self._sectors_per_cluster;
        bytes[3..5].copy_from_slice(&self._reserved_sectors.to_le_bytes());
        bytes[5] = self._num_fats;
        bytes[6..8].copy_from_slice(&self._root_entries.to_le_bytes());
        bytes[8..10].copy_from_slice(&self._total_sectors.to_le_bytes());
        bytes[10] = self._media_descriptor;
        bytes[11..13].copy_from_slice(&self._sectors_per_fat.to_le_bytes());
        bytes[13..15].copy_from_slice(&self._sectors_per_track.to_le_bytes());
        bytes[15..17].copy_from_slice(&self._num_heads.to_le_bytes());
        bytes[17..21].copy_from_slice(&self._hidden_sectors.to_le_bytes());
        bytes[21..25].copy_from_slice(&self._large_sectors.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::{FAT16_SYSTEM_ID, MBR_SIGNATURE};
    use std::path::PathBuf;

    #[test]
    fn test_boot_sector_new() {
        let boot_sector = BootSector::new();
        let bytes = boot_sector.as_bytes();

        // Test size
        assert_eq!(bytes.len(), SECTOR_SIZE);

        // Test boot signature
        assert_eq!(bytes[SECTOR_SIZE - 2], 0x55);
        assert_eq!(bytes[SECTOR_SIZE - 1], 0xAA);
        assert_eq!(bytes[SECTOR_SIZE - 3], 0x4);
    }

    #[test]
    fn test_disk_geometry_default() {
        let geometry = DiskGeometry::default();

        assert_eq!(geometry.cylinders, 80);
        assert_eq!(geometry.heads, 2);
        assert_eq!(geometry.sectors, 18);
        assert_eq!(geometry._bytes_per_sector, 512);
    }

    #[test]
    fn test_disk_image_new() {
        let path = PathBuf::from("drive_c/");
        let disk_image = DiskImage::new(&path).unwrap();

        // Test basic structure initialization
        assert_eq!(disk_image.boot_sector.len(), SECTOR_SIZE);
        assert!(!disk_image.write_protected);

        // Test FAT table initialization
        assert!(!disk_image.fat_table.is_empty());
        assert_eq!(disk_image.fat_table[0], FAT16_MEDIA_DESCRIPTOR);
        assert_eq!(disk_image.fat_table[1], 0xFF);
        assert_eq!(disk_image.fat_table[2], 0xFF);

        // Test partition setup
        assert!(disk_image.mbr.partitions[0].bootable);
        assert_eq!(disk_image.mbr.partitions[0].system_id, FAT16_SYSTEM_ID);
        assert_eq!(disk_image.mbr.partitions[0].start_lba, 63);
    }

    #[test]
    fn test_disk_image_read_sector() {
        let path = PathBuf::from("drive_c/");
        let disk_image = DiskImage::new(&path).unwrap();

        // Test reading MBR (sector 0)
        let mbr_sector = disk_image.read_sector(0).unwrap();
        assert_eq!(mbr_sector.len(), SECTOR_SIZE);
        assert_eq!(&mbr_sector[SECTOR_SIZE - 2..], &MBR_SIGNATURE);

        // Test reading boot sector (sector 63)
        let boot_sector = disk_image.read_sector(63).unwrap();
        assert_eq!(boot_sector.len(), SECTOR_SIZE);
        assert_eq!(boot_sector[SECTOR_SIZE - 2], 0x55);
        assert_eq!(boot_sector[SECTOR_SIZE - 1], 0xAA);

        // Test reading FAT1 sector
        let fat_sector = disk_image.read_sector(FAT1_START).unwrap();
        assert_eq!(fat_sector[0], FAT16_MEDIA_DESCRIPTOR); // 0xF8 for fixed disk

        // Test reading beyond disk size
        let far_sector = disk_image.read_sector(FAT16_TOTAL_SECTORS + 1000);
        assert!(far_sector.is_some()); // Returns zeroed sector
    }

    #[test]
    fn test_disk_region_detection() {
        let path = PathBuf::from("drive_c/");
        let disk_image = DiskImage::new(&path).unwrap();

        // Test region detection
        assert_eq!(disk_image.sector_to_region(0), DiskRegion::BootSector);
        assert_eq!(disk_image.sector_to_region(FAT1_START), DiskRegion::FAT1);
        assert_eq!(disk_image.sector_to_region(FAT2_START), DiskRegion::FAT2);
        assert_eq!(
            disk_image.sector_to_region(ROOT_DIR_START),
            DiskRegion::RootDirectory
        );
        assert_eq!(disk_image.sector_to_region(DATA_START), DiskRegion::Data);
    }

    #[test]
    fn test_bios_parameter_block() {
        let bpb = BiosParameterBlock::new(
            FAT16_SECTORS_PER_CLUSTER,
            FAT16_RESERVED_SECTORS,
            FAT16_NUMBER_OF_FATS,
            FAT16_ROOT_ENTRIES,
            FAT16_TOTAL_SECTORS as u16,
            FAT16_MEDIA_DESCRIPTOR,
            FAT16_SECTORS_PER_FAT,
        );

        assert_eq!(bpb._bytes_per_sector, BYTES_PER_SECTOR);
        assert_eq!(bpb._sectors_per_cluster, FAT16_SECTORS_PER_CLUSTER);
        assert_eq!(bpb._reserved_sectors, FAT16_RESERVED_SECTORS);
        assert_eq!(bpb._num_fats, FAT16_NUMBER_OF_FATS);
        assert_eq!(bpb._root_entries, FAT16_ROOT_ENTRIES);
        assert_eq!(bpb._media_descriptor, FAT16_MEDIA_DESCRIPTOR);
        assert_eq!(bpb._sectors_per_fat, FAT16_SECTORS_PER_FAT);
    }

    #[test]
    fn test_bios_parameter_block_to_bytes() {
        let bpb = BiosParameterBlock::new(
            FAT16_SECTORS_PER_CLUSTER,
            FAT16_RESERVED_SECTORS,
            FAT16_NUMBER_OF_FATS,
            FAT16_ROOT_ENTRIES,
            FAT16_TOTAL_SECTORS as u16,
            FAT16_MEDIA_DESCRIPTOR,
            FAT16_SECTORS_PER_FAT,
        );

        let bytes = bpb.into_bytes();

        // Test size
        assert_eq!(bytes.len(), 27);

        // Test key fields
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), BYTES_PER_SECTOR);
        assert_eq!(bytes[2], FAT16_SECTORS_PER_CLUSTER);
        assert_eq!(
            u16::from_le_bytes([bytes[3], bytes[4]]),
            FAT16_RESERVED_SECTORS
        );
        assert_eq!(bytes[5], FAT16_NUMBER_OF_FATS);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), FAT16_ROOT_ENTRIES);
        assert_eq!(bytes[10], FAT16_MEDIA_DESCRIPTOR);
    }
}

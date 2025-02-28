// Disk geometry constants
pub const SECTOR_SIZE: usize = 512;
pub const BYTES_PER_SECTOR: u16 = 512;
pub const SECTORS_PER_TRACK: u16 = 63; // Maximum standard value
pub const HEADS_PER_CYLINDER: u16 = 16; // Maximum standard value
#[allow(dead_code)]
pub const CYLINDERS: u16 = 1024; // Standard maximum for BIOS

// MBR constants
pub const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
pub const PARTITION_ENTRY_SIZE: usize = 16;
pub const NUM_PARTITIONS: usize = 4;
pub const PARTITION_TABLE_OFFSET: usize = 446;

// FAT16 constants
pub const FAT16_MEDIA_DESCRIPTOR: u8 = 0xF8; // Fixed disk
pub const FAT16_SYSTEM_ID: u8 = 0x06; // FAT16 partition type
pub const FAT16_SECTORS_PER_CLUSTER: u8 = 64; // Maximum cluster size for FAT16
pub const FAT16_RESERVED_SECTORS: u16 = 1; // Boot sector
pub const FAT16_NUMBER_OF_FATS: u8 = 2; // Two copies of FAT
pub const FAT16_ROOT_ENTRIES: u16 = 512; // Fixed size root directory
pub const FAT16_TOTAL_SECTORS: u32 = 4_194_304; // 2GB volume (4194304 * 512 = 2GB)
pub const FAT16_SECTORS_PER_FAT: u16 = 256; // Each FAT entry is 2 bytes, need enough sectors for 65536 clusters

pub mod disk_image;
mod mbr;

pub use disk_image::*;
pub use mbr::*;

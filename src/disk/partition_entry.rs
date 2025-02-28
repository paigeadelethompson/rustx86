#[derive(Debug, Clone, Copy)]
pub struct PartitionEntry {
    pub bootable: bool,
    pub start_head: u8,
    pub start_sector: u8,
    pub start_cylinder: u16,
    pub system_id: u8,
    pub end_head: u8,
    pub end_sector: u8,
    pub end_cylinder: u16,
    pub start_lba: u32,
    pub total_sectors: u32,
}

impl PartitionEntry {
    pub fn new() -> Self {
        // For FAT16:
        // - Maximum partition size is 2GB with 512-byte sectors
        // - 2GB = 2,147,483,648 bytes
        // - With 512-byte sectors, that's 4,194,304 sectors
        let total_sectors = 4_194_304;

        // Calculate CHS values for a 2GB partition
        // Standard geometry: 16 heads, 63 sectors per track
        let heads = 16;
        let sectors_per_track = 63;
        let cylinders = (total_sectors / (heads as u32 * sectors_per_track as u32)) as u16;

        // End CHS values
        let end_cylinder = cylinders - 1;
        let end_head = (heads - 1) as u8;
        let end_sector = sectors_per_track as u8;

        PartitionEntry {
            bootable: true,
            start_head: 1,
            start_sector: 1,
            start_cylinder: 0,
            system_id: 0x06,  // FAT16
            end_head,
            end_sector,
            end_cylinder,
            start_lba: 63,    // Standard offset for first partition
            total_sectors,    // 2GB partition
        }
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        
        bytes[0] = if self.bootable { 0x80 } else { 0x00 };
        bytes[1] = self.start_head;
        bytes[2] = ((self.start_cylinder & 0x300) >> 2) as u8 | self.start_sector;
        bytes[3] = self.start_cylinder as u8;
        bytes[4] = self.system_id;
        bytes[5] = self.end_head;
        bytes[6] = ((self.end_cylinder & 0x300) >> 2) as u8 | self.end_sector;
        bytes[7] = self.end_cylinder as u8;
        bytes[8..12].copy_from_slice(&self.start_lba.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.total_sectors.to_le_bytes());
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        PartitionEntry {
            bootable: bytes[0] == 0x80,
            start_head: bytes[1],
            start_sector: bytes[2] & 0x3F,
            start_cylinder: ((bytes[2] as u16 & 0xC0) << 2) as u16,
            system_id: bytes[4],
            end_head: bytes[5],
            end_sector: bytes[6] & 0x3F,
            end_cylinder: ((bytes[6] as u16 & 0xC0) << 2) as u16,
            start_lba: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            total_sectors: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        }
    }
} 
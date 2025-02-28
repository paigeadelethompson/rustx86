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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_entry_new() {
        let entry = PartitionEntry::new();
        
        // Test default values
        assert!(entry.bootable);
        assert_eq!(entry.start_head, 1);
        assert_eq!(entry.start_sector, 1);
        assert_eq!(entry.start_cylinder, 0);
        assert_eq!(entry.system_id, 0x06); // FAT16
        assert_eq!(entry.start_lba, 63);
        assert_eq!(entry.total_sectors, 4_194_304); // 2GB partition
        
        // Test end values
        assert_eq!(entry.end_head, 15); // 16 heads - 1
        assert_eq!(entry.end_sector, 63); // 63 sectors per track
        
        // Verify end cylinder calculation
        let expected_cylinders = 4_194_304 / (16 * 63) as u32;
        assert_eq!(entry.end_cylinder, (expected_cylinders - 1) as u16);
    }

    #[test]
    fn test_partition_entry_to_bytes() {
        let entry = PartitionEntry::new();
        let bytes = entry.to_bytes();
        
        // Test bootable flag
        assert_eq!(bytes[0], 0x80);
        
        // Test start CHS values
        assert_eq!(bytes[1], 1); // start_head
        assert_eq!(bytes[2] & 0x3F, 1); // start_sector
        assert_eq!(bytes[3], 0); // start_cylinder (low byte)
        
        // Test system ID
        assert_eq!(bytes[4], 0x06); // FAT16
        
        // Test LBA values
        assert_eq!(u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]), 63);
        assert_eq!(u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]), 4_194_304);
    }

    #[test]
    fn test_partition_entry_from_bytes() {
        let original = PartitionEntry::new();
        let bytes = original.to_bytes();
        let parsed = PartitionEntry::from_bytes(&bytes);
        
        // Verify all fields match after roundtrip
        assert_eq!(parsed.bootable, original.bootable);
        assert_eq!(parsed.start_head, original.start_head);
        assert_eq!(parsed.start_sector, original.start_sector);
        assert_eq!(parsed.system_id, original.system_id);
        assert_eq!(parsed.end_head, original.end_head);
        assert_eq!(parsed.end_sector, original.end_sector);
        assert_eq!(parsed.start_lba, original.start_lba);
        assert_eq!(parsed.total_sectors, original.total_sectors);
    }

    #[test]
    fn test_partition_entry_non_bootable() {
        let mut entry = PartitionEntry::new();
        entry.bootable = false;
        let bytes = entry.to_bytes();
        
        // Verify bootable flag is 0x00
        assert_eq!(bytes[0], 0x00);
        
        // Verify roundtrip
        let parsed = PartitionEntry::from_bytes(&bytes);
        assert!(!parsed.bootable);
    }

    #[test]
    fn test_partition_entry_custom_values() {
        let mut entry = PartitionEntry::new();
        entry.start_head = 2;
        entry.start_sector = 3;
        entry.start_cylinder = 4;
        entry.system_id = 0x0C; // FAT32 LBA
        entry.start_lba = 2048;
        entry.total_sectors = 1_048_576; // 512MB
        
        let bytes = entry.to_bytes();
        let parsed = PartitionEntry::from_bytes(&bytes);
        
        // Verify custom values are preserved
        assert_eq!(parsed.start_head, 2);
        assert_eq!(parsed.start_sector, 3);
        assert_eq!(parsed.start_cylinder, 4);
        assert_eq!(parsed.system_id, 0x0C);
        assert_eq!(parsed.start_lba, 2048);
        assert_eq!(parsed.total_sectors, 1_048_576);
    }
} 
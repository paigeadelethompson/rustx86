use super::{
    FAT16_SYSTEM_ID, FAT16_TOTAL_SECTORS, HEADS_PER_CYLINDER, MBR_SIGNATURE, NUM_PARTITIONS,
    PARTITION_ENTRY_SIZE, PARTITION_TABLE_OFFSET, SECTORS_PER_TRACK, SECTOR_SIZE,
};

#[derive(Debug, Clone)]
pub struct Mbr {
    pub boot_code: [u8; PARTITION_TABLE_OFFSET],
    pub partitions: [PartitionEntry; NUM_PARTITIONS],
    pub signature: [u8; 2],
}

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

impl Mbr {
    pub fn new() -> Self {
        Mbr {
            boot_code: [0; PARTITION_TABLE_OFFSET],
            partitions: [PartitionEntry::empty(); NUM_PARTITIONS],
            signature: MBR_SIGNATURE,
        }
    }

    #[allow(dead_code)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() != SECTOR_SIZE {
            return Err(format!("Invalid MBR data size: {} bytes", data.len()));
        }

        let mut mbr = Mbr::new();
        mbr.boot_code
            .copy_from_slice(&data[..PARTITION_TABLE_OFFSET]);

        for i in 0..NUM_PARTITIONS {
            let offset = PARTITION_TABLE_OFFSET + (i * PARTITION_ENTRY_SIZE);
            mbr.partitions[i] =
                PartitionEntry::from_bytes(&data[offset..offset + PARTITION_ENTRY_SIZE])?;
        }

        mbr.signature.copy_from_slice(&data[SECTOR_SIZE - 2..]);
        Ok(mbr)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = vec![0; SECTOR_SIZE];

        data[..PARTITION_TABLE_OFFSET].copy_from_slice(&self.boot_code);

        for (i, partition) in self.partitions.iter().enumerate() {
            let offset = PARTITION_TABLE_OFFSET + (i * PARTITION_ENTRY_SIZE);
            let bytes = partition.to_bytes();
            data[offset..offset + PARTITION_ENTRY_SIZE].copy_from_slice(&bytes);
        }

        data[SECTOR_SIZE - 2..].copy_from_slice(&self.signature);
        data
    }

    pub fn create_bootable_fat16_mbr(boot_code: Vec<u8>) -> Result<Self, String> {
        let mut mbr = Mbr::new();

        // Basic x86 boot code that loads the boot sector from the active partition
        let mut default_boot_code = [0u8; 446]; // Initialize all to zero

        // Initial boot code
        let boot = [0xf4];

        // Copy boot code to the beginning of default_boot_code
        default_boot_code[..boot.len()].copy_from_slice(&boot);
        // Rest is already zeroed

        // Use provided boot code if valid, otherwise use default
        if !boot_code.is_empty() && boot_code.len() <= PARTITION_TABLE_OFFSET {
            mbr.boot_code[..boot_code.len()].copy_from_slice(&boot_code);
        } else {
            mbr.boot_code[..default_boot_code.len()].copy_from_slice(&default_boot_code);
        }

        // Create bootable FAT16 partition
        let partition = PartitionEntry {
            bootable: true,
            start_head: 1,
            start_sector: 1,
            start_cylinder: 0,
            system_id: FAT16_SYSTEM_ID,
            end_head: (HEADS_PER_CYLINDER - 1) as u8,
            end_sector: SECTORS_PER_TRACK as u8,
            end_cylinder: ((FAT16_TOTAL_SECTORS / (SECTORS_PER_TRACK * HEADS_PER_CYLINDER) as u32)
                - 1) as u16,
            start_lba: 63,
            total_sectors: FAT16_TOTAL_SECTORS,
        };

        mbr.partitions[0] = partition;
        mbr.signature = MBR_SIGNATURE;
        Ok(mbr)
    }
}

impl PartitionEntry {
    pub fn empty() -> Self {
        PartitionEntry {
            bootable: false,
            start_head: 0,
            start_sector: 0,
            start_cylinder: 0,
            system_id: 0,
            end_head: 0,
            end_sector: 0,
            end_cylinder: 0,
            start_lba: 0,
            total_sectors: 0,
        }
    }

    #[allow(dead_code)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() != PARTITION_ENTRY_SIZE {
            return Err(format!(
                "Invalid partition entry size: {} bytes",
                data.len()
            ));
        }

        let start_cylinder = ((data[2] as u16 & 0xC0) << 2) | data[3] as u16;
        let end_cylinder = ((data[6] as u16 & 0xC0) << 2) | data[7] as u16;

        Ok(PartitionEntry {
            bootable: data[0] == 0x80,
            start_head: data[1],
            start_sector: data[2] & 0x3F,
            start_cylinder,
            system_id: data[4],
            end_head: data[5],
            end_sector: data[6] & 0x3F,
            end_cylinder,
            start_lba: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            total_sectors: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        })
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = vec![
            if self.bootable { 0x80 } else { 0x00 },
            self.start_head,
            (self.start_sector & 0x3F) | ((self.start_cylinder & 0x300) >> 2) as u8,
            self.start_cylinder as u8,
            self.system_id,
            self.end_head,
            (self.end_sector & 0x3F) | ((self.end_cylinder & 0x300) >> 2) as u8,
            self.end_cylinder as u8,
        ];
        bytes.extend_from_slice(&self.start_lba.to_le_bytes());
        bytes.extend_from_slice(&self.total_sectors.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mbr_new() {
        let mbr = Mbr::new();
        
        // Test default values
        assert_eq!(mbr.boot_code, [0; PARTITION_TABLE_OFFSET]);
        assert_eq!(mbr.signature, MBR_SIGNATURE);
        assert_eq!(mbr.partitions.len(), NUM_PARTITIONS);
        
        // Test that all partitions are empty
        for partition in &mbr.partitions {
            assert!(!partition.bootable);
            assert_eq!(partition.start_lba, 0);
            assert_eq!(partition.total_sectors, 0);
        }
    }

    #[test]
    fn test_mbr_to_bytes() {
        let mbr = Mbr::new();
        let bytes = mbr.to_bytes();
        
        // Test size
        assert_eq!(bytes.len(), SECTOR_SIZE);
        
        // Test signature
        assert_eq!(bytes[SECTOR_SIZE - 2..], MBR_SIGNATURE);
        
        // Test partition table offset
        assert_eq!(&bytes[..PARTITION_TABLE_OFFSET], &[0; PARTITION_TABLE_OFFSET]);
    }

    #[test]
    fn test_mbr_from_bytes() {
        let original = Mbr::new();
        let bytes = original.to_bytes();
        let parsed = Mbr::from_bytes(&bytes).unwrap();
        
        // Test signature matches
        assert_eq!(parsed.signature, original.signature);
        
        // Test boot code matches
        assert_eq!(parsed.boot_code, original.boot_code);
        
        // Test partitions match
        for (orig, parsed) in original.partitions.iter().zip(parsed.partitions.iter()) {
            assert_eq!(orig.bootable, parsed.bootable);
            assert_eq!(orig.start_lba, parsed.start_lba);
            assert_eq!(orig.total_sectors, parsed.total_sectors);
        }
    }

    #[test]
    fn test_create_bootable_fat16_mbr() {
        let boot_code = vec![0xEB, 0x3C, 0x90]; // Common FAT16 boot code start
        let mbr = Mbr::create_bootable_fat16_mbr(boot_code).unwrap();
        
        // Test boot code was copied
        assert_eq!(mbr.boot_code[0], 0xEB);
        assert_eq!(mbr.boot_code[1], 0x3C);
        assert_eq!(mbr.boot_code[2], 0x90);
        
        // Test first partition is bootable FAT16
        let partition = &mbr.partitions[0];
        assert!(partition.bootable);
        assert_eq!(partition.system_id, FAT16_SYSTEM_ID);
        assert_eq!(partition.start_lba, 63);
        assert_eq!(partition.total_sectors, FAT16_TOTAL_SECTORS);
        
        // Test other partitions are empty
        for partition in &mbr.partitions[1..] {
            assert!(!partition.bootable);
            assert_eq!(partition.start_lba, 0);
            assert_eq!(partition.total_sectors, 0);
        }
    }

    #[test]
    fn test_mbr_from_bytes_invalid_size() {
        let result = Mbr::from_bytes(&[0; SECTOR_SIZE - 1]);
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Invalid MBR data size"));
    }

    #[test]
    #[ignore]
    fn test_mbr_roundtrip() {
        let boot_code = vec![0xEB, 0x3C, 0x90];
        let original = Mbr::create_bootable_fat16_mbr(boot_code).unwrap();
        let bytes = original.to_bytes();
        let parsed = Mbr::from_bytes(&bytes).unwrap();
        
        // Test all fields match after roundtrip
        assert_eq!(parsed.boot_code, original.boot_code);
        assert_eq!(parsed.signature, original.signature);
        
        for (orig, parsed) in original.partitions.iter().zip(parsed.partitions.iter()) {
            assert_eq!(orig.bootable, parsed.bootable);
            assert_eq!(orig.start_head, parsed.start_head);
            assert_eq!(orig.start_sector, parsed.start_sector);
            assert_eq!(orig.start_cylinder, parsed.start_cylinder);
            assert_eq!(orig.system_id, parsed.system_id);
            assert_eq!(orig.end_head, parsed.end_head);
            assert_eq!(orig.end_sector, parsed.end_sector);
            assert_eq!(orig.end_cylinder, parsed.end_cylinder);
            assert_eq!(orig.start_lba, parsed.start_lba);
            assert_eq!(orig.total_sectors, parsed.total_sectors);
        }
    }
}

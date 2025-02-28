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

        Ok(PartitionEntry {
            bootable: data[0] == 0x80,
            start_head: data[1],
            start_sector: data[2] & 0x3F,
            start_cylinder: (((data[2] & 0xC0) as u16) << 2) | data[3] as u16,
            system_id: data[4],
            end_head: data[5],
            end_sector: data[6] & 0x3F,
            end_cylinder: (((data[6] & 0xC0) as u16) << 2) | data[7] as u16,
            start_lba: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            total_sectors: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        })
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = vec![
            if self.bootable { 0x80 } else { 0x00 },
            self.start_head,
            self.start_sector | ((self.start_cylinder & 0x300) >> 2) as u8,
            self.start_cylinder as u8,
            self.system_id,
            self.end_head,
            self.end_sector | ((self.end_cylinder & 0x300) >> 2) as u8,
            self.end_cylinder as u8,
        ];
        bytes.extend_from_slice(&self.start_lba.to_le_bytes());
        bytes.extend_from_slice(&self.total_sectors.to_le_bytes());
        bytes
    }
}

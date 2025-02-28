#[derive(Clone)]
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

impl BiosParameterBlock {
    pub fn from_boot_sector(data: &[u8]) -> Result<Self, String> {
        if data.len() < BPB_SIZE {
            return Err(format!("Invalid boot sector size: {} bytes", data.len()));
        }

        Ok(BiosParameterBlock {
            bytes_per_sector: u16::from_le_bytes([data[11], data[12]]),
            sectors_per_cluster: data[13],
            reserved_sectors: u16::from_le_bytes([data[14], data[15]]),
            number_of_fats: data[16],
            root_entries: u16::from_le_bytes([data[17], data[18]]),
            total_sectors: u16::from_le_bytes([data[19], data[20]]),
            media_descriptor: data[21],
            sectors_per_fat: u16::from_le_bytes([data[22], data[23]]),
        })
    }

    pub fn default() -> Self {
        let total_sectors = 2880;
        let sectors_per_cluster = 1;
        let clusters = total_sectors / sectors_per_cluster;
        let fat_size_bytes = (clusters * 2) as usize;
        let sectors_per_fat = (fat_size_bytes + SECTOR_SIZE - 1) / SECTOR_SIZE;

        BiosParameterBlock {
            bytes_per_sector: SECTOR_SIZE as u16,
            sectors_per_cluster,
            reserved_sectors: 1,
            number_of_fats: 2,
            root_entries: 224,
            total_sectors: total_sectors as u16,
            media_descriptor: 0xF0,
            sectors_per_fat: sectors_per_fat as u16,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; BPB_SIZE];
        bytes[11..13].copy_from_slice(&self.bytes_per_sector.to_le_bytes());
        bytes[13] = self.sectors_per_cluster;
        bytes[14..16].copy_from_slice(&self.reserved_sectors.to_le_bytes());
        bytes[16] = self.number_of_fats;
        bytes[17..19].copy_from_slice(&self.root_entries.to_le_bytes());
        bytes[19..21].copy_from_slice(&self.total_sectors.to_le_bytes());
        bytes[21] = self.media_descriptor;
        bytes[22..24].copy_from_slice(&self.sectors_per_fat.to_le_bytes());
        bytes
    }
} 
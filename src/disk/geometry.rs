#[derive(Debug)]
pub struct DiskGeometry {
    pub cylinders: u16,
    pub heads: u8,
    pub sectors: u8,
}

impl DiskGeometry {
    pub fn default() -> Self {
        DiskGeometry {
            cylinders: 80,
            heads: 2,
            sectors: 18,
        }
    }

    pub fn total_sectors(&self) -> u32 {
        (self.cylinders as u32) * (self.heads as u32) * (self.sectors as u32)
    }

    pub fn chs_to_lba(&self, cylinder: u16, head: u8, sector: u8) -> u32 {
        ((cylinder as u32) * (self.heads as u32) + (head as u32)) * (self.sectors as u32) + ((sector as u32) - 1)
    }

    pub fn lba_to_chs(&self, lba: u32) -> (u16, u8, u8) {
        let cylinder = (lba / (self.heads as u32 * self.sectors as u32)) as u16;
        let temp = lba % (self.heads as u32 * self.sectors as u32);
        let head = (temp / self.sectors as u32) as u8;
        let sector = (temp % self.sectors as u32 + 1) as u8;
        (cylinder, head, sector)
    }
} 
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_geometry_default() {
        let geometry = DiskGeometry::default();
        assert_eq!(geometry.cylinders, 80);
        assert_eq!(geometry.heads, 2);
        assert_eq!(geometry.sectors, 18);
    }

    #[test]
    fn test_total_sectors() {
        let geometry = DiskGeometry::default();
        // 80 cylinders * 2 heads * 18 sectors = 2880 sectors
        assert_eq!(geometry.total_sectors(), 2880);

        let custom_geometry = DiskGeometry {
            cylinders: 100,
            heads: 4,
            sectors: 20,
        };
        // 100 cylinders * 4 heads * 20 sectors = 8000 sectors
        assert_eq!(custom_geometry.total_sectors(), 8000);
    }

    #[test]
    fn test_chs_to_lba() {
        let geometry = DiskGeometry::default();
        
        // Test first sector (cylinder 0, head 0, sector 1)
        assert_eq!(geometry.chs_to_lba(0, 0, 1), 0);
        
        // Test second sector (cylinder 0, head 0, sector 2)
        assert_eq!(geometry.chs_to_lba(0, 0, 2), 1);
        
        // Test first sector of second head (cylinder 0, head 1, sector 1)
        assert_eq!(geometry.chs_to_lba(0, 1, 1), 18);
        
        // Test first sector of second cylinder (cylinder 1, head 0, sector 1)
        assert_eq!(geometry.chs_to_lba(1, 0, 1), 36);
    }

    #[test]
    fn test_lba_to_chs() {
        let geometry = DiskGeometry::default();
        
        // Test LBA 0 (should be cylinder 0, head 0, sector 1)
        assert_eq!(geometry.lba_to_chs(0), (0, 0, 1));
        
        // Test LBA 1 (should be cylinder 0, head 0, sector 2)
        assert_eq!(geometry.lba_to_chs(1), (0, 0, 2));
        
        // Test LBA 18 (should be cylinder 0, head 1, sector 1)
        assert_eq!(geometry.lba_to_chs(18), (0, 1, 1));
        
        // Test LBA 36 (should be cylinder 1, head 0, sector 1)
        assert_eq!(geometry.lba_to_chs(36), (1, 0, 1));
    }

    #[test]
    fn test_chs_lba_roundtrip() {
        let geometry = DiskGeometry::default();
        
        // Test a range of values to ensure conversion works both ways
        for lba in 0..geometry.total_sectors() {
            let (c, h, s) = geometry.lba_to_chs(lba);
            let new_lba = geometry.chs_to_lba(c, h, s);
            assert_eq!(lba, new_lba, "LBA {} failed roundtrip conversion", lba);
        }
    }
} 
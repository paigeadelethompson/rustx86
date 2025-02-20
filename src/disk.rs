use std::fs::{self, File};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions};

// Add these constants for disk geometry limits
const MAX_CYLINDERS: u16 = 1024;
const MAX_HEADS: u8 = 16;
const MAX_SECTORS: u8 = 63;
const SECTOR_SIZE: usize = 512;

pub struct DiskImage {
    data: Vec<u8>,
    cylinders: u16,
    heads: u8,
    sectors: u8,
    write_protected: bool,
    changed: bool,
}

impl DiskImage {
    pub fn new(cylinders: u16, heads: u8, sectors: u8) -> Result<Self, String> {
        // Validate geometry
        if cylinders == 0 || cylinders > MAX_CYLINDERS {
            return Err(format!("Invalid number of cylinders: {}", cylinders));
        }
        if heads == 0 || heads > MAX_HEADS {
            return Err(format!("Invalid number of heads: {}", heads));
        }
        if sectors == 0 || sectors > MAX_SECTORS {
            return Err(format!("Invalid number of sectors: {}", sectors));
        }

        let size = cylinders as usize * heads as usize * sectors as usize * SECTOR_SIZE;
        Ok(DiskImage {
            data: vec![0; size],
            cylinders,
            heads,
            sectors,
            write_protected: false,
            changed: false,
        })
    }

    pub fn validate_geometry(&self, cylinder: u16, head: u8, sector: u8) -> Result<(), String> {
        if cylinder >= self.cylinders {
            return Err(format!("Cylinder {} out of range (max {})", cylinder, self.cylinders - 1));
        }
        if head >= self.heads {
            return Err(format!("Head {} out of range (max {})", head, self.heads - 1));
        }
        if sector == 0 || sector > self.sectors {
            return Err(format!("Sector {} out of range (1-{})", sector, self.sectors));
        }
        Ok(())
    }

    pub fn read_sector(&self, cylinder: u16, head: u8, sector: u8) -> Option<&[u8]> {
        if self.validate_geometry(cylinder, head, sector).is_err() {
            return None;
        }

        let offset = self.calculate_offset(cylinder, head, sector);
        Some(&self.data[offset..offset + 512])
    }

    pub fn write_sector(&mut self, cylinder: u16, head: u8, sector: u8, data: &[u8]) -> Option<()> {
        if self.write_protected {
            return None;
        }

        if self.validate_geometry(cylinder, head, sector).is_err() {
            return None;
        }

        let offset = self.calculate_offset(cylinder, head, sector);
        self.data[offset..offset + 512].copy_from_slice(data);
        self.changed = true;
        Some(())
    }

    fn calculate_offset(&self, cylinder: u16, head: u8, sector: u8) -> usize {
        ((cylinder as usize * self.heads as usize + head as usize) 
            * self.sectors as usize + (sector - 1) as usize) * 512
    }

    pub fn get_geometry(&self) -> (u16, u8, u8) {
        (self.cylinders, self.heads, self.sectors)
    }

    pub fn is_write_protected(&self) -> bool {
        self.write_protected
    }

    pub fn set_write_protected(&mut self, protected: bool) {
        self.write_protected = protected;
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }

    pub fn clear_changed(&mut self) {
        self.changed = false;
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

pub fn create_disk_image() -> Result<DiskImage, String> {
    let mut disk = DiskImage::new(1024, 16, 63)
        .map_err(|e| format!("Failed to create disk image: {}", e))?;
    
    // Initialize disk with zeros
    disk.data.resize(512 * 1024 * 1024, 0); // 512MB
    
    // Create a cursor for the disk data
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_all(&disk.data)
        .map_err(|e| format!("Failed to write disk data: {}", e))?;
    cursor.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek cursor: {}", e))?;
    
    // Create a FAT12 file system
    let fs = FileSystem::new(&mut cursor, FsOptions::new())
        .map_err(|e| format!("Failed to create file system: {}", e))?;
    
    // Drop the file system before returning the disk
    drop(fs);
    
    // Copy the data back to the disk
    disk.data = cursor.into_inner();
    
    Ok(disk)
}

pub fn create_disk_from_directory(dir_path: &str) -> io::Result<DiskImage> {
    // Create a new disk image (512MB should be enough for basic DOS usage)
    let mut disk = DiskImage::new(1024, 16, 63)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    // Initialize disk with zeros
    disk.data.resize(512 * 1024 * 1024, 0); // 512MB
    
    // Create a FAT filesystem in the disk image
    {
        let mut cursor = Cursor::new(&mut disk.data);
        let format_options = FormatVolumeOptions::new();
        fatfs::format_volume(&mut cursor, format_options)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Mount the filesystem
        let fs = FileSystem::new(&mut cursor, fatfs::FsOptions::new())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Copy files from the directory to the disk image
        copy_directory_to_fs(Path::new(dir_path), &fs.root_dir(), "")?;
    } // fs and cursor are dropped here, releasing the borrow

    Ok(disk)
}

fn copy_directory_to_fs<T: fatfs::ReadWriteSeek>(
    src: &Path,
    dst: &fatfs::Dir<T>,
    current_path: &str,
) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();
        
        if path.is_dir() {
            let new_path = if current_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", current_path, name)
            };
            
            dst.create_dir(&name)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            
            let subdir = dst.open_dir(&name)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            
            copy_directory_to_fs(&path, &subdir, &new_path)?;
        } else {
            let mut src_file = File::open(&path)?;
            let mut contents = Vec::new();
            src_file.read_to_end(&mut contents)?;
            
            let mut dst_file = dst.create_file(&name)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            
            dst_file.write_all(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
    }
    
    Ok(())
}

pub struct Disk {
    data: Vec<u8>,
}

impl Disk {
    pub fn new() -> Self {
        Disk {
            data: Vec::new(),
        }
    }

    pub fn read_sector(&self, sector: u32) -> &[u8] {
        let start = (sector * 512) as usize;
        let end = start + 512;
        if end <= self.data.len() {
            &self.data[start..end]
        } else {
            &[]
        }
    }

    pub fn write_sector(&mut self, sector: u32, data: &[u8]) {
        let start = (sector * 512) as usize;
        if start + data.len() > self.data.len() {
            self.data.resize(start + data.len(), 0);
        }
        self.data[start..start + data.len()].copy_from_slice(data);
    }
} 
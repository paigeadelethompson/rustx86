use std::path::Path;
use std::io;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: [u8; 8],
    pub ext: [u8; 3],
    pub attr: u8,
    pub reserved: [u8; 10],
    pub time: u16,
    pub date: u16,
    pub start_cluster: u16,
    pub file_size: u32,
}

impl DirEntry {
    pub fn new() -> Self {
        DirEntry {
            name: [0x20; 8],
            ext: [0x20; 3],
            attr: 0,
            reserved: [0; 10],
            time: 0,
            date: 0,
            start_cluster: 0,
            file_size: 0,
        }
    }

    pub fn from_host_file(path: &Path) -> Result<Self, String> {
        let mut entry = DirEntry::new();

        let filename = path.file_name()
            .ok_or_else(|| "Invalid filename".to_string())?
            .to_str()
            .ok_or_else(|| "Invalid filename encoding".to_string())?;

        let (name, ext) = match filename.rfind('.') {
            Some(pos) => {
                let (n, e) = filename.split_at(pos);
                (n, &e[1..])
            }
            None => (filename, ""),
        };

        entry.set_name(name)?;
        entry.set_extension(ext)?;

        let metadata = path.metadata()
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        entry.attr = if metadata.is_dir() { 0x10 } else { 0x20 };
        entry.file_size = metadata.len() as u32;

        Ok(entry)
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), String> {
        if name.len() > 8 {
            return Err("Filename too long".to_string());
        }
        for (i, c) in name.bytes().enumerate() {
            self.name[i] = c;
        }
        Ok(())
    }

    pub fn set_extension(&mut self, ext: &str) -> Result<(), String> {
        if ext.len() > 3 {
            return Err("Extension too long".to_string());
        }
        for (i, c) in ext.bytes().enumerate() {
            self.ext[i] = c;
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; 32];
        bytes[0..8].copy_from_slice(&self.name);
        bytes[8..11].copy_from_slice(&self.ext);
        bytes[11] = self.attr;
        bytes[12..22].copy_from_slice(&self.reserved);
        bytes[22..24].copy_from_slice(&self.time.to_le_bytes());
        bytes[24..26].copy_from_slice(&self.date.to_le_bytes());
        bytes[26..28].copy_from_slice(&self.start_cluster.to_le_bytes());
        bytes[28..32].copy_from_slice(&self.file_size.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_dir_entry_new() {
        let entry = DirEntry::new();
        
        // Test default values
        assert_eq!(entry.name, [0x20; 8]);
        assert_eq!(entry.ext, [0x20; 3]);
        assert_eq!(entry.attr, 0);
        assert_eq!(entry.reserved, [0; 10]);
        assert_eq!(entry.time, 0);
        assert_eq!(entry.date, 0);
        assert_eq!(entry.start_cluster, 0);
        assert_eq!(entry.file_size, 0);
    }

    #[test]
    fn test_set_name() {
        let mut entry = DirEntry::new();
        
        // Test valid name
        assert!(entry.set_name("TEST").is_ok());
        assert_eq!(&entry.name[..4], b"TEST");
        assert_eq!(&entry.name[4..], &[0x20; 4]);
        
        // Test name too long
        assert!(entry.set_name("TOOLONGNAME").is_err());
        
        // Test empty name
        assert!(entry.set_name("").is_ok());
        assert_eq!(entry.name, [0x20; 8]);
    }

    #[test]
    fn test_set_extension() {
        let mut entry = DirEntry::new();
        
        // Test valid extension
        assert!(entry.set_extension("TXT").is_ok());
        assert_eq!(&entry.ext, b"TXT");
        
        // Test extension too long
        assert!(entry.set_extension("LONG").is_err());
        
        // Test empty extension
        assert!(entry.set_extension("").is_ok());
        assert_eq!(entry.ext, [0x20; 3]);
    }

    #[test]
    fn test_from_host_file() -> Result<(), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let file_path = dir.path().join("test.txt");
        
        // Create a test file
        let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
        file.write_all(b"Hello, World!").map_err(|e| e.to_string())?;
        
        // Create directory entry from file
        let entry = DirEntry::from_host_file(&file_path)?;
        
        // Test file name and extension
        assert_eq!(&entry.name[..4], b"TEST");
        assert_eq!(&entry.name[4..], &[0x20; 4]);
        assert_eq!(&entry.ext, b"TXT");
        
        // Test attributes and size
        assert_eq!(entry.attr, 0x20); // Regular file
        assert_eq!(entry.file_size, 13); // "Hello, World!" is 13 bytes
        
        Ok(())
    }

    #[test]
    fn test_from_host_directory() -> Result<(), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let subdir_path = dir.path().join("testdir");
        std::fs::create_dir(&subdir_path).map_err(|e| e.to_string())?;
        
        // Create directory entry from directory
        let entry = DirEntry::from_host_file(&subdir_path)?;
        
        // Test directory name
        assert_eq!(&entry.name[..7], b"TESTDIR");
        assert_eq!(entry.ext, [0x20; 3]);
        
        // Test attributes
        assert_eq!(entry.attr, 0x10); // Directory
        assert_eq!(entry.file_size, 0); // Directories have size 0
        
        Ok(())
    }

    #[test]
    fn test_to_bytes() {
        let mut entry = DirEntry::new();
        entry.set_name("TEST").unwrap();
        entry.set_extension("TXT").unwrap();
        entry.attr = 0x20; // Regular file
        entry.time = 0x5000;
        entry.date = 0x4000;
        entry.start_cluster = 2;
        entry.file_size = 1024;
        
        let bytes = entry.to_bytes();
        
        // Test size
        assert_eq!(bytes.len(), 32);
        
        // Test name and extension
        assert_eq!(&bytes[0..8], b"TEST    ");
        assert_eq!(&bytes[8..11], b"TXT");
        
        // Test other fields
        assert_eq!(bytes[11], 0x20); // attr
        assert_eq!(u16::from_le_bytes([bytes[22], bytes[23]]), 0x5000); // time
        assert_eq!(u16::from_le_bytes([bytes[24], bytes[25]]), 0x4000); // date
        assert_eq!(u16::from_le_bytes([bytes[26], bytes[27]]), 2); // start_cluster
        assert_eq!(u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]), 1024); // file_size
    }

    #[test]
    fn test_invalid_filename() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("a".repeat(100) + ".txt");
        
        // Create a file with too long name
        File::create(&file_path).unwrap();
        
        // Attempt to create directory entry
        let result = DirEntry::from_host_file(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Filename too long"));
    }
} 
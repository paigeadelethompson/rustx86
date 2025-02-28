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
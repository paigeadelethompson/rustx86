use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Parser)]
#[command(name = "extract")]
#[command(about = "Tool for working with FreeDOS ISO", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract FreeDOS packages from ISO
    ExtractAll {
        /// Directory containing the ISO
        #[arg(value_name = "ISO_DIR")]
        iso_dir: PathBuf,
    },
}

const SECTOR_SIZE: usize = 2048;

#[derive(Debug)]
struct IsoDirectory {
    name: String,
    sector: u32,
    size: u32,
    is_dir: bool,
}

fn read_volume_descriptor(file: &mut File) -> Result<u32> {
    // Primary Volume Descriptor is at sector 16
    file.seek(SeekFrom::Start(16 * SECTOR_SIZE as u64))?;

    let mut sector = [0u8; SECTOR_SIZE];
    file.read_exact(&mut sector)?;

    if sector[0] != 1 || &sector[1..6] != b"CD001" {
        anyhow::bail!("Invalid ISO9660 format");
    }

    // Root directory record starts at byte 156
    let extent = u32::from_le_bytes(sector[158..162].try_into().unwrap());
    Ok(extent)
}

fn read_directory_record(data: &[u8], pos: usize) -> Option<(IsoDirectory, usize)> {
    if pos >= data.len() {
        return None;
    }

    let record_len = data[pos] as usize;
    if record_len == 0 || pos + record_len > data.len() {
        return None;
    }

    let name_len = data[pos + 32] as usize;
    if pos + 33 + name_len > data.len() {
        return None;
    }

    let flags = data[pos + 25];
    let is_dir = (flags & 0x02) != 0;

    // Handle Rock Ridge extensions
    let mut name = String::new();
    let mut i = pos + 33;
    while i < pos + 33 + name_len {
        if data[i] == b';' {
            break;
        }
        name.push(data[i] as char);
        i += 1;
    }

    if name.is_empty() {
        return Some((
            IsoDirectory {
                name: String::new(),
                sector: 0,
                size: 0,
                is_dir: false,
            },
            pos + record_len,
        ));
    }

    let extent = u32::from_le_bytes(data[pos + 2..pos + 6].try_into().unwrap());
    let size = u32::from_le_bytes(data[pos + 10..pos + 14].try_into().unwrap());

    Some((
        IsoDirectory {
            name,
            sector: extent,
            size,
            is_dir,
        },
        pos + record_len,
    ))
}

fn read_directory(file: &mut File, sector: u32) -> Result<Vec<IsoDirectory>> {
    let mut dirs = Vec::new();
    let mut current_sector = sector;
    let mut buffer = vec![0u8; SECTOR_SIZE * 4]; // Read multiple sectors at once

    loop {
        let offset = current_sector as u64 * SECTOR_SIZE as u64;
        file.seek(SeekFrom::Start(offset))?;

        let bytes_read = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        let mut pos = 0;
        let mut found_entries = false;

        while pos < bytes_read {
            match read_directory_record(&buffer[..bytes_read], pos) {
                Some((dir, next_pos)) => {
                    if !dir.name.is_empty() && dir.name != "." && dir.name != ".." {
                        dirs.push(dir);
                        found_entries = true;
                    }
                    pos = next_pos;
                }
                None => break,
            }
        }

        if !found_entries {
            break;
        }

        current_sector += (bytes_read / SECTOR_SIZE) as u32;
    }

    Ok(dirs)
}

fn extract_file(file: &mut File, dir: &IsoDirectory, dest: &Path) -> Result<()> {
    println!(
        "Extracting {} ({} bytes) from sector {}",
        dir.name, dir.size, dir.sector
    );

    let mut data = vec![0u8; dir.size as usize];
    file.seek(SeekFrom::Start(dir.sector as u64 * SECTOR_SIZE as u64))?;
    file.read_exact(&mut data)?;

    let dest_path = dest.join(&dir.name);
    std::fs::write(dest_path, data)?;

    Ok(())
}

fn extract_all(iso_dir: &Path) -> Result<()> {
    let fs_dir = Path::new("drive_c").join("fs");
    std::fs::create_dir_all(&fs_dir)?;

    let iso_path = iso_dir.join("freedos.iso");
    println!("Reading ISO file: {}", iso_path.display());

    let mut file = File::open(iso_path)?;

    println!("Reading root directory...");
    let root_sector = read_volume_descriptor(&mut file)?;
    let root_entries = read_directory(&mut file, root_sector)?;

    println!("Found {} entries in root", root_entries.len());
    let mut zip_count = 0;

    // First extract essential boot files
    let boot_files = [
        "COMMAND.COM",
        "IO.SYS",
        "MSDOS.SYS",
        "FDCONFIG.SYS",
        "AUTOEXEC.BAT",
        "KERNEL.SYS",
    ];

    for entry in &root_entries {
        if !entry.is_dir {
            if boot_files.contains(&entry.name.as_str()) {
                println!("Extracting boot file: {}", entry.name);
                extract_file(&mut file, entry, &fs_dir)?;
            } else if entry.name.to_uppercase().ends_with(".ZIP") {
                // Extract the ZIP file data
                let mut data = vec![0u8; entry.size as usize];
                file.seek(SeekFrom::Start(entry.sector as u64 * SECTOR_SIZE as u64))?;
                file.read_exact(&mut data)?;

                // Write the ZIP file temporarily
                let zip_path = fs_dir.join(&entry.name);
                std::fs::write(&zip_path, &data)?;
                zip_count += 1;

                // Extract the ZIP contents using zip library
                let zip_file = std::fs::File::open(&zip_path)?;
                let mut archive = ZipArchive::new(zip_file)?;

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let outpath = fs_dir.join(file.name());

                    if file.name().ends_with('/') {
                        std::fs::create_dir_all(&outpath)?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            std::fs::create_dir_all(p)?;
                        }
                        let mut outfile = std::fs::File::create(&outpath)?;
                        std::io::copy(&mut file, &mut outfile)?;
                    }
                }

                // Delete the temporary ZIP file
                std::fs::remove_file(&zip_path)?;
            }
        }
    }

    println!(
        "\nExtracted and processed {} ZIP files to drive_c/fs",
        zip_count
    );
    println!("ZIP files have been cleaned up after extraction");

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ExtractAll { iso_dir } => {
            extract_all(iso_dir).with_context(|| "Failed to extract ISO")?;
        }
    }

    Ok(())
}

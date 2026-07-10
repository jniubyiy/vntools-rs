use std::env;
use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic, lzss::lzss_decompress, cp932::cp932_to_utf8};

const MAGIC: [u8; 4] = *b"ARCX";

struct ArcxEntry {
    filename: String,
    offset: u64,
    size: u64,
    unpacked_size: u64,
    is_packed: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .ARCX файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_arcx(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_arcx(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header)?;

    if !check_magic(&header, &MAGIC) {
        return Err("Не ARCX архив".into());
    }

    let count = read_u32_le(&header[4..8]) as usize;

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let mut entry_buf = [0u8; 128];
        file.read_exact(&mut entry_buf)?;
        let name_bytes = &entry_buf[..entry_buf.iter().position(|&b| b == 0).unwrap_or(128)];
        let filename = cp932_to_utf8(name_bytes)?;
        let offset = read_u32_le(&entry_buf[100..104]) as u64;
        let size = read_u32_le(&entry_buf[104..108]) as u64;
        let unpacked_size = read_u32_le(&entry_buf[108..112]) as u64;
        let is_packed = size != unpacked_size;
        entries.push(ArcxEntry { filename, offset, size, unpacked_size, is_packed });
    }

    // Извлекаем
    for entry in entries {
        let mut file = File::open(input)?;
        file.seek(SeekFrom::Start(entry.offset))?;

        let out_path = PathBuf::from(&entry.filename);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = File::create(&out_path)?;

        if !entry.is_packed {
            let mut buffer = vec![0u8; 8192];
            let mut remaining = entry.size;
            while remaining > 0 {
                let to_read = std::cmp::min(remaining, buffer.len() as u64);
                file.read_exact(&mut buffer[..to_read as usize])?;
                out_file.write_all(&buffer[..to_read as usize])?;
                remaining -= to_read;
            }
        } else {
            let mut compressed = vec![0u8; entry.size as usize];
            file.read_exact(&mut compressed)?;
            let mut decompressed = vec![0u8; entry.unpacked_size as usize];
            lzss_decompress(&mut decompressed, &compressed)?;
            out_file.write_all(&decompressed)?;
        }
        println!("  Извлечён: {}", entry.filename);
    }

    Ok(())
}
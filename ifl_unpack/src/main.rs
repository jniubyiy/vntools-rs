use std::env;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic, lzss::lzss_decompress};

const MAGIC: [u8; 4] = *b"IFLS";

struct IflEntry {
    filename: String,
    offset: u64,
    size: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .ifl файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_ifl(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_ifl(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    if !check_magic(&header, &MAGIC) {
        return Err("Не IFL архив".into());
    }

    let count = read_u32_le(&header[8..12]) as usize;

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let mut entry = [0u8; 24];
        file.read_exact(&mut entry)?;
        let name = String::from_utf8_lossy(&entry[0..16])
            .trim_end_matches('\0')
            .to_string();
        let offset = read_u32_le(&entry[16..20]) as u64;
        let size   = read_u32_le(&entry[20..24]) as u64;
        entries.push(IflEntry { filename: name, offset, size });
    }

    for entry in entries {
        let mut file = File::open(input)?;
        file.seek(SeekFrom::Start(entry.offset))?;

        // Проверяем, является ли файл .grd (сжатый LZSS)
        let is_grd = entry.filename.ends_with(".grd");
        let out_path = PathBuf::from(if is_grd {
            entry.filename.replace(".grd", ".bmp")
        } else {
            entry.filename.clone()
        });

        let mut out_file = File::create(&out_path)?;

        if is_grd {
            // Читаем заголовок: первые 12 байт
            let mut grd_header = [0u8; 12];
            file.read_exact(&mut grd_header)?;
            // raw_size = offset 4
            let raw_size = read_u32_le(&grd_header[4..8]) as usize;
            let compressed_size = entry.size - 12;
            let mut compressed = vec![0u8; compressed_size as usize];
            file.read_exact(&mut compressed)?;

            let mut decompressed = vec![0u8; raw_size];
            lzss_decompress(&mut decompressed, &compressed)?;
            out_file.write_all(&decompressed)?;
            println!("  Распакован и извлечён: {}", out_path.display());
        } else {
            // Просто копируем
            let mut buffer = vec![0u8; 8192];
            let mut remaining = entry.size;
            while remaining > 0 {
                let to_read = std::cmp::min(remaining, buffer.len() as u64);
                file.read_exact(&mut buffer[..to_read as usize])?;
                out_file.write_all(&buffer[..to_read as usize])?;
                remaining -= to_read;
            }
            println!("  Извлечён: {}", entry.filename);
        }
    }

    Ok(())
}
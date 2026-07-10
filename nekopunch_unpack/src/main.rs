use std::env;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic, lzss::lzss_decompress, cp932::cp932_to_utf8};

const MAGIC: [u8; 4] = *b"PACK";

struct NekoEntry {
    filename: String,
    offset: u64,
    size: u64,
    orig_size: u64,
    compressed: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Использование: nekopunch_unpack -name32|-name64 <archive>");
        eprintln!("Пример: nekopunch_unpack -name32 archive.pak");
        return;
    }

    let is_32 = match args[1].as_str() {
        "-name32" => true,
        "-name64" => false,
        _ => {
            eprintln!("Неверный аргумент: {}", args[1]);
            return;
        }
    };

    let input = PathBuf::from(&args[2]);
    if let Err(e) = unpack_nekopunch(&input, is_32) {
        eprintln!("Ошибка: {}", e);
    }
}

fn unpack_nekopunch(input: &PathBuf, is_32: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if !check_magic(&magic, &MAGIC) {
        return Err("Не nekopunch архив".into());
    }

    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    let count = read_u32_le(&header[0..4]) as usize;
    let compressed_flag = read_u32_le(&header[4..8]) != 0;

    let entry_size = if is_32 { 44 } else { 76 };
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        let mut buf = vec![0u8; entry_size];
        file.read_exact(&mut buf)?;
        let name_len = buf.iter().position(|&b| b == 0).unwrap_or(entry_size);
        let filename = cp932_to_utf8(&buf[..name_len])?;

        let (offset, size, orig_size) = if is_32 {
            let off = read_u32_le(&buf[40..44]) as u64;
            let sz = read_u32_le(&buf[36..40]) as u64;
            let osz = read_u32_le(&buf[32..36]) as u64;
            (off, sz, osz)
        } else {
            let off = read_u32_le(&buf[72..76]) as u64;
            let sz = read_u32_le(&buf[68..72]) as u64;
            let osz = read_u32_le(&buf[64..68]) as u64;
            (off, sz, osz)
        };

        let compressed = compressed_flag && size != orig_size;
        entries.push(NekoEntry {
            filename,
            offset,
            size,
            orig_size,
            compressed,
        });
    }

    for entry in entries {
        let mut file = File::open(input)?;
        file.seek(SeekFrom::Start(entry.offset))?;
        let out_path = PathBuf::from(&entry.filename);
        let mut out_file = File::create(&out_path)?;

        if !entry.compressed {
            let mut buffer = vec![0u8; 8192];
            let mut remaining = entry.size;
            while remaining > 0 {
                let to_read = std::cmp::min(remaining, buffer.len() as u64);
                file.read_exact(&mut buffer[..to_read as usize])?;
                out_file.write_all(&buffer[..to_read as usize])?;
                remaining -= to_read;
            }
        } else {
            let mut compressed_data = vec![0u8; entry.size as usize];
            file.read_exact(&mut compressed_data)?;
            let mut decompressed = vec![0u8; entry.orig_size as usize];
            lzss_decompress(&mut decompressed, &compressed_data)?;
            out_file.write_all(&decompressed)?;
        }
        println!("  Извлечён: {}", entry.filename);
    }

    Ok(())
}
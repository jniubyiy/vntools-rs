use std::env;
use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic};

const MAGIC: [u8; 8] = *b"SM2MPX10";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .dat файл (архив SM2MPX10) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_archive(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_archive(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header)?;

    if !check_magic(&header, &MAGIC) {
        return Err("Неверный заголовок SM2MPX10".into());
    }

    let count = read_u32_le(&header[8..12]) as usize;
    let _index_size = read_u32_le(&header[12..16]) as usize;

    println!("Файлов в архиве: {}", count);

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let mut entry = [0u8; 20];
        file.read_exact(&mut entry)?;
        let name = String::from_utf8_lossy(&entry[0..12])
            .trim_end_matches('\0')
            .to_string();
        let offset = read_u32_le(&entry[12..16]) as u64;
        let size   = read_u32_le(&entry[16..20]) as u64;
        entries.push((name, offset, size));
    }

    let out_dir = input.with_extension("extracted");
    fs::create_dir_all(&out_dir)?;

    for (name, offset, size) in entries {
        let mut file = File::open(input)?;
        file.seek(SeekFrom::Start(offset))?;
        let out_path = out_dir.join(&name);
        let mut out_file = File::create(&out_path)?;
        let mut buffer = vec![0u8; 8192];
        let mut remaining = size;
        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buffer.len() as u64);
            file.read_exact(&mut buffer[..to_read as usize])?;
            out_file.write_all(&buffer[..to_read as usize])?;
            remaining -= to_read;
        }
        println!("  Извлечён: {}", name);
    }

    println!("✅ Распаковка завершена в {}", out_dir.display());
    Ok(())
}
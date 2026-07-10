use std::env;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic};
use flate2::read::ZlibDecoder;

const MAGIC: [u8; 16] = *b"Graphic PackData";

struct PakEntry {
    filename: String,
    offset: u64,
    compressed_size: u64,
    uncompressed_size: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .pak файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_pak(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_pak(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 20];
    file.read_exact(&mut header)?;

    if !check_magic(&header, &MAGIC) {
        return Err("Не Cromwell PAK архив".into());
    }

    let count = read_u32_le(&header[16..20]) as usize;

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let mut entry = [0u8; 20];
        file.read_exact(&mut entry)?;
        let name = String::from_utf8_lossy(&entry[0..12])
            .trim_end_matches('\0')
            .to_string();
        let offset = read_u32_le(&entry[12..16]) as u64;
        let uncompressed_size = read_u32_le(&entry[16..20]) as u64;
        entries.push(PakEntry {
            filename: name,
            offset,
            compressed_size: 0,
            uncompressed_size,
        });
    }

    // Вычисляем сжатый размер
    for i in 0..count {
        let next_offset = if i + 1 < count {
            entries[i+1].offset
        } else {
            file.seek(SeekFrom::End(0))? as u64
        };
        entries[i].compressed_size = next_offset - entries[i].offset;
    }

    // Извлекаем
    for entry in entries {
        let mut file = File::open(input)?;
        file.seek(SeekFrom::Start(entry.offset))?;
        let mut compressed = vec![0u8; entry.compressed_size as usize];
        file.read_exact(&mut compressed)?;

        let out_path = PathBuf::from(&entry.filename);
        let mut out_file = File::create(&out_path)?;

        if entry.compressed_size == entry.uncompressed_size {
            out_file.write_all(&compressed)?;
        } else {
            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::with_capacity(entry.uncompressed_size as usize);
            decoder.read_to_end(&mut decompressed)?;
            out_file.write_all(&decompressed)?;
        }
        println!("  Извлечён: {}", entry.filename);
    }

    Ok(())
}
use std::env;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic};

const MAGIC: [u8; 16] = *b"VoiceOggPackFile";

struct OpkEntry {
    filename: String,
    offset: u64,
    size: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .opk файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_opk(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_opk(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 20];
    file.read_exact(&mut header)?;

    if !check_magic(&header, &MAGIC) {
        return Err("Не OPK архив".into());
    }

    let count = read_u32_le(&header[16..20]) as usize;

    // Переходим в конец, находим позицию имён
    file.seek(SeekFrom::End(-(count as i64 * 8)))?;
    let name_start = file.stream_position()?;

    // Смещения записей находятся после 20 байт
    file.seek(SeekFrom::Start(20))?;
    let mut offsets = Vec::with_capacity(count + 1);
    offsets.push(read_u32_le(&file.read_u32()?)?); // первое смещение
    for _ in 1..count {
        let off = read_u32_le(&file.read_u32()?)?;
        offsets.push(off);
    }
    // Последнее смещение = размер файла
    let file_size = file.seek(SeekFrom::End(0))?;
    offsets.push(file_size as u32);

    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let name_bytes = &mut [0u8; 8];
        file.seek(SeekFrom::Start(name_start + i as u64 * 8))?;
        file.read_exact(name_bytes)?;
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string() + ".ogg";
        let offset = offsets[i] as u64;
        let size = (offsets[i+1] - offsets[i]) as u64;
        entries.push(OpkEntry { filename: name, offset, size });
    }

    for entry in entries {
        let mut file = File::open(input)?;
        file.seek(SeekFrom::Start(entry.offset))?;
        let out_path = PathBuf::from(&entry.filename);
        let mut out_file = File::create(&out_path)?;
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

    Ok(())
}
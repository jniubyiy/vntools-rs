use std::env;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, cp932::cp932_to_utf8};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите архив Studio Sakura на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_studio_sakura(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_studio_sakura(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?; // первые 4 байта (неизвестно)
    file.seek(SeekFrom::Current(32))?; // пропускаем 32 байта
    let count = read_u32_le(&buf) as usize;

    for _ in 0..count {
        let mut name_buf = [0u8; 256];
        file.read_exact(&mut name_buf)?;
        let name_len = name_buf.iter().position(|&b| b == 0).unwrap_or(256);
        let filename = cp932_to_utf8(&name_buf[..name_len])?;

        // Убираем .pr3, если есть
        let out_name = if filename.ends_with(".pr3") {
            filename.trim_end_matches(".pr3").to_string()
        } else {
            filename.clone()
        };

        let mut entry_header = [0u8; 16];
        file.read_exact(&mut entry_header)?;
        let entry_size = read_u32_le(&entry_header[0..4]) as u64;
        let entry_offset = read_u32_le(&entry_header[4..8]) as u64;

        let current_pos = file.stream_position()?;
        file.seek(SeekFrom::Start(entry_offset))?;

        // Проверяем, сжато ли (ACMPRS03)
        let mut magic_check = [0u8; 8];
        file.read_exact(&mut magic_check)?;
        if &magic_check == b"ACMPRS03" {
            // LZSS распаковка как в C
            let out_path = PathBuf::from(&out_name);
            let mut out_file = File::create(&out_path)?;

            // Пропускаем ещё 28 байт (всего 36)
            file.seek(SeekFrom::Current(28))?;

            let mut window = [0u8; 4096];
            window.fill(0x20);
            let mut win_pos = 4078;

            let mut written = 0;
            while written < entry_size {
                let control = file.read_u8()?;
                let mut bit = 1;
                while bit < 256 && written < entry_size {
                    if (control & bit) != 0 {
                        let b = file.read_u8()?;
                        out_file.write_all(&[b])?;
                        window[win_pos] = b;
                        win_pos = (win_pos + 1) & 4095;
                        written += 1;
                    } else {
                        let match_pos = file.read_u8()? as usize;
                        let match_len = file.read_u8()? as usize;
                        let match_pos = match_pos | ((match_len & 0xF0) << 4);
                        let mut match_len = (match_len & 0x0F) + 3;
                        while match_len > 0 && written < entry_size {
                            let b = window[match_pos];
                            out_file.write_all(&[b])?;
                            window[win_pos] = b;
                            win_pos = (win_pos + 1) & 4095;
                            match_pos = (match_pos + 1) & 4095;
                            written += 1;
                            match_len -= 1;
                        }
                    }
                    bit <<= 1;
                }
            }
            println!("  Распакован: {}", out_name);
        } else {
            // Несжатый файл: копируем оставшиеся данные как есть
            // Но мы уже прочитали 8 байт magic, нужно перемотать назад на 8 байт
            file.seek(SeekFrom::Current(-8))?;
            let remaining = entry_size - (file.stream_position()? - entry_offset);
            let out_path = PathBuf::from(&out_name);
            let mut out_file = File::create(&out_path)?;
            let mut buffer = vec![0u8; 8192];
            let mut left = remaining;
            while left > 0 {
                let to_read = std::cmp::min(left, buffer.len() as u64);
                file.read_exact(&mut buffer[..to_read as usize])?;
                out_file.write_all(&buffer[..to_read as usize])?;
                left -= to_read;
            }
            println!("  Извлечён (несжатый): {}", out_name);
        }

        file.seek(SeekFrom::Start(current_pos))?;
    }

    Ok(())
}
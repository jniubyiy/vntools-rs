use std::env;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use vntools_common::{read_u32_le, check_magic};

const MAGIC: [u8; 8] = *b"GGPFAIKE";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .ggp файл(ы) на эту программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = convert_ggp(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn convert_ggp(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 36];
    file.read_exact(&mut header)?;

    if !check_magic(&header, &MAGIC) {
        return Err("Неверный заголовок GGP".into());
    }

    // XOR ключ
    let mut key = [0u8; 8];
    for i in 0..8 {
        key[i] = header[i] ^ header[i + 12];
    }

    let data_offset = read_u32_le(&header[20..24]) as u64;
    let data_size   = read_u32_le(&header[24..28]) as u64;

    let mut encrypted = vec![0u8; data_size as usize];
    file.seek(SeekFrom::Start(data_offset))?;
    file.read_exact(&mut encrypted)?;

    // Расшифровка XOR
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % 8])
        .collect();

    let output_path = input.with_extension("png");
    let mut out_file = File::create(&output_path)?;
    out_file.write_all(&decrypted)?;

    println!("✅ Конвертировано: {}", output_path.display());
    Ok(())
}
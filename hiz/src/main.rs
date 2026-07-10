use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use vntools_common::{read_u32_le, lzss::lzss_decompress, bmp::write_bmp_rgb565};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .hiz файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = unpack_hiz(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn unpack_hiz(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    let width = read_u32_le(&header[0..4]);
    let height = read_u32_le(&header[4..8]);
    let data_size = read_u32_le(&header[8..12]) as usize;

    let mut compressed = vec![0u8; data_size];
    file.read_exact(&mut compressed)?;

    let expected_raw = (width * height * 2) as usize;
    let mut raw = vec![0u8; expected_raw];
    lzss_decompress(&mut raw, &compressed)?;

    let out_path = input.with_extension("bmp");
    let mut out_file = File::create(&out_path)?;
    write_bmp_rgb565(&mut out_file, width, height, &raw)?;
    println!("✅ Конвертировано: {}", out_path.display());
    Ok(())
}
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use vntools_common::read_u32_le;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .ng3 файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = convert_ng3(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn convert_ng3(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    if &header[0..4] != b"NG3\0" {
        return Err("Не NG3 файл".into());
    }

    let width = read_u32_le(&header[4..8]);
    let height = read_u32_le(&header[8..12]);

    // Читаем палитру (256 цветов * 3 байта BGR)
    let mut palette = [[0u8; 3]; 256];
    for i in 0..256 {
        palette[i] = [file.read_u8()?, file.read_u8()?, file.read_u8()?];
    }

    // Буфер для хранения 24-битных RGB пикселей (3 байта на пиксель)
    let pixel_count = (width * height) as usize;
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    let mut written = 0;

    while written < pixel_count {
        let c = file.read_u8()?;
        if c == 1 {
            // Одиночный пиксель по палитре
            let _ = file.read_u8()?; // пропускаем один байт (обычно 1)
            let idx = file.read_u8()? as usize;
            let color = palette[idx];
            rgb_data.extend_from_slice(&color); // BGR
            written += 1;
        } else if c == 2 {
            // Повторяющийся пиксель по палитре
            let _ = file.read_u8()?; // пропускаем один байт (обычно 2)
            let idx = file.read_u8()? as usize;
            let count = file.read_u8()? as usize;
            let color = palette[idx];
            for _ in 0..count {
                rgb_data.extend_from_slice(&color);
                written += 1;
                if written >= pixel_count { break; }
            }
        } else {
            // Прямые RGB пиксели (3 байта)
            let r = file.read_u8()?;
            let g = file.read_u8()?;
            let b = file.read_u8()?;
            rgb_data.extend_from_slice(&[b, g, r]); // в BMP порядок BGR
            written += 1;
        }
    }

    // Записываем BMP 24-бит без палитры
    let out_path = input.with_extension("bmp");
    let mut out_file = File::create(&out_path)?;

    let stride = ((width * 3 + 3) & !3) as usize;
    let data_size = stride * height as usize;
    let file_size = 14 + 40 + data_size;

    let mut bmp = Vec::with_capacity(file_size);
    // BITMAPFILEHEADER
    bmp.extend_from_slice(b"BM");
    bmp.extend(&(file_size as u32).to_le_bytes());
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&(14 + 40).to_le_bytes());

    // BITMAPINFOHEADER
    bmp.extend(&(40u32).to_le_bytes());
    bmp.extend(&width.to_le_bytes());
    bmp.extend(&height.to_le_bytes());
    bmp.extend(&[1, 0]);
    bmp.extend(&[24, 0]); // 24 бита
    bmp.extend(&[0, 0, 0, 0]); // BI_RGB
    bmp.extend(&(data_size as u32).to_le_bytes());
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&[0, 0, 0, 0]);

    // Пиксели снизу вверх
    let mut row_buf = vec![0u8; stride];
    for y in (0..height).rev() {
        let row_start = (y * width) as usize * 3;
        let row_end = row_start + (width as usize * 3);
        let row_data = &rgb_data[row_start..row_end];
        row_buf[..row_data.len()].copy_from_slice(row_data);
        // Остаток row_buf уже нули (инициализация)
        bmp.extend_from_slice(&row_buf);
    }

    out_file.write_all(&bmp)?;
    println!("✅ Конвертировано: {}", out_path.display());
    Ok(())
}
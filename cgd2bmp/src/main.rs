use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use vntools_common::{read_u32_le, bmp::write_bmp_rgb565};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Перетащите .cgd файл(ы) на программу.");
        return;
    }
    for arg in &args[1..] {
        let path = PathBuf::from(arg);
        if let Err(e) = convert_cgd(&path) {
            eprintln!("Ошибка при обработке {}: {}", path.display(), e);
        }
    }
}

fn convert_cgd(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input)?;
    let mut header = [0u8; 20];
    file.read_exact(&mut header)?;

    if &header[0..4] != b"cgd\0" {
        return Err("Не CGD файл".into());
    }

    let width = read_u32_le(&header[12..16]);
    let height = read_u32_le(&header[16..20]);

    // Остаток файла — это данные в формате RGBA (или что-то ещё), но C-код просто копирует их как есть.
    // В оригинале cgd2bmp записывает их в BMP с заголовком, где используется BI_BITFIELDS для ARGB.
    // Но так как мы не знаем точный формат, можно просто взять данные как есть и записать в BMP с 32 битами.
    // Однако в C-коде делается fseek(bmp, 122, SEEK_SET); и while(size--) fputc(fgetc(cgd), bmp);
    // Это значит, что они копируют байты после заголовка в BMP как есть, считая, что это уже готовые пиксели.
    // Я просто скопирую оставшиеся данные в BMP 32-bit.

    let remaining = file.metadata()?.len() - file.stream_position()?;
    let mut pixel_data = vec![0u8; remaining as usize];
    file.read_exact(&mut pixel_data)?;

    let out_path = input.with_extension("bmp");
    let mut out_file = File::create(&out_path)?;

    // Простой BMP 32-bit (ARGB) без сжатия
    // Заголовок: 14 + 40 = 54, плюс 4 маски? В C-коде они используют 122 байта с масками.
    // Я сделаю 124 байта с масками для RGBA.
    let row_bytes = width as usize * 4;
    let stride = (row_bytes + 3) & !3;
    let data_size = stride * height as usize;
    let header_size = 14 + 40 + 16; // 4 маски по 4 байта = 16
    let file_size = header_size + data_size;

    let mut bmp = Vec::with_capacity(file_size as usize);
    // BITMAPFILEHEADER
    bmp.extend_from_slice(b"BM");
    bmp.extend(&(file_size as u32).to_le_bytes());
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&(header_size as u32).to_le_bytes());

    // BITMAPINFOHEADER
    bmp.extend(&(40u32).to_le_bytes());
    bmp.extend(&width.to_le_bytes());
    bmp.extend(&height.to_le_bytes());
    bmp.extend(&[1, 0]);
    bmp.extend(&[32, 0]); // 32 bit
    bmp.extend(&(3u32).to_le_bytes()); // BI_BITFIELDS
    bmp.extend(&(data_size as u32).to_le_bytes());
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&[0, 0, 0, 0]);
    bmp.extend(&[0, 0, 0, 0]);

    // Маски (B,G,R,A) - little endian
    bmp.extend(&0x00FF0000u32.to_le_bytes()); // R
    bmp.extend(&0x0000FF00u32.to_le_bytes()); // G
    bmp.extend(&0x000000FFu32.to_le_bytes()); // B
    bmp.extend(&0xFF000000u32.to_le_bytes()); // A

    // Данные пикселей (снизу вверх)
    let mut row_data = vec![0u8; stride];
    for y in (0..height).rev() {
        let src_start = (y * width) as usize * 4;
        let src_end = src_start + row_bytes;
        row_data[..row_bytes].copy_from_slice(&pixel_data[src_start..src_end]);
        bmp.extend_from_slice(&row_data);
    }

    out_file.write_all(&bmp)?;
    println!("✅ Конвертировано: {}", out_path.display());
    Ok(())
}
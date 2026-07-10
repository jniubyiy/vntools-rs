use std::io::Write;

/// Записывает 24-битный BMP с палитрой (256 цветов).
/// Данные идут в формате: сначала палитра (3 байта на цвет BGR), затем индексы.
/// Ширина и высота должны быть > 0.
pub fn write_bmp_palette<W: Write>(
    writer: &mut W,
    width: u32,
    height: u32,
    palette: &[[u8; 3]; 256],
    indices: &[u8],
) -> Result<(), std::io::Error> {
    let pixel_count = (width * height) as usize;
    if indices.len() < pixel_count {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough indices",
        ));
    }

    let stride = ((width * 3 + 3) & !3) as usize; // padding to 4 bytes
    let data_size = stride * height as usize;
    let file_size = 14 + 40 + 256 * 4 + data_size;

    // Заголовок BMP
    let mut header = Vec::with_capacity(14 + 40 + 256 * 4);
    // BITMAPFILEHEADER
    header.extend_from_slice(b"BM");
    header.extend(&(file_size as u32).to_le_bytes());
    header.extend(&[0, 0, 0, 0]); // reserved
    header.extend(&((14 + 40 + 256 * 4) as u32).to_le_bytes()); // offset to pixels

    // BITMAPINFOHEADER
    header.extend(&(40u32).to_le_bytes());
    header.extend(&width.to_le_bytes());
    header.extend(&height.to_le_bytes());
    header.extend(&[1, 0]); // planes
    header.extend(&[24, 0]); // bit count
    header.extend(&[0, 0, 0, 0]); // compression (BI_RGB)
    header.extend(&(data_size as u32).to_le_bytes());
    header.extend(&[0, 0, 0, 0]); // ppm x
    header.extend(&[0, 0, 0, 0]); // ppm y
    header.extend(&[0, 0, 0, 0]); // colors used
    header.extend(&[0, 0, 0, 0]); // important colors

    // Палитра (BGRX)
    for color in palette.iter() {
        header.push(color[2]); // R
        header.push(color[1]); // G
        header.push(color[0]); // B
        header.push(0);        // reserved
    }

    writer.write_all(&header)?;

    // Пиксели (снизу вверх)
    let mut row_buffer = vec![0u8; stride];
    for y in (0..height).rev() {
        row_buffer.fill(0);
        let row_start = (y * width) as usize;
        let row_end = row_start + width as usize;
        let row_indices = &indices[row_start..row_end];
        // Преобразуем индексы в BGR
        for (x, &idx) in row_indices.iter().enumerate() {
            let col = &palette[idx as usize];
            let pos = x * 3;
            row_buffer[pos] = col[2]; // R
            row_buffer[pos + 1] = col[1]; // G
            row_buffer[pos + 2] = col[0]; // B
        }
        writer.write_all(&row_buffer)?;
    }

    Ok(())
}

/// Записывает BMP с 16-битными RGB565 данными (для hiz).
pub fn write_bmp_rgb565<W: Write>(
    writer: &mut W,
    width: u32,
    height: u32,
    rgb565_data: &[u8], // ожидается 2 байта на пиксель, little-endian RGB565
) -> Result<(), std::io::Error> {
    let row_bytes = (width * 2) as usize;
    let stride = (row_bytes + 3) & !3; // padding до 4
    let data_size = stride * height as usize;
    let file_size = 14 + 40 + 12 + data_size; // 12 байт для масок BI_BITFIELDS

    let mut header = Vec::with_capacity(14 + 40 + 12);
    // BITMAPFILEHEADER
    header.extend_from_slice(b"BM");
    header.extend(&(file_size as u32).to_le_bytes());
    header.extend(&[0, 0, 0, 0]);
    header.extend(&((14 + 40 + 12) as u32).to_le_bytes()); // offset

    // BITMAPINFOHEADER
    header.extend(&(40u32).to_le_bytes());
    header.extend(&width.to_le_bytes());
    header.extend(&height.to_le_bytes());
    header.extend(&[1, 0]);
    header.extend(&[16, 0]); // 16 bit
    header.extend(&(3u32).to_le_bytes()); // BI_BITFIELDS
    header.extend(&(data_size as u32).to_le_bytes());
    header.extend(&[0, 0, 0, 0]);
    header.extend(&[0, 0, 0, 0]);
    header.extend(&[0, 0, 0, 0]);
    header.extend(&[0, 0, 0, 0]);

    // Маски RGB565 (little-endian)
    header.extend(&0xF800u32.to_le_bytes()); // R mask
    header.extend(&0x07E0u32.to_le_bytes()); // G mask
    header.extend(&0x001Fu32.to_le_bytes()); // B mask

    writer.write_all(&header)?;

    // Пиксели (снизу вверх)
    let mut row_buf = vec![0u8; stride];
    for y in (0..height).rev() {
        let src_start = (y * width) as usize * 2;
        let src_end = src_start + row_bytes;
        row_buf[..row_bytes].copy_from_slice(&rgb565_data[src_start..src_end]);
        // Дополнение нулями уже в row_buf (инициализировано нулями)
        writer.write_all(&row_buf)?;
    }

    Ok(())
}
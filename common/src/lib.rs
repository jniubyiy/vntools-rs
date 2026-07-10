pub mod lzss;
pub mod cp932;
pub mod bmp;

use std::io::{Read, Write};
use std::fs::File;

/// Читает u32 в little-endian из массива байт
pub fn read_u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Читает u16 в little-endian из массива байт
pub fn read_u16_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

/// Читает u32 из файла в little-endian
pub fn read_u32_from_file(file: &mut File) -> Result<u32, std::io::Error> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Читает один байт из файла
pub fn read_u8_from_file(file: &mut File) -> Result<u8, std::io::Error> {
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Проверяет магический заголовок
pub fn check_magic(data: &[u8], magic: &[u8]) -> bool {
    data.len() >= magic.len() && &data[..magic.len()] == magic
}

/// Копирует данные из одного читателя в писатель
pub fn copy_data<R: Read, W: Write>(mut reader: R, writer: &mut W, size: u64) -> Result<(), std::io::Error> {
    let mut buffer = [0u8; 8192];
    let mut remaining = size;
    while remaining > 0 {
        let to_read = std::cmp::min(remaining, buffer.len() as u64);
        reader.read_exact(&mut buffer[..to_read as usize])?;
        writer.write_all(&buffer[..to_read as usize])?;
        remaining -= to_read;
    }
    Ok(())
}
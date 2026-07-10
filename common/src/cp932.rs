use encoding::all::CP932;
use encoding::{DecoderTrap, EncoderTrap};

/// Преобразует CP932-байты в UTF-8 строку.
/// Возвращает `String` или ошибку.
pub fn cp932_to_utf8(bytes: &[u8]) -> Result<String, &'static str> {
    CP932.decode(bytes, DecoderTrap::Strict)
        .map_err(|_| "CP932 decoding error")
}

/// Преобразует UTF-8 строку в CP932 (не используется)
#[allow(dead_code)]
pub fn utf8_to_cp932(s: &str) -> Result<Vec<u8>, &'static str> {
    CP932.encode(s, EncoderTrap::Strict)
        .map_err(|_| "CP932 encoding error")
}
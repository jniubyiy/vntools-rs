const WINDOW_SIZE: usize = 4096;
const START_POS: usize = 4078;

/// Декомпрессия LZSS.
/// Возвращает `Ok(())` при успехе, иначе ошибку.
pub fn lzss_decompress(output: &mut [u8], input: &[u8]) -> Result<(), &'static str> {
    let mut window = [0u8; WINDOW_SIZE];
    window.fill(0x20);

    let mut win_pos = START_POS;
    let mut input_pos = 0;
    let mut output_pos = 0;
    let output_len = output.len();

    while input_pos < input.len() && output_pos < output_len {
        let control = input[input_pos];
        input_pos += 1;

        let mut bit = 1u16; // Используем u16, чтобы можно было дойти до 256
        while bit < 256 {
            if output_pos >= output_len || input_pos >= input.len() {
                break;
            }

            if (control as u16 & bit) != 0 {
                // Literal byte
                let b = input[input_pos];
                input_pos += 1;
                output[output_pos] = b;
                window[win_pos] = b;
                output_pos += 1;
                win_pos = (win_pos + 1) & (WINDOW_SIZE - 1);
            } else {
                if input_pos + 1 >= input.len() {
                    return Err("Unexpected end of input");
                }
                let match_pos = input[input_pos] as usize;
                let match_len = input[input_pos + 1] as usize;
                input_pos += 2;

                let mut match_pos = match_pos | ((match_len & 0xF0) << 4);
                let mut match_len = (match_len & 0x0F) + 3;

                while match_len > 0 && output_pos < output_len {
                    let b = window[match_pos];
                    output[output_pos] = b;
                    window[win_pos] = b;
                    output_pos += 1;
                    win_pos = (win_pos + 1) & (WINDOW_SIZE - 1);
                    match_pos = (match_pos + 1) & (WINDOW_SIZE - 1);
                    match_len -= 1;
                }
            }
            bit <<= 1;
        }
    }

    if output_pos != output_len {
        return Err("Output buffer not fully filled");
    }
    Ok(())
}

/// Вычисляет максимальный размер сжатых данных (для упаковки, не используется)
pub fn lzss_compress_bound(input_size: usize) -> usize {
    input_size + input_size / 8 + 18
}

/// Сжатие LZSS (не реализовано, так как не требуется для извлечения)
#[allow(dead_code)]
pub fn lzss_compress(_output: &mut [u8], _input: &[u8]) -> Result<usize, &'static str> {
    unimplemented!("Compression not needed")
}
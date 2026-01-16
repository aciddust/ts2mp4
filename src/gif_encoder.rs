use crate::frame_extractor::RgbFrame;
use std::collections::HashMap;
use std::io::{self, ErrorKind, Write};

pub struct GifOptions {
    pub fps: u16,        // 프레임레이트 (1-100)
    pub loop_count: u16, // 0 = 무한 반복
    pub max_colors: u16, // 최대 색상 수 (2-256)
}

impl Default for GifOptions {
    fn default() -> Self {
        Self {
            fps: 10,
            loop_count: 0,
            max_colors: 256,
        }
    }
}

/// RGB 프레임들을 GIF로 인코딩
pub fn encode_gif(frames: &[RgbFrame], options: &GifOptions) -> io::Result<Vec<u8>> {
    if frames.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "No frames provided",
        ));
    }

    let mut output = Vec::new();

    // GIF 헤더
    write_gif_header(&mut output, &frames[0], options)?;

    // 각 프레임 인코딩
    for frame in frames {
        write_gif_frame(&mut output, frame, options)?;
    }

    // GIF 트레일러
    output.write_all(&[0x3B])?;

    Ok(output)
}

/// GIF 헤더 작성
fn write_gif_header(
    output: &mut Vec<u8>,
    first_frame: &RgbFrame,
    options: &GifOptions,
) -> io::Result<()> {
    // GIF89a 시그니처
    output.write_all(b"GIF89a")?;

    // Logical Screen Descriptor
    output.write_all(&first_frame.width.to_le_bytes())?;
    output.write_all(&first_frame.height.to_le_bytes())?;

    // Packed Fields: Global Color Table 사용, 8비트 컬러
    let gct_flag = 1 << 7; // Global Color Table 존재
    let color_resolution = 7 << 4; // 8비트 컬러
    let gct_size = 7; // 2^(7+1) = 256 colors
    output.write_all(&[gct_flag | color_resolution | gct_size])?;

    // Background Color Index
    output.write_all(&[0x00])?;

    // Pixel Aspect Ratio
    output.write_all(&[0x00])?;

    // Global Color Table (256 colors)
    let palette = generate_global_palette(options.max_colors as usize);
    for color in &palette {
        output.write_all(&[color.0, color.1, color.2])?;
    }

    // Netscape 확장 (반복 설정)
    output.write_all(&[
        0x21, 0xFF, 0x0B, // Extension Introducer, Application Extension, Block Size
        b'N', b'E', b'T', b'S', b'C', b'A', b'P', b'E', b'2', b'.', b'0', 0x03,
        0x01, // Sub-block size, block ID
    ])?;
    output.write_all(&options.loop_count.to_le_bytes())?;
    output.write_all(&[0x00])?; // Block Terminator

    Ok(())
}

/// GIF 프레임 작성
fn write_gif_frame(output: &mut Vec<u8>, frame: &RgbFrame, options: &GifOptions) -> io::Result<()> {
    // Graphics Control Extension
    let delay_cs = (100 / options.fps as u16).max(2); // centiseconds
    output.write_all(&[
        0x21,
        0xF9,
        0x04, // Extension Introducer, Graphic Control Label, Block Size
        0x00, // Packed Fields (no transparency)
        (delay_cs & 0xFF) as u8,
        ((delay_cs >> 8) & 0xFF) as u8,
        0x00, // Transparent Color Index
        0x00, // Block Terminator
    ])?;

    // Image Descriptor
    output.write_all(&[0x2C])?; // Image Separator
    output.write_all(&[0x00, 0x00])?; // Left
    output.write_all(&[0x00, 0x00])?; // Top
    output.write_all(&frame.width.to_le_bytes())?;
    output.write_all(&frame.height.to_le_bytes())?;
    output.write_all(&[0x00])?; // Packed Fields (no local color table)

    // Image Data
    let indexed_data = rgb_to_indexed(&frame.data, frame.width, frame.height, options.max_colors);
    let compressed = lzw_compress(&indexed_data)?;

    output.write_all(&compressed)?;

    Ok(())
}

/// 전역 팔레트 생성 (균일 분포)
fn generate_global_palette(max_colors: usize) -> Vec<(u8, u8, u8)> {
    let mut palette = Vec::with_capacity(256);
    let colors_per_channel = (max_colors as f32).powf(1.0 / 3.0).ceil() as usize;

    for r in 0..colors_per_channel {
        for g in 0..colors_per_channel {
            for b in 0..colors_per_channel {
                if palette.len() >= 256 {
                    break;
                }
                let red = ((r * 255) / (colors_per_channel - 1).max(1)) as u8;
                let green = ((g * 255) / (colors_per_channel - 1).max(1)) as u8;
                let blue = ((b * 255) / (colors_per_channel - 1).max(1)) as u8;
                palette.push((red, green, blue));
            }
        }
    }

    // 나머지를 검은색으로 채움
    while palette.len() < 256 {
        palette.push((0, 0, 0));
    }

    palette
}

/// RGB 데이터를 인덱스 컬러로 변환
fn rgb_to_indexed(rgb_data: &[u8], width: u16, height: u16, max_colors: u16) -> Vec<u8> {
    let palette = generate_global_palette(max_colors as usize);
    let mut indexed = Vec::with_capacity((width as usize) * (height as usize));

    for i in (0..rgb_data.len()).step_by(3) {
        if i + 2 >= rgb_data.len() {
            break;
        }

        let r = rgb_data[i];
        let g = rgb_data[i + 1];
        let b = rgb_data[i + 2];

        // 가장 가까운 팔레트 색상 찾기
        let index = find_closest_color(r, g, b, &palette);
        indexed.push(index);
    }

    indexed
}

/// 가장 가까운 팔레트 색상의 인덱스 찾기
fn find_closest_color(r: u8, g: u8, b: u8, palette: &[(u8, u8, u8)]) -> u8 {
    let mut min_dist = u32::MAX;
    let mut best_index = 0;

    for (i, &(pr, pg, pb)) in palette.iter().enumerate() {
        let dr = (r as i32 - pr as i32).abs() as u32;
        let dg = (g as i32 - pg as i32).abs() as u32;
        let db = (b as i32 - pb as i32).abs() as u32;
        let dist = dr * dr + dg * dg + db * db;

        if dist < min_dist {
            min_dist = dist;
            best_index = i;
        }
    }

    best_index as u8
}

/// LZW 압축
fn lzw_compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // LZW Minimum Code Size
    let min_code_size = 8u8;
    output.write_all(&[min_code_size])?;

    // LZW 압축 구현
    let clear_code = 1u16 << min_code_size;
    let eoi_code = clear_code + 1;
    let mut code_size = min_code_size + 1;
    let mut next_code = eoi_code + 1;

    let mut dict: HashMap<Vec<u8>, u16> = HashMap::new();
    for i in 0..clear_code {
        dict.insert(vec![i as u8], i);
    }

    let mut bit_buffer = Vec::new();

    // Clear code로 시작
    write_code(&mut bit_buffer, clear_code, code_size);

    let mut w = Vec::new();

    for &k in data {
        let mut wk = w.clone();
        wk.push(k);

        if dict.contains_key(&wk) {
            w = wk;
        } else {
            // w의 코드 출력
            if let Some(&code) = dict.get(&w) {
                write_code(&mut bit_buffer, code, code_size);
            }

            // 새 코드 추가
            if next_code < 4096 {
                dict.insert(wk, next_code);
                next_code += 1;

                // 코드 사이즈 증가
                if next_code > (1 << code_size) && code_size < 12 {
                    code_size += 1;
                }
            } else {
                // 테이블 리셋
                write_code(&mut bit_buffer, clear_code, code_size);
                dict.clear();
                for i in 0..clear_code {
                    dict.insert(vec![i as u8], i);
                }
                code_size = min_code_size + 1;
                next_code = eoi_code + 1;
            }

            w = vec![k];
        }
    }

    // 마지막 코드 출력
    if !w.is_empty() {
        if let Some(&code) = dict.get(&w) {
            write_code(&mut bit_buffer, code, code_size);
        }
    }

    // End of Information 코드
    write_code(&mut bit_buffer, eoi_code, code_size);

    // 비트 버퍼를 바이트로 변환
    let mut byte_buffer = Vec::new();
    for chunk in bit_buffer.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << i;
            }
        }
        byte_buffer.push(byte);
    }

    // 서브블록으로 나누기 (최대 255바이트)
    for chunk in byte_buffer.chunks(255) {
        output.write_all(&[chunk.len() as u8])?;
        output.write_all(chunk)?;
    }

    // Block Terminator
    output.write_all(&[0x00])?;

    Ok(output)
}

/// 코드를 비트 버퍼에 작성
fn write_code(bit_buffer: &mut Vec<bool>, code: u16, bit_count: u8) {
    for i in 0..bit_count {
        bit_buffer.push((code & (1 << i)) != 0);
    }
}

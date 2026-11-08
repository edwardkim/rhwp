//! 일반 텍스트가 한컴 Print와 같은 COLR 기본 윤곽을 사용하도록 한다.
//!
//! 원본 프로그램과 명시적인 컬러 GlyphOutline 자원은 변경하지 않는다.
//! COLR의 기본 윤곽 출력 계약: https://learn.microsoft.com/en-us/typography/opentype/spec/colr

use ttf_parser::{Face, GlyphId, Tag};

const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const MAX_TABLES: usize = (u16::MAX / 16) as usize;

/// 컬러 기본 glyph 모두에 윤곽이 있을 때만 일반 텍스트용 독립 SFNT 사본을 만든다.
/// 미지원·무효·컬러 전용 프로그램은 `None`을 반환해 기존 공급 경로를 보존한다.
pub fn ordinary_text_font_program(data: &[u8], face_index: u32) -> Option<Vec<u8>> {
    if data.len() > MAX_FONT_BYTES {
        return None;
    }
    let face = Face::parse(data, face_index).ok()?;
    let raw = face.raw_face();
    raw.table(Tag::from_bytes(b"COLR"))?;
    raw.table(Tag::from_bytes(b"CPAL"))?;
    let has_glyf = raw.table(Tag::from_bytes(b"glyf")).is_some();
    let has_cff = raw.table(Tag::from_bytes(b"CFF ")).is_some()
        || raw.table(Tag::from_bytes(b"CFF2")).is_some();
    if has_glyf == has_cff {
        return None;
    }
    let mut color_glyphs = 0;
    for index in 0..face.number_of_glyphs() {
        let glyph = GlyphId(index);
        if face.is_color_glyph(glyph) {
            // 색 레이어만 있는 glyph를 빈 기본 glyph로 바꾸어 숨기지 않는다.
            face.glyph_bounding_box(glyph)?;
            color_glyphs += 1;
        }
    }
    if color_glyphs == 0 {
        return None;
    }
    let records: Vec<_> = raw.table_records.into_iter().collect();
    // 제외할 표의 중복도 수용하지 않는다. 원본 디렉터리 자체를 먼저 검증한다.
    if records.len() > MAX_TABLES || records.windows(2).any(|pair| pair[0].tag >= pair[1].tag) {
        return None;
    }
    let tables: Vec<_> = records
        .into_iter()
        .filter(|record| !matches!(&record.tag.to_bytes(), b"COLR" | b"CPAL" | b"DSIG"))
        .collect();
    let count = tables.len();
    if count == 0 || count > MAX_TABLES {
        return None;
    }
    let count = u16::try_from(count).ok()?;
    let power = 1u16 << (15 - count.leading_zeros());
    let mut result = vec![0; 12 + tables.len() * 16];
    result[..4].copy_from_slice(if has_cff { b"OTTO" } else { b"\0\x01\0\0" });
    result[4..6].copy_from_slice(&count.to_be_bytes());
    result[6..8].copy_from_slice(&(power * 16).to_be_bytes());
    result[8..10].copy_from_slice(&(15 - power.leading_zeros() as u16).to_be_bytes());
    result[10..12].copy_from_slice(&((count - power) * 16).to_be_bytes());
    let mut head_adjustment = None;
    for (index, table) in tables.iter().enumerate() {
        let bytes = raw.table(table.tag)?;
        let offset = result.len();
        let end = offset.checked_add(bytes.len())?;
        let padded_end = end.checked_add(3)? & !3;
        if padded_end > MAX_FONT_BYTES {
            return None;
        }
        result.extend_from_slice(bytes);
        result.resize(padded_end, 0);
        if table.tag == Tag::from_bytes(b"head") {
            if bytes.len() < 54 {
                return None;
            }
            head_adjustment = Some(offset + 8);
            result[offset + 8..offset + 12].fill(0);
        }
        let sum = checksum(&result[offset..end]);
        let record = &mut result[12 + index * 16..28 + index * 16];
        record[..4].copy_from_slice(&table.tag.to_bytes());
        record[4..8].copy_from_slice(&sum.to_be_bytes());
        record[8..12].copy_from_slice(&(offset as u32).to_be_bytes());
        record[12..16].copy_from_slice(&(bytes.len() as u32).to_be_bytes());
    }
    let head_adjustment = head_adjustment?;
    let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&result));
    result[head_adjustment..head_adjustment + 4].copy_from_slice(&adjustment.to_be_bytes());
    Some(result)
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

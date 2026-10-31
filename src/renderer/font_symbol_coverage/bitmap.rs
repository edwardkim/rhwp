//! 빈 glyf를 대신하는 bitmap 자료의 잉크 유무를 보수적으로 확인한다.
use ttf_parser::{Face, GlyphId, RasterImageFormat, Tag};

pub(super) fn may_supply_ink(face: &Face<'_>, glyph: GlyphId) -> bool {
    let raw = face.raw_face();
    // PNG/sbix와 미해석 표현은 빈 잉크로 추정하지 않는다.
    if raw.table(Tag::from_bytes(b"sbix")).is_some() {
        return true;
    }
    for (tag, table) in [
        (b"EBLC", face.tables().ebdt),
        (b"CBLC", face.tables().cbdt),
        (b"bloc", face.tables().bdat),
    ] {
        let Some(locations) = raw.table(Tag::from_bytes(tag)) else {
            continue;
        };
        let Some(table) = table else { return true };
        let Some(header) = locations.get(..8) else {
            return true;
        };
        let count = u32::from_be_bytes(header[4..8].try_into().unwrap()) as usize;
        // 빈 glyph라는 증명에만 적용하는 파싱 예산이다. 제한 초과는 정상
        // 공급 경로를 보존한다. 규격의 strike 수 제한으로 해석하지 않는다.
        if count > 256 || count > locations.len().saturating_sub(8) / 48 {
            return true;
        }
        let mut sizes = std::collections::BTreeSet::new();
        for index in 0..count {
            let record = &locations[8 + index * 48..8 + (index + 1) * 48];
            let first = u16::from_be_bytes([record[40], record[41]]);
            let last = u16::from_be_bytes([record[42], record[43]]);
            if first > last {
                return true;
            }
            if (first..=last).contains(&glyph.0) {
                sizes.insert(u16::from(record[44]));
            }
        }
        for size in sizes {
            // strike 전체 범위에 있어도 실제 하위 인덱스에는 없을 수 있다.
            // 파싱 실패·불지원 형식·불연속 범위는 빈 bitmap으로 단정하지 않는다.
            let Some(image) = table.get(glyph, size) else {
                return true;
            };
            match image.format {
                RasterImageFormat::BitmapMono
                | RasterImageFormat::BitmapMonoPacked
                | RasterImageFormat::BitmapGray2
                | RasterImageFormat::BitmapGray2Packed
                | RasterImageFormat::BitmapGray4
                | RasterImageFormat::BitmapGray4Packed
                | RasterImageFormat::BitmapGray8 => {
                    // EBDT 1-bit의 0은 흰색이다. padding bit까지 모두 0일 때만
                    // 빈 것으로 입증하며 nonzero padding은 보수적으로 보존한다.
                    if image.data.iter().any(|byte| *byte != 0) {
                        return true;
                    }
                }
                _ => return true,
            }
        }
    }
    false
}

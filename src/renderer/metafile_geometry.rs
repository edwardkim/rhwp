//! 메타파일을 SVG로 변환해도 원본 자르기 좌표계를 보존한다.

/// WMF 자르기 좌표가 기준으로 삼는 전체 범위.
///
/// 한컴 저장본에 imgDim(표 116 기준 이미지 크기)이 있으면 그 명시 범위를 먼저 소비한다.
/// imgDim이 없고 자르기가 메타파일의 원래 크기 전체(`wmf_natural_extent_hu`)와 같으면
/// 그림 전체를 고른 것이므로 그 크기가 기준이다 — 원본 개체 크기와 자르기가 같은 전체 창을
/// 기준으로 인정하는 `Picture::crop_reference_size` 와 같은 계약이다(#9379, 한/글 2020 Print 는
/// 이 그림을 자르지 않고 틀 전체에 그린다). 그 밖의 WMF 자르기는 가로·세로 각각 15000 범위다
/// (#6469). 비트맵의 HU/픽셀 폴백이나 shape orgSz로 대신하지 않는다.
pub(crate) fn wmf_crop_reference(
    declared: Option<(u32, u32)>,
    wmf: Option<&[u8]>,
    crop: Option<(i32, i32, i32, i32)>,
) -> Option<(u32, u32)> {
    declared.filter(|&(w, h)| w > 0 && h > 0).or_else(|| {
        let natural = wmf.and_then(wmf_natural_extent_hu);
        Some(
            natural
                .filter(|&(w, h)| {
                    crop.is_some_and(|crop| {
                        i64::from(crop.0) == 0
                            && i64::from(crop.1) == 0
                            && i64::from(crop.2) == i64::from(w)
                            && i64::from(crop.3) == i64::from(h)
                    })
                })
                .unwrap_or((15000, 15000)),
        )
    })
}

/// 한컴이 WMF 그림의 원래 크기를 자르기 좌표(HWPUNIT)로 적는 값.
///
/// 원래 크기를 1/120 인치 단위로 반올림한 뒤 단위당 60 HWPUNIT으로 적는다.
/// - Placeable WMF: 경계 상자 / 인치당 논리 단위.
/// - 그 밖: 첫 META_SETWINDOWEXT 창 크기의 절댓값을 96 DPI 픽셀로 읽는다.
///
/// 실측(#9379): KAF 회의자료 창 282×228 → (21180, 17100), 240×89 → (18000, 6660),
/// 386×176 → (28980, 13200), 3972×657 → (297900, 49260); Placeable 1580×1590 @1000
/// → (11400, 11460). 크기를 정할 수 없으면 None.
pub(crate) fn wmf_natural_extent_hu(data: &[u8]) -> Option<(u32, u32)> {
    const PLACEABLE_KEY: [u8; 4] = [0xD7, 0xCD, 0xC6, 0x9A];
    const META_SETWINDOWEXT: u16 = 0x020C;
    let i16_at = |offset: usize| -> Option<i16> {
        Some(i16::from_le_bytes(
            data.get(offset..offset + 2)?.try_into().ok()?,
        ))
    };
    // 1/120 인치 반올림(0.5는 0에서 먼) → 60 HU.
    let hu = |sixscore: f64| -> Option<u32> {
        let units = sixscore.round();
        (units.is_finite() && units > 0.0 && units <= f64::from(u32::MAX / 60))
            .then(|| units as u32 * 60)
    };
    if data.get(..4)? == PLACEABLE_KEY {
        let (left, top, right, bottom) = (i16_at(6)?, i16_at(8)?, i16_at(10)?, i16_at(12)?);
        let inch = u16::from_le_bytes(data.get(14..16)?.try_into().ok()?);
        if inch == 0 {
            return None;
        }
        let span = |a: i16, b: i16| (f64::from(b) - f64::from(a)).abs() / f64::from(inch) * 120.0;
        return Some((hu(span(left, right))?, hu(span(top, bottom))?));
    }
    // 표준 메타파일 머리(18바이트) 뒤 레코드: [크기(워드) u32][함수 u16][인자…].
    let mut offset = 18usize;
    while let Some(head) = data.get(offset..offset + 6) {
        let words = u32::from_le_bytes(head[0..4].try_into().ok()?) as usize;
        let function = u16::from_le_bytes(head[4..6].try_into().ok()?);
        if function == META_SETWINDOWEXT {
            // 인자 순서는 높이(y), 너비(x)다.
            let (height, width) = (i16_at(offset + 6)?, i16_at(offset + 8)?);
            let px = |v: i16| f64::from(v).abs() * 1.25;
            return Some((hu(px(width))?, hu(px(height))?));
        }
        if words < 3 || function == 0 {
            return None;
        }
        offset = offset.checked_add(words.checked_mul(2)?)?;
    }
    None
}

/// 변환기가 만든 SVG의 논리 viewport 크기. 정수 픽셀로 반올림하지 않는다.
/// WMF 변환기의 width/height도 이 viewport와 같으므로 Canvas source rect와
/// 중첩 SVG의 source rect가 같은 좌표계를 소비한다.
pub(crate) fn svg_viewport(data: &[u8]) -> Option<(f64, f64)> {
    let xml = roxmltree::Document::parse(std::str::from_utf8(data).ok()?).ok()?;
    let root = xml.root_element();
    if root.tag_name().name() != "svg" {
        return None;
    }
    let values = root
        .attribute("viewBox")?
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() != 4 || values.iter().any(|n| !n.is_finite()) {
        return None;
    }
    let (w, h) = (values[2], values[3]);
    (w > 0.0 && h > 0.0).then_some((w, h))
}

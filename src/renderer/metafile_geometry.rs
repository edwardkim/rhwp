//! 메타파일을 SVG로 변환해도 원본 자르기 좌표계를 보존한다.

/// WMF의 이미지 전체 좌표 범위는 가로·세로 각각 15000이다.
/// 한컴 저장본에 imgDim이 있으면 그 명시 범위를 먼저 소비한다.
/// 비트맵의 HU/픽셀 폴백이나 shape orgSz로 대신하지 않는다.
pub(super) fn wmf_crop_reference(declared: Option<(u32, u32)>) -> Option<(u32, u32)> {
    declared
        .filter(|&(w, h)| w > 0 && h > 0)
        .or(Some((15000, 15000)))
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

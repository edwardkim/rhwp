// Sampling declared by a bounded PDF importer in a PNG resource.
// `rhWP` is a private ancillary, unsafe-to-copy chunk immediately after IHDR.
// Raw BinData preservation keeps this contract through HWP/HWPX save/reopen;
// ordinary images without the declaration retain the renderer's profile policy.
fn declaration(data: &[u8]) -> Option<(u8, u8)> {
    // JPEG APP15도 같은 버전·값과 CRC를 갖는다. 압축 decoder는 이 부가
    // 표식을 무시하고, 출력 재생만 실제 색 확장의 재구성 계약을 소비한다.
    if data.starts_with(&[0xff, 0xd8, 0xff, 0xef, 0, 12]) && data.len() >= 16 {
        let payload = &data[6..16];
        return (&payload[..4] == b"rhWP"
            && matches!(payload[4], 2 | 3)
            && payload[5] == 1
            && crc32fast::hash(&payload[..6])
                == u32::from_be_bytes(payload[6..10].try_into().ok()?))
        .then_some((payload[4], 1));
    }
    if data.len() < 47
        || &data[..8] != b"\x89PNG\r\n\x1a\n"
        || data[8..16] != [0, 0, 0, 13, b'I', b'H', b'D', b'R']
        || data[33..41] != [0, 0, 0, 2, b'r', b'h', b'W', b'P']
        || !matches!(data[41], 1 | 2 | 4 | 5 | 6)
        || data[42] > if data[41] == 6 { 2 } else { 1 }
    {
        return None;
    }
    let expected = u32::from_be_bytes(data[43..47].try_into().ok()?);
    (crc32fast::hash(&data[37..43]) == expected).then_some((data[41], data[42]))
}

pub(crate) fn interpolation(data: &[u8]) -> Option<bool> {
    declaration(data)
        .and_then(|(version, value)| (matches!(version, 1 | 6) && value <= 1).then_some(value != 0))
}

/// PDF에서 색 공간·마스크 합성을 마친 표본이다. 선택적 보간 여부와 무관하다.
/// 이전 v1 PDF 자원도 같은 출처이며 일반 PNG·한컴 Print 표식은 포함하지 않는다.
pub(crate) fn pdf_source(data: &[u8]) -> bool {
    matches!(declaration(data), Some((1 | 6, _)))
}

/// 두 번째 버전은 한컴 2024의 실제 색 확장 payload만 표시한다.
/// 일반 그림·팔레트 전달·다른 Print 버전의 표본 정책을 바꾸지 않는다.
pub(crate) fn print_bitmap_reconstruction(data: &[u8]) -> bool {
    declaration(data) == Some((2, 1))
}

pub(crate) fn print_jpeg_transfer(data: &[u8]) -> bool {
    data.starts_with(&[0xff, 0xd8]) && matches!(declaration(data), Some((2 | 3, 1)))
}

/// 최종 인쇄 장치에서 합성한 WMF 표본. 원본 비트맵 격자 준비와 구분한다.
pub(crate) fn metafile_surface(data: &[u8]) -> bool {
    declaration(data) == Some((4, 1))
}

pub(crate) fn mark_metafile_surface(data: &mut Vec<u8>) {
    if data.len() < 33 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return;
    }
    let mut chunk = [0, 0, 0, 2, b'r', b'h', b'W', b'P', 4, 1, 0, 0, 0, 0];
    let crc = crc32fast::hash(&chunk[4..10]);
    chunk[10..14].copy_from_slice(&crc.to_be_bytes());
    data.splice(33..33, chunk);
}

/// 색과 이진 마스크가 별도로 표본화된 Print 2024 자원이다.
pub(crate) fn print_separate_alpha(data: &[u8]) -> bool {
    declaration(data) == Some((5, 1))
}

pub(crate) fn mark_print_separate_alpha(data: &mut Vec<u8>) {
    if data.len() < 33 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return;
    }
    let mut chunk = [0, 0, 0, 2, b'r', b'h', b'W', b'P', 5, 1, 0, 0, 0, 0];
    let crc = crc32fast::hash(&chunk[4..10]);
    chunk[10..14].copy_from_slice(&crc.to_be_bytes());
    data.splice(33..33, chunk);
}

pub(super) fn mark_print_bitmap(data: &mut Vec<u8>) {
    if data.len() < 33 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return;
    }
    let mut chunk = [0, 0, 0, 2, b'r', b'h', b'W', b'P', 2, 1, 0, 0, 0, 0];
    let crc = crc32fast::hash(&chunk[4..10]);
    chunk[10..14].copy_from_slice(&crc.to_be_bytes());
    data.splice(33..33, chunk);
}

pub(super) fn mark_print_jpeg(data: &mut Vec<u8>, expanded: bool) {
    if !data.starts_with(&[0xff, 0xd8]) {
        return;
    }
    let mut segment = [0xff, 0xef, 0, 12, b'r', b'h', b'W', b'P', 2, 1, 0, 0, 0, 0];
    segment[8] = if expanded { 2 } else { 3 };
    let crc = crc32fast::hash(&segment[4..10]);
    segment[10..14].copy_from_slice(&crc.to_be_bytes());
    data.splice(2..2, segment);
}

pub(crate) fn preserve(source: &[u8], output: &mut Vec<u8>) {
    if pdf_source(source)
        && !pdf_source(output)
        && output.len() >= 33
        && output[..16] == source[..16]
    {
        output.splice(33..33, source[33..47].iter().copied());
    }
}

/// PDF 32000-1 8.9.5.3 defines interpolation for source samples enlarged onto
/// device pixels. Minification still needs the backend's coverage/prefilter;
/// forcing point sampling there discards source pixels (and thin image lines).
pub(crate) fn magnification(
    declared: Option<bool>,
    source: (f64, f64),
    destination: (f64, f64),
) -> Option<bool> {
    if [source.0, source.1, destination.0, destination.1]
        .iter()
        .any(|v| !v.is_finite() || *v <= 0.)
        || destination.0 < source.0
        || destination.1 < source.1
    {
        return None;
    }
    declared
}

pub(crate) fn dimensions(data: &[u8]) -> Option<(f64, f64)> {
    interpolation(data)?;
    Some((
        u32::from_be_bytes(data[16..20].try_into().ok()?) as f64,
        u32::from_be_bytes(data[20..24].try_into().ok()?) as f64,
    ))
}

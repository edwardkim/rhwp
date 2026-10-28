// Print 알파와 색의 공통 정수 표본 계약.

/// 표준 16×16 Bayer 순위. 마스크는 top-down, RGB555 색 표면은 bottom-up이다.
pub(super) fn bayer_rank(x: u32, y: u32) -> u8 {
    const BASE: [[u16; 2]; 2] = [[0, 2], [3, 1]];
    let mut rank = 0;
    for bit in 0..4 {
        rank = rank * 4 + BASE[((y >> bit) & 1) as usize][((x >> bit) & 1) as usize];
    }
    rank as u8
}

/// 실제 PArgb 표면의 정수 premultiply/unpremultiply 계약.
pub(super) fn pargb_roundtrip(channel: u8, alpha: u8) -> u8 {
    if alpha == 0 {
        return 0;
    }
    let premultiplied = (u32::from(channel) * u32::from(alpha) + 127) / 255;
    let reciprocal = (255 * 65536) / u32::from(alpha);
    ((premultiplied * reciprocal) >> 16).min(255) as u8
}

/// 독립 Print의 색 전달식: PArgb 정수 경계의 합성이다.
/// 실제 한컴 내부의 API 호출 횟수를 뜻하지 않는다. 원본·서로 다른 크기의
/// 256단계 alpha 대조군은 같은 식과 RGB555/JPEG에서 전픽셀 일치했다.
pub(super) fn print_color(channel: u8, alpha: u8) -> u8 {
    let straight = pargb_roundtrip(channel, alpha);
    pargb_roundtrip(straight, alpha)
}

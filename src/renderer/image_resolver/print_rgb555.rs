//! Print 장치의 RGB555 양자화와 표준 Bayer 순서 디더링.

/// top-down RGBA의 RGB만 bottom-up DIB 순서로 양자화한다.
/// 알파와 raster 크기는 이 색 변환의 계약에 포함하지 않는다.
pub(super) fn quantize(rgba: &mut [u8], width: u32, height: u32) -> Option<()> {
    let stride = usize::try_from(width).ok()?.checked_mul(4)?;
    let rows = usize::try_from(height).ok()?;
    if stride == 0 || rows == 0 || rgba.len() != stride.checked_mul(rows)? {
        return None;
    }
    // 재귀 Bayer 순위: 2×2 [[0,2],[3,1]]을 두 축으로 확장한다.
    const BAYER: [[u16; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    for (y, row) in rgba.chunks_exact_mut(stride).enumerate() {
        let ranks = BAYER[(rows - 1 - y) % 4];
        for (x, pixel) in row.chunks_exact_mut(4).enumerate() {
            let bias = ranks[x % 4] / 2;
            for channel in &mut pixel[..3] {
                let q = ((u16::from(*channel) + bias) >> 3).min(31) as u8;
                *channel = (q << 3) | (q >> 2);
            }
        }
    }
    Some(())
}

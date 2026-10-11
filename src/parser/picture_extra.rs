//! HWP5 그림 레코드의 가변 효과 뒤에 오는 원본 크기 위치.

use super::byte_reader::ByteReader;

/// 테두리 투명도·instance ID·효과를 건너뛴다 (HWP5 표 107~114).
/// 알 수 없거나 잘린 효과는 크기/투명도로 해석하지 않고 raw 바이트로 보존한다.
pub(crate) fn picture_dimensions_offset(extra: &[u8]) -> Option<usize> {
    let mut r = ByteReader::new(extra);
    r.skip(5).ok()?; // border opacity + instance ID
    let flags = r.read_u32().ok()?;
    if flags & !0x0f != 0 {
        return None;
    }
    if flags & 1 != 0 {
        r.skip(44).ok()?; // shadow, excluding its variable color
        skip_effect_color(&mut r)?;
    }
    if flags & 2 != 0 {
        r.skip(8).ok()?; // glow alpha + radius
        skip_effect_color(&mut r)?;
    }
    if flags & 4 != 0 {
        r.skip(4).ok()?; // soft-edge radius (float), not image width
    }
    if flags & 8 != 0 {
        // 표 112의 각 필드는 4바이트이며 14개다.
        r.skip(14 * 4).ok()?;
    }
    Some(r.position())
}

fn skip_effect_color(r: &mut ByteReader<'_>) -> Option<()> {
    // RGB(type 0)의 저장 구조만 확인됐다. 다른 색상 타입의 길이는 추정하지 않는다.
    if r.read_u32().ok()? != 0 {
        return None;
    }
    r.skip(4).ok()?; // RGB
    let count = usize::try_from(r.read_u32().ok()?).ok()?;
    r.skip(count.checked_mul(8)?).ok()?; // effect type + float value
    Some(())
}

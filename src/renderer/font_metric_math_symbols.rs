// scripts/extract_verified_symbol_metrics.py의 SFNT cmap/hmtx 직접 추출.
// U+2200..U+22FF. 메트릭 결측과 실제 cmap 결측을 구분하며 기존 폭은 덮어쓰지 않는다.
use super::FontMetric;
// face: Haansoft Batang; key: Haansoft Batang; SHA-256: 35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05; program em: 1024; DB em: 1024.
static VERIFIED_SYMBOL_0_WIDTHS: [u16; 256] = [
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 512, 512, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
];
static VERIFIED_SYMBOL_0_PRESENT: [bool; 256] = [
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
];
// face: 휴먼명조; key: HumanMyeongJo; SHA-256: d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6; program em: 512; DB em: 1024.
static VERIFIED_SYMBOL_1_WIDTHS: [u16; 256] = [
    1024, 0, 1024, 1024, 0, 0, 0, 1024, 1024, 0, 0, 1024, 0, 0, 0, 1024, 0, 1024, 0, 0, 0, 0, 0, 0,
    0, 0, 1024, 0, 0, 1024, 1024, 0, 1024, 0, 0, 0, 0, 1024, 0, 1024, 1024, 1024, 1024, 1024, 1024,
    0, 1024, 0, 0, 0, 0, 0, 1024, 1024, 0, 0, 0, 0, 0, 0, 1024, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1024, 1024, 0, 0,
    1024, 1024, 0, 0, 0, 0, 1024, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 1024, 1024, 0, 0, 1024, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1024, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0,
];
static VERIFIED_SYMBOL_1_PRESENT: [bool; 256] = [
    true, false, true, true, false, false, false, true, true, false, false, true, false, false,
    false, true, false, true, false, false, false, false, false, false, false, false, true, false,
    false, true, true, false, true, false, false, false, false, true, false, true, true, true,
    true, true, true, false, true, false, false, false, false, false, true, true, false, false,
    false, false, false, false, true, true, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, true,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    true, true, false, false, true, true, false, false, false, false, true, true, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, true, true, false, false, true, true, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, true, false, false, false, false, false, false, false, false, false,
    false, false, true, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false,
];

fn entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {
    if !(0x2200..=0x22FF).contains(&(ch as u32)) {
        return None;
    }
    let (present, widths) = match (metric.name, metric.em_size, metric.bold, metric.italic) {
        ("Haansoft Batang", 1024, false, false) => {
            (&VERIFIED_SYMBOL_0_PRESENT, &VERIFIED_SYMBOL_0_WIDTHS)
        }
        ("HumanMyeongJo", 1024, false, false) => {
            (&VERIFIED_SYMBOL_1_PRESENT, &VERIFIED_SYMBOL_1_WIDTHS)
        }
        _ => return None,
    };
    let index = (ch as u32 - 0x2200) as usize;
    Some((present[index], widths[index]))
}

pub(super) fn has_glyph(metric: &FontMetric, ch: char) -> Option<bool> {
    entry(metric, ch).map(|(present, _)| present)
}

pub(super) fn width(metric: &FontMetric, ch: char) -> Option<u16> {
    let (present, width) = entry(metric, ch)?;
    (present && width > 0).then_some(width)
}

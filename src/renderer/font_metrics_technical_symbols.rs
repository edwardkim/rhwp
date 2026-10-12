// scripts/extract_verified_symbol_metrics.py의 SFNT cmap/hmtx 직접 추출.
// U+2300..U+23FF. 메트릭 결측과 실제 cmap 결측을 구분하며 기존 폭은 덮어쓰지 않는다.
// face: Haansoft Batang; key: Haansoft Batang; SHA-256: 35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05; program em: 1024; DB em: 1024.
static VERIFIED_TECHNICAL_0_WIDTHS: [u16; 256] = [
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
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static VERIFIED_TECHNICAL_0_PRESENT: [bool; 256] = [
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
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
];
// face: 휴먼명조; key: HumanMyeongJo; SHA-256: d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6; program em: 512; DB em: 1024.
static VERIFIED_TECHNICAL_1_WIDTHS: [u16; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];
static VERIFIED_TECHNICAL_1_PRESENT: [bool; 256] = [
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, true, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false,
];

fn verified_technical_symbol_entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {
    // 독립 한컴 2020 Print의 Batang 프로그램: U+25A1 전진은 911/1024em이다.
    // 기존 DB가 수집하지 않은 기하 기호를 전각 추정으로 바꾸지 않는다.
    // 근거: 2599643_port_call_form-2020.pdf의 xref 12, glyph 15, hmtx.
    if (metric.name, metric.em_size, metric.bold, metric.italic, ch)
        == ("Batang", 1024, false, false, '\u{25A1}')
    {
        return Some((true, 911));
    }
    if !(0x2300..=0x23FF).contains(&(ch as u32)) {
        return None;
    }
    let (present, widths) = match (metric.name, metric.em_size, metric.bold, metric.italic) {
        ("Haansoft Batang", 1024, false, false) => {
            (&VERIFIED_TECHNICAL_0_PRESENT, &VERIFIED_TECHNICAL_0_WIDTHS)
        }
        ("HumanMyeongJo", 1024, false, false) => {
            (&VERIFIED_TECHNICAL_1_PRESENT, &VERIFIED_TECHNICAL_1_WIDTHS)
        }
        _ => return None,
    };
    let index = (ch as u32 - 0x2300) as usize;
    Some((present[index], widths[index]))
}

pub(crate) fn verified_program_has_glyph(metric: &FontMetric, ch: char) -> Option<bool> {
    verified_technical_symbol_entry(metric, ch).map(|(present, _)| present)
}

fn verified_technical_symbol_width(metric: &FontMetric, ch: char) -> Option<u16> {
    let (present, width) = verified_technical_symbol_entry(metric, ch)?;
    (present && width > 0).then_some(width)
}

// HFT 보정이 섞인 기존 표와 별개인 원 프로그램의 실제 hmtx.
pub(crate) fn verified_program_punctuation_width(metric: &FontMetric, ch: char) -> Option<u16> {
    match (
        metric.name,
        metric.em_size,
        metric.bold,
        metric.italic,
        ch as u32,
    ) {
        // HMKMG.TTF SHA-256 9e43aaf13552cd66b6f73e5811be3c7858bdb03d8d35dc7662c89bf2ea54b280.
        // cmap의 periodcentered gid12190과 hmtx512/unitsPerEm512를 직접 확인했다.
        // 80250 독립 Print p10의 같은 face·이웃 원점도 전각을 입증한다.
        ("휴먼고딕", 512, false, false, 183) => Some(512),
        // H2GTRM.TTF SHA-256 0b18f81f9d463f6c498853af3bdad577e9c881a5b59dc7ef4c43287588d696d5.
        // periodcentered/quoteleft/quoteright의 hmtx는 각각 1024/unitsPerEm1024다.
        // 실제 SVG 프로그램의 glyf/hmtx/cmap이 원 파일과 같고, 독립 개인정보 Print
        // 전 66쪽의 가운데점 29개·작은따옴표 8개도 1em이다. HFT 표에는 적용하지 않는다.
        ("HYGothic-Medium", 1024, false, false, 183 | 8216 | 8217) => Some(1024),
        ("Haansoft Batang", 1024, false, false, 183) => Some(341),
        ("Haansoft Batang", 1024, false, false, 8216) => Some(276),
        ("Haansoft Batang", 1024, false, false, 8217) => Some(276),
        ("Haansoft Batang", 1024, false, false, 8220) => Some(379),
        ("Haansoft Batang", 1024, false, false, 8221) => Some(379),
        ("HumanMyeongJo", 1024, false, false, 183) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8216) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8217) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8220) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8221) => Some(1024),
        // 공식 H2GTRM.TTF: SHA 0b18f81f9d463f6c498853af3bdad577e9c881a5b59dc7ef4c43287588d696d5.
        // 원본 한컴 Print와 같은 periodcentered·곡선 따옴표의 실제 hmtx다.
        ("HYGothic-Medium", 1024, false, false, 183 | 8216 | 8217 | 8220 | 8221) => Some(1024),
        _ => None,
    }
}

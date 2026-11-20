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
    // 같은 HBATANG 35f84328…의 Arrows 전체 cmap/hmtx: 112개 모두
    // 1024/1024em이다. 독립 편람 Print의 결측 화살표도 이 프로그램을 쓴다.
    // 기존 DB의 미수집 구간만 보충하며 다른 face·굵기·기울임은 추정하지 않는다.
    if (0x2190..=0x21FF).contains(&(ch as u32))
        && (metric.name, metric.em_size, metric.bold, metric.italic)
            == ("Haansoft Batang", 1024, false, false)
    {
        return Some((true, 1024));
    }
    // 독립 한컴 2020 Print의 Batang 프로그램: U+25A1 전진은 911/1024em이다.
    // 기존 DB가 수집하지 않은 기하 기호를 전각 추정으로 바꾸지 않는다.
    // 근거: 2599643_port_call_form-2020.pdf의 xref 12, glyph 15, hmtx.
    if (metric.name, metric.em_size, metric.bold, metric.italic, ch)
        == ("Batang", 1024, false, false, '\u{25A1}')
    {
        return Some((true, 911));
    }
    // 현재 HBATANG bef63718…의 U+257A는 1024/1024em이다. 독립 화학 원본
    // Print의 Haansoft Batang(xref 28, glyph 6)도 같은 전진·사각 잉크를 쓴다.
    // BatangChe에 없는 표식을 다른 반각 글꼴의 잉크로 그리지 않게 공급을 인증한다.
    if (metric.name, metric.em_size, metric.bold, metric.italic, ch)
        == ("Haansoft Batang", 1024, false, false, '\u{257A}')
    {
        return Some((true, 1024));
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

/// MT.TTF(d1050921…ff44)의 Unicode cmap에는 공백을 제외한 ASCII가 없다.
/// HBATANG.TTF(bef63718…4068)는 그 전 범위를 제공하며 독립 Print는 같은 공급자를 쓴다.
/// 12쪽의 U+2024도 HBATANG에서 178/1024em이며, 두 위치에서 인쇄 전진을 확인했다.
fn mbatang_print_missing_glyph(ch: char) -> bool {
    ch.is_ascii_graphic() || matches!(ch, '·' | '․' | '◦' | '「' | '」')
}

pub(crate) fn verified_program_has_glyph(metric: &FontMetric, ch: char) -> Option<bool> {
    if mbatang_print_missing_glyph(ch) && !metric.bold && !metric.italic {
        match (metric.name, metric.em_size) {
            ("MBatang", 1000) => return Some(false),
            ("Haansoft Batang", 1024) => return Some(true),
            _ => {}
        }
    }
    // batang.TTC 0ea38ac5… 및 84b0ba79…의 BatangChe regular cmap 결측이다.
    // 독립 Print는 같은 선언 체인의 Haansoft Batang으로 U+257A를 그린다.
    // 실제 제공 프로그램의 hit와 명시적 HFT 계약은 호출자의 우선순위를 유지한다.
    if (metric.name, metric.em_size, metric.bold, metric.italic, ch)
        == ("BatangChe", 1024, false, false, '\u{257A}')
    {
        return Some(false);
    }
    // malgun.ttf 7a183cf1…192f / malgunbd.ttf e8cbc0b2…9082의 실제 cmap에는
    // U+2981이 없다. 설치한 다른 맑은 고딕 TTF 3종도 같은 결측이다.
    // 독립 시행령 Print는 HCRDotum의 동일 윤곽·970/1000em을 사용한다.
    // 공급자는 기존 선언 체인이 고르며 다른 글자·기울임의 결측은 추정하지 않는다.
    if ch == '\u{2981}'
        && metric.name == "Malgun Gothic"
        && metric.em_size == 2048
        && !metric.italic
    {
        return Some(false);
    }
    // 같은 등록 프로그램의 cmap 결측이다. 폭 표에 없다는 사실만으로
    // 결측을 추정하지 않으며, 다른 weight/style 프로그램에는 적용하지 않는다.
    // U+2027도 같은 원 프로그램에서 결측이고 Haansoft Batang이 공급한다.
    // 설치 프로그램 HMKMM d033df71/HBATANG bef63718의 cmap으로 확인했으며,
    // 자연 전진은 기존 메트릭의 178/1024em이며, 독립 Print의 다음 글자 원점으로 대조했다.
    if matches!(ch, '\u{2024}' | '\u{2027}') {
        match (metric.name, metric.em_size, metric.bold, metric.italic) {
            // arial.ttf SHA-256 b23742143bae95dfaf50d8d530e798c7863336b254b899024b0b12b09c93abfd.
            // 실제 regular 프로그램의 cmap 결측이며 다른 Arial face에는 적용하지 않는다.
            ("Arial", 2048, false, false) => return Some(false),
            ("HumanMyeongJo", 1024, false, false) => return Some(false),
            // H2HDRM.TTF e50c7200521c3ea59840e5dd42e79708905a74a607f2300ed862fe2491caf831:
            // 원 프로그램 cmap의 확인된 결측이다. 폭 표의 미수집을 뜻하지 않는다.
            ("HYHeadLine-Medium", 1024, false, false) if ch == '\u{2024}' => return Some(false),
            // [#8419] batang.ttc 51d9a8d4205239c865e24b6eb7e6e6dbacfd59c3e2c46f623db0af45e169c9d8:
            // Batang regular cmap에 U+2024·U+2027이 모두 없다. 후자의 공급자도
            // 화학 원본 27쪽의 독립 Print xref28·glyph7(178/1024em)로 확인했다.
            ("Batang", 1024, false, false) => return Some(false),
            // 9213 한컴 2024 Print 15쪽의 BatangChe U+2027만 동일한
            // Haansoft Batang glyph로 공급한다. 다른 문자와 스타일은 유지한다.
            ("BatangChe", 1024, false, false) if ch == '\u{2027}' => return Some(false),
            // H2GTRE.TTF ca822120…5c234의 실제 cmap 결측이다.
            // 독립 Print에서 HBATANG 공급을 확인한 U+2027만 해소한다.
            ("HYGothic-Extra", 1024, false, false) if ch == '\u{2027}' => {
                return Some(false)
            }
            // gulim.TTC ed750275…776c의 GulimChe regular cmap 결측.
            // 독립2020 Print에서 공급자를 확인한 U+2024만 해소한다.
            ("GulimChe", 1024, false, false) if ch == '\u{2024}' => return Some(false),
            // H2MJSM.TTF/gulim.TTC의 실제 cmap 결측. 서로 다른 기호
            // 슬롯을 교차한 Print에서 각각 HBATANG/HDOTUM 공급도 확인했다.
            ("HYSinMyeongJo-Medium" | "Gulim", 1024, false, false) => return Some(false),
            // [#8619] gulim.ttc 05a38ff214ee932c6147a4e697b494040fd68ce654795f10f723c01d7bb8ee12:
            // Dotum regular cmap에 U+2024·U+2027이 모두 없지만 공급자는 U+2024만
            // 독립 Print로 확인했다. U+2027의 기존 처리는 바꾸지 않는다.
            ("Dotum", 1024, false, false) if ch == '\u{2024}' => return Some(false),
            ("Haansoft Batang", 1024, false, false) => return Some(true),
            ("Haansoft Dotum", 1024, false, false) => return Some(true),
            _ => {}
        }
    }
    verified_technical_symbol_entry(metric, ch).map(|(present, _)| present)
}

fn verified_technical_symbol_width(metric: &FontMetric, ch: char) -> Option<u16> {
    let (present, width) = verified_technical_symbol_entry(metric, ch)?;
    (present && width > 0).then_some(width)
}

// HFT 보정이 섞인 기존 표와 별개인 원 프로그램의 실제 hmtx.
// 문서의 유형이 미상이어도 독립 Print와 원 프로그램에서 확인한 부분 glyph다.
// 호출자가 명시적 HFT 슬롯은 제외하며 전체 메트릭 표의 신뢰도를 승격하지 않는다.
pub(crate) fn verified_non_hft_glyph_width(metric: &FontMetric, ch: char) -> Option<u16> {
    match (metric.name, metric.em_size, metric.bold, metric.italic, ch) {
        // 두 gulim TTC의 Dotum face에서 H18543의 hmtx는 384/1024다.
        ("Dotum", 1024, false, false, '\u{25AA}') => Some(384),
        _ => None,
    }
}

pub(crate) fn verified_program_punctuation_width(metric: &FontMetric, ch: char) -> Option<u16> {
    if let Some(width) = verified_non_hft_glyph_width(metric, ch) {
        return Some(width);
    }
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
        ("Haansoft Batang", 1024, false, false, 8228) => Some(178),
        ("HumanMyeongJo", 1024, false, false, 183) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8216) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8217) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8220) => Some(1024),
        ("HumanMyeongJo", 1024, false, false, 8221) => Some(1024),
        // 공식 H2GTRM.TTF: SHA 0b18f81f9d463f6c498853af3bdad577e9c881a5b59dc7ef4c43287588d696d5.
        // 원본 한컴 Print와 같은 periodcentered·곡선 따옴표의 실제 hmtx다.
        ("HYGothic-Medium", 1024, false, false, 8220 | 8221) => Some(1024),
        _ => None,
    }
}

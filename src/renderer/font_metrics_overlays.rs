// Measured/manual metric overlay region — Task #4964 W6.
//
// These five entries are reconstructed from the tracked #2430 Hancom COM ladder evidence.
// They intentionally reuse selected Latin/Hangul tables from the historical generated region.
// Do not move them ahead of the generated entries or alter their order.

/// 원본 face의 일부 글리프만 독립적으로 확인한 경우의 자연 전진폭.
/// 미확인 글자/스타일은 이 표로 추정하지 않고 기존 표시 폴백을 사용한다.
struct OriginalGlyphMetric {
    face: &'static str,
    bold: bool,
    italic: bool,
    character: char,
    advance_units: u16,
    em_units: u16,
}

// #7445: pdf/hwp3-sample16-hwp5-hwp-2024.pdf 23쪽의 HCI Poppy Type3.
// 슬래시 네 군데의 /Widths와 CharProc d1이 모두 605/1000이다.
// 표시 폴백 Palatino의 656/2048은 원본 HFT 글리프의 폭이 아니다.
static ORIGINAL_GLYPH_METRICS: &[OriginalGlyphMetric] = &[
    // 수동 Hancom2024 Print sample16 p41 T5: ToUnicode003C/003E/003D,
    // Widths605·FontMatrix.001·각 CharProc d1=605. 굵은 원본 HCI Poppy를
    // 표시 폴백의 반각 기호로 재면 혼합 소제목의 뒤 한글이 당겨진다.
    OriginalGlyphMetric {
        face: "HCI Poppy",
        bold: true,
        italic: false,
        character: '<',
        advance_units: 605,
        em_units: 1000,
    },
    OriginalGlyphMetric {
        face: "HCI Poppy",
        bold: true,
        italic: false,
        character: '>',
        advance_units: 605,
        em_units: 1000,
    },
    OriginalGlyphMetric {
        face: "HCI Poppy",
        bold: true,
        italic: false,
        character: '=',
        advance_units: 605,
        em_units: 1000,
    },
    OriginalGlyphMetric {
        face: "HCI Poppy",
        bold: false,
        italic: false,
        character: '/',
        advance_units: 605,
        em_units: 1000,
    },
    // worklife_balance_index_156607916-2020.pdf p4/p5 T10의
    // ToUnicode 002B와 Widths605/FontMatrix.001. 대체 Palatino의
    // 0.5em으로 재면 괄호 안 다음 숫자가 기호마다 약1.75CSSpx 당겨진다.
    OriginalGlyphMetric {
        face: "HCI Poppy",
        bold: false,
        italic: false,
        character: '+',
        advance_units: 605,
        em_units: 1000,
    },
];

// samples/hwp3-empty-cell.hwp의 HCI Tulip regular: 독립 한컴 2020 Print
// pdf/hwp3-empty-cell-2020.pdf /T1의 ToUnicode와 /Widths, FontMatrix 0.001.
// 한컴 재저장본의 charPr 12도 '1. ' 등 번호 런의 LATIN face를 HCI Tulip로
// 보존한다. 표시 폴백 HY견명조의 숫자 폭(약 2/3 em)을 원본 폭으로 쓰면
// 번호 뒤 한글이 밀린다. 확인한 글리프만 사용하고 미확인 문자/굵기/기울임은
// 기존 폴백에 맡긴다. 전체 face의 메트릭 표로 확장하거나 추정하지 않는다.
// useFontSpace=false의 일반 공백은 문서의 반각 공백 계약이 소유한다.
// /T1의 공백 글리프 폭 250을 그 문서 전진폭으로 덮어쓰지 않는다.
static HCI_TULIP_REGULAR_ADVANCES: &[(char, u16)] = &[
    ('(', 332),
    (')', 332),
    (',', 248),
    ('.', 248),
    ('0', 498),
    ('1', 498),
    ('2', 498),
    ('3', 498),
    ('4', 498),
    ('5', 498),
    ('6', 498),
    ('7', 498),
    (':', 277),
    ('N', 722),
    ('O', 722),
    ('[', 332),
    (']', 332),
];

// 독립 Print의 원본 HFT 자연 폭. 호출자가 선언 프로그램을 확인한 경우만 쓴다.
// worklife_balance_index_156607916-2020.pdf p4 T16 ToUnicode2019:
// Widths225, FontMatrix.001, CharProc d1=225. 국립국어원2022 p3 T9에서도
// 좌우2018/2019는225/1000. 동명 TTF나 표시 대체의 전각 폭으로 확장하지 않는다.
pub(crate) fn has_declared_original_glyph_metrics(face: &str) -> bool {
    has_original_glyph_metrics(face) || face == "한양중고딕"
}

pub(crate) fn declared_original_glyph_advance_em(
    face: &str,
    bold: bool,
    italic: bool,
    character: char,
) -> Option<f64> {
    original_glyph_advance_em(face, bold, italic, character).or_else(|| {
        (face == "한양중고딕" && !bold && !italic
            && matches!(character, '\u{2018}' | '\u{2019}'))
            .then_some(225.0 / 1000.0)
    })
}

pub(crate) fn has_original_glyph_metrics(face: &str) -> bool {
    face == "HCI Tulip"
        || ORIGINAL_GLYPH_METRICS
        .iter()
        .any(|metric| metric.face == face)
}

pub(crate) fn original_glyph_advance_em(
    face: &str,
    bold: bool,
    italic: bool,
    character: char,
) -> Option<f64> {
    if face == "HCI Tulip" && !bold && !italic {
        return HCI_TULIP_REGULAR_ADVANCES
            .iter()
            .find(|(ch, _)| *ch == character)
            .map(|(_, advance)| f64::from(*advance) / 1000.0);
    }
    ORIGINAL_GLYPH_METRICS
        .iter()
        .find(|metric| {
            metric.face == face
                && metric.bold == bold
                && metric.italic == italic
                && metric.character == character
        })
        .map(|metric| f64::from(metric.advance_units) / f64::from(metric.em_units))
}

pub // [#2430] 한양신명조 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.942).
// 한글은 한양신명조 를 HYSinMyeongJo-Medium 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYSinMyeongJo-Medium 공유.
// U+0022는 독립 한컴2024 Print의 Type3 /Widths 42.3/100 em → 433/1024로 정정한다.
// 원본115쪽 pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf 8/9쪽의30개와
// 분할197쪽 pdf/issue4138-picture-liter-20261014-2024.pdf 13쪽의 실제 줄 소속이 근거다.
// HFT 한양신명조의 Latin/Symbol 슬롯이며, 대체 Batang의0.416em이나 TTF hmtx를 사용하지 않는다.
// U+002E는 독립 Print 원본 8/9쪽 88개·분할 148쪽 31개의 Type3 전진폭
// 0.244em을 250/1024로 양자화한다. 246/1024는 물리 폭 경계에서 문단 끝
// `다.`를 앞줄에 잘못 수용했다. 표시 대체 글꼴의 hmtx는 근거로 쓰지 않는다.
// 독립 원본115쪽·같은 입력의 분할197쪽 Print에서 CharProc SHA와 자연 폭이
// 일치하는 비공백 ASCII77개를 교차 검증했다. 1000em 자연 폭을 이 표의1024em으로
// 최근접 양자화한다. 공백·미확인17개·별도 HY신명조·휴먼명조 메트릭은 보존한다.
// 출처와 glyph별 SHA/old/new는 stage5 C31의 cross-verified HFT 증거를 따른다.
static HANYANGSINMYEONGJO_LATIN_0: [u16; 95] = [518, 333, 433, 465, 614, 939, 833, 395, 404, 383, 537, 581, 241, 586, 250, 399, 512, 512, 512, 512, 512, 512, 512, 512, 512, 512, 283, 299, 746, 584, 752, 526, 939, 813, 721, 707, 767, 693, 670, 760, 803, 345, 465, 811, 660, 982, 796, 766, 644, 773, 720, 664, 793, 790, 799, 1062, 772, 787, 613, 371, 404, 360, 439, 509, 395, 521, 569, 497, 556, 494, 360, 602, 573, 284, 322, 570, 281, 879, 578, 528, 547, 561, 415, 498, 359, 575, 595, 860, 604, 588, 485, 430, 290, 430, 509];
/// [#7092] 한양신명조의 `·`(U+00B7) — 한컴 정본 실측 393/1024 = 0.384 em.
///
/// 한양중고딕(`HANYANGJUNGGOTHIC_B7`)과 같은 갈래다. 아래 `0x00A0-0x00FF` 구간을
/// `FONT_276_LATIN_1`(= `HYSinMyeongJo-Medium`, 윈도우 `H2MJSM.TTF`)에서 빌려 오는데,
/// 그 글꼴의 `periodcentered` 는 1024/1024 = 전각이라 이 글자만 한양 실제값과 어긋난다.
///
/// 문서 둘이 독립적으로 같은 값을 준다. 두 문서 모두 `·` 런의 글꼴을 rhwp 가
/// `한양신명조` 로만 풀고, 첫 문서는 **개수까지 1:1 로 맞는다**(rhwp 38개 ↔ 정본 38개).
///
/// ```text
///   samples/21868765_별표2_보건소_분장사무.pdf         Type3 n=38  /W 0.3842  (393.4/1024)
///     p1 '.일반서무·복무·보안' · '서무·복무·보안및공인' · '보건의지도·감독'
///   pdf/task2097/21298295_byeolpyo5_disaster-hwp-2020.pdf
///                                                   Type3 n=19  /W 0.3840  (393.2/1024)
/// ```
///
/// 한양중고딕이 390 인 것과 값이 다르다 — 서로 다른 글꼴이니 각자 실측을 쓴다.
static HANYANGSINMYEONGJO_B7: [u16; 1] = [393];
// 한컴 HFT U+2219 불릿의 전진폭은 1em이다. 독립 PDF Type3 /HFT8의 /W=1000.
// U+2218도 같은 전각이다. 로드맵 독립 Print PDF 44쪽의 /T8은
// ToUnicode <08> <2218>, 같은 코드의 /Widths 1000, FontMatrix 0.001을 명시한다.
// 둘을 같은 문자로 바꾸지 않고 각 코드의 자연 전진폭만 보존한다.
static HANYANGSINMYEONGJO_BULLET: [u16; 1] = [1024];
// 독립 한컴 Print의 Type3 자연 전진폭: 분할149/153쪽의 “/”는0.441em,
// 149/187쪽의 ±는0.617em이다. 같은 HFT 소유의 glyph를 TTF 전각 표나
// 일반 구두점0.5em 휴리스틱으로 재지 않는다. 1024HU표의 최근접 값만 쓴다.
static HANYANGSINMYEONGJO_DOUBLE_QUOTES: [u16; 2] = [452, 452];
static HANYANGSINMYEONGJO_PLUS_MINUS: [u16; 1] = [632];
// 독립 원본83쪽·분할140쪽 Print의 같은 HFT 곱셈 글리프는616/1000em이다.
// 대체 H2MJSM의 전각 폭을 재사용하지 않고 확인한 이 기호만631/1024로 보존한다.
static HANYANGSINMYEONGJO_MULTIPLICATION: [u16; 1] = [631];
// 두 독립 Print의 섭씨/화씨 단위는 서로 다른 원문 문자를 보존하며 각각1em이다.
// U+2109는 기존 범위 누락으로 반각 fallback이었으므로 확인한 이 glyph만 보충한다.
static HANYANGSINMYEONGJO_FAHRENHEIT: [u16; 1] = [1024];
// worklife 독립 Print p3..5 T14/T19의 ToUnicode=2027, Widths=500,
// FontMatrix=.001. 기호 슬롯을 교차한 한컴2020 probe도 같은 HFT 전진이다.
// TrueType HYSinMyeongJo의 cmap 결측/대체 글리프 폭과는 다른 프로그램이다.
static HANYANGSINMYEONGJO_HYPHENATION_POINT: [u16; 1] = [512];
static HANYANGSINMYEONGJO_LATIN_RANGES: [LatinRange; 16] = [
    LatinRange {
        start: 0x2027,
        end: 0x2027,
        widths: &HANYANGSINMYEONGJO_HYPHENATION_POINT,
    },
    LatinRange {
        start: 0x00D7,
        end: 0x00D7,
        widths: &HANYANGSINMYEONGJO_MULTIPLICATION,
    },
    // 독립 HWP5 변환본 Print 5쪽의 •도 HFT 전각 글리프다.
    // 굵은 글줄은 이 기본 폭에 한글과 같은 상대 굵은 전진을 적용한다.
    LatinRange {
        start: 0x2022,
        end: 0x2022,
        widths: &HANYANGSINMYEONGJO_BULLET,
    },
    LatinRange {
        start: 0x2109,
        end: 0x2109,
        widths: &HANYANGSINMYEONGJO_FAHRENHEIT,
    },
    LatinRange {
        start: 0x201C,
        end: 0x201D,
        widths: &HANYANGSINMYEONGJO_DOUBLE_QUOTES,
    },
    LatinRange {
        start: 0x00B1,
        end: 0x00B1,
        widths: &HANYANGSINMYEONGJO_PLUS_MINUS,
    },
    LatinRange {
        start: 0x2218,
        end: 0x2218,
        widths: &HANYANGSINMYEONGJO_BULLET,
    },
    LatinRange {
        start: 0x2219,
        end: 0x2219,
        widths: &HANYANGSINMYEONGJO_BULLET,
    },
    LatinRange {
        start: 0x00B7,
        end: 0x00B7,
        widths: &HANYANGSINMYEONGJO_B7,
    },
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGSINMYEONGJO_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_276_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_276_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_276_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_276_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_276_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_276_LATIN_6,
    },
];

// [#2430] 한양중고딕 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.871).
// 한글은 한양중고딕 를 HYGothic-Medium 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYGothic-Medium 공유.
// 한컴 2020·2024의 독립 `80168` PDF에서 이 face의 Type3 /Widths는 숫자
// 0~9 모두 500/1000em이다. COM 래더의 509/1024em은 측정 반올림 오차이며,
// 여러 숫자가 있는 줄에서는 누적되어 PDF의 줄 경계를 넘긴다.
static HANYANGJUNGGOTHIC_LATIN_0: [u16; 95] = [
    518, 254, 371, 544, 544, 886, 658, 223, 298, 290, 377, 526, 254, 526, 254, 272, 512, 512, 512,
    512, 512, 512, 512, 512, 512, 512, 254, 254, 596, 535, 596, 535, 1026, 649, 640, 719, 711, 667,
    605, 772, 702, 254, 483, 649, 553, 825, 711, 763, 649, 772, 711, 649, 596, 702, 649, 921, 632,
    632, 596, 263, 290, 263, 456, 509, 263, 544, 535, 491, 544, 544, 263, 544, 544, 202, 202, 483,
    202, 825, 544, 553, 544, 544, 316, 491, 263, 544, 491, 711, 483, 474, 500, 316, 246, 316, 509,
];
/// [#7092] 한양중고딕의 `·`(U+00B7) — 한컴 정본 실측 390/1024 = 0.381 em.
///
/// 아래 `0x00A0-0x00FF` 구간은 `FONT_267_LATIN_1`(= `HYGothic-Medium`, 윈도우
/// `H2GTRM.TTF`)에서 **빌려 온다.** 그 글꼴의 `periodcentered` 는 1024/1024 = 전각이라
/// 이 글자만 한양 실제값과 어긋난다. 한/글은 한양 계열을 자기 글꼴로 그리고(정본 PDF 의
/// Type3 자원), 그 `/W` 가 0.381 em 이다.
///
/// 근거는 문서 둘이 독립적으로 같은 값을 준다 — 두 문서 모두 `·` 런의 글꼴을 rhwp 가
/// `한양중고딕` 으로 풀고, 같은 줄이 정본에서 Type3 로 그려진다.
///
/// ```text
///   76076_regulatory_analysis  p18  '가피하게덮개·울을개방하고'   /W 0.381  전진 0.391
///   76076_regulatory_analysis  p18  '로덮개·울등을설치하'         /W 0.381  전진 0.380
///   80168_regulatory_analysis  Type3 n=54                       /W 0.381  전진 중앙 0.386
/// ```
///
/// 같은 문서의 `휴먼명조` 는 정본에서 TrueType(`INPILL+휴먼명조`)으로 나가고 `/W` 가
/// 1.001 em 이다 — 빌려 온 전각값이 그쪽에는 맞는다. 그래서 이 정정은 U+00B7 한 글자,
/// 한양중고딕 한 글꼴로 한정한다. 나머지 한양 셋은 앵커된 실측이 없어 손대지 않는다.
///
/// 이 표는 한/글이 **자기 HFT 글리프로 그리는 실현**(위 두 문서 모두 FACE_NAME 대체 글꼴이
/// HFT `한양중고딕`)의 값이다. 문서가 대체 글꼴로 TTF `HY중고딕` 을 적으면 한/글은 그
/// TrueType(정본 자원 `H2gtrM`)으로 그리고 `·` 도 1024/1024 라, 그 경우는
/// `style_resolver::verified_ttf_substitute` 가 측정 face 를 `HY중고딕` 으로 돌려 이 표를
/// 타지 않는다(#7092 78494·75544·issue1891_external_bindata_link).
static HANYANGJUNGGOTHIC_B7: [u16; 1] = [390];
static HANYANGJUNGGOTHIC_LATIN_RANGES: [LatinRange; 8] = [
    LatinRange {
        start: 0x00B7,
        end: 0x00B7,
        widths: &HANYANGJUNGGOTHIC_B7,
    },
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGJUNGGOTHIC_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_267_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_267_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_267_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_267_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_267_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_267_LATIN_6,
    },
];

// [#2430] 한양견명조 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.911).
// 한글은 한양견명조 를 HYMyeongJo-Extra 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYMyeongJo-Extra 공유.
static HANYANGKYUNMYEONGJO_LATIN_0: [u16; 95] = [
    509, 412, 544, 474, 649, 974, 860, 388, 412, 395, 544, 649, 290, 658, 325, 430, 579, 579, 579,
    579, 579, 579, 579, 579, 579, 579, 342, 351, 754, 658, 754, 579, 956, 833, 825, 772, 816, 737,
    719, 833, 851, 386, 526, 877, 711, 1009, 833, 816, 693, 833, 763, 702, 842, 833, 825, 1097,
    816, 816, 667, 404, 430, 395, 430, 509, 395, 561, 605, 535, 596, 535, 447, 640, 614, 316, 333,
    614, 316, 912, 632, 570, 596, 596, 456, 535, 395, 614, 640, 886, 649, 623, 526, 465, 316, 465,
    509,
];
static HANYANGKYUNMYEONGJO_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGKYUNMYEONGJO_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_271_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_271_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_271_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_271_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_271_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_271_LATIN_6,
    },
];

// [#2430] 한양견고딕 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.905).
// 한글은 한양견고딕 를 HYGothic-Extra 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYGothic-Extra 공유.
static HANYANGKYUNGOTHIC_LATIN_0: [u16; 95] = [
    509, 342, 540, 596, 570, 930, 728, 308, 377, 386, 430, 535, 333, 526, 333, 351, 579, 579, 579,
    579, 579, 579, 579, 579, 579, 579, 333, 333, 737, 535, 719, 623, 1026, 737, 737, 737, 737, 684,
    623, 798, 737, 281, 570, 737, 623, 851, 737, 798, 684, 790, 737, 684, 623, 737, 684, 965, 684,
    684, 623, 342, 351, 342, 439, 509, 272, 570, 623, 570, 623, 570, 342, 623, 623, 281, 281, 570,
    281, 912, 623, 623, 623, 623, 395, 570, 342, 623, 570, 798, 570, 570, 509, 412, 307, 412, 509,
];
static HANYANGKYUNGOTHIC_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGKYUNGOTHIC_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_266_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_266_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_266_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_266_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_266_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_266_LATIN_6,
    },
];

// [#2430] 휴먼명조 ASCII. 한글은 휴먼명조 를 HYSinMyeongJo-Medium 와 다른 실폭으로 렌더한다 —
// LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYSinMyeongJo-Medium 공유.
//
// 값은 휴먼명조 TrueType(`HMKMM.TTF`, upm 512)의 hmtx 전진폭 × 2 다. #2430 의 COM 래더 실측표는
// 이 값과 −6~+152/1024 어긋나 `,` `.` `(` `)` 가 장치 반올림 경계 아래로 떨어졌다(10pt 에서
// 한 단위 좁음). 한/글 2024 PDF 의 `가c` 사다리(94자 × 10pt·자간 −10% / 10pt / 13pt)는
// 이 표로 셈한 #7702 장치 전진폭과 94/94 모두 같고, 종전 표는 37·58·70자가 다르다.
static HUMANMYEONGJO_LATIN_0: [u16; 95] = [
    512, 208, 512, 672, 516, 768, 788, 296, 320, 320, 512, 512, 276, 512, 276, 320, 512, 512, 512,
    512, 512, 512, 512, 512, 512, 512, 276, 276, 512, 512, 512, 440, 796, 676, 616, 656, 696, 640,
    596, 720, 696, 264, 404, 660, 592, 800, 708, 732, 588, 704, 668, 508, 640, 692, 680, 956, 728,
    688, 612, 328, 320, 328, 372, 512, 332, 520, 556, 496, 520, 536, 400, 528, 568, 256, 272, 536,
    256, 816, 564, 524, 552, 552, 412, 420, 348, 552, 552, 740, 500, 524, 464, 300, 208, 300, 512,
];
// [#7092] 휴먼명조 전용 Latin-1 — `·`(U+00B7) 슬롯만 갈라 놓는다.
//
// 이 오버레이는 #2430 에서 LATIN_0(ASCII)만 실측으로 갈고 나머지 범위는
// `HYSinMyeongJo-Medium`(FONT_276) 표를 **공유**했다. 그런데 두 글꼴은 가운뎃점에서
// 갈린다. HY신명조는 글꼴 파일이 전각(`H2MJSM.TTF` `periodcentered` 1024/1024)이고
// 한/글 정본도 0.999em 이다(#7092). 휴먼명조는 선행 가드(9d006fd03)가 한/글 PDF 좌표로
// 0.33em 을 실측했다. 표를 공유하는 한 두 값을 함께 줄 수 없어 슬롯 하나만 분리한다.
//
// 값 307 = `1024 × 0.3` — 종전 전역 가드가 이 글꼴에 내던 폭과 같아 휴먼명조 조판은
// 달라지지 않는다.
//
// [#7092] ⚠ 이 슬롯은 **정본이 둘로 갈려** 아직 움직이지 않는다. 저장소 정본을 전수로
// 재면 같은 이름이 두 realization 으로 나온다.
//
// ```text
//   한/글이 TrueType `휴먼명조` 로 그린 쪽   `·` 전각비 1.000 (n=36, 전각 100%)
//     예) pdf/80168_regulatory_analysis-2022.pdf p99·p134 '피규제 기업·소상공인'
//   한/글이 Type3 서브셋(HFT)으로 그린 쪽     `·` 전각비 0.384
//     예) pdf/pr6481-visual/pr6481-issue6310-…-2020.pdf p3 차례 '2. 성·연령별 사망' (T5)
//   두 문서 모두 `휴먼명조` 를 alt_type=1(TTF)로 선언한다 — 선언만으로는 갈리지 않는다.
//   설치 글꼴 `ttfs/hwp/HMKMM.TTF`(휴먼명조, upm 512)의 `periodcentered` 는 512 = 1.000 em.
// ```
//
// 어느 realization 을 기준으로 삼을지는 출력 환경 판정이 필요하고, 값을 1024 로 올리면
// `issue6310` 차례 줄이 오른쪽 탭 쪽번호와 겹친다(실측 text-overlap 4 → 5). 그래서 이
// 슬롯은 그대로 두고, 같은 이슈에서 **고정폭 표의 작은따옴표**만 닫았다
// (`text_measurement.rs` 의 `quote_width_is_authentic`). 비고정폭 face 의 `·`·따옴표는
// realization 판정과 기호 슬롯 선택이 함께 필요해 #7092 잔여로 남는다.
static HUMANMYEONGJO_LATIN_1: [u16; 96] = [
    512, 1024, 512, 512, 1024, 512, 512, 1024, 1024, 512, 1024, 512, 512, 1024, 512, 512, 1024,
    1024, 1024, 1024, 1024, 512, 1024, 307, 1024, 1024, 1024, 512, 1024, 1024, 1024, 1024, 512,
    512, 512, 512, 512, 512, 1024, 512, 512, 512, 512, 512, 512, 512, 512, 512, 1024, 512, 512,
    512, 512, 512, 512, 1024, 1024, 512, 512, 512, 512, 512, 1024, 1024, 512, 512, 512, 512, 512,
    512, 1024, 512, 512, 512, 512, 512, 512, 512, 512, 512, 1024, 512, 512, 512, 512, 512, 512,
    1024, 1024, 512, 512, 512, 512, 512, 1024, 512,
];

static HUMANMYEONGJO_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HUMANMYEONGJO_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &HUMANMYEONGJO_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_276_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_276_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_276_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_276_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_276_LATIN_6,
    },
];

// [#6036] 한컴 윤고딕 720 — 한컴오피스 동봉 실폰트(HANYoonGothic720.ttf, upem 1000)
// hmtx 직접 추출. 한글 음절 지배 폭 880/1000em (11168/11172, 예외 4자는
// ±6 이내라 단일 폭으로 수렴). ASCII 는 hmtx 전량. 보도자료 기관명 머리에 흔한
// 글꼴인데 메트릭 부재로 맑은 고딕 대체 폭(과대)으로 측정돼 글상자 clip 이
// 글자를 깎았다(156509073 "경찰청" 한글 50.2pt vs rhwp 64.0pt, clip 55.1pt).
static HANYOONGOTHIC720_LATIN_0: [u16; 95] = [
    290, 262, 328, 587, 511, 784, 686, 201, 350, 350, 440, 586, 247, 531, 247, 423, 578, 347, 529,
    525, 542, 535, 555, 500, 568, 555, 291, 291, 593, 554, 593, 495, 854, 659, 624, 652, 687, 588,
    553, 695, 670, 246, 453, 599, 536, 803, 676, 734, 601, 734, 628, 546, 572, 674, 639, 856, 569,
    572, 576, 366, 423, 366, 431, 431, 368, 527, 601, 499, 601, 530, 319, 601, 583, 234, 249, 495,
    234, 860, 583, 580, 601, 601, 343, 416, 317, 597, 482, 707, 469, 485, 480, 396, 330, 396, 604,
];
static HANYOONGOTHIC720_LATIN_RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007E,
    widths: &HANYOONGOTHIC720_LATIN_0,
}];
static HANYOONGOTHIC720_HANGUL_WIDTHS: [u16; 1] = [880];
static HANYOONGOTHIC720_HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &FONT_0_HANGUL_CHO,
    jung_map: &FONT_0_HANGUL_JUNG,
    jong_map: &FONT_0_HANGUL_JONG,
    widths: &HANYOONGOTHIC720_HANGUL_WIDTHS,
};

// [#6036] 한컴 윤고딕 740 — 한컴오피스 동봉 실폰트(HANYoonGothic740.ttf, upem 1000)
// hmtx 직접 추출. 한글 음절 지배 폭 920/1000em (11172/11172, 예외 0자는
// ±6 이내라 단일 폭으로 수렴). ASCII 는 hmtx 전량. 보도자료 기관명 머리에 흔한
// 글꼴인데 메트릭 부재로 맑은 고딕 대체 폭(과대)으로 측정돼 글상자 clip 이
// 글자를 깎았다(156509073 "경찰청" 한글 50.2pt vs rhwp 64.0pt, clip 55.1pt).
static HANYOONGOTHIC740_LATIN_0: [u16; 95] = [
    290, 276, 345, 618, 538, 825, 722, 211, 369, 369, 462, 617, 259, 558, 259, 445, 588, 370, 548,
    554, 562, 554, 575, 517, 589, 575, 306, 306, 624, 583, 624, 521, 899, 686, 649, 679, 715, 611,
    575, 724, 698, 252, 470, 654, 557, 837, 704, 765, 625, 765, 653, 567, 595, 702, 665, 893, 591,
    595, 599, 385, 445, 385, 453, 453, 388, 546, 621, 527, 621, 571, 342, 621, 600, 232, 245, 538,
    232, 881, 600, 609, 621, 621, 364, 435, 342, 600, 504, 741, 491, 497, 492, 417, 347, 417, 635,
];
static HANYOONGOTHIC740_LATIN_RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007E,
    widths: &HANYOONGOTHIC740_LATIN_0,
}];
static HANYOONGOTHIC740_HANGUL_WIDTHS: [u16; 1] = [920];
static HANYOONGOTHIC740_HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &FONT_0_HANGUL_CHO,
    jung_map: &FONT_0_HANGUL_JUNG,
    jong_map: &FONT_0_HANGUL_JONG,
    widths: &HANYOONGOTHIC740_HANGUL_WIDTHS,
};

// [#6036] 한컴 윤고딕 760 — 한컴오피스 동봉 실폰트(HANYoonGothic760.ttf, upem 1000)
// hmtx 직접 추출. 한글 음절 지배 폭 960/1000em (11172/11172, 예외 0자는
// ±6 이내라 단일 폭으로 수렴). ASCII 는 hmtx 전량. 보도자료 기관명 머리에 흔한
// 글꼴인데 메트릭 부재로 맑은 고딕 대체 폭(과대)으로 측정돼 글상자 clip 이
// 글자를 깎았다(156509073 "경찰청" 한글 50.2pt vs rhwp 64.0pt, clip 55.1pt).
static HANYOONGOTHIC760_LATIN_0: [u16; 95] = [
    300, 293, 380, 655, 570, 874, 765, 223, 391, 391, 490, 654, 274, 591, 274, 471, 628, 397, 590,
    596, 605, 596, 619, 557, 634, 619, 324, 324, 661, 617, 661, 552, 953, 740, 701, 732, 770, 660,
    622, 780, 753, 280, 511, 706, 603, 900, 759, 823, 675, 823, 705, 613, 644, 757, 717, 959, 639,
    644, 648, 408, 471, 408, 479, 479, 411, 594, 660, 573, 660, 615, 381, 660, 651, 261, 275, 586,
    261, 949, 651, 648, 660, 660, 401, 472, 373, 651, 544, 780, 536, 542, 538, 442, 367, 442, 672,
];
static HANYOONGOTHIC760_LATIN_RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007E,
    widths: &HANYOONGOTHIC760_LATIN_0,
}];
static HANYOONGOTHIC760_HANGUL_WIDTHS: [u16; 1] = [960];
static HANYOONGOTHIC760_HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &FONT_0_HANGUL_CHO,
    jung_map: &FONT_0_HANGUL_JUNG,
    jong_map: &FONT_0_HANGUL_JONG,
    widths: &HANYOONGOTHIC760_HANGUL_WIDTHS,
};
// [#7293] 신명 중명조 — 한/글 2020 정본(engine 2020)의 Type3 `/Widths` 실측.
//
// 이 face 는 `layout-name`·`paint` 평면에는 규칙이 있는데 **metric 평면에만 없어서**,
// 측정이 미등록 글꼴 폴백(라틴 `font_size * 0.5`)으로 떨어졌다. 차례 줄
// `Ⅲ. EU법` 에서 `E`·`U` 전진이 정확히 0.5em 이 되어 줄 글자부가 좁았다.
//
// 근거: `1170000-200500003_…(최종본).hwp`(HWP3·법제처, 이 face 가 197,257자 = 98.6%)의
// 정본 PDF. 한/글은 이 글꼴을 Type3 로 그리므로 `/Widths`(FontMatrix .001)가 실측 폭
// 표 그 자체다. 문서 안 Type3 25개 중 이 face 의 것 2개(공유 ASCII 51자 일치율 0.98,
// 한글 전부 1000/1000)를 병합해 66자를 얻었다.
//
// ⚠ `U+0020`(공백)은 **뺐다** — `/Widths` 는 1000 이라고 적지만 실제 배치 전진은
// 241/1000 이다(표본 68). 나머지 52자는 배치 전진과 2% 안에서 일치한다.
// ⚠ 실측하지 않은 글자는 구간에 넣지 않았다 — 조회가 `None` 으로 떨어져 종전 폴백을
// 그대로 쓴다(없는 값을 지어내지 않는다).
// ⚠ 한글은 `hangul: None` 이다. 정본이 전부 1000/1000(=1.0em)이고 그것은 미등록
// 폴백과 같은 값이라, 별도 표를 만들 근거가 없다.
static SINMYEONG_JUNGMYEONGJO_L0: [u16; 3] = [410, 512, 512];
static SINMYEONG_JUNGMYEONGJO_L1: [u16; 15] = [
    410, 512, 410, 512, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 512,
];
static SINMYEONG_JUNGMYEONGJO_L2: [u16; 1] = [635];
static SINMYEONG_JUNGMYEONGJO_L3: [u16; 14] = [
    841, 836, 843, 909, 839, 808, 900, 961, 473, 663, 882, 788, 1082, 941,
];
static SINMYEONG_JUNGMYEONGJO_L4: [u16; 1] = [788];
static SINMYEONG_JUNGMYEONGJO_L5: [u16; 6] = [849, 745, 783, 940, 841, 1110];
static SINMYEONG_JUNGMYEONGJO_L6: [u16; 1] = [725];
static SINMYEONG_JUNGMYEONGJO_L7: [u16; 9] = [670, 670, 552, 690, 608, 395, 642, 727, 389];
static SINMYEONG_JUNGMYEONGJO_L8: [u16; 16] = [
    714, 391, 1022, 726, 611, 690, 664, 527, 574, 498, 727, 651, 899, 651, 650, 594,
];

static SINMYEONG_JUNGMYEONGJO_LATIN_RANGES: [LatinRange; 9] = [
    LatinRange {
        start: 0x0027,
        end: 0x0029,
        widths: &SINMYEONG_JUNGMYEONGJO_L0,
    },
    LatinRange {
        start: 0x002C,
        end: 0x003A,
        widths: &SINMYEONG_JUNGMYEONGJO_L1,
    },
    LatinRange {
        start: 0x003F,
        end: 0x003F,
        widths: &SINMYEONG_JUNGMYEONGJO_L2,
    },
    LatinRange {
        start: 0x0041,
        end: 0x004E,
        widths: &SINMYEONG_JUNGMYEONGJO_L3,
    },
    LatinRange {
        start: 0x0050,
        end: 0x0050,
        widths: &SINMYEONG_JUNGMYEONGJO_L4,
    },
    LatinRange {
        start: 0x0052,
        end: 0x0057,
        widths: &SINMYEONG_JUNGMYEONGJO_L5,
    },
    LatinRange {
        start: 0x005A,
        end: 0x005A,
        widths: &SINMYEONG_JUNGMYEONGJO_L6,
    },
    LatinRange {
        start: 0x0061,
        end: 0x0069,
        widths: &SINMYEONG_JUNGMYEONGJO_L7,
    },
    LatinRange {
        start: 0x006B,
        end: 0x007A,
        widths: &SINMYEONG_JUNGMYEONGJO_L8,
    },
];

// 한컴 고딕 설치 TTF hmtx에서 읽은 폭. 같은 원 face를 측정과 SVG 내장이 공유한다.
static HANCOM_GOTHIC_REGULAR_0: [u16; 95] = [
    264, 446, 297, 583, 583, 892, 892, 297, 446, 446, 446, 583, 297, 583, 297, 446, 583, 583, 583,
    583, 583, 583, 583, 583, 583, 583, 297, 297, 446, 583, 446, 669, 1052, 644, 627, 639, 721, 596,
    554, 710, 718, 247, 410, 626, 529, 884, 710, 752, 586, 752, 610, 592, 621, 696, 635, 961, 617,
    611, 594, 446, 961, 446, 434, 446, 297, 560, 588, 490, 588, 559, 340, 588, 592, 244, 301, 530,
    244, 892, 592, 577, 588, 588, 383, 475, 357, 592, 530, 788, 528, 530, 473, 446, 446, 446, 669,
];
static HANCOM_GOTHIC_REGULAR_1: [u16; 96] = [
    0, 446, 0, 0, 892, 0, 0, 446, 800, 0, 378, 0, 0, 478, 892, 0, 446, 800, 260, 260, 300, 0, 800,
    446, 446, 260, 385, 0, 892, 892, 892, 669, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    892, 0, 0, 0, 0, 0, 0, 800, 892, 0, 0, 0, 0, 0, 892, 892, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 800, 800, 0, 0, 0, 0, 0, 800, 0,
];
static HANCOM_GOTHIC_REGULAR_2: [u16; 112] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 672, 0, 0, 303, 303, 0, 0, 446,
    446, 0, 0, 892, 892, 586, 0, 0, 932, 892, 0, 0, 0, 0, 0, 0, 0, 0, 0, 892, 0, 486, 486, 0, 0, 0,
    0, 0, 0, 0, 932, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 436, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANCOM_GOTHIC_REGULAR_3: [u16; 64] = [
    932, 486, 486, 932, 0, 0, 0, 0, 486, 486, 486, 486, 486, 486, 486, 486, 486, 486, 0, 932, 486,
    486, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANCOM_GOTHIC_REGULAR_4: [u16; 240] = [
    0, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 932, 932, 932, 932, 0, 932, 932, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANCOM_GOTHIC_REGULAR_RANGES: [LatinRange; 5] = [
    LatinRange {
        start: 32,
        end: 126,
        widths: &HANCOM_GOTHIC_REGULAR_0,
    },
    LatinRange {
        start: 160,
        end: 255,
        widths: &HANCOM_GOTHIC_REGULAR_1,
    },
    LatinRange {
        start: 8192,
        end: 8303,
        widths: &HANCOM_GOTHIC_REGULAR_2,
    },
    LatinRange {
        start: 12288,
        end: 12351,
        widths: &HANCOM_GOTHIC_REGULAR_3,
    },
    LatinRange {
        start: 65280,
        end: 65519,
        widths: &HANCOM_GOTHIC_REGULAR_4,
    },
];
static HANCOM_GOTHIC_BOLD_0: [u16; 95] = [
    264, 446, 297, 583, 583, 892, 892, 297, 446, 446, 446, 583, 297, 583, 297, 446, 583, 583, 583,
    583, 583, 583, 583, 583, 583, 583, 297, 297, 446, 583, 446, 669, 1052, 644, 627, 639, 721, 596,
    554, 710, 718, 247, 410, 626, 529, 884, 710, 752, 586, 752, 610, 592, 621, 696, 635, 961, 617,
    611, 594, 446, 961, 446, 434, 446, 297, 560, 588, 490, 588, 559, 340, 588, 592, 244, 301, 530,
    244, 892, 592, 577, 588, 588, 383, 475, 357, 592, 530, 788, 528, 530, 473, 446, 446, 446, 669,
];
static HANCOM_GOTHIC_BOLD_1: [u16; 96] = [
    0, 446, 0, 0, 892, 0, 0, 446, 800, 0, 378, 0, 0, 478, 892, 0, 446, 800, 260, 260, 300, 0, 800,
    446, 446, 260, 385, 0, 892, 892, 892, 669, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    892, 0, 0, 0, 0, 0, 0, 800, 892, 0, 0, 0, 0, 0, 892, 892, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 800, 800, 0, 0, 0, 0, 0, 800, 0,
];
static HANCOM_GOTHIC_BOLD_2: [u16; 112] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 672, 0, 0, 303, 303, 0, 0, 446,
    446, 0, 0, 892, 892, 586, 0, 0, 932, 892, 0, 0, 0, 0, 0, 0, 0, 0, 0, 892, 0, 486, 486, 0, 0, 0,
    0, 0, 0, 0, 932, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 436, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANCOM_GOTHIC_BOLD_3: [u16; 64] = [
    932, 486, 486, 932, 0, 0, 0, 0, 486, 486, 486, 486, 486, 486, 486, 486, 486, 486, 0, 932, 486,
    486, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANCOM_GOTHIC_BOLD_4: [u16; 240] = [
    0, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932, 932,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 932, 932, 932, 932, 0, 932, 932, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANCOM_GOTHIC_BOLD_RANGES: [LatinRange; 5] = [
    LatinRange {
        start: 32,
        end: 126,
        widths: &HANCOM_GOTHIC_BOLD_0,
    },
    LatinRange {
        start: 160,
        end: 255,
        widths: &HANCOM_GOTHIC_BOLD_1,
    },
    LatinRange {
        start: 8192,
        end: 8303,
        widths: &HANCOM_GOTHIC_BOLD_2,
    },
    LatinRange {
        start: 12288,
        end: 12351,
        widths: &HANCOM_GOTHIC_BOLD_3,
    },
    LatinRange {
        start: 65280,
        end: 65519,
        widths: &HANCOM_GOTHIC_BOLD_4,
    },
];
static HANCOM_GOTHIC_WIDTHS: [u16; 1] = [932];
static HANCOM_GOTHIC_HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &FONT_0_HANGUL_CHO,
    jung_map: &FONT_0_HANGUL_JUNG,
    jong_map: &FONT_0_HANGUL_JONG,
    widths: &HANCOM_GOTHIC_WIDTHS,
};

// 한컴 윤고딕 240: HANYGO240.ttf의 hmtx 원 전진폭. 한글 11172음절은 모두 900/1000em.
// 독립 한컴 PDF의 12pt 한글 전진폭과 일치하며, 결측 글리프는 0으로 보존한다.
static HANYGODIC240_0: [u16; 95] = [
    350, 408, 415, 611, 602, 857, 748, 270, 380, 380, 480, 680, 350, 660, 350, 480, 750, 479, 601,
    619, 621, 619, 628, 558, 636, 628, 350, 350, 500, 660, 500, 689, 860, 733, 702, 773, 733, 664,
    635, 792, 743, 308, 543, 714, 611, 859, 752, 794, 680, 800, 693, 664, 645, 733, 743, 913, 684,
    680, 634, 400, 480, 400, 530, 500, 276, 678, 675, 616, 678, 629, 354, 675, 650, 308, 318, 621,
    308, 883, 649, 629, 672, 675, 429, 575, 354, 640, 600, 799, 578, 600, 566, 450, 400, 450, 900,
];
static HANYGODIC240_1: [u16; 96] = [
    350, 900, 616, 611, 629, 680, 400, 575, 900, 906, 374, 742, 685, 515, 900, 500, 265, 672, 360,
    360, 340, 640, 577, 340, 340, 360, 438, 742, 900, 900, 900, 900, 733, 733, 733, 733, 733, 733,
    1027, 773, 664, 664, 664, 664, 308, 308, 308, 308, 782, 752, 794, 794, 794, 794, 794, 661, 794,
    733, 733, 733, 733, 680, 680, 702, 678, 678, 678, 678, 678, 678, 1027, 616, 629, 629, 629, 629,
    308, 308, 308, 308, 628, 649, 629, 629, 629, 629, 629, 660, 629, 640, 640, 640, 640, 600, 675,
    600,
];
static HANYGODIC240_2: [u16; 112] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 900, 0, 0, 240, 240, 0, 0, 410,
    410, 0, 0, 900, 900, 0, 0, 0, 630, 900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 900, 0, 252, 423, 0, 0, 0,
    0, 0, 0, 0, 900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANYGODIC240_3: [u16; 256] = [
    900, 0, 414, 900, 0, 0, 0, 900, 900, 0, 0, 900, 0, 0, 0, 900, 0, 900, 0, 0, 0, 0, 0, 0, 0, 0,
    900, 0, 0, 900, 900, 0, 900, 0, 0, 0, 0, 900, 0, 900, 900, 900, 900, 900, 900, 0, 900, 0, 0, 0,
    0, 0, 900, 900, 0, 0, 0, 0, 0, 0, 900, 900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 900, 900, 0, 0, 900, 900, 0, 0, 0, 0, 900,
    900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 900, 900, 0, 0, 900,
    900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    900, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANYGODIC240_4: [u16; 64] = [
    900, 900, 900, 900, 0, 0, 0, 0, 430, 430, 450, 450, 360, 360, 360, 360, 450, 450, 0, 900, 297,
    297, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANYGODIC240_5: [u16; 96] = [
    0, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 1000, 900, 900, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 0, 900, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900,
    0,
];
static HANYGODIC240_6: [u16; 240] = [
    0, 900, 900, 900, 900, 900, 900, 900, 390, 390, 900, 900, 900, 900, 900, 480, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900,
    900, 900, 390, 900, 390, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900,
    900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 450, 900, 450, 900,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 900, 900, 900, 900, 0, 900, 900, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static HANYGODIC240_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 32,
        end: 126,
        widths: &HANYGODIC240_0,
    },
    LatinRange {
        start: 160,
        end: 255,
        widths: &HANYGODIC240_1,
    },
    LatinRange {
        start: 8192,
        end: 8303,
        widths: &HANYGODIC240_2,
    },
    LatinRange {
        start: 8704,
        end: 8959,
        widths: &HANYGODIC240_3,
    },
    LatinRange {
        start: 12288,
        end: 12351,
        widths: &HANYGODIC240_4,
    },
    LatinRange {
        start: 12592,
        end: 12687,
        widths: &HANYGODIC240_5,
    },
    LatinRange {
        start: 65280,
        end: 65519,
        widths: &HANYGODIC240_6,
    },
];
static HANYGODIC240_WIDTHS: [u16; 1] = [900];
static HANYGODIC240_HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &FONT_0_HANGUL_CHO,
    jung_map: &FONT_0_HANGUL_JUNG,
    jong_map: &FONT_0_HANGUL_JONG,
    widths: &HANYGODIC240_WIDTHS,
};

/// [#7702] `HCI Poppy` 의 `·`(U+00B7) — 한컴 정본 실측 786/2048 = 0.384 em.
///
/// `HCI Poppy` 는 한/글이 자기 HFT 글리프로 그리는 영문 글꼴이고, rhwp 에서는 표시와 폭
/// 모두 `Palatino Linotype` 으로 대신한다. ASCII 는 그 표가 이미 맞지만(정본 24,662자, 중앙
/// 오차 0.004 em — #7391) 가운뎃점만 다르다. `Palatino Linotype` 의 `periodcentered` 는
/// 512/2048 = 0.25 em 인데, 한컴 PDF 의 HFT Type3 영문 글꼴(같은 글꼴이 ASCII·`“”`·`·` 를
/// 함께 그린다)은 `·` 를 0.38 em 으로 전진시킨다.
///
/// ```text
///   pdf/**  한글 휴먼명조 뒤 Type3 영문 글꼴의 `·`   60문서 n=381   전진 중앙 0.381 em
///           같은 Type3 의 `/W` 비 `·`/`0` = 3.200/4.1667 = 0.768 → 0.384 em (`0` = 0.5 em)
///           같은 글꼴의 `“` 0.49 · `‘` 0.27 em        ← Palatino 1024·569/2048 과 일치
/// ```
///
/// 한양신명조의 `·`(393/1024)와 같은 값이다. 굵은·기울인 변형은 실측이 없어 Palatino 표를
/// 그대로 쓴다.
static HCI_POPPY_B7: [u16; 1] = [786];
static HCI_POPPY_LATIN_RANGES: [LatinRange; 5] = [
    LatinRange {
        start: 0x00B7,
        end: 0x00B7,
        widths: &HCI_POPPY_B7,
    },
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &FONT_455_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_455_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_455_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_455_LATIN_3,
    },
];

// 공개 OFL 프로그램의 SFNT cmap/hmtx 직접 추출. 화면 위치에서 역산하지 않는다.
// Noto Sans Symbols2 Regular, em1000, SHA-256:
// 876598db5f1e3bef2bc77c4d94c5b0d5d2c1620be4878a06f18fb5434e331688.
// 전체 증거: mydocs/pr/assets/planet6897_followup_20261011/shadow-circle-font/font-program-evidence.json.
static NOTO_SYMBOLS2_SHADOWED_CIRCLES: [u16; 2] = [916, 916];
static NOTO_SYMBOLS2_CIRCLE_RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x1F53E,
    end: 0x1F53F,
    widths: &NOTO_SYMBOLS2_SHADOWED_CIRCLES,
}];

static MEASURED_FONT_METRIC_OVERLAYS: [FontMetric; 18] = [
    FontMetric {
        name: "HanyangSinMyeongJo",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGSINMYEONGJO_LATIN_RANGES,
        hangul: Some(&FONT_276_HANGUL),
    },
    FontMetric {
        name: "HanyangJungGothic",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGJUNGGOTHIC_LATIN_RANGES,
        hangul: Some(&FONT_267_HANGUL),
    },
    FontMetric {
        name: "HanyangKyunMyeongJo",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGKYUNMYEONGJO_LATIN_RANGES,
        hangul: Some(&FONT_271_HANGUL),
    },
    FontMetric {
        name: "HanyangKyunGothic",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGKYUNGOTHIC_LATIN_RANGES,
        hangul: Some(&FONT_266_HANGUL),
    },
    FontMetric {
        name: "HumanMyeongJo",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HUMANMYEONGJO_LATIN_RANGES,
        hangul: Some(&FONT_276_HANGUL),
    },
    // [#6036] 한컴 윤고딕 계열 — 동봉 TTF hmtx 실측(upem 1000). 이름은 HWPX
    // 저장 원문("한컴 윤고딕 NNN")으로 두어 별칭 사영 없이 직접 매칭된다.
    FontMetric {
        name: "한컴 윤고딕 720",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANYOONGOTHIC720_LATIN_RANGES,
        hangul: Some(&HANYOONGOTHIC720_HANGUL),
    },
    FontMetric {
        name: "한컴 윤고딕 740",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANYOONGOTHIC740_LATIN_RANGES,
        hangul: Some(&HANYOONGOTHIC740_HANGUL),
    },
    FontMetric {
        name: "한컴 윤고딕 760",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANYOONGOTHIC760_LATIN_RANGES,
        hangul: Some(&HANYOONGOTHIC760_HANGUL),
    },
    FontMetric {
        name: "신명 중명조",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &SINMYEONG_JUNGMYEONGJO_LATIN_RANGES,
        hangul: None,
    },
    FontMetric {
        name: "한컴 고딕",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANCOM_GOTHIC_REGULAR_RANGES,
        hangul: Some(&HANCOM_GOTHIC_HANGUL),
    },
    FontMetric {
        name: "한컴 고딕",
        bold: true,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANCOM_GOTHIC_BOLD_RANGES,
        hangul: Some(&HANCOM_GOTHIC_HANGUL),
    },
    FontMetric {
        name: "한컴 윤고딕 240",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANYGODIC240_RANGES,
        hangul: Some(&HANYGODIC240_HANGUL),
    },
    FontMetric {
        name: "Haan YGodic 240",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &HANYGODIC240_RANGES,
        hangul: Some(&HANYGODIC240_HANGUL),
    },
    // [#7702] HCI Poppy — Palatino Linotype 표에 실측 `·` 만 얹는다(위 `HCI_POPPY_B7`).
    FontMetric {
        name: "HCI Poppy",
        bold: false,
        italic: false,
        em_size: 2048,
        latin_ranges: &HCI_POPPY_LATIN_RANGES,
        hangul: None,
    },
    FontMetric {
        name: "HCI Poppy",
        bold: false,
        italic: true,
        em_size: 2048,
        latin_ranges: &FONT_456_LATIN_RANGES,
        hangul: None,
    },
    FontMetric {
        name: "HCI Poppy",
        bold: true,
        italic: false,
        em_size: 2048,
        latin_ranges: &FONT_457_LATIN_RANGES,
        hangul: None,
    },
    FontMetric {
        name: "HCI Poppy",
        bold: true,
        italic: true,
        em_size: 2048,
        latin_ranges: &FONT_458_LATIN_RANGES,
        hangul: None,
    },
    FontMetric {
        name: "Noto Sans Symbols2",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &NOTO_SYMBOLS2_CIRCLE_RANGES,
        hangul: None,
    },
];

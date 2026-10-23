//! 한컴 Print에서 확인한 신명 HFT 프로그램의 고유 ASCII 전진폭.
//!
//! Type3 /Widths와 ToUnicode를 읽은 값이며 글리프 윤곽선이나 글꼴 파일은 아니다.
//! 근거: pdf/issue7642-hft-char-style-diagnostic/char-style-ascii-2020.pdf,
//! Hancom 11.0.0.9136, Print method 0. 생성·해시는 Stage92 기록을 따른다.
//! 굵기 간격은 intrinsic 폭과 분리하여 문자 측정 walk에서 적용한다.

use super::{FontMetric, LatinRange};

#[path = "font_metric_konghan_hft.rs"]
mod konghan;

static ASCII_0: [u16; 95] = [
    400, 500, 400, 660, 660, 1000, 1000, 400, 500, 500, 500, 1000, 400, 500, 400, 500, 660, 660,
    660, 660, 660, 660, 660, 660, 660, 660, 500, 500, 500, 1000, 500, 660, 1000, 782, 713, 764,
    771, 761, 730, 819, 771, 340, 585, 710, 689, 872, 769, 818, 719, 819, 767, 712, 746, 771, 726,
    988, 683, 716, 742, 500, 500, 500, 500, 500, 400, 662, 622, 565, 622, 618, 480, 620, 628, 294,
    294, 562, 294, 896, 627, 621, 623, 622, 467, 574, 456, 627, 567, 783, 573, 559, 649, 500, 500,
    500, 1000,
];
static RANGES_0: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007e,
    widths: &ASCII_0,
}];

static ASCII_1: [u16; 95] = [
    400, 500, 400, 620, 620, 1000, 1000, 400, 500, 500, 500, 1000, 400, 500, 400, 500, 620, 620,
    620, 620, 620, 620, 620, 620, 620, 620, 500, 500, 500, 1000, 500, 620, 1000, 779, 696, 756,
    759, 694, 663, 811, 755, 299, 558, 698, 619, 867, 752, 812, 719, 808, 758, 698, 656, 754, 725,
    1010, 720, 710, 641, 500, 500, 500, 500, 500, 400, 571, 619, 521, 619, 575, 400, 573, 578, 244,
    243, 519, 243, 860, 578, 574, 577, 577, 363, 528, 315, 577, 524, 747, 550, 502, 511, 500, 500,
    500, 1000,
];
static RANGES_1: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007e,
    widths: &ASCII_1,
}];

// 한양신명조 HFT의 정체·굵게 Type3 /Widths는 동일하다.
// 독립 한컴 Print: hft-full-ascii-saved-2020.pdf, stage2394 폭 추출 기록.
static ASCII_2: [u16; 95] = [
    1000, 325, 423, 450, 602, 920, 814, 386, 395, 374, 524, 567, 235, 572, 244, 390, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 276, 292, 729, 570, 734, 511, 920, 794, 704, 690, 749, 677,
    654, 742, 784, 337, 450, 792, 645, 959, 777, 748, 629, 755, 703, 648, 774, 773, 780, 1037, 754,
    769, 599, 362, 397, 352, 429, 500, 382, 509, 556, 485, 543, 482, 352, 588, 560, 277, 314, 557,
    274, 858, 564, 516, 534, 548, 405, 486, 351, 562, 581, 840, 590, 574, 474, 418, 280, 421, 500,
];
static RANGES_2: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007e,
    widths: &ASCII_2,
}];

// 명시적 대체 글꼴이 없는 한양중고딕의 Type3 프로그램 폭이다.
// HWP5 법령의 명시적 HY중고딕 TrueType 대체 프로그램과 구분한다.
static ASCII_3: [u16; 95] = [
    1000, 248, 420, 535, 529, 867, 643, 240, 288, 281, 369, 518, 246, 518, 245, 263, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 247, 245, 583, 519, 582, 524, 999, 632, 629, 699, 693, 649,
    594, 751, 689, 246, 470, 631, 539, 803, 692, 749, 637, 754, 692, 636, 584, 688, 632, 904, 619,
    619, 585, 253, 283, 253, 448, 500, 256, 534, 527, 483, 531, 532, 255, 529, 529, 195, 196, 473,
    195, 809, 529, 539, 535, 531, 310, 480, 255, 531, 479, 697, 470, 466, 485, 310, 237, 306, 500,
];
static RANGES_3: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007e,
    widths: &ASCII_3,
}];

/// HWP3 시절 `고딕` HFT 프로그램의 ASCII 는 모두 반각(0.5em) 고정폭이다.
/// 근거: pdf/hwp3-sample11-hwpx-hwpx-2020.pdf · pdf/hwp3-sample11-hwp-2020.pdf (Hwp 2022 Print)
/// 66쪽 `A.V.Aho, P.J. Weinberger, B. Kernigham` 의 Type3 글꼴 — 문자 코드 0x20–0x7E 의
/// glyph 폭 95개가 모두 500/1000, 글자 원점 간격도 0.494–0.506em(600dpi 격자). HWP3 원본
/// 경로(선언 `고딕`, 비HFT)의 반각 측정과 같은 값이다. 굵게는 다른 HFT 프로그램과 같이
/// 약 0.05em 추가 전진을 받는다(pdf/hwp3-sample4-hwp5-hwp-2020.pdf 3쪽 굵은 한글 1.055em).
static ASCII_GOTHIC: [u16; 95] = [500; 95];
const GOTHIC_METRIC_NAME: &str = "고딕 (HFT)";
static RANGES_GOTHIC: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007e,
    widths: &ASCII_GOTHIC,
}];

pub(super) static METRICS: [FontMetric; 6] = [
    FontMetric {
        name: "신명 디나루 (HFT)",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &RANGES_0,
        hangul: None,
    },
    FontMetric {
        name: "신명 신그래픽 (HFT)",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &RANGES_1,
        hangul: None,
    },
    FontMetric {
        name: "한양신명조 (HFT)",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &RANGES_2,
        hangul: None,
    },
    FontMetric {
        name: "한양중고딕 (HFT)",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &RANGES_3,
        hangul: None,
    },
    FontMetric {
        name: GOTHIC_METRIC_NAME,
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &RANGES_GOTHIC,
        hangul: None,
    },
    konghan::METRIC,
];

/// HFT로 해소한 선언 face에만 원본 프로그램의 메트릭 이름을 공급한다.
pub(crate) fn metric_face_name(requested: &str) -> Option<&'static str> {
    match requested {
        "한양신명조" | "HanyangSinMyeongJo" => Some("한양신명조 (HFT)"),
        "한양중고딕" | "HanyangJungGothic" => Some("한양중고딕 (HFT)"),
        "신명 디나루" => Some("신명 디나루 (HFT)"),
        "신명 신그래픽" => Some("신명 신그래픽 (HFT)"),
        "중간공한" | "Konghan Normal" => Some("중간공한 (HFT)"),
        "고딕" => Some(GOTHIC_METRIC_NAME),
        _ => None,
    }
}

pub(crate) fn is_metric_face(name: &str) -> bool {
    METRICS.iter().any(|metric| metric.name == name)
}

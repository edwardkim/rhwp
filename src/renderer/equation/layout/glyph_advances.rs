//! 실제 표시하는 수학 글리프의 전진폭을 줄 구성과 장식 폭에 사용한다.
//!
//! GUST 라이선스로 배포하는 assets/fonts/LatinModernMath-Regular.woff2의
//! hmtx/cmap에서 추출했다. UPM 1000, 원본 SHA-256:
//! 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! 문서 좌표나 특정 수식의 관측 위치가 아니라 글꼴의 em 메트릭이다.

const ROMAN_ASCII: [u16; 95] = [
    332, 278, 374, 833, 500, 833, 778, 278, 389, 389, 500, 778, 278, 333, 278, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 278, 278, 778, 778, 778, 472, 778, 750, 708, 722, 764, 681,
    653, 785, 750, 361, 514, 778, 625, 917, 750, 778, 681, 778, 736, 556, 722, 750, 750, 1028, 750,
    750, 611, 278, 500, 278, 556, 333, 500, 500, 556, 444, 556, 444, 306, 500, 556, 278, 306, 528,
    278, 833, 556, 500, 556, 528, 392, 394, 389, 556, 528, 722, 528, 528, 444, 500, 278, 500, 556,
];

const ITALIC_UPPER: [u16; 26] = [
    750, 759, 715, 828, 738, 643, 786, 831, 440, 555, 849, 681, 970, 803, 763, 642, 791, 759, 613,
    584, 683, 583, 944, 828, 581, 683,
];

const ITALIC_LOWER: [u16; 26] = [
    529, 429, 433, 520, 466, 490, 477, 576, 345, 412, 521, 298, 878, 600, 485, 503, 446, 451, 469,
    361, 572, 485, 716, 572, 490, 465,
];

const BOLD_ITALIC_UPPER: [u16; 26] = [
    869, 866, 817, 938, 810, 689, 887, 982, 511, 631, 971, 756, 1142, 950, 837, 723, 869, 872, 693,
    637, 800, 678, 1093, 947, 675, 773,
];

const BOLD_ITALIC_LOWER: [u16; 26] = [
    633, 521, 513, 610, 554, 568, 545, 668, 405, 471, 604, 348, 1032, 713, 585, 601, 542, 529, 531,
    415, 681, 567, 831, 659, 590, 555,
];

fn advance_units(ch: char) -> Option<u16> {
    let code = u32::from(ch);
    match code {
        0x20..=0x7E => Some(ROMAN_ASCII[(code - 0x20) as usize]),
        0x1D434..=0x1D44D => Some(ITALIC_UPPER[(code - 0x1D434) as usize]),
        0x1D44E..=0x1D467 if code != 0x1D455 => Some(ITALIC_LOWER[(code - 0x1D44E) as usize]),
        0x210E => Some(ITALIC_LOWER[(b'h' - b'a') as usize]),
        0x1D468..=0x1D481 => Some(BOLD_ITALIC_UPPER[(code - 0x1D468) as usize]),
        0x1D482..=0x1D49B => Some(BOLD_ITALIC_LOWER[(code - 0x1D482) as usize]),
        _ => super::symbol_advances::advance_units(code),
    }
}

pub(super) fn text_width(text: &str, font_size: f64) -> Option<f64> {
    let units = text.chars().try_fold(0_u32, |sum, ch| {
        advance_units(ch).map(|advance| sum + u32::from(advance))
    })?;
    Some(f64::from(units) * font_size / 1000.0)
}

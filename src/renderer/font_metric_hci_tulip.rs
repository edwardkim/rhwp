//! 원본 HCI Tulip 프로그램의 독립 한컴 Print 전진폭.
//!
//! `pdf/issue1950-paper-fixed-footer/issue1950_hwp3_tab_charoffset-2020.pdf`의
//! Type3 T3 /Widths와 ToUnicode에서 확인했다. 숫자498, 하이픈332/1000이다.
//! 미수집 글자는 표시용 프로그램으로 넘기며 윤곽이나 위치 보정값은 포함하지 않는다.

use super::{FontMetric, LatinRange};

static HYPHEN: [u16; 1] = [332];
static DIGITS: [u16; 10] = [498; 10];
static RANGES: [LatinRange; 2] = [
    LatinRange {
        start: 0x002d,
        end: 0x002d,
        widths: &HYPHEN,
    },
    LatinRange {
        start: 0x0030,
        end: 0x0039,
        widths: &DIGITS,
    },
];

pub(super) static METRICS: [FontMetric; 1] = [FontMetric {
    name: "HCI Tulip",
    bold: false,
    italic: false,
    em_size: 1000,
    latin_ranges: &RANGES,
    hangul: None,
}];

//! 원본 HCI Poppy의 독립 한컴 Print Type3 전진폭이다.
//! ASCII 일반/굵은체: mydocs/pr/assets/issue_8160/poppy-ascii-metrics.json.
//! 일반체는 원본 폭 표를 사용한다. 굵은체는 PDF 글리프 Widths와 실제 문자
//! 전진이 다르므로, 왼쪽 정렬·다섯 크기 독립 표본에서 둘의 일치를
//! 확인한 기호만 제공한다. 그 밖의 굵은 문자는 기존 스타일 표를 유지한다.
//! 아래 기존 PDF 근거는 일반체 비ASCII 따옴표에만 적용한다.
//! 표시용 Palatino의 hmtx와 구분하며 미측정 문자는 범위에 넣지 않는다.
//! 기준: pdf/issue7642-hwp3-original-print/hwp3-sample16-2020.pdf
//! SHA-256: f1f2e6dd265d098ea468bae14b44e98a1609dd7eba293f91d73ba16704be73c5
//! Font xref31, Widths xref1738, ToUnicode xref1739, FontMatrix=0.001.

use super::{FontMetric, LatinRange};

static ASCII_WIDTHS: [u16; 95] = [
    250, 277, 369, 605, 498, 839, 777, 277, 332, 332, 388, 605, 248, 332, 248, 605, 498, 498, 498,
    498, 498, 498, 498, 498, 498, 498, 248, 248, 605, 605, 605, 443, 746, 777, 609, 707, 773, 609,
    554, 761, 830, 335, 332, 726, 609, 945, 830, 785, 603, 785, 667, 525, 613, 777, 722, 998, 667,
    667, 667, 332, 605, 332, 605, 498, 277, 498, 552, 443, 609, 478, 332, 554, 582, 291, 232, 554,
    291, 882, 582, 546, 599, 560, 394, 423, 324, 603, 564, 832, 515, 554, 498, 332, 605, 332, 605,
];

static QUOTE_LEFT_WIDTHS: [u16; 1] = [277];
static DOUBLE_QUOTE_WIDTHS: [u16; 2] = [498, 498];

static RANGES: [LatinRange; 3] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &ASCII_WIDTHS,
    },
    LatinRange {
        start: 0x2018,
        end: 0x2018,
        widths: &QUOTE_LEFT_WIDTHS,
    },
    LatinRange {
        start: 0x201C,
        end: 0x201D,
        widths: &DOUBLE_QUOTE_WIDTHS,
    },
];

// Type3 굵은 글리프의 Widths 전체를 전진폭으로 쓰면 영문 제목이 좁아진다.
// 실제 Print 전진과 같은 원본 기호 폭만 제공하고 미확인 범위는 비워 둔다.
static SYMBOL_605: [u16; 1] = [605];
static SYMBOL_277: [u16; 1] = [277];
static SYMBOL_746: [u16; 1] = [746];
static ANGLE_WIDTHS: [u16; 3] = [605, 605, 605];

static BOLD_RANGES: [LatinRange; 9] = [
    LatinRange {
        start: 0x0023,
        end: 0x0023,
        widths: &SYMBOL_605,
    },
    LatinRange {
        start: 0x0027,
        end: 0x0027,
        widths: &SYMBOL_277,
    },
    LatinRange {
        start: 0x002B,
        end: 0x002B,
        widths: &SYMBOL_605,
    },
    LatinRange {
        start: 0x003C,
        end: 0x003E,
        widths: &ANGLE_WIDTHS,
    },
    LatinRange {
        start: 0x0040,
        end: 0x0040,
        widths: &SYMBOL_746,
    },
    LatinRange {
        start: 0x005C,
        end: 0x005C,
        widths: &SYMBOL_605,
    },
    LatinRange {
        start: 0x005E,
        end: 0x005E,
        widths: &SYMBOL_605,
    },
    LatinRange {
        start: 0x0060,
        end: 0x0060,
        widths: &SYMBOL_277,
    },
    LatinRange {
        start: 0x007C,
        end: 0x007C,
        widths: &SYMBOL_605,
    },
];

pub(super) static METRICS: [FontMetric; 2] = [
    FontMetric {
        name: "HCI Poppy",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &RANGES,
        hangul: None,
    },
    FontMetric {
        name: "HCI Poppy",
        bold: true,
        italic: false,
        em_size: 1000,
        latin_ranges: &BOLD_RANGES,
        hangul: None,
    },
];

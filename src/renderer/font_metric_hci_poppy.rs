//! 원본 HCI Poppy regular의 독립 한컴 Print Type3 전진폭이다.
//! 표시용 Palatino의 hmtx와 구분하며 미측정 문자는 범위에 넣지 않는다.
//! 기준: pdf/issue7642-hwp3-original-print/hwp3-sample16-2020.pdf
//! SHA-256: f1f2e6dd265d098ea468bae14b44e98a1609dd7eba293f91d73ba16704be73c5
//! Font xref31, Widths xref1738, ToUnicode xref1739, FontMatrix=0.001.

use super::{FontMetric, LatinRange};

static WIDTHS_0: [u16; 1] = [250];

static WIDTHS_1: [u16; 2] = [369, 605];

static WIDTHS_2: [u16; 2] = [839, 777];

static WIDTHS_3: [u16; 19] = [
    332, 332, 388, 605, 248, 332, 248, 605, 498, 498, 498, 498, 498, 498, 498, 498, 498, 498, 248,
];

static WIDTHS_4: [u16; 4] = [605, 605, 605, 443];

static WIDTHS_5: [u16; 16] = [
    777, 609, 707, 773, 609, 554, 761, 830, 335, 332, 726, 609, 945, 830, 785, 603,
];

static WIDTHS_6: [u16; 7] = [667, 525, 613, 777, 722, 998, 667];

static WIDTHS_7: [u16; 2] = [667, 332];

static WIDTHS_8: [u16; 1] = [332];

static WIDTHS_9: [u16; 1] = [498];

static WIDTHS_10: [u16; 26] = [
    498, 552, 443, 609, 478, 332, 554, 582, 291, 232, 554, 291, 882, 582, 546, 599, 560, 394, 423,
    324, 603, 564, 832, 515, 554, 498,
];

static WIDTHS_11: [u16; 1] = [277];

static WIDTHS_12: [u16; 2] = [498, 498];

static RANGES: [LatinRange; 13] = [
    LatinRange {
        start: 0x0020,
        end: 0x0020,
        widths: &WIDTHS_0,
    },
    LatinRange {
        start: 0x0022,
        end: 0x0023,
        widths: &WIDTHS_1,
    },
    LatinRange {
        start: 0x0025,
        end: 0x0026,
        widths: &WIDTHS_2,
    },
    LatinRange {
        start: 0x0028,
        end: 0x003A,
        widths: &WIDTHS_3,
    },
    LatinRange {
        start: 0x003C,
        end: 0x003F,
        widths: &WIDTHS_4,
    },
    LatinRange {
        start: 0x0041,
        end: 0x0050,
        widths: &WIDTHS_5,
    },
    LatinRange {
        start: 0x0052,
        end: 0x0058,
        widths: &WIDTHS_6,
    },
    LatinRange {
        start: 0x005A,
        end: 0x005B,
        widths: &WIDTHS_7,
    },
    LatinRange {
        start: 0x005D,
        end: 0x005D,
        widths: &WIDTHS_8,
    },
    LatinRange {
        start: 0x005F,
        end: 0x005F,
        widths: &WIDTHS_9,
    },
    LatinRange {
        start: 0x0061,
        end: 0x007A,
        widths: &WIDTHS_10,
    },
    LatinRange {
        start: 0x2018,
        end: 0x2018,
        widths: &WIDTHS_11,
    },
    LatinRange {
        start: 0x201C,
        end: 0x201D,
        widths: &WIDTHS_12,
    },
];

pub(super) static METRICS: [FontMetric; 1] = [FontMetric {
    name: "HCI Poppy",
    bold: false,
    italic: false,
    em_size: 1000,
    latin_ranges: &RANGES,
    hangul: None,
}];

//! 원본 HCI Poppy HFT의 em=512 전진폭이다.
//! 근거: mydocs/pr/assets/issue_8160/poppy-original-widths.json.
//! 독립 한컴 Print의 다섯 크기 일반체·굵은체 927개 전진 관측과 대조했다.
//! PDF Type3 Widths는 floor(width * 1000 / 512)이므로 원래 분수를 복원하지
//! 않고 조판에 쓰면 1800dpi 격자에서 글자마다 폭이 달라질 수 있다.
//! 굵은체도 원본 HFT 전진을 사용한다. Type3 윤곽은 일반체에 stroke를 더해
//! 표시하므로 윤곽의 Widths를 굵은체 전진으로 사용하지 않는다.
//! 표시용 글꼴의 hmtx와 구분하고 확인하지 않은 기울임체에는 적용하지 않는다.

use super::{FontMetric, LatinRange};

static ASCII_WIDTHS: [u16; 95] = [
    128, 142, 189, 310, 255, 430, 398, 142, 170, 170, 199, 310, 127, 170, 127, 310, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 127, 127, 310, 310, 310, 227, 382, 398, 312, 362, 396, 312,
    284, 390, 425, 172, 170, 372, 312, 484, 425, 402, 309, 402, 342, 269, 314, 398, 370, 511, 342,
    342, 342, 170, 310, 170, 310, 255, 142, 255, 283, 227, 312, 245, 170, 284, 298, 149, 119, 284,
    149, 452, 298, 280, 307, 287, 202, 217, 166, 309, 289, 426, 264, 284, 255, 170, 310, 170, 310,
];

static BOLD_ASCII_WIDTHS: [u16; 95] = [
    128, 142, 205, 310, 255, 455, 426, 142, 170, 170, 227, 310, 127, 170, 127, 151, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 127, 127, 310, 310, 310, 227, 382, 398, 342, 370, 426, 312,
    284, 426, 426, 199, 199, 398, 312, 511, 426, 426, 312, 426, 370, 312, 342, 398, 398, 511, 342,
    342, 342, 170, 310, 170, 310, 255, 142, 255, 312, 227, 312, 255, 199, 284, 312, 170, 170, 312,
    170, 455, 312, 284, 312, 312, 199, 227, 170, 312, 284, 426, 255, 284, 255, 158, 310, 158, 310,
];

static QUOTE_LEFT_WIDTHS: [u16; 1] = [142];
static DOUBLE_QUOTE_WIDTHS: [u16; 2] = [255, 255];

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

static BOLD_RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x007E,
    widths: &BOLD_ASCII_WIDTHS,
}];

// 영문 HFT에 없는 곱셈·나눗셈은 독립 Print의 fallback Type3 프로그램을 쓴다.
// ASCII 원본의 em=512에 반올림해 섞지 않고 프로그램의 em=1000을 보존한다.
// 8·10·13·20·26pt 일반체의 이웃 원점과 /Widths를 대조했다. 굵기 전진은
// 다른 계약이므로 이 표는 확인한 일반체만 선택한다.
static SYMBOL_RANGES: [LatinRange; 2] = [
    LatinRange {
        start: 0x00D7,
        end: 0x00D7,
        widths: &[616],
    },
    LatinRange {
        start: 0x00F7,
        end: 0x00F7,
        widths: &[623],
    },
];

pub(super) static METRICS: [FontMetric; 3] = [
    FontMetric {
        name: "HCI Poppy",
        bold: false,
        italic: false,
        em_size: 512,
        latin_ranges: &RANGES,
        hangul: None,
    },
    FontMetric {
        name: "HCI Poppy",
        bold: true,
        italic: false,
        em_size: 512,
        latin_ranges: &BOLD_RANGES,
        hangul: None,
    },
    FontMetric {
        name: "HCI Poppy",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &SYMBOL_RANGES,
        hangul: None,
    },
];

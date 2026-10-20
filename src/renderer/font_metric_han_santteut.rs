//! 한컴산뜻돋움의 실제 전진폭을 Native와 WASM이 함께 소비한다.
//! 공식 설치 글꼴의 hmtx/cmap 메트릭이며 글꼴 바이트·윤곽을 포함하지 않는다.
//! Regular SHA-256: 423874746eb96c13b5255599135447312ed50c624d252eea1c6f3c67a83b681d.
//! Bold SHA-256: b7c90dadfc6459cba605d8556ac281bc205141c5e58c51b6d9fa9e8c5f78c5bc.
//! MCP 2020의 96개 수식 Print PDF에 사용된 같은 face의 subset 전진폭도 대조했다.

use super::{FontMetric, HangulMetric, LatinRange};

static HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &[0; 19],
    jung_map: &[0; 21],
    jong_map: &[0; 28],
    widths: &[932],
};

static REGULAR_ASCII: [u16; 95] = [
    264, 446, 297, 583, 583, 892, 892, 297, 446, 446, 446, 583, 297, 583, 297, 446, 583, 583, 583,
    583, 583, 583, 583, 583, 583, 583, 297, 297, 446, 583, 446, 669, 1052, 644, 627, 639, 721, 596,
    554, 710, 718, 247, 410, 626, 529, 884, 710, 752, 586, 752, 610, 592, 621, 696, 635, 961, 617,
    611, 594, 446, 961, 446, 434, 446, 297, 560, 588, 490, 588, 559, 340, 588, 592, 244, 301, 530,
    244, 892, 592, 577, 588, 588, 383, 475, 357, 592, 530, 788, 528, 530, 473, 446, 446, 446, 669,
];

static REGULAR_LATIN1: [u16; 96] = [
    264, 446, 0, 0, 892, 0, 0, 446, 800, 892, 378, 0, 0, 478, 892, 0, 446, 800, 260, 260, 300, 0,
    800, 446, 446, 260, 385, 0, 892, 892, 892, 669, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 892, 0, 0, 0, 0, 0, 0, 800, 892, 0, 0, 0, 0, 0, 892, 892, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 800, 800, 0, 0, 0, 0, 0, 800, 0,
];

static REGULAR_RANGES: [LatinRange; 2] = [
    LatinRange {
        start: 0x20,
        end: 0x7e,
        widths: &REGULAR_ASCII,
    },
    LatinRange {
        start: 0xa0,
        end: 0xff,
        widths: &REGULAR_LATIN1,
    },
];

static BOLD_ASCII: [u16; 95] = [
    264, 446, 297, 583, 583, 892, 892, 297, 446, 446, 446, 583, 297, 583, 297, 446, 583, 583, 583,
    583, 583, 583, 583, 583, 583, 583, 297, 297, 446, 583, 446, 669, 1052, 644, 627, 639, 721, 596,
    554, 710, 718, 247, 410, 626, 529, 884, 710, 752, 586, 752, 610, 592, 621, 696, 635, 961, 617,
    611, 594, 446, 961, 446, 434, 446, 297, 560, 588, 490, 588, 559, 340, 588, 592, 244, 301, 530,
    244, 892, 592, 577, 588, 588, 383, 475, 357, 592, 530, 788, 528, 530, 473, 446, 446, 446, 669,
];

static BOLD_LATIN1: [u16; 96] = [
    264, 446, 0, 0, 892, 0, 0, 446, 800, 892, 378, 0, 0, 478, 892, 0, 446, 800, 260, 260, 300, 0,
    800, 446, 446, 260, 385, 0, 892, 892, 892, 669, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 892, 0, 0, 0, 0, 0, 0, 800, 892, 0, 0, 0, 0, 0, 892, 892, 0, 0, 0, 0, 0, 0, 892, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 892, 0, 0, 0, 0, 0, 0, 800, 800, 0, 0, 0, 0, 0, 800, 0,
];

static BOLD_RANGES: [LatinRange; 2] = [
    LatinRange {
        start: 0x20,
        end: 0x7e,
        widths: &BOLD_ASCII,
    },
    LatinRange {
        start: 0xa0,
        end: 0xff,
        widths: &BOLD_LATIN1,
    },
];

// 같은 설치본의 SFNT family(1/16) 이름만 직접 색인한다.
// PostScript 이름을 family 별칭이나 Bold face로 간주하지 않는다.
pub(super) static METRICS: [FontMetric; 4] = [
    FontMetric {
        name: "한컴산뜻돋움",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &REGULAR_RANGES,
        hangul: Some(&HANGUL),
    },
    FontMetric {
        name: "한컴산뜻돋움",
        bold: true,
        italic: false,
        em_size: 1000,
        latin_ranges: &BOLD_RANGES,
        hangul: Some(&HANGUL),
    },
    FontMetric {
        name: "Han Santteut Dotum",
        bold: false,
        italic: false,
        em_size: 1000,
        latin_ranges: &REGULAR_RANGES,
        hangul: Some(&HANGUL),
    },
    FontMetric {
        name: "Han Santteut Dotum",
        bold: true,
        italic: false,
        em_size: 1000,
        latin_ranges: &BOLD_RANGES,
        hangul: Some(&HANGUL),
    },
];

//! 독립 한컴 Print로 확인한 시스템 HFT의 고유 전진폭.
//!
//! 일반·굵게, 12·16pt의 한글11172자와 ASCII94자에서 같은 /Widths를 확인했다.
//! 공백은 실제 glyph로 출력되지 않아 표에서 미확인으로 남기며 기존 공백 계약을 쓴다.
//! 굵게의 추가 전진은 공통 HFT 측정 walk가 원점에만 적용한다.
//! 입력 SHA-256: 52a8b3d98bdb4dbfb4a2ea7c9fc3fc2a689ae815c40c7bf2d9c5e52b191d83ad.
//! PDF SHA-256: 55390717754f3c64735c46ff9da8b49c73ff4c81b09f6300a15812e23a5e77dc.
//! Hancom11.0.0.9136/Print0, 표본 PDF·윤곽은 로컬 전용. stage8 C438 참조.

use super::super::{FontMetric, HangulMetric, LatinRange};

static HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &[0; 19],
    jung_map: &[0; 21],
    jong_map: &[0; 28],
    widths: &[880],
};

static ASCII: [u16; 95] = [
    0, 186, 279, 515, 510, 640, 592, 210, 300, 250, 330, 485, 200, 406, 180, 390, 485, 485, 485,
    485, 485, 485, 485, 485, 485, 485, 234, 234, 434, 455, 370, 476, 694, 589, 594, 602, 620, 552,
    520, 628, 616, 190, 428, 584, 476, 748, 627, 628, 572, 628, 600, 564, 550, 608, 582, 808, 566,
    556, 520, 300, 400, 246, 378, 460, 210, 442, 492, 456, 496, 464, 270, 497, 478, 176, 234, 434,
    168, 740, 478, 478, 494, 498, 296, 426, 300, 478, 436, 638, 415, 433, 412, 300, 168, 245, 470,
];

static RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x20,
    end: 0x7e,
    widths: &ASCII,
}];

pub(super) const METRIC: FontMetric = FontMetric {
    name: "시스템 (HFT)",
    bold: false,
    italic: false,
    em_size: 1000,
    latin_ranges: &RANGES,
    hangul: Some(&HANGUL),
};

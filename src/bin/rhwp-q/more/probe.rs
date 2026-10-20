//! `volume-probe`의 공통 문서 순회와 slot 규칙.
//!
//! 이전 구현은 50개 shard마다 280개의 거의 같은 문서 순회 함수를 생성했다. 각 함수의
//! 차이는 slot·probe 번호와 Control 특징 하나에 적용하는 3배 가중치뿐이므로, 이 모듈은
//! 문서를 한 번만 순회하고 같은 wrapping 산술을 규칙 테이블로 재현한다.

use rhwp::model::control::Control;
use rhwp::model::document::Document;

const SLOT_COUNT: usize = 50;
const PROBES_PER_SLOT: usize = 280;
const SLOT_SEED_FACTOR: u64 = 10_009;
const SLOT_FEATURE_STRIDE: usize = 10;

#[derive(Clone, Copy)]
#[repr(usize)]
enum Feature {
    Hyperlink,
    Ruby,
    Picture,
    Equation,
    PageHide,
    IndexMark,
    AutoNumber,
    CharOverlap,
    Form,
    Table,
    Field,
    Bookmark,
    HiddenComment,
    Header,
    Footer,
    Footnote,
    Endnote,
    Shape,
}

const FEATURE_ORDER: [Feature; 18] = [
    Feature::Hyperlink,
    Feature::Ruby,
    Feature::Picture,
    Feature::Equation,
    Feature::PageHide,
    Feature::IndexMark,
    Feature::AutoNumber,
    Feature::CharOverlap,
    Feature::Form,
    Feature::Table,
    Feature::Field,
    Feature::Bookmark,
    Feature::HiddenComment,
    Feature::Header,
    Feature::Footer,
    Feature::Footnote,
    Feature::Endnote,
    Feature::Shape,
];

struct ProbeStats {
    common: u64,
    feature_totals: [u64; FEATURE_ORDER.len()],
}

impl ProbeStats {
    fn add_common(&mut self, value: u64) {
        self.common = self.common.wrapping_add(value);
    }

    fn add_feature(&mut self, feature: Feature, value: u64) {
        self.add_common(value);
        let total = &mut self.feature_totals[feature as usize];
        *total = total.wrapping_add(value);
    }
}

/// 기존 생성 shard가 관찰하던 값들을 문서 한 번의 순회에서 모은다.
fn collect_stats(doc: &Document) -> ProbeStats {
    let mut stats = ProbeStats {
        common: 0,
        feature_totals: [0; FEATURE_ORDER.len()],
    };

    for section in &doc.sections {
        stats.add_common(section.paragraphs.len() as u64);
        for paragraph in &section.paragraphs {
            stats.add_common(paragraph.controls.len() as u64);
            stats.add_common(paragraph.text.chars().count() as u64);

            for control in &paragraph.controls {
                match control {
                    Control::Hyperlink(value) => stats.add_feature(
                        Feature::Hyperlink,
                        value.url.len() as u64 + value.text.len() as u64,
                    ),
                    Control::Ruby(value) => stats.add_feature(
                        Feature::Ruby,
                        value.main_text.len() as u64
                            + value.ruby_text.len() as u64
                            + u64::from(value.sz_ratio),
                    ),
                    Control::Picture(value) => stats.add_feature(
                        Feature::Picture,
                        u64::from(value.instance_id)
                            + u64::from(value.common.width as u32)
                            + u64::from(value.reverse)
                            + u64::from(value.lock),
                    ),
                    Control::Equation(value) => stats.add_feature(
                        Feature::Equation,
                        value.script.len() as u64
                            + u64::from(value.font_size)
                            + value.font_name.len() as u64,
                    ),
                    Control::PageHide(value) => stats.add_feature(
                        Feature::PageHide,
                        u64::from(value.hide_header)
                            + u64::from(value.hide_footer)
                            + u64::from(value.hide_page_num),
                    ),
                    Control::IndexMark(value) => stats.add_feature(
                        Feature::IndexMark,
                        value.first_key.len() as u64 + value.second_key.len() as u64,
                    ),
                    Control::AutoNumber(value) => stats.add_feature(
                        Feature::AutoNumber,
                        u64::from(value.number) + u64::from(value.format),
                    ),
                    Control::CharOverlap(value) => {
                        stats.add_feature(Feature::CharOverlap, value.chars.len() as u64)
                    }
                    Control::Form(value) => stats.add_feature(
                        Feature::Form,
                        value.name.len() as u64
                            + value.text.len() as u64
                            + u64::from(value.enabled),
                    ),
                    Control::Table(value) => {
                        stats.add_feature(
                            Feature::Table,
                            u64::from(value.row_count)
                                + u64::from(value.col_count)
                                + value.cells.len() as u64,
                        );
                        stats.add_common(value.cells.len() as u64);
                        for cell in &value.cells {
                            stats.add_common(cell.paragraphs.len() as u64);
                            for paragraph in &cell.paragraphs {
                                stats.add_common(paragraph.text.chars().count() as u64);
                                stats.add_common(paragraph.controls.len() as u64);
                            }
                        }
                    }
                    Control::Field(value) => stats.add_feature(
                        Feature::Field,
                        value.command.len() as u64 + u64::from(value.field_id),
                    ),
                    Control::Bookmark(value) => {
                        stats.add_feature(Feature::Bookmark, value.name.len() as u64)
                    }
                    Control::HiddenComment(value) => {
                        stats.add_feature(Feature::HiddenComment, value.paragraphs.len() as u64)
                    }
                    Control::Header(value) => {
                        stats.add_feature(Feature::Header, value.paragraphs.len() as u64)
                    }
                    Control::Footer(value) => {
                        stats.add_feature(Feature::Footer, value.paragraphs.len() as u64)
                    }
                    Control::Footnote(value) => {
                        stats.add_feature(Feature::Footnote, value.paragraphs.len() as u64)
                    }
                    Control::Endnote(value) => {
                        stats.add_feature(Feature::Endnote, value.paragraphs.len() as u64)
                    }
                    Control::Shape(value) => {
                        stats.add_feature(Feature::Shape, u64::from(value.common().width as u32))
                    }
                    _ => stats.add_common(1),
                }
            }
        }
    }

    stats
}

pub fn probe_slot(slot: u32, doc: &Document) -> u64 {
    debug_assert!((slot as usize) < SLOT_COUNT);

    let stats = collect_stats(doc);
    let slot = slot as usize;
    let seed = (slot as u64).wrapping_mul(SLOT_SEED_FACTOR);
    let mut acc = 0u64;

    for probe in 0..PROBES_PER_SLOT {
        let feature = FEATURE_ORDER[(slot * SLOT_FEATURE_STRIDE + probe) % FEATURE_ORDER.len()];
        let value = seed
            .wrapping_add(probe as u64)
            .wrapping_add(stats.common)
            .wrapping_add(stats.feature_totals[feature as usize].wrapping_mul(2));
        acc = acc.wrapping_add(value);
    }

    acc
}

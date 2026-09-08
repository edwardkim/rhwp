//! [#6840] 참조 없는 `BinData` 정리 — 발췌·부분 제출에서 파일이 줄지 않던 문제.
//!
//! `extract-pages` 는 본문·서식은 제대로 걷어내는데 `BinData`(그림) 스트림은 한 개도
//! 버리지 않았다. 그림이 많은 문서는 쪽을 한 장만 남겨도 파일이 거의 그대로다.
//!
//! ```text
//!   104쪽 6.5MB 문서에서 77쪽 한 장만 남긴 실측
//!                      파일        BinData            그 외
//!     원본           6,521,856   204개 6,177,814     209,205
//!     --from 77 --to 77  5,225,984   204개 5,078,167      24,184
//!                        ↑ 본문은 88% 줄었는데 파일은 20%       ↑ 스트림 0개 삭제
//! ```
//!
//! 이 모듈은 산출 문서가 실제로 참조하는 항목만 남기고 나머지를 버린다.
//!
//! ## 왜 재번호매김이 필요한가
//!
//! `ImageAttr.bin_data_id` 는 **1 기준 순번(위치)** 이라 `bin_data_content` 의 인덱스다
//! (`renderer::layout::utils::find_bin_data_index`). 항목을 지우면 뒤의 참조가 통째로
//! 한 칸씩 밀려 **그림이 조용히 뒤바뀐다.** 그래서 지우기와 동시에 모든 참조를 옮긴다.
//!
//! `BinDataContent.id`(= `BinData.storage_id`, 저장 시 `BIN%04X` 스트림 이름)는 순번과
//! 다른 값이다. 이 모듈은 storage id 를 **건드리지 않는다** — 이름이 바뀌면 스트림과
//! 레코드가 어긋나고, 구멍이 있는 문서에서 기존 id 와 충돌한다.
//!
//! ## 참조처
//!
//! 순회 구조는 `converters::hwpx_to_hwp` 의 `walk_controls`(프로덕션에서 `OleShape` 를
//! 실제로 갈아끼우는 워커)를 그대로 따른다 — 담는 그릇 목록을 두 벌로 두지 않는다.
//!
//! | 참조처 | 어디에 |
//! | --- | --- |
//! | `Picture.image_attr.bin_data_id` | `Control::Picture` · `ShapeObject::Picture` |
//! | `DrawingObjAttr.fill.image.bin_data_id` | 모든 도형의 채우기 |
//! | `BorderFill.fill.image.bin_data_id` | **DocInfo** — 문단 밖이라 항상 참조로 센다 |
//! | `OleShape.bin_data_id` | OLE 개체 (`u32`) |
//!
//! `BorderFill` 은 문단이 아니라 DocInfo 에 있고 `border_fill_id` 로 참조되므로, 어떤
//! 문단이 남았는지와 무관하게 **살아 있는 참조로 본다.** 그쪽까지 걷어내려면 문단·셀·
//! 글자 모양의 `border_fill_id` 를 전수로 다시 세야 하는데, 이 명령의 목적(발췌 크기
//! 줄이기)에 비해 위험이 크다.
//!
//! ## ⚠ 내장 글꼴은 **순번이 아니라 storage id** 로 참조한다
//!
//! `Font::resolved_bin_data_id` 와 `SubstFont::resolved_bin_data_id` 는 `BinDataContent.id`
//! 다 — `load_bounded_embedded_font_bytes` 가 `content.id == font_id` 로 찾는다
//! (`queries::rendering`). 그림의 1 기준 순번과 **다른 축**이므로
//!
//! - 그 storage id 를 가진 항목은 참조가 없어도 남긴다
//! - 그 참조값은 **옮기지 않는다** (storage id 는 이 정리에서 불변이다)
//!
//! 이 축을 빠뜨리면 발췌본에서 내장 글꼴이 통째로 사라진다.

use std::collections::BTreeSet;

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::shape::{DrawingObjAttr, ShapeObject};

/// 정리 결과 요약.
///
/// 버린 바이트는 세지 않는다 — `BinDataBytes::len()` 은 `Lazy` 항목을 압축 해제하므로
/// 버리려고 훑는 것만으로 원본 전체를 메모리에 펴게 된다.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BinDataPruneReport {
    /// 정리 전 항목 수
    pub before: usize,
    /// 정리 후 항목 수
    pub after: usize,
}

impl BinDataPruneReport {
    /// 버린 항목 수.
    pub fn removed(&self) -> usize {
        self.before.saturating_sub(self.after)
    }
}

/// 문서 안의 모든 `u16` `bin_data_id` 참조를 방문한다.
///
/// `f` 가 값을 바꾸면 그대로 반영된다 — 모으기와 옮기기가 **같은 순회**를 쓰므로 두
/// 단계가 서로 다른 자리를 볼 수 없다.
fn visit_bin_data_refs(doc: &mut Document, f: &mut dyn FnMut(&mut u16)) {
    fn visit_drawing(drawing: &mut DrawingObjAttr, f: &mut dyn FnMut(&mut u16)) {
        if let Some(image) = drawing.fill.image.as_mut() {
            f(&mut image.bin_data_id);
        }
        if let Some(text_box) = drawing.text_box.as_mut() {
            visit_paragraphs(&mut text_box.paragraphs, f);
        }
        if let Some(caption) = drawing.caption.as_mut() {
            visit_paragraphs(&mut caption.paragraphs, f);
        }
    }

    fn visit_shape(shape: &mut ShapeObject, f: &mut dyn FnMut(&mut u16)) {
        match shape {
            // `drawing_mut()` 이 `None` 을 주는 두 변형은 각자 캡션을 갖는다.
            ShapeObject::Picture(pic) => {
                f(&mut pic.image_attr.bin_data_id);
                if let Some(caption) = pic.caption.as_mut() {
                    visit_paragraphs(&mut caption.paragraphs, f);
                }
            }
            ShapeObject::Group(group) => {
                for child in &mut group.children {
                    visit_shape(child, f);
                }
                if let Some(caption) = group.caption.as_mut() {
                    visit_paragraphs(&mut caption.paragraphs, f);
                }
            }
            ShapeObject::Chart(chart) => {
                visit_drawing(&mut chart.drawing, f);
                if let Some(caption) = chart.caption.as_mut() {
                    visit_paragraphs(&mut caption.paragraphs, f);
                }
            }
            ShapeObject::Ole(ole) => {
                visit_drawing(&mut ole.drawing, f);
                if let Some(caption) = ole.caption.as_mut() {
                    visit_paragraphs(&mut caption.paragraphs, f);
                }
                // OLE 의 참조는 `u32` 다. `u16` 범위 밖이면 순번 의미가 아니므로
                // 손대지 않는다 — 모으는 쪽도 같은 조건으로 건너뛴다.
                if let Ok(mut id) = u16::try_from(ole.bin_data_id) {
                    f(&mut id);
                    ole.bin_data_id = u32::from(id);
                }
            }
            _ => {
                if let Some(drawing) = shape.drawing_mut() {
                    visit_drawing(drawing, f);
                }
            }
        }
    }

    fn visit_controls(controls: &mut [Control], f: &mut dyn FnMut(&mut u16)) {
        for control in controls {
            match control {
                Control::Picture(pic) => {
                    f(&mut pic.image_attr.bin_data_id);
                    if let Some(caption) = pic.caption.as_mut() {
                        visit_paragraphs(&mut caption.paragraphs, f);
                    }
                }
                Control::Shape(shape) => visit_shape(shape, f),
                Control::Table(table) => {
                    for cell in &mut table.cells {
                        visit_paragraphs(&mut cell.paragraphs, f);
                    }
                    if let Some(caption) = table.caption.as_mut() {
                        visit_paragraphs(&mut caption.paragraphs, f);
                    }
                }
                Control::Header(h) => visit_paragraphs(&mut h.paragraphs, f),
                Control::Footer(h) => visit_paragraphs(&mut h.paragraphs, f),
                Control::Footnote(n) => visit_paragraphs(&mut n.paragraphs, f),
                Control::Endnote(n) => visit_paragraphs(&mut n.paragraphs, f),
                Control::HiddenComment(c) => visit_paragraphs(&mut c.paragraphs, f),
                Control::Field(field) => visit_paragraphs(&mut field.memo_paragraphs, f),
                Control::SectionDef(section_def) => {
                    for master_page in &mut section_def.master_pages {
                        visit_paragraphs(&mut master_page.paragraphs, f);
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_paragraphs(
        paragraphs: &mut [crate::model::paragraph::Paragraph],
        f: &mut dyn FnMut(&mut u16),
    ) {
        for para in paragraphs {
            visit_controls(&mut para.controls, f);
        }
    }

    for section in &mut doc.sections {
        visit_paragraphs(&mut section.paragraphs, f);
    }
    // DocInfo 의 테두리/배경 이미지 채우기는 문단과 무관하게 살아 있는 참조다.
    for border_fill in &mut doc.doc_info.border_fills {
        if let Some(image) = border_fill.fill.image.as_mut() {
            f(&mut image.bin_data_id);
        }
    }
}

/// 산출 문서가 참조하지 않는 `BinData` 를 버리고 남은 참조를 옮긴다.
///
/// 참조가 없거나 버릴 것이 없으면 문서를 건드리지 않는다.
pub(crate) fn prune_unreferenced_bin_data(doc: &mut Document) -> BinDataPruneReport {
    let before = doc.bin_data_content.len();
    if before == 0 {
        return BinDataPruneReport::default();
    }

    // 1. 살아 있는 1 기준 순번을 모은다. 범위 밖 값(HWPX 차트 sparse id 등)은
    //    인덱스 의미가 아니므로 건드리지 않고 그대로 둔다.
    let mut referenced: BTreeSet<usize> = BTreeSet::new();
    visit_bin_data_refs(doc, &mut |id| {
        if *id > 0 && (*id as usize) <= before {
            referenced.insert(*id as usize - 1);
        }
    });

    // 1b. 내장 글꼴이 쓰는 storage id 를 가진 항목도 남긴다 — 순번 축이 아니므로
    //     참조값은 건드리지 않고 보존 대상에만 더한다.
    let font_storage: BTreeSet<u16> = doc
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .flat_map(|font| {
            [
                font.is_embedded
                    .then_some(font.resolved_bin_data_id)
                    .flatten(),
                font.subst_font
                    .as_ref()
                    .and_then(|sf| sf.is_embedded.then_some(sf.resolved_bin_data_id).flatten()),
            ]
        })
        .flatten()
        .collect();
    if !font_storage.is_empty() {
        for (index, content) in doc.bin_data_content.iter().enumerate() {
            if font_storage.contains(&content.id) {
                referenced.insert(index);
            }
        }
    }

    if referenced.len() == before {
        return BinDataPruneReport {
            before,
            after: before,
        };
    }

    // 2. 옛 인덱스 → 새 인덱스. 남는 항목의 상대 순서는 유지한다.
    let mut new_index = vec![None; before];
    for (next, old) in referenced.iter().enumerate() {
        new_index[*old] = Some(next);
    }

    // 3. 참조를 옮긴다. 살아 있는 참조는 반드시 새 자리를 갖는다.
    visit_bin_data_refs(doc, &mut |id| {
        if *id == 0 || (*id as usize) > before {
            return;
        }
        if let Some(next) = new_index[*id as usize - 1] {
            *id = (next + 1) as u16;
        }
    });

    // 4. 두 목록을 같은 규칙으로 거른다 — 인덱스 정합이 깨지면 그림이 뒤바뀐다.
    let dropped_storage: BTreeSet<u16> = doc
        .bin_data_content
        .iter()
        .enumerate()
        .filter(|(i, _)| new_index[*i].is_none())
        .map(|(_, c)| c.id)
        .collect();

    let mut i = 0usize;
    doc.bin_data_content.retain(|_| {
        let keep = new_index[i].is_some();
        i += 1;
        keep
    });

    // `bin_data_list` 는 storage id 로 짝지어져 있다 — 남은 content 가 쓰지 않는
    // storage id 의 레코드만 버린다.
    let live_storage: BTreeSet<u16> = doc.bin_data_content.iter().map(|c| c.id).collect();
    doc.doc_info.bin_data_list.retain(|bd| {
        !dropped_storage.contains(&bd.storage_id) || live_storage.contains(&bd.storage_id)
    });

    BinDataPruneReport {
        before,
        after: doc.bin_data_content.len(),
    }
}

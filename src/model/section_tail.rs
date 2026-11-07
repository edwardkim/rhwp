//! HWP5 구역 끝 목록의 저장 소유. 적용/렌더링 플래그와 저장 위치는 별개다.

use super::control::{Control, Field};
use super::document::{Document, RawRecord, Section};
use super::paragraph::Paragraph;

/// 미해석 정보까지 포함하는 원본 목록. 본문 편집으로 이 범위를 버리지 않는다.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SectionTail {
    pub records: Vec<RawRecord>,
    /// 마지막 구역 스트림이 보존하는 문서 전체 메모 본문의 봉인.
    #[serde(skip)]
    pub(crate) memo_seal: Option<[u8; 32]>,
    /// 로드 후 모델 정규화가 끝난 시점의 바탕쪽 투영.
    #[serde(skip)]
    pub(crate) projection_seal: Option<[u8; 32]>,
}

impl SectionTail {
    /// 한 메모는 같은 레벨의 MEMO_LIST와 LIST_HEADER, 이어지는 문단으로 구성된다.
    /// 다른 목록과 미해석 레코드도 원본 순서로 돌려준다.
    pub(crate) fn record_groups(&self) -> impl Iterator<Item = (bool, &[RawRecord])> {
        use crate::parser::tags::{HWPTAG_LIST_HEADER, HWPTAG_MEMO_LIST};

        let mut remaining = self.records.as_slice();
        std::iter::from_fn(move || {
            let head = remaining.first()?;
            let memo = head.level == 1
                && head.tag_id == HWPTAG_MEMO_LIST
                && head.data.len() >= 4
                && remaining
                    .get(1)
                    .is_some_and(|r| r.level == 1 && r.tag_id == HWPTAG_LIST_HEADER);
            let headers = if memo { 2 } else { 1 };
            let length = remaining[headers..]
                .iter()
                .position(|r| {
                    r.level == 1 && matches!(r.tag_id, HWPTAG_MEMO_LIST | HWPTAG_LIST_HEADER)
                })
                .map_or(remaining.len(), |offset| headers + offset);
            let (group, rest) = remaining.split_at(length);
            remaining = rest;
            Some((memo, group))
        })
    }
}

pub(crate) fn projection_digest(doc: &Document) -> [u8; 32] {
    let masters: Vec<_> = doc
        .sections
        .iter()
        .map(|s| &s.section_def.master_pages)
        .collect();
    crate::model::raw_provenance::value_digest(&masters, "구역 끝 바탕쪽 투영")
}

pub(crate) fn validate_preservation(doc: &Document) -> Result<(), String> {
    if !doc.sections.iter().any(|s| s.trailing_lists.is_some()) {
        return Ok(());
    }
    let digest = projection_digest(doc);
    for (index, section) in doc.sections.iter().enumerate() {
        if let Some(tail) = &section.trailing_lists {
            if tail.projection_seal != Some(digest) {
                return Err(format!("구역 {index}의 원본 바탕쪽 목록과 현재 모델의 참조가 달라졌습니다. 이 목록 자체의 재작성은 아직 지원하지 않습니다."));
            }
        }
    }
    Ok(())
}

impl Field {
    pub(crate) fn memo_index_from_command(&self) -> u32 {
        if self.memo_index != 0 {
            return self.memo_index;
        }
        self.command
            .strip_prefix("MEMO/")
            .and_then(|c| c.split('/').nth(1))
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    }
}

pub(crate) fn memo_digest(lists: &[(u32, &[Paragraph])]) -> [u8; 32] {
    crate::model::raw_provenance::value_digest(lists, "문서 메모 목록")
}
pub(crate) fn collect<'a, S: crate::storage::Storage>(
    sections: &'a [Section],
    storage: &S,
) -> Result<Vec<(u32, &'a [Paragraph])>, S::Error> {
    fn collect<'a, S: crate::storage::Storage>(
        paragraphs: &'a [Paragraph],
        lists: &mut Vec<(u32, &'a [Paragraph])>,
        storage: &S,
    ) -> Result<(), S::Error> {
        for para in paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Field(field)
                        if (field.field_type == crate::model::control::FieldType::Memo
                            || field.command.starts_with("MEMO/"))
                            && !field.memo_paragraphs.is_empty() =>
                    {
                        // The tail is keyed by the memo number; a field that kept
                        // only its command (HWPX conversion) must still match.
                        storage.push(
                            lists,
                            (
                                field.memo_index_from_command(),
                                field.memo_paragraphs.as_slice(),
                            ),
                        )?;
                    }
                    Control::Table(table) => {
                        for cell in &table.cells {
                            collect(&cell.paragraphs, lists, storage)?;
                        }
                    }
                    Control::Shape(shape) => collect_shape_memos(shape, lists, storage)?,
                    Control::Header(header) => collect(&header.paragraphs, lists, storage)?,
                    Control::Footer(footer) => collect(&footer.paragraphs, lists, storage)?,
                    Control::Footnote(note) => collect(&note.paragraphs, lists, storage)?,
                    Control::Endnote(note) => collect(&note.paragraphs, lists, storage)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn collect_shape_memos<'a, S: crate::storage::Storage>(
        shape: &'a crate::model::shape::ShapeObject,
        lists: &mut Vec<(u32, &'a [Paragraph])>,
        storage: &S,
    ) -> Result<(), S::Error> {
        if let Some(drawing) = shape.drawing() {
            if let Some(text_box) = &drawing.text_box {
                collect(&text_box.paragraphs, lists, storage)?;
            }
        }
        if let crate::model::shape::ShapeObject::Group(group) = shape {
            for child in &group.children {
                collect_shape_memos(child, lists, storage)?;
            }
        }
        Ok(())
    }

    let mut memo_lists = storage.vec(0)?;
    for section in sections {
        collect(&section.paragraphs, &mut memo_lists, storage)?;
    }
    Ok(memo_lists)
}

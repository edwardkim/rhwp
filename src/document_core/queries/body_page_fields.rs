//! 본문 쪽번호 필드의 실제 줄 소속에서 표시값을 확정하고 측정·배치에 함께 전달한다.
//!
//! 원문 text/UTF-16 축과 파일의 num 속성은 보존한다. 값이 줄 나눔을 바꾸면 같은
//! composer/frame 입력으로 다시 계산한다. 순환/과도한 반복은 좌표를 맞추지 않고 출력 오류다.
use std::collections::{BTreeMap, HashSet};

use crate::{
    document_core::DocumentCore,
    error::HwpError,
    model::control::{AutoNumber, AutoNumberType, Control},
    renderer::{
        format_number,
        layout::LayoutEngine,
        render_tree::{RenderNode, RenderNodeType},
        NumberFormat,
    },
};

const MAX_FIELD_RESOLUTION_PASSES: usize = 32;
type FieldKey = (usize, usize, usize); // 구역, 문단, 모델 scalar 위치

#[derive(Clone)]
struct BodyField {
    number: AutoNumber,
    logical_position: usize,
}

fn fields(core: &DocumentCore) -> BTreeMap<FieldKey, BodyField> {
    let mut out = BTreeMap::new();
    for (section_index, section) in core.document.sections.iter().enumerate() {
        for (para_index, para) in section.paragraphs.iter().enumerate() {
            if !para.controls.iter().any(|control| matches!(control,
                Control::AutoNumber(number) if matches!(number.number_type, AutoNumberType::Page | AutoNumberType::TotalPage))) {
                continue;
            }
            for kind in [AutoNumberType::Page, AutoNumberType::TotalPage] {
                let positions = LayoutEngine::resolve_auto_number_placeholder_addresses(para, kind);
                let controls = para.controls.iter().filter_map(|control| match control {
                    Control::AutoNumber(number) if number.number_type == kind => Some(number),
                    _ => None,
                });
                for (position, number) in positions.into_iter().zip(controls) {
                    out.insert(
                        (section_index, para_index, position.source),
                        BodyField {
                            number: number.clone(),
                            logical_position: position.logical,
                        },
                    );
                }
            }
        }
    }
    out
}

fn display(number: &AutoNumber, value: u32) -> Result<String, HwpError> {
    let digits = if number.format == 0 {
        value.to_string()
    } else {
        format_number(
            u16::try_from(value).map_err(|_| {
                HwpError::RenderError("본문 쪽번호 형식의 16비트 범위를 넘었습니다".into())
            })?,
            NumberFormat::from_hwp_format(number.format),
        )
    };
    Ok(format!(
        "{}{}{}",
        if number.prefix_char == '\0' {
            String::new()
        } else {
            number.prefix_char.to_string()
        },
        digits,
        if number.suffix_char == '\0' {
            String::new()
        } else {
            number.suffix_char.to_string()
        }
    ))
}

fn collect_owners(
    node: &RenderNode,
    section: usize,
    page_number: u32,
    candidates: &BTreeMap<FieldKey, BodyField>,
    owners: &mut BTreeMap<FieldKey, u32>,
) -> Result<(), HwpError> {
    // 같은 문단 번호를 쓰더라도 본문과 중첩 흐름은 서로 다른 소유자다.
    if matches!(
        node.node_type,
        RenderNodeType::Header
            | RenderNodeType::Footer
            | RenderNodeType::MasterPage
            | RenderNodeType::TextBox
            | RenderNodeType::FootnoteArea
            | RenderNodeType::Table(_)
    ) {
        return Ok(());
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let (Some(para), Some(start)) = (run.para_index, run.char_start) {
            let end = start + run.logical_char_count();
            // TextRun은 인라인 개체도 한 칸인 편집 주소를 발행한다.
            // 저장 표시값의 원문 키는 유지하고, 소유 판정에서 같은 논리 주소를 쓴다.
            for (&key, field) in candidates.range((section, para, 0)..=(section, para, usize::MAX))
            {
                if field.logical_position < start || field.logical_position >= end {
                    continue;
                }
                if owners
                    .insert(key, page_number)
                    .is_some_and(|old| old != page_number)
                {
                    return Err(HwpError::RenderError(
                        "본문 쪽번호 필드의 줄 소속이 여러 쪽에 중복됩니다".into(),
                    ));
                }
            }
        }
    }
    for child in &node.children {
        collect_owners(child, section, page_number, candidates, owners)?;
    }
    Ok(())
}

impl DocumentCore {
    pub(super) fn settle_body_page_fields(&mut self, force_breaks: &[HashSet<usize>]) {
        let candidates = fields(self);
        if candidates.is_empty()
            && self.document.sections.iter().all(|s| {
                s.paragraphs
                    .iter()
                    .all(|p| p.layout_field_values.is_empty())
            })
        {
            return;
        }
        if let Err(error) = self.resolve_body_page_fields(&candidates, force_breaks) {
            self.body_page_field_error = Some(error.to_string());
            self.invalidate_page_tree_cache();
        }
    }

    fn resolve_body_page_fields(
        &mut self,
        candidates: &BTreeMap<FieldKey, BodyField>,
        force_breaks: &[HashSet<usize>],
    ) -> Result<(), HwpError> {
        let mut seen = HashSet::new();
        for _ in 0..MAX_FIELD_RESOLUTION_PASSES {
            let total_pages = self.page_count();
            let mut owners = BTreeMap::new();
            let mut global_page = 0;
            for (section, pagination) in self.pagination.iter().enumerate() {
                for page in &pagination.pages {
                    let mut partial = BTreeMap::new();
                    for item in page.column_contents.iter().flat_map(|column| &column.items) {
                        let para = item.para_index();
                        let range =
                            candidates.range((section, para, 0)..=(section, para, usize::MAX));
                        match item {
                            crate::renderer::pagination::PageItem::FullParagraph { .. } => {
                                // FullParagraph는 모델 문단 전체의 명시적 소유다. 개체 뒤 run이
                                // char_start를 생략해도 좌표/글꼴로 소속을 추측하지 않는다.
                                for (&key, _) in range {
                                    if owners
                                        .insert(key, page.page_number)
                                        .is_some_and(|old| old != page.page_number)
                                    {
                                        return Err(HwpError::RenderError(
                                            "본문 쪽번호 필드의 문단 소속이 중복됩니다".into(),
                                        ));
                                    }
                                }
                            }
                            crate::renderer::pagination::PageItem::PartialParagraph { .. } => {
                                partial.extend(range.map(|(&key, value)| (key, value.clone())));
                            }
                            _ => {}
                        }
                    }
                    if !partial.is_empty() {
                        let tree = self.build_page_tree(global_page)?;
                        collect_owners(
                            &tree.root,
                            section,
                            page.page_number,
                            &partial,
                            &mut owners,
                        )?;
                    }
                    global_page += 1;
                }
            }
            let mut next = BTreeMap::new();
            for (&key, field) in candidates {
                let number = &field.number;
                let owner = owners.get(&key).ok_or_else(|| {
                    HwpError::RenderError(format!(
                        "본문 쪽번호 필드의 실제 줄 소속이 없습니다: 구역{} 문단{} 문자{}",
                        key.0, key.1, key.2
                    ))
                })?;
                let value = if number.number_type == AutoNumberType::TotalPage {
                    total_pages
                } else {
                    *owner
                };
                next.insert(key, display(number, value)?);
            }
            let mut changed_sections = Vec::new();
            for (section_index, section) in self.document.sections.iter_mut().enumerate() {
                let mut changed = false;
                for (para_index, para) in section.paragraphs.iter_mut().enumerate() {
                    let values: Vec<_> = next
                        .range(
                            (section_index, para_index, 0)
                                ..=(section_index, para_index, usize::MAX),
                        )
                        .map(|(key, value)| (key.2, value.clone()))
                        .collect();
                    if para.layout_field_values != values {
                        para.layout_field_values = values;
                        changed = true;
                    }
                }
                if changed {
                    changed_sections.push(section_index);
                }
            }
            if changed_sections.is_empty() {
                return Ok(());
            }
            let state: Vec<_> = next.into_iter().collect();
            if !seen.insert(state) {
                return Err(HwpError::RenderError(
                    "본문 쪽번호 표시값과 줄 소속의 재조판이 순환합니다".into(),
                ));
            }
            for section in changed_sections {
                self.recompose_section(section);
            }
            self.paginate_pass(force_breaks);
        }
        Err(HwpError::RenderError(format!(
            "본문 쪽번호 표시값이 {MAX_FIELD_RESOLUTION_PASSES}회 안에 수렴하지 않았습니다"
        )))
    }
}

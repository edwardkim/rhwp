//! 텍스트 삽입/삭제/문단 분리·병합/범위 삭제/문단 쿼리 관련 native 메서드

use super::super::helpers::get_textbox_from_shape;
use super::super::queries::field_query::rebuild_char_offsets;
use crate::document_core::{ActiveFieldInfo, DocumentCore};
use crate::error::HwpError;
use crate::model::control::{Control, FieldType};
use crate::model::event::DocumentEvent;
use crate::model::page::ColumnDef;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{ShapeObject, TextWrap, VertRelTo};
use crate::renderer::composer::{compose_paragraph, reflow_line_segs, ComposedParagraph};
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::{resolve_styles, ResolvedStyleSet};

fn recalculate_cell_paragraph_vpos(
    paragraphs: &mut [Paragraph],
    start_para: usize,
    ignore_reset_at: Option<usize>,
    styles: &ResolvedStyleSet,
    dpi: f64,
    is_hwp3_variant: bool,
) {
    if paragraphs.is_empty() || start_para >= paragraphs.len() {
        return;
    }

    // RowBreak 거대 셀은 후속 문단 vpos를 뒤로 되돌려 다음 조각의 로컬 원점을
    // 표현하기도 한다. 그 경계까지 선형 편집 결과를 연결하되, 경계 이후 저장
    // 좌표는 페이지 분할 신호이므로 이동하지 않는다.
    // [Task #2299] 합성 seg(TAG_IMPLEMENTATION_PROPERTY, #1811)의 vpos=0 은 배치 전
    // placeholder 이지 분할 신호가 아니다 — 섹션 recalc 와 동일하게 정지 대상에서
    // 제외한다 (로드가 합성한 중간-셀 문단에서 가짜 정지 → 꼬리 미갱신 방지).
    let stop_para = paragraphs
        .windows(2)
        .enumerate()
        .skip(start_para)
        .find_map(|(idx, pair)| {
            let previous = pair[0].line_segs.first()?.vertical_pos;
            let current_seg = pair[1].line_segs.first()?;
            let current = current_seg.vertical_pos;
            let is_synthetic = current_seg.tag
                & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                != 0;
            let reset_para = idx + 1;
            let is_inserted_paragraph = ignore_reset_at == Some(reset_para);
            (current < previous && !is_inserted_paragraph && !is_synthetic).then_some(reset_para)
        })
        .unwrap_or(paragraphs.len());

    let boundary_gaps: Vec<i32> = paragraphs
        .windows(2)
        .map(|pair| {
            let spacing_after = styles
                .para_styles
                .get(pair[0].para_shape_id as usize)
                .map(|style| style.spacing_after)
                .unwrap_or(0.0);
            let spacing_before = styles
                .para_styles
                .get(pair[1].para_shape_id as usize)
                .map(|style| style.spacing_before)
                .unwrap_or(0.0);
            let spacing_before =
                crate::renderer::hwp3_variant_flow_spacing_before(spacing_before, is_hwp3_variant);
            crate::renderer::px_to_hwpunit(spacing_after + spacing_before, dpi)
        })
        .collect();

    let mut next_vpos = if start_para > 0 {
        let previous = &paragraphs[start_para - 1];
        previous
            .line_segs
            .last()
            .map(|seg| {
                seg.vertical_pos
                    + seg.line_height
                    + seg.line_spacing
                    + boundary_gaps[start_para - 1]
            })
            .unwrap_or(0)
    } else {
        paragraphs[0]
            .line_segs
            .first()
            .map(|seg| seg.vertical_pos)
            .unwrap_or(0)
    };

    for para_idx in start_para..stop_para {
        let para = &mut paragraphs[para_idx];
        if let Some(first_vpos) = para.line_segs.first().map(|seg| seg.vertical_pos) {
            let delta = next_vpos - first_vpos;
            for seg in &mut para.line_segs {
                seg.vertical_pos += delta;
            }
            if let Some(last) = para.line_segs.last() {
                next_vpos = last.vertical_pos + last.line_height + last.line_spacing;
            }
        }
        if let Some(gap) = boundary_gaps.get(para_idx) {
            next_vpos += gap;
        }
    }
}

fn shift_paragraph_vpos_origin(para: &mut Paragraph, target_vpos: i32) {
    let Some(current_vpos) = para.line_segs.first().map(|seg| seg.vertical_pos) else {
        return;
    };
    let delta = target_vpos - current_vpos;
    for seg in &mut para.line_segs {
        seg.vertical_pos += delta;
    }
}

/// [Issue #2214] 문단 첫 줄을 원점으로 한 상대 flow advance.
/// line count가 아니라 후속 문단의 배치 위치를 실제로 바꾸는 높이 신호다.
fn relative_paragraph_flow_advance(paragraph: &Paragraph) -> Option<i64> {
    let first = paragraph.line_segs.first()?;
    let last = paragraph.line_segs.last()?;
    Some(
        i64::from(last.vertical_pos) + i64::from(last.line_height) + i64::from(last.line_spacing)
            - i64::from(first.vertical_pos),
    )
}

#[derive(Clone, Copy)]
struct FieldEndInsertion {
    control_idx: usize,
    start_char_idx: usize,
    end_char_idx: usize,
}

#[derive(Clone, Copy)]
struct FieldStartInsertion {
    control_idx: usize,
    start_char_idx: usize,
    end_char_idx: usize,
}

#[derive(Clone, Copy)]
struct SquareOleWrapChainForEnter {
    bottom_vpos: i32,
    column_start: i32,
    segment_width: i32,
}

fn active_field_matches(
    active_field: Option<&ActiveFieldInfo>,
    section_idx: usize,
    para_idx: usize,
    cell_path: Option<&[(usize, usize, usize)]>,
    control_idx: usize,
) -> bool {
    active_field.is_some_and(|af| {
        af.section_idx == section_idx
            && af.para_idx == para_idx
            && af.control_idx == control_idx
            && match (&af.cell_path, cell_path) {
                (None, None) => true,
                (Some(a), Some(b)) => a.as_slice() == b,
                _ => false,
            }
    })
}

fn inactive_field_end_insertions(
    para: &Paragraph,
    active_field: Option<&ActiveFieldInfo>,
    section_idx: usize,
    para_idx: usize,
    cell_path: Option<&[(usize, usize, usize)]>,
    char_offset: usize,
) -> Vec<FieldEndInsertion> {
    para.field_ranges
        .iter()
        .filter_map(|fr| {
            match para.controls.get(fr.control_idx) {
                Some(Control::Field(field)) if field.field_type == FieldType::ClickHere => {}
                _ => return None,
            }
            // 빈 누름틀은 active 상태가 아직 반영되기 전 첫 입력도 값으로 받아야 한다.
            if fr.start_char_idx == fr.end_char_idx || fr.end_char_idx != char_offset {
                return None;
            }
            if active_field_matches(
                active_field,
                section_idx,
                para_idx,
                cell_path,
                fr.control_idx,
            ) {
                return None;
            }
            Some(FieldEndInsertion {
                control_idx: fr.control_idx,
                start_char_idx: fr.start_char_idx,
                end_char_idx: fr.end_char_idx,
            })
        })
        .collect()
}

fn para_has_visible_text_for_enter(para: &Paragraph) -> bool {
    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
}

fn is_empty_topbottom_table_anchor_for_enter(para: &Paragraph) -> bool {
    !para_has_visible_text_for_enter(para)
        && para.controls.iter().any(|ctrl| {
            matches!(
                ctrl,
                Control::Table(table)
                    if !table.common.treat_as_char
                        && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
                        && matches!(table.common.vert_rel_to, VertRelTo::Para)
            )
        })
}

fn square_ole_anchor_wrap_chain_for_enter(para: &Paragraph) -> Option<SquareOleWrapChainForEnter> {
    if para_has_visible_text_for_enter(para) || !para.char_offsets.is_empty() {
        return None;
    }
    let line_seg = para.line_segs.first()?;
    if line_seg.column_start <= 0 || line_seg.segment_width <= 0 {
        return None;
    }

    para.controls.iter().find_map(|ctrl| {
        let Control::Shape(shape) = ctrl else {
            return None;
        };
        if !matches!(shape.as_ref(), ShapeObject::Ole(_))
            || shape.common().treat_as_char
            || !matches!(shape.common().text_wrap, TextWrap::Square)
        {
            return None;
        }

        let height = shape.common().height.min(i32::MAX as u32) as i32;
        if height <= 0 {
            return None;
        }
        Some(SquareOleWrapChainForEnter {
            bottom_vpos: line_seg.vertical_pos.saturating_add(height),
            column_start: line_seg.column_start,
            segment_width: line_seg.segment_width,
        })
    })
}

fn is_empty_stored_square_wrap_line_for_enter(para: &Paragraph) -> bool {
    !para_has_visible_text_for_enter(para)
        && para.char_offsets.is_empty()
        && para.controls.is_empty()
        && para
            .line_segs
            .first()
            .is_some_and(|seg| seg.column_start > 0 && seg.segment_width > 0)
}

fn is_contentless_empty_paragraph_for_merge(para: &Paragraph) -> bool {
    para.text.is_empty() && para.char_offsets.is_empty() && para.controls.is_empty()
}

fn has_same_stored_wrap_line(lhs: &Paragraph, rhs: &Paragraph) -> bool {
    match (lhs.line_segs.first(), rhs.line_segs.first()) {
        (Some(a), Some(b)) => {
            a.column_start == b.column_start && a.segment_width == b.segment_width
        }
        _ => false,
    }
}

fn square_ole_wrap_chain_for_enter(
    paragraphs: &[Paragraph],
    para_idx: usize,
) -> Option<SquareOleWrapChainForEnter> {
    let anchor = paragraphs.get(para_idx)?;
    if let Some(chain) = square_ole_anchor_wrap_chain_for_enter(anchor) {
        return Some(chain);
    }
    if !is_empty_stored_square_wrap_line_for_enter(anchor) {
        return None;
    }

    let mut idx = para_idx;
    while idx > 0 {
        idx -= 1;
        let prev = paragraphs.get(idx)?;
        if let Some(chain) = square_ole_anchor_wrap_chain_for_enter(prev) {
            return (chain.column_start == anchor.line_segs[0].column_start
                && chain.segment_width == anchor.line_segs[0].segment_width)
                .then_some(chain);
        }
        if is_empty_stored_square_wrap_line_for_enter(prev)
            && has_same_stored_wrap_line(prev, anchor)
        {
            continue;
        }
        return None;
    }
    None
}

fn next_line_vpos_after_para_for_enter(para: &Paragraph) -> i32 {
    para.line_segs
        .last()
        .map(|seg| {
            seg.vertical_pos
                .saturating_add(seg.line_height)
                .saturating_add(seg.line_spacing)
        })
        .unwrap_or(0)
}

fn empty_paragraph_after_normal_flow(anchor: &Paragraph) -> Paragraph {
    let mut para = empty_paragraph_after_table_anchor(anchor);
    if let Some(seg) = para.line_segs.first_mut() {
        seg.column_start = 0;
        seg.segment_width = 0;
        seg.vertical_pos = 0;
    }
    para
}

fn empty_paragraph_after_table_anchor(anchor: &Paragraph) -> Paragraph {
    let mut para = Paragraph::new_empty_like(anchor);
    if let Some(seg) = para.line_segs.first_mut() {
        if let Some(anchor_seg) = anchor.line_segs.first() {
            seg.segment_width = anchor_seg.segment_width;
        }
    }
    let mut raw_header_extra = vec![0u8; 10];
    raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
    raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());
    para.raw_header_extra = raw_header_extra;
    para.has_para_text = false;
    para
}

fn empty_paragraph_after_square_wrap_anchor(anchor: &Paragraph) -> Paragraph {
    let mut para = empty_paragraph_after_table_anchor(anchor);
    if let (Some(seg), Some(anchor_seg)) = (para.line_segs.first_mut(), anchor.line_segs.first()) {
        *seg = anchor_seg.clone();
        seg.text_start = 0;
        seg.vertical_pos = 0;
    }
    para
}

fn inactive_field_start_insertions(
    para: &Paragraph,
    active_field: Option<&ActiveFieldInfo>,
    section_idx: usize,
    para_idx: usize,
    cell_path: Option<&[(usize, usize, usize)]>,
    char_offset: usize,
) -> Vec<FieldStartInsertion> {
    para.field_ranges
        .iter()
        .filter_map(|fr| {
            match para.controls.get(fr.control_idx) {
                Some(Control::Field(field)) if field.field_type == FieldType::ClickHere => {}
                _ => return None,
            }
            // 빈 누름틀은 시작/끝 경계가 없고 첫 입력이 필드 값이어야 한다.
            if fr.start_char_idx == fr.end_char_idx || fr.start_char_idx != char_offset {
                return None;
            }
            if active_field_matches(
                active_field,
                section_idx,
                para_idx,
                cell_path,
                fr.control_idx,
            ) {
                return None;
            }
            Some(FieldStartInsertion {
                control_idx: fr.control_idx,
                start_char_idx: fr.start_char_idx,
                end_char_idx: fr.end_char_idx,
            })
        })
        .collect()
}

fn keep_inactive_field_end_outside(
    para: &mut Paragraph,
    insertions: &[FieldEndInsertion],
    inserted_len: usize,
) {
    if inserted_len == 0 || insertions.is_empty() {
        return;
    }
    for target in insertions {
        if let Some(fr) = para.field_ranges.iter_mut().find(|fr| {
            fr.control_idx == target.control_idx
                && fr.start_char_idx == target.start_char_idx
                && fr.end_char_idx == target.end_char_idx + inserted_len
        }) {
            fr.end_char_idx = target.end_char_idx;
        }
    }
}

fn keep_inactive_field_start_outside(
    para: &mut Paragraph,
    insertions: &[FieldStartInsertion],
    inserted_len: usize,
) {
    if inserted_len == 0 || insertions.is_empty() {
        return;
    }
    for target in insertions {
        if let Some(fr) = para.field_ranges.iter_mut().find(|fr| {
            fr.control_idx == target.control_idx
                && fr.start_char_idx == target.start_char_idx
                && fr.end_char_idx == target.end_char_idx + inserted_len
        }) {
            fr.start_char_idx = target.start_char_idx + inserted_len;
        }
    }
}

fn has_clickhere_field_range(para: &Paragraph) -> bool {
    para.field_ranges.iter().any(|fr| {
        matches!(
            para.controls.get(fr.control_idx),
            Some(Control::Field(field)) if field.field_type == FieldType::ClickHere
        )
    })
}

impl DocumentCore {
    pub fn insert_text_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        // 인덱스 범위 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 텍스트 삽입
        let new_chars_count = text.chars().count();
        let active_field = self.active_field.clone();
        let outside_insertions = inactive_field_end_insertions(
            &self.document.sections[section_idx].paragraphs[para_idx],
            active_field.as_ref(),
            section_idx,
            para_idx,
            None,
            char_offset,
        );
        let before_insertions = inactive_field_start_insertions(
            &self.document.sections[section_idx].paragraphs[para_idx],
            active_field.as_ref(),
            section_idx,
            para_idx,
            None,
            char_offset,
        );
        {
            let para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            para.insert_text_at(char_offset, text);
            keep_inactive_field_start_outside(para, &before_insertions, new_chars_count);
            keep_inactive_field_end_outside(para, &outside_insertions, new_chars_count);
            if has_clickhere_field_range(para) {
                rebuild_char_offsets(para);
            }
        }

        // line_segs 재계산 (리플로우) → vpos 재계산 → 재구성 → 재페이지네이션
        // 다단 문서에서 편집 후 문단이 다른 단으로 재배치될 수 있으므로
        // para_column_map 변경 감지 + 재reflow 수렴 루프 (최대 3회)
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[para_idx],
        );
        self.reflow_paragraph(section_idx, para_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            None,
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();
        }

        let new_offset = char_offset + new_chars_count;

        // 캐럿 위치 갱신 (DocProperties)
        // caret_char_pos는 UTF-16 코드 유닛 기준
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let caret_utf16_pos = if new_offset < para.char_offsets.len() {
            para.char_offsets[new_offset]
        } else if !para.char_offsets.is_empty() {
            let last = para.char_offsets.len() - 1;
            let last_char = para.text.chars().nth(last);
            para.char_offsets[last]
                + last_char
                    .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                    .unwrap_or(1)
        } else {
            // 텍스트 없이 컨트롤만 있는 경우
            (para.controls.len() as u32) * 8
        };
        self.document.doc_properties.caret_list_id = section_idx as u32;
        self.document.doc_properties.caret_para_id = para_idx as u32;
        self.document.doc_properties.caret_char_pos = caret_utf16_pos;

        // DocInfo raw_stream 내 캐럿 위치만 surgical update (전체 재직렬화 방지)
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let _ = crate::serializer::doc_info::surgical_update_caret(
                raw,
                section_idx as u32,
                para_idx as u32,
                caret_utf16_pos,
            );
        }

        self.event_log.push(DocumentEvent::TextInserted {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
            len: new_chars_count,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
    }

    /// 텍스트 삭제 (네이티브 에러 타입)
    pub fn delete_text_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        // 인덱스 범위 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 텍스트 삭제
        self.document.sections[section_idx].paragraphs[para_idx].delete_text_at(char_offset, count);

        // line_segs 재계산 (리플로우) → 재구성 → 재페이지네이션
        // 다단 수렴 루프 (최대 3회)
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[para_idx],
        );
        self.reflow_paragraph(section_idx, para_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            None,
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();
        }

        // 캐럿 위치 갱신 (DocProperties)
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let caret_utf16_pos = if char_offset < para.char_offsets.len() {
            para.char_offsets[char_offset]
        } else if !para.char_offsets.is_empty() {
            let last = para.char_offsets.len() - 1;
            let last_char = para.text.chars().nth(last);
            para.char_offsets[last]
                + last_char
                    .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                    .unwrap_or(1)
        } else {
            (para.controls.len() as u32) * 8
        };
        self.document.doc_properties.caret_list_id = section_idx as u32;
        self.document.doc_properties.caret_para_id = para_idx as u32;
        self.document.doc_properties.caret_char_pos = caret_utf16_pos;

        // DocInfo raw_stream 내 캐럿 위치만 surgical update (전체 재직렬화 방지)
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let _ = crate::serializer::doc_info::surgical_update_caret(
                raw,
                section_idx as u32,
                para_idx as u32,
                caret_utf16_pos,
            );
        }

        self.event_log.push(DocumentEvent::TextDeleted {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
            count,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
    }

    /// 지정된 문단의 line_segs를 컬럼 너비 기반으로 재계산한다.
    pub(crate) fn reflow_paragraph(&mut self, section_idx: usize, para_idx: usize) {
        let section = &self.document.sections[section_idx];
        let page_def = &section.section_def.page_def;
        // 해당 문단에 적용되는 ColumnDef를 찾음 (구역 내 다단↔단일단 전환 지원)
        let column_def = Self::find_column_def_for_paragraph(&section.paragraphs, para_idx);
        let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);

        // 페이지네이션 매핑에서 문단의 소속 단 인덱스 조회
        let col_idx = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0) as usize;
        let col_area = layout
            .column_areas
            .get(col_idx)
            .unwrap_or(&layout.column_areas[0]);

        // 문단 여백 계산
        let para = &section.paragraphs[para_idx];
        let para_style = self.styles.para_styles.get(para.para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let available_width = col_area.width - margin_left - margin_right;

        reflow_line_segs(
            &mut self.document.sections[section_idx].paragraphs[para_idx],
            available_width,
            &self.styles,
            self.dpi,
        );
    }

    /// 표 셀 내부 문단에 텍스트 삽입 (네이티브)
    pub fn insert_text_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        self.insert_text_in_cell_native_impl(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
            text,
            true,
        )
    }

    /// 표 셀 내부 단일 텍스트 삽입 후 전체 페이지네이션을 호출자가 지연한다.
    /// 결과 JSON의 `cellFlowChanged`는 상대 line advance 변화 여부다.
    pub fn insert_text_in_cell_native_deferred_pagination(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        self.insert_text_in_cell_native_impl(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
            text,
            false,
        )
    }

    fn insert_text_in_cell_native_impl(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        text: &str,
        paginate_immediately: bool,
    ) -> Result<String, HwpError> {
        // 셀 문단 접근 검증 및 텍스트 삽입
        let active_field = self.active_field.clone();
        let cell_path = [(control_idx, cell_idx, cell_para_idx)];
        let cell_para = self.get_cell_paragraph_mut(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        let flow_advance_before = relative_paragraph_flow_advance(cell_para);
        let local_contribution_before =
            crate::renderer::layout::LayoutEngine::paragraph_contributes_to_table_nested_text_flag(
                cell_para,
            );
        let new_chars_count = text.chars().count();
        let outside_insertions = inactive_field_end_insertions(
            cell_para,
            active_field.as_ref(),
            section_idx,
            cell_para_idx,
            Some(&cell_path),
            char_offset,
        );
        let before_insertions = inactive_field_start_insertions(
            cell_para,
            active_field.as_ref(),
            section_idx,
            cell_para_idx,
            Some(&cell_path),
            char_offset,
        );
        cell_para.insert_text_at(char_offset, text);
        keep_inactive_field_start_outside(cell_para, &before_insertions, new_chars_count);
        keep_inactive_field_end_outside(cell_para, &outside_insertions, new_chars_count);
        if has_clickhere_field_range(cell_para) {
            rebuild_char_offsets(cell_para);
        }

        // 부모 컨트롤 dirty 마킹 (표 또는 글상자)
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);

        // 셀 폭 기반 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        );

        let (flow_advance_after, local_contribution_after, cell_para_after) = {
            let cell_para_after = self
                .get_cell_paragraph_ref(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                )
                .ok_or_else(|| {
                    HwpError::RenderError("삽입 뒤 셀 문단을 다시 찾을 수 없습니다".to_string())
                })?;
            (
                relative_paragraph_flow_advance(cell_para_after),
                crate::renderer::layout::LayoutEngine::paragraph_contributes_to_table_nested_text_flag(
                    cell_para_after,
                ),
                cell_para_after.clone(),
            )
        };
        let cell_flow_changed = flow_advance_before != flow_advance_after;

        // Table의 일반 cell만 pointer-key layout cache의 owner다. 표 캡션 sentinel과
        // Shape/Picture 텍스트 경로에는 cell_units cache가 없으므로 적용하지 않는다.
        if cell_idx != 65534 {
            let control = &self.document.sections[section_idx].paragraphs[parent_para_idx].controls
                [control_idx];
            if let Control::Table(table) = control {
                if let Some(edited_cell) = table.cells.get(cell_idx) {
                    self.layout_engine.invalidate_cell_units_after_text_insert(
                        edited_cell,
                        table,
                        local_contribution_before,
                        local_contribution_after,
                    );
                }
            }
        }

        // [#2214/#2195] paginate 시 생성된 render_normalized는 원본 IR의 파생
        // 복사본이다. deferred edit은 pagination을 실행하지 않으므로 원본만 바꾸면
        // page tree가 편집 전 복사본을 계속 읽는다. 편집된 cell paragraph만 같은
        // normalized path에 반영하고 그 복사본의 pointer-key cache도 국소 제거한다.
        if !paginate_immediately {
            self.refresh_render_normalized_cell_paragraph_after_edit(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                cell_para_after,
                local_contribution_before,
                local_contribution_after,
            );
        }
        // raw 스트림 무효화, 재페이지네이션 (셀 편집 → composed 불변, section dirty만 설정)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.invalidate_page_tree_cache_from(0);
        if paginate_immediately {
            self.paginate_if_needed();
        }

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        let result_fields = if paginate_immediately {
            format!("\"charOffset\":{}", new_offset)
        } else {
            format!(
                "\"charOffset\":{},\"cellFlowChanged\":{}",
                new_offset, cell_flow_changed
            )
        };
        Ok(super::super::helpers::json_ok_with(&result_fields))
    }

    /// 표 셀 내부 문단에서 텍스트 삭제 (네이티브)
    pub fn delete_text_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        // 셀 문단 접근 검증 및 텍스트 삭제
        let cell_para = self.get_cell_paragraph_mut(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        cell_para.delete_text_at(char_offset, count);

        // 부모 컨트롤 dirty 마킹 (표 또는 글상자)
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);

        // 셀 폭 기반 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        );

        // raw 스트림 무효화, 재페이지네이션 (셀 편집 → composed 불변)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
    }

    /// 표 셀 또는 글상자 내부 문단에 대한 가변 참조를 얻는다.
    pub(crate) fn get_cell_paragraph_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<&mut crate::model::paragraph::Paragraph, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "부모 문단 인덱스 {} 범위 초과",
                parent_para_idx
            )));
        }
        let para = &mut section.paragraphs[parent_para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과",
                control_idx
            )));
        }
        match &mut para.controls[control_idx] {
            Control::Table(t) => {
                // cell_idx == 65534: 표 캡션 접근 (TypeScript에서 표 캡션 편집 시 사용)
                if cell_idx == 65534 {
                    let cap = t.caption.as_mut().ok_or_else(|| {
                        HwpError::RenderError("지정된 표 컨트롤에 캡션이 없습니다".to_string())
                    })?;
                    if cell_para_idx >= cap.paragraphs.len() {
                        return Err(HwpError::RenderError(format!(
                            "캡션 문단 인덱스 {} 범위 초과 (총 {}개)",
                            cell_para_idx,
                            cap.paragraphs.len()
                        )));
                    }
                    return Ok(&mut cap.paragraphs[cell_para_idx]);
                }
                if cell_idx >= t.cells.len() {
                    return Err(HwpError::RenderError(format!(
                        "셀 인덱스 {} 범위 초과 (총 {}개)",
                        cell_idx,
                        t.cells.len()
                    )));
                }
                let cell = &mut t.cells[cell_idx];
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "셀 문단 인덱스 {} 범위 초과 (총 {}개)",
                        cell_para_idx,
                        cell.paragraphs.len()
                    )));
                }
                Ok(&mut cell.paragraphs[cell_para_idx])
            }
            Control::Shape(shape) => {
                if cell_idx != 0 {
                    return Err(HwpError::RenderError(format!(
                        "글상자 셀 인덱스는 0이어야 합니다 (요청: {})",
                        cell_idx
                    )));
                }
                let tb =
                    super::super::helpers::get_textbox_from_shape_mut(shape).ok_or_else(|| {
                        HwpError::RenderError(
                            "지정된 Shape 컨트롤에 텍스트 박스가 없습니다".to_string(),
                        )
                    })?;
                if cell_para_idx >= tb.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "글상자 문단 인덱스 {} 범위 초과 (총 {}개)",
                        cell_para_idx,
                        tb.paragraphs.len()
                    )));
                }
                Ok(&mut tb.paragraphs[cell_para_idx])
            }
            Control::Picture(pic) => {
                let cap = pic.caption.as_mut().ok_or_else(|| {
                    HwpError::RenderError("지정된 그림 컨트롤에 캡션이 없습니다".to_string())
                })?;
                if cell_para_idx >= cap.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "캡션 문단 인덱스 {} 범위 초과 (총 {}개)",
                        cell_para_idx,
                        cap.paragraphs.len()
                    )));
                }
                Ok(&mut cap.paragraphs[cell_para_idx])
            }
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 표, 글상자 또는 그림이 아닙니다".to_string(),
            )),
        }
    }

    /// 부모 컨트롤(표 또는 글상자)의 dirty를 마킹한다.
    pub(crate) fn mark_cell_control_dirty(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) {
        if let Some(ctrl) = self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            match ctrl {
                Control::Table(t) => {
                    t.dirty = true;
                }
                // Shape는 별도 dirty 필드가 없으므로 section dirty만으로 충분
                _ => {}
            }
        }
    }

    pub(crate) fn reflow_cell_paragraph(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) {
        use crate::renderer::hwpunit_to_px;

        // 셀/글상자 폭과 패딩 읽기 (불변 참조) — path 변형과 공유하는 helper 사용.
        let (cell_width, pad_left, pad_right) = match self.document.sections[section_idx].paragraphs
            [parent_para_idx]
            .controls
            .get(control_idx)
            .and_then(|control| Self::cell_metrics_for_control(control, cell_idx))
        {
            Some(metrics) => metrics,
            None => return,
        };

        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let cell_width_px = hwpunit_to_px(cell_width as i32, self.dpi);
        let pad_left_px = hwpunit_to_px(pad_left as i32, self.dpi);
        let pad_right_px = hwpunit_to_px(pad_right as i32, self.dpi);
        let available_width = (cell_width_px - pad_left_px - pad_right_px).max(0.0);

        // 문단 여백 계산
        let para_shape_id = {
            let cell_para = self.get_cell_paragraph_ref(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            );
            match cell_para {
                Some(p) => p.para_shape_id,
                None => return,
            }
        };
        let para_style = styles.para_styles.get(para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let final_width = (available_width - margin_left - margin_right).max(0.0);

        // 가변 참조로 리플로우 실행
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                let cell_para = if cell_idx == 65534 {
                    table
                        .caption
                        .as_mut()
                        .and_then(|caption| caption.paragraphs.get_mut(cell_para_idx))
                } else {
                    table
                        .cells
                        .get_mut(cell_idx)
                        .and_then(|cell| cell.paragraphs.get_mut(cell_para_idx))
                };
                if let Some(cell_para) = cell_para {
                    reflow_line_segs(cell_para, final_width, &styles, self.dpi);
                }
            }
            Some(Control::Shape(shape)) => {
                if let Some(tb) = super::super::helpers::get_textbox_from_shape_mut(shape) {
                    if let Some(cell_para) = tb.paragraphs.get_mut(cell_para_idx) {
                        reflow_line_segs(cell_para, final_width, &styles, self.dpi);
                    }
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    if let Some(cell_para) = cap.paragraphs.get_mut(cell_para_idx) {
                        reflow_line_segs(cell_para, final_width, &styles, self.dpi);
                    }
                }
            }
            _ => {}
        }
    }

    fn recalculate_cell_paragraph_vpos_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        start_para: usize,
        ignore_reset_at: Option<usize>,
    ) {
        let styles = &self.styles;
        let dpi = self.dpi;
        let is_hwp3_variant = self.document.layout_profile().hwp3_layout();
        let Some(control) = self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        else {
            return;
        };
        let paragraphs = match control {
            Control::Table(table) if cell_idx == 65534 => {
                let Some(caption) = table.caption.as_mut() else {
                    return;
                };
                &mut caption.paragraphs
            }
            Control::Table(table) => {
                let Some(cell) = table.cells.get_mut(cell_idx) else {
                    return;
                };
                &mut cell.paragraphs
            }
            Control::Shape(shape) => {
                let Some(textbox) = super::super::helpers::get_textbox_from_shape_mut(shape) else {
                    return;
                };
                &mut textbox.paragraphs
            }
            Control::Picture(picture) => {
                let Some(caption) = picture.caption.as_mut() else {
                    return;
                };
                &mut caption.paragraphs
            }
            _ => return,
        };
        recalculate_cell_paragraph_vpos(
            paragraphs,
            start_para,
            ignore_reset_at,
            styles,
            dpi,
            is_hwp3_variant,
        );
    }

    /// [#2755] 컨트롤+cell_idx 로부터 셀 폭·좌우 패딩(HWPUNIT)을 해석한다.
    ///
    /// `reflow_cell_paragraph`(flat)와 `reflow_cell_paragraph_by_path`(중첩)가 공유한다.
    /// `None` = 표/글상자/그림 캡션이 아니거나 대상 셀/텍스트박스가 없음.
    fn cell_metrics_for_control(control: &Control, cell_idx: usize) -> Option<(u32, i16, i16)> {
        match control {
            Control::Table(table) => {
                if cell_idx == 65534 {
                    // 표 캡션: Top/Bottom 은 max_width, Left/Right 는 width.
                    let cap = table.caption.as_ref()?;
                    use crate::model::shape::CaptionDirection;
                    let w = match cap.direction {
                        CaptionDirection::Left | CaptionDirection::Right => cap.width,
                        _ => cap.max_width,
                    };
                    Some((w, 0, 0))
                } else {
                    let cell = table.cells.get(cell_idx)?;
                    let pad_l = if cell.apply_inner_margin {
                        cell.padding.left
                    } else {
                        table.padding.left
                    };
                    let pad_r = if cell.apply_inner_margin {
                        cell.padding.right
                    } else {
                        table.padding.right
                    };
                    Some((cell.width, pad_l, pad_r))
                }
            }
            Control::Shape(shape) => {
                let tb = super::super::helpers::get_textbox_from_shape(shape)?;
                let common = shape.common();
                Some((common.width as u32, tb.margin_left, tb.margin_right))
            }
            Control::Picture(pic) => Some((pic.common.width as u32, 0, 0)),
            _ => None,
        }
    }

    /// [#2755] path 의 CellPathEntry 사슬을 따라 **최내곽** 셀의 폭·좌우 패딩(HWPUNIT)을
    /// 해석한다. 마지막 엔트리를 제외한 각 엔트리에서 다음 중첩 컨트롤을 담은 컨테이너
    /// 문단(`cell_para_idx`)으로 하강한다 — `get_cell_paragraphs_mut_by_path` 의 불변 짝이다.
    fn resolve_innermost_cell_metrics(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Option<(u32, i16, i16)> {
        let mut para = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(parent_para_idx)?;
        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let control = para.controls.get(ctrl_idx)?;
            if i + 1 == path.len() {
                return Self::cell_metrics_for_control(control, cell_idx);
            }
            para = match control {
                Control::Table(t) => t.cells.get(cell_idx)?.paragraphs.get(cell_para_idx)?,
                Control::Shape(s) => super::super::helpers::get_textbox_from_shape(s)?
                    .paragraphs
                    .get(cell_para_idx)?,
                Control::Picture(p) => p.caption.as_ref()?.paragraphs.get(cell_para_idx)?,
                _ => return None,
            };
        }
        None
    }

    /// [#2755] path 기반 셀 리플로우 (깊이 ≥ 2 중첩 표 지원).
    ///
    /// `reflow_cell_paragraph`(flat)는 최외곽 표만 리플로우한다. 이 변형은 path 의
    /// CellPathEntry 사슬로 **최내곽** 셀의 폭을 해석하고, 그 폭으로 최내곽 셀의
    /// `cell_para_idx` 문단을 재래핑한다. 깊이 1 에서는 flat 형제와 동일한 결과를 낸다.
    pub(crate) fn reflow_cell_paragraph_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        cell_para_idx: usize,
    ) {
        use crate::renderer::hwpunit_to_px;

        let Some((cell_width, pad_left, pad_right)) =
            self.resolve_innermost_cell_metrics(section_idx, parent_para_idx, path)
        else {
            return;
        };
        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let dpi = self.dpi;
        let cell_width_px = hwpunit_to_px(cell_width as i32, dpi);
        let pad_left_px = hwpunit_to_px(pad_left as i32, dpi);
        let pad_right_px = hwpunit_to_px(pad_right as i32, dpi);
        let available_width = (cell_width_px - pad_left_px - pad_right_px).max(0.0);

        let Ok(paras) = self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)
        else {
            return;
        };
        let Some(cell_para) = paras.get_mut(cell_para_idx) else {
            return;
        };
        let para_style = styles.para_styles.get(cell_para.para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let final_width = (available_width - margin_left - margin_right).max(0.0);
        reflow_line_segs(cell_para, final_width, &styles, dpi);
    }

    /// [#2755] path 기반 셀 문단 vpos 재계산 (깊이 ≥ 2 중첩 표 지원).
    /// `recalculate_cell_paragraph_vpos_native` 의 path 변형.
    fn recalculate_cell_paragraph_vpos_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        start_para: usize,
        ignore_reset_at: Option<usize>,
    ) {
        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let dpi = self.dpi;
        let is_hwp3_variant = self.document.layout_profile().hwp3_layout();
        if let Ok(paras) = self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)
        {
            recalculate_cell_paragraph_vpos(
                paras,
                start_para,
                ignore_reset_at,
                &styles,
                dpi,
                is_hwp3_variant,
            );
        }
    }

    // ─── Phase 3 네이티브 구현: 커서 이동 API ─────────────────

    pub(crate) fn delete_range_native(
        &mut self,
        section_idx: usize,
        start_para: usize,
        start_offset: usize,
        end_para: usize,
        end_offset: usize,
        cell_ctx: Option<(usize, usize, usize)>,
    ) -> Result<String, HwpError> {
        // Section raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;
        // DocInfo raw_stream은 유지 (전체 재직렬화 시 FIX-4 문제 발생)

        if let Some((ppi, ci, cei)) = cell_ctx {
            // ─── 셀 내 deleteRange ───
            if start_para == end_para {
                // 같은 문단 내 삭제
                let count = end_offset - start_offset;
                if count > 0 {
                    let cell_para =
                        self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, start_para)?;
                    cell_para.delete_text_at(start_offset, count);
                    self.reflow_cell_paragraph(section_idx, ppi, ci, cei, start_para);
                }
            } else {
                // 다중 문단 셀 내 삭제
                // 1) 마지막 문단 앞부분 삭제
                if end_offset > 0 {
                    let cell_para =
                        self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, end_para)?;
                    cell_para.delete_text_at(0, end_offset);
                }
                // 2) 중간 문단 역순 제거 — 셀 내 문단은 cell.paragraphs에서 직접 제거
                for mid_para in (start_para + 1..end_para).rev() {
                    let cell = self.get_cell_mut(section_idx, ppi, ci, cei)?;
                    if mid_para < cell.paragraphs.len() {
                        cell.paragraphs.remove(mid_para);
                    }
                }
                // 3) 첫 문단 뒷부분 삭제
                {
                    let cell_para =
                        self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, start_para)?;
                    let para_len = cell_para.text.chars().count();
                    if start_offset < para_len {
                        cell_para.delete_text_at(start_offset, para_len - start_offset);
                    }
                }
                // 4) 첫-마지막 문단 병합 (마지막 문단이 이제 start_para+1에 위치)
                let cell = self.get_cell_mut(section_idx, ppi, ci, cei)?;
                if start_para + 1 < cell.paragraphs.len() {
                    let next_para = cell.paragraphs.remove(start_para + 1);
                    cell.paragraphs[start_para].merge_from(&next_para);
                }
                self.reflow_cell_paragraph(section_idx, ppi, ci, cei, start_para);
            }

            // 부모 컨트롤 dirty 마킹 + 재페이지네이션
            self.mark_cell_control_dirty(section_idx, ppi, ci);
            self.mark_section_dirty(section_idx);
            self.paginate_if_needed();
            self.event_log.push(DocumentEvent::CellTextChanged {
                section: section_idx,
                para: ppi,
                ctrl: ci,
                cell: cei,
            });
            Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}",
                start_para, start_offset
            )))
        } else {
            // ─── 본문 deleteRange ───
            if start_para == end_para {
                // 같은 문단 내 삭제
                let count = end_offset - start_offset;
                if count > 0 {
                    self.document.sections[section_idx].paragraphs[start_para]
                        .delete_text_at(start_offset, count);
                    // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
                    let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                        &self.document.sections[section_idx].paragraphs[start_para],
                    );
                    self.reflow_paragraph(section_idx, start_para);
                    let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
                    crate::renderer::composer::recalculate_section_vpos(
                        &mut self.document.sections[section_idx].paragraphs,
                        start_para,
                        None,
                        stored_end_for_reset,
                        &self.styles,
                        self.dpi,
                        doc_hwp3_layout,
                    );
                }
                // 변경 문단만 재구성
                self.recompose_paragraph(section_idx, start_para);
            } else {
                // 1) 마지막 문단 앞부분 삭제
                if end_offset > 0 {
                    self.document.sections[section_idx].paragraphs[end_para]
                        .delete_text_at(0, end_offset);
                }
                // 2) 중간 문단 역순 제거 (composed도 동기)
                for mid_para in (start_para + 1..end_para).rev() {
                    self.document.sections[section_idx]
                        .paragraphs
                        .remove(mid_para);
                    self.remove_composed_paragraph(section_idx, mid_para);
                }
                // 3) 첫 문단 뒷부분 삭제
                {
                    let para_len = self.document.sections[section_idx].paragraphs[start_para]
                        .text
                        .chars()
                        .count();
                    if start_offset < para_len {
                        self.document.sections[section_idx].paragraphs[start_para]
                            .delete_text_at(start_offset, para_len - start_offset);
                    }
                }
                // 4) 첫-마지막 문단 병합 (마지막 문단이 이제 start_para+1에 위치)
                if start_para + 1 < self.document.sections[section_idx].paragraphs.len() {
                    let next = self.document.sections[section_idx]
                        .paragraphs
                        .remove(start_para + 1);
                    self.remove_composed_paragraph(section_idx, start_para + 1);
                    self.document.sections[section_idx].paragraphs[start_para].merge_from(&next);
                }
                // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
                let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                    &self.document.sections[section_idx].paragraphs[start_para],
                );
                self.reflow_paragraph(section_idx, start_para);
                let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
                crate::renderer::composer::recalculate_section_vpos(
                    &mut self.document.sections[section_idx].paragraphs,
                    start_para,
                    None,
                    stored_end_for_reset,
                    &self.styles,
                    self.dpi,
                    doc_hwp3_layout,
                );
                // 병합된 문단 재구성
                self.recompose_paragraph(section_idx, start_para);
            }

            // 재페이지네이션
            self.paginate_if_needed();

            // 캐럿 위치 갱신
            self.document.doc_properties.caret_list_id = section_idx as u32;
            self.document.doc_properties.caret_para_id = start_para as u32;

            self.event_log.push(DocumentEvent::TextDeleted {
                section: section_idx,
                para: start_para,
                offset: start_offset,
                count: 0,
            });
            Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}",
                start_para, start_offset
            )))
        }
    }

    /// 표 셀에 대한 가변 참조를 얻는다.
    pub(crate) fn get_cell_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<&mut crate::model::table::Cell, HwpError> {
        let section = &mut self.document.sections[section_idx];
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("부모 문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let ctrl = para.controls.get_mut(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;
        match ctrl {
            Control::Table(ref mut table) => table
                .cells
                .get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx))),
            _ => Err(HwpError::RenderError(
                "테이블 컨트롤이 아닙니다".to_string(),
            )),
        }
    }

    // ─── Phase 4 네이티브 끝 ────────────────────────────────

    // ─── Phase 3 네이티브 끝 ─────────────────────────────────

    /// 표 셀 내부 문단에 대한 불변 참조를 얻는다.
    pub(crate) fn get_cell_paragraph_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Option<&crate::model::paragraph::Paragraph> {
        let para = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(parent_para_idx)?;
        match para.controls.get(control_idx)? {
            Control::Table(table) => {
                if cell_idx == 65534 {
                    return table.caption.as_ref()?.paragraphs.get(cell_para_idx);
                }
                table.cells.get(cell_idx)?.paragraphs.get(cell_para_idx)
            }
            Control::Shape(shape) => {
                if cell_idx != 0 {
                    return None;
                }
                get_textbox_from_shape(shape)?.paragraphs.get(cell_para_idx)
            }
            Control::Picture(pic) => pic.caption.as_ref()?.paragraphs.get(cell_para_idx),
            _ => None,
        }
    }

    pub fn split_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        if char_offset == 0
            && is_empty_topbottom_table_anchor_for_enter(
                &self.document.sections[section_idx].paragraphs[para_idx],
            )
        {
            self.document.sections[section_idx].raw_stream = None;
            let new_para_idx = para_idx + 1;
            let new_para = empty_paragraph_after_table_anchor(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.document.sections[section_idx]
                .paragraphs
                .insert(new_para_idx, new_para);

            let old_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            self.reflow_paragraph(section_idx, new_para_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                Some(new_para_idx..new_para_idx + 1),
                None,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.insert_composed_paragraph(section_idx, new_para_idx);
            self.paginate_if_needed();

            for _ in 0..2 {
                let new_col = self
                    .para_column_map
                    .get(section_idx)
                    .and_then(|m| m.get(para_idx))
                    .copied()
                    .unwrap_or(0);
                if new_col == old_col {
                    break;
                }
                self.reflow_paragraph(section_idx, new_para_idx);
                let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
                crate::renderer::composer::recalculate_section_vpos(
                    &mut self.document.sections[section_idx].paragraphs,
                    para_idx,
                    Some(new_para_idx..new_para_idx + 1),
                    None,
                    &self.styles,
                    self.dpi,
                    doc_hwp3_layout,
                );
                self.recompose_paragraph(section_idx, new_para_idx);
                self.paginate_if_needed();
            }

            self.event_log.push(DocumentEvent::ParagraphSplit {
                section: section_idx,
                para: para_idx,
                offset: char_offset,
            });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":0",
                new_para_idx
            )));
        }

        let square_ole_enter_chain = {
            let paragraphs = &self.document.sections[section_idx].paragraphs;
            (char_offset == 0)
                .then(|| square_ole_wrap_chain_for_enter(paragraphs, para_idx))
                .flatten()
        };
        if let Some(chain) = square_ole_enter_chain {
            self.document.sections[section_idx].raw_stream = None;
            let new_para_idx = para_idx + 1;
            // [Task #2299] 판정 기준을 실제 배치 기준과 일치시킨다: vpos 재계산이
            // 신규 문단을 anchor end + 문단 여백 gap 에 배치하므로, gap 을 빼고
            // 판정하면 wrap 영역 바깥(bottom_vpos 이하)에 wrap-줄 폭 문단이 놓인다.
            let anchor = &self.document.sections[section_idx].paragraphs[para_idx];
            // 신규 문단은 anchor 서식을 상속하므로(gap = anchor.after + anchor.before)
            // hwp3 변환은 spacing_before 성분에만 적용한다 — recalc 의 boundary_gap
            // 과 동일 산식.
            let enter_gap = {
                let (after, before) = self
                    .styles
                    .para_styles
                    .get(anchor.para_shape_id as usize)
                    .map(|style| (style.spacing_after, style.spacing_before))
                    .unwrap_or((0.0, 0.0));
                let before = crate::renderer::hwp3_variant_flow_spacing_before(
                    before,
                    self.document.layout_profile().hwp3_layout(),
                );
                crate::renderer::px_to_hwpunit(after + before, self.dpi)
            };
            let next_vpos = next_line_vpos_after_para_for_enter(anchor).saturating_add(enter_gap);
            let keep_wrap_zone = next_vpos < chain.bottom_vpos;
            let new_para = if keep_wrap_zone {
                empty_paragraph_after_square_wrap_anchor(
                    &self.document.sections[section_idx].paragraphs[para_idx],
                )
            } else {
                empty_paragraph_after_normal_flow(
                    &self.document.sections[section_idx].paragraphs[para_idx],
                )
            };
            self.document.sections[section_idx]
                .paragraphs
                .insert(new_para_idx, new_para);

            if !keep_wrap_zone {
                self.reflow_paragraph(section_idx, new_para_idx);
            }
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                Some(new_para_idx..new_para_idx + 1),
                None,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.insert_composed_paragraph(section_idx, new_para_idx);
            self.paginate_if_needed();

            self.event_log.push(DocumentEvent::ParagraphSplit {
                section: section_idx,
                para: para_idx,
                offset: char_offset,
            });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":0",
                new_para_idx
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);

        // 새 문단을 현재 문단 뒤에 삽입
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx]
            .paragraphs
            .insert(new_para_idx, new_para);
        for i in para_idx..=new_para_idx {
            if !self.document.sections[section_idx].paragraphs[i]
                .field_ranges
                .is_empty()
            {
                rebuild_char_offsets(&mut self.document.sections[section_idx].paragraphs[i]);
            }
        }

        // 양쪽 문단 리플로우 → vpos 재계산 → 재구성 → 재페이지네이션 + 다단 수렴 루프
        let old_col1 = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[para_idx],
        );
        self.reflow_paragraph(section_idx, para_idx);
        self.reflow_paragraph(section_idx, new_para_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            Some(new_para_idx..new_para_idx + 1),
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.insert_composed_paragraph(section_idx, new_para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col1 = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col1 == old_col1 {
                break;
            }
            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
            self.reflow_paragraph(section_idx, new_para_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                Some(new_para_idx..new_para_idx + 1),
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.recompose_paragraph(section_idx, new_para_idx);
            self.paginate_if_needed();
        }

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            new_para_idx
        )))
    }

    /// 강제 쪽 나누기 삽입 (Ctrl+Enter)
    /// 커서 위치에서 문단을 분할하고, 새 문단에 ColumnBreakType::Page를 설정한다.
    pub fn insert_page_break_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::ColumnBreakType;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx]
            .paragraphs
            .insert(new_para_idx, new_para);
        for i in para_idx..=new_para_idx {
            if !self.document.sections[section_idx].paragraphs[i]
                .field_ranges
                .is_empty()
            {
                rebuild_char_offsets(&mut self.document.sections[section_idx].paragraphs[i]);
            }
        }

        // 새 문단에 쪽 나누기 설정
        self.document.sections[section_idx].paragraphs[new_para_idx].column_type =
            ColumnBreakType::Page;
        self.document.sections[section_idx].paragraphs[new_para_idx].raw_break_type = 0x04;

        // 분할된 두 문단 리플로우
        self.reflow_paragraph(section_idx, para_idx);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[new_para_idx],
        );
        self.reflow_paragraph(section_idx, new_para_idx);

        // 삽입 지점부터 구역 끝까지 vpos 재계산 (페이지 재배치에 필요)
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            new_para_idx,
            Some(new_para_idx..new_para_idx + 1),
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );

        // 전체 구역 재구성 + 재페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            new_para_idx
        )))
    }

    /// 단 나누기 삽입 (Ctrl+Shift+Enter)
    /// 커서 위치에서 문단을 분리하고 새 문단에 단 나누기 설정.
    /// 1단 문서에서는 쪽 나누기와 동일하게 동작.
    pub fn insert_column_break_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::ColumnBreakType;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx]
            .paragraphs
            .insert(new_para_idx, new_para);

        // 새 문단에 단 나누기 설정
        self.document.sections[section_idx].paragraphs[new_para_idx].column_type =
            ColumnBreakType::Column;
        self.document.sections[section_idx].paragraphs[new_para_idx].raw_break_type = 0x08;

        // 분할된 두 문단 리플로우
        self.reflow_paragraph(section_idx, para_idx);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[new_para_idx],
        );
        self.reflow_paragraph(section_idx, new_para_idx);

        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            new_para_idx,
            Some(new_para_idx..new_para_idx + 1),
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            new_para_idx
        )))
    }

    /// 다단 설정 변경
    ///
    /// 구역의 초기 ColumnDef 컨트롤을 찾아 수정한다.
    /// 없으면 첫 문단에 새 ColumnDef를 삽입한다.
    ///
    /// ColumnDef는 문단 컨트롤로 저장되며, SectionDef와 독립적이다.
    /// 수정 후 recompose + repaginate로 조판을 갱신한다.
    pub fn set_column_def_native(
        &mut self,
        section_idx: usize,
        column_count: u16,
        column_type: u8, // 0=일반(Normal), 1=배분(Distribute), 2=평행(Parallel)
        same_width: bool,
        spacing_hu: i16, // 단 간격 (HWPUNIT)
    ) -> Result<String, HwpError> {
        use crate::model::page::{ColumnDirection, ColumnType};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }

        let col_type = match column_type {
            1 => ColumnType::Distribute,
            2 => ColumnType::Parallel,
            _ => ColumnType::Normal,
        };

        // 구역의 초기 ColumnDef 찾기 (find_initial_column_def와 동일 로직)
        let mut found = false;
        let paragraphs = &mut self.document.sections[section_idx].paragraphs;
        for para in paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::ColumnDef(ref mut cd) = ctrl {
                    cd.column_count = column_count;
                    cd.column_type = col_type;
                    cd.same_width = same_width;
                    cd.spacing = spacing_hu;
                    if same_width {
                        cd.widths.clear();
                        cd.gaps.clear();
                    }
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }

        // 기존 ColumnDef가 없으면 첫 문단에 삽입
        if !found {
            let cd = ColumnDef {
                column_count,
                column_type: col_type,
                same_width,
                spacing: spacing_hu,
                direction: ColumnDirection::LeftToRight,
                ..Default::default()
            };
            if !self.document.sections[section_idx].paragraphs.is_empty() {
                self.document.sections[section_idx].paragraphs[0]
                    .controls
                    .push(Control::ColumnDef(cd));
            }
        }

        // 조판 갱신
        self.document.sections[section_idx].raw_stream = None;
        self.rebuild_section(section_idx);

        Ok("{\"ok\":true}".to_string())
    }

    /// 문단 병합 (네이티브 에러 타입)
    pub fn merge_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx == 0 {
            return Err(HwpError::RenderError(
                "첫 번째 문단은 병합할 수 없습니다".to_string(),
            ));
        }
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        let preserve_square_ole_wrap_line = {
            let paragraphs = &self.document.sections[section_idx].paragraphs;
            let prev_idx = para_idx - 1;
            is_contentless_empty_paragraph_for_merge(&paragraphs[para_idx])
                && square_ole_wrap_chain_for_enter(paragraphs, prev_idx).is_some()
        };

        // 현재 문단을 이전 문단에 병합
        let current_para = self.document.sections[section_idx]
            .paragraphs
            .remove(para_idx);
        let prev_idx = para_idx - 1;
        let merge_point =
            self.document.sections[section_idx].paragraphs[prev_idx].merge_from(&current_para);

        if preserve_square_ole_wrap_line {
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                prev_idx,
                None,
                None,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.remove_composed_paragraph(section_idx, para_idx);
            self.recompose_paragraph(section_idx, prev_idx);
            self.paginate_if_needed();

            self.event_log.push(DocumentEvent::ParagraphMerged {
                section: section_idx,
                para: para_idx,
            });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}",
                prev_idx, merge_point
            )));
        }

        // 병합된 문단 리플로우 → vpos 재계산 → 재구성 → 재페이지네이션 + 다단 수렴 루프
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(prev_idx))
            .copied()
            .unwrap_or(0);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[prev_idx],
        );
        self.reflow_paragraph(section_idx, prev_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            prev_idx,
            None,
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.remove_composed_paragraph(section_idx, para_idx);
        self.recompose_paragraph(section_idx, prev_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(prev_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[prev_idx],
            );
            self.reflow_paragraph(section_idx, prev_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                prev_idx,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, prev_idx);
            self.paginate_if_needed();
        }

        self.event_log.push(DocumentEvent::ParagraphMerged {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":{}",
            prev_idx, merge_point
        )))
    }

    /// 문단 삭제 (네이티브 에러 타입)
    pub fn delete_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if section.paragraphs.len() <= 1 {
            return Err(HwpError::RenderError(
                "구역의 마지막 문단은 삭제할 수 없습니다".to_string(),
            ));
        }
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        let removed_char_count = self.document.sections[section_idx].paragraphs[para_idx]
            .text
            .chars()
            .count();
        self.document.sections[section_idx].raw_stream = None;
        self.document.sections[section_idx]
            .paragraphs
            .remove(para_idx);

        let reflow_idx = if para_idx > 0 { para_idx - 1 } else { 0 };
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(reflow_idx))
            .copied()
            .unwrap_or(0);
        self.remove_composed_paragraph(section_idx, para_idx);
        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = self.document.sections[section_idx]
            .paragraphs
            .get(reflow_idx)
            .and_then(crate::renderer::composer::paragraph_flow_end);
        if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
            self.reflow_paragraph(section_idx, reflow_idx);
        }
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            reflow_idx,
            None,
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
            self.recompose_paragraph(section_idx, reflow_idx);
        }
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(reflow_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = self.document.sections[section_idx]
                .paragraphs
                .get(reflow_idx)
                .and_then(crate::renderer::composer::paragraph_flow_end);
            if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
                self.reflow_paragraph(section_idx, reflow_idx);
            }
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                reflow_idx,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
                self.recompose_paragraph(section_idx, reflow_idx);
            }
            self.paginate_if_needed();
        }

        let new_count = self.document.sections[section_idx].paragraphs.len();
        self.event_log.push(DocumentEvent::ParagraphDeleted {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"removedCharCount\":{},\"newParagraphCount\":{}",
            removed_char_count, new_count
        )))
    }

    /// 빈 문단 삽입 (네이티브 에러 타입)
    ///
    /// `para_idx == paragraphs.len()` 이면 구역 끝에 추가(append).
    pub fn insert_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let para_count = self.document.sections[section_idx].paragraphs.len();
        if para_idx > para_count {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개, 최대 {})",
                para_idx, para_count, para_count
            )));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 새 문단은 앞 문단의 서식을 상속한다 (한글에서 문단 끝에 Enter 를 친 것과 같은 결과).
        // para_idx == 0 이면 앞 문단이 없으므로 뒤로 밀려날 현재 0번 문단을 상속원으로 쓴다.
        // 두 경우 모두 없을 때(빈 구역)만 상속원이 존재하지 않는다.
        let template_idx = para_idx.saturating_sub(1);
        let paragraphs = &mut self.document.sections[section_idx].paragraphs;
        let new_para = paragraphs
            .get(template_idx)
            .map_or_else(Paragraph::new_empty, Paragraph::new_empty_like);
        paragraphs.insert(para_idx, new_para);

        // 구역의 첫 문단은 "여기서 구역이 시작한다"는 나누기 표식을 지닌다. 이 표식은 문단
        // 내용이 아니라 **자리**에 딸린 속성이므로, 그 앞에 문단을 끼우면 새 첫 문단으로
        // 옮겨야 한다. 그대로 두면 밀려난 문단이 1번에서 계속 구역 시작을 주장해 거기서
        // 쪽이 끊기고, 새 문단만 홀로 남은 빈 쪽이 생긴다.
        //
        // 0번이 아닌 자리는 옮기지 않는다 — 그쪽 표식은 사용자가 그 문단에 직접 넣은
        // 쪽/단 나누기이므로 문단을 따라가는 게 맞다.
        if para_idx == 0 {
            if let [new_first, displaced, ..] = &mut paragraphs[..] {
                new_first.column_type = std::mem::take(&mut displaced.column_type);
                new_first.raw_break_type = std::mem::take(&mut displaced.raw_break_type);
            }
        }

        let reflow_target = if para_idx > 0 { para_idx - 1 } else { para_idx };
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(reflow_target))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            reflow_target,
            Some(para_idx..para_idx + 1),
            None,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.insert_composed_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(reflow_target))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            self.reflow_paragraph(section_idx, para_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                reflow_target,
                Some(para_idx..para_idx + 1),
                None,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();
        }

        let new_count = self.document.sections[section_idx].paragraphs.len();
        self.event_log.push(DocumentEvent::ParagraphInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"newParagraphCount\":{}",
            para_idx, new_count
        )))
    }

    /// 셀 내부 문단 분할 (네이티브 에러 타입)
    pub fn split_paragraph_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // 셀 문단 검증 및 분할
        let cell_para = self.get_cell_paragraph_mut(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        let original_vpos = cell_para.line_segs.first().map(|seg| seg.vertical_pos);
        let new_para = cell_para.split_at(char_offset);

        // 새 문단을 셀/글상자에 삽입
        let new_cell_para_idx = cell_para_idx + 1;
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                table.cells[cell_idx]
                    .paragraphs
                    .insert(new_cell_para_idx, new_para);
                table.dirty = true;
            }
            Some(Control::Shape(shape)) => {
                if let Some(tb) = super::super::helpers::get_textbox_from_shape_mut(shape) {
                    tb.paragraphs.insert(new_cell_para_idx, new_para);
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    cap.paragraphs.insert(new_cell_para_idx, new_para);
                }
            }
            _ => {}
        }

        // 양쪽 문단 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            new_cell_para_idx,
        );
        if let Some(vpos) = original_vpos {
            let cell_para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            shift_paragraph_vpos_origin(cell_para, vpos);
        }
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some(new_cell_para_idx),
        );

        // raw 스트림 무효화, section dirty, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":0",
            new_cell_para_idx
        )))
    }

    /// 셀 내부 문단 병합 (네이티브 에러 타입)
    ///
    /// cell_para_idx 문단을 이전 문단(cell_para_idx - 1)에 병합한다.
    pub fn merge_paragraph_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<String, HwpError> {
        if cell_para_idx == 0 {
            return Err(HwpError::RenderError(
                "셀 첫 번째 문단은 병합할 수 없습니다".to_string(),
            ));
        }

        // 검증: 셀 문단 인덱스 범위 확인
        {
            let cell_para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            let _ = cell_para; // 검증만 수행
        }

        // 문단 제거 및 이전 문단에 병합
        let prev_idx = cell_para_idx - 1;
        let original_vpos = self
            .get_cell_paragraph_ref(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                prev_idx,
            )
            .and_then(|para| para.line_segs.first().map(|seg| seg.vertical_pos));
        let merge_point;
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                let removed = table.cells[cell_idx].paragraphs.remove(cell_para_idx);
                merge_point = table.cells[cell_idx].paragraphs[prev_idx].merge_from(&removed);
                table.dirty = true;
            }
            Some(Control::Shape(shape)) => {
                if let Some(tb) = super::super::helpers::get_textbox_from_shape_mut(shape) {
                    let removed = tb.paragraphs.remove(cell_para_idx);
                    merge_point = tb.paragraphs[prev_idx].merge_from(&removed);
                } else {
                    return Err(HwpError::RenderError(
                        "지정된 Shape 컨트롤에 텍스트 박스가 없습니다".to_string(),
                    ));
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    let removed = cap.paragraphs.remove(cell_para_idx);
                    merge_point = cap.paragraphs[prev_idx].merge_from(&removed);
                } else {
                    return Err(HwpError::RenderError(
                        "지정된 그림 컨트롤에 캡션이 없습니다".to_string(),
                    ));
                }
            }
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표, 글상자 또는 그림이 아닙니다".to_string(),
                ));
            }
        }

        // 병합된 문단 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            prev_idx,
        );
        if let Some(vpos) = original_vpos {
            let cell_para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                prev_idx,
            )?;
            shift_paragraph_vpos_origin(cell_para, vpos);
        }
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            prev_idx,
            None,
        );

        // raw 스트림 무효화, section dirty, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":{}",
            prev_idx, merge_point
        )))
    }

    // ─── Phase 1 Native: 기본 편집 보조 API ────────────────────

    /// 구역 내 문단 수 (네이티브)
    pub fn get_paragraph_count_native(&self, section_idx: usize) -> Result<usize, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        Ok(section.paragraphs.len())
    }

    /// 문단 글자 수 (네이티브)
    pub fn get_paragraph_length_native(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<usize, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            ))
        })?;
        Ok(para.text.chars().count())
    }

    /// 문단에 텍스트박스가 있는 Shape 컨트롤의 인덱스를 반환 (네이티브)
    /// 없으면 -1 반환
    pub fn get_textbox_control_index_native(&self, section_idx: usize, para_idx: usize) -> i32 {
        let section = match self.document.sections.get(section_idx) {
            Some(s) => s,
            None => return -1,
        };
        let para = match section.paragraphs.get(para_idx) {
            Some(p) => p,
            None => return -1,
        };
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Shape(shape) = ctrl {
                if get_textbox_from_shape(shape.as_ref()).is_some() {
                    return ci as i32;
                }
            }
        }
        -1
    }

    /// 문서 트리에서 다음 편집 가능한 컨트롤/본문을 찾는다.
    /// `(sec, para, ctrl_idx)`에서 시작, delta=+1(앞), delta=-1(뒤) 방향으로 탐색.
    /// ctrl_idx가 -1이면 해당 문단의 본문 텍스트에서 출발한 것으로 간주.
    ///
    /// 반환 JSON:
    ///   `{"type":"textbox","sec":N,"para":N,"ci":N}`
    ///   `{"type":"table","sec":N,"para":N,"ci":N}`
    ///   `{"type":"body","sec":N,"para":N}`
    ///   `{"type":"none"}`
    pub fn find_next_editable_control_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        ctrl_idx: i32,
        delta: i32,
    ) -> String {
        let sections = &self.document.sections;

        // 헬퍼: 문단 내 편집 가능한 컨트롤을 방향에 따라 검색
        fn find_in_para(
            sections: &[crate::model::document::Section],
            sec: usize,
            para: usize,
            start_ci: i32,
            forward: bool,
        ) -> Option<(usize, &'static str)> {
            let section = sections.get(sec)?;
            let p = section.paragraphs.get(para)?;
            let controls = &p.controls;
            if forward {
                let from = if start_ci < 0 {
                    0usize
                } else {
                    (start_ci as usize) + 1
                };
                for ci in from..controls.len() {
                    match &controls[ci] {
                        Control::Shape(shape) => {
                            if get_textbox_from_shape(shape.as_ref()).is_some() {
                                return Some((ci, "textbox"));
                            }
                        }
                        Control::Table(_) => {
                            return Some((ci, "table"));
                        }
                        _ => {}
                    }
                }
            } else {
                let until = if start_ci < 0 {
                    controls.len()
                } else {
                    start_ci as usize
                };
                for ci in (0..until).rev() {
                    match &controls[ci] {
                        Control::Shape(shape) => {
                            if get_textbox_from_shape(shape.as_ref()).is_some() {
                                return Some((ci, "textbox"));
                            }
                        }
                        Control::Table(_) => {
                            return Some((ci, "table"));
                        }
                        _ => {}
                    }
                }
            }
            None
        }

        // 헬퍼: 문단이 편집 가능한 컨트롤을 하나라도 갖고 있는지
        fn has_navigable_control(
            sections: &[crate::model::document::Section],
            sec: usize,
            para: usize,
        ) -> bool {
            sections.get(sec)
                .and_then(|s| s.paragraphs.get(para))
                .map(|p| p.controls.iter().any(|c| {
                    matches!(c, Control::Table(_))
                    || matches!(c, Control::Shape(s) if get_textbox_from_shape(s.as_ref()).is_some())
                }))
                .unwrap_or(false)
        }

        let forward = delta > 0;

        // 1) 같은 문단에서 탐색
        if let Some((ci, ty)) = find_in_para(sections, section_idx, para_idx, ctrl_idx, forward) {
            return format!(
                "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}",
                ty, section_idx, para_idx, ci
            );
        }

        // 2) 같은 섹션의 다른 문단 탐색
        if let Some(section) = sections.get(section_idx) {
            let para_count = section.paragraphs.len();
            let para_range: Box<dyn Iterator<Item = usize>> = if forward {
                Box::new((para_idx + 1)..para_count)
            } else if para_idx > 0 {
                Box::new((0..para_idx).rev())
            } else {
                Box::new(std::iter::empty())
            };
            for pi in para_range {
                let search_start = if forward {
                    -1
                } else {
                    section.paragraphs[pi].controls.len() as i32
                };
                if let Some((ci, ty)) =
                    find_in_para(sections, section_idx, pi, search_start, forward)
                {
                    return format!(
                        "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}",
                        ty, section_idx, pi, ci
                    );
                }
                // 네비게이션 가능한 컨트롤이 없는 문단 → body
                if !has_navigable_control(sections, section_idx, pi) {
                    return format!(
                        "{{\"type\":\"body\",\"sec\":{},\"para\":{}}}",
                        section_idx, pi
                    );
                }
            }
        }

        // 3) 다른 섹션 탐색
        let sec_range: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new((section_idx + 1)..sections.len())
        } else if section_idx > 0 {
            Box::new((0..section_idx).rev())
        } else {
            Box::new(std::iter::empty())
        };
        for si in sec_range {
            if let Some(section) = sections.get(si) {
                let para_range: Box<dyn Iterator<Item = usize>> = if forward {
                    Box::new(0..section.paragraphs.len())
                } else {
                    Box::new((0..section.paragraphs.len()).rev())
                };
                for pi in para_range {
                    let search_start = if forward {
                        -1
                    } else {
                        section.paragraphs[pi].controls.len() as i32
                    };
                    if let Some((ci, ty)) = find_in_para(sections, si, pi, search_start, forward) {
                        return format!(
                            "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}",
                            ty, si, pi, ci
                        );
                    }
                    if !has_navigable_control(sections, si, pi) {
                        return format!("{{\"type\":\"body\",\"sec\":{},\"para\":{}}}", si, pi);
                    }
                }
            }
        }

        // 4) 문서 경계
        "{\"type\":\"none\"}".to_string()
    }

    /// 커서에서 이전 방향으로 가장 가까운 선택 가능 컨트롤을 찾는다.
    /// F11 키 기능: 표, 그림, 글상자, 수식, 누름틀 등을 객체 선택.
    ///
    /// 반환 JSON:
    ///   `{"type":"table"|"shape"|"picture"|"equation"|"field","sec":N,"para":N,"ci":N}`
    ///   `{"type":"none"}`
    pub fn find_nearest_control_backward_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> String {
        let sections = &self.document.sections;

        // 컨트롤 타입 분류 (선택 가능한 것만)
        fn classify_control(ctrl: &Control) -> Option<&'static str> {
            match ctrl {
                Control::Table(_) => Some("table"),
                Control::Picture(_) => Some("picture"),
                Control::Shape(_) => Some("shape"),
                Control::Equation(_) => Some("equation"),
                Control::Field(_) => Some("field"),
                Control::Bookmark(_) => Some("bookmark"),
                _ => None,
            }
        }

        // 문단 내에서 char_offset 이전의 컨트롤을 역순으로 탐색
        fn find_in_para_before(
            para: &crate::model::paragraph::Paragraph,
            char_offset: usize,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in (0..para.controls.len()).rev() {
                if let Some(&pos) = positions.get(ci) {
                    if pos < char_offset {
                        if let Some(ty) = classify_control(&para.controls[ci]) {
                            return Some((ci, pos, ty));
                        }
                    }
                }
            }
            None
        }

        // 문단 전체에서 마지막 선택 가능 컨트롤 찾기
        fn find_last_in_para(
            para: &crate::model::paragraph::Paragraph,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in (0..para.controls.len()).rev() {
                if let Some(ty) = classify_control(&para.controls[ci]) {
                    let pos = positions.get(ci).copied().unwrap_or(0);
                    return Some((ci, pos, ty));
                }
            }
            None
        }

        fn fmt_result(ty: &str, sec: usize, para: usize, ci: usize, char_pos: usize) -> String {
            format!(
                "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{},\"charPos\":{}}}",
                ty, sec, para, ci, char_pos
            )
        }

        // 1) 같은 문단에서 char_offset 이전 탐색
        if let Some(section) = sections.get(section_idx) {
            if let Some(para) = section.paragraphs.get(para_idx) {
                if let Some((ci, cp, ty)) = find_in_para_before(para, char_offset) {
                    return fmt_result(ty, section_idx, para_idx, ci, cp);
                }
            }
        }

        // 2) 이전 문단들 역순 탐색 (같은 섹션)
        if let Some(section) = sections.get(section_idx) {
            for pi in (0..para_idx).rev() {
                if let Some((ci, cp, ty)) = find_last_in_para(&section.paragraphs[pi]) {
                    return fmt_result(ty, section_idx, pi, ci, cp);
                }
            }
        }

        // 3) 이전 섹션 역순 탐색
        for si in (0..section_idx).rev() {
            if let Some(section) = sections.get(si) {
                for pi in (0..section.paragraphs.len()).rev() {
                    if let Some((ci, cp, ty)) = find_last_in_para(&section.paragraphs[pi]) {
                        return fmt_result(ty, si, pi, ci, cp);
                    }
                }
            }
        }

        "{\"type\":\"none\"}".to_string()
    }

    /// 현재 위치 이후의 가장 가까운 선택 가능 컨트롤을 찾는다 (Shift+F11).
    pub fn find_nearest_control_forward_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> String {
        let sections = &self.document.sections;

        fn classify_control(ctrl: &Control) -> Option<&'static str> {
            match ctrl {
                Control::Table(_) => Some("table"),
                Control::Picture(_) => Some("picture"),
                Control::Shape(_) => Some("shape"),
                Control::Equation(_) => Some("equation"),
                Control::Field(_) => Some("field"),
                Control::Bookmark(_) => Some("bookmark"),
                _ => None,
            }
        }

        fn find_in_para_after(
            para: &crate::model::paragraph::Paragraph,
            char_offset: usize,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in 0..para.controls.len() {
                if let Some(&pos) = positions.get(ci) {
                    if pos > char_offset {
                        if let Some(ty) = classify_control(&para.controls[ci]) {
                            return Some((ci, pos, ty));
                        }
                    }
                }
            }
            None
        }

        fn find_first_in_para(
            para: &crate::model::paragraph::Paragraph,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in 0..para.controls.len() {
                if let Some(ty) = classify_control(&para.controls[ci]) {
                    let pos = positions.get(ci).copied().unwrap_or(0);
                    return Some((ci, pos, ty));
                }
            }
            None
        }

        fn fmt_result(ty: &str, sec: usize, para: usize, ci: usize, char_pos: usize) -> String {
            format!(
                "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{},\"charPos\":{}}}",
                ty, sec, para, ci, char_pos
            )
        }

        // 1) 같은 문단에서 char_offset 이후 탐색
        if let Some(section) = sections.get(section_idx) {
            if let Some(para) = section.paragraphs.get(para_idx) {
                if let Some((ci, cp, ty)) = find_in_para_after(para, char_offset) {
                    return fmt_result(ty, section_idx, para_idx, ci, cp);
                }
            }
        }

        // 2) 이후 문단 정순 탐색 (같은 섹션)
        if let Some(section) = sections.get(section_idx) {
            for pi in (para_idx + 1)..section.paragraphs.len() {
                if let Some((ci, cp, ty)) = find_first_in_para(&section.paragraphs[pi]) {
                    return fmt_result(ty, section_idx, pi, ci, cp);
                }
            }
        }

        // 3) 이후 섹션 정순 탐색
        for si in (section_idx + 1)..sections.len() {
            if let Some(section) = sections.get(si) {
                for pi in 0..section.paragraphs.len() {
                    if let Some((ci, cp, ty)) = find_first_in_para(&section.paragraphs[pi]) {
                        return fmt_result(ty, si, pi, ci, cp);
                    }
                }
            }
        }

        "{\"type\":\"none\"}".to_string()
    }

    /// 문단 텍스트 부분 추출 (네이티브)
    pub fn get_text_range_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            ))
        })?;
        let text_chars: Vec<char> = para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (문단 길이 {})",
                char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        let result: String = text_chars[char_offset..end].iter().collect();
        Ok(result)
    }

    /// 셀 내 문단 수 (네이티브)
    pub fn get_cell_paragraph_count_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<usize, HwpError> {
        let para = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
        match para.controls.get(control_idx) {
            Some(Control::Table(table)) => {
                if cell_idx == 65534 {
                    let cap = table
                        .caption
                        .as_ref()
                        .ok_or_else(|| HwpError::RenderError("표에 캡션이 없습니다".to_string()))?;
                    return Ok(cap.paragraphs.len());
                }
                let cell = table.cells.get(cell_idx).ok_or_else(|| {
                    HwpError::RenderError(format!(
                        "셀 인덱스 {} 범위 초과 (총 {}개)",
                        cell_idx,
                        table.cells.len()
                    ))
                })?;
                Ok(cell.paragraphs.len())
            }
            Some(Control::Shape(shape)) => {
                let text_box = get_textbox_from_shape(shape)
                    .ok_or_else(|| HwpError::RenderError("도형에 글상자가 없습니다".to_string()))?;
                Ok(text_box.paragraphs.len())
            }
            Some(Control::Picture(pic)) => {
                let caption = pic
                    .caption
                    .as_ref()
                    .ok_or_else(|| HwpError::RenderError("그림에 캡션이 없습니다".to_string()))?;
                Ok(caption.paragraphs.len())
            }
            _ => Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {}가 표/글상자가 아닙니다",
                control_idx
            ))),
        }
    }

    /// 셀 내 문단 글자 수 (네이티브)
    pub fn get_cell_paragraph_length_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<usize, HwpError> {
        let cell_para = self
            .get_cell_paragraph_ref(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "셀 문단 접근 실패: sec={}, para={}, ctrl={}, cell={}, cellPara={}",
                    section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
                ))
            })?;
        Ok(cell_para.text.chars().count())
    }

    /// 셀 내 텍스트 부분 추출 (네이티브)
    pub fn get_text_in_cell_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let cell_para = self
            .get_cell_paragraph_ref(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "셀 문단 접근 실패: sec={}, para={}, ctrl={}, cell={}, cellPara={}",
                    section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
                ))
            })?;
        let text_chars: Vec<char> = cell_para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (셀 문단 길이 {})",
                char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        let result: String = text_chars[char_offset..end].iter().collect();
        Ok(result)
    }

    // ─── Phase 1 Native 끝 ──────────────────────────────────

    // ─── Phase 2 Native: 커서/히트 테스트 API ────────────────────

    /// 문단이 포함된 글로벌 페이지 번호 목록을 반환한다.
    pub(crate) fn find_pages_for_paragraph(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<Vec<u32>, HwpError> {
        if section_idx >= self.pagination.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.pagination.len()
            )));
        }
        let mut global_offset = 0u32;
        for (sec_i, pr) in self.pagination.iter().enumerate() {
            if sec_i == section_idx {
                let mut result = Vec::new();
                for (local_i, page) in pr.pages.iter().enumerate() {
                    let global_page = global_offset + local_i as u32;
                    for col in &page.column_contents {
                        for item in &col.items {
                            let pi = match item {
                                crate::renderer::pagination::PageItem::FullParagraph {
                                    para_index,
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialParagraph {
                                    para_index,
                                    ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Table {
                                    para_index, ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialTable {
                                    para_index,
                                    ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Shape {
                                    para_index, ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::EndnoteSeparator {
                                    ..
                                } => None,
                            };
                            if pi == Some(para_idx) {
                                if result.last() != Some(&global_page) {
                                    result.push(global_page);
                                }
                            }
                        }
                        // 어울림 문단도 페이지 탐색 대상에 포함
                        for wp in &col.wrap_around_paras {
                            if wp.para_index == para_idx || wp.table_para_index == para_idx {
                                if result.last() != Some(&global_page) {
                                    result.push(global_page);
                                }
                            }
                        }
                    }
                }
                // 전역 wrap_around_paras에서도 확인
                if result.is_empty() {
                    for wp in &pr.wrap_around_paras {
                        if wp.para_index == para_idx {
                            // 표 호스트 문단의 페이지에서 렌더링됨
                            if let Ok(table_pages) =
                                self.find_pages_for_paragraph(section_idx, wp.table_para_index)
                            {
                                return Ok(table_pages);
                            }
                        }
                    }
                }
                return if result.is_empty() {
                    Err(HwpError::RenderError(format!(
                        "문단 (sec={}, para={})이 페이지에 없습니다",
                        section_idx, para_idx
                    )))
                } else {
                    Ok(result)
                };
            }
            global_offset += pr.pages.len() as u32;
        }
        Err(HwpError::RenderError(format!(
            "구역 인덱스 {} 범위 초과",
            section_idx
        )))
    }
}

fn find_text_y(node: &crate::renderer::render_tree::RenderNode, text: &str) -> Option<f64> {
    use crate::renderer::render_tree::RenderNodeType;
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(text) {
            return Some(node.bbox.y);
        }
    }
    for child in &node.children {
        if let Some(y) = find_text_y(child, text) {
            return Some(y);
        }
    }
    None
}

// ─── 중첩 표 path 기반 편집 API ──────────────────────────────────

impl DocumentCore {
    /// cellPath를 따라가서 최종 셀의 문단 목록에 대한 가변 참조를 얻는다.
    /// path: [(control_index, cell_index, cell_para_index), ...]
    pub(crate) fn get_cell_paragraphs_mut_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&mut Vec<Paragraph>, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let mut para: &mut Paragraph = section
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let is_last = i == path.len() - 1;
            let paragraphs = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => {
                    let cell = t.cells.get_mut(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!("경로[{}]: 셀 {} 범위 초과", i, cell_idx))
                    })?;
                    &mut cell.paragraphs
                }
                Some(Control::Shape(shape)) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let text_box = super::super::helpers::get_textbox_from_shape_mut(shape)
                        .ok_or_else(|| {
                            HwpError::RenderError(format!(
                                "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                                i, ctrl_idx
                            ))
                        })?;
                    &mut text_box.paragraphs
                }
                Some(Control::Picture(pic)) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 그림 캡션의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let caption = pic.caption.as_mut().ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: controls[{}] 그림에 캡션이 없습니다",
                            i, ctrl_idx
                        ))
                    })?;
                    &mut caption.paragraphs
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{}]: controls[{}]가 표/글상자/그림 캡션이 아닙니다",
                        i, ctrl_idx
                    )))
                }
            };
            if is_last {
                return Ok(paragraphs);
            }
            para = paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!(
                    "경로[{}]: 컨테이너 문단 {} 범위 초과",
                    i, cell_para_idx
                ))
            })?;
        }
        unreachable!()
    }

    /// cellPath를 따라가서 최종 셀의 문단에 대한 가변 참조를 얻는다.
    /// path: [(control_index, cell_index, cell_para_index), ...]
    pub(crate) fn get_cell_paragraph_mut_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&mut Paragraph, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }
        let last_path_index = path.len() - 1;
        let cell_para_idx = path[last_path_index].2;
        let cell_paragraphs =
            self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)?;
        cell_paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "경로[{}]: 셀문단 {} 범위 초과",
                last_path_index, cell_para_idx
            ))
        })
    }

    /// path 기반 셀 텍스트 삽입 (중첩 표 지원)
    pub fn insert_text_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        // 깊이 1 표는 일반 셀 삽입 경로가 셀 폭 리플로우와 vpos 재계산을 이미 담당한다.
        // IME가 cellPath를 항상 전달하더라도 같은 편집 계약을 사용해야 한다.
        if path.len() == 1 {
            let (control_idx, cell_idx, cell_para_idx) = path[0];
            return self.insert_text_in_cell_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                text,
            );
        }

        let new_chars_count = text.chars().count();
        let active_field = self.active_field.clone();
        let cell_para = self.get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, path)?;
        let cell_para_idx = path.last().map(|entry| entry.2).unwrap_or(0);
        let outside_insertions = inactive_field_end_insertions(
            cell_para,
            active_field.as_ref(),
            section_idx,
            cell_para_idx,
            Some(path),
            char_offset,
        );
        let before_insertions = inactive_field_start_insertions(
            cell_para,
            active_field.as_ref(),
            section_idx,
            cell_para_idx,
            Some(path),
            char_offset,
        );
        cell_para.insert_text_at(char_offset, text);
        keep_inactive_field_start_outside(cell_para, &before_insertions, new_chars_count);
        keep_inactive_field_end_outside(cell_para, &outside_insertions, new_chars_count);
        if has_clickhere_field_range(cell_para) {
            rebuild_char_offsets(cell_para);
        }

        // 최외곽 표 dirty 마킹
        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);

        // 리플로우 (최외곽 표 기준 — 중첩 표 셀 폭은 별도 계산이 필요하나 우선 section dirty로 처리)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
    }

    /// path 기반 셀 텍스트 삭제 (중첩 표 지원)
    pub fn delete_text_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        // [#2755] 깊이 1 표/글상자/캡션은 flat 셀 삭제 경로가 셀 폭 리플로우와 vpos 재계산을
        // 이미 담당한다. `insert_text_in_cell_by_path`(:3389)의 깊이 1 위임 가드와 동형이며,
        // flat `delete_text_in_cell_native`(:955)는 모든 컨테이너 종류를 처리한다.
        if path.len() == 1 {
            let (control_idx, cell_idx, cell_para_idx) = path[0];
            return self.delete_text_in_cell_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                count,
            );
        }

        let cell_para = self.get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, path)?;
        cell_para.delete_text_at(char_offset, count);

        // [#2755] 깊이 ≥ 2 중첩 셀도 flat `delete_text_in_cell_native` 처럼 최내곽 셀 폭으로
        // 재래핑하고 vpos 를 재계산한다(깊이 1 은 위 위임 가드가 flat 경로로 처리).
        let inner_cell_para_idx = path.last().map(|e| e.2).unwrap_or(0);
        self.reflow_cell_paragraph_by_path(section_idx, parent_para_idx, path, inner_cell_para_idx);
        self.recalculate_cell_paragraph_vpos_by_path(
            section_idx,
            parent_para_idx,
            path,
            inner_cell_para_idx,
            None,
        );

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
    }

    /// path 기반 셀 내 범위 삭제 (중첩 표 지원). `deleteRangeInCell`(flat)의 cellPath 변형.
    ///
    /// flat deleteRangeInCell 은 controlIndex/cellIndex 를 최외곽(cellPath[0]) 축으로 받아
    /// 중첩 셀에서 바깥 셀을 삭제한다. 이 변형은 path 로 최내곽 셀을 해석해 그 셀의
    /// 문단 목록에 직접 범위 삭제를 적용한다. start_para/end_para 는 최내곽 셀 내부 인덱스다.
    #[allow(clippy::too_many_arguments)]
    pub fn delete_range_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        start_para: usize,
        start_offset: usize,
        end_para: usize,
        end_offset: usize,
    ) -> Result<String, HwpError> {
        {
            let paras = self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)?;
            if start_para == end_para {
                let count = end_offset.saturating_sub(start_offset);
                if count > 0 {
                    if let Some(p) = paras.get_mut(start_para) {
                        p.delete_text_at(start_offset, count);
                    }
                }
            } else {
                // 1) 마지막 문단 앞부분 삭제
                if end_offset > 0 {
                    if let Some(p) = paras.get_mut(end_para) {
                        p.delete_text_at(0, end_offset);
                    }
                }
                // 2) 중간 문단 역순 제거
                for mid in (start_para + 1..end_para).rev() {
                    if mid < paras.len() {
                        paras.remove(mid);
                    }
                }
                // 3) 첫 문단 뒷부분 삭제
                if let Some(p) = paras.get_mut(start_para) {
                    let para_len = p.text.chars().count();
                    if start_offset < para_len {
                        p.delete_text_at(start_offset, para_len - start_offset);
                    }
                }
                // 4) 첫-마지막 문단 병합 (마지막이 이제 start_para+1)
                if start_para + 1 < paras.len() {
                    let next = paras.remove(start_para + 1);
                    paras[start_para].merge_from(&next);
                }
            }
        }

        // [#2755] flat `delete_range_native` 셀 분기와 동일하게 병합 생존 문단(start_para)을
        // 셀 폭 기준으로 재래핑한다. by_path 리플로우는 최내곽 셀 폭을 해석하므로 깊이 1·2+ 를
        // 모두 처리하고, by_path 본문이 표/글상자/그림 캡션의 다중 문단까지 처리하는 것과도
        // 정합한다(flat delete_range 는 vpos 재계산을 하지 않으므로 여기서도 리플로우만 한다).
        self.reflow_cell_paragraph_by_path(section_idx, parent_para_idx, path, start_para);

        // dirty/이벤트는 delete_text_in_cell_by_path 와 동형(최외곽 컨트롤 기준).
        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":{}",
            start_para, start_offset
        )))
    }

    /// path 기반 셀 문단 분할 (중첩 표 지원)
    pub fn split_paragraph_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // [#2755] 빈 경로는 패닉이 아니라 Err 로 거절한다. `parse_cell_path` 가 "[]" 에
        // Ok(Vec::new()) 를 반환하므로 빈 경로가 여기 도달할 수 있고, wasm 에서 Rust 패닉은
        // HwpDocument 인스턴스 전체를 무효화한다. get_cell_paragraph(s)_mut_by_path 형제와 동형.
        let last = path
            .last()
            .ok_or_else(|| HwpError::RenderError("경로가 비어있습니다".to_string()))?;
        let cell_para_idx = last.2;
        // [#2755] 리플로우 후 재적용할 원본 vpos(분할 대상 문단 첫 seg).
        let mut split_origin_vpos: Option<i32> = None;

        // 셀에 접근하여 문단 분할
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let mut para: &mut Paragraph = section
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        // path를 따라 마지막 셀까지 진입
        for (i, &(ctrl_idx, cell_idx, _cpi)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };
            let cell = table
                .cells
                .get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;
            if i == path.len() - 1 {
                // 이 셀에서 문단 분할. 리플로우/shift/recalc 는 borrow 해제 후 루프 밖에서 한다.
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError("셀문단 범위 초과".to_string()));
                }
                split_origin_vpos = cell.paragraphs[cell_para_idx]
                    .line_segs
                    .first()
                    .map(|seg| seg.vertical_pos);
                let new_para = cell.paragraphs[cell_para_idx].split_at(char_offset);
                cell.paragraphs.insert(cell_para_idx + 1, new_para);
                break;
            }
            para = cell
                .paragraphs
                .get_mut(_cpi)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        // [#2755] flat split 형제(:2387)와 동일하게 분할된 두 문단을 셀 폭으로 재래핑한 뒤
        // vpos 를 재계산한다(리플로우가 line_segs 를 재작성하므로 shift/recalc 를 그 뒤에 둔다).
        self.reflow_cell_paragraph_by_path(section_idx, parent_para_idx, path, cell_para_idx);
        self.reflow_cell_paragraph_by_path(section_idx, parent_para_idx, path, cell_para_idx + 1);
        if let Some(vpos) = split_origin_vpos {
            if let Ok(paras) =
                self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)
            {
                if let Some(p) = paras.get_mut(cell_para_idx) {
                    shift_paragraph_vpos_origin(p, vpos);
                }
            }
        }
        self.recalculate_cell_paragraph_vpos_by_path(
            section_idx,
            parent_para_idx,
            path,
            cell_para_idx,
            Some(cell_para_idx + 1),
        );

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        let new_cpi = cell_para_idx + 1;
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":0",
            new_cpi
        )))
    }

    /// path 기반 셀 문단 병합 (중첩 표 지원)
    pub fn merge_paragraph_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<String, HwpError> {
        // [#2755] 빈 경로는 패닉이 아니라 Err 로 거절한다(split_paragraph_in_cell_by_path 동형).
        let last = path
            .last()
            .ok_or_else(|| HwpError::RenderError("경로가 비어있습니다".to_string()))?;
        let cell_para_idx = last.2;
        if cell_para_idx == 0 {
            return Err(HwpError::RenderError(
                "첫 문단은 병합할 수 없습니다".to_string(),
            ));
        }
        let prev_idx = cell_para_idx - 1;
        // [#2755] 리플로우 후 재적용할 원본 vpos(병합 생존 문단 첫 seg).
        let mut merge_origin_vpos: Option<i32> = None;

        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let mut para: &mut Paragraph = section
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        let mut merge_point = 0usize;
        for (i, &(ctrl_idx, cell_idx, _cpi)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };
            let cell = table
                .cells
                .get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;
            if i == path.len() - 1 {
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError("셀문단 범위 초과".to_string()));
                }
                merge_origin_vpos = cell.paragraphs[prev_idx]
                    .line_segs
                    .first()
                    .map(|seg| seg.vertical_pos);
                let removed = cell.paragraphs.remove(cell_para_idx);
                let prev = &mut cell.paragraphs[prev_idx];
                merge_point = prev.text.chars().count();
                prev.merge_from(&removed);
                break;
            }
            para = cell
                .paragraphs
                .get_mut(_cpi)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        // [#2755] flat merge 형제(:2486)와 동일하게 병합 생존 문단을 셀 폭으로 재래핑한 뒤
        // vpos 를 재계산한다(리플로우가 line_segs 를 재작성하므로 shift/recalc 를 그 뒤에 둔다).
        self.reflow_cell_paragraph_by_path(section_idx, parent_para_idx, path, prev_idx);
        if let Some(vpos) = merge_origin_vpos {
            if let Ok(paras) =
                self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)
            {
                if let Some(p) = paras.get_mut(prev_idx) {
                    shift_paragraph_vpos_origin(p, vpos);
                }
            }
        }
        self.recalculate_cell_paragraph_vpos_by_path(
            section_idx,
            parent_para_idx,
            path,
            prev_idx,
            None,
        );

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        let prev_cpi = cell_para_idx - 1;
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":{}",
            prev_cpi, merge_point
        )))
    }

    /// path 기반 셀 텍스트 조회 (중첩 표 지원)
    pub fn get_text_in_cell_by_path(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, path)?;
        let text_chars: Vec<char> = para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (셀 문단 길이 {})",
                char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        Ok(text_chars[char_offset..end].iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::CharShapeRef;

    /// insert_paragraph_native 는 새 문단의 서식을 이웃에서 상속해야 한다.
    ///
    /// `Paragraph::new_empty()` 를 그대로 삽입하면 para_shape_id/style_id 는 0 이 되고
    /// char_shapes 는 비어 저장기가 charPrIDRef="0" 을 쓴다. 0 은 기본 서식이 아니라
    /// 문서 header 의 0번 항목이므로, 삽입된 문단만 다른 서식으로 보인다.
    ///
    /// 경계별로 검증한다: para_idx == 0 / 중간 / 끝(== len) / 상속원 없음.
    fn set_shape(
        core: &mut DocumentCore,
        idx: usize,
        para_shape_id: u16,
        style_id: u8,
        char_shape_id: u32,
    ) {
        let para = &mut core.document.sections[0].paragraphs[idx];
        para.para_shape_id = para_shape_id;
        para.style_id = style_id;
        para.char_shapes = vec![CharShapeRef {
            start_pos: 0,
            char_shape_id,
        }];
    }

    fn shape_of(core: &DocumentCore, idx: usize) -> (u16, u8, Option<u32>) {
        let p = &core.document.sections[0].paragraphs[idx];
        (
            p.para_shape_id,
            p.style_id,
            p.char_shapes.first().map(|cs| cs.char_shape_id),
        )
    }

    /// 서로 다른 서식을 가진 두 문단을 공개 API 로만 구성한다
    /// (composed / para_column_map 을 엔진이 동기화하도록 둔다).
    fn core_with_two_shaped_paragraphs() -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "첫째").unwrap();
        core.split_paragraph_native(0, 0, 2).unwrap();
        core.insert_text_native(0, 1, 0, "둘째").unwrap();
        set_shape(&mut core, 0, 12, 3, 7);
        set_shape(&mut core, 1, 14, 5, 9);
        core
    }

    #[test]
    fn delete_range_in_cell_by_path_deletes_within_resolved_cell() {
        use crate::model::control::Control;
        use crate::model::table::{Cell, Table};

        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let mut cell_para = Paragraph::default();
        cell_para.text = "ABCDE".to_string();
        cell_para.char_count = 5;
        cell_para.char_offsets = vec![0, 1, 2, 3, 4];
        let table = Table {
            cells: vec![Cell {
                paragraphs: vec![cell_para],
                ..Default::default()
            }],
            ..Default::default()
        };
        core.document.sections[0].paragraphs[0]
            .controls
            .push(Control::Table(Box::new(table)));
        let ctrl_idx = core.document.sections[0].paragraphs[0].controls.len() - 1;

        // 셀 문단 offset 1..3(BC) 삭제. path 로 최내곽 셀을 해석해야 한다.
        core.delete_range_in_cell_by_path(0, 0, &[(ctrl_idx, 0, 0)], 0, 1, 0, 3)
            .unwrap();

        let Control::Table(t) = &core.document.sections[0].paragraphs[0].controls[ctrl_idx] else {
            panic!("expected table");
        };
        assert_eq!(
            t.cells[0].paragraphs[0].text, "ADE",
            "path 로 해석한 셀에서 BC 가 삭제돼야 한다"
        );
    }

    #[test]
    fn delete_range_in_nested_cell_by_path_preserves_outer_cell() {
        use crate::model::control::Control;
        use crate::model::table::{Cell, Table};

        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let mut inner_para = Paragraph::default();
        inner_para.text = "INNER".to_string();
        inner_para.char_count = 5;
        inner_para.char_offsets = vec![0, 1, 2, 3, 4];
        let nested_table = Table {
            cells: vec![Cell {
                paragraphs: vec![inner_para],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut outer_para = Paragraph::default();
        outer_para.text = "OUTER".to_string();
        outer_para.char_count = 5;
        outer_para.char_offsets = vec![0, 1, 2, 3, 4];
        outer_para
            .controls
            .push(Control::Table(Box::new(nested_table)));
        let nested_ctrl_idx = outer_para.controls.len() - 1;
        let outer_table = Table {
            cells: vec![Cell {
                paragraphs: vec![outer_para],
                ..Default::default()
            }],
            ..Default::default()
        };
        core.document.sections[0].paragraphs[0]
            .controls
            .push(Control::Table(Box::new(outer_table)));
        let outer_ctrl_idx = core.document.sections[0].paragraphs[0].controls.len() - 1;
        let path = [(outer_ctrl_idx, 0, 0), (nested_ctrl_idx, 0, 0)];

        // INNER의 1..3(NN)만 지우고, 같은 컨테이너의 바깥 셀 OUTER는 보존해야 한다.
        core.delete_range_in_cell_by_path(0, 0, &path, 0, 1, 0, 3)
            .unwrap();

        let Control::Table(outer) =
            &core.document.sections[0].paragraphs[0].controls[outer_ctrl_idx]
        else {
            panic!("expected outer table");
        };
        assert_eq!(outer.cells[0].paragraphs[0].text, "OUTER");
        let Control::Table(inner) = &outer.cells[0].paragraphs[0].controls[nested_ctrl_idx] else {
            panic!("expected nested table");
        };
        assert_eq!(inner.cells[0].paragraphs[0].text, "IER");
    }

    /// [#2755] 셀 폭 200 HWPUNIT + 권위 `line_segs` 를 가진 1×1 표 문서를 만든다.
    ///
    /// `formatting.rs` 의 `cell_reflow_width_tests::core_with_narrow_cell` 과 동형이며,
    /// `line_seg_starts` 로 저장된 줄 경계를 직접 지정해 "실제 `.hwp`/`.hwpx` 에서 파싱한
    /// 셀 문단"(권위 `line_segs` 보유) 상태를 재현한다. 기존 `by_path` 테스트는
    /// `Paragraph::default()` 를 써 `line_segs` 가 비어 있었고, 그 경우 레이아웃의
    /// `recompose_for_cell_width` 가 폭 기준으로 재래핑해 주므로 결함이 관측되지 않았다.
    ///
    /// 페이지 본문 폭(수만 HWPUNIT)과 셀 폭(200 HWPUNIT)을 극단적으로 벌려, 어떤 폰트 폭
    /// 추정치를 쓰든 "페이지 폭 사용" 과 "셀 폭 사용" 이 줄 수로 갈리게 한다.
    fn core_with_narrow_cell_line_segs(text: &str, line_seg_starts: &[u32]) -> DocumentCore {
        use crate::model::control::Control;
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::page::PageDef;
        use crate::model::paragraph::LineSeg;
        use crate::model::table::{Cell, Table};

        let mut doc = Document::default();

        let mut cell_para = Paragraph {
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
            char_count: text.chars().count() as u32,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: line_seg_starts
                .iter()
                .map(|&text_start| LineSeg {
                    text_start,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        cell_para.has_para_text = true;

        let mut table = Table {
            row_count: 1,
            col_count: 1,
            ..Default::default()
        };
        table.cells = vec![Cell {
            row: 0,
            col: 0,
            col_span: 1,
            row_span: 1,
            width: 200, // 셀 폭 — 페이지 본문 폭(수만 HWPUNIT)의 1% 미만
            paragraphs: vec![cell_para],
            ..Default::default()
        }];

        let mut para = Paragraph::default();
        para.controls.push(Control::Table(Box::new(table)));

        let mut section = Section {
            section_def: SectionDef {
                page_def: PageDef {
                    width: 59528,
                    height: 84188,
                    margin_left: 8504,
                    margin_right: 8504,
                    margin_top: 5668,
                    margin_bottom: 4252,
                    margin_header: 4252,
                    margin_footer: 4252,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        section.paragraphs.push(para);
        doc.sections.push(section);

        let mut core = DocumentCore::new_empty();
        core.document = doc;
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];
        core
    }

    /// 첫 셀의 첫 문단을 꺼낸다.
    fn narrow_cell_para(core: &DocumentCore) -> &Paragraph {
        use crate::model::control::Control;
        let Control::Table(t) = &core.document.sections[0].paragraphs[0].controls[0] else {
            panic!("표 컨트롤이어야 함");
        };
        &t.cells[0].paragraphs[0]
    }

    /// [#2755] 항목 1 — `delete_range_in_cell_by_path` 가 깊이 1 셀에서 셀 폭 리플로우를
    /// 수행해야 한다.
    ///
    /// 형제 `delete_range_native` 의 셀 분기는 단일/다중 문단 양쪽에서
    /// `reflow_cell_paragraph` 를 호출한다. 리플로우가 없으면 저장된 줄 경계가 그대로 남아
    /// 셀이 계속 2줄로 조판되고(둘째 줄은 빈 줄) 행 높이도 줄지 않는다.
    #[test]
    fn delete_range_in_cell_by_path_reflows_depth1_cell_line_segs() {
        let text = "A".repeat(40);
        let mut core = core_with_narrow_cell_line_segs(&text, &[0, 20]);

        // 40자 중 39자를 지운다 — 남는 텍스트는 1자다.
        core.delete_range_in_cell_by_path(0, 0, &[(0, 0, 0)], 0, 0, 0, 39)
            .expect("범위 삭제가 성공해야 함");

        let para = narrow_cell_para(&core);
        assert_eq!(para.text.chars().count(), 1, "39자가 삭제돼야 함");
        let line_count = para.line_segs.len();
        assert_eq!(
            line_count, 1,
            "1자만 남았으므로 셀 폭 리플로우 후 1줄이어야 함 (실제 {line_count}줄 — \
             리플로우가 없으면 저장된 2줄 경계가 그대로 남는다)"
        );
    }

    /// [#2755] 항목 3 — `delete_text_in_cell_by_path` 도 같은 계약을 지켜야 한다.
    ///
    /// 삽입 쌍둥이 `insert_text_in_cell_by_path` 는 #2172 에서 깊이 1 위임 가드를 받았고,
    /// flat `delete_text_in_cell_native` 는 리플로우와 vpos 재계산을 모두 수행한다.
    #[test]
    fn delete_text_in_cell_by_path_reflows_depth1_cell_line_segs() {
        let text = "A".repeat(40);
        let mut core = core_with_narrow_cell_line_segs(&text, &[0, 20]);

        core.delete_text_in_cell_by_path(0, 0, &[(0, 0, 0)], 0, 39)
            .expect("텍스트 삭제가 성공해야 함");

        let para = narrow_cell_para(&core);
        assert_eq!(para.text.chars().count(), 1, "39자가 삭제돼야 함");
        let line_count = para.line_segs.len();
        assert_eq!(
            line_count, 1,
            "1자만 남았으므로 셀 폭 리플로우 후 1줄이어야 함 (실제 {line_count}줄)"
        );
    }

    /// [#2755] 항목 4 — 빈 `cellPath` 는 패닉이 아니라 `Err` 여야 한다.
    ///
    /// `parse_cell_path` 는 `"[]"` 에 대해 `Ok(Vec::new())` 를 반환하므로 빈 경로가 코어까지
    /// 도달할 수 있다. wasm 에서 Rust 패닉은 `HwpDocument` 인스턴스 전체를 무효화한다.
    /// 형제 `get_cell_paragraphs_mut_by_path` / `get_cell_paragraph_mut_by_path` 는 이미
    /// 빈 경로를 `Err` 로 거절한다.
    #[test]
    fn cell_paragraph_ops_by_path_reject_empty_path_with_error() {
        let mut core = core_with_narrow_cell_line_segs("AB", &[0]);

        assert!(
            core.split_paragraph_in_cell_by_path(0, 0, &[], 1).is_err(),
            "빈 경로 분할은 Err 여야 한다"
        );
        assert!(
            core.merge_paragraph_in_cell_by_path(0, 0, &[]).is_err(),
            "빈 경로 병합은 Err 여야 한다"
        );
    }

    /// [#2755] 깊이 2 중첩 표: 바깥 표(1셀, 폭 5000) 문단 안에 안쪽 표(1셀, 폭 200 + 권위
    /// line_segs)를 두고, path = [(outer,0,0),(inner,0,0)] 를 함께 돌려준다.
    ///
    /// 깊이 ≥ 2 에서 `reflow_cell_paragraph`(flat 좌표)로는 최내곽 셀을 재래핑할 수 없었다.
    /// `reflow_cell_paragraph_by_path` 가 최내곽 셀 폭(200)을 해석해 재래핑하는지 검증한다.
    fn core_with_nested_narrow_cell(
        text: &str,
        line_seg_starts: &[u32],
    ) -> (DocumentCore, Vec<(usize, usize, usize)>) {
        use crate::model::control::Control;
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::page::PageDef;
        use crate::model::paragraph::LineSeg;
        use crate::model::table::{Cell, Table};

        let mut inner_para = Paragraph {
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
            char_count: text.chars().count() as u32,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: line_seg_starts
                .iter()
                .map(|&text_start| LineSeg {
                    text_start,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        inner_para.has_para_text = true;

        let inner_table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                row: 0,
                col: 0,
                col_span: 1,
                row_span: 1,
                width: 200, // 최내곽 셀 폭
                paragraphs: vec![inner_para],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut outer_cell_para = Paragraph::default();
        outer_cell_para
            .controls
            .push(Control::Table(Box::new(inner_table)));
        let inner_ctrl_idx = outer_cell_para.controls.len() - 1;

        let outer_table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                row: 0,
                col: 0,
                col_span: 1,
                row_span: 1,
                width: 5000, // 바깥 셀은 넉넉히 — 안쪽 셀 폭이 실제 리플로우 기준임을 분리
                paragraphs: vec![outer_cell_para],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut body_para = Paragraph::default();
        body_para
            .controls
            .push(Control::Table(Box::new(outer_table)));
        let outer_ctrl_idx = body_para.controls.len() - 1;

        let mut section = Section {
            section_def: SectionDef {
                page_def: PageDef {
                    width: 59528,
                    height: 84188,
                    margin_left: 8504,
                    margin_right: 8504,
                    margin_top: 5668,
                    margin_bottom: 4252,
                    margin_header: 4252,
                    margin_footer: 4252,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        section.paragraphs.push(body_para);
        let mut doc = Document::default();
        doc.sections.push(section);

        let mut core = DocumentCore::new_empty();
        core.document = doc;
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];
        let path = vec![(outer_ctrl_idx, 0, 0), (inner_ctrl_idx, 0, 0)];
        (core, path)
    }

    /// [#2755] 깊이 2 — `delete_range_in_cell_by_path` 가 최내곽 셀을 재래핑한다.
    #[test]
    fn delete_range_in_nested_cell_by_path_reflows_inner_cell() {
        let text = "A".repeat(40);
        let (mut core, path) = core_with_nested_narrow_cell(&text, &[0, 20]);

        core.delete_range_in_cell_by_path(0, 0, &path, 0, 0, 0, 39)
            .expect("범위 삭제가 성공해야 함");

        let paras = core.get_cell_paragraphs_mut_by_path(0, 0, &path).unwrap();
        assert_eq!(paras[0].text.chars().count(), 1, "39자가 삭제돼야 함");
        assert_eq!(
            paras[0].line_segs.len(),
            1,
            "깊이 2 안쪽 셀도 1자만 남으면 1줄로 재래핑돼야 함"
        );
    }

    /// [#2755] 깊이 2 — `delete_text_in_cell_by_path` 가 최내곽 셀을 재래핑한다.
    #[test]
    fn delete_text_in_nested_cell_by_path_reflows_inner_cell() {
        let text = "A".repeat(40);
        let (mut core, path) = core_with_nested_narrow_cell(&text, &[0, 20]);

        core.delete_text_in_cell_by_path(0, 0, &path, 0, 39)
            .expect("텍스트 삭제가 성공해야 함");

        let paras = core.get_cell_paragraphs_mut_by_path(0, 0, &path).unwrap();
        assert_eq!(paras[0].text.chars().count(), 1, "39자가 삭제돼야 함");
        assert_eq!(
            paras[0].line_segs.len(),
            1,
            "깊이 2 안쪽 셀도 재래핑돼야 함"
        );
    }

    /// [#2755] 깊이 2 — `split_paragraph_in_cell_by_path` 가 분할된 두 문단을 재래핑한다.
    #[test]
    fn split_paragraph_in_nested_cell_by_path_reflows_inner_cell() {
        let text = "A".repeat(40);
        let (mut core, path) = core_with_nested_narrow_cell(&text, &[0, 20]);

        // 20 지점에서 분할 → 앞뒤 각각 20자. 폭 200 재래핑이면 각 문단이 여러 줄로 나뉜다.
        core.split_paragraph_in_cell_by_path(0, 0, &path, 20)
            .expect("문단 분할이 성공해야 함");

        let paras = core.get_cell_paragraphs_mut_by_path(0, 0, &path).unwrap();
        assert_eq!(paras.len(), 2, "안쪽 셀 문단이 2개로 분할돼야 함");
        assert!(
            paras[0].line_segs.len() > 1,
            "앞 문단(20자)이 좁은 셀 폭으로 재래핑되면 여러 줄이어야 함 (실제 {}줄 — \
             재래핑이 없으면 split_at 이 남긴 1줄)",
            paras[0].line_segs.len()
        );
        assert!(
            paras[1].line_segs.len() > 1,
            "뒤 문단(20자)도 재래핑돼야 함 (실제 {}줄)",
            paras[1].line_segs.len()
        );
    }

    /// [#2755] 깊이 2 — `merge_paragraph_in_cell_by_path` 가 병합 생존 문단을 재래핑한다.
    #[test]
    fn merge_paragraph_in_nested_cell_by_path_reflows_inner_cell() {
        // 안쪽 셀에 짧은 문단 2개를 두고 병합하면 40자가 합쳐져 좁은 폭에서 여러 줄이 된다.
        let (mut core, path) = core_with_nested_narrow_cell(&"A".repeat(20), &[0]);
        {
            let paras = core.get_cell_paragraphs_mut_by_path(0, 0, &path).unwrap();
            let mut second = Paragraph {
                text: "B".repeat(20),
                char_offsets: (0..20).collect(),
                char_count: 20,
                char_shapes: vec![CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                }],
                line_segs: vec![crate::model::paragraph::LineSeg {
                    text_start: 0,
                    ..Default::default()
                }],
                ..Default::default()
            };
            second.has_para_text = true;
            paras.push(second);
        }

        // 두 번째 문단(index 1)을 첫 번째에 병합.
        let merge_path = vec![path[0], (path[1].0, path[1].1, 1)];
        core.merge_paragraph_in_cell_by_path(0, 0, &merge_path)
            .expect("문단 병합이 성공해야 함");

        let paras = core.get_cell_paragraphs_mut_by_path(0, 0, &path).unwrap();
        assert_eq!(paras.len(), 1, "병합 후 문단은 1개여야 함");
        assert_eq!(paras[0].text.chars().count(), 40, "40자가 합쳐져야 함");
        assert!(
            paras[0].line_segs.len() > 1,
            "병합 문단(40자)이 좁은 셀 폭으로 재래핑되면 여러 줄이어야 함 (실제 {}줄)",
            paras[0].line_segs.len()
        );
    }

    #[test]
    fn insert_paragraph_inherits_shape_from_previous_paragraph() {
        let mut core = core_with_two_shaped_paragraphs();

        // 중간 삽입: 앞 문단(idx 0)에서 상속
        core.insert_paragraph_native(0, 1).unwrap();
        assert_eq!(
            shape_of(&core, 1),
            (12, 3, Some(7)),
            "중간 삽입은 앞 문단 상속"
        );
        assert_eq!(shape_of(&core, 2), (14, 5, Some(9)), "밀려난 문단은 불변");
    }

    #[test]
    fn insert_paragraph_at_zero_inherits_from_following_paragraph() {
        let mut core = core_with_two_shaped_paragraphs();

        // para_idx == 0: 앞 문단이 없으므로 뒤로 밀려날 현재 0번을 상속원으로 쓴다
        core.insert_paragraph_native(0, 0).unwrap();
        assert_eq!(
            shape_of(&core, 0),
            (12, 3, Some(7)),
            "0번 삽입은 뒤 문단 상속"
        );
    }

    #[test]
    fn insert_paragraph_at_end_inherits_from_last_paragraph() {
        let mut core = core_with_two_shaped_paragraphs();
        let para_count = core.document.sections[0].paragraphs.len();

        // para_idx == len: 맨 끝에 덧붙이기, 마지막 문단에서 상속
        core.insert_paragraph_native(0, para_count).unwrap();
        assert_eq!(
            shape_of(&core, para_count),
            (14, 5, Some(9)),
            "끝 삽입은 마지막 문단 상속"
        );
    }

    /// 상속원이 존재하지 않는 유일한 경우 — new_empty() 로 후퇴한다.
    #[test]
    fn new_empty_like_without_char_shapes_leaves_char_shapes_empty() {
        let mut template = Paragraph::new_empty();
        template.para_shape_id = 12;
        template.style_id = 3;
        assert!(template.char_shapes.is_empty());

        let para = Paragraph::new_empty_like(&template);
        assert_eq!(para.para_shape_id, 12);
        assert_eq!(para.style_id, 3);
        assert!(
            para.char_shapes.is_empty(),
            "템플릿에 글자모양이 없으면 빈 채로 둔다"
        );
    }

    /// new_empty_like 는 템플릿 문단 *끝* 글자모양만, start_pos 를 0 으로
    /// 정규화해 가져온다 — 새 문단은 템플릿 뒤에 이어지므로(문단 끝 Enter)
    /// 혼합 글자모양 문단에서 첫 엔트리(7)가 아니라 끝 엔트리(9)가 기준이다.
    #[test]
    fn new_empty_like_takes_last_char_shape_at_pos_zero() {
        let mut template = Paragraph::new_empty();
        template.text = "가나다".to_string();
        template.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 7,
            },
            CharShapeRef {
                start_pos: 2,
                char_shape_id: 9,
            },
        ];

        let para = Paragraph::new_empty_like(&template);
        assert_eq!(para.char_shapes.len(), 1, "끝 글자모양만 상속");
        assert_eq!(para.char_shapes[0].start_pos, 0);
        assert_eq!(para.char_shapes[0].char_shape_id, 9);
        assert!(para.text.is_empty(), "텍스트는 상속하지 않는다");
    }

    #[test]
    fn test_page_overflow_with_enter() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        assert_eq!(core.page_count(), 1, "초기 페이지 수");
        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            1,
            "초기 문단 수"
        );

        // Enter를 500번 입력하여 페이지 오버플로우 유발
        for i in 0..500 {
            let para_count = core.document.sections[0].paragraphs.len();
            core.split_paragraph_native(0, para_count - 1, 0).unwrap();
        }

        let para_count = core.document.sections[0].paragraphs.len();
        let page_count = core.page_count();
        assert_eq!(para_count, 501, "문단 수");
        assert!(
            page_count >= 2,
            "페이지 수: {} (2 이상이어야 함)",
            page_count
        );
    }

    #[test]
    fn test_paragraph_y_positions_after_split() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 첫 문단에 긴 텍스트 입력 (줄바꿈 발생)
        let long_text = "The quick brown fox jumps over the lazy dog. ";
        let text = long_text.repeat(5);
        core.insert_text_native(0, 0, 0, &text).unwrap();

        // 첫 문단이 여러 줄로 구성되는지 확인
        let para0_lines = core.composed[0][0].lines.len();
        eprintln!("문단0 줄 수: {}", para0_lines);
        assert!(
            para0_lines >= 2,
            "첫 문단은 2줄 이상이어야 함: {}",
            para0_lines
        );

        // Enter로 문단 분리 (텍스트 끝에서)
        let text_len = core.document.sections[0].paragraphs[0].text.chars().count();
        core.split_paragraph_native(0, 0, text_len).unwrap();

        // 두 번째 문단에 텍스트 입력
        core.insert_text_native(0, 1, 0, "Second paragraph")
            .unwrap();

        // 렌더 트리 빌드 (페이지 0)
        let tree = core.build_page_tree(0).unwrap();
        let tree_str = format!("{:?}", tree);

        // 렌더 트리에서 문단들의 Y 좌표를 추출
        // 두 번째 문단 "Second" 텍스트가 존재하는지 확인
        assert!(
            tree_str.contains("Second paragraph"),
            "두 번째 문단 텍스트가 렌더 트리에 없음"
        );

        // 렌더 트리에서 TextRun Y 좌표 확인
        let para0_last_y = find_text_y(&tree.root, "dog.");
        let para1_y = find_text_y(&tree.root, "Second");
        eprintln!(
            "문단0 마지막줄 Y: {:?}, 문단1 Y: {:?}",
            para0_last_y, para1_y
        );

        if let (Some(y0), Some(y1)) = (para0_last_y, para1_y) {
            assert!(
                y1 > y0,
                "문단1 Y({:.1})가 문단0 Y({:.1})보다 커야 함 (겹침 감지)",
                y1,
                y0
            );
        }
    }

    /// 줄간격 160%(기본값)에서 페이지 넘김 확인
    #[test]
    fn test_page_break_with_default_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 텍스트를 넣고 Enter로 문단 분리 반복 → 페이지 넘김 검증
        let text = "Line spacing 160 percent default.";
        for i in 0..100 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }

        let page_count = core.page_count();
        eprintln!("160% 줄간격: 문단 101개, 페이지 수: {}", page_count);
        assert!(
            page_count >= 2,
            "160% 줄간격에서 페이지 넘김 필요: {}",
            page_count
        );
    }

    /// 줄간격 100%에서 200%보다 더 많은 문단이 한 페이지에 들어가는지 확인
    /// (비교 대상이 160%면 height_for_fit 모델의 trail_ls 절약 효과로 1페이지 역전 가능 → 200% 사용)
    #[test]
    fn test_page_break_with_tight_line_spacing() {
        // 100% 줄간격 문서
        let mut core100 = DocumentCore::new_empty();
        core100.create_blank_document_native().unwrap();
        let text = "Tight spacing test line.";
        // 첫 문단에 줄간격 100% 적용
        core100
            .apply_para_format_native(0, 0, r#"{"lineSpacing":100}"#)
            .unwrap();
        for i in 0..500 {
            let para_count = core100.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core100.insert_text_native(0, last, 0, text).unwrap();
            core100.split_paragraph_native(0, last, text.len()).unwrap();
            // 새 문단에도 100% 적용
            let new_last = core100.document.sections[0].paragraphs.len() - 1;
            core100
                .apply_para_format_native(0, new_last, r#"{"lineSpacing":100}"#)
                .unwrap();
        }
        let pages_100 = core100.page_count();

        // 200% 줄간격 문서 (비교 기준)
        let mut core200 = DocumentCore::new_empty();
        core200.create_blank_document_native().unwrap();
        core200
            .apply_para_format_native(0, 0, r#"{"lineSpacing":200}"#)
            .unwrap();
        for i in 0..500 {
            let para_count = core200.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core200.insert_text_native(0, last, 0, text).unwrap();
            core200.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core200.document.sections[0].paragraphs.len() - 1;
            core200
                .apply_para_format_native(0, new_last, r#"{"lineSpacing":200}"#)
                .unwrap();
        }
        let pages_200 = core200.page_count();

        eprintln!(
            "100% → {}페이지, 200% → {}페이지 (문단 501개)",
            pages_100, pages_200
        );
        // 100%는 200%보다 같거나 적은 페이지 수
        assert!(
            pages_100 <= pages_200,
            "100% 줄간격({})이 200%({})보다 적은/같은 페이지 수여야 함",
            pages_100,
            pages_200
        );
    }

    /// 줄간격 300%에서 160%보다 더 빨리 페이지가 넘어가는지 확인
    #[test]
    fn test_page_break_with_wide_line_spacing() {
        // 300% 줄간격
        let mut core300 = DocumentCore::new_empty();
        core300.create_blank_document_native().unwrap();
        let text = "Wide spacing test line.";
        core300
            .apply_para_format_native(0, 0, r#"{"lineSpacing":300}"#)
            .unwrap();
        for i in 0..30 {
            let para_count = core300.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core300.insert_text_native(0, last, 0, text).unwrap();
            core300.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core300.document.sections[0].paragraphs.len() - 1;
            core300
                .apply_para_format_native(0, new_last, r#"{"lineSpacing":300}"#)
                .unwrap();
        }
        let pages_300 = core300.page_count();

        // 160% 줄간격 (동일 문단 수)
        let mut core160 = DocumentCore::new_empty();
        core160.create_blank_document_native().unwrap();
        for i in 0..30 {
            let para_count = core160.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core160.insert_text_native(0, last, 0, text).unwrap();
            core160.split_paragraph_native(0, last, text.len()).unwrap();
        }
        let pages_160 = core160.page_count();

        eprintln!(
            "300% → {}페이지, 160% → {}페이지 (문단 31개)",
            pages_300, pages_160
        );
        assert!(
            pages_300 >= pages_160,
            "300% 줄간격({})이 160%({})보다 많은/같은 페이지 수여야 함",
            pages_300,
            pages_160
        );
    }

    /// 혼합 줄간격: 문단마다 다른 줄간격에서 페이지 넘김 정상 동작 확인
    #[test]
    fn test_page_break_with_mixed_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let spacings = [160, 100, 300, 250, 120, 200];
        let text = "Mixed spacing paragraph content here.";

        for i in 0..120 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            // 현재 문단에 다양한 줄간격 적용
            let spacing = spacings[i % spacings.len()];
            let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
            core.apply_para_format_native(0, last, &json).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }

        let page_count = core.page_count();
        let para_count = core.document.sections[0].paragraphs.len();
        eprintln!(
            "혼합 줄간격: 문단 {}개, 페이지 수: {}",
            para_count, page_count
        );
        assert!(
            page_count >= 2,
            "혼합 줄간격에서 페이지 넘김 필요: {}",
            page_count
        );

        // 각 페이지에 문단이 배치되었는지 확인 (렌더 트리 빌드 가능)
        for p in 0..page_count {
            let tree = core.build_page_tree(p as u32);
            assert!(
                tree.is_ok(),
                "페이지 {} 렌더 트리 빌드 실패: {:?}",
                p,
                tree.err()
            );
        }
    }

    /// 고정(Fixed) 줄간격에서 페이지 넘김 정상 동작 확인
    #[test]
    fn test_page_break_with_fixed_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let text = "Fixed spacing paragraph.";
        // Fixed 줄간격 30px
        core.apply_para_format_native(0, 0, r#"{"lineSpacing":30,"lineSpacingType":"Fixed"}"#)
            .unwrap();

        for i in 0..50 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core.document.sections[0].paragraphs.len() - 1;
            core.apply_para_format_native(
                0,
                new_last,
                r#"{"lineSpacing":30,"lineSpacingType":"Fixed"}"#,
            )
            .unwrap();
        }

        let page_count = core.page_count();
        eprintln!("Fixed 줄간격: 문단 51개, 페이지 수: {}", page_count);
        assert!(
            page_count >= 1,
            "Fixed 줄간격에서 페이지 수 확인: {}",
            page_count
        );

        // 렌더 트리 정상 빌드 확인
        for p in 0..page_count {
            let tree = core.build_page_tree(p as u32);
            assert!(tree.is_ok(), "페이지 {} 렌더 트리 빌드 실패", p);
        }
    }

    /// 각 줄간격별 페이지당 수용 줄 수가 논리적으로 맞는지 확인
    #[test]
    fn test_line_count_per_page_varies_by_spacing() {
        let spacings = vec![100, 160, 250, 300];
        let mut page_counts = Vec::new();

        for spacing in &spacings {
            let mut core = DocumentCore::new_empty();
            core.create_blank_document_native().unwrap();
            let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
            core.apply_para_format_native(0, 0, &json).unwrap();

            let text = "Test line for spacing comparison.";
            for _ in 0..60 {
                let last = core.document.sections[0].paragraphs.len() - 1;
                core.insert_text_native(0, last, 0, text).unwrap();
                core.split_paragraph_native(0, last, text.len()).unwrap();
                let new_last = core.document.sections[0].paragraphs.len() - 1;
                core.apply_para_format_native(0, new_last, &json).unwrap();
            }
            page_counts.push((*spacing, core.page_count()));
        }

        eprintln!("줄간격별 페이지 수 (문단 61개):");
        for (spacing, pages) in &page_counts {
            eprintln!("  {}% → {}페이지", spacing, pages);
        }

        // 줄간격이 클수록 페이지 수가 많아야 함
        for i in 1..page_counts.len() {
            assert!(
                page_counts[i].1 >= page_counts[i - 1].1,
                "줄간격 {}%({})가 {}%({})보다 적은 페이지 수",
                page_counts[i].0,
                page_counts[i].1,
                page_counts[i - 1].0,
                page_counts[i - 1].1
            );
        }
    }

    /// 기존 문서 중간 문단의 줄간격을 10%씩 증가시키면 페이지 경계를 정확히 돌파하는지 검증
    #[test]
    fn test_page_boundary_with_incremental_spacing_increase() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 160% 줄간격으로 30개의 multi-line 문단 생성 (1페이지에 거의 맞도록)
        // height_for_fit 모델에서 trailing line_spacing은 제외되므로,
        // single-line 문단으로는 spacing 증가 효과가 약화됨 → multi-line text 사용
        let text = "Test paragraph for spacing. ".repeat(20);
        let text = text.as_str();
        for _ in 0..29 {
            let last = core.document.sections[0].paragraphs.len() - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }
        // 마지막 문단에도 텍스트
        let last = core.document.sections[0].paragraphs.len() - 1;
        core.insert_text_native(0, last, 0, text).unwrap();

        let initial_pages = core.page_count();
        eprintln!(
            "초기 페이지 수: {} (30 multi-line 문단 160%)",
            initial_pages
        );

        // 문단 15~25의 줄간격을 10%씩 증가 (170%, 180%, ..., 270%)
        let mut prev_pages = initial_pages;
        let mut boundary_crossed_at = 0;
        for step in 0..20 {
            let spacing = 170 + step * 10; // 170% → 360%
            for para_idx in 5..30 {
                if para_idx < core.document.sections[0].paragraphs.len() {
                    let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
                    core.apply_para_format_native(0, para_idx, &json).unwrap();
                }
            }
            let pages = core.page_count();
            if pages > prev_pages && boundary_crossed_at == 0 {
                boundary_crossed_at = spacing;
                eprintln!(
                    "  페이지 경계 돌파: {}% 줄간격에서 {}→{}페이지",
                    spacing, prev_pages, pages
                );
            }
            prev_pages = pages;
        }

        eprintln!("최종 페이지 수: {} (줄간격 360%)", prev_pages);
        assert!(
            prev_pages > initial_pages,
            "줄간격 증가로 페이지 수 증가 필요: {} → {}",
            initial_pages,
            prev_pages
        );
        assert!(
            boundary_crossed_at > 0,
            "페이지 경계 돌파 시점이 감지되어야 함"
        );

        // 모든 페이지 렌더 트리 정상 빌드 확인
        for p in 0..prev_pages {
            let tree = core.build_page_tree(p as u32);
            assert!(
                tree.is_ok(),
                "페이지 {} 렌더 트리 빌드 실패: {:?}",
                p,
                tree.err()
            );
        }
    }
}

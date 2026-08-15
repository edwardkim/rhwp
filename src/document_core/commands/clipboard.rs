//! 내부 클립보드 + HTML 내보내기 관련 native 메서드

use super::super::helpers::{
    border_line_type_to_u8_val, clipboard_color_to_css, clipboard_escape_html, color_ref_to_css,
    detect_clipboard_image_mime, get_textbox_from_shape, get_textbox_from_shape_mut,
    utf16_pos_to_char_idx,
};
use super::super::queries::field_query::rebuild_char_offsets;
use crate::document_core::{ClipboardData, DocumentCore};
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::{FieldRange, LineSeg, Paragraph};

/// [Task #1161] 떠 있는 개체 반복 붙여넣기 cascade 1 회당 위치 오프셋(HWPUNIT).
/// 약 2mm (1mm = 7200/25.4 ≈ 283.46 HWPUNIT). 한컴 정합은 작업지시자 시각 대조로 미세조정.
const PASTE_CASCADE_STEP_HU: u32 = 567;

/// [#4413] 셀 안 컨트롤을 HTML로 변환하는 재귀(표 안의 표 안의 표…)의 깊이 상한.
/// `table_extract::MAX_NEST_DEPTH`/`explain::MAX_NEST_DEPTH`/`hidden_text::MAX_NEST_DEPTH`와
/// 같은 값·형태 — 병적으로 깊은 중첩 문서에서 export 재귀가 스택을 태우지 않게 막는다.
const MAX_NEST_DEPTH: usize = 8;

/// 셀 HTML 내보내기에서 지원하지 않는 컨트롤을 경고 주석에 남기기 위한 표시 이름.
/// `Control::Table`/`Control::Picture`는 `control_to_html`이 직접 처리하므로 이 경로를
/// 타지 않지만, 매치 순서가 바뀌어도 무해한 이름을 반환하도록 모든 변형을 다룬다.
fn control_kind_label(control: &Control) -> &'static str {
    match control {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Table(_) => "Table",
        Control::Shape(_) => "Shape",
        Control::Picture(_) => "Picture",
        Control::Header(_) => "Header",
        Control::Footer(_) => "Footer",
        Control::Footnote(_) => "Footnote",
        Control::Endnote(_) => "Endnote",
        Control::AutoNumber(_) => "AutoNumber",
        Control::NewNumber(_) => "NewNumber",
        Control::PageNumberPos(_) => "PageNumberPos",
        Control::Bookmark(_) => "Bookmark",
        Control::IndexMark(_) => "IndexMark",
        Control::PageNumCtrl(_) => "PageNumCtrl",
        Control::Hyperlink(_) => "Hyperlink",
        Control::Ruby(_) => "Ruby",
        Control::CharOverlap(_) => "CharOverlap",
        Control::PageHide(_) => "PageHide",
        Control::HiddenComment(_) => "HiddenComment",
        Control::Equation(_) => "Equation",
        Control::Field(_) => "Field",
        Control::Form(_) => "Form",
        Control::Unknown(_) => "Unknown",
    }
}

/// [#2550] 압축 해제 상한 초과 항목(deflate bomb 포함)에 대한 공통 오류.
///
/// 범위 초과(`범위 초과`)와 같은 `RenderError` 계열이라 호출부 처리 경로가 같다.
fn bin_data_over_limit_error(bin_data_id: u16) -> HwpError {
    HwpError::RenderError(format!(
        "바이너리 데이터 {} 압축 해제 상한 {}MB 초과",
        bin_data_id,
        crate::model::bin_data::MAX_BIN_DATA_BYTES / (1024 * 1024)
    ))
}

fn clipboard_paragraphs_contain_field(paragraphs: &[Paragraph]) -> bool {
    paragraphs.iter().any(|para| !para.field_ranges.is_empty())
}

fn clipboard_control_char_code(ctrl: &Control) -> u16 {
    match ctrl {
        Control::SectionDef(_) | Control::ColumnDef(_) => 0x0002,
        Control::Field(_) => 0x0003,
        Control::Table(_)
        | Control::Shape(_)
        | Control::Picture(_)
        | Control::Hyperlink(_)
        | Control::Ruby(_)
        | Control::Equation(_)
        | Control::Form(_)
        | Control::Unknown(_) => 0x000B,
        Control::HiddenComment(_) => 0x000F,
        Control::Header(_) | Control::Footer(_) => 0x0010,
        Control::Footnote(_) | Control::Endnote(_) => 0x0011,
        Control::AutoNumber(_) | Control::NewNumber(_) => 0x0012,
        Control::PageNumberPos(_) | Control::PageHide(_) => 0x0015,
        Control::Bookmark(_) | Control::IndexMark(_) => 0x0016,
        Control::PageNumCtrl(_) => 0x0015,
        Control::CharOverlap(_) => 0x0017,
    }
}

fn recompute_clipboard_control_mask(para: &Paragraph) -> u32 {
    let mut mask = 0u32;
    for ctrl in &para.controls {
        mask |= 1u32 << clipboard_control_char_code(ctrl);
    }
    if !para.field_ranges.is_empty() {
        mask |= 1u32 << 0x0004;
    }
    if para.text.contains('\t') {
        mask |= 1u32 << 0x0009;
    }
    if para.text.contains('\n') {
        mask |= 1u32 << 0x000A;
    }
    mask
}

fn strip_structural_controls_for_text_clipboard(para: &mut Paragraph) {
    // [#4149] clip 사본이지만 다중 문단 붙여넣기에서 중간 문단이 통째로 문서에
    // 스플라이스되어 렌더 입력이 될 수 있다 — 컨트롤 제거로 compose 입력이
    // 바뀌므로 단일줄 과밀 memo 를 무효화한다.
    para.invalidate_single_line_overflow_memo();
    let old_controls = std::mem::take(&mut para.controls);
    let old_records = std::mem::take(&mut para.ctrl_data_records);
    let mut index_map = vec![None; old_controls.len()];
    let mut new_controls = Vec::new();
    let mut new_records = Vec::new();

    for (old_idx, ctrl) in old_controls.into_iter().enumerate() {
        if matches!(ctrl, Control::SectionDef(_) | Control::ColumnDef(_)) {
            continue;
        }
        index_map[old_idx] = Some(new_controls.len());
        new_records.push(old_records.get(old_idx).cloned().flatten());
        new_controls.push(ctrl);
    }

    para.field_ranges = para
        .field_ranges
        .drain(..)
        .filter_map(|mut fr| {
            let new_idx = index_map.get(fr.control_idx).and_then(|idx| *idx)?;
            fr.control_idx = new_idx;
            Some(fr)
        })
        .collect();
    para.controls = new_controls;
    para.ctrl_data_records = new_records;
    para.control_mask = recompute_clipboard_control_mask(para);
    if !para.field_ranges.is_empty() {
        rebuild_char_offsets(para);
    }
}

fn text_to_split_logical_offset(para: &Paragraph, text_offset: usize) -> usize {
    let control_positions = para.control_text_positions();
    if control_positions.is_empty() {
        return text_offset;
    }

    // 클립보드 range trimming 은 곧바로 Paragraph::split_at()을 호출한다.
    // 따라서 커서 탐색용 논리 offset이 아니라 split_at()과 같은 movable control
    // 기준으로 변환해야 non-TAC 그림이 텍스트 앞에 있어도 첫 글자를 잃지 않는다.
    let before_count = para
        .controls
        .iter()
        .enumerate()
        .filter(|(_, ctrl)| Paragraph::is_split_movable_control(ctrl))
        .filter_map(|(ci, _)| control_positions.get(ci))
        .filter(|&&pos| pos < text_offset)
        .count();
    text_offset + before_count
}

fn clip_paragraph_text_range_for_clipboard(
    source: &Paragraph,
    start_char_offset: usize,
    end_char_offset: usize,
) -> Paragraph {
    let text_len = source.text.chars().count();
    let start = start_char_offset.min(text_len);
    let end = end_char_offset.min(text_len).max(start);

    let mut clipped = source.clone();
    if end < text_len {
        let end_logical = text_to_split_logical_offset(&clipped, end);
        let _ = clipped.split_at(end_logical);
    }
    if start == 0 {
        return clipped;
    }

    let control_positions = clipped.control_text_positions();
    let old_controls = clipped.controls.clone();
    let old_records = clipped.ctrl_data_records.clone();
    let old_ranges = clipped.field_ranges.clone();

    let start_logical = text_to_split_logical_offset(&clipped, start);
    let mut suffix = clipped.split_at(start_logical);
    let mut keep_control = vec![false; old_controls.len()];

    for range in &old_ranges {
        if range.start_char_idx >= start
            && range.end_char_idx <= end
            && range.control_idx < keep_control.len()
        {
            keep_control[range.control_idx] = true;
        }
    }

    for (idx, ctrl) in old_controls.iter().enumerate() {
        if matches!(
            ctrl,
            Control::SectionDef(_) | Control::ColumnDef(_) | Control::Field(_)
        ) {
            continue;
        }
        let pos = control_positions.get(idx).copied().unwrap_or(text_len);
        if pos >= start && pos <= end {
            keep_control[idx] = true;
        }
    }

    let mut index_map = vec![None; old_controls.len()];
    let mut new_controls = Vec::new();
    let mut new_records = Vec::new();
    for (old_idx, ctrl) in old_controls.into_iter().enumerate() {
        if !keep_control.get(old_idx).copied().unwrap_or(false) {
            continue;
        }
        index_map[old_idx] = Some(new_controls.len());
        new_records.push(old_records.get(old_idx).cloned().flatten());
        new_controls.push(ctrl);
    }

    let new_field_ranges: Vec<FieldRange> = old_ranges
        .into_iter()
        .filter_map(|mut range| {
            if range.start_char_idx < start || range.end_char_idx > end {
                return None;
            }
            let new_control_idx = index_map.get(range.control_idx).and_then(|idx| *idx)?;
            range.start_char_idx -= start;
            range.end_char_idx -= start;
            range.control_idx = new_control_idx;
            Some(range)
        })
        .collect();

    suffix.controls = new_controls;
    suffix.ctrl_data_records = new_records;
    suffix.field_ranges = new_field_ranges;
    suffix.control_mask = recompute_clipboard_control_mask(&suffix);
    if !suffix.field_ranges.is_empty() {
        rebuild_char_offsets(&mut suffix);
    }
    suffix
}

fn collect_max_clipboard_field_id(para: &Paragraph, max_id: &mut u32) {
    for ctrl in &para.controls {
        match ctrl {
            Control::Field(field) => {
                *max_id = (*max_id).max(field.field_id);
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    for cell_para in &cell.paragraphs {
                        collect_max_clipboard_field_id(cell_para, max_id);
                    }
                }
                if let Some(caption) = &table.caption {
                    for cap_para in &caption.paragraphs {
                        collect_max_clipboard_field_id(cap_para, max_id);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(text_box) = get_textbox_from_shape(shape) {
                    for tb_para in &text_box.paragraphs {
                        collect_max_clipboard_field_id(tb_para, max_id);
                    }
                }
            }
            Control::Picture(pic) => {
                if let Some(caption) = &pic.caption {
                    for cap_para in &caption.paragraphs {
                        collect_max_clipboard_field_id(cap_para, max_id);
                    }
                }
            }
            _ => {}
        }
    }
}

fn assign_new_clipboard_field_ids(para: &mut Paragraph, next_id: &mut u32) {
    for ctrl in &mut para.controls {
        match ctrl {
            Control::Field(field) => {
                field.field_id = (*next_id).max(1);
                *next_id = next_id.saturating_add(1).max(1);
            }
            Control::Table(table) => {
                for cell in &mut table.cells {
                    for cell_para in &mut cell.paragraphs {
                        assign_new_clipboard_field_ids(cell_para, next_id);
                    }
                }
                if let Some(caption) = &mut table.caption {
                    for cap_para in &mut caption.paragraphs {
                        assign_new_clipboard_field_ids(cap_para, next_id);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(text_box) = get_textbox_from_shape_mut(shape) {
                    for tb_para in &mut text_box.paragraphs {
                        assign_new_clipboard_field_ids(tb_para, next_id);
                    }
                }
            }
            Control::Picture(pic) => {
                if let Some(caption) = &mut pic.caption {
                    for cap_para in &mut caption.paragraphs {
                        assign_new_clipboard_field_ids(cap_para, next_id);
                    }
                }
            }
            _ => {}
        }
    }
}

impl DocumentCore {
    pub fn has_internal_clipboard_native(&self) -> bool {
        self.clipboard.is_some()
    }

    /// 내부 클립보드의 플레인 텍스트를 반환한다.
    pub fn get_clipboard_text_native(&self) -> String {
        self.clipboard
            .as_ref()
            .map(|c| c.plain_text.clone())
            .unwrap_or_default()
    }

    /// 내부 클립보드를 초기화한다.
    pub fn clear_clipboard_native(&mut self) {
        self.clipboard = None;
    }

    fn renumber_pasted_field_ids(&self, clip_paras: &mut [Paragraph]) {
        let mut max_id = 0u32;
        for section in &self.document.sections {
            for para in &section.paragraphs {
                collect_max_clipboard_field_id(para, &mut max_id);
            }
        }
        let mut next_id = max_id.saturating_add(1).max(1);
        for para in clip_paras {
            assign_new_clipboard_field_ids(para, &mut next_id);
        }
    }

    /// 선택 영역을 내부 클립보드에 복사한다.
    ///
    /// 같은 구역 내 start ~ end 범위의 문단을 클립보드에 저장.
    /// 반환값: JSON `{"ok":true,"text":"<plain_text>"}`
    pub fn copy_selection_native(
        &mut self,
        section_idx: usize,
        start_para_idx: usize,
        start_char_offset: usize,
        end_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        // 인덱스 범위 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &self.document.sections[section_idx];
        if start_para_idx >= section.paragraphs.len() || end_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 범위 초과 (start={}, end={}, total={})",
                start_para_idx,
                end_para_idx,
                section.paragraphs.len()
            )));
        }
        if start_para_idx > end_para_idx {
            return Err(HwpError::RenderError(
                "시작 위치가 끝 위치보다 뒤에 있음".to_string(),
            ));
        }

        let mut clip_paragraphs = Vec::new();

        if start_para_idx == end_para_idx {
            // 단일 문단 내 선택
            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(
                &section.paragraphs[start_para_idx],
                start_char_offset,
                end_char_offset,
            ));
        } else {
            // 다중 문단 선택
            // 첫 번째 문단: start_offset부터 끝까지
            let first_text_len = section.paragraphs[start_para_idx].text.chars().count();
            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(
                &section.paragraphs[start_para_idx],
                start_char_offset,
                first_text_len,
            ));

            // 중간 문단: 전체 복사
            for i in (start_para_idx + 1)..end_para_idx {
                clip_paragraphs.push(section.paragraphs[i].clone());
            }

            // 마지막 문단: 처음부터 end_offset까지
            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(
                &section.paragraphs[end_para_idx],
                0,
                end_char_offset,
            ));
        }

        // 구조적 컨트롤(SectionDef, ColumnDef 등) 제거 — 텍스트 복사에 불필요
        for para in &mut clip_paragraphs {
            strip_structural_controls_for_text_clipboard(para);
        }

        // 플레인 텍스트 추출
        let plain_text: String = clip_paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let escaped = super::super::helpers::json_escape(&plain_text);

        self.clipboard = Some(ClipboardData {
            paragraphs: clip_paragraphs,
            plain_text: plain_text.clone(),
        });

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"text\":\"{}\"",
            escaped
        )))
    }

    /// 표 셀 내부 선택 영역을 내부 클립보드에 복사한다.
    pub fn copy_selection_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        start_cell_para_idx: usize,
        start_char_offset: usize,
        end_cell_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        // 셀 문단 리스트 접근
        let cell_paragraphs = {
            let section =
                self.document.sections.get(section_idx).ok_or_else(|| {
                    HwpError::RenderError(format!("구역 {} 범위 초과", section_idx))
                })?;
            let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx))
            })?;
            let table = match para.controls.get(control_idx) {
                Some(Control::Table(t)) => t,
                _ => return Err(HwpError::RenderError("표가 아님".to_string())),
            };
            let cell = table
                .cells
                .get(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?;
            &cell.paragraphs
        };

        if start_cell_para_idx >= cell_paragraphs.len()
            || end_cell_para_idx >= cell_paragraphs.len()
        {
            return Err(HwpError::RenderError(
                "셀 문단 인덱스 범위 초과".to_string(),
            ));
        }

        let mut clip_paragraphs = Vec::new();

        if start_cell_para_idx == end_cell_para_idx {
            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(
                &cell_paragraphs[start_cell_para_idx],
                start_char_offset,
                end_char_offset,
            ));
        } else {
            let first_text_len = cell_paragraphs[start_cell_para_idx].text.chars().count();
            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(
                &cell_paragraphs[start_cell_para_idx],
                start_char_offset,
                first_text_len,
            ));

            for i in (start_cell_para_idx + 1)..end_cell_para_idx {
                clip_paragraphs.push(cell_paragraphs[i].clone());
            }

            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(
                &cell_paragraphs[end_cell_para_idx],
                0,
                end_char_offset,
            ));
        }

        for para in &mut clip_paragraphs {
            strip_structural_controls_for_text_clipboard(para);
        }

        let plain_text: String = clip_paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let escaped = super::super::helpers::json_escape(&plain_text);

        self.clipboard = Some(ClipboardData {
            paragraphs: clip_paragraphs,
            plain_text: plain_text.clone(),
        });

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"text\":\"{}\"",
            escaped
        )))
    }

    /// 전체 cellPath가 가리키는 중첩 셀의 선택 영역을 내부 클립보드에 복사한다(#4272).
    pub fn copy_selection_in_cell_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        start_cell_para_idx: usize,
        start_char_offset: usize,
        end_cell_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }
        if start_cell_para_idx > end_cell_para_idx {
            return Err(HwpError::RenderError(
                "시작 위치가 끝 위치보다 뒤에 있음".to_string(),
            ));
        }

        let mut clip_paragraphs = Vec::new();
        for cell_para_idx in start_cell_para_idx..=end_cell_para_idx {
            let mut para_path = path.to_vec();
            para_path.last_mut().unwrap().2 = cell_para_idx;
            let para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, &para_path)?;
            let start = if cell_para_idx == start_cell_para_idx {
                start_char_offset
            } else {
                0
            };
            let end = if cell_para_idx == end_cell_para_idx {
                end_char_offset
            } else {
                para.text.chars().count()
            };
            clip_paragraphs.push(clip_paragraph_text_range_for_clipboard(para, start, end));
        }

        for para in &mut clip_paragraphs {
            strip_structural_controls_for_text_clipboard(para);
        }
        let plain_text = clip_paragraphs
            .iter()
            .map(|para| para.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let escaped = super::super::helpers::json_escape(&plain_text);
        self.clipboard = Some(ClipboardData {
            paragraphs: clip_paragraphs,
            plain_text,
        });

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"text\":\"{}\"",
            escaped
        )))
    }

    /// 컨트롤 객체(표, 이미지, 도형)를 내부 클립보드에 복사한다.
    pub fn copy_control_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        control_idx: usize,
    ) -> Result<String, HwpError> {
        // [Task #1161] cell_path 가 비면 본문, 아니면 셀/글상자 안 문단.
        let para = self.resolve_control_para(section_idx, para_idx, cell_path)?;
        let control = para
            .controls
            .get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        // 컨트롤을 포함하는 단일 문단 생성
        // text는 비워둠 (serialize_para_text가 controls에서 확장 제어문자를 생성)
        // control_mask: 1 << ctrl_char_code (Table/Shape=0x000B→bit11=0x800 등)
        let ctrl_char_code = match control {
            Control::Table(_) | Control::Shape(_) | Control::Picture(_) => 0x000Bu16,
            Control::SectionDef(_) | Control::ColumnDef(_) => 0x0002u16,
            Control::Footnote(_) | Control::Endnote(_) => 0x0011u16,
            Control::Header(_) | Control::Footer(_) => 0x0010u16,
            Control::AutoNumber(_) | Control::NewNumber(_) => 0x0012u16,
            _ => 0x000Bu16,
        };
        // 컨트롤 치수에 맞는 line_segs 생성 (insert_picture_native 패턴)
        let ctrl_line_seg = {
            let ctrl_height = match control {
                Control::Picture(pic) => pic.common.height as i32,
                Control::Shape(shape) => shape.common().height as i32,
                _ => 0,
            };
            if ctrl_height > 0 {
                LineSeg {
                    text_start: 0,
                    line_height: ctrl_height,
                    text_height: ctrl_height,
                    baseline_distance: (ctrl_height * 850) / 1000,
                    line_spacing: 600,
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    ..Default::default()
                }
            } else {
                LineSeg {
                    text_start: 0,
                    line_height: 400,
                    text_height: 400,
                    baseline_distance: 320,
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    ..Default::default()
                }
            }
        };

        let clip_para = Paragraph {
            text: String::new(),
            char_count: 9, // 확장 제어문자(8 code units) + 문단끝(1)
            control_mask: 1u32 << ctrl_char_code,
            char_offsets: vec![],
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                // 원본 문단 첫 글자가 아니라 컨트롤 앵커 위치의 글자모양을 복사한다.
                char_shape_id: para
                    .char_shape_id_at(
                        para.control_text_positions()
                            .get(control_idx)
                            .copied()
                            .unwrap_or(0),
                    )
                    .unwrap_or(0),
            }],
            line_segs: vec![ctrl_line_seg],
            para_shape_id: para.para_shape_id,
            style_id: para.style_id,
            controls: vec![control.clone()],
            ctrl_data_records: vec![para.ctrl_data_records.get(control_idx).cloned().flatten()],
            has_para_text: true,
            ..Default::default()
        };

        let plain_text = match control {
            Control::Table(_) => "[표]".to_string(),
            Control::Picture(_) => "[그림]".to_string(),
            Control::Shape(_) => "[도형]".to_string(),
            _ => "[컨트롤]".to_string(),
        };

        self.clipboard = Some(ClipboardData {
            paragraphs: vec![clip_para],
            plain_text: plain_text.clone(),
        });
        // [Task #1161] 새 컨트롤 복사 → cascade 리셋(다음 첫 붙여넣기부터 누적 시작).
        self.paste_cascade_count = 0;

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"text\":\"{}\"",
            plain_text
        )))
    }

    /// 내부 클립보드의 내용을 캐럿 위치에 붙여넣는다 (본문 문단).
    pub fn paste_internal_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let mut clip_paras = match &self.clipboard {
            Some(c) if !c.paragraphs.is_empty() => c.paragraphs.clone(),
            _ => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };
        let contains_field = clipboard_paragraphs_contain_field(&clip_paras);

        // 인덱스 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }

        self.document.sections[section_idx].raw_stream = None;
        self.renumber_pasted_field_ids(&mut clip_paras);

        let clip_count = clip_paras.len();

        if clip_count == 1 && clip_paras[0].controls.is_empty() {
            // 단일 문단 텍스트 붙여넣기 (컨트롤 없음)
            let clip_text = clip_paras[0].text.clone();
            let clip_char_shapes = clip_paras[0].char_shapes.clone();
            let clip_char_offsets = clip_paras[0].char_offsets.clone();
            let new_chars = clip_text.chars().count();

            // 텍스트 삽입
            self.document.sections[section_idx].paragraphs[para_idx]
                .insert_text_at(char_offset, &clip_text);

            // 클립보드의 글자 모양 적용
            self.apply_clipboard_char_shapes(
                section_idx,
                para_idx,
                char_offset,
                &clip_char_shapes,
                &clip_char_offsets,
                new_chars,
            );

            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
            // [Task #2299] 붙여넣기로 문단 높이가 변했으므로 하류 vpos 를 재연결한다.
            // 생략하면 후속 문단 first < 커진 end 세임이 저장돼 이후 편집의 리셋
            // 보존이 이를 단/쪽 경계로 오인한다.
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

            let new_offset = char_offset + new_chars;
            self.event_log.push(DocumentEvent::ContentPasted {
                section: section_idx,
                para: para_idx,
            });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{},\"containsField\":{}",
                para_idx, new_offset, contains_field
            )));
        }

        // 다중 문단 또는 컨트롤 포함 붙여넣기
        // 1. 현재 문단을 캐럿 위치에서 분할
        let right_half =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);

        // 2. 왼쪽 절반에 첫 번째 클립보드 문단 병합
        self.document.sections[section_idx].paragraphs[para_idx].merge_from(&clip_paras[0]);

        // 3. 나머지 클립보드 문단 삽입
        let mut insert_idx = para_idx + 1;
        for i in 1..clip_count {
            self.document.sections[section_idx]
                .paragraphs
                .insert(insert_idx, clip_paras[i].clone());
            insert_idx += 1;
        }

        // 4. 마지막 삽입된 문단에 오른쪽 절반 병합
        let last_para_idx = insert_idx - 1;
        let merge_point =
            self.document.sections[section_idx].paragraphs[last_para_idx].merge_from(&right_half);

        for i in para_idx..=last_para_idx {
            if !self.document.sections[section_idx].paragraphs[i]
                .field_ranges
                .is_empty()
            {
                rebuild_char_offsets(&mut self.document.sections[section_idx].paragraphs[i]);
            }
        }

        // 5. 영향받는 모든 문단 리플로우
        for i in para_idx..=last_para_idx {
            self.reflow_paragraph(section_idx, i);
        }

        // [Task #2299] 삽입 문단들의 vpos 를 흐름에 연결한다. 클립보드 클론의
        // 원본 좌표/placeholder 를 방치하면 이후 편집의 vpos 재계산이 이를 저장
        // 단/쪽 리셋으로 오인해 영구 고착시킨다 — 신규 구간은 리셋 보존에서 제외.
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            Some(para_idx + 1..last_para_idx + 1),
            None,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );

        // 6. 선택적 재구성: 삽입된 문단 composed 추가 + 영향 문단 재구성
        self.recompose_paragraph(section_idx, para_idx);
        for i in para_idx + 1..=last_para_idx {
            self.insert_composed_paragraph(section_idx, i);
        }
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":{},\"containsField\":{}",
            last_para_idx, merge_point, contains_field
        )))
    }

    fn paste_paragraphs_into_cell_paragraphs(
        cell_paras: &mut Vec<Paragraph>,
        cell_para_idx: usize,
        char_offset: usize,
        clip_paras: &[Paragraph],
    ) -> Result<(usize, usize), HwpError> {
        if cell_para_idx >= cell_paras.len() {
            return Err(HwpError::RenderError(format!(
                "셀 문단 {} 범위 초과",
                cell_para_idx
            )));
        }

        let clip_count = clip_paras.len();
        if clip_count == 1 && clip_paras[0].controls.is_empty() {
            let clip_text = clip_paras[0].text.clone();
            let new_chars = clip_text.chars().count();

            cell_paras[cell_para_idx].insert_text_at(char_offset, &clip_text);

            let clip_char_shapes = clip_paras[0].char_shapes.clone();
            let clip_char_offsets = clip_paras[0].char_offsets.clone();
            Self::apply_clipboard_char_shapes_to_para(
                &mut cell_paras[cell_para_idx],
                char_offset,
                &clip_char_shapes,
                &clip_char_offsets,
                new_chars,
            );

            return Ok((cell_para_idx, char_offset + new_chars));
        }

        let right_half = cell_paras[cell_para_idx].split_at(char_offset);
        cell_paras[cell_para_idx].merge_from(&clip_paras[0]);

        let mut insert_idx = cell_para_idx + 1;
        for clip_para in clip_paras.iter().skip(1) {
            cell_paras.insert(insert_idx, clip_para.clone());
            insert_idx += 1;
        }

        let last_para_idx = insert_idx - 1;
        let merge_point = cell_paras[last_para_idx].merge_from(&right_half);
        for para in &mut cell_paras[cell_para_idx..=last_para_idx] {
            if !para.field_ranges.is_empty() {
                rebuild_char_offsets(para);
            }
        }
        Ok((last_para_idx, merge_point))
    }

    /// 내부 클립보드의 내용을 표 셀 내부에 붙여넣는다.
    pub fn paste_internal_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let mut clip_paras = match &self.clipboard {
            Some(c) if !c.paragraphs.is_empty() => c.paragraphs.clone(),
            _ => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };
        let contains_field = clipboard_paragraphs_contain_field(&clip_paras);
        self.renumber_pasted_field_ids(&mut clip_paras);

        let (last_para_idx, merge_point) = {
            let section =
                self.document.sections.get_mut(section_idx).ok_or_else(|| {
                    HwpError::RenderError(format!("구역 {} 범위 초과", section_idx))
                })?;
            section.raw_stream = None;
            let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx))
            })?;
            let control = para.controls.get_mut(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx))
            })?;
            let cell_paras = match control {
                Control::Table(t) => {
                    &mut t
                        .cells
                        .get_mut(cell_idx)
                        .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?
                        .paragraphs
                }
                Control::Shape(s) => {
                    &mut super::super::helpers::get_textbox_from_shape_mut(s)
                        .ok_or_else(|| HwpError::RenderError("글상자 없음".to_string()))?
                        .paragraphs
                }
                Control::Picture(p) => {
                    &mut p
                        .caption
                        .as_mut()
                        .ok_or_else(|| HwpError::RenderError("캡션 없음".to_string()))?
                        .paragraphs
                }
                _ => return Err(HwpError::RenderError("표/글상자/캡션이 아님".to_string())),
            };
            Self::paste_paragraphs_into_cell_paragraphs(
                cell_paras,
                cell_para_idx,
                char_offset,
                &clip_paras,
            )?
        };

        for i in cell_para_idx..=last_para_idx {
            self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, i);
        }
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            Some(Control::Table(t)) => {
                t.dirty = true;
            }
            _ => {}
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted {
            section: section_idx,
            para: parent_para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIdx\":{},\"charOffset\":{},\"containsField\":{}",
            last_para_idx, merge_point, contains_field
        )))
    }

    /// 내부 클립보드의 내용을 cellPath가 가리키는 중첩 표 셀에 붙여넣는다.
    pub fn paste_internal_in_cell_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let mut clip_paras = match &self.clipboard {
            Some(c) if !c.paragraphs.is_empty() => c.paragraphs.clone(),
            _ => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };
        let contains_field = clipboard_paragraphs_contain_field(&clip_paras);
        self.renumber_pasted_field_ids(&mut clip_paras);
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let cell_para_idx = path[path.len() - 1].2;
        let (last_para_idx, merge_point) = {
            let cell_paras =
                self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)?;
            Self::paste_paragraphs_into_cell_paragraphs(
                cell_paras,
                cell_para_idx,
                char_offset,
                &clip_paras,
            )?
        };

        // [#2825] flat 형제 paste_internal_in_cell_native 는 붙여넣기 직후
        // reflow_cell_paragraph 로 셀 폭 기준 재래핑을 하지만, path 버전은 이 호출이
        // 없어 깊이 ≥2 중첩 셀에 붙여넣은 문단이 이전 line_segs 를 그대로 유지했다.
        // #2755 가 delete/split/merge by_path 에 도입한 reflow_cell_paragraph_by_path
        // 를 붙여넣기 경로에도 동일하게 적용한다.
        for i in cell_para_idx..=last_para_idx {
            self.reflow_cell_paragraph_by_path(section_idx, parent_para_idx, path, i);
        }

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted {
            section: section_idx,
            para: parent_para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIdx\":{},\"charOffset\":{},\"containsField\":{}",
            last_para_idx, merge_point, contains_field
        )))
    }

    /// 클립보드의 글자 모양(CharShape)을 삽입된 텍스트 범위에 적용한다.
    pub(crate) fn apply_clipboard_char_shapes(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        insert_offset: usize,
        clip_char_shapes: &[crate::model::paragraph::CharShapeRef],
        clip_char_offsets: &[u32],
        inserted_chars: usize,
    ) {
        Self::apply_clipboard_char_shapes_to_para(
            &mut self.document.sections[section_idx].paragraphs[para_idx],
            insert_offset,
            clip_char_shapes,
            clip_char_offsets,
            inserted_chars,
        );
    }

    /// 클립보드의 글자 모양을 특정 문단에 적용한다 (정적 메서드).
    pub(crate) fn apply_clipboard_char_shapes_to_para(
        para: &mut Paragraph,
        insert_offset: usize,
        clip_char_shapes: &[crate::model::paragraph::CharShapeRef],
        clip_char_offsets: &[u32],
        inserted_chars: usize,
    ) {
        if clip_char_shapes.is_empty() {
            return;
        }

        for i in 0..clip_char_shapes.len() {
            let cs = &clip_char_shapes[i];

            // UTF-16 위치를 char 인덱스로 변환
            let start_char_idx = clip_char_offsets
                .iter()
                .position(|&off| off >= cs.start_pos)
                .unwrap_or(0);

            let end_char_idx = if i + 1 < clip_char_shapes.len() {
                clip_char_offsets
                    .iter()
                    .position(|&off| off >= clip_char_shapes[i + 1].start_pos)
                    .unwrap_or(inserted_chars)
            } else {
                inserted_chars
            };

            if start_char_idx < end_char_idx && end_char_idx <= inserted_chars {
                para.apply_char_shape_range(
                    insert_offset + start_char_idx,
                    insert_offset + end_char_idx,
                    cs.char_shape_id,
                );
            }
        }
    }

    /// 내부 클립보드에 붙여넣기 가능한 개체 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
    /// SectionDef, ColumnDef 등 구조적 컨트롤은 개체가 아니므로 제외한다.
    pub fn clipboard_has_control_native(&self) -> bool {
        self.clipboard
            .as_ref()
            .map(|c| {
                c.paragraphs
                    .first()
                    .map(|p| {
                        p.controls.iter().any(|ctrl| {
                            matches!(
                                ctrl,
                                Control::Table(_) | Control::Picture(_) | Control::Shape(_)
                            )
                        })
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// 내부 클립보드의 컨트롤 객체를 캐럿 위치에 붙여넣는다 (본문).
    ///
    /// 클립보드에 컨트롤이 없으면 `{"ok":false}` 반환.
    /// 반환값: JSON `{"ok":true,"paraIdx":<idx>,"controlIdx":0}`
    pub fn paste_control_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // 클립보드에서 컨트롤 문단 확인
        let mut clip_para = match &self.clipboard {
            Some(c) => match c.paragraphs.first() {
                Some(p) if !p.controls.is_empty() => p.clone(),
                _ => return Ok("{\"ok\":false,\"error\":\"no control in clipboard\"}".to_string()),
            },
            None => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };

        // [Task #1161] 떠 있는 개체(treat_as_char=false) 반복 붙여넣기 시 한컴처럼
        // cascade 오프셋을 누적해 동일 위치 겹침을 방지한다. inline(글자처럼 취급)은
        // 텍스트 흐름이 위치를 정하므로 제외(첫 붙여넣기부터 +1*step).
        {
            let cascade = self.paste_cascade_count.saturating_add(1);
            let common = match clip_para.controls.first_mut() {
                Some(Control::Picture(p)) if !p.common.treat_as_char => Some(&mut p.common),
                Some(Control::Shape(s)) if !s.common().treat_as_char => Some(s.common_mut()),
                _ => None,
            };
            if let Some(common) = common {
                let off = cascade.saturating_mul(PASTE_CASCADE_STEP_HU);
                common.vertical_offset = common.vertical_offset.saturating_add(off);
                common.horizontal_offset = common.horizontal_offset.saturating_add(off);
                self.paste_cascade_count = cascade;
            }
        }

        // 인덱스 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 커서 위치 문단의 속성 상속 (빈 문단 생성용) — 커서 offset 의 글자모양 기준.
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shape_id_at(char_offset).unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 편집 영역 폭
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let content_width =
            (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32).max(7200) as u32;

        // 삽입 위치 결정 (create_shape_control_native 패턴)
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let is_empty_para = para.text.is_empty() && para.controls.is_empty();

        let insert_para_idx;
        // [Task #2299] 분할 삽입이면 우측 절반까지 신규 구간에 포함해야 한다.
        let mut did_split_for_control = false;
        if is_empty_para && char_offset == 0 {
            self.document.sections[section_idx].paragraphs[para_idx] = clip_para;
            insert_para_idx = para_idx;
        } else if char_offset == 0 && para.controls.is_empty() {
            self.document.sections[section_idx]
                .paragraphs
                .insert(para_idx, clip_para);
            insert_para_idx = para_idx;
        } else {
            if char_offset > 0 && !para.text.is_empty() {
                did_split_for_control = true;
                let new_para =
                    self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, new_para);
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, clip_para);
                insert_para_idx = para_idx + 1;
            } else {
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, clip_para);
                insert_para_idx = para_idx + 1;
            }
        }

        // 삽입된 문단의 line_segs 보정: 컨트롤 치수 반영
        // copy_control_native()에서 line_segs가 기본값(line_height:400, segment_width:0)으로
        // 하드코딩되므로, 실제 컨트롤 크기에 맞게 재설정한다.
        // (insert_picture_native 패턴: line_height=pic.height, segment_width=content_width)
        {
            let inserted = &mut self.document.sections[section_idx].paragraphs[insert_para_idx];
            let ctrl_height = inserted
                .controls
                .first()
                .map(|ctrl| match ctrl {
                    Control::Picture(pic) => pic.common.height as i32,
                    Control::Shape(shape) => shape.common().height as i32,
                    _ => 0,
                })
                .unwrap_or(0);
            if let Some(ls) = inserted.line_segs.first_mut() {
                ls.segment_width = content_width as i32;
                if ctrl_height > 0 {
                    ls.line_height = ctrl_height;
                    ls.text_height = ctrl_height;
                    ls.baseline_distance = (ctrl_height * 850) / 1000;
                    ls.line_spacing = 600;
                }
            }
        }

        // 컨트롤 아래에 빈 문단 추가 (HWP 표준)
        let mut empty_raw = vec![0u8; 10];
        empty_raw[0..2].copy_from_slice(&1u16.to_le_bytes());
        empty_raw[4..6].copy_from_slice(&1u16.to_le_bytes());
        let empty_para = Paragraph {
            text: String::new(),
            char_count: 1,
            char_count_msb: false,
            control_mask: 0,
            para_shape_id: default_para_shape_id,
            style_id: 0,
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: content_width as i32,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: false,
            raw_header_extra: empty_raw,
            ..Default::default()
        };
        self.document.sections[section_idx]
            .paragraphs
            .insert(insert_para_idx + 1, empty_para);

        // [Task #2299] 신규 문단들(컨트롤 host·이웃 빈 문단·분할 우측)의 placeholder
        // vpos 를 흐름에 연결한다 — 방치하면 이후 편집의 vpos 재계산이 저장 단/쪽
        // 리셋으로 오인해 영구 고착시킨다. 분할 좌측은 높이가 바뀌어 reflow 한다.
        let fresh_end = insert_para_idx + 2 + usize::from(did_split_for_control);
        if did_split_for_control {
            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
        }
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            Some(insert_para_idx..fresh_end),
            None,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );

        // 리플로우 + 페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted {
            section: section_idx,
            para: insert_para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":0",
            insert_para_idx
        )))
    }

    // === 클립보드 HTML 생성 ===

    /// 선택 영역을 HTML 문자열로 변환한다 (본문).
    pub fn export_selection_html_native(
        &self,
        section_idx: usize,
        start_para_idx: usize,
        start_char_offset: usize,
        end_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;

        if start_para_idx >= section.paragraphs.len() || end_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError("문단 범위 초과".to_string()));
        }

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");

        for pi in start_para_idx..=end_para_idx {
            let para = &section.paragraphs[pi];
            let start = if pi == start_para_idx {
                Some(start_char_offset)
            } else {
                None
            };
            let end = if pi == end_para_idx {
                Some(end_char_offset)
            } else {
                None
            };
            html.push_str(&self.paragraph_to_html(para, start, end));
        }

        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 선택 영역을 HTML 문자열로 변환한다 (셀 내부).
    pub fn export_selection_in_cell_html_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        start_cell_para_idx: usize,
        start_char_offset: usize,
        end_cell_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let para = section
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;
        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => return Err(HwpError::RenderError("표가 아님".to_string())),
        };
        let cell = table
            .cells
            .get(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?;

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");

        for pi in start_cell_para_idx..=end_cell_para_idx {
            if pi >= cell.paragraphs.len() {
                break;
            }
            let cpara = &cell.paragraphs[pi];
            let start = if pi == start_cell_para_idx {
                Some(start_char_offset)
            } else {
                None
            };
            let end = if pi == end_cell_para_idx {
                Some(end_char_offset)
            } else {
                None
            };
            // [#4413] 이 셀은 본문 직속(depth 0) 표의 셀이므로 그 안 문단에
            // 붙은 컨트롤(중첩 표·그림 등)은 depth 1부터 검사한다.
            html.push_str(&self.cell_paragraph_to_html(cpara, start, end, 1));
        }

        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 전체 cellPath가 가리키는 중첩 셀의 선택 영역을 HTML로 변환한다(#4272).
    pub fn export_selection_in_cell_html_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        start_cell_para_idx: usize,
        start_char_offset: usize,
        end_cell_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }
        if start_cell_para_idx > end_cell_para_idx {
            return Err(HwpError::RenderError(
                "시작 위치가 끝 위치보다 뒤에 있음".to_string(),
            ));
        }

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");
        // [#4413] path 의 각 항목이 표 중첩 한 단계이므로, 그 안 문단의
        // 컨트롤은 path.len() 깊이(= 이 셀 자체가 이미 path.len()번 중첩된
        // 지점)에서부터 검사한다. cell_path 가 1개면 depth 1부터 시작하는
        // `export_selection_in_cell_html_native`와 같은 규칙이다.
        let control_depth = path.len();
        for cell_para_idx in start_cell_para_idx..=end_cell_para_idx {
            let mut para_path = path.to_vec();
            para_path.last_mut().unwrap().2 = cell_para_idx;
            let para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, &para_path)?;
            let start = (cell_para_idx == start_cell_para_idx).then_some(start_char_offset);
            let end = (cell_para_idx == end_cell_para_idx).then_some(end_char_offset);
            html.push_str(&self.cell_paragraph_to_html(para, start, end, control_depth));
        }
        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 컨트롤 객체를 HTML 문자열로 변환한다.
    pub fn export_control_html_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        control_idx: usize,
    ) -> Result<String, HwpError> {
        // [Task #1161] cell_path 가 비면 본문, 아니면 셀/글상자 안 문단.
        let para = self.resolve_control_para(section_idx, para_idx, cell_path)?;
        let control = para
            .controls
            .get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");
        // 직접 선택해 내보내는 컨트롤은 문서 안 실제 중첩 위치와 무관하게 새
        // export 루트로 취급한다 — depth 0부터 MAX_NEST_DEPTH 예산을 새로 받는다.
        html.push_str(&self.control_to_html(control, 0));
        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 단일 문단을 HTML로 변환한다 (선택적 범위 지정).
    pub(crate) fn paragraph_to_html(
        &self,
        para: &Paragraph,
        start_offset: Option<usize>,
        end_offset: Option<usize>,
    ) -> String {
        let chars: Vec<char> = para.text.chars().collect();
        let start_idx = start_offset.unwrap_or(0).min(chars.len());
        let end_idx = end_offset.unwrap_or(chars.len()).min(chars.len());
        if start_idx >= end_idx {
            return String::new();
        }

        // 문단 스타일 CSS
        let para_css = self.para_style_to_css(para.para_shape_id);
        let mut html = format!("<p style=\"margin:0;{}\">\n", para_css);

        // CharShapeRef 경계에서 스타일이 바뀌는 지점을 찾아 span 분할
        let style_ranges = self.get_char_style_ranges(para, start_idx, end_idx);

        for (range_start, range_end, char_shape_id) in &style_ranges {
            let segment: String = chars[*range_start..*range_end]
                .iter()
                .filter(|c| !c.is_control() || **c == '\t')
                .collect();

            if segment.is_empty() {
                continue;
            }

            let css = self.char_style_to_css(*char_shape_id);
            html.push_str(&format!(
                "<span style=\"{}\">{}</span>",
                css,
                clipboard_escape_html(&segment)
            ));
        }

        html.push_str("</p>\n");
        html
    }

    /// 문단 내 char 인덱스 범위에서 CharShapeRef 경계를 기준으로 (start, end, char_shape_id) 목록을 반환한다.
    pub(crate) fn get_char_style_ranges(
        &self,
        para: &Paragraph,
        start_idx: usize,
        end_idx: usize,
    ) -> Vec<(usize, usize, u32)> {
        if para.char_shapes.is_empty() {
            return vec![(start_idx, end_idx, 0)];
        }

        // CharShapeRef의 start_pos (UTF-16) → char index 변환
        let mut boundaries: Vec<(usize, u32)> = Vec::new();
        for cs in &para.char_shapes {
            let char_idx = utf16_pos_to_char_idx(&para.char_offsets, cs.start_pos);
            boundaries.push((char_idx, cs.char_shape_id));
        }

        let mut ranges = Vec::new();
        for i in 0..boundaries.len() {
            let (bound_start, shape_id) = boundaries[i];
            let bound_end = if i + 1 < boundaries.len() {
                boundaries[i + 1].0
            } else {
                end_idx
            };

            // 범위와 교차하는 부분만 포함
            let rs = bound_start.max(start_idx);
            let re = bound_end.min(end_idx);
            if rs < re {
                ranges.push((rs, re, shape_id));
            }
        }

        // 시작점 이전에 스타일이 없으면 첫 CharShapeRef의 스타일 사용
        if ranges.is_empty() && !boundaries.is_empty() {
            let last_before = boundaries
                .iter()
                .rev()
                .find(|(idx, _)| *idx <= start_idx)
                .map(|(_, id)| *id)
                .unwrap_or(boundaries[0].1);
            ranges.push((start_idx, end_idx, last_before));
        }

        ranges
    }

    /// CharShape ID → CSS 인라인 스타일 문자열 변환.
    pub(crate) fn char_style_to_css(&self, char_shape_id: u32) -> String {
        let cs = match self.styles.char_styles.get(char_shape_id as usize) {
            Some(s) => s,
            None => return String::new(),
        };

        let mut css = String::new();

        // font-family (한국어 + 영어 폰트 목록)
        let mut fonts: Vec<&str> = Vec::new();
        if !cs.font_family.is_empty() {
            fonts.push(&cs.font_family);
        }
        if cs.font_families.len() > 1
            && !cs.font_families[1].is_empty()
            && cs.font_families[1] != cs.font_family
        {
            fonts.push(&cs.font_families[1]);
        }
        if !fonts.is_empty() {
            let font_list: Vec<String> = fonts
                .iter()
                .map(|f| format!("'{}'", clipboard_escape_html(f)))
                .collect();
            css.push_str(&format!("font-family:{};", font_list.join(",")));
        }

        // font-size (px → pt 변환: pt = px * 72 / 96)
        if cs.font_size > 0.0 {
            let pt = cs.font_size * 72.0 / self.dpi;
            css.push_str(&format!("font-size:{:.1}pt;", pt));
        }

        // font-weight / font-style
        if cs.bold {
            css.push_str("font-weight:bold;");
        }
        if cs.italic {
            css.push_str("font-style:italic;");
        }

        // color
        let color = clipboard_color_to_css(cs.text_color);
        css.push_str(&format!("color:{};", color));

        // text-decoration
        let has_underline = !matches!(cs.underline, crate::model::style::UnderlineType::None);
        if has_underline && cs.strikethrough {
            css.push_str("text-decoration:underline line-through;");
        } else if has_underline {
            css.push_str("text-decoration:underline;");
        } else if cs.strikethrough {
            css.push_str("text-decoration:line-through;");
        }

        // letter-spacing (0이 아닌 경우만)
        if cs.letter_spacing.abs() > 0.1 {
            css.push_str(&format!("letter-spacing:{:.1}px;", cs.letter_spacing));
        }

        css
    }

    /// ParaShape ID → CSS 인라인 스타일 문자열 변환.
    pub(crate) fn para_style_to_css(&self, para_shape_id: u16) -> String {
        let ps = match self.styles.para_styles.get(para_shape_id as usize) {
            Some(s) => s,
            None => return String::new(),
        };

        let mut css = String::new();

        // text-align
        let align = match ps.alignment {
            crate::model::style::Alignment::Left => "left",
            crate::model::style::Alignment::Right => "right",
            crate::model::style::Alignment::Center => "center",
            crate::model::style::Alignment::Justify => "justify",
            crate::model::style::Alignment::Distribute => "justify",
            crate::model::style::Alignment::Split => "justify",
        };
        css.push_str(&format!("text-align:{};", align));

        // margin-left / margin-right (px)
        if ps.margin_left > 0.1 {
            css.push_str(&format!("margin-left:{:.1}px;", ps.margin_left));
        }
        if ps.margin_right > 0.1 {
            css.push_str(&format!("margin-right:{:.1}px;", ps.margin_right));
        }

        // text-indent
        if ps.indent.abs() > 0.1 {
            css.push_str(&format!("text-indent:{:.1}px;", ps.indent));
        }

        // line-height
        match ps.line_spacing_type {
            crate::model::style::LineSpacingType::Percent => {
                css.push_str(&format!("line-height:{:.0}%;", ps.line_spacing));
            }
            crate::model::style::LineSpacingType::Fixed => {
                css.push_str(&format!("line-height:{:.1}px;", ps.line_spacing));
            }
            _ => {}
        }

        css
    }

    /// Control 객체를 HTML로 변환한다.
    ///
    /// `depth`는 이 컨트롤을 담은 셀의 표 중첩 깊이(0 = 최상위 표 바로 안)다.
    /// `Control::Table`이 `depth >= MAX_NEST_DEPTH`이면 재귀를 멈추고 생략
    /// 사실을 주석으로 남긴다 — `table_extract::MAX_NEST_DEPTH`와 같은 값·형태.
    /// `Control::Table`/`Control::Picture` 외 변형은 아직 셀 안에서 내보내기를
    /// 지원하지 않는다 [#4413]. 지원 범위 확장은 #4414 소관이라 여기서는 조용히
    /// 버리지 않고 어떤 컨트롤이 생략됐는지 HTML 주석 경고로 남긴다.
    pub(crate) fn control_to_html(&self, control: &Control, depth: usize) -> String {
        match control {
            Control::Table(table) => {
                if depth >= MAX_NEST_DEPTH {
                    return format!(
                        "<!-- rhwp: 표 중첩 깊이 상한({})을 넘어 생략됨 -->\n",
                        MAX_NEST_DEPTH
                    );
                }
                self.table_to_html_at_depth(table, depth)
            }
            Control::Picture(pic) => self.picture_to_html(pic),
            other => format!(
                "<!-- rhwp: 셀 안 {} 컨트롤은 클립보드 HTML 내보내기 미지원 - 경고: 내용 생략됨 -->\n",
                control_kind_label(other)
            ),
        }
    }

    /// 최상위 Table 컨트롤을 HTML <table>로 변환한다.
    ///
    /// 재귀 깊이는 내부 구현에서만 관리해, 기존 최상위 변환 진입점의 계약을
    /// 유지한다.
    pub(crate) fn table_to_html(&self, table: &crate::model::table::Table) -> String {
        self.table_to_html_at_depth(table, 0)
    }

    /// Table 컨트롤을 현재 중첩 깊이의 HTML <table>로 변환한다.
    /// `depth`는 이 표 자체의 중첩 깊이(0 = 최상위) — 셀 안 문단의 컨트롤을
    /// 처리할 때는 `depth + 1`을 넘겨, 그 컨트롤이 표이면 `control_to_html`이
    /// 상한을 검사한 뒤 그 값으로 재귀한다.
    fn table_to_html_at_depth(&self, table: &crate::model::table::Table, depth: usize) -> String {
        use crate::renderer::style_resolver::ResolvedBorderStyle;

        let mut html = String::from(
            "<table style=\"border-collapse:collapse;\" cellpadding=\"0\" cellspacing=\"0\">\n",
        );

        // 행별로 그룹화
        let max_row = table.cells.iter().map(|c| c.row).max().unwrap_or(0);
        for row in 0..=max_row {
            html.push_str("<tr>\n");
            let mut row_cells: Vec<&crate::model::table::Cell> =
                table.cells.iter().filter(|c| c.row == row).collect();
            row_cells.sort_by_key(|c| c.col);

            for cell in &row_cells {
                // 병합된 셀은 첫 번째 셀만 출력 (rowspan/colspan 은 merge 된 셀 정보)
                let mut td_style = String::new();

                // 셀 배경/테두리 (BorderFill) — border_fill_id는 1-based
                // (styles.border_styles는 0-based). 다른 소비처(예:
                // renderer/layout/table_layout.rs, document_core/queries/hidden_text.rs)와
                // 동일하게 -1 보정한다. [#4412]
                if cell.border_fill_id > 0 {
                    let idx = (cell.border_fill_id as usize).saturating_sub(1);
                    if let Some(bs) = self.styles.border_styles.get(idx) {
                        self.apply_border_fill_css(&mut td_style, bs);
                    }
                }

                // 셀 패딩
                td_style.push_str("padding:1px 5px;");

                // vertical-align
                td_style.push_str("vertical-align:top;");

                let mut td_attrs = format!("style=\"{}\"", td_style);
                if cell.col_span > 1 {
                    td_attrs.push_str(&format!(" colspan=\"{}\"", cell.col_span));
                }
                if cell.row_span > 1 {
                    td_attrs.push_str(&format!(" rowspan=\"{}\"", cell.row_span));
                }

                html.push_str(&format!("<td {}>\n", td_attrs));

                // 셀 내부 문단들 — 텍스트뿐 아니라 문단에 붙은 컨트롤(중첩
                // 표·그림 등)도 함께 내보낸다 [#4413]. 이 표 자체는 depth이므로
                // 그 셀 문단의 컨트롤은 depth+1에서 검사한다.
                for cpara in &cell.paragraphs {
                    html.push_str(&self.cell_paragraph_to_html(cpara, None, None, depth + 1));
                }

                html.push_str("</td>\n");
            }
            html.push_str("</tr>\n");
        }

        html.push_str("</table>\n");
        html
    }

    /// 셀 안 문단을 HTML로 변환한다 — 텍스트(`paragraph_to_html`)에 더해
    /// `para.controls`(중첩 표·그림 등)도 순회해 이어붙인다.
    ///
    /// [#4413] 셀 내용 내보내기가 `para.controls`를 전혀 보지 않아 표 셀 안의
    /// 중첩 표·이미지가 경고 없이 통째로 사라지던 결함 수정. `paragraph_to_html`
    /// 자체는 본문(셀 밖) 선택 내보내기와도 공유하므로 그대로 두고, 셀 전용
    /// 진입점에서만 컨트롤을 이어붙인다.
    ///
    /// `depth`는 이 문단을 담은 셀이 속한 표의 중첩 깊이다 — `control_to_html`에
    /// 그대로 전달해 `MAX_NEST_DEPTH` 상한을 적용한다.
    pub(crate) fn cell_paragraph_to_html(
        &self,
        para: &Paragraph,
        start_offset: Option<usize>,
        end_offset: Option<usize>,
        depth: usize,
    ) -> String {
        let mut html = self.paragraph_to_html(para, start_offset, end_offset);
        for ctrl in &para.controls {
            html.push_str(&self.control_to_html(ctrl, depth));
        }
        html
    }

    /// BorderFill 스타일을 CSS로 변환하여 추가한다.
    pub(crate) fn apply_border_fill_css(
        &self,
        css: &mut String,
        bs: &crate::renderer::style_resolver::ResolvedBorderStyle,
    ) {
        // 배경색
        if let Some(fill_color) = bs.fill_color {
            if fill_color != 0xFFFFFF && fill_color != 0 {
                css.push_str(&format!(
                    "background-color:{};",
                    clipboard_color_to_css(fill_color)
                ));
            }
        }

        // 테두리 (좌, 우, 상, 하)
        let sides = ["left", "right", "top", "bottom"];
        for (i, side) in sides.iter().enumerate() {
            let bl = &bs.borders[i];
            if bl.width > 0 {
                let color = clipboard_color_to_css(bl.color);
                let px = (bl.width as f64).max(1.0);
                css.push_str(&format!("border-{}:{:.1}px solid {};", side, px, color));
            }
        }
    }

    /// Picture 컨트롤을 HTML <img>로 변환한다.
    pub(crate) fn picture_to_html(&self, pic: &crate::model::image::Picture) -> String {
        use base64::Engine;

        let bin_data_id = pic.image_attr.bin_data_id;
        if bin_data_id == 0 {
            return String::new();
        }

        // 이미지 데이터 찾기 (bin_data_id는 1-indexed 순번)
        let image_data = if bin_data_id > 0 {
            self.document
                .bin_data_content
                .get((bin_data_id - 1) as usize)
        } else {
            None
        };

        // [#2550] 상한 초과(deflate bomb 포함)는 이미지 누락과 같은 빈 조각으로 접는다.
        if let Some(bytes) = image_data.and_then(|bdc| {
            bdc.data
                .load_limited(crate::model::bin_data::MAX_BIN_DATA_BYTES)
        }) {
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let mime_type = detect_clipboard_image_mime(&bytes);

            // 크기 계산 (HWPUNIT → px)
            let w = crate::renderer::hwpunit_to_px(pic.common.width as i32, self.dpi);
            let h = crate::renderer::hwpunit_to_px(pic.common.height as i32, self.dpi);

            format!(
                "<img src=\"data:{};base64,{}\" width=\"{:.0}\" height=\"{:.0}\" />\n",
                mime_type, base64_data, w, h
            )
        } else {
            String::new()
        }
    }

    /// 컨트롤의 이미지 바이너리 데이터를 반환한다.
    /// Picture 컨트롤만 지원하며, 다른 타입은 에러를 반환한다.
    pub fn get_control_image_data_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        control_idx: usize,
    ) -> Result<Vec<u8>, HwpError> {
        // [Task #1161] cell_path 가 비면 본문, 아니면 셀/글상자 안 문단.
        let para = self.resolve_control_para(section_idx, para_idx, cell_path)?;
        let control = para
            .controls
            .get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        let pic = match control {
            Control::Picture(p) => p,
            _ => {
                return Err(HwpError::RenderError(
                    "Picture 컨트롤이 아닙니다".to_string(),
                ))
            }
        };

        let bin_data_id = pic.image_attr.bin_data_id;
        if bin_data_id == 0 {
            return Err(HwpError::RenderError(
                "이미지 데이터 없음 (bin_data_id=0)".to_string(),
            ));
        }

        let bdc = self
            .document
            .bin_data_content
            .get((bin_data_id - 1) as usize)
            .ok_or_else(|| {
                HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id))
            })?;

        bdc.data
            .load_limited(crate::model::bin_data::MAX_BIN_DATA_BYTES)
            .ok_or_else(|| bin_data_over_limit_error(bin_data_id))
    }

    /// 컨트롤의 이미지 MIME 타입을 반환한다.
    pub fn get_control_image_mime_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        control_idx: usize,
    ) -> Result<String, HwpError> {
        // [Task #1161] cell_path 가 비면 본문, 아니면 셀/글상자 안 문단.
        let para = self.resolve_control_para(section_idx, para_idx, cell_path)?;
        let control = para
            .controls
            .get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        let pic = match control {
            Control::Picture(p) => p,
            _ => {
                return Err(HwpError::RenderError(
                    "Picture 컨트롤이 아닙니다".to_string(),
                ))
            }
        };

        let bin_data_id = pic.image_attr.bin_data_id;
        if bin_data_id == 0 {
            return Err(HwpError::RenderError(
                "이미지 데이터 없음 (bin_data_id=0)".to_string(),
            ));
        }

        let bdc = self
            .document
            .bin_data_content
            .get((bin_data_id - 1) as usize)
            .ok_or_else(|| {
                HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id))
            })?;

        let bytes = bdc
            .data
            .load_limited(crate::model::bin_data::MAX_BIN_DATA_BYTES)
            .ok_or_else(|| bin_data_over_limit_error(bin_data_id))?;
        Ok(detect_clipboard_image_mime(&bytes).to_string())
    }

    /// BinData ID(1-based)로 이미지 바이너리 데이터를 반환한다.
    pub fn get_bin_data_image_data_native(&self, bin_data_id: u16) -> Result<Vec<u8>, HwpError> {
        if bin_data_id == 0 {
            return Err(HwpError::RenderError(
                "이미지 데이터 없음 (bin_data_id=0)".to_string(),
            ));
        }
        let bdc = self
            .document
            .bin_data_content
            .get((bin_data_id - 1) as usize)
            .ok_or_else(|| {
                HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id))
            })?;
        bdc.data
            .load_limited(crate::model::bin_data::MAX_BIN_DATA_BYTES)
            .ok_or_else(|| bin_data_over_limit_error(bin_data_id))
    }

    /// BinData ID(1-based)로 이미지 MIME 타입을 반환한다.
    pub fn get_bin_data_image_mime_native(&self, bin_data_id: u16) -> Result<String, HwpError> {
        if bin_data_id == 0 {
            return Err(HwpError::RenderError(
                "이미지 데이터 없음 (bin_data_id=0)".to_string(),
            ));
        }
        let bdc = self
            .document
            .bin_data_content
            .get((bin_data_id - 1) as usize)
            .ok_or_else(|| {
                HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id))
            })?;
        let bytes = bdc
            .data
            .load_limited(crate::model::bin_data::MAX_BIN_DATA_BYTES)
            .ok_or_else(|| bin_data_over_limit_error(bin_data_id))?;
        Ok(detect_clipboard_image_mime(&bytes).to_string())
    }

    // === 클립보드 HTML 붙여넣기 ===
}

#[cfg(test)]
mod char_shape_inherit_tests {
    use crate::document_core::DocumentCore;
    use crate::model::paragraph::CharShapeRef;

    /// 혼합 글자모양 문단: 텍스트 20자, 글자 인덱스 0~9 는 34, 10~ 는 37.
    fn core_with_mixed_shape_paragraph() -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "0123456789abcdefghij")
            .unwrap();
        let para = &mut core.document.sections[0].paragraphs[0];
        // 컨트롤(SectionDef 등)이 UTF-16 앞자리를 차지하므로 경계는 char_offsets 로 계산.
        let boundary = para.char_offsets[10];
        para.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 34,
            },
            CharShapeRef {
                start_pos: boundary,
                char_shape_id: 37,
            },
        ];
        core
    }

    /// 혼합 문단 offset 12(글자모양 37 구간)에 인라인 표를 만들고 복사하면,
    /// 클립보드 문단은 문단 첫 글자모양(34)이 아니라 컨트롤 앵커 위치의
    /// 글자모양(37)을 가져야 한다.
    #[test]
    fn copy_control_uses_char_shape_at_control_anchor() {
        let mut core = core_with_mixed_shape_paragraph();
        let res = core
            .create_table_ex_native(0, 0, 12, 1, 1, true, None, None)
            .unwrap();
        let ctrl_idx: usize = res
            .split("\"controlIdx\":")
            .nth(1)
            .and_then(|s| s.split([',', '}']).next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap();

        core.copy_control_native(0, 0, &[], ctrl_idx).unwrap();

        let clip_para = core
            .clipboard
            .as_ref()
            .and_then(|c| c.paragraphs.first())
            .expect("클립보드 문단");
        assert_eq!(
            clip_para.char_shapes.first().map(|cs| cs.char_shape_id),
            Some(37),
            "클립보드 문단이 컨트롤 앵커 글자모양(37)이 아닌 값을 가짐"
        );
    }

    /// 혼합 문단 offset 10 에 컨트롤을 붙여넣으면, 컨트롤 아래에 생성되는
    /// 빈 문단은 커서 offset 글자모양(37)을 상속해야 한다.
    #[test]
    fn paste_control_empty_neighbor_inherits_char_shape_at_cursor_offset() {
        let mut core = core_with_mixed_shape_paragraph();
        let res = core
            .create_table_ex_native(0, 0, 12, 1, 1, true, None, None)
            .unwrap();
        let ctrl_idx: usize = res
            .split("\"controlIdx\":")
            .nth(1)
            .and_then(|s| s.split([',', '}']).next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap();
        core.copy_control_native(0, 0, &[], ctrl_idx).unwrap();

        let res = core.paste_control_native(0, 0, 10).unwrap();
        let insert_para_idx: usize = res
            .split("\"paraIdx\":")
            .nth(1)
            .and_then(|s| s.split([',', '}']).next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap();

        let empty_neighbor = &core.document.sections[0].paragraphs[insert_para_idx + 1];
        assert_eq!(
            empty_neighbor
                .char_shapes
                .first()
                .map(|cs| cs.char_shape_id),
            Some(37),
            "붙여넣기 후 빈 이웃 문단이 커서 offset 글자모양(37)이 아닌 값을 상속"
        );
    }
}

/// [#2825] `paste_internal_in_cell_by_path_native` 가 깊이 ≥2 중첩 셀에서도
/// 최내곽 셀 폭으로 재래핑(reflow)하는지 검증한다.
#[cfg(test)]
mod nested_cell_paste_reflow_tests {
    use crate::document_core::{ClipboardData, DocumentCore};
    use crate::model::control::Control;
    use crate::model::document::{Document, Section, SectionDef};
    use crate::model::page::PageDef;
    use crate::model::paragraph::{CharShapeRef, Paragraph};
    use crate::model::table::{Cell, Table};

    /// 바깥 표(셀 폭 5000, 넉넉함) 문단 안에 안쪽 표(셀 폭 200, 좁음)를 중첩시키고,
    /// 안쪽 셀에는 빈 문단 하나만 둔다. path 는 [(outer,0,0),(inner,0,0)].
    fn core_with_nested_narrow_empty_cell() -> (DocumentCore, Vec<(usize, usize, usize)>) {
        let inner_para = Paragraph::default();

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

    /// [#2825] red→green: 40자 텍스트를 내부 클립보드로 채운 뒤 폭 200 최내곽 셀에
    /// 붙여넣으면, 붙여넣기 직후 셀 폭 기준으로 재래핑돼 여러 줄이어야 한다.
    /// 수정 전에는 `paste_internal_in_cell_by_path_native` 가 재래핑을 호출하지
    /// 않아 `line_segs` 가 1줄(insert_text_at 직후 미보정 상태)로 남았다.
    #[test]
    fn paste_in_nested_cell_by_path_reflows_inner_cell() {
        let (mut core, path) = core_with_nested_narrow_empty_cell();
        let text = "A".repeat(40);
        core.clipboard = Some(ClipboardData {
            paragraphs: vec![Paragraph {
                text: text.clone(),
                char_offsets: (0..text.chars().count() as u32).collect(),
                char_count: text.chars().count() as u32,
                char_shapes: vec![CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                }],
                has_para_text: true,
                ..Default::default()
            }],
            plain_text: text,
        });

        core.paste_internal_in_cell_by_path_native(0, 0, &path, 0)
            .expect("중첩 셀 붙여넣기가 성공해야 함");

        let paras = core.get_cell_paragraphs_mut_by_path(0, 0, &path).unwrap();
        assert!(
            paras[0].line_segs.len() > 1,
            "폭 200 최내곽 셀에 40자를 붙여넣으면 여러 줄로 재래핑돼야 함 (실제 {}줄)",
            paras[0].line_segs.len()
        );
    }
}

/// [#4412] `table_to_html`(문서 간 복사·붙여넣기 HTML 경로)이 `cell.border_fill_id`를
/// 1-based 보정 없이 `styles.border_styles`(0-based)에 그대로 인덱싱해, 실제 BorderFill보다
/// 한 칸 뒤(id 기준 +1)의 BorderFill 색상이 붙던 결함의 회귀 테스트.
#[cfg(test)]
mod clipboard_border_fill_offset_tests {
    use crate::document_core::DocumentCore;
    use crate::model::style::{BorderFill, BorderLine, BorderLineType, Fill, FillType, SolidFill};
    use crate::model::table::{Cell, Table};
    use crate::renderer::style_resolver::resolve_styles;

    /// 4방향 동일한 실선 테두리 + 단색 채우기를 갖는 BorderFill을 만든다.
    fn border_fill(border_color: u32, fill_color: u32) -> BorderFill {
        let line = BorderLine {
            line_type: BorderLineType::Solid,
            width: 2,
            color: border_color,
        };
        BorderFill {
            borders: [line, line, line, line],
            fill: Fill {
                fill_type: FillType::Solid,
                solid: Some(SolidFill {
                    background_color: fill_color,
                    pattern_color: 0,
                    pattern_type: 0,
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// border_fills = [default0, default1, decoy(회색), REAL(녹/청), decoy2(주황/자홍)].
    /// `cell.border_fill_id = 4`(1-based)는 index 3(REAL)을 가리켜야 하며, 색이 한 칸
    /// 밀려 index 4(decoy2)를 가리키면 안 된다.
    #[test]
    fn table_to_html_uses_correct_border_fill_for_1_based_id() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        core.document.doc_info.border_fills = vec![
            border_fill(0x000000, 0x000000), // index 0 → id 1 (default0)
            border_fill(0x000000, 0x000000), // index 1 → id 2 (default1)
            border_fill(0xC0C0C0, 0xC0C0C0), // index 2 → id 3 (decoy, 회색)
            border_fill(0x00FF00, 0xFFFF00), // index 3 → id 4 (REAL, 테두리#00ff00/배경#00ffff)
            border_fill(0x008CFF, 0xFF00FF), // index 4 → id 5 (decoy2, 테두리#ff8c00/배경#ff00ff)
        ];
        core.styles = resolve_styles(&core.document.doc_info, core.dpi);

        let table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                row: 0,
                col: 0,
                col_span: 1,
                row_span: 1,
                width: 2000,
                border_fill_id: 4,
                ..Default::default()
            }],
            ..Default::default()
        };

        let html = core.table_to_html(&table);

        assert!(
            html.contains("border-left:2.0px solid #00ff00"),
            "REAL(id=4, index 3)의 테두리색(#00ff00)이 출력되지 않음:\n{html}"
        );
        assert!(
            html.contains("background-color:#00ffff"),
            "REAL(id=4, index 3)의 배경색(#00ffff)이 출력되지 않음:\n{html}"
        );
        assert!(
            !html.contains("#ff8c00") && !html.contains("#ff00ff"),
            "decoy2(id=5, index 4)의 색상이 한 칸 밀려 출력됨:\n{html}"
        );
    }
}

/// #4413: `paragraph_to_html`이 `para.text`만 읽고 `para.controls`를 보지 않아
/// 표 셀 안의 중첩 표·이미지가 문서 간 복사에서 통째로 사라지던 결함의 회귀 테스트.
#[cfg(test)]
mod cell_control_export_tests {
    use super::MAX_NEST_DEPTH;
    use crate::document_core::DocumentCore;
    use crate::model::control::{Bookmark, Control};
    use crate::model::document::{Document, Section};
    use crate::model::image::{ImageAttr, Picture};
    use crate::model::paragraph::Paragraph;
    use crate::model::shape::CommonObjAttr;
    use crate::model::table::{Cell, Table};

    /// 본문 문단 하나에 표 컨트롤(1x1)을 붙이고, 그 유일한 셀의 문단으로
    /// `outer_cell_para`를 넣는다. 반환값은 (core, parent_para_idx, control_idx, cell_idx).
    fn core_with_body_table(outer_cell_para: Paragraph) -> (DocumentCore, usize, usize, usize) {
        let outer_table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                row: 0,
                col: 0,
                col_span: 1,
                row_span: 1,
                width: 5000,
                paragraphs: vec![outer_cell_para],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut body_para = Paragraph::default();
        body_para
            .controls
            .push(Control::Table(Box::new(outer_table)));

        let mut section = Section::default();
        section.paragraphs.push(body_para);
        let mut doc = Document::default();
        doc.sections.push(section);

        let mut core = DocumentCore::new_empty();
        core.document = doc;

        (core, 0, 0, 0)
    }

    fn make_inner_table_with_marker(marker: &str) -> Table {
        let mut inner_para = Paragraph::default();
        inner_para.text = marker.to_string();
        inner_para.char_offsets = (0..inner_para.text.chars().count() as u32).collect();

        Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                row: 0,
                col: 0,
                col_span: 1,
                row_span: 1,
                width: 2000,
                paragraphs: vec![inner_para],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// [적대적 검증 대조군] 내부 클립보드(같은 문서)는 원래 셀의 컨트롤을
    /// 보존한다 — 이슈 #4413이 명시한 대조군. `copy_selection_in_cell_native`는
    /// `Paragraph::clone()`으로 셀 문단을 통째로 복사하고, `strip_structural_
    /// controls_for_text_clipboard`가 SectionDef/ColumnDef만 제거하므로
    /// `Control::Table`은 그대로 남는다.
    #[test]
    fn internal_clipboard_preserves_nested_table_in_cell_control_group() {
        let mut outer_cell_para = Paragraph::default();
        outer_cell_para
            .controls
            .push(Control::Table(Box::new(make_inner_table_with_marker(
                "INNERMARK",
            ))));

        let (mut core, parent_para_idx, control_idx, cell_idx) =
            core_with_body_table(outer_cell_para);

        core.copy_selection_in_cell_native(0, parent_para_idx, control_idx, cell_idx, 0, 0, 0, 0)
            .expect("내부 클립보드 복사가 성공해야 함");

        let clip = core.clipboard.as_ref().expect("클립보드가 채워져야 함");
        assert_eq!(clip.paragraphs.len(), 1);
        assert!(
            matches!(clip.paragraphs[0].controls.first(), Some(Control::Table(_))),
            "내부 클립보드(같은 문서 안 복사)는 중첩 표 컨트롤을 보존해야 함(대조군). 실제 controls: {:?}",
            clip.paragraphs[0].controls
        );
    }

    /// #4413 red→green: 셀 문단에 붙은 `Control::Table`(중첩 표)이 셀 내용 HTML
    /// 내보내기에서 사라지면 안 된다. 수정 전에는 `export_selection_in_cell_html_native`가
    /// `paragraph_to_html`만 호출해(controls 미참조) 안쪽 표가 전혀 나타나지 않았다.
    #[test]
    fn nested_table_in_cell_is_exported_to_html() {
        let mut outer_cell_para = Paragraph::default();
        outer_cell_para
            .controls
            .push(Control::Table(Box::new(make_inner_table_with_marker(
                "INNERMARK",
            ))));

        let (core, parent_para_idx, control_idx, cell_idx) = core_with_body_table(outer_cell_para);

        let html = core
            .export_selection_in_cell_html_native(
                0,
                parent_para_idx,
                control_idx,
                cell_idx,
                0,
                0,
                0,
                0,
            )
            .expect("셀 내용 HTML 내보내기가 성공해야 함");

        assert_eq!(
            html.matches("<table").count(),
            1,
            "셀 안 중첩 표가 <table 하나로 내보내져야 함. 실제 HTML: {html}"
        );
        assert!(
            html.contains("INNERMARK"),
            "안쪽 표 셀 텍스트가 내보낸 HTML에 있어야 함. 실제 HTML: {html}"
        );
    }

    /// #4413 red→green, cellPath 버전(#4272): `export_selection_in_cell_html_by_path_native`도
    /// 같은 결함을 공유했다 — cellPath 하나짜리 얕은 셀도 컨트롤을 보지 않았다.
    #[test]
    fn nested_table_in_cell_by_path_is_exported_to_html() {
        let mut outer_cell_para = Paragraph::default();
        outer_cell_para
            .controls
            .push(Control::Table(Box::new(make_inner_table_with_marker(
                "INNERMARK",
            ))));

        let (core, parent_para_idx, control_idx, cell_idx) = core_with_body_table(outer_cell_para);
        let path = vec![(control_idx, cell_idx, 0)];

        let html = core
            .export_selection_in_cell_html_by_path_native(0, parent_para_idx, &path, 0, 0, 0, 0)
            .expect("cellPath 셀 내용 HTML 내보내기가 성공해야 함");

        assert!(
            html.contains("<table"),
            "cellPath 경로로 내보내도 셀 안 중첩 표가 나와야 함. 실제 HTML: {html}"
        );
        assert!(
            html.contains("INNERMARK"),
            "cellPath 경로로 내보내도 안쪽 표 셀 텍스트가 있어야 함. 실제 HTML: {html}"
        );
    }

    /// #4413 red→green: 셀 문단에 붙은 `Control::Picture`(셀 안 이미지)가 셀 내용
    /// HTML 내보내기에서 사라지면 안 된다.
    #[test]
    fn picture_in_cell_is_exported_to_html() {
        let mut outer_cell_para = Paragraph::default();
        outer_cell_para
            .controls
            .push(Control::Picture(Box::new(Picture {
                common: CommonObjAttr {
                    treat_as_char: true,
                    width: 1000,
                    height: 1000,
                    ..Default::default()
                },
                image_attr: ImageAttr {
                    bin_data_id: 1,
                    ..Default::default()
                },
                ..Default::default()
            })));

        let (mut core, parent_para_idx, control_idx, cell_idx) =
            core_with_body_table(outer_cell_para);
        core.document
            .bin_data_content
            .push(crate::model::bin_data::BinDataContent {
                id: 1,
                data: crate::model::bin_data::BinDataBytes::from_shared(vec![
                    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A,
                ]),
                extension: "png".to_string(),
            });

        let html = core
            .export_selection_in_cell_html_native(
                0,
                parent_para_idx,
                control_idx,
                cell_idx,
                0,
                0,
                0,
                0,
            )
            .expect("셀 내용 HTML 내보내기가 성공해야 함");

        assert!(
            html.contains("<img"),
            "셀 안 이미지가 <img>로 내보내져야 함. 실제 HTML: {html}"
        );
    }

    /// [본문 대조군] 본문(셀 밖) 그림은 애초에 문제가 없었다 — 직접 컨트롤을
    /// 선택해 내보내는 `export_control_html_native` 경로는 셀 순회와 무관하게
    /// 항상 `control_to_html`을 탔다. 위치(셀 안 vs 본문 직접 선택)에 결함이
    /// 국한됨을 확정하는 대조군.
    #[test]
    fn body_level_picture_export_via_control_html_was_already_fine() {
        let pic = Picture {
            common: CommonObjAttr {
                treat_as_char: true,
                width: 1000,
                height: 1000,
                ..Default::default()
            },
            image_attr: ImageAttr {
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut body_para = Paragraph::default();
        body_para.controls.push(Control::Picture(Box::new(pic)));

        let mut section = Section::default();
        section.paragraphs.push(body_para);
        let mut doc = Document::default();
        doc.sections.push(section);
        doc.bin_data_content
            .push(crate::model::bin_data::BinDataContent {
                id: 1,
                data: crate::model::bin_data::BinDataBytes::from_shared(vec![
                    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A,
                ]),
                extension: "png".to_string(),
            });

        let mut core = DocumentCore::new_empty();
        core.document = doc;

        let html = core
            .export_control_html_native(0, 0, &[], 0)
            .expect("본문 컨트롤 HTML 내보내기가 성공해야 함");

        assert!(
            html.contains("<img"),
            "본문에서 직접 선택한 그림은 원래도 <img>로 내보내져야 함. 실제 HTML: {html}"
        );
    }

    /// #4413 수정: `Control::Table`/`Control::Picture` 외의 셀 안 컨트롤(예: 책갈피)은
    /// 아직 지원 대상이 아니다(#4414 소관) — 조용히 버리지 않고 HTML 주석 경고를
    /// 남겨야 한다.
    #[test]
    fn unsupported_cell_control_leaves_warning_comment_not_silent_drop() {
        let mut outer_cell_para = Paragraph::default();
        outer_cell_para
            .controls
            .push(Control::Bookmark(Bookmark::default()));

        let (core, parent_para_idx, control_idx, cell_idx) = core_with_body_table(outer_cell_para);

        let html = core
            .export_selection_in_cell_html_native(
                0,
                parent_para_idx,
                control_idx,
                cell_idx,
                0,
                0,
                0,
                0,
            )
            .expect("셀 내용 HTML 내보내기가 성공해야 함");

        assert!(
            html.contains("<!--") && html.contains("Bookmark"),
            "지원하지 않는 셀 컨트롤은 조용히 버리지 말고 어떤 컨트롤이 생략됐는지 \
             HTML 주석 경고를 남겨야 함. 실제 HTML: {html}"
        );
    }

    /// #4413 수정: 표 중첩 HTML 변환 재귀에 `MAX_NEST_DEPTH` 상한이 실제로 걸린다.
    /// `MAX_NEST_DEPTH + 4`단 깊이로 표를 중첩시키고 최상위에서 내보내면, 가장
    /// 안쪽(마커 포함) 표는 상한을 넘어 렌더링되지 않고 생략 주석만 남아야 한다.
    /// 상한이 없으면 병적으로 깊은 중첩 문서에서 export 재귀가 스택을 태울 수 있다.
    #[test]
    fn nested_table_recursion_is_capped_at_max_depth() {
        const LEVELS: usize = MAX_NEST_DEPTH + 4;

        let mut table = make_inner_table_with_marker("DEEPMARK");
        for _ in 0..LEVELS {
            let mut wrapper_para = Paragraph::default();
            wrapper_para.controls.push(Control::Table(Box::new(table)));
            table = Table {
                row_count: 1,
                col_count: 1,
                cells: vec![Cell {
                    row: 0,
                    col: 0,
                    col_span: 1,
                    row_span: 1,
                    width: 2000,
                    paragraphs: vec![wrapper_para],
                    ..Default::default()
                }],
                ..Default::default()
            };
        }

        let mut body_para = Paragraph::default();
        body_para.controls.push(Control::Table(Box::new(table)));

        let mut section = Section::default();
        section.paragraphs.push(body_para);
        let mut doc = Document::default();
        doc.sections.push(section);

        let mut core = DocumentCore::new_empty();
        core.document = doc;

        let html = core
            .export_control_html_native(0, 0, &[], 0)
            .expect("최상위 표 컨트롤 HTML 내보내기가 성공해야 함");

        assert!(
            !html.contains("DEEPMARK"),
            "깊이 상한을 넘는 가장 안쪽 표는 렌더링되면 안 됨. 실제 HTML: {html}"
        );
        assert!(
            html.contains("깊이 상한"),
            "깊이 상한에 걸리면 생략 사실을 주석으로 남겨야 함. 실제 HTML: {html}"
        );
    }
}

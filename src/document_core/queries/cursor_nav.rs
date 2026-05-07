//! 커서 이동/줄 정보/경로 탐색/선택 영역 관련 native 메서드

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::render_tree::PageRenderTree;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use super::super::helpers::{LineInfoResult, utf16_pos_to_char_idx, has_table_control, get_textbox_from_shape, navigable_text_len};

impl DocumentCore {
    pub(crate) fn get_line_info_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let para = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs.get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)))?;

        Self::compute_line_info(para, char_offset)
    }

    /// 셀 내 문단의 줄 정보를 반환한다 (네이티브).
    pub(crate) fn get_line_info_in_cell_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let para = self.get_cell_paragraph_ref(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!(
                "셀 문단 참조 실패: sec={} ppi={} ci={} cei={} cpi={}",
                section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
            )))?;

        Self::compute_line_info(para, char_offset)
    }

    /// 문단의 line_segs에서 charOffset이 속한 줄 정보를 계산한다 (JSON 반환).
    pub(crate) fn compute_line_info(
        para: &crate::model::paragraph::Paragraph,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let info = Self::compute_line_info_struct(para, char_offset)?;
        Ok(format!(
            "{{\"lineIndex\":{},\"lineCount\":{},\"charStart\":{},\"charEnd\":{}}}",
            info.line_index, info.line_count, info.char_start, info.char_end
        ))
    }

    /// 문단의 line_segs에서 charOffset이 속한 줄 정보를 구조체로 반환한다.
    pub(crate) fn compute_line_info_struct(
        para: &crate::model::paragraph::Paragraph,
        char_offset: usize,
    ) -> Result<LineInfoResult, HwpError> {
        let char_count = navigable_text_len(para);
        let line_segs = &para.line_segs;

        if line_segs.is_empty() {
            return Ok(LineInfoResult {
                line_index: 0,
                line_count: 1,
                char_start: 0,
                char_end: char_count,
            });
        }

        let line_char_starts = Self::build_line_char_starts(para);
        let line_count = line_char_starts.len();

        // charOffset이 속한 줄 찾기
        let mut line_index = 0;
        for i in 1..line_count {
            if char_offset >= line_char_starts[i] {
                line_index = i;
            } else {
                break;
            }
        }

        let char_start = line_char_starts[line_index];
        let raw_char_end = if line_index + 1 < line_count {
            line_char_starts[line_index + 1]
        } else {
            char_count
        };
        // 강제 줄바꿈(\n, 0x000A)이 줄 끝에 있으면 그 앞 위치를 char_end로 사용
        // (End 키가 다음 줄로 넘어가는 것을 방지)
        let char_end = if line_index + 1 < line_count && raw_char_end > char_start {
            let chars: Vec<char> = para.text.chars().collect();
            if raw_char_end > 0 && chars.get(raw_char_end - 1) == Some(&'\n') {
                raw_char_end - 1
            } else {
                raw_char_end
            }
        } else {
            raw_char_end
        };

        Ok(LineInfoResult { line_index, line_count, char_start, char_end })
    }

    /// 문단의 line_segs에서 각 줄의 시작 char index 배열을 구한다.
    pub(crate) fn build_line_char_starts(para: &crate::model::paragraph::Paragraph) -> Vec<usize> {
        let char_offsets = &para.char_offsets;
        para.line_segs.iter().map(|ls| {
            if ls.text_start == 0 { 0 } else { utf16_pos_to_char_idx(char_offsets, ls.text_start) }
        }).collect()
    }

    /// 특정 줄의 문자 범위(charStart, charEnd)를 반환한다.
    pub(crate) fn get_line_char_range(para: &crate::model::paragraph::Paragraph, line_index: usize) -> (usize, usize) {
        let char_count = navigable_text_len(para);
        if para.line_segs.is_empty() {
            return (0, char_count);
        }
        let starts = Self::build_line_char_starts(para);
        let line_count = starts.len();
        if line_index >= line_count {
            return (char_count, char_count);
        }
        let char_start = starts[line_index];
        let char_end = if line_index + 1 < line_count { starts[line_index + 1] } else { char_count };
        (char_start, char_end)
    }

    /// 문서에 저장된 캐럿 위치를 반환한다 (네이티브).
    pub(crate) fn get_caret_position_native(&self) -> Result<String, HwpError> {
        let props = &self.document.doc_properties;
        let section_idx = props.caret_list_id as usize;
        let para_idx = props.caret_para_id as usize;
        let caret_utf16 = props.caret_char_pos;

        // 범위 검증
        let section = match self.document.sections.get(section_idx) {
            Some(s) => s,
            None => {
                // 범위 초과 시 문서 시작 반환
                return Ok("{\"sectionIndex\":0,\"paragraphIndex\":0,\"charOffset\":0}".to_string());
            }
        };

        let para = match section.paragraphs.get(para_idx) {
            Some(p) => p,
            None => {
                return Ok("{\"sectionIndex\":0,\"paragraphIndex\":0,\"charOffset\":0}".to_string());
            }
        };

        // UTF-16 → char index 변환
        let char_offset = if caret_utf16 == 0 {
            0
        } else {
            utf16_pos_to_char_idx(&para.char_offsets, caret_utf16)
        };

        // char_offset이 문단 길이를 초과하지 않도록
        let safe_offset = char_offset.min(navigable_text_len(para));

        Ok(format!(
            "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{}}}",
            section_idx, para_idx, safe_offset
        ))
    }

    /// 표의 행/열/셀 수를 반환한다 (네이티브).
    pub(crate) fn move_vertical_native(
        &self,
        sec: usize,
        para: usize,
        char_offset: usize,
        delta: i32,
        preferred_x: f64,
        cell_ctx: Option<(usize, usize, usize, usize)>,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        // ═══ PHASE 1: preferredX 결정 ═══
        let actual_px = if preferred_x < 0.0 {
            match self.get_cursor_rect_values(sec, para, char_offset, cell_ctx) {
                Ok((_, x, _, _)) => x,
                Err(_) => 0.0,
            }
        } else {
            preferred_x
        };

        // ═══ PHASE 2: 현재 줄 정보 + 목표 줄 결정 ═══
        let current_para = self.resolve_paragraph(sec, para, cell_ctx)?;
        let line_info = Self::compute_line_info_struct(current_para, char_offset)
            .unwrap_or(LineInfoResult { line_index: 0, line_count: 1, char_start: 0, char_end: navigable_text_len(current_para) });
        let target_line = line_info.line_index as i32 + delta;

        // ═══ PHASE 3: 목표 위치 결정 ═══
        // 결과: (sec, para, char_offset, cell_ctx)
        let new_pos: (usize, usize, usize, Option<(usize, usize, usize, usize)>);

        if target_line >= 0 && (target_line as usize) < line_info.line_count {
            // CASE A: 같은 문단 내 다른 줄
            // PartialParagraph로 같은 문단이 두 칼럼에 걸칠 수 있으므로
            // 현재 줄과 목표 줄의 칼럼이 다르면 preferredX를 변환한다.
            let px_for_target = if cell_ctx.is_none() {
                let cur_col = self.find_column_for_line(sec, para, line_info.line_index);
                let tgt_col = self.find_column_for_line(sec, para, target_line as usize);
                match (cur_col, tgt_col) {
                    (Some((fc, fx, _)), Some((tc, tx, _))) if fc != tc => {
                        let relative_x = actual_px - fx;
                        tx + relative_x
                    }
                    _ => actual_px,
                }
            } else {
                actual_px
            };
            let target_range = Self::get_line_char_range(current_para, target_line as usize);
            let new_offset = self.find_char_at_x_on_line(sec, para, cell_ctx, target_range, px_for_target)?;
            new_pos = (sec, para, new_offset, cell_ctx);
        } else if cell_ctx.is_some() {
            // CASE C: 셀 내부 경계
            new_pos = self.handle_cell_boundary(sec, para, char_offset, delta, actual_px, cell_ctx.unwrap())?;
        } else {
            // CASE B: 본문 문단/구역 경계
            new_pos = self.handle_body_boundary(sec, para, delta, actual_px)?;
        }

        // ═══ PHASE 4: 최종 커서 좌표 계산 + 결과 포맷 ═══
        let (rect_valid, page_idx, fx, fy, fh) = match self.get_cursor_rect_values(
            new_pos.0, new_pos.1, new_pos.2, new_pos.3,
        ) {
            Ok((p, x, y, h)) => (true, p, x, y, h),
            Err(_) => (false, 0, 0.0, 0.0, 16.0),
        };

        // JSON 직렬화
        let pos_json = if let Some((ppi, ci, cei, cpi)) = new_pos.3 {
            // 글상자 여부: cell_index==0이고 컨트롤이 Shape
            let is_tb = cei == 0 && self.document.sections.get(new_pos.0)
                .and_then(|s| s.paragraphs.get(ppi))
                .and_then(|p| p.controls.get(ci))
                .map(|c| matches!(c, Control::Shape(_)))
                .unwrap_or(false);
            let tb_str = if is_tb { ",\"isTextBox\":true" } else { "" };
            format!(
                "\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}{}",
                new_pos.0, new_pos.1, new_pos.2, ppi, ci, cei, cpi, tb_str
            )
        } else {
            format!(
                "\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{}",
                new_pos.0, new_pos.1, new_pos.2
            )
        };

        let rect_valid_str = if rect_valid { "" } else { ",\"rectValid\":false" };
        Ok(format!(
            "{{{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1},\"preferredX\":{:.1}{}}}",
            pos_json, page_idx, fx, fy, fh, actual_px, rect_valid_str
        ))
    }

    /// 문단 참조를 얻는다 (본문/셀 통합).
    /// 오버플로우 타겟 글상자인 경우 소스의 오버플로우 문단으로 리디렉트한다.
    pub(crate) fn resolve_paragraph(
        &self,
        sec: usize,
        para: usize,
        cell_ctx: Option<(usize, usize, usize, usize)>,
    ) -> Result<&Paragraph, HwpError> {
        if let Some((ppi, ci, cei, cpi)) = cell_ctx {
            // 글상자(cei==0)이면 오버플로우 타겟인지 확인
            if cei == 0 {
                if let Some(p) = self.resolve_overflow_paragraph(sec, ppi, ci, cpi) {
                    return Ok(p);
                }
            }
            self.get_cell_paragraph_ref(sec, ppi, ci, cei, cpi)
                .ok_or_else(|| HwpError::RenderError(format!(
                    "셀 문단 참조 실패: sec={} ppi={} ci={} cei={} cpi={}", sec, ppi, ci, cei, cpi
                )))
        } else {
            self.document.sections.get(sec)
                .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", sec)))?
                .paragraphs.get(para)
                .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para)))
        }
    }

    /// 오버플로우 타겟 글상자의 문단을 소스의 오버플로우 문단으로 리디렉트한다.
    fn resolve_overflow_paragraph(
        &self,
        sec: usize,
        ppi: usize,
        ci: usize,
        cpi: usize,
    ) -> Option<&Paragraph> {
        let overflow_links = self.get_overflow_links(sec);
        let link = overflow_links.iter().find(|l|
            l.target_parent_para == ppi && l.target_ctrl_idx == ci
        )?;
        let section = self.document.sections.get(sec)?;
        let src_para = section.paragraphs.get(link.source_parent_para)?;
        if let Control::Shape(s) = src_para.controls.get(link.source_ctrl_idx)? {
            let src_tb = get_textbox_from_shape(s)?;
            src_tb.paragraphs.get(link.overflow_start + cpi)
        } else {
            None
        }
    }

    /// 오버플로우 타겟 글상자의 유효 문단 수를 반환한다.
    fn overflow_para_count(&self, sec: usize, ppi: usize, ci: usize) -> Option<usize> {
        let overflow_links = self.get_overflow_links(sec);
        let link = overflow_links.iter().find(|l|
            l.target_parent_para == ppi && l.target_ctrl_idx == ci
        )?;
        let section = self.document.sections.get(sec)?;
        let src_para = section.paragraphs.get(link.source_parent_para)?;
        if let Control::Shape(s) = src_para.controls.get(link.source_ctrl_idx)? {
            let src_tb = get_textbox_from_shape(s)?;
            Some(src_tb.paragraphs.len() - link.overflow_start)
        } else {
            None
        }
    }

    /// 오버플로우 소스 글상자의 렌더 문단 수(overflow_start)를 반환한다.
    fn source_rendered_para_count(&self, sec: usize, ppi: usize, ci: usize) -> Option<usize> {
        let overflow_links = self.get_overflow_links(sec);
        let link = overflow_links.iter().find(|l|
            l.source_parent_para == ppi && l.source_ctrl_idx == ci
        )?;
        Some(link.overflow_start)
    }

    /// JSON 문자열에서 CellPathEntry 배열을 파싱한다.
    pub(crate) fn parse_cell_path(path_json: &str) -> Result<Vec<(usize, usize, usize)>, HwpError> {
        // 경량 JSON 파서: [{"controlIndex":N,"cellIndex":N,"cellParaIndex":N}, ...]
        let trimmed = path_json.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(HwpError::RenderError("cellPath JSON은 배열이어야 합니다".to_string()));
        }
        let inner = &trimmed[1..trimmed.len()-1];
        if inner.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        // 중괄호 기준으로 각 엔트리 분리
        let mut depth = 0;
        let mut start = 0;
        for (i, ch) in inner.char_indices() {
            match ch {
                '{' => { if depth == 0 { start = i; } depth += 1; }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let entry_str = &inner[start..=i];
                        let ci = super::super::helpers::json_usize(entry_str, "controlIndex")?;
                        let cei = super::super::helpers::json_usize(entry_str, "cellIndex")?;
                        let cpi = super::super::helpers::json_usize(entry_str, "cellParaIndex")?;
                        entries.push((ci, cei, cpi));
                    }
                }
                _ => {}
            }
        }
        Ok(entries)
    }

    /// 경로 기반으로 표를 탐색한다.
    /// path: [(control_index, cell_index, cell_para_index), ...]
    /// 마지막 엔트리의 control_index로 도달한 표를 반환.
    pub(crate) fn resolve_table_by_path<'a>(
        &'a self,
        sec: usize,
        parent_para: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&'a crate::model::table::Table, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let mut para = self.document.sections.get(sec)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", sec)))?
            .paragraphs.get(parent_para)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para)))?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let table = match para.controls.get(ctrl_idx) {
                Some(Control::Table(t)) => t,
                _ => return Err(HwpError::RenderError(format!(
                    "경로[{}]: controls[{}]가 표가 아닙니다", i, ctrl_idx
                ))),
            };

            if i == path.len() - 1 {
                return Ok(table);
            }

            // 다음 레벨로 진입: 셀 → 문단 → 다음 표
            let cell = table.cells.get(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!(
                    "경로[{}]: 셀 {} 범위 초과 (총 {}개)", i, cell_idx, table.cells.len()
                )))?;
            para = cell.paragraphs.get(cell_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!(
                    "경로[{}]: 셀문단 {} 범위 초과 (총 {}개)", i, cell_para_idx, cell.paragraphs.len()
                )))?;
        }

        unreachable!()
    }

    /// 경로 기반으로 셀을 탐색한다 (마지막 엔트리의 cell_index).
    pub(crate) fn resolve_cell_by_path<'a>(
        &'a self,
        sec: usize,
        parent_para: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&'a crate::model::table::Cell, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let last = path.last().unwrap();
        let table = self.resolve_table_by_path(sec, parent_para, path)?;
        table.cells.get(last.1)
            .ok_or_else(|| HwpError::RenderError(format!(
                "셀 {} 범위 초과 (총 {}개)", last.1, table.cells.len()
            )))
    }

    /// 경로 기반으로 셀/글상자 내 문단을 탐색한다 (표와 글상자 모두 지원).
    pub(crate) fn resolve_paragraph_by_path<'a>(
        &'a self,
        sec: usize,
        parent_para: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&'a Paragraph, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let mut para = self.document.sections.get(sec)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", sec)))?
            .paragraphs.get(parent_para)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para)))?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let next_para = match para.controls.get(ctrl_idx) {
                Some(Control::Table(table)) => {
                    let cell = table.cells.get(cell_idx)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 셀 {} 범위 초과 (총 {}개)", i, cell_idx, table.cells.len()
                        )))?;
                    cell.paragraphs.get(cell_para_idx)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 셀문단 {} 범위 초과 (총 {}개)", i, cell_para_idx, cell.paragraphs.len()
                        )))?
                }
                Some(Control::Shape(shape)) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})", i, cell_idx
                        )));
                    }
                    let text_box = get_textbox_from_shape(shape)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다", i, ctrl_idx
                        )))?;
                    text_box.paragraphs.get(cell_para_idx)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 글상자문단 {} 범위 초과 (총 {}개)", i, cell_para_idx, text_box.paragraphs.len()
                        )))?
                }
                _ => return Err(HwpError::RenderError(format!(
                    "경로[{}]: controls[{}]가 표/글상자가 아닙니다", i, ctrl_idx
                ))),
            };

            para = next_para;
        }

        Ok(para)
    }

    /// 경로가 가리키는 컨테이너(표 셀/글상자)의 문단 수를 반환한다.
    pub(crate) fn resolve_container_para_count_by_path(
        &self,
        sec: usize,
        parent_para: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<usize, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let mut para = self.document.sections.get(sec)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", sec)))?
            .paragraphs.get(parent_para)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para)))?;

        // 중간 경로 탐색 (마지막 엔트리 제외)
        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path[..path.len()-1].iter().enumerate() {
            let next_para = match para.controls.get(ctrl_idx) {
                Some(Control::Table(table)) => {
                    let cell = table.cells.get(cell_idx)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 셀 {} 범위 초과", i, cell_idx
                        )))?;
                    cell.paragraphs.get(cell_para_idx)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 셀문단 {} 범위 초과", i, cell_para_idx
                        )))?
                }
                Some(Control::Shape(shape)) => {
                    let text_box = get_textbox_from_shape(shape)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 글상자가 아닙니다", i
                        )))?;
                    text_box.paragraphs.get(cell_para_idx)
                        .ok_or_else(|| HwpError::RenderError(format!(
                            "경로[{}]: 글상자문단 {} 범위 초과", i, cell_para_idx
                        )))?
                }
                _ => return Err(HwpError::RenderError(format!(
                    "경로[{}]: controls[{}]가 표/글상자가 아닙니다", i, ctrl_idx
                ))),
            };
            para = next_para;
        }

        // 마지막 엔트리: 컨테이너의 문단 수 반환
        let last = path.last().unwrap();
        match para.controls.get(last.0) {
            Some(Control::Table(table)) => {
                let cell = table.cells.get(last.1)
                    .ok_or_else(|| HwpError::RenderError(format!(
                        "셀 {} 범위 초과", last.1
                    )))?;
                Ok(cell.paragraphs.len())
            }
            Some(Control::Shape(shape)) => {
                let text_box = get_textbox_from_shape(shape)
                    .ok_or_else(|| HwpError::RenderError(
                        "글상자가 아닙니다".to_string()
                    ))?;
                Ok(text_box.paragraphs.len())
            }
            _ => Err(HwpError::RenderError(format!(
                "controls[{}]가 표/글상자가 아닙니다", last.0
            ))),
        }
    }

    /// 커서 좌표를 (pageIndex, x, y, height) 튜플로 반환한다 (본문/셀 통합).
    pub(crate) fn get_cursor_rect_values(
        &self,
        sec: usize,
        para: usize,
        char_offset: usize,
        cell_ctx: Option<(usize, usize, usize, usize)>,
    ) -> Result<(u32, f64, f64, f64), HwpError> {
        let json = if let Some((ppi, ci, cei, cpi)) = cell_ctx {
            self.get_cursor_rect_in_cell_native(sec, ppi, ci, cei, cpi, char_offset)?
        } else {
            self.get_cursor_rect_native(sec, para, char_offset)?
        };
        use super::super::helpers::json_f64;
        let page_idx = json_f64(&json, "pageIndex").unwrap_or(0.0) as u32;
        let x = json_f64(&json, "x").unwrap_or(0.0);
        let y = json_f64(&json, "y").unwrap_or(0.0);
        let height = json_f64(&json, "height").unwrap_or(0.0);
        Ok((page_idx, x, y, height))
    }

    /// 렌더 트리에서 특정 줄(char_range)의 TextRun을 찾아 preferredX에 가장 가까운 문자를 반환한다.
    pub(crate) fn find_char_at_x_on_line(
        &self,
        sec: usize,
        para: usize,
        cell_ctx: Option<(usize, usize, usize, usize)>,
        char_range: (usize, usize),
        preferred_x: f64,
    ) -> Result<usize, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        // 해당 문단이 포함된 페이지의 렌더 트리 빌드
        let pages = if let Some((ppi, _, _, _)) = cell_ctx {
            self.find_pages_for_paragraph(sec, ppi)?
        } else {
            self.find_pages_for_paragraph(sec, para)?
        };

        struct RunMatch {
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
        }

        fn collect_matching_runs(
            node: &RenderNode,
            sec: usize,
            para: usize,
            cell_ctx: Option<(usize, usize, usize, usize)>,
            char_range: (usize, usize),
            result: &mut Vec<RunMatch>,
        ) {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                let matches = if let Some((ppi, ci, cei, cpi)) = cell_ctx {
                    tr.section_index == Some(sec)
                        && tr.cell_context.as_ref().map_or(false, |ctx| {
                            ctx.parent_para_index == ppi
                                && ctx.path[0].control_index == ci
                                && ctx.path[0].cell_index == cei
                                && ctx.path[0].cell_para_index == cpi
                        })
                } else {
                    tr.section_index == Some(sec)
                        && tr.para_index == Some(para)
                        && tr.cell_context.is_none()
                };
                // 번호/글머리표 TextRun (char_start: None)은 건너뛴다
                if let (true, Some(cs)) = (matches, tr.char_start) {
                    let cc = tr.text.chars().count();
                    // 이 run이 목표 줄의 char_range에 겹치는지 확인
                    if cs < char_range.1 && cs + cc > char_range.0 {
                        let positions = compute_char_positions(&tr.text, &tr.style);
                        result.push(RunMatch {
                            char_start: cs,
                            char_count: cc,
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                        });
                    }
                }
            }
            for child in &node.children {
                collect_matching_runs(child, sec, para, cell_ctx, char_range, result);
            }
        }

        // 페이지 순회하며 매칭 run 수집
        for &page_num in &pages {
            let tree = self.build_page_tree(page_num)?;
            let mut runs = Vec::new();
            collect_matching_runs(&tree.root, sec, para, cell_ctx, char_range, &mut runs);

            if !runs.is_empty() {
                // preferredX에 가장 가까운 문자 찾기
                runs.sort_by_key(|a| a.char_start);
                let mut best_offset = char_range.0;
                let mut best_dist = f64::MAX;

                for run in &runs {
                    for i in 0..=run.char_count {
                        let global_offset = run.char_start + i;
                        // char_range 범위 내의 문자만 고려
                        if global_offset < char_range.0 || global_offset > char_range.1 {
                            continue;
                        }
                        let x = run.bbox_x + if i < run.char_positions.len() {
                            run.char_positions[i]
                        } else if !run.char_positions.is_empty() {
                            *run.char_positions.last().unwrap()
                        } else {
                            0.0
                        };
                        let dist = (x - preferred_x).abs();
                        if dist < best_dist {
                            best_dist = dist;
                            best_offset = global_offset;
                        }
                    }
                }
                return Ok(best_offset);
            }
        }

        // 렌더 트리에서 못 찾은 경우 → 줄 시작으로 폴백
        Ok(char_range.0)
    }

    /// 본문 문단/구역 경계를 넘어 이동한다.
    pub(crate) fn handle_body_boundary(
        &self,
        sec: usize,
        para: usize,
        delta: i32,
        preferred_x: f64,
    ) -> Result<(usize, usize, usize, Option<(usize, usize, usize, usize)>), HwpError> {
        let section = self.document.sections.get(sec)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", sec)))?;
        let target_para_i = para as i32 + delta;

        // 구역 경계 처리
        if target_para_i < 0 {
            if sec == 0 {
                return Ok((sec, para, 0, None)); // 문서 시작 — 이동 안 함
            }
            let prev_sec = sec - 1;
            let prev_para_count = self.document.sections[prev_sec].paragraphs.len();
            if prev_para_count == 0 {
                return Ok((sec, para, 0, None));
            }
            return self.enter_paragraph(prev_sec, prev_para_count - 1, delta, preferred_x);
        }

        let target_para = target_para_i as usize;
        if target_para >= section.paragraphs.len() {
            if sec + 1 >= self.document.sections.len() {
                // 문서 끝 — 이동 안 함
                let para_len = navigable_text_len(&self.document.sections[sec].paragraphs[para]);
                return Ok((sec, para, para_len, None));
            }
            return self.enter_paragraph(sec + 1, 0, delta, preferred_x);
        }

        // 칼럼 경계를 넘는 경우 preferredX를 대상 칼럼 좌표계로 변환
        let adjusted_px = self.transform_preferred_x_across_columns(sec, para, target_para, preferred_x);
        self.enter_paragraph(sec, target_para, delta, adjusted_px)
    }

    /// 목표 문단으로 진입한다 (표면 표 문단이면 셀 내부로).
    pub(crate) fn enter_paragraph(
        &self,
        sec: usize,
        target_para: usize,
        delta: i32,
        preferred_x: f64,
    ) -> Result<(usize, usize, usize, Option<(usize, usize, usize, usize)>), HwpError> {
        let para_ref = self.document.sections.get(sec)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", sec)))?
            .paragraphs.get(target_para)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", target_para)))?;

        // 표 컨트롤 확인
        if let Some(ctrl_idx) = has_table_control(para_ref) {
            if let Some(Control::Table(ref table)) = para_ref.controls.get(ctrl_idx) {
                if delta > 0 {
                    // ArrowDown → 첫 셀(0,0)의 첫 줄
                    if let Some(first_cell) = table.cells.first() {
                        if !first_cell.paragraphs.is_empty() {
                            let cell_para = &first_cell.paragraphs[0];
                            let range = Self::get_line_char_range(cell_para, 0);
                            let cell_ctx = Some((target_para, ctrl_idx, 0, 0));
                            let offset = self.find_char_at_x_on_line(sec, 0, cell_ctx, range, preferred_x)
                                .unwrap_or(0);
                            return Ok((sec, 0, offset, cell_ctx));
                        }
                    }
                    return Ok((sec, 0, 0, Some((target_para, ctrl_idx, 0, 0))));
                } else {
                    // ArrowUp → 마지막 셀의 마지막 줄
                    let last_cell_idx = table.cells.len().saturating_sub(1);
                    if let Some(last_cell) = table.cells.get(last_cell_idx) {
                        let last_cpi = last_cell.paragraphs.len().saturating_sub(1);
                        if let Some(cell_para) = last_cell.paragraphs.get(last_cpi) {
                            let last_line = if cell_para.line_segs.is_empty() { 0 } else { cell_para.line_segs.len() - 1 };
                            let range = Self::get_line_char_range(cell_para, last_line);
                            let cell_ctx = Some((target_para, ctrl_idx, last_cell_idx, last_cpi));
                            let offset = self.find_char_at_x_on_line(sec, last_cpi, cell_ctx, range, preferred_x)
                                .unwrap_or(navigable_text_len(cell_para));
                            return Ok((sec, last_cpi, offset, cell_ctx));
                        }
                    }
                    let last_cell_idx = table.cells.len().saturating_sub(1);
                    return Ok((sec, 0, 0, Some((target_para, ctrl_idx, last_cell_idx, 0))));
                }
            }
        }

        // 일반 문단
        let target_line = if delta > 0 { 0 } else {
            if para_ref.line_segs.is_empty() { 0 } else { para_ref.line_segs.len() - 1 }
        };
        let range = Self::get_line_char_range(para_ref, target_line);
        let offset = self.find_char_at_x_on_line(sec, target_para, None, range, preferred_x)
            .unwrap_or(if delta > 0 { 0 } else { navigable_text_len(para_ref) });
        Ok((sec, target_para, offset, None))
    }

    /// 셀 내부 경계를 넘어 이동한다 (셀 문단 경계, 셀 간 이동, 표 탈출).
    pub(crate) fn handle_cell_boundary(
        &self,
        sec: usize,
        _para: usize,
        _char_offset: usize,
        delta: i32,
        preferred_x: f64,
        (ppi, ci, cei, cpi): (usize, usize, usize, usize),
    ) -> Result<(usize, usize, usize, Option<(usize, usize, usize, usize)>), HwpError> {
        let table_para = self.document.sections.get(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?
            .paragraphs.get(ppi)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        // 글상자인 경우: 문단 간 이동만, 셀 이동 없이 경계에서 본문 탈출
        if let Some(Control::Shape(shape)) = table_para.controls.get(ci) {
            if let Some(text_box) = get_textbox_from_shape(shape) {
                // 오버플로우 타겟: 소스의 오버플로우 문단 수 사용
                // 오버플로우 소스: 렌더 문단 수(overflow_start)만 사용
                let effective_para_count = self.overflow_para_count(sec, ppi, ci)
                    .or_else(|| self.source_rendered_para_count(sec, ppi, ci))
                    .unwrap_or(text_box.paragraphs.len());

                if delta > 0 && cpi + 1 < effective_para_count {
                    let next_cpi = cpi + 1;
                    let cell_ctx = Some((ppi, ci, 0, next_cpi));
                    let next_para = self.resolve_paragraph(sec, next_cpi, cell_ctx)?;
                    let range = Self::get_line_char_range(next_para, 0);
                    let offset = self.find_char_at_x_on_line(sec, next_cpi, cell_ctx, range, preferred_x)
                        .unwrap_or(0);
                    return Ok((sec, next_cpi, offset, cell_ctx));
                }
                if delta < 0 && cpi > 0 {
                    let prev_cpi = cpi - 1;
                    let cell_ctx = Some((ppi, ci, 0, prev_cpi));
                    let prev_para = self.resolve_paragraph(sec, prev_cpi, cell_ctx)?;
                    let last_line = if prev_para.line_segs.is_empty() { 0 } else { prev_para.line_segs.len() - 1 };
                    let range = Self::get_line_char_range(prev_para, last_line);
                    let offset = self.find_char_at_x_on_line(sec, prev_cpi, cell_ctx, range, preferred_x)
                        .unwrap_or(navigable_text_len(prev_para));
                    return Ok((sec, prev_cpi, offset, cell_ctx));
                }
                // 글상자 경계 → 본문 탈출
                return self.exit_table_vertical(sec, ppi, delta, preferred_x);
            }
        }

        let table = match table_para.controls.get(ci) {
            Some(Control::Table(t)) => t,
            _ => return Err(HwpError::RenderError("표 컨트롤이 아닙니다".to_string())),
        };

        let cell = table.cells.get(cei)
            .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;

        // 1. 셀 내 다른 문단으로 이동 시도
        if delta > 0 && cpi + 1 < cell.paragraphs.len() {
            let next_cpi = cpi + 1;
            let next_para = &cell.paragraphs[next_cpi];
            let range = Self::get_line_char_range(next_para, 0);
            let cell_ctx = Some((ppi, ci, cei, next_cpi));
            let offset = self.find_char_at_x_on_line(sec, next_cpi, cell_ctx, range, preferred_x)
                .unwrap_or(0);
            return Ok((sec, next_cpi, offset, cell_ctx));
        }
        if delta < 0 && cpi > 0 {
            let prev_cpi = cpi - 1;
            let prev_para = &cell.paragraphs[prev_cpi];
            let last_line = if prev_para.line_segs.is_empty() { 0 } else { prev_para.line_segs.len() - 1 };
            let range = Self::get_line_char_range(prev_para, last_line);
            let cell_ctx = Some((ppi, ci, cei, prev_cpi));
            let offset = self.find_char_at_x_on_line(sec, prev_cpi, cell_ctx, range, preferred_x)
                .unwrap_or(navigable_text_len(prev_para));
            return Ok((sec, prev_cpi, offset, cell_ctx));
        }

        // 2. 위/아래 셀로 이동 시도
        let target_row = if delta > 0 {
            (cell.row + cell.row_span) as i32
        } else {
            cell.row as i32 - 1
        };

        if target_row >= 0 && (target_row as u16) < table.row_count {
            if let Some(target_cell_idx) = table.cell_index_at(target_row as u16, cell.col) {
                let target_cell = &table.cells[target_cell_idx];
                let (target_cpi, target_line) = if delta > 0 {
                    (0, 0)
                } else {
                    let last_cpi = target_cell.paragraphs.len().saturating_sub(1);
                    let last_line = if let Some(p) = target_cell.paragraphs.get(last_cpi) {
                        if p.line_segs.is_empty() { 0 } else { p.line_segs.len() - 1 }
                    } else { 0 };
                    (last_cpi, last_line)
                };

                if let Some(target_para) = target_cell.paragraphs.get(target_cpi) {
                    let range = Self::get_line_char_range(target_para, target_line);
                    let cell_ctx = Some((ppi, ci, target_cell_idx, target_cpi));
                    let offset = self.find_char_at_x_on_line(sec, target_cpi, cell_ctx, range, preferred_x)
                        .unwrap_or(0);
                    return Ok((sec, target_cpi, offset, cell_ctx));
                }
            }
        }

        // 3. 표 탈출
        self.exit_table_vertical(sec, ppi, delta, preferred_x)
    }

    /// 표 밖으로 나가기 (위/아래 방향).
    pub(crate) fn exit_table_vertical(
        &self,
        sec: usize,
        ppi: usize,
        delta: i32,
        preferred_x: f64,
    ) -> Result<(usize, usize, usize, Option<(usize, usize, usize, usize)>), HwpError> {
        let section = &self.document.sections[sec];
        if delta > 0 {
            let next = ppi + 1;
            if next < section.paragraphs.len() {
                return self.enter_paragraph(sec, next, delta, preferred_x);
            }
            // 구역 끝
            if sec + 1 < self.document.sections.len() {
                return self.enter_paragraph(sec + 1, 0, delta, preferred_x);
            }
            // 문서 끝 — 표 마지막 위치 유지
            Ok((sec, 0, 0, None))
        } else {
            if ppi > 0 {
                return self.enter_paragraph(sec, ppi - 1, delta, preferred_x);
            }
            // 구역 시작
            if sec > 0 {
                let prev_sec = sec - 1;
                let prev_count = self.document.sections[prev_sec].paragraphs.len();
                if prev_count > 0 {
                    return self.enter_paragraph(prev_sec, prev_count - 1, delta, preferred_x);
                }
            }
            // 문서 시작 — 이동 안 함
            Ok((sec, 0, 0, None))
        }
    }

    // ─── 다단 칼럼 경계 헬퍼 ────────────────────────────────

    /// 문단이 속한 칼럼의 영역(x, width)을 반환한다.
    /// 단일 단이면 None.
    pub(crate) fn get_column_area_for_paragraph(
        &self,
        sec: usize,
        para: usize,
    ) -> Option<(u16, f64, f64)> {
        let col_idx = self.para_column_map
            .get(sec)
            .and_then(|m| m.get(para))
            .copied()
            .unwrap_or(0);

        // 해당 문단이 포함된 페이지 찾기
        let pages = self.find_pages_for_paragraph(sec, para).ok()?;
        let first_page = *pages.first()?;
        let (page_content, _, _) = self.find_page(first_page).ok()?;
        let areas = &page_content.layout.column_areas;
        if areas.len() <= 1 {
            return None; // 단일 단
        }
        let area = areas.get(col_idx as usize)?;
        Some((col_idx, area.x, area.width))
    }

    /// preferredX를 현재 칼럼 좌표계에서 대상 칼럼 좌표계로 변환한다.
    /// 두 칼럼이 같거나 단일 단이면 원래 값을 그대로 반환한다.
    pub(crate) fn transform_preferred_x_across_columns(
        &self,
        sec: usize,
        from_para: usize,
        to_para: usize,
        preferred_x: f64,
    ) -> f64 {
        let from_col = self.get_column_area_for_paragraph(sec, from_para);
        let to_col = self.get_column_area_for_paragraph(sec, to_para);
        match (from_col, to_col) {
            (Some((fc, fx, _)), Some((tc, tx, _))) if fc != tc => {
                // 칼럼 상대 좌표 보존: (preferred_x - from_area.x) + to_area.x
                let relative_x = preferred_x - fx;
                tx + relative_x
            }
            _ => preferred_x,
        }
    }

    /// 특정 문단의 특정 줄이 속한 칼럼의 영역(col_idx, x, width)을 반환한다.
    /// 페이지네이션 결과의 PartialParagraph에서 start_line/end_line을 검사하여
    /// 해당 줄이 어떤 칼럼에 배치되었는지 판별한다.
    /// 단일 단이면 None.
    pub(crate) fn find_column_for_line(
        &self,
        sec: usize,
        para: usize,
        line_index: usize,
    ) -> Option<(u16, f64, f64)> {
        use crate::renderer::pagination::PageItem;

        let pages = self.find_pages_for_paragraph(sec, para).ok()?;
        for &page_num in &pages {
            let (page_content, _, _) = self.find_page(page_num).ok()?;
            let areas = &page_content.layout.column_areas;
            if areas.len() <= 1 {
                return None; // 단일 단
            }
            for col in &page_content.column_contents {
                for item in &col.items {
                    match item {
                        PageItem::FullParagraph { para_index } if *para_index == para => {
                            // 문단 전체가 이 칼럼에 있음 — 모든 줄이 이 칼럼
                            let area = areas.get(col.column_index as usize)?;
                            return Some((col.column_index, area.x, area.width));
                        }
                        PageItem::PartialParagraph { para_index, start_line, end_line }
                            if *para_index == para && line_index >= *start_line && line_index < *end_line =>
                        {
                            let area = areas.get(col.column_index as usize)?;
                            return Some((col.column_index, area.x, area.width));
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    // ─── Phase 4 네이티브: Selection API ─────────────────────

    /// 선택 영역의 줄별 사각형을 계산한다 (본문/셀 공통).
    ///
    /// cell_ctx: Some((ppi, ci, cei)) 면 셀 내부, None 이면 본문.
    /// start/end_para_idx: 셀 내부일 때는 cellParaIndex.
    pub(crate) fn get_selection_rects_native(
        &self,
        section_idx: usize,
        start_para_idx: usize,
        start_char_offset: usize,
        end_para_idx: usize,
        end_char_offset: usize,
        cell_ctx: Option<(usize, usize, usize)>,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        // ── 커서 위치를 pre-built tree에서 직접 찾는 헬퍼 ──
        #[derive(Clone)]
        struct CursorHit { page: u32, x: f64, y: f64, h: f64 }

        #[derive(Clone, Copy)]
        enum CursorBias {
            Leading,
            Trailing,
        }

        fn cursor_score(offset: usize, char_start: usize, char_count: usize, bias: CursorBias) -> u8 {
            let char_end = char_start + char_count;
            match bias {
                CursorBias::Leading if offset == char_start => 0,
                CursorBias::Leading if offset < char_end => 1,
                CursorBias::Leading => 2,
                CursorBias::Trailing if offset == char_end => 0,
                CursorBias::Trailing if offset > char_start => 1,
                CursorBias::Trailing => 2,
            }
        }

        fn update_best_cursor(best: &mut Option<(u8, CursorHit)>, score: u8, hit: CursorHit) {
            if best.as_ref().map_or(true, |(best_score, _)| score < *best_score) {
                *best = Some((score, hit));
            }
        }

        fn find_body_cursor(
            node: &RenderNode, sec: usize, para: usize,
            offset: usize, page: u32, bias: CursorBias,
        ) -> Option<CursorHit> {
            fn visit(
                node: &RenderNode,
                sec: usize,
                para: usize,
                offset: usize,
                page: u32,
                bias: CursorBias,
                best: &mut Option<(u8, CursorHit)>,
            ) {
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    if tr.section_index == Some(sec)
                        && tr.para_index == Some(para)
                        && tr.cell_context.is_none()
                    {
                        let cs = tr.char_start.unwrap_or(0);
                        let cc = tr.text.chars().count();
                        if offset >= cs && offset <= cs + cc {
                            let pos = compute_char_positions(&tr.text, &tr.style);
                            let lo = offset - cs;
                            let xr = if lo < pos.len() { pos[lo] }
                                     else if !pos.is_empty() { *pos.last().unwrap() }
                                     else { 0.0 };
                            update_best_cursor(
                                best,
                                cursor_score(offset, cs, cc, bias),
                                CursorHit {
                                    page,
                                    x: node.bbox.x + xr,
                                    y: node.bbox.y,
                                    h: node.bbox.height,
                                },
                            );
                        }
                    }
                }
                for child in &node.children {
                    visit(child, sec, para, offset, page, bias, best);
                }
            }

            let mut best = None;
            visit(node, sec, para, offset, page, bias, &mut best);
            best.map(|(_, hit)| hit)
        }

        fn find_cell_cursor(
            node: &RenderNode, ppi: usize, ci: usize, cei: usize,
            cpi: usize, offset: usize, page: u32, bias: CursorBias,
        ) -> Option<CursorHit> {
            fn visit(
                node: &RenderNode,
                ppi: usize,
                ci: usize,
                cei: usize,
                cpi: usize,
                offset: usize,
                page: u32,
                bias: CursorBias,
                best: &mut Option<(u8, CursorHit)>,
            ) {
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    let matches_cell = tr.cell_context.as_ref().map_or(false, |ctx| {
                        ctx.path.first().map_or(false, |entry| {
                            ctx.parent_para_index == ppi
                                && entry.control_index == ci
                                && entry.cell_index == cei
                                && entry.cell_para_index == cpi
                        })
                    });
                    if matches_cell {
                        let cs = tr.char_start.unwrap_or(0);
                        let cc = tr.text.chars().count();
                        if offset >= cs && offset <= cs + cc {
                            let pos = compute_char_positions(&tr.text, &tr.style);
                            let lo = offset - cs;
                            let xr = if lo < pos.len() { pos[lo] }
                                     else if !pos.is_empty() { *pos.last().unwrap() }
                                     else { 0.0 };
                            update_best_cursor(
                                best,
                                cursor_score(offset, cs, cc, bias),
                                CursorHit {
                                    page,
                                    x: node.bbox.x + xr,
                                    y: node.bbox.y,
                                    h: node.bbox.height,
                                },
                            );
                        }
                    }
                }
                for child in &node.children {
                    visit(child, ppi, ci, cei, cpi, offset, page, bias, best);
                }
            }

            let mut best = None;
            visit(node, ppi, ci, cei, cpi, offset, page, bias, &mut best);
            best.map(|(_, hit)| hit)
        }

        // ── 페이지별 렌더 트리 캐시 (최대 2페이지) ──
        let mut tree_cache: Vec<(u32, crate::renderer::render_tree::PageRenderTree)> = Vec::new();

        // 선택 범위에 관련된 페이지 번호 수집 (중복 제거)
        let lookup_para = if let Some((ppi, _, _)) = cell_ctx { ppi } else { start_para_idx };
        let page_nums = self.find_pages_for_paragraph(section_idx, lookup_para)?;
        // 끝 문단이 다른 페이지에 있을 수 있으므로 추가
        if cell_ctx.is_none() && end_para_idx != start_para_idx {
            if let Ok(end_pages) = self.find_pages_for_paragraph(section_idx, end_para_idx) {
                for &p in &end_pages {
                    if !page_nums.contains(&p) {
                        // page_nums에 없는 페이지만 추가 (tree_cache에서 처리)
                        let _ = p; // 아래에서 on-demand로 빌드
                    }
                }
            }
        }
        // 주요 페이지 트리 미리 빌드
        for &pn in &page_nums {
            tree_cache.push((pn, self.build_page_tree(pn)?));
        }

        // 캐시에서 트리 참조를 가져오거나, 없으면 빌드 후 추가
        macro_rules! get_tree {
            ($page:expr) => {{
                let pg = $page;
                if !tree_cache.iter().any(|(p, _)| *p == pg) {
                    tree_cache.push((pg, self.build_page_tree(pg)?));
                }
                &tree_cache.iter().find(|(p, _)| *p == pg).unwrap().1
            }};
        }

        // 페이지에서 커서 위치 찾기 (캐시된 트리 사용)
        macro_rules! find_cursor {
            ($para_idx:expr, $offset:expr, $bias:expr) => {{
                let mut result: Option<CursorHit> = None;
                for (pn, tree) in tree_cache.iter() {
                    let hit = if let Some((ppi, ci, cei)) = cell_ctx {
                        find_cell_cursor(&tree.root, ppi, ci, cei, $para_idx, $offset, *pn, $bias)
                    } else {
                        find_body_cursor(&tree.root, section_idx, $para_idx, $offset, *pn, $bias)
                    };
                    if hit.is_some() { result = hit; break; }
                }
                result
            }};
        }

        // ── 단 영역 조회 헬퍼 ──
        let find_column_area = |page: u32, rx: f64| -> (f64, f64) {
            self.find_page(page)
                .map(|(pc, _, _)| {
                    let areas = &pc.layout.column_areas;
                    areas.iter()
                        .find(|ca| rx >= ca.x - 2.0 && rx <= ca.x + ca.width + 2.0)
                        .or_else(|| {
                            areas.iter().min_by(|a, b| {
                                let da = (rx - (a.x + a.width / 2.0)).abs();
                                let db = (rx - (b.x + b.width / 2.0)).abs();
                                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                            })
                        })
                        .map(|ca| (ca.x, ca.x + ca.width))
                        .unwrap_or((0.0, 0.0))
                })
                .unwrap_or((0.0, 0.0))
        };

        // ── 메인 루프 ──
        let mut rects: Vec<String> = Vec::new();

        for para_idx in start_para_idx..=end_para_idx {
            let para = if let Some((ppi, ci, cei)) = cell_ctx {
                self.get_cell_paragraph_ref(section_idx, ppi, ci, cei, para_idx)
                    .ok_or_else(|| HwpError::RenderError(format!(
                        "셀 문단 참조 실패: sec={} ppi={} ci={} cei={} cpi={}",
                        section_idx, ppi, ci, cei, para_idx
                    )))?
            } else {
                self.document.sections.get(section_idx)
                    .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
                    .paragraphs.get(para_idx)
                    .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)))?
            };

            let char_count = navigable_text_len(para);
            let line_count = Self::build_line_char_starts(para).len().max(1);

            let sel_start = if para_idx == start_para_idx { start_char_offset } else { 0 };
            let sel_end = if para_idx == end_para_idx { end_char_offset } else { char_count };
            if sel_start >= sel_end { continue; }

            // 본문 문단이 다른 페이지에 있을 수 있으므로 트리 캐시에 추가
            if cell_ctx.is_none() {
                if let Ok(pp) = self.find_pages_for_paragraph(section_idx, para_idx) {
                    for &pn in &pp {
                        if !tree_cache.iter().any(|(p, _)| *p == pn) {
                            tree_cache.push((pn, self.build_page_tree(pn)?));
                        }
                    }
                }
            }

            for line_idx in 0..line_count {
                let (line_char_start, line_char_end) = Self::get_line_char_range(para, line_idx);
                let range_start = sel_start.max(line_char_start);
                let range_end = sel_end.min(line_char_end);
                if range_start >= range_end { continue; }

                let left_hit = find_cursor!(para_idx, range_start, CursorBias::Leading);
                // range_end가 줄바꿈 등 비렌더링 문자 위치이면 한 칸 앞으로 재시도
                let right_hit = find_cursor!(para_idx, range_end, CursorBias::Trailing)
                    .or_else(|| if range_end > range_start { find_cursor!(para_idx, range_end - 1, CursorBias::Trailing) } else { None });

                if let (Some(lh), Some(rh)) = (left_hit, right_hit) {
                    let partial_start = range_start > line_char_start;

                    let selection_continues = cell_ctx.is_none() && (
                        (range_end < sel_end) ||
                        (para_idx < end_para_idx && range_end == sel_end) ||
                        // 같은 문단 내 강제 줄바꿈: 줄 끝까지 선택되고 다음 줄 시작이 sel_end이면 확장
                        (range_end == sel_end && range_end >= line_char_end && line_idx + 1 < line_count)
                    );

                    let (area_left, area_right) = if cell_ctx.is_none() {
                        find_column_area(rh.page, rh.x)
                    } else {
                        (0.0, 0.0)
                    };

                    // y/h는 항상 left_hit 기준 (right_hit가 다음 줄에 있을 수 있음)
                    let (page_idx, rect_x, rect_y, rect_h) = if !partial_start && cell_ctx.is_none() {
                        (lh.page, area_left, lh.y, lh.h)
                    } else {
                        (lh.page, lh.x, lh.y, lh.h)
                    };

                    let width = if selection_continues {
                        (area_right - rect_x).max(0.0)
                    } else if !partial_start && cell_ctx.is_none() {
                        (rh.x - rect_x).max(0.0)
                    } else {
                        (rh.x - lh.x).abs()
                    };

                    if width > 0.01 {
                        rects.push(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"width\":{:.1},\"height\":{:.1}}}",
                            page_idx, rect_x, rect_y, width, rect_h
                        ));
                    }
                }
            }
        }

        Ok(format!("[{}]", rects.join(",")))
    }

}

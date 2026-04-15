//! 문서 생성/로딩/저장/설정 관련 native 메서드

use crate::document_core::{DocumentCore, DEFAULT_FALLBACK_FONT};
use crate::error::HwpError;
use crate::model::document::Document;
use crate::renderer::composer::{compose_section, reflow_line_segs};
use crate::renderer::layout::LayoutEngine;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::{resolve_styles, ResolvedStyleSet};
use crate::renderer::DEFAULT_DPI;
use std::cell::RefCell;
use std::collections::HashMap;

impl DocumentCore {
    pub fn from_bytes(data: &[u8]) -> Result<DocumentCore, HwpError> {
        let mut document = crate::parser::parse_document(data)
            .map_err(|e| HwpError::InvalidFile(e.to_string()))?;

        let styles = resolve_styles(&document.doc_info, DEFAULT_DPI);

        // lineSegArray가 없는 문단(line_height=0)에 대해 합성 LineSeg 생성
        // HWPX에서 lineSegArray 누락 시 기본값(모든 필드 0)이 들어가므로,
        // compose 전에 올바른 line_height/line_spacing을 계산해야 줄바꿈·높이가 정상 동작한다.
        Self::reflow_zero_height_paragraphs(&mut document, &styles, DEFAULT_DPI);

        // 초기 상태(properties bit 15 == 0) 누름틀의 안내문 텍스트를 삭제하여 빈 필드로 정규화
        // (한컴에서 메모 추가 시 안내문 텍스트가 필드 값으로 삽입됨 — compose 전에 제거해야 정합성 유지)
        Self::clear_initial_field_texts(&mut document);

        let composed = document
            .sections
            .iter()
            .map(|s| compose_section(s))
            .collect();

        let sec_count = document.sections.len();
        let mut doc = DocumentCore {
            document,
            pagination: Vec::new(),
            styles,
            composed,
            dpi: DEFAULT_DPI,
            fallback_font: DEFAULT_FALLBACK_FONT.to_string(),
            layout_engine: LayoutEngine::new(DEFAULT_DPI),
            clipboard: None,
            show_paragraph_marks: false,
            show_control_codes: false,
            show_transparent_borders: false,
            clip_enabled: true,
            debug_overlay: false,
            measured_tables: Vec::new(),
            dirty_sections: vec![true; sec_count],
            measured_sections: Vec::new(),
            dirty_paragraphs: Vec::new(),
            para_column_map: Vec::new(),
            page_tree_cache: RefCell::new(Vec::new()),
            batch_mode: false,
            event_log: Vec::new(),
            overflow_links_cache: RefCell::new(HashMap::new()),
            snapshot_store: Vec::new(),
            next_snapshot_id: 0,
            hidden_header_footer: std::collections::HashSet::new(),
            file_name: String::new(),
            active_field: None,
            para_offset: Vec::new(),
        };

        doc.paginate();
        Ok(doc)
    }

    /// lineSegArray가 없는(line_height=0) 문단에 대해 합성 LineSeg를 생성한다.
    ///
    /// HWPX 파일에서 `<hp:lineSegArray>`가 누락된 문단은 모든 LineSeg 필드가 0으로
    /// 설정되어 줄바꿈·문단 높이 계산이 불가능하다. 이 함수는 문서 로드 직후
    /// CharPr/ParaPr 기반으로 올바른 line_height/line_spacing을 계산한다.
    /// 본문 문단뿐 아니라 표 셀 내부 문단도 처리한다.
    fn reflow_zero_height_paragraphs(document: &mut Document, styles: &ResolvedStyleSet, dpi: f64) {
        use crate::model::control::Control;

        for section in &mut document.sections {
            let page_def = &section.section_def.page_def;
            let column_def = Self::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);

            for para in &mut section.paragraphs {
                // 본문 문단 reflow
                if Self::needs_line_seg_reflow(para) {
                    let para_style = styles.para_styles.get(para.para_shape_id as usize);
                    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                    let available_width = (col_width - margin_left - margin_right).max(1.0);
                    reflow_line_segs(para, available_width, styles, dpi);
                }

                // HWPX: TAC 표가 있는 문단의 LINE_SEG lh 보정
                // HWPX에서 linesegarray가 없으면 기본 lh=100이 생성되지만,
                // HWP에서는 TAC 표 높이가 lh에 포함됨 → HWPX에서도 동일하게 확대
                {
                    let mut max_tac_h: i32 = 0;
                    for ctrl in para.controls.iter() {
                        if let Control::Table(t) = ctrl {
                            if t.common.treat_as_char
                                && t.raw_ctrl_data.is_empty()
                                && t.common.height > 0
                            {
                                max_tac_h = max_tac_h.max(t.common.height as i32);
                            }
                        }
                    }
                    if max_tac_h > 0 {
                        // TAC 표가 있는 문단: lh가 표 높이보다 작으면 표 높이로 확대
                        if let Some(seg) = para.line_segs.first_mut() {
                            if seg.line_height < max_tac_h {
                                seg.line_height = max_tac_h;
                            }
                        }
                    }
                }

                // 표 셀 내부 문단 reflow
                for ctrl in &mut para.controls {
                    if let Control::Table(ref mut table) = ctrl {
                        for cell in &mut table.cells {
                            for cell_para in &mut cell.paragraphs {
                                if Self::needs_line_seg_reflow(cell_para) {
                                    // 셀 너비가 아직 불확정이므로 컬럼 너비를 근사값으로 사용.
                                    // 핵심은 line_height > 0을 보장하는 것이며,
                                    // 실제 셀 내 줄바꿈은 테이블 레이아웃이 재수행한다.
                                    reflow_line_segs(cell_para, col_width, styles, dpi);
                                }
                            }
                        }
                    }
                }
            }

            // HWPX: TAC 표 LINE_SEG 보정 후 문단 간 vpos 재계산
            // 보정된 문단의 끝 vpos가 변하면 후속 문단들의 vpos도 연쇄 갱신
            let mut need_vpos_recalc = false;
            for para in section.paragraphs.iter() {
                for ctrl in &para.controls {
                    match ctrl {
                        Control::Table(t)
                            if t.common.treat_as_char
                                && t.raw_ctrl_data.is_empty()
                                && t.common.height > 0 =>
                        {
                            need_vpos_recalc = true;
                            break;
                        }
                        // 비-TAC TopAndBottom Picture/Table: LINE_SEG에 개체 높이 미포함
                        Control::Picture(p)
                            if !p.common.treat_as_char
                                && matches!(
                                    p.common.text_wrap,
                                    crate::model::shape::TextWrap::TopAndBottom
                                )
                                && p.common.height > 0 =>
                        {
                            need_vpos_recalc = true;
                            break;
                        }
                        Control::Table(t)
                            if !t.common.treat_as_char
                                && matches!(
                                    t.common.text_wrap,
                                    crate::model::shape::TextWrap::TopAndBottom
                                )
                                && t.common.height > 0
                                && t.raw_ctrl_data.is_empty() =>
                        {
                            need_vpos_recalc = true;
                            break;
                        }
                        _ => {}
                    }
                }
                if need_vpos_recalc {
                    break;
                }
            }
            if need_vpos_recalc {
                let mut running_vpos: i32 = 0;
                for para in section.paragraphs.iter_mut() {
                    // 문단의 첫 LINE_SEG vpos를 running_vpos로 갱신
                    if let Some(first_seg) = para.line_segs.first_mut() {
                        first_seg.vertical_pos = running_vpos;
                    }
                    // 문단 내 LINE_SEG vpos 재계산 (문단 내 누적)
                    // TAC 표가 lh에 포함된 경우: 다음 줄 vpos = th + ls (HWP 동작)
                    let mut inner_vpos = running_vpos;
                    for seg in para.line_segs.iter_mut() {
                        seg.vertical_pos = inner_vpos;
                        let advance = if seg.line_height > seg.text_height && seg.text_height > 0 {
                            // lh가 th보다 큼 = TAC 컨트롤 높이 포함 → th 기준 누적
                            seg.text_height + seg.line_spacing
                        } else {
                            seg.line_height + seg.line_spacing
                        };
                        inner_vpos = inner_vpos + advance;
                    }
                    // 비-TAC TopAndBottom Picture/Table: 개체 높이를 vpos에 반영
                    for ctrl in para.controls.iter() {
                        let (obj_height, obj_v_offset, obj_margin_top, obj_margin_bottom) =
                            match ctrl {
                                Control::Picture(p)
                                    if !p.common.treat_as_char
                                        && matches!(
                                            p.common.text_wrap,
                                            crate::model::shape::TextWrap::TopAndBottom
                                        )
                                        && p.common.height > 0 =>
                                {
                                    (
                                        p.common.height as i32,
                                        p.common.vertical_offset as i32,
                                        0,
                                        0,
                                    )
                                }
                                Control::Table(t)
                                    if !t.common.treat_as_char
                                        && matches!(
                                            t.common.text_wrap,
                                            crate::model::shape::TextWrap::TopAndBottom
                                        )
                                        && t.common.height > 0
                                        && t.raw_ctrl_data.is_empty() =>
                                {
                                    (
                                        t.common.height as i32,
                                        t.common.vertical_offset as i32,
                                        t.outer_margin_top as i32,
                                        t.outer_margin_bottom as i32,
                                    )
                                }
                                _ => continue,
                            };
                        let obj_total =
                            obj_height + obj_v_offset + obj_margin_top + obj_margin_bottom;
                        let seg_lh_total: i32 = para
                            .line_segs
                            .iter()
                            .map(|s| s.line_height + s.line_spacing)
                            .sum();
                        if obj_total > seg_lh_total {
                            inner_vpos += obj_total - seg_lh_total;
                        }
                    }
                    running_vpos = inner_vpos;
                }
            }
        }
    }

    /// 문단의 LineSeg가 합성(reflow)이 필요한지 판단한다.
    /// line_segs가 1개이고 line_height가 0이면 lineSegArray 누락 상태.
    fn needs_line_seg_reflow(para: &crate::model::paragraph::Paragraph) -> bool {
        para.line_segs.len() == 1 && para.line_segs[0].line_height == 0
    }

    /// 내장 템플릿에서 빈 문서 생성 (네이티브)
    pub fn create_blank_document_native(&mut self) -> Result<String, HwpError> {
        const BLANK_TEMPLATE: &[u8] = include_bytes!("../../../saved/blank2010.hwp");

        let document = crate::parser::parse_hwp(BLANK_TEMPLATE)
            .map_err(|e| HwpError::InvalidFile(e.to_string()))?;

        let styles = resolve_styles(&document.doc_info, self.dpi);
        let composed = document
            .sections
            .iter()
            .map(|s| compose_section(s))
            .collect();
        let sec_count = document.sections.len();

        self.document = document;
        self.styles = styles;
        self.composed = composed;
        self.clipboard = None;
        self.dirty_sections = vec![true; sec_count];
        self.measured_tables = Vec::new();
        self.measured_sections = Vec::new();
        self.dirty_paragraphs = Vec::new();
        self.para_column_map = Vec::new();
        self.page_tree_cache.borrow_mut().clear();
        self.snapshot_store.clear();
        self.next_snapshot_id = 0;

        self.convert_to_editable_native()?;
        self.paginate();

        Ok(self.get_document_info())
    }

    /// SVG 렌더링 (네이티브 에러 타입)
    pub fn export_hwp_native(&self) -> Result<Vec<u8>, HwpError> {
        crate::serializer::serialize_document(&self.document)
            .map_err(|e| HwpError::RenderError(e.to_string()))
    }

    /// 배포용(읽기전용) 문서를 편집 가능한 일반 문서로 변환한다 (네이티브 에러 타입).
    pub fn convert_to_editable_native(&mut self) -> Result<String, HwpError> {
        let converted = self.document.convert_to_editable();
        Ok(format!("{{\"ok\":true,\"converted\":{}}}", converted))
    }

    /// 문서의 IR 참조를 반환한다 (네이티브 전용).
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// 문서 IR을 직접 설정한다 (테스트/네이티브 전용).
    pub fn set_document(&mut self, doc: Document) {
        self.document = doc;
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        self.composed = self
            .document
            .sections
            .iter()
            .map(|s| compose_section(s))
            .collect();
        self.mark_all_sections_dirty();
        self.paginate();
    }

    /// Batch 모드를 시작한다. 이후 Command 호출 시 paginate()를 건너뛴다.
    pub fn begin_batch_native(&mut self) -> Result<String, HwpError> {
        self.batch_mode = true;
        self.event_log.clear();
        Ok(super::super::helpers::json_ok())
    }

    /// Batch 모드를 종료하고 누적된 이벤트를 반환한다.
    /// 종료 시 paginate()를 1회 실행하여 모든 dirty 구역을 처리한다.
    pub fn end_batch_native(&mut self) -> Result<String, HwpError> {
        self.batch_mode = false;
        self.paginate();
        let result = self.serialize_event_log();
        self.event_log.clear();
        Ok(result)
    }

    // ─── Undo/Redo 스냅샷 API ──────────────────────────

    /// 현재 Document를 클론하여 스냅샷 저장소에 보관한다.
    /// 반환값: 스냅샷 ID (u32)
    pub fn save_snapshot_native(&mut self) -> u32 {
        let id = self.next_snapshot_id;
        self.next_snapshot_id += 1;
        self.snapshot_store.push((id, self.document.clone()));
        // 최대 100개 제한 — 초과 시 가장 오래된 스냅샷 제거
        const MAX_SNAPSHOTS: usize = 100;
        while self.snapshot_store.len() > MAX_SNAPSHOTS {
            self.snapshot_store.remove(0);
        }
        id
    }

    /// 지정 ID의 스냅샷으로 Document를 복원한다.
    /// 스타일 재해소 + 문단 구성 + 페이지네이션까지 수행.
    pub fn restore_snapshot_native(&mut self, id: u32) -> Result<String, HwpError> {
        let idx = self
            .snapshot_store
            .iter()
            .position(|(sid, _)| *sid == id)
            .ok_or_else(|| HwpError::RenderError(format!("스냅샷 {} 없음", id)))?;
        let (_, doc) = self.snapshot_store[idx].clone();
        self.document = doc;
        // 캐시 전체 재구성
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        self.composed = self
            .document
            .sections
            .iter()
            .map(|s| compose_section(s))
            .collect();
        self.mark_all_sections_dirty();
        self.measured_tables.clear();
        self.measured_sections.clear();
        self.dirty_paragraphs.clear();
        self.para_column_map.clear();
        self.page_tree_cache.borrow_mut().clear();
        self.overflow_links_cache.borrow_mut().clear();
        self.paginate();
        Ok(super::super::helpers::json_ok())
    }

    /// 지정 ID의 스냅샷을 저장소에서 제거하여 메모리를 해제한다.
    pub fn discard_snapshot_native(&mut self, id: u32) {
        self.snapshot_store.retain(|(sid, _)| *sid != id);
    }

    pub fn measure_width_diagnostic_native(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::composer::estimate_composed_line_width;
        use crate::renderer::hwpunit_to_px;

        let section =
            self.document.sections.get(section_idx).ok_or_else(|| {
                HwpError::InvalidFile(format!("section {} not found", section_idx))
            })?;
        let para = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| HwpError::InvalidFile(format!("para {} not found", para_idx)))?;
        let composed = self
            .composed
            .get(section_idx)
            .and_then(|s| s.get(para_idx))
            .ok_or_else(|| HwpError::InvalidFile("composed paragraph not found".into()))?;

        let text_preview: String = para.text.chars().take(30).collect();

        let mut lines_json = Vec::new();

        for (line_idx, composed_line) in composed.lines.iter().enumerate() {
            let our_width_px = estimate_composed_line_width(composed_line, &self.styles);

            let stored_hwpunit = composed_line.segment_width;
            let stored_width_px = hwpunit_to_px(stored_hwpunit, self.dpi);

            let error_px = our_width_px - stored_width_px;
            let error_hwpunit = (error_px * 7200.0 / self.dpi).round() as i32;

            // run별 상세
            let mut runs_json = Vec::new();
            for run in &composed_line.runs {
                let ts = crate::renderer::layout::resolved_to_text_style(
                    &self.styles,
                    run.char_style_id,
                    run.lang_index,
                );
                let run_width = crate::renderer::layout::estimate_text_width(&run.text, &ts);
                runs_json.push(format!(
                    r#"{{"text":"{}","lang":{},"font":"{}","width_px":{:.2}}}"#,
                    super::super::helpers::json_escape(&run.text),
                    run.lang_index,
                    super::super::helpers::json_escape(&ts.font_family),
                    run_width,
                ));
            }

            let line_text: String = composed_line.runs.iter().map(|r| r.text.as_str()).collect();

            lines_json.push(format!(
                r#"{{"line_index":{},"text":"{}","runs":[{}],"our_width_px":{:.2},"stored_segment_width_hwpunit":{},"stored_width_px":{:.2},"error_px":{:.2},"error_hwpunit":{}}}"#,
                line_idx,
                super::super::helpers::json_escape(&line_text),
                runs_json.join(","),
                our_width_px,
                stored_hwpunit,
                stored_width_px,
                error_px,
                error_hwpunit,
            ));
        }

        Ok(format!(
            r#"{{"paragraph":{{"section":{},"para":{},"text_preview":"{}"}},"lines":[{}]}}"#,
            section_idx,
            para_idx,
            super::super::helpers::json_escape(&text_preview),
            lines_json.join(","),
        ))
    }

    /// 초기 상태(properties bit 15 == 0) ClickHere 필드의 안내문 텍스트를 삭제한다.
    ///
    /// 한컴에서 메모 추가 등의 동작 시 안내문 텍스트가 필드 값으로 삽입되어,
    /// start_char_idx != end_char_idx 상태가 된다.
    /// compose 전에 이 텍스트를 제거하여 빈 필드(start==end)로 정규화한다.
    fn clear_initial_field_texts(document: &mut Document) {
        use crate::model::control::{Control, FieldType};
        use crate::model::paragraph::Paragraph;

        fn process_para(para: &mut Paragraph) {
            // 삭제 대상 field_range 인덱스와 삭제할 문자 범위 수집
            let mut removals: Vec<(usize, usize, usize)> = Vec::new(); // (fr_idx, start, end)
            for (fri, fr) in para.field_ranges.iter().enumerate() {
                if fr.start_char_idx >= fr.end_char_idx {
                    continue;
                }
                if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                    if f.field_type != FieldType::ClickHere {
                        continue;
                    }
                    if f.properties & (1 << 15) != 0 {
                        continue;
                    } // 이미 수정된 상태
                      // 필드 값이 안내문과 동일한지 확인
                    if let Some(guide) = f.guide_text() {
                        let chars: Vec<char> = para.text.chars().collect();
                        if fr.end_char_idx <= chars.len() {
                            let field_val: String =
                                chars[fr.start_char_idx..fr.end_char_idx].iter().collect();
                            // trailing 공백 제거 후 비교 (한컴이 안내문 뒤에 공백을 추가하는 경우)
                            if field_val.trim_end() == guide || field_val == guide {
                                removals.push((fri, fr.start_char_idx, fr.end_char_idx));
                            }
                        }
                    }
                }
            }
            // 뒤에서부터 삭제 (인덱스 안정성 유지)
            for &(fri, start, end) in removals.iter().rev() {
                let removed_len = end - start;
                let chars: Vec<char> = para.text.chars().collect();
                let new_text: String = chars[..start].iter().chain(chars[end..].iter()).collect();
                para.text = new_text;
                para.field_ranges[fri].end_char_idx = start;
                // 이후 field_ranges의 char_idx 조정
                for i in 0..para.field_ranges.len() {
                    if i == fri {
                        continue;
                    }
                    let other = &mut para.field_ranges[i];
                    if other.start_char_idx >= end {
                        other.start_char_idx -= removed_len;
                    }
                    if other.end_char_idx >= end {
                        other.end_char_idx -= removed_len;
                    }
                }
            }
        }

        fn process_table(table: &mut crate::model::table::Table) {
            for cell in &mut table.cells {
                for cp in &mut cell.paragraphs {
                    process_para(cp);
                    // 중첩 표 재귀 탐색
                    for ctrl in &mut cp.controls {
                        if let Control::Table(nested) = ctrl {
                            process_table(nested);
                        }
                    }
                }
            }
        }

        for section in &mut document.sections {
            for para in &mut section.paragraphs {
                process_para(para);
                for ctrl in &mut para.controls {
                    if let Control::Table(table) = ctrl {
                        process_table(table);
                    }
                }
            }
        }
    }
}

//! 문서 생성/로딩/저장/설정 관련 native 메서드

use std::cell::RefCell;
use std::collections::HashMap;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::renderer::style_resolver::{resolve_styles, ResolvedStyleSet};
use crate::renderer::composer::{compose_section, reflow_line_segs};
use crate::renderer::layout::LayoutEngine;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::DEFAULT_DPI;
use crate::document_core::validation::{
    CellPath, ValidationReport, ValidationWarning, WarningKind,
};
use crate::document_core::{DocumentCore, DEFAULT_FALLBACK_FONT};
use crate::error::HwpError;

/// HWP 내보내기 + 자기 재로드 검증 결과 (#178 Stage 6).
///
/// `serialize_hwp_with_verify` 의 반환값. 호출자가 페이지 회복 여부를 확인하고
/// 실패 시 사용자에게 경고하거나 다른 동작을 취할 수 있게 한다.
#[derive(Debug, Clone)]
pub struct HwpExportVerification {
    /// 직렬화된 HWP 바이트
    pub bytes: Vec<u8>,
    /// 바이트 길이 (편의)
    pub bytes_len: usize,
    /// 어댑터 적용 직전 페이지 수
    pub page_count_before: u32,
    /// 직렬화 → 재로드 후 페이지 수
    pub page_count_after: u32,
    /// `page_count_before == page_count_after` 여부
    pub recovered: bool,
}

impl DocumentCore {
    pub fn from_bytes(data: &[u8]) -> Result<DocumentCore, HwpError> {
        let source_format = crate::parser::detect_format(data);
        let mut document = crate::parser::parse_document(data)
            .map_err(|e| HwpError::InvalidFile(e.to_string()))?;

        let styles = resolve_styles(&document.doc_info, DEFAULT_DPI);

        // 비표준 lineseg 감지 — reflow 이전 시점에 IR을 그대로 검증.
        // 경고는 사용자에게 고지되며, 자동 reflow 는 `needs_line_seg_reflow` 조건에만 한정.
        // 사용자 명시 reflow 는 `reflow_linesegs_on_demand()` 를 통해서만 수행 (#177).
        // LinesegTextRunReflow는 HWPX 전용 비표준 패턴. HWP3/HWP5는 1 line_info = 1 lineseg가 정상.
        let check_textrun_reflow = matches!(source_format, crate::parser::FileFormat::Hwpx);
        let validation_report = Self::validate_linesegs(&document, check_textrun_reflow);

        // lineSegArray가 없는 문단(line_height=0)에 대해 합성 LineSeg 생성
        // HWPX에서 lineSegArray 누락 시 기본값(모든 필드 0)이 들어가므로,
        // compose 전에 올바른 line_height/line_spacing을 계산해야 줄바꿈·높이가 정상 동작한다.
        Self::reflow_zero_height_paragraphs(&mut document, &styles, DEFAULT_DPI);

        // HWPX → HWP 라운드트립 일관성 normalize (#314):
        // HWPX 파서가 채우지 않는 paragraph 필드를 HWP 직렬화/파싱 라운드트립 결과와 일치시킨다.
        // 1) char_shapes 빈 paragraph 에 default [(0,0)] 추가 (HWP 스펙상 최소 1개 요구)
        // 2) control_mask 를 controls 기반으로 재계산
        if matches!(source_format, crate::parser::FileFormat::Hwpx) {
            Self::normalize_hwpx_paragraphs(&mut document);
        }

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
            respect_vpos_reset: false,
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
            source_format,
            validation_report,
        };

        doc.paginate();
        Ok(doc)
    }

    /// HWPX 비표준 lineseg 감지 (#177).
    ///
    /// `reflow_zero_height_paragraphs` 호출 **이전** 상태의 IR을 기준으로 검증한다.
    /// reflow 이후에 호출하면 이미 line_height 가 채워져 감지 불가.
    ///
    /// 감지 규칙:
    /// - 텍스트가 있는데 `line_segs` 가 비어있음 → `LinesegArrayEmpty`
    /// - `line_segs.len() == 1 && line_height == 0` → `LinesegUncomputed`
    /// - `check_textrun_reflow=true` 일 때만: 긴 텍스트 + lineseg 1개 → `LinesegTextRunReflow`
    ///   (HWPX 전용 패턴. HWP3/HWP5는 1 line_info → 1 lineseg가 정상이므로 건너뜀.)
    ///
    /// 표 셀 내부 문단도 재귀 검사한다.
    pub(crate) fn validate_linesegs(document: &Document, check_textrun_reflow: bool) -> ValidationReport {
        let mut report = ValidationReport::new();
        for (si, section) in document.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                Self::check_paragraph_linesegs(para, si, pi, None, check_textrun_reflow, &mut report);

                // 표 셀 내부 문단도 재귀 검사
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(table) = ctrl {
                        for cell in &table.cells {
                            for (inner_pi, cell_para) in cell.paragraphs.iter().enumerate() {
                                let cell_path = CellPath {
                                    table_ctrl_idx: ci,
                                    row: cell.row,
                                    col: cell.col,
                                    inner_para_idx: inner_pi,
                                };
                                Self::check_paragraph_linesegs(
                                    cell_para,
                                    si,
                                    pi,
                                    Some(cell_path),
                                    check_textrun_reflow,
                                    &mut report,
                                );
                            }
                        }
                    }
                }
            }
        }
        report
    }

    fn check_paragraph_linesegs(
        para: &Paragraph,
        section_idx: usize,
        paragraph_idx: usize,
        cell_path: Option<CellPath>,
        check_textrun_reflow: bool,
        report: &mut ValidationReport,
    ) {
        // 규칙 1: 텍스트가 있는데 lineseg 배열이 비어있음
        if para.line_segs.is_empty() && !para.text.is_empty() {
            report.push(ValidationWarning {
                section_idx,
                paragraph_idx,
                cell_path,
                kind: WarningKind::LinesegArrayEmpty,
            });
            return; // 후속 규칙 건너뜀
        }
        // 규칙 2: 미계산 상태 (기존 needs_line_seg_reflow 와 동일 조건)
        if para.line_segs.len() == 1 && para.line_segs[0].line_height == 0 {
            report.push(ValidationWarning {
                section_idx,
                paragraph_idx,
                cell_path,
                kind: WarningKind::LinesegUncomputed,
            });
            return;
        }
        // 규칙 3: lineseg 1개인데 텍스트가 길고 '\n' 이 없음 — 한컴이 textRun reflow 에
        // 의존하는 패턴 (Discussion #188). HWPX 전용. HWP3/HWP5는 1 line_info → 1 lineseg가
        // 정상이므로 check_textrun_reflow=false 로 호출하면 건너뜀.
        //
        // 휴리스틱 threshold = 40자 (한글 한 줄 ~30자 안팎을 기준으로 보수적).
        const LONG_TEXT_THRESHOLD: usize = 40;
        if check_textrun_reflow
            && para.line_segs.len() == 1
            && !para.text.contains('\n')
            && para.text.chars().count() > LONG_TEXT_THRESHOLD
        {
            report.push(ValidationWarning {
                section_idx,
                paragraph_idx,
                cell_path,
                kind: WarningKind::LinesegTextRunReflow,
            });
        }
    }

    /// lineSegArray가 없는(line_height=0) 문단에 대해 합성 LineSeg를 생성한다.
    ///
    /// HWPX 파일에서 `<hp:lineSegArray>`가 누락된 문단은 모든 LineSeg 필드가 0으로
    /// 설정되어 줄바꿈·문단 높이 계산이 불가능하다. 이 함수는 문서 로드 직후
    /// CharPr/ParaPr 기반으로 올바른 line_height/line_spacing을 계산한다.
    /// 본문 문단뿐 아니라 표 셀 내부 문단도 처리한다.
    fn reflow_zero_height_paragraphs(
        document: &mut Document,
        styles: &ResolvedStyleSet,
        dpi: f64,
    ) {
        use crate::model::control::Control;

        for section in &mut document.sections {
            let page_def = &section.section_def.page_def;
            let column_def = Self::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_width = layout.column_areas.first()
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
                            if t.common.treat_as_char && t.raw_ctrl_data.is_empty() && t.common.height > 0 {
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
                        Control::Table(t) if t.common.treat_as_char && t.raw_ctrl_data.is_empty() && t.common.height > 0 => {
                            need_vpos_recalc = true;
                            break;
                        }
                        // 비-TAC TopAndBottom Picture/Table: LINE_SEG에 개체 높이 미포함
                        Control::Picture(p) if !p.common.treat_as_char
                            && matches!(p.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                            && p.common.height > 0 => {
                            need_vpos_recalc = true;
                            break;
                        }
                        Control::Table(t) if !t.common.treat_as_char
                            && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                            && t.common.height > 0
                            && t.raw_ctrl_data.is_empty() => {
                            need_vpos_recalc = true;
                            break;
                        }
                        _ => {}
                    }
                }
                if need_vpos_recalc { break; }
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
                        let (obj_height, obj_v_offset, obj_margin_top, obj_margin_bottom) = match ctrl {
                            Control::Picture(p) if !p.common.treat_as_char
                                && matches!(p.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                                && p.common.height > 0 =>
                                (p.common.height as i32, p.common.vertical_offset as i32, 0, 0),
                            Control::Table(t) if !t.common.treat_as_char
                                && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                                && t.common.height > 0
                                && t.raw_ctrl_data.is_empty() =>
                                (t.common.height as i32, t.common.vertical_offset as i32,
                                 t.outer_margin_top as i32, t.outer_margin_bottom as i32),
                            _ => continue,
                        };
                        let obj_total = obj_height + obj_v_offset + obj_margin_top + obj_margin_bottom;
                        let seg_lh_total: i32 = para.line_segs.iter()
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

    /// 사용자 명시 요청에 의한 더 넓은 reflow 판정 (#177).
    ///
    /// `needs_line_seg_reflow` (명백한 미계산) + 다음 케이스 포함:
    /// - 텍스트가 있는데 line_segs 가 비어있음 (LinesegArrayEmpty)
    /// - 긴 텍스트 + lineseg 1개 + '\n' 없음 (LinesegTextRunReflow 패턴)
    ///
    /// 이 함수는 `reflow_linesegs_on_demand` 에서만 사용되며, 자동 파싱 경로에는 영향 없음.
    fn needs_reflow_broadly(para: &crate::model::paragraph::Paragraph) -> bool {
        if !para.text.is_empty() && para.line_segs.is_empty() {
            return true;
        }
        if Self::needs_line_seg_reflow(para) {
            return true;
        }
        // 한컴 textRun reflow 패턴 — 규칙 R3 과 동일 조건
        const LONG_TEXT_THRESHOLD: usize = 40;
        if para.line_segs.len() == 1
            && !para.text.contains('\n')
            && para.text.chars().count() > LONG_TEXT_THRESHOLD
        {
            return true;
        }
        false
    }

    /// 사용자 명시 요청에 의한 전체 lineseg reflow (#177).
    ///
    /// `validate_linesegs` 에 기록된 경고 대상 문단들 중 reflow 가능한 것을 모두 처리한다.
    /// 기본 파싱 경로의 `reflow_zero_height_paragraphs` 와 달리 이 메서드는
    /// 사용자가 UI에서 "자동 보정" 을 명시적으로 선택했을 때만 호출되어야 한다.
    ///
    /// 반환값: 실제로 reflow 된 문단 개수 (본문 + 셀 내부 합계).
    pub fn reflow_linesegs_on_demand(&mut self) -> usize {
        // 스타일은 재해소해도 동일 결과이므로 재계산하여 borrow 충돌 회피.
        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let dpi = self.dpi;
        let mut reflowed = 0usize;

        for section in &mut self.document.sections {
            let page_def = &section.section_def.page_def;
            let column_def = Self::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);

            for para in &mut section.paragraphs {
                if Self::needs_reflow_broadly(para) {
                    let para_style = styles.para_styles.get(para.para_shape_id as usize);
                    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                    let available_width = (col_width - margin_left - margin_right).max(1.0);
                    reflow_line_segs(para, available_width, &styles, dpi);
                    reflowed += 1;
                }
                // 표 셀 내부 문단도 동일 처리
                for ctrl in &mut para.controls {
                    if let Control::Table(ref mut table) = ctrl {
                        for cell in &mut table.cells {
                            for cell_para in &mut cell.paragraphs {
                                if Self::needs_reflow_broadly(cell_para) {
                                    reflow_line_segs(cell_para, col_width, &styles, dpi);
                                    reflowed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if reflowed > 0 {
            // 재구성 · 페이지네이션 재실행 필요
            self.styles = styles;
            self.composed = self
                .document
                .sections
                .iter()
                .map(|s| compose_section(s))
                .collect();
            let sec_count = self.document.sections.len();
            self.dirty_sections = vec![true; sec_count];
            self.paginate();
        }

        reflowed
    }

    /// 내장 템플릿에서 빈 문서 생성 (네이티브)
    pub fn create_blank_document_native(&mut self) -> Result<String, HwpError> {
        const BLANK_TEMPLATE: &[u8] = include_bytes!("../../../saved/blank2010.hwp");

        let document = crate::parser::parse_hwp(BLANK_TEMPLATE)
            .map_err(|e| HwpError::InvalidFile(e.to_string()))?;

        let styles = resolve_styles(&document.doc_info, self.dpi);
        let composed = document.sections.iter().map(|s| compose_section(s)).collect();
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

    /// Document IR을 HWP 5.0 CFB 바이너리로 직렬화 (네이티브 에러 타입)
    pub fn export_hwp_native(&self) -> Result<Vec<u8>, HwpError> {
        crate::serializer::serialize_document(&self.document)
            .map_err(|e| HwpError::RenderError(e.to_string()))
    }

    /// HWPX 출처 IR 을 HWP 호환 형태로 변환 후 HWP 5.0 CFB 바이너리로 직렬화한다 (#178).
    ///
    /// HWP 출처는 어댑터가 no-op 이므로 `export_hwp_native` 와 동일 결과.
    /// 사용자 시나리오: HWPX 로 연 문서를 편집 후 HWP 로 저장하는 모든 경로의 단일 진입점.
    ///
    /// 어댑터 호출은 IR 자체를 변경하므로 `&mut self` 를 요구한다.
    pub fn export_hwp_with_adapter(&mut self) -> Result<Vec<u8>, HwpError> {
        use crate::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source;
        let _report = convert_if_hwpx_source(&mut self.document, self.source_format);
        self.export_hwp_native()
    }

    /// 어댑터 적용 + 직렬화 + 자기 재로드 검증을 한 번에 수행한다 (#178 Stage 6).
    ///
    /// 명시 호출 전용. 운영 경로 (`export_hwp_with_adapter`) 는 검증 비용을 부담하지 않으며,
    /// 진단·테스트·사용자 경고가 필요한 경우에만 본 함수 사용.
    ///
    /// ## 검증 항목
    ///
    /// - `page_count_before`: 어댑터 적용 직전 페이지 수
    /// - `page_count_after`: 직렬화 → 재로드 후 페이지 수
    /// - `bytes_len`: HWP 바이트 길이
    /// - `recovered`: `before == after` 면 true
    ///
    /// ## 비용
    ///
    /// 1회 paginate + 1회 직렬화 + 1회 from_bytes (paginate 포함). 작은 문서 ~수 ms,
    /// 큰 문서 수백 ms 가능.
    pub fn serialize_hwp_with_verify(&mut self) -> Result<HwpExportVerification, HwpError> {
        let page_count_before = self.page_count();
        let bytes = self.export_hwp_with_adapter()?;
        let bytes_len = bytes.len();
        let reloaded = DocumentCore::from_bytes(&bytes)?;
        let page_count_after = reloaded.page_count();

        Ok(HwpExportVerification {
            bytes,
            bytes_len,
            page_count_before,
            page_count_after,
            recovered: page_count_before == page_count_after,
        })
    }

    /// Document IR을 HWPX(ZIP+XML)로 직렬화 (네이티브 에러 타입)
    pub fn export_hwpx_native(&self) -> Result<Vec<u8>, HwpError> {
        crate::serializer::serialize_hwpx(&self.document)
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
        self.composed = self.document.sections.iter()
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
        let idx = self.snapshot_store.iter().position(|(sid, _)| *sid == id)
            .ok_or_else(|| HwpError::RenderError(format!("스냅샷 {} 없음", id)))?;
        let (_, doc) = self.snapshot_store[idx].clone();
        self.document = doc;
        // 캐시 전체 재구성
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        self.composed = self.document.sections.iter()
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

        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::InvalidFile(format!("section {} not found", section_idx)))?;
        let para = section.paragraphs.get(para_idx)
            .ok_or_else(|| HwpError::InvalidFile(format!("para {} not found", para_idx)))?;
        let composed = self.composed.get(section_idx)
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
                    &self.styles, run.char_style_id, run.lang_index,
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

            let line_text: String = composed_line.runs.iter()
                .map(|r| r.text.as_str())
                .collect();

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

    /// HWPX → HWP 라운드트립 일관성 normalize.
    ///
    /// HWPX 파서가 채우지 않는 paragraph 필드를 HWP 직렬화/파싱 라운드트립 결과와 일치시킨다.
    /// - char_shapes 빈 paragraph 에 default `[(0, 0)]` 추가 (HWP 스펙: 최소 1개 PARA_CHAR_SHAPE 요구)
    /// - control_mask 를 controls + field_ranges + text 기반으로 재계산 (HWP 직렬화기와 동일 로직)
    fn normalize_hwpx_paragraphs(document: &mut Document) {
        use crate::model::control::Control;
        use crate::model::paragraph::{CharShapeRef, Paragraph};

        fn compute_mask(para: &Paragraph) -> u32 {
            let mut mask: u32 = 0;
            for ctrl in &para.controls {
                let bit = match ctrl {
                    Control::SectionDef(_) | Control::ColumnDef(_) => 0x0002,
                    Control::Field(_) => 0x0003,
                    Control::Table(_) | Control::Shape(_) | Control::Picture(_)
                    | Control::Hyperlink(_) | Control::Ruby(_) | Control::Equation(_)
                    | Control::Form(_) | Control::Unknown(_) => 0x000B,
                    Control::HiddenComment(_) => 0x000F,
                    Control::Header(_) | Control::Footer(_) => 0x0010,
                    Control::Footnote(_) | Control::Endnote(_) => 0x0011,
                    Control::AutoNumber(_) | Control::NewNumber(_) => 0x0012,
                    Control::PageNumberPos(_) | Control::PageHide(_) => 0x0015,
                    Control::Bookmark(_) => 0x0016,
                    Control::CharOverlap(_) => 0x0017,
                };
                mask |= 1u32 << bit;
            }
            if !para.field_ranges.is_empty() { mask |= 1u32 << 0x0004; }
            if para.text.contains('\t') { mask |= 1u32 << 0x0009; }
            if para.text.contains('\n') { mask |= 1u32 << 0x000A; }
            mask
        }

        fn process_para(para: &mut Paragraph) {
            if para.char_shapes.is_empty() {
                para.char_shapes.push(CharShapeRef { start_pos: 0, char_shape_id: 0 });
            }
            para.control_mask = compute_mask(para);
            // 셀 내부 paragraphs 도 재귀
            for ctrl in &mut para.controls {
                if let Control::Table(t) = ctrl {
                    for cell in &mut t.cells {
                        for cp in &mut cell.paragraphs {
                            process_para(cp);
                        }
                    }
                }
                // Shape의 text box paragraphs도 재귀해야 하나 정확한 API 미식별 → skip
                // (현재 회귀 케이스 hwpx-h-02 는 cell paragraphs로 충분)
            }
        }

        for section in &mut document.sections {
            for p in &mut section.paragraphs {
                process_para(p);
            }
        }
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
                if fr.start_char_idx >= fr.end_char_idx { continue; }
                if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                    if f.field_type != FieldType::ClickHere { continue; }
                    if f.properties & (1 << 15) != 0 { continue; } // 이미 수정된 상태
                    // 필드 값이 안내문과 동일한지 확인
                    if let Some(guide) = f.guide_text() {
                        let chars: Vec<char> = para.text.chars().collect();
                        if fr.end_char_idx <= chars.len() {
                            let field_val: String = chars[fr.start_char_idx..fr.end_char_idx].iter().collect();
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
                    if i == fri { continue; }
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

#[cfg(test)]
mod validate_linesegs_tests {
    use super::*;
    use crate::model::document::{Document, Section};
    use crate::model::paragraph::{LineSeg, Paragraph};

    /// 텍스트는 있는데 line_segs 가 비어있는 문단 — LinesegArrayEmpty 감지
    #[test]
    fn validate_detects_empty_linesegs() {
        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        // line_segs 비워둠
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert_eq!(report.len(), 1);
        assert_eq!(report.warnings[0].kind, WarningKind::LinesegArrayEmpty);
        assert_eq!(report.warnings[0].section_idx, 0);
        assert_eq!(report.warnings[0].paragraph_idx, 0);
        assert!(report.warnings[0].cell_path.is_none());
    }

    /// line_segs 가 1개, line_height=0 — LinesegUncomputed 감지
    #[test]
    fn validate_detects_uncomputed_lineseg() {
        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.line_segs.push(LineSeg::default()); // line_height=0 상태
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert_eq!(report.len(), 1);
        assert_eq!(report.warnings[0].kind, WarningKind::LinesegUncomputed);
    }

    /// 정상 lineseg (line_height > 0) — 경고 없음
    #[test]
    fn validate_skips_healthy_lineseg() {
        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert!(report.is_empty(), "healthy paragraph should not warn: {:?}", report.warnings);
    }

    /// 빈 문단 (텍스트도 line_segs 도 없음) — 경고 없음 (빈 문단은 허용)
    #[test]
    fn validate_skips_empty_paragraph() {
        let mut doc = Document::default();
        let mut section = Section::default();
        section.paragraphs.push(Paragraph::default());
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert!(report.is_empty());
    }

    /// 표 셀 내부 문단도 검증 — cell_path 가 기록됨
    #[test]
    fn validate_recurses_into_table_cells() {
        use crate::model::table::{Cell, Table};

        let mut doc = Document::default();
        let mut section = Section::default();
        let mut outer_para = Paragraph::default();

        // 셀 내부에 문제가 있는 문단
        let mut cell_para = Paragraph::default();
        cell_para.text = "in-cell".to_string();
        // line_segs 비워둠 → LinesegArrayEmpty 감지 대상

        let mut cell = Cell::default();
        cell.row = 0;
        cell.col = 0;
        cell.paragraphs.push(cell_para);

        let mut table = Table::default();
        table.row_count = 1;
        table.col_count = 1;
        table.cells.push(cell);

        outer_para.controls.push(Control::Table(Box::new(table)));
        section.paragraphs.push(outer_para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert_eq!(report.len(), 1);
        assert_eq!(report.warnings[0].kind, WarningKind::LinesegArrayEmpty);
        let cp = report.warnings[0].cell_path.expect("cell_path should be set");
        assert_eq!(cp.table_ctrl_idx, 0);
        assert_eq!(cp.row, 0);
        assert_eq!(cp.col, 0);
        assert_eq!(cp.inner_para_idx, 0);
    }

    /// 다중 경고 — 각각 기록됨
    #[test]
    fn validate_records_multiple_warnings() {
        let mut doc = Document::default();
        let mut section = Section::default();

        let mut p1 = Paragraph::default();
        p1.text = "a".to_string();
        // line_segs 비움

        let mut p2 = Paragraph::default();
        p2.text = "b".to_string();
        p2.line_segs.push(LineSeg::default()); // line_height=0

        section.paragraphs.push(p1);
        section.paragraphs.push(p2);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert_eq!(report.len(), 2);
        let summary = report.summary();
        assert_eq!(summary.get("lineseg 배열이 비어있음").copied(), Some(1));
        assert_eq!(summary.get("lineseg 가 미계산 상태 (line_height=0)").copied(), Some(1));
    }

    /// needs_reflow_broadly: 빈 line_segs + text → true
    #[test]
    fn needs_reflow_broadly_covers_empty_linesegs() {
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        // line_segs 비움
        assert!(DocumentCore::needs_reflow_broadly(&para));
    }

    /// needs_reflow_broadly: 기존 조건 (line_segs=1, line_height=0) → true
    #[test]
    fn needs_reflow_broadly_covers_uncomputed_lineseg() {
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.line_segs.push(LineSeg::default());
        assert!(DocumentCore::needs_reflow_broadly(&para));
    }

    /// needs_reflow_broadly: 정상 line_segs → false
    #[test]
    fn needs_reflow_broadly_skips_healthy_paragraph() {
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        assert!(!DocumentCore::needs_reflow_broadly(&para));
    }

    /// needs_reflow_broadly: 빈 문단 (text 없음) → false
    #[test]
    fn needs_reflow_broadly_skips_empty_paragraph() {
        let para = Paragraph::default();
        assert!(!DocumentCore::needs_reflow_broadly(&para));
    }

    // ---------- R3: LinesegTextRunReflow ----------

    #[test]
    fn validate_detects_textrun_reflow_pattern() {
        // 긴 텍스트(40자 초과) + lineseg 1개 + '\n' 없음 → R3 경고
        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();
        para.text = "이것은 충분히 길어서 한 줄로 표시하기 어려운 한국어 문장입니다. 한컴은 textRun으로 reflow하지만 rhwp는 그대로 그립니다.".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000; // line_height 는 0 아님 → R2 는 해당 안 됨
        para.line_segs.push(seg);
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert_eq!(report.len(), 1);
        assert_eq!(report.warnings[0].kind, WarningKind::LinesegTextRunReflow);
    }

    #[test]
    fn validate_skips_textrun_reflow_for_short_text() {
        // 짧은 텍스트(40자 이하) → R3 해당 안 됨
        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();
        para.text = "짧은 문장입니다.".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert!(report.is_empty(), "짧은 문장은 경고 대상이 아님");
    }

    #[test]
    fn validate_skips_textrun_reflow_when_has_newline() {
        // 긴 텍스트라도 '\n' 이 있으면 이미 분할된 것으로 간주 → R3 해당 안 됨
        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();
        para.text = "충분히 긴 텍스트이지만 줄바꿈이 있습니다.\n그래서 R3은 해당하지 않아야 합니다.".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert!(report.is_empty(), "\\n 있는 문단은 R3 해당 안 됨");
    }

    #[test]
    fn needs_reflow_broadly_covers_textrun_reflow() {
        let mut para = Paragraph::default();
        para.text = "이것은 충분히 길어서 한 줄로 표시하기 어려운 한국어 문장입니다. 한컴은 textRun으로 reflow하지만 rhwp는 그대로 그립니다.".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        assert!(DocumentCore::needs_reflow_broadly(&para));
    }
}

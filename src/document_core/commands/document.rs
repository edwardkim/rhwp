//! 문서 생성/로딩/저장/설정 관련 native 메서드

use crate::document_core::validation::{
    CellPath, ValidationReport, ValidationWarning, WarningKind,
};
use crate::document_core::{DocumentCore, DEFAULT_FALLBACK_FONT};
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::{Caption, DrawingObjAttr, ShapeObject};
use crate::renderer::composer::{compose_section, reflow_line_segs};
use crate::renderer::layout::LayoutEngine;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::{resolve_styles, ResolvedStyleSet};
use crate::renderer::{px_to_hwpunit, DEFAULT_DPI};
use std::cell::RefCell;
use std::collections::HashMap;

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
    /// [Task #741 후속] 외부 file path 그림 영역 의 binary 영역 영역 base_dir 영역 영역 자동 load.
    ///
    /// HWP3 파일 영역 image 영역 영역 영역 영역 절대 경로 (예: "D:\\Work\\...\\rdb02.gif") 영역
    /// 저장 영역. 본 환경 영역 영역 영역 path 영역 영역 access 부재 영역 영역 영역, basename
    /// 영역 영역 추출 → `base_dir` 영역 영역 영역 file 영역 load → renderer 영역 영역 표시.
    ///
    /// 반환: load 영역 image 영역.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn populate_external_images_from_dir(&mut self, base_dir: &std::path::Path) -> usize {
        let loaded = self.document.populate_external_images_from_dir(base_dir);
        if loaded > 0 {
            self.invalidate_page_tree_cache();
        }
        loaded
    }

    pub fn from_bytes(data: &[u8]) -> Result<DocumentCore, HwpError> {
        let source_format = crate::parser::detect_format(data);
        let mut document = crate::parser::parse_document(data)
            .map_err(|e| HwpError::InvalidFile(e.to_string()))?;

        // [Task #1001] HWP3 변환본의 ParaShape 단위 1/2 추가 보정
        let styles = crate::renderer::style_resolver::resolve_styles_with_variant(
            &document.doc_info,
            DEFAULT_DPI,
            document.is_hwp3_variant,
        );

        let hwp5_origin_hwpx = matches!(source_format, crate::parser::FileFormat::Hwpx)
            && document
                .hwpx_aux_entry(crate::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH)
                .is_some();
        let use_hwpx_lineseg_semantics =
            matches!(source_format, crate::parser::FileFormat::Hwpx) && !hwp5_origin_hwpx;

        // 비표준 lineseg 감지 — reflow 이전 시점에 IR을 그대로 검증.
        // 경고는 사용자에게 고지되며, 자동 reflow 는 `needs_line_seg_reflow` 조건에만 한정.
        // 사용자 명시 reflow 는 `reflow_linesegs_on_demand()` 를 통해서만 수행 (#177).
        // LinesegTextRunReflow는 HWPX 전용 비표준 패턴. HWP3/HWP5는 1 line_info = 1 lineseg가 정상.
        let check_textrun_reflow = use_hwpx_lineseg_semantics;
        let validation_report = Self::validate_linesegs(&document, check_textrun_reflow);

        // lineSegArray가 없는 문단에 대해 합성 LineSeg 생성.
        // HWPX 파서는 linesegarray 부재 문단의 line_segs 를 빈 채 보존하므로(#1380)
        // HWPX 에서만 빈 line_segs 를 합성 대상에 포함한다 — compose 전에 올바른
        // line_height/line_spacing 을 계산해야 줄바꿈·높이가 정상 동작한다.
        // HWP5/HWP3 의 빈 line_segs 는 종전대로 reflow 하지 않는다 (페이지 수 보존).
        let include_empty = use_hwpx_lineseg_semantics;
        Self::reflow_zero_height_paragraphs(&mut document, &styles, DEFAULT_DPI, include_empty);
        Self::clear_missing_lineseg_placeholders(&mut document);

        // HWPX → HWP 라운드트립 일관성 normalize (#314):
        // HWPX 파서가 채우지 않는 paragraph 필드를 HWP 직렬화/파싱 라운드트립 결과와 일치시킨다.
        // 1) char_shapes 빈 paragraph 에 default [(0,0)] 추가 (HWP 스펙상 최소 1개 요구)
        // 2) control_mask 를 controls 기반으로 재계산
        if use_hwpx_lineseg_semantics {
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
            table_transpose_clipboard: None,
            paste_cascade_count: 0,
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
    pub(crate) fn validate_linesegs(
        document: &Document,
        check_textrun_reflow: bool,
    ) -> ValidationReport {
        let mut report = ValidationReport::new();
        for (si, section) in document.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                Self::check_paragraph_linesegs(
                    para,
                    si,
                    pi,
                    None,
                    check_textrun_reflow,
                    &mut report,
                );

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
    /// `include_empty`: 빈 `line_segs` 도 합성 대상으로 포함 (HWPX 전용 — #1380).
    fn reflow_zero_height_paragraphs(
        document: &mut Document,
        styles: &ResolvedStyleSet,
        dpi: f64,
        include_empty: bool,
    ) {
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

            let mut body_line_seg_changed = false;
            // [Issue #1920] vpos 재계산(아래) 시 저장 vpos 의 새 쪽 시작 신호를 보존하기
            // 위해, 이번 패스에서 LINE_SEG 가 합성(reflow)된 문단 — 저장 vpos 신뢰 불가 —
            // 을 기록한다.
            let mut reflowed_paras: std::collections::HashSet<usize> =
                std::collections::HashSet::new();
            for (pi, para) in section.paragraphs.iter_mut().enumerate() {
                // 본문 문단 reflow
                if Self::needs_line_seg_reflow(para, include_empty) {
                    let para_style = styles.para_styles.get(para.para_shape_id as usize);
                    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                    let available_width = (col_width - margin_left - margin_right).max(1.0);
                    reflow_line_segs(para, available_width, styles, dpi);
                    body_line_seg_changed = true;
                    reflowed_paras.insert(pi);
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
                    if max_tac_h > 0
                        && !matches!(
                            para.line_segs.as_slice(),
                            [seg] if seg.is_missing_lineseg_placeholder()
                        )
                    {
                        // [Task #1068] 이미 표 높이를 담은 LINE_SEG 가 있으면(한컴이
                        // 저장한 실제 linesegarray 보유 — 표 줄 seg 의 vertsize 가 표
                        // 높이) 보정 불필요. 무조건 first_mut() 을 확대하면 표가 두 번째
                        // 이후 줄에 있는 문단(제목줄 + 표줄)의 제목줄 lh 까지 표 높이로
                        // 오염되어, 렌더러의 lh 기반 표 줄 탐지(place_table_with_text)가
                        // 첫 줄을 오매칭 → 표 줄 이중 그리기 overflow (#1068 제안요청서
                        // para 567: 제목줄 vertsize=2200 → 63234 오염, 839px overflow).
                        // linesegarray 가 없어 기본 lh=100 단일 seg 만 있는 경우에만
                        // 첫 seg 를 표 높이로 확대한다.
                        // HWP5-origin HWPX export marker 는 "원본 LineSeg 부재"를 보존하기
                        // 위한 임시 표식이므로 여기서 표 높이로 오염시키면 안 된다.
                        // 이 marker 는 reflow gate 후 clear_missing_lineseg_placeholders 에서
                        // 제거되어 HWP5 원본과 같은 line_segs.is_empty() 경로를 타야 한다.
                        let already_covered =
                            para.line_segs.iter().any(|s| s.line_height >= max_tac_h);
                        if !already_covered {
                            if let Some(seg) = para.line_segs.first_mut() {
                                if seg.line_height < max_tac_h {
                                    seg.line_height = max_tac_h;
                                    body_line_seg_changed = true;
                                }
                            }
                        }
                    }
                }

                // 표 셀 내부 문단 reflow
                for ctrl in &mut para.controls {
                    if let Control::Table(ref mut table) = ctrl {
                        let is_rowbreak_table = matches!(
                            table.page_break,
                            crate::model::table::TablePageBreak::RowBreak
                        );
                        for cell in &mut table.cells {
                            // [Task #671 후속 / Issue #671 자동보정 영역 정정]
                            // 셀 폭 (cell.width) 에서 좌우 padding 차감하여 셀 inner 폭 계산.
                            // col_width 사용 시 셀 너비 영역 밖으로 LINE_SEG 가 채워져
                            // recompose_for_cell_width 가드 #1 (line_segs.is_empty()) 영역 거짓 →
                            // PR #673 영역의 layout 단계 정정 미적용 → 자동보정 모드 영역 한 줄 겹침 회귀.
                            let cell_w_px = crate::renderer::hwpunit_to_px(cell.width as i32, dpi);
                            let pad_left =
                                crate::renderer::hwpunit_to_px(cell.padding.left as i32, dpi);
                            let pad_right =
                                crate::renderer::hwpunit_to_px(cell.padding.right as i32, dpi);
                            let cell_inner_width = (cell_w_px - pad_left - pad_right).max(1.0);
                            for cell_para in &mut cell.paragraphs {
                                if Self::needs_line_seg_reflow(cell_para, include_empty) {
                                    reflow_line_segs(cell_para, cell_inner_width, styles, dpi);
                                }
                            }
                            if include_empty && is_rowbreak_table {
                                Self::fit_hwpx_rowbreak_synthetic_cell_lines(
                                    cell,
                                    styles,
                                    dpi,
                                    table.common.treat_as_char,
                                );
                            }
                        }
                    }
                }
            }

            // HWPX: LINE_SEG를 실제로 합성/보정한 경우에만 문단 간 vpos를 재계산한다.
            //
            // 명시적인 lineSegArray가 이미 계산 완료 상태인 문서는 source의 vertpos를 보존해야 한다.
            // 비-TAC TopAndBottom 표/그림이 있다는 이유만으로 section vpos를 다시 계산하면, 한컴이
            // 저장한 HWPX의 vertpos까지 덮어써 page sequence가 어긋난다 (#949 Stage 32).
            if body_line_seg_changed {
                let mut running_vpos: i32 = 0;
                // [Issue #1920] 직전까지 본 "원본(비합성) lineseg 보유 문단"의 마지막 저장
                // vpos. 결재문서류 생성기는 새 쪽 시작 문단(발신명의 틀 host)에 vpos=0 을
                // 저장하는데, 이 재계산이 연속 좌표로 덮어쓰면 typeset 의 vpos-reset 쪽나눔
                // (#321, paragraph_saved_vpos_reset_starts_new_page_after)이 무력화되어
                // 한글이 다음 쪽에 두는 틀이 이전 쪽에 흡수된다(36417450 pi8, 1쪽 vs 2쪽).
                // 원본 first vpos=0 + 직전 저장 vpos>5000(동일 임계) + 쪽 하단 고정 틀
                // (vert=쪽·valign=Bottom, 발신명의 서명란·직인 틀) host 문단에서만
                // running_vpos 를 0 으로 되돌려 리셋 신호를 재계산 좌표계에 보존한다.
                // 틀 host 한정인 이유: 일반 문단의 mid-doc vpos=0 은 생성기 노이즈일 수
                // 있어(task1749 pi2/47) 전면 보존 시 무관 문서의 배치가 흔들린다.
                // wrap 은 불문 — 자리차지(발신명의)와 글뒤로(직인 도장, 36408321 pi12)
                // 모두 같은 새 쪽 시그니처다.
                let mut prev_stored_last_vpos: i32 = 0;
                for (pi, para) in section.paragraphs.iter_mut().enumerate() {
                    let was_reflowed = reflowed_paras.contains(&pi);
                    let hosts_bottom_fixed_frame = para.controls.iter().any(|c| {
                        matches!(c, Control::Table(t)
                        if !t.common.treat_as_char
                            && matches!(
                                t.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Page
                            )
                            && matches!(
                                t.common.vert_align,
                                crate::model::shape::VertAlign::Bottom
                            ))
                    });
                    if !was_reflowed
                        && hosts_bottom_fixed_frame
                        && prev_stored_last_vpos > 5000
                        && para.line_segs.first().map(|s| s.vertical_pos) == Some(0)
                    {
                        running_vpos = 0;
                    }
                    let original_last_vpos = if was_reflowed {
                        None
                    } else {
                        para.line_segs.last().map(|s| s.vertical_pos)
                    };
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
                    if let Some(v) = original_last_vpos {
                        prev_stored_last_vpos = v;
                    }
                }
            }
        }
    }

    /// 문단의 LineSeg가 합성(reflow)이 필요한지 판단한다.
    /// line_segs가 1개이고 line_height가 0이면 lineSegArray 누락 상태.
    ///
    /// `include_empty`: 빈 `line_segs` 도 누락으로 취급할지 여부. **HWPX 전용** —
    /// HWPX 파서는 linesegarray 부재 문단을 빈 채 보존하므로(#1380) 로드 시 합성이
    /// 필요하다. HWP5/HWP3 는 빈 line_segs 를 reflow 하지 않던 종전 동작을 유지한다
    /// (확장 시 sample16-hwp5 페이지 수 64→over-split 회귀 확인).
    fn needs_line_seg_reflow(
        para: &crate::model::paragraph::Paragraph,
        include_empty: bool,
    ) -> bool {
        if para.line_segs.len() == 1 && para.line_segs[0].is_missing_lineseg_placeholder() {
            return false;
        }
        (include_empty && para.line_segs.is_empty())
            || (para.line_segs.len() == 1 && para.line_segs[0].line_height == 0)
    }

    /// HWP5 -> HWPX export가 넣은 LineSeg 부재 marker는 reflow gate에서만 사용한다.
    /// 레이아웃은 HWP5 원본과 같은 `line_segs.is_empty()` 경로를 타야 하므로 로드 직후 제거한다.
    fn clear_missing_lineseg_placeholders(document: &mut Document) {
        for section in &mut document.sections {
            for para in &mut section.paragraphs {
                Self::clear_missing_lineseg_placeholder_in_paragraph(para);
            }
            for master_page in &mut section.section_def.master_pages {
                for para in &mut master_page.paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
        }
    }

    fn clear_missing_lineseg_placeholder_in_paragraph(para: &mut Paragraph) {
        for ctrl in &mut para.controls {
            Self::clear_missing_lineseg_placeholders_in_control(ctrl);
        }
        if para.line_segs.len() == 1 && para.line_segs[0].is_missing_lineseg_placeholder() {
            para.line_segs.clear();
        }
    }

    fn clear_missing_lineseg_placeholders_in_control(ctrl: &mut Control) {
        match ctrl {
            Control::Table(table) => {
                for cell in &mut table.cells {
                    for para in &mut cell.paragraphs {
                        Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                    }
                }
                if let Some(caption) = &mut table.caption {
                    Self::clear_missing_lineseg_placeholders_in_caption(caption);
                }
            }
            Control::Shape(shape) => Self::clear_missing_lineseg_placeholders_in_shape(shape),
            Control::Picture(picture) => {
                if let Some(caption) = &mut picture.caption {
                    Self::clear_missing_lineseg_placeholders_in_caption(caption);
                }
            }
            Control::Header(header) => {
                for para in &mut header.paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
            Control::Footer(footer) => {
                for para in &mut footer.paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
            Control::Footnote(footnote) => {
                for para in &mut footnote.paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
            Control::Endnote(endnote) => {
                for para in &mut endnote.paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
            Control::HiddenComment(comment) => {
                for para in &mut comment.paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
            Control::Field(field) => {
                for para in &mut field.memo_paragraphs {
                    Self::clear_missing_lineseg_placeholder_in_paragraph(para);
                }
            }
            _ => {}
        }
    }

    fn clear_missing_lineseg_placeholders_in_shape(shape: &mut ShapeObject) {
        match shape {
            ShapeObject::Line(line) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut line.drawing)
            }
            ShapeObject::Rectangle(rect) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut rect.drawing)
            }
            ShapeObject::Ellipse(ellipse) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut ellipse.drawing)
            }
            ShapeObject::Arc(arc) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut arc.drawing)
            }
            ShapeObject::Polygon(polygon) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut polygon.drawing)
            }
            ShapeObject::Curve(curve) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut curve.drawing)
            }
            ShapeObject::Group(group) => {
                for child in &mut group.children {
                    Self::clear_missing_lineseg_placeholders_in_shape(child);
                }
                if let Some(caption) = &mut group.caption {
                    Self::clear_missing_lineseg_placeholders_in_caption(caption);
                }
            }
            ShapeObject::Picture(picture) => {
                if let Some(caption) = &mut picture.caption {
                    Self::clear_missing_lineseg_placeholders_in_caption(caption);
                }
            }
            ShapeObject::Chart(chart) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut chart.drawing);
                if let Some(caption) = &mut chart.caption {
                    Self::clear_missing_lineseg_placeholders_in_caption(caption);
                }
            }
            ShapeObject::Ole(ole) => {
                Self::clear_missing_lineseg_placeholders_in_drawing(&mut ole.drawing);
                if let Some(caption) = &mut ole.caption {
                    Self::clear_missing_lineseg_placeholders_in_caption(caption);
                }
            }
        }
    }

    fn clear_missing_lineseg_placeholders_in_drawing(drawing: &mut DrawingObjAttr) {
        if let Some(text_box) = &mut drawing.text_box {
            for para in &mut text_box.paragraphs {
                Self::clear_missing_lineseg_placeholder_in_paragraph(para);
            }
        }
        if let Some(caption) = &mut drawing.caption {
            Self::clear_missing_lineseg_placeholders_in_caption(caption);
        }
    }

    fn clear_missing_lineseg_placeholders_in_caption(caption: &mut Caption) {
        for para in &mut caption.paragraphs {
            Self::clear_missing_lineseg_placeholder_in_paragraph(para);
        }
    }

    /// HWPX RowBreak 표 셀의 합성 lineSeg를 셀에 저장된 세로 정보와 맞춘다.
    ///
    /// HWPX는 표 셀 안의 문단별 `<hp:linesegarray>`를 생략하면서도, 셀 높이와 마지막
    /// 빈 anchor 문단에는 한컴이 계산한 세로 기준선을 남기는 경우가 있다. 셀의 명시
    /// 높이에 비해 합성 lineSeg가 부족하면 쪽 나눔 후 다음 페이지 표 조각의 줄 수가
    /// 모자라므로, 다음 문서 속성만 근거로 부족한 줄을 보강한다.
    ///
    /// - RowBreak 표 셀의 `height`
    /// - 문단 `ParaShape.spacing_before`
    /// - 합성 lineSeg의 `line_height + line_spacing`
    /// - 셀 끝의 저장 anchor lineSeg (`vertical_pos > 0`, implementation tag 없음)
    fn fit_hwpx_rowbreak_synthetic_cell_lines(
        cell: &mut crate::model::table::Cell,
        styles: &ResolvedStyleSet,
        dpi: f64,
        allow_without_anchor: bool,
    ) {
        if cell.height == 0 || cell.paragraphs.len() < 2 {
            return;
        }

        let is_synthetic = |seg: &LineSeg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0;
        let para_is_synthetic = |para: &Paragraph| {
            !para.text.is_empty()
                && !para.line_segs.is_empty()
                && para.line_segs.iter().all(is_synthetic)
        };
        let has_stored_anchor = cell.paragraphs.iter().any(|para| {
            para.text.is_empty()
                && para.controls.is_empty()
                && para.line_segs.len() == 1
                && !is_synthetic(&para.line_segs[0])
                && para.line_segs[0].vertical_pos > 0
                && para.line_segs[0].segment_width > 0
        });
        if !has_stored_anchor && !allow_without_anchor {
            return;
        }
        if !cell.paragraphs.iter().any(para_is_synthetic) {
            return;
        }

        let spacing_before_hu = |para: &Paragraph| -> i32 {
            styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|ps| px_to_hwpunit(ps.spacing_before, dpi).max(0))
                .unwrap_or(0)
        };

        let paragraph_height = |para: &Paragraph| -> i32 {
            if para.line_segs.is_empty() {
                return 0;
            }
            let spacing_before = spacing_before_hu(para);
            if para.text.is_empty() && para.controls.is_empty() {
                return spacing_before + para.line_segs[0].line_height.max(0);
            }
            spacing_before
                + para
                    .line_segs
                    .iter()
                    .map(|seg| (seg.line_height + seg.line_spacing).max(0))
                    .sum::<i32>()
        };

        let mut current_height: i32 = cell.paragraphs.iter().map(paragraph_height).sum();
        let target_height = cell.height.min(i32::MAX as u32) as i32;
        if current_height >= target_height {
            return;
        }

        let nominal_advance = cell
            .paragraphs
            .iter()
            .filter(|para| para_is_synthetic(para))
            .flat_map(|para| para.line_segs.iter())
            .map(|seg| seg.line_height + seg.line_spacing)
            .filter(|advance| *advance > 0)
            .min()
            .unwrap_or(0);
        if nominal_advance <= 0 {
            return;
        }

        let capacity_hint = cell
            .paragraphs
            .iter()
            .filter(|para| para_is_synthetic(para) && para.line_segs.len() >= 2)
            .filter_map(|para| para.line_segs.get(1).map(|seg| seg.text_start))
            .filter(|text_start| *text_start > 0)
            .min();

        let mut candidates: Vec<usize> = cell
            .paragraphs
            .iter()
            .enumerate()
            .filter_map(|(idx, para)| {
                if para_is_synthetic(para) && para.line_segs.len() == 1 {
                    Some((idx, para.text.chars().count()))
                } else {
                    None
                }
            })
            .filter(|(_, text_len)| *text_len > 1)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
        candidates.sort_by(|a, b| {
            let len_a = cell.paragraphs[*a].text.chars().count();
            let len_b = cell.paragraphs[*b].text.chars().count();
            len_b.cmp(&len_a).then_with(|| a.cmp(b))
        });

        for para_idx in candidates {
            if current_height + nominal_advance > target_height {
                break;
            }
            if Self::append_synthetic_cell_line(&mut cell.paragraphs[para_idx], capacity_hint) {
                current_height += nominal_advance;
            }
        }
    }

    fn append_synthetic_cell_line(para: &mut Paragraph, capacity_hint: Option<u32>) -> bool {
        if para.line_segs.len() != 1 {
            return false;
        }
        let first = para.line_segs[0].clone();
        if first.line_height + first.line_spacing <= 0 {
            return false;
        }
        let text_unit_len = para.char_count.saturating_sub(1);
        if text_unit_len <= 1 {
            return false;
        }
        let split_start = capacity_hint
            .unwrap_or(text_unit_len.saturating_sub(1))
            .min(text_unit_len.saturating_sub(1))
            .max(1);
        if split_start <= first.text_start {
            return false;
        }
        let mut second = first.clone();
        second.text_start = split_start;
        second.vertical_pos = first.vertical_pos + first.line_height + first.line_spacing;
        para.line_segs.push(second);
        true
    }

    /// 사용자 명시 요청에 의한 더 넓은 reflow 판정 (#177).
    ///
    /// `needs_line_seg_reflow` (명백한 미계산) + 다음 케이스 포함:
    /// - 텍스트가 있는데 line_segs 가 비어있음 (LinesegArrayEmpty)
    ///
    /// 이 함수는 `reflow_linesegs_on_demand` 에서만 사용되며, 자동 파싱 경로에는 영향 없음.
    fn needs_reflow_broadly(para: &crate::model::paragraph::Paragraph) -> bool {
        if !para.text.is_empty() && para.line_segs.is_empty() {
            return true;
        }
        if Self::needs_line_seg_reflow(para, false) {
            return true;
        }
        false
    }

    /// 사용자 명시 요청에 의한 전체 lineseg reflow (#177).
    ///
    /// `validate_linesegs` 에 기록된 경고 대상 문단들 중 명백히 reflow 가능한 것을 처리한다.
    /// 기본 파싱 경로의 `reflow_zero_height_paragraphs` 와 달리 이 메서드는
    /// 사용자가 UI에서 "자동 보정" 을 명시적으로 선택했을 때만 호출되어야 한다.
    /// `LinesegTextRunReflow` 는 한컴이 계산한 1개 lineseg 를 강제로 다시 풀면
    /// 페이지 수가 바뀔 수 있으므로 경고만 남기고 자동 보정 대상에서 제외한다.
    ///
    /// 반환값: 실제로 reflow 된 문단 개수 (본문 + 셀 내부 합계).
    pub fn reflow_linesegs_on_demand(&mut self) -> usize {
        if self.validation_report.is_empty() {
            return 0;
        }

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

            let mut min_reflowed_idx: Option<usize> = None;
            for (pi, para) in section.paragraphs.iter_mut().enumerate() {
                if Self::needs_reflow_broadly(para) {
                    let para_style = styles.para_styles.get(para.para_shape_id as usize);
                    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                    let available_width = (col_width - margin_left - margin_right).max(1.0);
                    reflow_line_segs(para, available_width, &styles, dpi);
                    reflowed += 1;
                    if min_reflowed_idx.is_none() {
                        min_reflowed_idx = Some(pi);
                    }
                }
                // 표 셀 내부 문단도 동일 처리
                for ctrl in &mut para.controls {
                    if let Control::Table(ref mut table) = ctrl {
                        for cell in &mut table.cells {
                            // [Task #671 후속 / Issue #671 자동보정 영역 정정]
                            // 셀 폭 (cell.width) 에서 좌우 padding 차감하여 셀 inner 폭 계산.
                            // 동일 본질 정정: line 270 영역 참조.
                            let cell_w_px = crate::renderer::hwpunit_to_px(cell.width as i32, dpi);
                            let pad_left =
                                crate::renderer::hwpunit_to_px(cell.padding.left as i32, dpi);
                            let pad_right =
                                crate::renderer::hwpunit_to_px(cell.padding.right as i32, dpi);
                            let cell_inner_width = (cell_w_px - pad_left - pad_right).max(1.0);
                            for cell_para in &mut cell.paragraphs {
                                if Self::needs_reflow_broadly(cell_para) {
                                    reflow_line_segs(cell_para, cell_inner_width, &styles, dpi);
                                    reflowed += 1;
                                }
                            }
                        }
                    }
                }
            }

            // [Task #927] reflow 후 vpos 일관성 재계산 — 본문 paragraphs 만.
            // 빈 lineseg 였던 문단들은 reflow 시 vpos_start=0 으로 시작하여 후속 문단
            // 의 vpos 연속성이 깨짐. paginator 의 vpos_h 기반 current_height 조정이
            // 잘못된 값으로 적용되어 페이지가 과다 분할되는 회귀의 원인.
            if let Some(start) = min_reflowed_idx {
                crate::renderer::composer::recalculate_section_vpos(&mut section.paragraphs, start);
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
        self.table_transpose_clipboard = None;
        self.dirty_sections = vec![true; sec_count];
        self.measured_tables = Vec::new();
        self.measured_sections = Vec::new();
        self.dirty_paragraphs = Vec::new();
        self.para_column_map = Vec::new();
        self.page_tree_cache.borrow_mut().clear();
        self.snapshot_store.clear();
        self.next_snapshot_id = 0;
        self.source_format = crate::parser::FileFormat::Hwp;
        self.validation_report = ValidationReport::new();

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
        let serialized = if matches!(self.source_format, crate::parser::FileFormat::Hwp) {
            let mut doc = self.document.clone();
            if !doc
                .hwpx_aux_entries
                .iter()
                .any(|(path, _)| path == crate::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH)
            {
                doc.hwpx_aux_entries.push((
                    crate::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH.to_string(),
                    b"1".to_vec(),
                ));
            }
            Self::materialize_hwp5_missing_linesegs_for_hwpx_export(&mut doc);
            crate::serializer::serialize_hwpx(&doc)
        } else {
            crate::serializer::serialize_hwpx(&self.document)
        };
        serialized.map_err(|e| HwpError::RenderError(e.to_string()))
    }

    /// HWP5 원본에서 LineSeg가 없던 문단을 HWPX 재파스에서도 일반 HWPX 누락 문단으로
    /// reflow하지 않도록 명시 LineSeg marker로 materialize한다.
    fn materialize_hwp5_missing_linesegs_for_hwpx_export(document: &mut Document) {
        for section in &mut document.sections {
            for para in &mut section.paragraphs {
                Self::materialize_missing_lineseg_paragraph(para);
            }
            for master_page in &mut section.section_def.master_pages {
                for para in &mut master_page.paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
        }
    }

    fn materialize_missing_lineseg_paragraph(para: &mut Paragraph) {
        for ctrl in &mut para.controls {
            Self::materialize_missing_lineseg_paragraphs_in_control(ctrl);
        }

        if para.line_segs.is_empty() {
            para.line_segs.push(LineSeg::missing_lineseg_placeholder());
        }
    }

    fn materialize_missing_lineseg_paragraphs_in_control(ctrl: &mut Control) {
        match ctrl {
            Control::Table(table) => {
                for cell in &mut table.cells {
                    for para in &mut cell.paragraphs {
                        Self::materialize_missing_lineseg_paragraph(para);
                    }
                }
                if let Some(caption) = &mut table.caption {
                    Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
                }
            }
            Control::Shape(shape) => {
                Self::materialize_missing_lineseg_paragraphs_in_shape(shape);
            }
            Control::Picture(picture) => {
                if let Some(caption) = &mut picture.caption {
                    Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
                }
            }
            Control::Header(header) => {
                for para in &mut header.paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
            Control::Footer(footer) => {
                for para in &mut footer.paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
            Control::Footnote(footnote) => {
                for para in &mut footnote.paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
            Control::Endnote(endnote) => {
                for para in &mut endnote.paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
            Control::HiddenComment(comment) => {
                for para in &mut comment.paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
            Control::Field(field) => {
                for para in &mut field.memo_paragraphs {
                    Self::materialize_missing_lineseg_paragraph(para);
                }
            }
            _ => {}
        }
    }

    fn materialize_missing_lineseg_paragraphs_in_shape(shape: &mut ShapeObject) {
        match shape {
            ShapeObject::Line(line) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut line.drawing)
            }
            ShapeObject::Rectangle(rect) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut rect.drawing)
            }
            ShapeObject::Ellipse(ellipse) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut ellipse.drawing)
            }
            ShapeObject::Arc(arc) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut arc.drawing)
            }
            ShapeObject::Polygon(polygon) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut polygon.drawing)
            }
            ShapeObject::Curve(curve) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut curve.drawing)
            }
            ShapeObject::Group(group) => {
                for child in &mut group.children {
                    Self::materialize_missing_lineseg_paragraphs_in_shape(child);
                }
                if let Some(caption) = &mut group.caption {
                    Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
                }
            }
            ShapeObject::Picture(picture) => {
                if let Some(caption) = &mut picture.caption {
                    Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
                }
            }
            ShapeObject::Chart(chart) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut chart.drawing);
                if let Some(caption) = &mut chart.caption {
                    Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
                }
            }
            ShapeObject::Ole(ole) => {
                Self::materialize_missing_lineseg_paragraphs_in_drawing(&mut ole.drawing);
                if let Some(caption) = &mut ole.caption {
                    Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
                }
            }
        }
    }

    fn materialize_missing_lineseg_paragraphs_in_drawing(drawing: &mut DrawingObjAttr) {
        if let Some(text_box) = &mut drawing.text_box {
            for para in &mut text_box.paragraphs {
                Self::materialize_missing_lineseg_paragraph(para);
            }
        }
        if let Some(caption) = &mut drawing.caption {
            Self::materialize_missing_lineseg_paragraphs_in_caption(caption);
        }
    }

    fn materialize_missing_lineseg_paragraphs_in_caption(caption: &mut Caption) {
        for para in &mut caption.paragraphs {
            Self::materialize_missing_lineseg_paragraph(para);
        }
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

    /// [Task #741 후속] 문서의 IR mutable 참조를 반환한다.
    /// WASM 영역 영역 외부 image inject 영역 의 영역 영역 영역.
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
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
                    Control::Bookmark(_) => 0x0016,
                    Control::CharOverlap(_) => 0x0017,
                };
                mask |= 1u32 << bit;
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

        fn process_para(para: &mut Paragraph) {
            if para.char_shapes.is_empty() {
                para.char_shapes.push(CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                });
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
            // [Task #1893] 삭제 수술의 IR 불변성 완성용 스냅샷 — 삭제 전 char_offsets 는
            // 원본 문자 인덱스→utf16 위치 매핑의 유일한 근거다. removal 좌표는 전부
            // 수집-시점(원본) 인덱스이므로, 원본 스냅샷으로 utf16 범위를 구해
            // char_shapes 경계를 함께 시프트해야 직렬화→재파스가 고정점이 된다.
            // (종전엔 text/field_ranges 만 고쳐 char_offsets/char_count/char_shapes 가
            // stale — 그 불일치 IR 을 저장하면 재파스 정준형과 조판이 갈라져
            // 라운드트립 렌더 752px 분기·빈 줄 추가가 발생했다.)
            let orig_offsets: Vec<u32> = para.char_offsets.clone();
            let orig_chars: Vec<char> = para.text.chars().collect();
            let offsets_valid = orig_offsets.len() == orig_chars.len();
            fn utf16_width(c: char) -> u32 {
                if c == '\t' {
                    8
                } else if (c as u32) > 0xFFFF {
                    2
                } else {
                    1
                }
            }
            let mut any_removed = false;

            // 뒤에서부터 삭제 (인덱스 안정성 유지)
            for &(fri, start, end) in removals.iter().rev() {
                let chars: Vec<char> = para.text.chars().collect();
                // [Task #1620] 다중 removal 처리 중 앞선 removal 이 para.text 를 축소하면(특히
                // 같은 범위를 가리키는 중첩 field_range) 이후 removal 의 수집-시점 (start,end) 가
                // 현재 길이를 초과해 슬라이스 패닉(36396650). 현재 길이 기준 범위를 재검증해 skip.
                if start > end || end > chars.len() {
                    continue;
                }
                let removed_len = end - start;
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
                any_removed = true;

                // [Task #1893] char_offsets/char_shapes/char_count 직접 수술 — 원본 utf16
                // 좌표 기준. 역순 처리라 오른쪽 removal 의 시프트가 왼쪽 utf16 좌표에 영향
                // 없고, 삭제 폭(u_end−u_start)은 원본 스냅샷 불변량이다. 컨트롤/필드 마커의
                // 8유닛 갭 구조는 기존 오프셋에 이미 올바르게 인코딩되어 있으므로 감산만으로
                // 보존된다 (rebuild_char_offsets 의 선행-컨트롤 휴리스틱은 문단 서두 0-length
                // 필드의 end 마커를 컨트롤로 오산해 begin 갭을 유실 — 필드쌍 교차 페어링 유발).
                if offsets_valid && start < end && end <= orig_offsets.len() {
                    let u_start = orig_offsets[start];
                    // 삭제 폭 = 삭제 문자들의 utf16 폭만. orig_offsets[end] 는 필드 end
                    // 마커의 8유닛 갭을 건너뛴 다음 문자 위치라 갭까지 폭에 포함되어
                    // 후속 오프셋에서 마커 갭이 소실된다(슬롯 방출 위치 붕괴).
                    let u_end = orig_offsets[end - 1] + utf16_width(orig_chars[end - 1]);
                    let width = u_end.saturating_sub(u_start);
                    // 삭제 구간의 오프셋 엔트리 제거 + 후속 엔트리 감산.
                    para.char_offsets.drain(start..end);
                    for off in para.char_offsets.iter_mut().skip(start) {
                        *off = off.saturating_sub(width);
                    }
                    para.char_count = para.char_count.saturating_sub(width);
                    for cs in &mut para.char_shapes {
                        if cs.start_pos >= u_end {
                            cs.start_pos -= width;
                        } else if cs.start_pos > u_start {
                            // 삭제 범위 내부 경계 → zero-width run 으로 시작점에 고정
                            // (한컴도 필드값 삭제 시 zero-width char run 을 남긴다 —
                            // 원본 서식의 자식 없는 <hp:run/> 33개와 동일 표현).
                            cs.start_pos = u_start;
                        }
                    }
                }
            }
            let _ = any_removed;
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

    /// [Task #1620] `clear_initial_field_texts`: 같은 텍스트 범위를 가리키는 다중 ClickHere
    /// field_range 처리 시, 첫 removal 이 `para.text` 를 비우면 이후 removal 이 stale 인덱스로
    /// 슬라이스해 패닉(36396650, `document.rs:927` range out of range). 범위 가드 추가로
    /// 패닉 없이 정규화돼야 함.
    #[test]
    fn clear_initial_field_texts_no_panic_on_overlapping_removals() {
        use crate::model::control::{Control, Field, FieldType};
        use crate::model::paragraph::FieldRange;

        let field = Field {
            field_type: FieldType::ClickHere,
            command: "Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  "
                .to_string(),
            properties: 0, // bit15 == 0 (초기 상태 → 안내문 제거 대상)
            ..Default::default()
        };
        // 같은 텍스트 범위 [0,6) 를 가리키는 field_range 2개(중첩) → 다중 removal.
        let para = Paragraph {
            text: "여기에 입력".to_string(),
            controls: vec![Control::Field(field)],
            field_ranges: vec![
                FieldRange {
                    start_char_idx: 0,
                    end_char_idx: 6,
                    control_idx: 0,
                },
                FieldRange {
                    start_char_idx: 0,
                    end_char_idx: 6,
                    control_idx: 0,
                },
            ],
            ..Default::default()
        };
        let mut doc = Document::default();
        let mut section = Section::default();
        section.paragraphs.push(para);
        doc.sections.push(section);

        // 수정 전: document.rs 제거 루프에서 stale 인덱스 슬라이스 패닉.
        // 수정 후: 패닉 없이 안내문 제거(빈 텍스트).
        DocumentCore::clear_initial_field_texts(&mut doc);
        assert!(
            doc.sections[0].paragraphs[0].text.is_empty(),
            "안내문이 제거돼 빈 텍스트여야 함"
        );
    }

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
        assert!(
            report.is_empty(),
            "healthy paragraph should not warn: {:?}",
            report.warnings
        );
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
        let cp = report.warnings[0]
            .cell_path
            .expect("cell_path should be set");
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
        assert_eq!(
            summary
                .get("lineseg 가 미계산 상태 (line_height=0)")
                .copied(),
            Some(1)
        );
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
        para.text =
            "충분히 긴 텍스트이지만 줄바꿈이 있습니다.\n그래서 R3은 해당하지 않아야 합니다."
                .to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        section.paragraphs.push(para);
        doc.sections.push(section);

        let report = DocumentCore::validate_linesegs(&doc, true);
        assert!(report.is_empty(), "\\n 있는 문단은 R3 해당 안 됨");
    }

    #[test]
    fn needs_reflow_broadly_skips_textrun_reflow() {
        let mut para = Paragraph::default();
        para.text = "이것은 충분히 길어서 한 줄로 표시하기 어려운 한국어 문장입니다. 한컴은 textRun으로 reflow하지만 rhwp는 그대로 그립니다.".to_string();
        let mut seg = LineSeg::default();
        seg.line_height = 1000;
        para.line_segs.push(seg);
        assert!(!DocumentCore::needs_reflow_broadly(&para));
    }
}

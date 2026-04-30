//! 레이아웃 엔진 (Layout Engine)
//!
//! 페이지 분할 결과를 받아 각 요소의 정확한 위치와 크기를 계산하고
//! 렌더 트리(PageRenderTree)를 생성한다.

use crate::model::paragraph::Paragraph;
use crate::model::shape::{Caption, CaptionDirection, CommonObjAttr, HorzAlign, HorzRelTo, VertAlign, VertRelTo};
use crate::model::style::{Alignment, BorderLine, BorderLineType, HeadType, Numbering, UnderlineType};
use crate::model::table::VerticalAlign;
use crate::model::footnote::{FootnoteShape, NumberFormat};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::header_footer::MasterPage;
use super::render_tree::*;
use super::page_layout::{LayoutRect, PageLayoutInfo};
use super::pagination::{ColumnContent, PageContent, PageItem, FootnoteRef, FootnoteSource};
use super::height_measurer::MeasuredTable;
use super::composer::{ComposedParagraph, compose_paragraph};
use super::style_resolver::ResolvedStyleSet;
use super::font_metrics_data;
use super::{TextStyle, ShapeStyle, LineStyle, PathCommand, StrokeDash, ArrowStyle, hwpunit_to_px, DEFAULT_DPI, AutoNumberCounter, format_number, NumberFormat as NumFmt};

/// layout_column_item의 읽기 전용 컨텍스트 (파라미터 묶음)
struct ColumnItemCtx<'a> {
    page_content: &'a PageContent,
    paragraphs: &'a [Paragraph],
    composed: &'a [ComposedParagraph],
    styles: &'a ResolvedStyleSet,
    bin_data_content: &'a [BinDataContent],
    measured_tables: &'a [MeasuredTable],
    layout: &'a PageLayoutInfo,
    col_area: &'a LayoutRect,
    outline_numbering_id: u16,
    multi_col_width: Option<i32>,
    prev_tac_seg_applied: bool,
    wrap_around_paras: &'a [super::pagination::WrapAroundPara],
}

/// 표 경로의 단일 레벨 (표 → 셀 → 문단)
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct CellPathEntry {
    /// 문단 내 컨트롤 인덱스 (표)
    pub control_index: usize,
    /// 표 내 셀 인덱스
    pub cell_index: usize,
    /// 셀 내 문단 인덱스
    pub cell_para_index: usize,
    /// 텍스트 방향 (0=가로, 1=세로/영문눕힘, 2=세로/영문세움)
    pub text_direction: u8,
}

/// 표 셀 내부 문단 편집용 컨텍스트 (중첩 표 경로 지원)
#[derive(Debug, Clone, serde::Serialize)]
pub struct CellContext {
    /// 최외곽 표를 소유한 구역 문단 인덱스
    pub parent_para_index: usize,
    /// 표 경로 (depth 1=단일 표, depth 2+=중첩 표)
    pub path: Vec<CellPathEntry>,
}

impl CellContext {
    /// 최외곽 표의 컨트롤 인덱스
    pub fn outermost_control(&self) -> usize { self.path[0].control_index }
    /// 최외곽 표의 셀 인덱스
    pub fn outermost_cell(&self) -> usize { self.path[0].cell_index }
    /// 최외곽 표의 셀 문단 인덱스
    pub fn outermost_cell_para(&self) -> usize { self.path[0].cell_para_index }
    /// 최내곽 레벨의 엔트리
    pub fn innermost(&self) -> &CellPathEntry { self.path.last().unwrap() }
    /// 텍스트 방향 (최내곽 기준)
    pub fn text_direction(&self) -> u8 { self.innermost().text_direction }
}

/// 문단 번호 상태 (수준별 카운터)
#[derive(Debug, Clone, Default)]
struct NumberingState {
    /// 현재 활성 numbering_id
    current_id: Option<u16>,
    /// 수준별 카운터 (0~6 → 1~7수준)
    counters: [u32; 7],
    /// numbering_id별 카운터 히스토리 ("이전 번호 목록에 이어" 지원)
    history: std::collections::HashMap<u16, [u32; 7]>,
}

impl NumberingState {
    /// 카운터를 초기 상태로 리셋
    fn reset(&mut self) {
        self.current_id = None;
        self.counters = [0; 7];
        self.history.clear();
    }

    /// 번호 문단 처리: 카운터를 갱신하고 현재 수준의 번호를 반환
    fn advance(
        &mut self,
        numbering_id: u16,
        level: u8,
        restart: Option<crate::model::paragraph::NumberingRestart>,
    ) -> [u32; 7] {
        use crate::model::paragraph::NumberingRestart;
        let level = (level as usize).min(6);

        // numbering_id가 변경되면 현재 카운터를 히스토리에 저장하고
        // 새 numbering_id의 히스토리에서 복원 (없으면 리셋)
        // HWP 동작:
        //   - 같은 id 연속 = "앞 번호 이어" (카운터 유지)
        //   - 다른 id (히스토리 있음) = "이전 번호 이어" (히스토리 복원)
        //   - 다른 id (히스토리 없음) = "새 번호 시작" (리셋)
        if self.current_id != Some(numbering_id) {
            if let Some(prev_id) = self.current_id {
                self.history.insert(prev_id, self.counters);
            }
            if let Some(saved) = self.history.get(&numbering_id).copied() {
                // 이전에 사용한 id → 히스토리에서 복원
                self.counters = saved;
            } else {
                // 처음 등장하는 id → 상위 레벨 카운터 상속, 현재 레벨 이하 리셋
                let prev = self.counters;
                self.counters = [0; 7];
                self.counters[..level].copy_from_slice(&prev[..level]);
            }
            self.current_id = Some(numbering_id);
        }

        // restart 모드 처리
        match restart {
            Some(NumberingRestart::ContinuePrevious) => {
                // 히스토리에서 복원 (이미 위에서 처리됨) — 카운터 증가만
            }
            Some(NumberingRestart::NewStart(start)) => {
                // 해당 수준의 카운터를 지정 값 - 1로 설정 (advance에서 +1 하므로)
                self.counters[level] = start.saturating_sub(1);
                // 하위 수준 리셋
                for i in (level + 1)..7 {
                    self.counters[i] = 0;
                }
            }
            None => {
                // 기본: 앞 번호 목록에 이어
            }
        }

        // 현재 수준 증가
        self.counters[level] += 1;


        // 하위 수준 리셋
        for i in (level + 1)..7 {
            self.counters[i] = 0;
        }

        self.counters
    }

}

/// 레이아웃 엔진
/// 레이아웃 검증 경고: 요소가 페이지 경계를 초과한 경우
#[derive(Debug, Clone)]
pub struct LayoutOverflow {
    /// 페이지 번호 (0-based)
    pub page_index: u32,
    /// 단 번호 (0-based)
    pub column_index: usize,
    /// 문단 인덱스
    pub para_index: usize,
    /// 요소 종류
    pub item_type: &'static str,
    /// 요소의 실제 Y 좌표 (배치 후)
    pub element_y: f64,
    /// 단 영역 하단 Y 좌표
    pub column_bottom: f64,
    /// 초과량 (px)
    pub overflow_px: f64,
}

impl std::fmt::Display for LayoutOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LAYOUT_OVERFLOW: page={}, col={}, para={}, type={}, y={:.1}, bottom={:.1}, overflow={:.1}px",
            self.page_index, self.column_index, self.para_index,
            self.item_type, self.element_y, self.column_bottom, self.overflow_px)
    }
}

/// 어울림 문단의 마지막 TextRun에 is_para_end를 강제 설정 (↵ 표시용)
fn force_para_end_on_last_run(col_node: &mut RenderNode) {
    if let Some(line_node) = col_node.children.last_mut() {
        if let Some(run_node) = line_node.children.last_mut() {
            if let RenderNodeType::TextRun(ref mut tr) = run_node.node_type {
                tr.is_para_end = true;
            }
        }
    }
}

pub struct LayoutEngine {
    /// DPI
    dpi: f64,
    /// 자동 번호 카운터
    auto_counter: std::cell::RefCell<AutoNumberCounter>,
    /// 문단 번호 상태
    numbering_state: std::cell::RefCell<NumberingState>,
    /// 투명선 표시 여부
    show_transparent_borders: std::cell::Cell<bool>,
    /// 잘림 보기: false이면 Body/셀 클립 해제
    clip_enabled: std::cell::Cell<bool>,
    /// 머리말/꼬리말 감추기 세트: (global_page_index, is_header)
    hidden_header_footer: std::cell::RefCell<std::collections::HashSet<(u32, bool)>>,
    /// 총 쪽수 (머리말/꼬리말 필드 치환용)
    total_pages: std::cell::Cell<u32>,
    /// 현재 페이지 번호 (바탕쪽 글상자 쪽번호 치환용)
    current_page_number: std::cell::Cell<u32>,
    /// 파일 이름 (머리말/꼬리말 필드 치환용)
    file_name: std::cell::RefCell<String>,
    /// 문단 테두리/배경 범위 수집
    /// (border_fill_id, x, y_start, width, y_end, top_inset, bottom_inset,
    ///  is_partial_start, is_partial_end, para_index)
    /// is_partial_start: 다른 컬럼/페이지에서 이어진 부분 (top edge 미렌더링)
    /// is_partial_end: 다음 컬럼/페이지로 이어지는 부분 (bottom edge 미렌더링)
    /// para_index: 본 range 가 속한 paragraph 인덱스 (Task #468: cross-column 박스 연속 검출용)
    para_border_ranges: std::cell::RefCell<Vec<(u16, f64, f64, f64, f64, f64, f64, bool, bool, usize)>>,
    /// 문단 외곽선 box geometry override (Task #463): wrap=Square 호스트 문단의
    /// 텍스트는 좁은 wrap_area 에서 layout 되지만, 외곽선은 원래 col_area 의
    /// 전체 너비로 그려야 PDF 와 일치한다 (인라인 floating 표를 박스가 둘러쌈).
    /// `layout_wrap_around_paras` 가 호출 직전에 Some(원래 col_area.x, col_area.width)
    /// 로 설정하고, 호출 직후 None 으로 복원한다.
    border_box_override: std::cell::Cell<Option<(f64, f64)>>,
    /// 레이아웃 검증 결과: 경계 초과 목록
    layout_overflows: std::cell::RefCell<Vec<LayoutOverflow>>,
    /// 빈 줄 감추기로 높이 0 처리된 문단 인덱스 집합
    hidden_empty_paras: std::cell::RefCell<std::collections::HashSet<usize>>,
    /// 현재 활성 필드 위치 — 안내문 렌더링 스킵용
    /// (section_idx, para_idx, control_idx, cell_path)
    /// cell_path: 셀 내 필드일 경우 Some(Vec<(ctrl, cell, para)>)
    active_field: std::cell::RefCell<Option<(usize, usize, usize, Option<Vec<(usize, usize, usize)>>)>>,
    /// 조판부호 표시 여부
    show_control_codes: std::cell::Cell<bool>,
    /// 현재 페이지 용지 너비 (표 HorzRelTo::Paper 위치 계산용)
    current_paper_width: std::cell::Cell<f64>,
    /// 현재 페이지 본문 영역 (표 HorzRelTo::Page / VertRelTo::Page 위치 계산용)
    /// (x, y, width, height). 미설정 시 (0, 0, 0, 0) — 호출부에서 col_area로 폴백.
    current_body_area: std::cell::Cell<(f64, f64, f64, f64)>,
}

mod text_measurement;
mod paragraph_layout;
mod table_layout;
mod table_partial;
mod table_cell_content;
mod shape_layout;
mod picture_footnote;
mod border_rendering;
mod utils;

pub(crate) use text_measurement::{resolved_to_text_style, estimate_text_width, estimate_text_width_unrounded, compute_char_positions, is_cjk_char, split_into_clusters, find_next_tab_stop, extract_tab_leaders_with_extended};
pub(crate) use paragraph_layout::{map_pua_bullet_char, ensure_min_baseline};
pub(crate) use utils::{resolve_numbering_id, find_bin_data, drawing_to_shape_style, drawing_to_line_style, layout_rect_to_bbox, format_page_number};
pub(crate) use border_rendering::{border_width_to_px, create_border_line_nodes};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod integration_tests;

impl LayoutEngine {
    pub fn new(dpi: f64) -> Self {
        Self {
            dpi,
            auto_counter: std::cell::RefCell::new(AutoNumberCounter::new()),
            numbering_state: std::cell::RefCell::new(NumberingState::default()),
            show_transparent_borders: std::cell::Cell::new(false),
            clip_enabled: std::cell::Cell::new(true),
            hidden_header_footer: std::cell::RefCell::new(std::collections::HashSet::new()),
            total_pages: std::cell::Cell::new(0),
            current_page_number: std::cell::Cell::new(0),
            file_name: std::cell::RefCell::new(String::new()),
            para_border_ranges: std::cell::RefCell::new(Vec::new()),
            border_box_override: std::cell::Cell::new(None),
            layout_overflows: std::cell::RefCell::new(Vec::new()),
            hidden_empty_paras: std::cell::RefCell::new(std::collections::HashSet::new()),
            active_field: std::cell::RefCell::new(None),
            show_control_codes: std::cell::Cell::new(false),
            current_paper_width: std::cell::Cell::new(0.0),
            current_body_area: std::cell::Cell::new((0.0, 0.0, 0.0, 0.0)),
        }
    }

    /// 기본 DPI(96)로 생성
    pub fn with_default_dpi() -> Self {
        Self::new(DEFAULT_DPI)
    }

    /// 레이아웃 검증 결과 조회 및 리셋
    pub fn take_overflows(&self) -> Vec<LayoutOverflow> {
        self.layout_overflows.borrow_mut().drain(..).collect()
    }

    /// 레이아웃 경계 초과 기록
    fn record_overflow(&self, overflow: LayoutOverflow) {
        eprintln!("{}", overflow);
        self.layout_overflows.borrow_mut().push(overflow);
    }

    /// 빈 줄 감추기 문단 집합 설정
    pub fn set_hidden_empty_paras(&self, paras: &std::collections::HashSet<usize>) {
        *self.hidden_empty_paras.borrow_mut() = paras.clone();
    }

    /// 번호 상태를 초기화한다.
    pub fn reset_numbering_state(&self) {
        self.numbering_state.borrow_mut().reset();
    }

    /// 이미 렌더된 인라인 이미지 노드의 y 좌표를 dy만큼 이동 (캡션 Top 보정)
    fn offset_inline_image_y(node: &mut RenderNode, para_index: usize, control_index: usize, dy: f64) {
        for child in node.children.iter_mut() {
            if let RenderNodeType::Image(ref img) = child.node_type {
                if img.para_index == Some(para_index) && img.control_index == Some(control_index) {
                    child.bbox.y += dy;
                    return;
                }
            }
            // 재귀 탐색 (line_node 등 하위 노드)
            Self::offset_inline_image_y(child, para_index, control_index, dy);
        }
    }

    /// 번호 카운터를 진행시킨다 (이전 페이지 문단의 번호 재계산용).
    pub fn advance_numbering(&self, numbering_id: u16, level: u8) {
        self.numbering_state.borrow_mut().advance(numbering_id, level, None);
    }

    /// 잘림 보기 여부를 설정한다.
    pub fn set_clip_enabled(&self, enabled: bool) {
        self.clip_enabled.set(enabled);
    }

    /// 투명선 표시 여부를 설정한다.
    pub fn set_show_transparent_borders(&self, enabled: bool) {
        self.show_transparent_borders.set(enabled);
    }

    /// 머리말/꼬리말 감추기 세트를 설정한다.
    pub fn set_hidden_header_footer(&self, hidden: &std::collections::HashSet<(u32, bool)>) {
        *self.hidden_header_footer.borrow_mut() = hidden.clone();
    }

    /// 총 쪽수를 설정한다 (머리말/꼬리말 필드 치환용).
    pub fn set_total_pages(&self, total: u32) {
        self.total_pages.set(total);
    }

    /// 파일 이름을 설정한다 (머리말/꼬리말 필드 치환용).
    pub fn set_file_name(&self, name: &str) {
        *self.file_name.borrow_mut() = name.to_string();
    }

    /// 활성 필드 설정 (안내문 렌더링 스킵용)
    pub fn set_active_field(&self, info: Option<(usize, usize, usize, Option<Vec<(usize, usize, usize)>>)>) {
        *self.active_field.borrow_mut() = info;
    }

    /// 조판부호 표시 여부 설정
    pub fn set_show_control_codes(&self, enabled: bool) {
        self.show_control_codes.set(enabled);
    }

    /// 자동 번호 카운터 초기화
    pub fn reset_auto_counter(&self) {
        self.auto_counter.borrow_mut().reset();
    }

    /// 페이지 분할 결과와 원본 문단으로부터 렌더 트리를 생성한다.
    ///
    /// - `paragraphs`: 본문 구역의 문단 슬라이스
    /// - `header_paragraphs`: 머리말 컨트롤이 속한 구역의 문단 슬라이스 (구역 간 상속 시 다를 수 있음)
    /// - `footer_paragraphs`: 꼬리말 컨트롤이 속한 구역의 문단 슬라이스
    pub fn build_render_tree(
        &self,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        header_paragraphs: &[Paragraph],
        footer_paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        footnote_shape: &FootnoteShape,
        bin_data_content: &[BinDataContent],
        active_master_page: Option<&MasterPage>,
        measured_tables: &[MeasuredTable],
        page_border_fill: Option<&crate::model::page::PageBorderFill>,
        outline_numbering_id: u16,
        wrap_around_paras: &[super::pagination::WrapAroundPara],
    ) -> PageRenderTree {
        let layout = &page_content.layout;
        let mut tree = PageRenderTree::new(
            page_content.page_index,
            layout.page_width,
            layout.page_height,
        );

        // 페이지 배경
        self.build_page_background(&mut tree, layout, page_border_fill, styles, bin_data_content);

        // 쪽 테두리선
        self.build_page_borders(&mut tree, layout, page_border_fill, styles);

        // 바탕쪽 (감추기 설정 시 건너뜀)
        let hide_master = page_content.page_hide.as_ref()
            .map(|ph| ph.hide_master_page).unwrap_or(false);
        if !hide_master {
            self.build_master_page(
                &mut tree, active_master_page, layout, composed, styles,
                bin_data_content, page_content.section_index, page_content.page_number,
            );
        }

        // 머리말 (감추기 설정 시 건너뜀)
        let hide_header = page_content.page_hide.as_ref()
            .map(|ph| ph.hide_header).unwrap_or(false);
        if !hide_header {
            self.build_header(&mut tree, page_content, header_paragraphs, composed, styles, layout, bin_data_content);
        }

        // 본문 영역 노드 (clip_rect은 콘텐츠 레이아웃 후 확정)
        let body_id = tree.next_id();
        let body_bbox = layout_rect_to_bbox(&layout.body_area);
        let mut body_node = RenderNode::new(
            body_id,
            RenderNodeType::Body {
                clip_rect: None, // 레이아웃 후 설정
            },
            body_bbox,
        );

        // 단별 콘텐츠 레이아웃
        let mut paper_images: Vec<RenderNode> = Vec::new();
        self.build_columns(
            &mut tree, &mut body_node, &mut paper_images,
            page_content, paragraphs, composed, styles,
            bin_data_content, measured_tables, layout, outline_numbering_id,
            wrap_around_paras,
        );

        // 단 구분선
        self.build_column_separators(&mut tree, &mut body_node, layout);

        // 콘텐츠 레이아웃 후 clip_rect 확정:
        // 자식 노드(표 등)의 실제 바운딩 박스를 재귀적으로 반영하여
        // body_area보다 큰 콘텐츠(표 외곽 테두리 등)가 잘리지 않도록 함
        if self.clip_enabled.get() {
            let mut clip = body_bbox;
            fn expand_clip(clip: &mut BoundingBox, node: &RenderNode) {
                let cb = &node.bbox;
                let child_bottom = cb.y + cb.height;
                let child_right = cb.x + cb.width;
                let clip_bottom = clip.y + clip.height;
                let clip_right = clip.x + clip.width;
                if child_bottom > clip_bottom {
                    clip.height = child_bottom - clip.y;
                }
                if child_right > clip_right {
                    clip.width = child_right - clip.x;
                }
                if cb.x < clip.x {
                    clip.width += clip.x - cb.x;
                    clip.x = cb.x;
                }
                if cb.y < clip.y {
                    clip.height += clip.y - cb.y;
                    clip.y = cb.y;
                }
                for child in &node.children {
                    expand_clip(clip, child);
                }
            }
            for child in &body_node.children {
                expand_clip(&mut clip, child);
            }
            body_node.node_type = RenderNodeType::Body {
                clip_rect: Some(clip),
            };
        }

        // 용지 기준 이미지: body clip 바깥에 배치 (배경 이미지 등)
        for img_node in paper_images {
            tree.root.children.push(img_node);
        }

        tree.root.children.push(body_node);

        // 각주 영역
        self.build_footnote_area(&mut tree, page_content, paragraphs, footnote_shape, styles, layout);

        // 꼬리말 + 쪽 번호 (감추기 설정 시 건너뜀)
        let hide_footer = page_content.page_hide.as_ref()
            .map(|ph| ph.hide_footer).unwrap_or(false);
        let mut footer_node = if !hide_footer {
            self.build_footer(&mut tree, page_content, footer_paragraphs, composed, styles, layout, bin_data_content)
        } else {
            let fid = tree.next_id();
            RenderNode::new(fid, RenderNodeType::Footer, layout_rect_to_bbox(&layout.footer_area))
        };
        self.build_page_number(&mut tree, &mut footer_node, page_content, layout);
        tree.root.children.push(footer_node);

        tree
    }

    /// 머리말/꼬리말 문단을 해당 영역에 레이아웃한다.
    fn layout_header_footer_paragraphs(
        &self,
        tree: &mut PageRenderTree,
        area_node: &mut RenderNode,
        hf_paragraphs: &[Paragraph],
        _composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        area: &LayoutRect,
        page_index: u32,
        page_number: u32,
        bin_data_content: &[BinDataContent],
    ) {
        let mut y_offset = area.y;
        for (i, para) in hf_paragraphs.iter().enumerate() {
            // 테이블 컨트롤이 있으면 테이블 렌더링
            let has_table = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            let has_shape = para.controls.iter().any(|c| matches!(c, Control::Shape(_)));
            let has_picture = para.controls.iter().any(|c| matches!(c, Control::Picture(_)));
            if has_table {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(t) = ctrl {
                        let alignment = styles.para_styles
                            .get(para.para_shape_id as usize)
                            .map(|s| s.alignment)
                            .unwrap_or(Alignment::Left);
                        // Task #445: 머리말/꼬리말 영역의 wrap=TopAndBottom + vert=Para 표는
                        // 첫 라인의 line_height/2 만큼 아래로 anchor 됨 (HWP 가 line center
                        // 기준으로 표를 배치하는 동작과 일치). 이 보정이 없으면 페이지 번호
                        // 박스가 본문 바닥과 붙어 보이는 문제(Task #445) 발생.
                        let line_anchor_offset = if matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                            && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                            && i == 0
                        {
                            let lh_hu = para.line_segs.first().map(|ls| ls.line_height as i32).unwrap_or(0);
                            hwpunit_to_px(lh_hu, self.dpi) / 2.0
                        } else {
                            0.0
                        };
                        let table_y = y_offset + line_anchor_offset;
                        y_offset = self.layout_table(
                            tree, area_node, t,
                            0, styles, area, table_y, bin_data_content,
                            None, 0,
                            Some((i, ci)), alignment,
                            None, 0.0, 0.0, None, None, None,
                        );
                    }
                }
            } else if has_picture {
                // Picture 컨트롤이 있는 문단
                let mut comp = compose_paragraph(para);
                self.substitute_hf_field_markers(&mut comp, page_number);
                if comp.tac_controls.is_empty() {
                    // 머리말/꼬리말 내 Picture: header/footer area 기준 배치
                    for (_ci, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Picture(pic) = ctrl {
                            let pic_container = LayoutRect {
                                x: area.x,
                                y: y_offset,
                                width: area.width,
                                height: area.height - (y_offset - area.y),
                            };
                            self.layout_picture(
                                tree, area_node, pic, &pic_container,
                                bin_data_content, Alignment::Left, None, None, None,
                            );
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            y_offset += pic_h;
                        }
                    }
                } else {
                    // TAC Picture: layout_paragraph에서 인라인 배치
                    y_offset = self.layout_paragraph(
                        tree, area_node, para, Some(&comp), styles, area, y_offset,
                        0, usize::MAX - i, None, Some(bin_data_content),
                    );
                }
            } else if has_shape {
                // Shape 컨트롤 렌더링 (머리말/꼬리말 내 글상자 등)
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(_) = ctrl {
                        self.layout_shape(
                            tree, area_node,
                            hf_paragraphs, i, ci,
                            0, // section_index
                            styles, area, area, area,
                            y_offset, Alignment::Left,
                            bin_data_content, &std::collections::HashMap::new(),
                        );
                    }
                }
                // 텍스트도 함께 렌더링
                if !para.text.is_empty() {
                    let mut comp = compose_paragraph(para);
                    self.substitute_hf_field_markers(&mut comp, page_number);
                    y_offset = self.layout_paragraph(
                        tree, area_node, para, Some(&comp), styles, area, y_offset,
                        0, usize::MAX - i, None, None,
                    );
                }
            } else {
                // 일반 텍스트 문단 레이아웃 (필드 마커 치환 포함)
                let mut comp = compose_paragraph(para);
                self.substitute_hf_field_markers(&mut comp, page_number);
                y_offset = self.layout_paragraph(
                    tree, area_node, para, Some(&comp), styles, area, y_offset,
                    0, usize::MAX - i, None, None,
                );
            }
            if y_offset >= area.y + area.height {
                break;
            }
        }
    }

    /// 머리말/꼬리말 ComposedParagraph의 필드 마커를 실제 값으로 치환한다.
    /// - `\u{0015}` → 현재 쪽번호
    /// - `\u{0016}` → 총 쪽수
    /// - `\u{0017}` → 파일 이름
    fn substitute_hf_field_markers(&self, comp: &mut ComposedParagraph, page_number: u32) {
        let total = self.total_pages.get();
        let file_name = self.file_name.borrow();
        let page_str = page_number.to_string();
        let total_str = total.to_string();

        for line in &mut comp.lines {
            let mut new_runs = Vec::new();
            for run in &line.runs {
                if !run.text.contains('\u{0015}') && !run.text.contains('\u{0016}') && !run.text.contains('\u{0017}') {
                    new_runs.push(run.clone());
                    continue;
                }
                // 마커가 포함된 런 → 치환 후 분할
                let replaced = run.text
                    .replace('\u{0015}', &page_str)
                    .replace('\u{0016}', &total_str)
                    .replace('\u{0017}', &file_name);
                let mut new_run = run.clone();
                new_run.text = replaced;
                new_runs.push(new_run);
            }
            line.runs = new_runs;
        }
    }

    /// 페이지 배경 노드를 생성하여 tree에 추가한다.
    fn build_page_background(
        &self,
        tree: &mut PageRenderTree,
        layout: &PageLayoutInfo,
        page_border_fill: Option<&crate::model::page::PageBorderFill>,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
    ) {
        let (page_bg_color, page_bg_gradient, page_bg_image) = if let Some(pbf) = page_border_fill {
            if pbf.border_fill_id > 0 {
                let bf_idx = (pbf.border_fill_id - 1) as usize;
                if let Some(bs) = styles.border_styles.get(bf_idx) {
                    let img = bs.image_fill.as_ref().and_then(|img_fill| {
                        find_bin_data(bin_data_content, img_fill.bin_data_id)
                            .map(|c| PageBackgroundImage {
                                data: c.data.clone(),
                                fill_mode: img_fill.fill_mode,
                            })
                    });
                    (bs.fill_color.or(Some(0x00FFFFFF)), bs.gradient.clone(), img)
                } else {
                    (Some(0x00FFFFFF), None, None)
                }
            } else {
                (Some(0x00FFFFFF), None, None)
            }
        } else {
            (Some(0x00FFFFFF), None, None)
        };

        let fill_area = page_border_fill.map(|pbf| (pbf.attr >> 3) & 0x03).unwrap_or(0);
        let bg_bbox = match fill_area {
            1 => BoundingBox::new(layout.body_area.x, layout.body_area.y, layout.body_area.width, layout.body_area.height),
            _ => BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
        };

        let bg_id = tree.next_id();
        let bg_node = RenderNode::new(
            bg_id,
            RenderNodeType::PageBackground(PageBackgroundNode {
                background_color: page_bg_color,
                border_color: None,
                border_width: 0.0,
                gradient: page_bg_gradient,
                image: page_bg_image,
            }),
            bg_bbox,
        );
        tree.root.children.push(bg_node);
    }

    /// 쪽 테두리선을 렌더링하여 tree에 추가한다.
    fn build_page_borders(
        &self,
        tree: &mut PageRenderTree,
        layout: &PageLayoutInfo,
        page_border_fill: Option<&crate::model::page::PageBorderFill>,
        styles: &ResolvedStyleSet,
    ) {
        if let Some(pbf) = page_border_fill.filter(|p| p.border_fill_id > 0) {
            let bf_idx = (pbf.border_fill_id - 1) as usize;
            if let Some(bs) = styles.border_styles.get(bf_idx) {
                let paper_based = (pbf.attr & 0x01) != 0;
                let (base_x, base_y, base_w, base_h) = if paper_based {
                    (0.0, 0.0, layout.page_width, layout.page_height)
                } else {
                    (layout.body_area.x, layout.body_area.y, layout.body_area.width, layout.body_area.height)
                };

                let sp_l = hwpunit_to_px(pbf.spacing_left as i32, self.dpi);
                let sp_r = hwpunit_to_px(pbf.spacing_right as i32, self.dpi);
                let sp_t = hwpunit_to_px(pbf.spacing_top as i32, self.dpi);
                let sp_b = hwpunit_to_px(pbf.spacing_bottom as i32, self.dpi);
                let bx = base_x + sp_l;
                let by = base_y + sp_t;
                let bw = base_w - sp_l - sp_r;
                let bh = base_h - sp_t - sp_b;

                let borders = &bs.borders;
                let top_nodes = create_border_line_nodes(tree, &borders[2], bx, by, bx + bw, by);
                for n in top_nodes { tree.root.children.push(n); }
                let bottom_nodes = create_border_line_nodes(tree, &borders[3], bx, by + bh, bx + bw, by + bh);
                for n in bottom_nodes { tree.root.children.push(n); }
                let left_nodes = create_border_line_nodes(tree, &borders[0], bx, by, bx, by + bh);
                for n in left_nodes { tree.root.children.push(n); }
                let right_nodes = create_border_line_nodes(tree, &borders[1], bx + bw, by, bx + bw, by + bh);
                for n in right_nodes { tree.root.children.push(n); }
            }
        }
    }

    /// 확장 바탕쪽을 기존 렌더 트리에 추가한다 (외부 호출용).
    pub(crate) fn build_master_page_into(
        &self,
        tree: &mut PageRenderTree,
        active_master_page: Option<&MasterPage>,
        layout: &PageLayoutInfo,
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        section_index: usize,
        page_number: u32,
    ) {
        self.build_master_page(tree, active_master_page, layout, composed, styles, bin_data_content, section_index, page_number);
    }

    /// 바탕쪽 영역 노드를 생성하여 tree에 추가한다.
    fn build_master_page(
        &self,
        tree: &mut PageRenderTree,
        active_master_page: Option<&MasterPage>,
        layout: &PageLayoutInfo,
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        section_index: usize,
        page_number: u32,
    ) {
        if let Some(mp) = active_master_page {
            // 영역 0×0 바탕쪽은 MEMO 컨트롤 오분류 방어용 가드 — 렌더링 skip
            if mp.text_width == 0 && mp.text_height == 0 {
                return;
            }
            self.current_page_number.set(page_number);
            if !mp.paragraphs.is_empty() {
                let mp_id = tree.next_id();
                let paper_area = LayoutRect {
                    x: 0.0, y: 0.0,
                    width: layout.page_width,
                    height: layout.page_height,
                };
                let body_area = &layout.body_area;
                let mut mp_node = RenderNode::new(
                    mp_id,
                    RenderNodeType::MasterPage,
                    layout_rect_to_bbox(&paper_area),
                );
                // 바탕쪽 문단 렌더링: 컨트롤(표/도형/그림)은 compute_object_position으로 배치,
                // 텍스트 문단은 layout_paragraph로 배치
                let mut mp_y_offset = paper_area.y;
                for (pi, para) in mp.paragraphs.iter().enumerate() {
                    let has_controls = !para.controls.is_empty();
                    if has_controls {
                        for (ci, ctrl) in para.controls.iter().enumerate() {
                            match ctrl {
                                Control::Shape(_) | Control::Equation(_) => {
                                    self.layout_shape(
                                        tree, &mut mp_node,
                                        &mp.paragraphs, pi, ci,
                                        section_index,
                                        styles, body_area, body_area, &paper_area,
                                        body_area.y, Alignment::Left,
                                        bin_data_content,
                                        &std::collections::HashMap::new(),
                                    );
                                }
                                Control::Picture(pic) => {
                                    let (pic_w, pic_h) = self.resolve_object_size(
                                        &pic.common, body_area, body_area, &paper_area,
                                    );
                                    let (pic_x, pic_y) = self.compute_object_position(
                                        &pic.common, pic_w, pic_h,
                                        body_area, body_area, body_area, &paper_area,
                                        body_area.y, Alignment::Left,
                                    );
                                    let pic_area = super::layout::LayoutRect {
                                        x: pic_x, y: pic_y, width: pic_w, height: pic_h,
                                    };
                                    self.layout_picture(
                                        tree, &mut mp_node, pic, &pic_area,
                                        bin_data_content, Alignment::Left,
                                        Some(section_index), None, None,
                                    );
                                }
                                Control::Table(t) => {
                                    let alignment = styles.para_styles
                                        .get(para.para_shape_id as usize)
                                        .map(|s| s.alignment)
                                        .unwrap_or(Alignment::Left);
                                    // 바탕쪽 표: paper_area를 col_area로 전달하여
                                    // compute_table_x/y_position이 올바르게 위치 계산
                                    self.layout_table(
                                        tree, &mut mp_node,
                                        t, section_index,
                                        styles, &paper_area, 0.0,
                                        bin_data_content, None, 0,
                                        Some((pi, ci)), alignment,
                                        None, 0.0, 0.0, None, None, None,
                                    );
                                }
                                _ => {}
                            }
                        }
                    } else if !para.text.is_empty() {
                        // 컨트롤 없는 텍스트 문단: vpos 기반 y 위치 사용
                        let mut comp = compose_paragraph(para);
                        self.substitute_hf_field_markers(&mut comp, page_number);
                        // 바탕쪽 탭은 레이아웃 위치 지정용이므로 탭 리더를 그리지 않음
                        comp.tab_extended.clear();
                        // LINE_SEG vpos로 문단 시작 y 결정 (빈 문단 건너뜀 보상)
                        if let Some(first_ls) = para.line_segs.first() {
                            let vpos_y = paper_area.y + hwpunit_to_px(first_ls.vertical_pos, self.dpi);
                            if vpos_y > mp_y_offset {
                                mp_y_offset = vpos_y;
                            }
                        }
                        mp_y_offset = self.layout_paragraph(
                            tree, &mut mp_node, para, Some(&comp), styles,
                            &paper_area, mp_y_offset,
                            0, usize::MAX - pi, None, None,
                        );
                    } else {
                        // 빈 문단: LINE_SEG vpos로 y 위치 갱신
                        if let Some(first_ls) = para.line_segs.first() {
                            let vpos_y = paper_area.y + hwpunit_to_px(first_ls.vertical_pos, self.dpi);
                            let lh = hwpunit_to_px(first_ls.line_height, self.dpi);
                            let ls = hwpunit_to_px(first_ls.line_spacing, self.dpi);
                            mp_y_offset = (vpos_y + lh + ls).max(mp_y_offset);
                        }
                    }
                }
                tree.root.children.push(mp_node);
            }
        }
    }

    /// 머리말 영역 노드를 생성하여 tree에 추가한다.
    fn build_header(
        &self,
        tree: &mut PageRenderTree,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        layout: &PageLayoutInfo,
        bin_data_content: &[BinDataContent],
    ) {
        self.current_page_number.set(page_content.page_number);
        let header_id = tree.next_id();
        let mut header_node = RenderNode::new(
            header_id,
            RenderNodeType::Header,
            layout_rect_to_bbox(&layout.header_area),
        );
        // 감추기 플래그가 설정된 페이지는 머리말 내용을 렌더링하지 않음
        let hidden = self.hidden_header_footer.borrow()
            .contains(&(page_content.page_index, true));
        if !hidden {
            if let Some(hf_ref) = &page_content.active_header {
                if let Some(para) = paragraphs.get(hf_ref.para_index) {
                    if let Some(ctrl) = para.controls.get(hf_ref.control_index) {
                        if let Control::Header(header) = ctrl {
                            self.layout_header_footer_paragraphs(
                                tree, &mut header_node,
                                &header.paragraphs, composed, styles,
                                &layout.header_area,
                                page_content.page_index,
                                page_content.page_number,
                                bin_data_content,
                            );
                        }
                    }
                }
            }
        }
        // Header bbox를 자식 노드 범위까지 확장 + 셀 클리핑 해제
        // (머리말 표 셀 내 Shape가 header_area 밖에 배치될 수 있음)
        Self::expand_bbox_to_children(&mut header_node);
        Self::disable_cell_clip_recursive(&mut header_node);
        tree.root.children.push(header_node);
    }

    /// 노드의 bbox를 자식 노드 범위까지 확장
    fn expand_bbox_to_children(node: &mut RenderNode) {
        let mut min_x = node.bbox.x;
        let mut min_y = node.bbox.y;
        let mut max_x = node.bbox.x + node.bbox.width;
        let mut max_y = node.bbox.y + node.bbox.height;
        for child in &node.children {
            min_x = min_x.min(child.bbox.x);
            min_y = min_y.min(child.bbox.y);
            max_x = max_x.max(child.bbox.x + child.bbox.width);
            max_y = max_y.max(child.bbox.y + child.bbox.height);
        }
        node.bbox.x = min_x;
        node.bbox.y = min_y;
        node.bbox.width = max_x - min_x;
        node.bbox.height = max_y - min_y;
    }

    /// 자식 노드의 TableCell clip을 재귀적으로 해제
    fn disable_cell_clip_recursive(node: &mut RenderNode) {
        if let RenderNodeType::TableCell(ref mut tc) = node.node_type {
            tc.clip = false;
        }
        for child in &mut node.children {
            Self::disable_cell_clip_recursive(child);
        }
    }

    /// 꼬리말 영역 노드를 생성하여 반환한다.
    fn build_footer(
        &self,
        tree: &mut PageRenderTree,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        layout: &PageLayoutInfo,
        bin_data_content: &[BinDataContent],
    ) -> RenderNode {
        self.current_page_number.set(page_content.page_number);
        let footer_id = tree.next_id();
        let mut footer_node = RenderNode::new(
            footer_id,
            RenderNodeType::Footer,
            layout_rect_to_bbox(&layout.footer_area),
        );
        // 감추기 플래그가 설정된 페이지는 꼬리말 내용을 렌더링하지 않음
        let hidden = self.hidden_header_footer.borrow()
            .contains(&(page_content.page_index, false));
        if !hidden {
            if let Some(hf_ref) = &page_content.active_footer {
                if let Some(para) = paragraphs.get(hf_ref.para_index) {
                    if let Some(ctrl) = para.controls.get(hf_ref.control_index) {
                        if let Control::Footer(footer) = ctrl {
                            self.layout_header_footer_paragraphs(
                                tree, &mut footer_node,
                                &footer.paragraphs, composed, styles,
                                &layout.footer_area,
                                page_content.page_index,
                                page_content.page_number,
                                bin_data_content,
                            );
                        }
                    }
                }
            }
        }
        Self::expand_bbox_to_children(&mut footer_node);
        Self::disable_cell_clip_recursive(&mut footer_node);
        footer_node
    }

    /// 단 구분선을 렌더링하여 body_node에 추가한다.
    fn build_column_separators(
        &self,
        tree: &mut PageRenderTree,
        body_node: &mut RenderNode,
        layout: &PageLayoutInfo,
    ) {
        if layout.column_areas.len() >= 2 && layout.separator_type > 0 {
            let line_width = border_width_to_px(layout.separator_width).max(0.5);
            let dash = match layout.separator_type {
                2 => StrokeDash::Dash,
                3 => StrokeDash::Dot,
                4 => StrokeDash::DashDot,
                5 => StrokeDash::DashDotDot,
                _ => StrokeDash::Solid,
            };
            for i in 0..layout.column_areas.len() - 1 {
                let left_col = &layout.column_areas[i];
                let right_col = &layout.column_areas[i + 1];
                let sep_x = (left_col.x + left_col.width + right_col.x) / 2.0;
                let sep_y1 = left_col.y;
                let sep_y2 = left_col.y + left_col.height;
                let sep_id = tree.next_id();
                let sep_node = RenderNode::new(
                    sep_id,
                    RenderNodeType::Line(LineNode::new(
                        sep_x, sep_y1, sep_x, sep_y2,
                        LineStyle {
                            color: layout.separator_color,
                            width: line_width,
                            dash,
                            ..Default::default()
                        },
                    )),
                    BoundingBox::new(sep_x - line_width / 2.0, sep_y1, line_width, sep_y2 - sep_y1),
                );
                body_node.children.push(sep_node);
            }
        }
    }

    /// 각주 영역 노드를 생성하여 tree에 추가한다.
    fn build_footnote_area(
        &self,
        tree: &mut PageRenderTree,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        footnote_shape: &FootnoteShape,
        styles: &ResolvedStyleSet,
        layout: &PageLayoutInfo,
    ) {
        let mut footnote_layout = layout.clone();
        if !page_content.footnotes.is_empty() {
            let fn_height = self.estimate_footnote_area_height(
                &page_content.footnotes, paragraphs, footnote_shape,
            );
            footnote_layout.update_footnote_area(fn_height);
        }

        if !page_content.footnotes.is_empty() {
            let fn_id = tree.next_id();
            let mut fn_node = RenderNode::new(
                fn_id,
                RenderNodeType::FootnoteArea,
                layout_rect_to_bbox(&footnote_layout.footnote_area),
            );

            self.layout_footnote_area(
                tree,
                &mut fn_node,
                &page_content.footnotes,
                paragraphs,
                styles,
                &footnote_layout.footnote_area,
                footnote_shape,
            );
            tree.root.children.push(fn_node);
        }
    }

    /// 쪽 번호를 렌더링한다.
    fn build_page_number(
        &self,
        tree: &mut PageRenderTree,
        footer_node: &mut RenderNode,
        page_content: &PageContent,
        layout: &PageLayoutInfo,
    ) {
        // 감추기(PageHide)에서 쪽 번호 감추기가 설정되어 있으면 건너뜀
        if let Some(ref ph) = page_content.page_hide {
            if ph.hide_page_num {
                return;
            }
        }
        if let Some(pnp) = &page_content.page_number_pos {
            if pnp.position == 0 {
                return;
            }
            let page_num_text = format_page_number(
                page_content.page_number, pnp.format,
                pnp.prefix_char, pnp.suffix_char, pnp.dash_char,
            );
            let target_area = match pnp.position {
                1..=3 | 7 | 9 => &layout.header_area,
                _ => &layout.footer_area,
            };

            let font_size = 10.0;
            let text_width = page_num_text.chars().count() as f64 * font_size * 0.6;

            let is_odd_page = page_content.page_number % 2 == 1;
            let x = match pnp.position {
                1 | 4 => target_area.x,
                3 | 6 => target_area.x + target_area.width - text_width,
                2 | 5 => target_area.x + (target_area.width - text_width) / 2.0,
                // 바깥쪽: 홀수쪽→오른쪽, 짝수쪽→왼쪽
                7 | 8 => if is_odd_page {
                    target_area.x + target_area.width - text_width
                } else {
                    target_area.x
                },
                // 안쪽: 홀수쪽→왼쪽, 짝수쪽→오른쪽
                9 | 10 => if is_odd_page {
                    target_area.x
                } else {
                    target_area.x + target_area.width - text_width
                },
                _ => target_area.x + (target_area.width - text_width) / 2.0,
            };

            let y = target_area.y + target_area.height / 2.0 + font_size / 3.0;

            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(font_size * 1.2, font_size)),
                BoundingBox::new(x, y - font_size, text_width, font_size * 1.2),
            );

            let run_id = tree.next_id();
            let run_node = RenderNode::new(
                run_id,
                RenderNodeType::TextRun(TextRunNode {
                    text: page_num_text,
                    style: TextStyle {
                        font_family: "바탕".to_string(),
                        font_size,
                        color: 0x000000,
                        ..Default::default()
                    },
                    char_shape_id: None,
                    para_shape_id: None,
                    section_index: None,
                    para_index: None,
                    char_start: None,
                    cell_context: None,
                    is_para_end: true,
                    is_line_break_end: false,
                    rotation: 0.0,
                    is_vertical: false,
                    char_overlap: None,
                    border_fill_id: 0,
                    baseline: font_size,
                    field_marker: FieldMarkerType::None,
                }),
                BoundingBox::new(x, y, text_width, font_size),
            );
            line_node.children.push(run_node);

            match pnp.position {
                1..=3 | 7 | 9 => tree.root.children.push(line_node),
                _ => footer_node.children.push(line_node),
            }
        }
    }

    /// 단별 콘텐츠를 레이아웃하여 body_node에 추가한다.
    #[allow(clippy::too_many_arguments)]
    fn build_columns(
        &self,
        tree: &mut PageRenderTree,
        body_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        measured_tables: &[MeasuredTable],
        layout: &PageLayoutInfo,
        outline_numbering_id: u16,
        wrap_around_paras: &[super::pagination::WrapAroundPara],
    ) {
        let mut prev_zone_y_end: f64 = 0.0;
        let mut current_zone_start_y: f64 = 0.0;
        let mut last_zone_y_offset: f64 = -1.0;

        // 다단 레이아웃: body_area 전체에 걸치는 TopAndBottom 개체의 예약 높이
        // (한 단에만 할당되더라도 모든 단에 적용)
        let body_wide_reserved: Vec<(usize, f64)> = if page_content.column_contents.len() > 1 {
            self.calculate_body_wide_shape_reserved(
                paragraphs, &page_content.column_contents, &layout.body_area,
            )
        } else {
            Vec::new()
        };

        for col_content in &page_content.column_contents {
            let zone_layout = col_content.zone_layout.as_ref().unwrap_or(layout);
            let col_idx = col_content.column_index as usize;
            let col_area_base = if col_idx < zone_layout.column_areas.len() {
                &zone_layout.column_areas[col_idx]
            } else {
                &zone_layout.body_area
            };

            let is_new_zone = (col_content.zone_y_offset - last_zone_y_offset).abs() > 0.1;
            if is_new_zone {
                if col_content.zone_y_offset > 0.0 {
                    current_zone_start_y = prev_zone_y_end;
                } else {
                    current_zone_start_y = 0.0;
                }
                last_zone_y_offset = col_content.zone_y_offset;
            }

            let col_area = if current_zone_start_y > col_area_base.y {
                LayoutRect {
                    x: col_area_base.x,
                    y: current_zone_start_y,
                    width: col_area_base.width,
                    height: (col_area_base.y + col_area_base.height - current_zone_start_y).max(0.0),
                }
            } else {
                *col_area_base
            };

            let (col_node, y_offset) = self.build_single_column(
                tree, paper_images,
                col_content, page_content,
                paragraphs, composed, styles,
                bin_data_content, measured_tables,
                layout, zone_layout, &col_area,
                outline_numbering_id,
                wrap_around_paras,
                &body_wide_reserved,
            );

            if y_offset > prev_zone_y_end {
                prev_zone_y_end = y_offset;
            }
            body_node.children.push(col_node);
        }
    }

    /// 단일 단의 콘텐츠를 레이아웃한다.
    #[allow(clippy::too_many_arguments)]
    fn build_single_column(
        &self,
        tree: &mut PageRenderTree,
        paper_images: &mut Vec<RenderNode>,
        col_content: &ColumnContent,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        measured_tables: &[MeasuredTable],
        layout: &PageLayoutInfo,
        zone_layout: &PageLayoutInfo,
        col_area: &LayoutRect,
        outline_numbering_id: u16,
        wrap_around_paras: &[super::pagination::WrapAroundPara],
        body_wide_reserved: &[(usize, f64)],
    ) -> (RenderNode, f64) {
        let col_node_id = tree.next_id();
        let mut col_node = RenderNode::new(
            col_node_id,
            RenderNodeType::Column(col_content.column_index),
            layout_rect_to_bbox(col_area),
        );

        // 현재 페이지 용지 너비 설정 (표 HorzRelTo::Paper 위치 계산용)
        self.current_paper_width.set(layout.page_width);
        // 현재 페이지 본문 영역 설정 (표 HorzRelTo::Page / VertRelTo::Page 계산용 — Task #347)
        let ba = &layout.body_area;
        self.current_body_area.set((ba.x, ba.y, ba.width, ba.height));

        // 문단 테두리 범위 수집 초기화
        self.para_border_ranges.borrow_mut().clear();

        // TopAndBottom 글상자/표/이미지의 앵커 문단별 예약 높이 목록
        let mut shape_reserved = self.calculate_shape_reserved_heights(
            paragraphs, &col_content.items, col_area, &layout.body_area,
        );
        // body_area 전체에 걸치는 개체의 예약 높이 병합 (현재 단에도 반영)
        for &(pi, bottom_y) in body_wide_reserved {
            if let Some(existing) = shape_reserved.iter_mut().find(|(p, _)| *p == pi) {
                if bottom_y > existing.1 {
                    existing.1 = bottom_y;
                }
            } else {
                shape_reserved.push((pi, bottom_y));
            }
        }
        let mut y_offset = col_area.y;
        // body_area 전체에 걸치는 개체: 단 시작 y_offset을 개체 하단 아래로 초기화
        for &(_, bottom_y) in body_wide_reserved {
            if bottom_y > y_offset {
                y_offset = bottom_y;
            }
        }
        // [Task #412] vpos 보정 anchor: 첫 PageItem 이 실제 렌더링되는 y_offset.
        // body_wide_reserved 푸시 후의 y_offset 이 첫 항목의 vpos(=base) 에 대응됨.
        // 이를 anchor 로 사용해야 vpos→y 변환이 정확함 (col_area.y 는 단 영역 top
        // 으로 vpos=0 이 아니라 vpos=base 도 아닌 일반적으로 어긋난 값).
        let col_anchor_y = y_offset;



        let mut para_start_y: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();

        let multi_col_width = if zone_layout.column_areas.len() > 1 {
            let widths: Vec<f64> = zone_layout.column_areas.iter().map(|a| a.width).collect();
            let max_w = widths.iter().cloned().fold(0.0f64, f64::max);
            let min_w = widths.iter().cloned().fold(f64::MAX, f64::min);
            let diff_hu = ((max_w - min_w) / self.dpi * 7200.0).round() as i32;
            if diff_hu > 1000 {
                Some((col_area.width / self.dpi * 7200.0).round() as i32)
            } else {
                None
            }
        } else {
            None
        };

        let col_width_hu = (col_area.width / self.dpi * 7200.0).round() as i32;
        let mut prev_layout_para: Option<usize> = None;
        let mut prev_tac_seg_applied = false;

        // 고정값 줄간격 TAC 표 병행 (Task #9): 표 하단 비교용
        let mut fix_table_start_y: f64 = 0.0;
        let mut fix_table_visual_h: f64 = 0.0;
        let mut fix_overlay_active = false;

        // vpos 보정을 위한 페이지 기준 vpos 계산
        // 페이지 첫 항목의 vpos를 기준점으로 삼아 모든 페이지에서 vpos 보정 적용
        let mut vpos_page_base: Option<i32> = col_content.items.first().and_then(|item| {
            match item {
                PageItem::FullParagraph { para_index } => {
                    paragraphs.get(*para_index)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)
                }
                PageItem::PartialParagraph { para_index, start_line, .. } => {
                    paragraphs.get(*para_index)
                        .and_then(|p| p.line_segs.get(*start_line))
                        .map(|seg| seg.vertical_pos)
                }
                PageItem::Table { para_index, .. } => {
                    paragraphs.get(*para_index)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)
                }
                // PartialTable/Shape: 지연 보정 사용
                _ => None,
            }
        });
        let mut vpos_lazy_base: Option<i32> = None;

        // 1차 패스: 표, 문단, 텍스트 렌더링 (글상자 제외)
        for item in col_content.items.iter() {
            // vpos 기반 y_offset 보정
            let item_para = match item {
                PageItem::FullParagraph { para_index } => *para_index,
                PageItem::PartialParagraph { para_index, .. } => *para_index,
                PageItem::Table { para_index, .. } => *para_index,
                PageItem::PartialTable { para_index, .. } => *para_index,
                PageItem::Shape { para_index, .. } => *para_index,
            };
            // TopAndBottom 글상자: 앵커 문단에 도달하면 y_offset을 글상자 하단 아래로 점프
            let mut shape_jumped = false;
            for &(anchor_pi, bottom_y) in &shape_reserved {
                if item_para == anchor_pi && bottom_y > y_offset {
                    y_offset = bottom_y;
                    shape_jumped = true;
                }
            }

            if !shape_jumped && !prev_tac_seg_applied {
            if let Some(prev_pi) = prev_layout_para {
                if item_para != prev_pi {
                    // 글앞으로/글뒤로/위아래 Shape·Picture가 있는 문단: vpos에 개체 높이가 포함되어 과대 → bypass
                    // - InFrontOfText/BehindText: 개체 vpos가 텍스트 라인 vpos와 별도 누적 → 합산 시 과대
                    // - TopAndBottom + vert=Para: 한컴이 후속 문단 vpos에 개체 높이를 더해 기록하므로
                    //   sequential y_offset이 이미 개체 바닥까지 진행된 상태에서 vpos 보정 lazy_base 산출
                    //   시 prev_pi의 텍스트 vpos_end만 쓰면 base가 개체 높이만큼 낮게 산출되어
                    //   다음 문단/표가 개체 높이만큼 추가 점프 (Task #409: 21페이지 차트→2x1 표 521px overflow)
                    let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
                        use crate::model::shape::{TextWrap, VertRelTo};
                        p.controls.iter().any(|c| match c {
                            Control::Shape(s) => {
                                let cm = s.common();
                                matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                                    || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                                        && matches!(cm.vert_rel_to, VertRelTo::Para)
                                        && !cm.treat_as_char)
                            }
                            Control::Picture(pic) => {
                                let cm = &pic.common;
                                if cm.treat_as_char { return false; }
                                matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                                    || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                                        && matches!(cm.vert_rel_to, VertRelTo::Para))
                            }
                            _ => false,
                        })
                    }).unwrap_or(false);
                    if !prev_has_overlay_shape {
                    if let Some(prev_para) = paragraphs.get(prev_pi) {
                        // Task #332 Stage 5: vpos correction trigger 조건 완화 —
                        // 기존엔 segment_width 가 col_width 와 ±3000 HWPUNIT 이내일 때만 적용해
                        // 짧은 단락/indent 가 있는 경우 trigger 누락 → drift 누적. 조건을 완화해
                        // 마지막 segment 를 사용하되 width 검증 자체는 가드 조건으로 약화.
                        let prev_seg = prev_para.line_segs.iter().rev().find(|ls| ls.segment_width > 0)
                            .or_else(|| prev_para.line_segs.last());
                        if let Some(seg) = prev_seg {
                            if !(seg.vertical_pos == 0 && prev_pi > 0) {
                                // [Task #412] vpos_end 결정:
                                // - page_path: 현재 paragraph 의 first seg vpos 를 직접 사용 (HWP 가 spacing_after 를
                                //   다음 paragraph 의 first vpos 에 인코딩하므로 prev.vpos+lh+ls 보다 정확).
                                //   현재 항목의 vpos 를 신뢰하지 못하는 경우(0 reset 등) prev 기반 fallback.
                                // - lazy_path: 기존 prev 기반 vpos_end 유지 (sequential 와의 역산 다리).
                                let prev_vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing;
                                let curr_first_vpos = paragraphs.get(item_para)
                                    .and_then(|p| p.line_segs.first())
                                    .map(|ls| ls.vertical_pos);
                                // [Task #412] page_base / lazy_base 경로 분리:
                                // - page_base: 첫 PageItem 이 명확한 vpos 를 가짐(FullParagraph/PartialParagraph/Table).
                                //   sequential 배치는 col_area.y + vpos*scale (절대 vpos 좌표) 으로 동작하므로
                                //   보정 공식도 base 차감 없이 col_area.y + vpos*scale 사용해야 함.
                                //   기존 base 차감은 첫 항목 vpos 만큼 보정값을 위로 어긋나게 하여
                                //   다단 우측 단(base 큼) 에서 보정이 발동하지 않는 문제 발생.
                                // - lazy_base: 첫 PageItem 이 신뢰 불가(Shape/PartialTable). sequential y_offset
                                //   으로부터 lazy_base 역산하여 단단을 잇는 다리 역할. 기존 base 차감 유지.
                                let (base, is_page_path) = if let Some(b) = vpos_page_base {
                                    (b, true)
                                } else if let Some(b) = vpos_lazy_base {
                                    (b, false)
                                } else {
                                    // 지연 보정: 첫 보정 시점에서 기준점 산출
                                    // sequential y_offset에서 역산하여 기준 vpos 결정
                                    let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32;
                                    let lazy_base = prev_vpos_end - y_delta_hu;
                                    // lazy_base가 음수이면 자리차지 표 등으로 y_offset이
                                    // vpos 누적보다 크게 밀린 것 → 역산 무효
                                    if lazy_base < 0 {
                                        // 보정 건너뛰기: base를 vpos_end로 설정하여
                                        // end_y = col_area.y + 0 → 검증 실패 → 보정 미적용
                                        (prev_vpos_end, false)
                                    } else {
                                        vpos_lazy_base = Some(lazy_base);
                                        (lazy_base, false)
                                    }
                                };
                                // [Task #412] vpos_end 결정 (page/lazy 공통):
                                // 현재 paragraph 의 first vpos 우선 사용. HWP 가 spacing_after 를 다음
                                // paragraph 의 first vpos 에 인코딩하므로 prev.vpos+lh+ls 보다 정확.
                                // vpos reset(0) 이거나 prev 보다 작아진 경우는 prev 기반 fallback.
                                let vpos_end = match curr_first_vpos {
                                    Some(v) if v > seg.vertical_pos => v,
                                    _ => prev_vpos_end,
                                };
                                // [Task #412] page_path: col_anchor_y (body_wide_reserved 푸시 적용 후) 가
                                // 첫 항목의 vpos(=base) 를 의미. 따라서 vpos=N 의 y = col_anchor_y + (N-base)*scale.
                                // lazy_path: lazy_base 는 col_area.y 가 vpos=lazy_base 가 되도록 역산되어 있어
                                //   col_area.y 기준 (vpos_end - base) 차감 공식이 일관.
                                let end_y = if is_page_path {
                                    col_anchor_y + hwpunit_to_px(vpos_end - base, self.dpi)
                                } else {
                                    col_area.y + hwpunit_to_px(vpos_end - base, self.dpi)
                                };
                                // 자가 검증: 보정값이 컬럼 영역 내에 있고
                                // 현재 y_offset보다 뒤로 가지 않아야 유효
                                let applied = end_y >= col_area.y && end_y <= col_area.y + col_area.height
                                    && end_y >= y_offset - 1.0;
                                if std::env::var("RHWP_VPOS_DEBUG").is_ok() {
                                    let path = if is_page_path { "page" } else { "lazy" };
                                    eprintln!(
                                        "VPOS_CORR: path={} pi={} prev_pi={} prev_vpos={} prev_lh={} prev_ls={} vpos_end={} base={} col_y={:.2} y_in={:.2} end_y={:.2} applied={}",
                                        path, item_para, prev_pi, seg.vertical_pos, seg.line_height, seg.line_spacing,
                                        vpos_end, base, col_area.y, y_offset, end_y, applied,
                                    );
                                }
                                if applied {
                                    y_offset = end_y;
                                }
                            }
                        }
                    }
                }
            }
            } // !prev_has_overlay_shape
            } // !shape_jumped
            prev_layout_para = Some(item_para);

            // Percent 전환: 표 하단과 비교 (Task #9)
            if fix_overlay_active {
                let is_fixed = paragraphs.get(item_para)
                    .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
                    .map(|ps| ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed)
                    .unwrap_or(false);
                if !is_fixed {
                    let table_bottom = fix_table_start_y + fix_table_visual_h;
                    if y_offset < table_bottom {
                        y_offset = table_bottom;
                    }
                    fix_overlay_active = false;
                }
            }

            let (new_y, was_tac) = self.layout_column_item(
                tree, &mut col_node, paper_images, &mut para_start_y,
                item, page_content, paragraphs, composed, styles,
                bin_data_content, measured_tables, layout, col_area,
                outline_numbering_id, multi_col_width, y_offset,
                prev_tac_seg_applied,
                wrap_around_paras,
            );
            y_offset = new_y;
            prev_tac_seg_applied = was_tac;

            // 고정값 줄간격 TAC 표 병행 (Task #9)
            if was_tac {
                if let Some(para) = paragraphs.get(item_para) {
                    if let Some(seg) = para.line_segs.first() {
                        if seg.line_spacing < 0 {
                            // 표 시작 y와 시각적 높이 저장 (Percent 전환 시 비교용)
                            let ps = styles.para_styles.get(para.para_shape_id as usize);
                            let sa = ps.map(|s| s.spacing_after).unwrap_or(0.0);
                            fix_table_start_y = y_offset - hwpunit_to_px(
                                seg.line_height + seg.line_spacing, self.dpi).max(0.0) - sa;
                            fix_table_visual_h = hwpunit_to_px(seg.line_height, self.dpi);
                            fix_overlay_active = true;
                        }
                    }
                }
            }

            // 표/Shape 처리 후 vpos 기준점 무효화
            // 표/Shape의 LINE_SEG lh는 개체 높이를 포함하여 실제 렌더링 높이와 다르므로
            // vpos 누적이 순차 y_offset과 drift를 일으킴 → 기준점 재산출 필요
            // 예외: Para-relative float 표(vert=Para, TopAndBottom, non-TAC)는
            // 앵커 문단에 attach되므로 후속 문단의 vpos 교정 기준점을 초기화하면 안 됨.
            // 초기화하면 한컴이 Para-float 기준으로 기록한 후속 문단 vpos가 잘못된
            // lazy_base로 교정되어 앵커 y가 상승 → body_bottom clamp → LAYOUT_OVERFLOW.
            let is_table_or_shape = matches!(item,
                PageItem::Table { .. } | PageItem::PartialTable { .. } | PageItem::Shape { .. });
            let is_para_float_table = if let PageItem::Table { para_index, control_index } = item {
                paragraphs
                    .get(*para_index)
                    .and_then(|p| p.controls.get(*control_index))
                    .map(|c| {
                        matches!(
                            c,
                            Control::Table(t)
                            if !t.common.treat_as_char
                                && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                                && matches!(t.common.vert_rel_to, VertRelTo::Para)
                        )
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            if was_tac || (is_table_or_shape && !is_para_float_table) {
                vpos_page_base = None;
                vpos_lazy_base = None;
            }

            // 자가 검증: 배치 후 y_offset이 단 영역 하단을 초과하는지 확인
            let col_bottom = col_area.y + col_area.height;
            let tolerance = 2.0; // 반올림 오차 허용 (2px)
            if y_offset > col_bottom + tolerance {
                let (item_type, para_idx) = match item {
                    PageItem::FullParagraph { para_index } => ("FullParagraph", *para_index),
                    PageItem::PartialParagraph { para_index, .. } => ("PartialParagraph", *para_index),
                    PageItem::Table { para_index, .. } => ("Table", *para_index),
                    PageItem::PartialTable { para_index, .. } => ("PartialTable", *para_index),
                    PageItem::Shape { para_index, .. } => ("Shape", *para_index),
                };
                self.record_overflow(LayoutOverflow {
                    page_index: page_content.page_index,
                    column_index: col_content.column_index as usize,
                    para_index: para_idx,
                    item_type,
                    element_y: y_offset,
                    column_bottom: col_bottom,
                    overflow_px: y_offset - col_bottom,
                });
            }
        }

        // 2차 패스: 글상자(Shape) z-order 정렬 후 렌더링
        self.layout_column_shapes_pass(
            tree, &mut col_node, paper_images,
            col_content, page_content,
            paragraphs, composed, styles,
            bin_data_content, layout, col_area,
            &para_start_y,
        );

        // 문단 테두리/배경 연속 그룹 병합 렌더링
        {
            let ranges = self.para_border_ranges.borrow();
            if !ranges.is_empty() {
                // 연속 ranges 를 시각적 stroke signature 로 병합 (Task #321 v6 근본 수정).
                // bf_id 가 달라도 동일한 stroke (line_type/width/color) 면 HWP/PDF 처럼 하나의
                // 사각형으로 보이도록 병합. invisible (any_w=false) 그룹은 별개로 유지.
                use crate::model::style::BorderLineType;
                type StrokeSig = Option<(BorderLineType, u8, u32)>;
                let stroke_sig = |bf_id: u16| -> StrokeSig {
                    let idx = (bf_id as usize).saturating_sub(1);
                    let bs = styles.border_styles.get(idx)?;
                    let top = &bs.borders[2];
                    let any_w = bs.borders.iter().any(|b|
                        !matches!(b.line_type, BorderLineType::None) && b.width > 0);
                    if any_w {
                        Some((top.line_type, top.width, top.color))
                    } else {
                        None
                    }
                };
                // 그룹 튜플: (bf_id, x, y_start, w, y_end, top_inset, bottom_inset,
                //              is_partial_start, is_partial_end, first_para_idx, last_para_idx)
                let mut groups: Vec<(u16, f64, f64, f64, f64, f64, f64, bool, bool, usize, usize)> = Vec::new();
                for &(bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end, para_idx) in ranges.iter() {
                    if let Some(last) = groups.last_mut() {
                        // bf_id 가 동일하면 기존 동작과 호환 (1차 병합).
                        // 다른 bf_id 지만 동일한 visible stroke 인 경우에만 시각 병합 (None ≠ None 으로 처리).
                        let last_sig = stroke_sig(last.0);
                        let cur_sig = stroke_sig(bf_id);
                        let same_visual = if last.0 == bf_id {
                            true
                        } else {
                            last_sig.is_some() && last_sig == cur_sig
                        };
                        if same_visual && (y_start - last.4) < 30.0 {
                            last.4 = y_end;
                            last.6 = bottom_inset;
                            // 그룹의 partial_end 는 마지막 range 의 값으로 갱신.
                            // partial_start 는 첫 range 값(last.7)을 유지.
                            last.8 = is_partial_end;
                            last.10 = para_idx;  // last_para_idx 갱신
                            // Task #463: 첫 항목이 PartialParagraph (좁은 geometry, 예: pi=50
                            // 우측 단 시작) 이고 후속 항목이 넓은 geometry 일 때, 박스가 좁게
                            // 굳어 후속 paragraph 가 박스 밖으로 튀어나오는 것을 방지하기 위해
                            // merge 그룹의 x/width 를 최대 범위로 확장한다.
                            let last_right = last.1 + last.3;
                            let cur_right = x + w;
                            let new_x = last.1.min(x);
                            let new_right = last_right.max(cur_right);
                            last.1 = new_x;
                            last.3 = new_right - new_x;
                            continue;
                        }
                    }
                    groups.push((bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end, para_idx, para_idx));
                }

                // Task #468: cross-column 박스 연속 검출.
                // sequential 인접 paragraph 가 같은 stroke_sig 면 박스가 다른 컬럼/페이지로 이어진 것.
                // [Task #471] bf_id 비교가 아닌 stroke_sig 비교 — 머지(Task #321 v6)가 visual
                // stroke 기준으로 동작하므로 그룹의 g.0 bf_id 는 첫 range 의 bf_id 만 보존됨.
                // 그룹의 visual sig 와 인접 paragraph 의 visual sig 비교가 정확.
                for g in groups.iter_mut() {
                    let bf_id = g.0;
                    if bf_id == 0 { continue; }
                    let first_pi = g.9;
                    let last_pi = g.10;
                    let group_sig = stroke_sig(bf_id);
                    if group_sig.is_none() { continue; }

                    let para_bf = |pi: usize| -> u16 {
                        composed.get(pi)
                            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
                            .map(|s| s.border_fill_id)
                            .unwrap_or(0)
                    };

                    if !g.7 && first_pi > 0 {
                        let prev_sig = stroke_sig(para_bf(first_pi - 1));
                        if prev_sig.is_some() && prev_sig == group_sig {
                            g.7 = true;
                        }
                    }

                    if !g.8 {
                        let next_sig = stroke_sig(para_bf(last_pi + 1));
                        if next_sig.is_some() && next_sig == group_sig {
                            g.8 = true;
                        }
                    }
                }

                // Task #445: paragraph border 가 col_area 바닥을 넘지 않도록 클램프.
                // vpos-reset 미지원으로 paragraph 가 col_bottom 너머에 layout 될 수 있는데,
                // border 까지 따라가면 페이지/꼬리말 영역까지 침범 (예: exam_kor p8 의 1671px).
                // 텍스트 자체의 overflow 처리는 별도 이슈.
                let col_top = col_area.y;
                let col_bot = col_area.y + col_area.height;
                for g in groups.iter_mut() {
                    if g.2 < col_top { g.2 = col_top; }
                    if g.4 > col_bot { g.4 = col_bot; }
                }
                groups.retain(|g| g.4 > g.2);

                let groups_len = groups.len();
                for (gi, (bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end, _, _)) in groups.clone().into_iter().enumerate() {
                    let height = y_end - y_start;
                    if height <= 0.0 { continue; }
                    // 인접한 다른 border 그룹 (간격 < 4px) 과는 inset 충돌 회피.
                    let prev_touches = gi > 0 && (y_start - groups[gi - 1].4) < 4.0;
                    let next_touches = gi + 1 < groups_len && (groups[gi + 1].2 - y_end) < 4.0;
                    let idx = (bf_id as usize).saturating_sub(1);
                    let border_style = styles.border_styles.get(idx);
                    let fill_color = border_style.and_then(|bs| bs.fill_color);
                    let (stroke_color, stroke_width) = if let Some(bs) = border_style {
                        let any_real_width = bs.borders.iter().any(|b|
                            !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0);
                        if any_real_width {
                            let top = &bs.borders[2];
                            (Some(top.color), super::layout::border_rendering::border_width_to_px(top.width))
                        } else {
                            (None, 0.0)
                        }
                    } else {
                        (None, 0.0)
                    };
                    // Task #321 v6: ParaShape::border_spacing 정식 반영 + stroke 있을 때 default 2px 최소.
                    // 인접 border 그룹과 충돌 방지를 위해 인접 경계는 inset 0.
                    const DEFAULT_MIN_INSET: f64 = 2.0;
                    let top_pad = if stroke_width > 0.0 && !prev_touches { top_inset.max(DEFAULT_MIN_INSET) } else { top_inset };
                    let bot_pad = if stroke_width > 0.0 && !next_touches { bottom_inset.max(DEFAULT_MIN_INSET) } else { bottom_inset };
                    // Task #469: cross-column / cross-page 로 이어진 partial 박스의 후속 부분은
                    // 이전/다음 컬럼에서 이미 inset 이 적용되었으므로 여기서 다시 col_top/col_bot
                    // 너머로 박스를 확장하면 안 된다 (헤더선/꼬리말선과 충돌).
                    // y_start/y_end 는 L1707 에서 col_top..col_bot 으로 이미 클램프됨.
                    let effective_top_pad = if is_partial_start { 0.0 } else { top_pad };
                    let effective_bot_pad = if is_partial_end { 0.0 } else { bot_pad };
                    let rect_y = y_start - effective_top_pad;
                    let rect_h = height + effective_top_pad + effective_bot_pad;
                    // Wrap inner edge 처리: partial_start 면 top, partial_end 면 bottom 미렌더링.
                    let skip_top = stroke_width > 0.0 && is_partial_start;
                    let skip_bottom = stroke_width > 0.0 && is_partial_end;
                    if !skip_top && !skip_bottom {
                        // 기존 경로: 단일 Rectangle (fill + 4면 stroke)
                        let rect_id = tree.next_id();
                        let rect_node = RenderNode::new(
                            rect_id,
                            RenderNodeType::Rectangle(super::render_tree::RectangleNode::new(
                                0.0,
                                super::ShapeStyle {
                                    fill_color,
                                    stroke_color,
                                    stroke_width,
                                    ..Default::default()
                                },
                                None,
                            )),
                            super::render_tree::BoundingBox::new(x, rect_y, w, rect_h),
                        );
                        col_node.children.insert(0, rect_node);
                    } else {
                        // wrap 케이스: fill 만 Rectangle 로, stroke 는 면별 LineNode 로 분해.
                        if fill_color.is_some() {
                            let rect_id = tree.next_id();
                            let rect_node = RenderNode::new(
                                rect_id,
                                RenderNodeType::Rectangle(super::render_tree::RectangleNode::new(
                                    0.0,
                                    super::ShapeStyle {
                                        fill_color,
                                        stroke_color: None,
                                        stroke_width: 0.0,
                                        ..Default::default()
                                    },
                                    None,
                                )),
                                super::render_tree::BoundingBox::new(x, rect_y, w, rect_h),
                            );
                            col_node.children.insert(0, rect_node);
                        }
                        let line_style = super::LineStyle {
                            color: stroke_color.unwrap_or(0),
                            width: stroke_width,
                            ..Default::default()
                        };
                        let mut push_line = |x1: f64, y1: f64, x2: f64, y2: f64| {
                            let lid = tree.next_id();
                            let lnode = RenderNode::new(
                                lid,
                                RenderNodeType::Line(super::render_tree::LineNode::new(
                                    x1, y1, x2, y2, line_style.clone(),
                                )),
                                super::render_tree::BoundingBox::new(
                                    x1.min(x2), y1.min(y2),
                                    (x2 - x1).abs().max(stroke_width),
                                    (y2 - y1).abs().max(stroke_width),
                                ),
                            );
                            col_node.children.insert(0, lnode);
                        };
                        let x_left = x;
                        let x_right = x + w;
                        let y_top = rect_y;
                        let y_bot = rect_y + rect_h;
                        // 좌·우 수직선은 항상 렌더
                        push_line(x_left, y_top, x_left, y_bot);
                        push_line(x_right, y_top, x_right, y_bot);
                        if !skip_top {
                            push_line(x_left, y_top, x_right, y_top);
                        }
                        if !skip_bottom {
                            push_line(x_left, y_bot, x_right, y_bot);
                        }
                    }
                }
            }
        }

        (col_node, y_offset)
    }

    /// 단 내 개별 PageItem을 레이아웃한다 (1차 패스).
    /// 반환값: (새 y_offset, TAC 표 line_seg 줄간격 적용 여부)
    #[allow(clippy::too_many_arguments)]
    fn layout_column_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        item: &PageItem,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        measured_tables: &[MeasuredTable],
        layout: &PageLayoutInfo,
        col_area: &LayoutRect,
        outline_numbering_id: u16,
        multi_col_width: Option<i32>,
        mut y_offset: f64,
        prev_tac_seg_applied: bool,
        wrap_around_paras: &[super::pagination::WrapAroundPara],
    ) -> (f64, bool) {
        let ctx = ColumnItemCtx {
            page_content, paragraphs, composed, styles, bin_data_content,
            measured_tables, layout, col_area, outline_numbering_id,
            multi_col_width, prev_tac_seg_applied, wrap_around_paras,
        };
        match item {
            PageItem::FullParagraph { para_index } => {
                // 빈 줄 감추기: 높이 0 처리된 문단은 문단부호만 렌더링하고 y_offset 변경 없음
                if self.hidden_empty_paras.borrow().contains(para_index) {
                    // 문단부호는 렌더링 (클리핑 바깥에 표시)
                    if let Some(comp) = composed.get(*para_index) {
                        if let Some(para) = paragraphs.get(*para_index) {
                            para_start_y.insert(*para_index, y_offset);
                            self.layout_paragraph(
                                tree, col_node, para, Some(comp), styles,
                                col_area, y_offset, page_content.section_index,
                                *para_index, multi_col_width, Some(bin_data_content),
                            );
                        }
                    }
                    return (y_offset, false);
                }
                if let Some(para) = paragraphs.get(*para_index) {
                    let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                    let has_block_table = para.controls.iter()
                        .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char
                            || (t.common.treat_as_char
                                && !crate::renderer::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls))));
                    if has_block_table {
                        let comp = composed.get(*para_index);
                        let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
                        if let Some(para_style) = styles.para_styles.get(para_style_id) {
                            // 번호 카운터 전진 (후속 문단의 번호 연속성 유지)
                            // Bullet은 카운터를 사용하지 않으므로 제외
                            if para_style.head_type == HeadType::Outline || para_style.head_type == HeadType::Number {
                                let nid = resolve_numbering_id(para_style.head_type, para_style.numbering_id, outline_numbering_id);
                                if nid > 0 {
                                    self.numbering_state.borrow_mut().advance(nid, para_style.para_level, para.numbering_restart);
                                }
                            }
                            if para_style.spacing_before > 0.0 {
                                y_offset += para_style.spacing_before;
                            }
                        }
                        // 어울림 표 호스트 문단의 텍스트는 layout_wrap_around_paras에서 처리
                        let is_wrap_host = para.controls.iter().any(|c| {
                            if let Control::Table(t) = c {
                                !t.common.treat_as_char && matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
                            } else { false }
                        });
                        // 블록 표/도형 외에 실제 텍스트가 있는지 확인
                        // (예: [선][선][표][표]참고문헌 → 표 아래에 텍스트 렌더링 필요)
                        let has_real_text = !is_wrap_host && para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}' && !c.is_whitespace());
                        if has_real_text {
                            if let Some(comp) = comp {
                                // 컨트롤 전용 줄(runs가 모두 제어문자)을 건너뛰고 텍스트 줄부터 렌더링
                                let text_start_line = comp.lines.iter().position(|line| {
                                    line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                                });
                                if let Some(start_line) = text_start_line {
                                    para_start_y.insert(*para_index, y_offset);
                                    y_offset = self.layout_partial_paragraph(
                                        tree,
                                        col_node,
                                        para,
                                        Some(comp),
                                        styles,
                                        col_area,
                                        y_offset,
                                        start_line,
                                        comp.lines.len(),
                                        page_content.section_index,
                                        *para_index,
                                        multi_col_width,
                                        Some(bin_data_content),
                                    );
                                }
                            }
                        }
                        return (y_offset, false);
                    }

                    let has_inline_tables = para.controls.iter()
                        .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char
                            && crate::renderer::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls)));

                    if has_inline_tables {
                        // 인라인 표 문단도 번호 카운터 전진 필요
                        self.apply_paragraph_numbering(
                            composed.get(*para_index), para, styles, outline_numbering_id,
                        );
                        para_start_y.insert(*para_index, y_offset);
                        y_offset = self.layout_inline_table_paragraph(
                            tree,
                            col_node,
                            para,
                            composed.get(*para_index),
                            styles,
                            col_area,
                            y_offset,
                            page_content.section_index,
                            *para_index,
                            bin_data_content,
                            measured_tables,
                        );
                    } else {
                        let comp = composed.get(*para_index);
                        let numbered_comp = self.apply_paragraph_numbering(
                            comp, para, styles, outline_numbering_id,
                        );
                        let final_comp = numbered_comp.as_ref().or(comp);

                        para_start_y.insert(*para_index, y_offset);
                        y_offset = self.layout_paragraph(
                            tree,
                            col_node,
                            para,
                            final_comp,
                            styles,
                            col_area,
                            y_offset,
                            page_content.section_index,
                            *para_index,
                            multi_col_width,
                            Some(bin_data_content),
                        );
                    }
                    // TAC Shape 높이 보정: 문단에 TAC Shape(개체묶기 등)가 있으면
                    // Shape 높이가 문단 텍스트 높이보다 클 수 있으므로 y_offset을 보정.
                    // LINE_SEG lh가 Shape+캡션+간격을 모두 포함하므로 max(Shape.height, lh)를 사용.
                    // 보정 시 원래 문단 간격(spacing_after)도 유지한다.
                    {
                        let has_tac_shape = para.controls.iter()
                            .any(|c| matches!(c, Control::Shape(s) if s.common().treat_as_char));
                        if has_tac_shape {
                            // LINE_SEG lh = 이미지+캡션+간격 전체 높이
                            let seg_lh: f64 = para.line_segs.iter()
                                .map(|seg| hwpunit_to_px(seg.line_height, self.dpi))
                                .fold(0.0f64, f64::max);
                            let shape_max_h: f64 = para.controls.iter()
                                .filter_map(|c| match c {
                                    Control::Shape(s) if s.common().treat_as_char => {
                                        Some(hwpunit_to_px(s.common().height as i32, self.dpi))
                                    }
                                    _ => None,
                                })
                                .fold(0.0f64, f64::max);
                            let effective_h = seg_lh.max(shape_max_h);
                            if effective_h > 0.0 {
                                let para_start = *para_start_y.get(para_index).unwrap_or(&y_offset);
                                let shape_bottom = para_start + effective_h;
                                if shape_bottom > y_offset {
                                    let spacing = styles.para_styles
                                        .get(para.para_shape_id as usize)
                                        .map(|s| s.spacing_after)
                                        .unwrap_or(0.0);
                                    y_offset = shape_bottom + spacing;
                                }
                            }
                        }
                    }
                    // 각주 위첨자: footnote_positions가 있으면 인라인으로 이미 처리됨
                    let has_inline_fn = composed.get(*para_index)
                        .map(|c| !c.footnote_positions.is_empty()).unwrap_or(false);
                    if !has_inline_fn {
                        self.add_footnote_superscripts(
                            tree, col_node, para, styles,
                        );
                    }
                }
            }
            PageItem::PartialParagraph { para_index, start_line, end_line } => {
                if let Some(para) = paragraphs.get(*para_index) {
                    // Task #318: wrap=Square 표 호스트 문단의 텍스트는
                    // layout_wrap_around_paras (자가 wrap 경로) 가 처리한다. PartialParagraph
                    // 측에서 같은 paragraph 를 layout_partial_paragraph 로 다시 호출하면
                    // 호스트 텍스트 + 인라인 수식이 중복 emit 됨 (#301 회귀).
                    // FullParagraph 경로 (`is_wrap_host` 가드, layout.rs:1639) 와 동일한 처리.
                    let is_wrap_host = para.controls.iter().any(|c| {
                        if let Control::Table(t) = c {
                            !t.common.treat_as_char
                                && matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
                        } else { false }
                    });
                    if is_wrap_host {
                        return (y_offset, false);
                    }

                    // TAC 블록 표 문단의 post-text PP: 텍스트가 공백만이면 건너뜀
                    // (Table PageItem에서 이미 y_offset이 결정됨)
                    if prev_tac_seg_applied {
                        let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                        let has_tac_block = para.controls.iter().any(|c|
                            matches!(c, Control::Table(t) if t.common.treat_as_char
                                && !crate::renderer::height_measurer::is_tac_table_inline(
                                    t, seg_width, &para.text, &para.controls)));
                        if has_tac_block {
                            let pp_text_only_ws = if let Some(comp) = composed.get(*para_index) {
                                comp.lines[*start_line..*end_line].iter().all(|line| {
                                    line.runs.iter().all(|r| r.text.chars().all(|c| c.is_whitespace() || c <= '\u{001F}' || c == '\u{FFFC}'))
                                })
                            } else { false };
                            if pp_text_only_ws {
                                // Table PageItem에서 이미 표 높이가 반영됨
                                // 공백만인 PartialParagraph는 높이 추가 없이 건너뜀
                                return (y_offset, true);
                            }
                        }
                    }
                    // 첫 부분에서만 번호 카운터 전진 + 번호 텍스트 적용
                    let comp = if *start_line == 0 {
                        let numbered = self.apply_paragraph_numbering(
                            composed.get(*para_index), para, styles, outline_numbering_id,
                        );
                        // numbered가 있으면 composed 업데이트는 불가하므로
                        // layout_partial_paragraph에 직접 전달
                        numbered.or_else(|| composed.get(*para_index).cloned())
                    } else {
                        composed.get(*para_index).cloned()
                    };
                    y_offset = self.layout_partial_paragraph(
                        tree,
                        col_node,
                        para,
                        comp.as_ref(),
                        styles,
                        col_area,
                        y_offset,
                        *start_line,
                        *end_line,
                        page_content.section_index,
                        *para_index,
                        None,
                        Some(bin_data_content),
                    );
                }
            }
            PageItem::Table { para_index, control_index } => {
                return self.layout_table_item(
                    tree, col_node, paper_images, para_start_y,
                    *para_index, *control_index, &ctx, y_offset,
                );
            }
            PageItem::PartialTable { para_index, control_index, start_row, end_row, is_continuation,
                split_start_content_offset, split_end_content_limit } => {
                y_offset = self.layout_partial_table_item(
                    tree, col_node, para_start_y,
                    *para_index, *control_index, *start_row, *end_row,
                    *is_continuation, *split_start_content_offset, *split_end_content_limit,
                    &ctx, y_offset,
                );
            }
            PageItem::Shape { para_index, control_index } => {
                y_offset = self.layout_shape_item(
                    tree, col_node, paper_images, para_start_y,
                    *para_index, *control_index, &ctx, y_offset,
                );
            }
        }
        (y_offset, false)
    }

    /// Table PageItem 레이아웃 (layout_column_item에서 분리)
    #[allow(clippy::too_many_arguments)]
    fn layout_table_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        para_index: usize,
        control_index: usize,
        ctx: &ColumnItemCtx,
        mut y_offset: f64,
    ) -> (f64, bool) {
        let ColumnItemCtx {
            page_content, paragraphs, composed, styles, bin_data_content,
            measured_tables, layout, col_area, multi_col_width,
            prev_tac_seg_applied, wrap_around_paras, ..
        } = ctx;
        // 표 앵커 문단의 y 위치 등록
        // TAC 표: 이전 TAC가 y_offset을 진행시킨 경우 갱신 (같은 문단 TAC+블록 구조)
        // 비-TAC 표: 문단 시작 y를 유지 (각 표가 독립적으로 vert offset 기준 배치)
        let is_current_tac = paragraphs.get(para_index)
            .and_then(|p| p.controls.get(control_index))
            .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
            .unwrap_or(false);
        if let Some(existing_y) = para_start_y.get(&para_index) {
            if is_current_tac && y_offset > *existing_y + 1.0 {
                para_start_y.insert(para_index, y_offset);
            }
        } else {
            para_start_y.insert(para_index, y_offset);
        }
        let para_y_for_table = *para_start_y.get(&para_index).unwrap_or(&y_offset);
        if let Some(para) = paragraphs.get(para_index) {
            let is_tac = para.controls.get(control_index)
                .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                .unwrap_or(false);
            // ── 표 위 간격 ──
            {
                let comp = composed.get(para_index);
                let ps_id = comp.map(|c| c.para_style_id as usize)
                    .unwrap_or(para.para_shape_id as usize);
                let is_column_top = (y_offset - col_area.y).abs() < 1.0;
                if is_tac {
                    if !prev_tac_seg_applied {
                        let outer_margin_top_px = if let Some(Control::Table(t)) = para.controls.get(control_index) {
                            hwpunit_to_px(t.outer_margin_top as i32, self.dpi)
                        } else {
                            0.0
                        };
                        if !is_column_top {
                            let spacing_before = styles.para_styles.get(ps_id)
                                .map(|ps| ps.spacing_before).unwrap_or(0.0);
                            if spacing_before > 0.0 {
                                y_offset += spacing_before;
                            }
                        }
                        if outer_margin_top_px > 0.0 {
                            y_offset += outer_margin_top_px;
                        }
                    }
                } else {
                    if let Some(ps) = styles.para_styles.get(ps_id) {
                        if ps.spacing_before > 0.0 && !is_column_top {
                            y_offset += ps.spacing_before;
                        }
                    }
                }
            }
            // ── 호스트 문단 텍스트 렌더링 ──
            let text_already_laid_out = page_content.column_contents.iter().any(|cc| {
                cc.items.iter().any(|it| {
                    matches!(it, PageItem::PartialParagraph { para_index: pi, .. } if *pi == para_index)
                })
            });
            if !is_tac && !text_already_laid_out {
                let host_is_not_square = if let Some(Control::Table(ht)) = para.controls.get(control_index) {
                    !matches!(ht.common.text_wrap, crate::model::shape::TextWrap::Square)
                } else { true };
                if host_is_not_square {
                    let has_real_text = para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                    if has_real_text {
                        if let Some(comp) = composed.get(para_index) {
                            let text_start_line = comp.lines.iter().position(|line| {
                                line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                            });
                            if let Some(start_line) = text_start_line {
                                let text_end_line = comp.lines.iter().rposition(|line| {
                                    line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                                }).map(|i| i + 1).unwrap_or(comp.lines.len());
                                para_start_y.insert(para_index, y_offset);
                                let _text_y_end = self.layout_partial_paragraph(
                                    tree, col_node, para, Some(comp), styles,
                                    col_area, y_offset, start_line, text_end_line,
                                    page_content.section_index, para_index,
                                    *multi_col_width, Some(bin_data_content),
                                );
                            }
                        }
                    }
                }
            }
            // ── 표 레이아웃 ──
            let mut tac_seg_applied = false;
            let tac_table_y_before = y_offset;  // Task #9: 표 렌더 전 y 보존
            if let Some(Control::Table(t)) = para.controls.get(control_index) {
                let mt = measured_tables.iter().find(|mt|
                    mt.para_index == para_index && mt.control_index == control_index
                );
                let para_style = styles.para_styles
                    .get(para.para_shape_id as usize);
                let alignment = para_style
                    .map(|s| s.alignment)
                    .unwrap_or(Alignment::Left);
                let margin_left = para_style
                    .map(|s| s.margin_left)
                    .unwrap_or(0.0);
                let indent = para_style
                    .map(|s| s.indent)
                    .unwrap_or(0.0);
                let effective_margin = if indent > 0.0 {
                    margin_left + indent
                } else {
                    margin_left
                };
                let margin_right = para_style
                    .map(|s| s.margin_right)
                    .unwrap_or(0.0);
                let table_y_before = y_offset;
                let tbl_is_square = matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square);
                // インラインTAC表: paragraph_layoutで計算された位置を使用
                let inline_pos = if is_tac {
                    tree.get_inline_shape_position(page_content.section_index, para_index, control_index)
                } else {
                    None
                };
                let tbl_inline_x = if let Some((ix, _)) = inline_pos {
                    Some(ix)
                } else if !is_tac && tbl_is_square {
                    // [Issue #480] Square wrap 표는 paragraph 영역 (col_area + margin) 기준으로 정렬.
                    // 이전 동작(col_area 기준)은 paragraph margin/indent 가 있는 경우 표가
                    // 단 사이 갭으로 떨어지는 문제 발생 (예: 페이지 14 [A] 박스).
                    // (Task #295: halign=Right 표가 좌측에 잘못 배치되는 문제 수정)
                    let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
                    let area_x = col_area.x + effective_margin;
                    let area_w = (col_area.width - effective_margin - margin_right).max(0.0);
                    let x = match t.common.horz_align {
                        crate::model::shape::HorzAlign::Right | crate::model::shape::HorzAlign::Outside =>
                            area_x + (area_w - tbl_w).max(0.0),
                        crate::model::shape::HorzAlign::Center =>
                            area_x + (area_w - tbl_w).max(0.0) / 2.0,
                        _ => area_x,
                    };
                    Some(x)
                } else if is_tac {
                    // TAC 문단에 PageItem::FullParagraph 가 발행되지 않아
                    // paragraph_layout 가 호출되지 않는 케이스(선행 공백만 있는 TAC 표 등):
                    // composed.lines[0] 의 runs 에서 TAC 이전 텍스트 폭을 직접
                    // 합산해 표 x 좌표에 반영한다. inline_shape_position 미세팅 상태에서
                    // 기본값 col_area.x(body_left) 으로 붕괴되는 현상 방지.
                    let leading = composed.get(para_index)
                        .map(|c| compute_tac_leading_width(c, control_index, styles))
                        .unwrap_or(0.0);
                    let base_x = col_area.x + effective_margin + leading;
                    // [Issue #291] ParaShape align 반영:
                    // TAC 표가 inline_shape_position 미설정 상태에서 단/문단 좌측에
                    // 붙어버리는 회귀를 막는다. ParaShape align=Right 인 경우 표를
                    // 단의 우측 끝 - 표 폭 - margin_right 위치로 이동시켜 한컴과 일치.
                    // align=Center 도 동일 원리로 처리.
                    let aligned_x = match para_style.map(|s| s.alignment) {
                        Some(crate::model::style::Alignment::Right) => {
                            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
                            let avail_right = col_area.x + col_area.width - margin_right;
                            (avail_right - tbl_w).max(base_x)
                        }
                        Some(crate::model::style::Alignment::Center) => {
                            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
                            let center = col_area.x + (col_area.width - tbl_w) / 2.0;
                            center.max(base_x)
                        }
                        _ => base_x,
                    };
                    Some(aligned_x)
                } else {
                    None
                };
                // vert=Paper로 body_area 위에 배치되는 표
                // 본문 영역 외부(머리말/꼬리말 자리)에 그려지는 페이지/페이퍼 앵커 TopAndBottom 표는
                // 본문 흐름의 y_offset을 진행시키지 않고 out-of-flow로 paper_images에 렌더한다.
                // (Task #295: vert=Page valign=Bottom 푸터 표가 좌단 y_offset을 본문 하단으로
                //  끌어올려 후속 콘텐츠를 깨뜨리는 문제 수정 — Paper만 다루던 기존 분기를 Page까지 확장)
                let renders_outside_body = !is_tac
                    && matches!(t.common.vert_rel_to,
                        crate::model::shape::VertRelTo::Paper | crate::model::shape::VertRelTo::Page)
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                    && {
                        let tbl_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                        let v_off = hwpunit_to_px(t.common.vertical_offset as i32, self.dpi);
                        let tbl_y = match t.common.vert_align {
                            crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside => v_off,
                            crate::model::shape::VertAlign::Center => (layout.page_height - tbl_h) / 2.0 + v_off,
                            crate::model::shape::VertAlign::Bottom | crate::model::shape::VertAlign::Outside => layout.page_height - tbl_h - v_off,
                        };
                        // 표 상단이 본문 위(머리말)이거나, 표 하단이 본문 아래(꼬리말)에 걸치는 경우
                        let body_bottom = layout.body_area.y + layout.body_area.height;
                        tbl_y < layout.body_area.y || tbl_y + tbl_h > body_bottom
                    };
                if renders_outside_body {
                    let tmp_id = tree.next_id();
                    let mut tmp_node = RenderNode::new(
                        tmp_id,
                        RenderNodeType::Column(0),
                        layout_rect_to_bbox(&layout.body_area),
                    );
                    let _table_y_end = self.layout_table(
                        tree, &mut tmp_node, t,
                        page_content.section_index, styles, &layout.body_area,
                        y_offset, bin_data_content, mt, 0,
                        Some((para_index, control_index)),
                        alignment, None, effective_margin, margin_right,
                        tbl_inline_x, None, Some(para_y_for_table),
                    );
                    for child in tmp_node.children.drain(..) {
                        paper_images.push(child);
                    }
                } else {
                    let table_y_start = if let Some((_, iy)) = inline_pos { iy } else { y_offset };
                    y_offset = self.layout_table(
                        tree, col_node, t,
                        page_content.section_index, styles, col_area,
                        table_y_start, bin_data_content, mt, 0,
                        Some((para_index, control_index)),
                        alignment, None, effective_margin, margin_right,
                        tbl_inline_x, None, Some(para_y_for_table),
                    );
                }
                let table_y_end = y_offset; // layout_table 반환값 보존
                // ── TAC 표: 줄간격 처리 ──
                // layout_table 반환값(표 하단)에 line_spacing을 더하여 다음 표 시작 y 결정
                if is_tac {
                    let seg_idx = control_index;
                    let tac_count_total = para.controls.iter()
                        .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                        .count();
                    let tac_idx_current = para.controls.iter().take(control_index + 1)
                        .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                        .count();
                    // TAC 표 사이에 non-TAC 표가 있는지 확인
                    let has_non_tac_between = para.controls.iter()
                        .skip(control_index + 1)
                        .take_while(|c| !matches!(c, Control::Table(t) if t.common.treat_as_char))
                        .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char));
                    if tac_idx_current < tac_count_total && !has_non_tac_between {
                        // 다음 TAC가 있으면: vpos 차이분만 추가 (= line_spacing)
                        // 이후 tac_seg_applied 경로의 line_spacing 추가를 스킵하기 위해
                        // 여기서 직접 return (spacing_after/line_spacing 이중 적용 방지)
                        if let (Some(seg), Some(next_seg)) = (para.line_segs.get(seg_idx), para.line_segs.get(seg_idx + 1)) {
                            let gap = next_seg.vertical_pos - (seg.vertical_pos + seg.line_height);
                            y_offset += hwpunit_to_px(gap, self.dpi);
                        }
                        return (y_offset, true);
                    } else {
                        // 마지막 TAC: line_end 보정 (vpos 기반)
                        // 표 실제 하단을 상한으로 clamp (ls는 이후 TAC seg handling에서 추가)
                        if let Some(seg) = para.line_segs.get(seg_idx) {
                            let line_end = para_y_for_table
                                + hwpunit_to_px(seg.vertical_pos + seg.line_height, self.dpi);
                            let clamped = line_end.min(table_y_end);
                            let max_correction = hwpunit_to_px(seg.line_spacing * 2 + 1000, self.dpi);
                            if clamped > y_offset && (clamped - y_offset) <= max_correction {
                                y_offset = clamped;
                            }
                        }
                    }
                    tac_seg_applied = true;
                }
                // ── 어울림 문단 렌더링 ──
                // 후속 wrap 문단이 없어도 호스트 본문이 표 옆에 wrap되어야 하므로
                // wrap_around_paras 비어 있어도 호출 (Task #295: pi=27 자가 wrap 누락 수정)
                let table_is_square = matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square);
                if !is_tac && table_is_square {
                    let wrap_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                    let wrap_sw = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                    let wrap_text_x = col_area.x + hwpunit_to_px(wrap_cs, self.dpi);
                    let wrap_text_width = hwpunit_to_px(wrap_sw, self.dpi);
                    // Task #463: 인라인 floating 표 우측 x 계산 (paragraph border box 확장용).
                    // table_layout::compute_table_x_position 와 동일 공식.
                    let tbl_x_right = compute_square_wrap_tbl_x_right(t, col_area, self.dpi);
                    self.layout_wrap_around_paras(
                        tree, col_node, paragraphs, composed, styles, col_area,
                        page_content.section_index,
                        para_index, wrap_around_paras,
                        table_y_before, y_offset,
                        wrap_text_x, wrap_text_width, 0.0,
                        bin_data_content,
                        Some(tbl_x_right),
                    );
                }
            }
            // ── 표 아래 간격 ──
            // out-of-flow로 그려진 표(머리말/꼬리말 자리)는 본문 흐름 간격을 추가하지 않는다.
            let is_outside_body = if let Some(Control::Table(t)) = para.controls.get(control_index) {
                !t.common.treat_as_char
                    && matches!(t.common.vert_rel_to,
                        crate::model::shape::VertRelTo::Paper | crate::model::shape::VertRelTo::Page)
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                    && {
                        let tbl_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                        let v_off = hwpunit_to_px(t.common.vertical_offset as i32, self.dpi);
                        let tbl_y = match t.common.vert_align {
                            crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside => v_off,
                            crate::model::shape::VertAlign::Center => (layout.page_height - tbl_h) / 2.0 + v_off,
                            crate::model::shape::VertAlign::Bottom | crate::model::shape::VertAlign::Outside => layout.page_height - tbl_h - v_off,
                        };
                        let body_bottom = layout.body_area.y + layout.body_area.height;
                        tbl_y < layout.body_area.y || tbl_y + tbl_h > body_bottom
                    }
            } else { false };
            if !tac_seg_applied && !is_outside_body {
                let comp = composed.get(para_index);
                let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
                if let Some(para_style) = styles.para_styles.get(para_style_id) {
                    if para_style.spacing_after > 0.0 {
                        y_offset += para_style.spacing_after;
                    }
                }
                if let Some(seg) = para.line_segs.last() {
                    let gap = if seg.line_spacing > 0 { seg.line_spacing } else { seg.line_height };
                    y_offset += hwpunit_to_px(gap, self.dpi);
                }
            }
            if tac_seg_applied {
                if let Some(seg) = para.line_segs.get(control_index) {
                    if seg.line_spacing > 0 {
                        y_offset += hwpunit_to_px(seg.line_spacing, self.dpi);
                    } else if seg.line_spacing < 0 {
                        // 음수 ls (Fixed 줄간격 TAC 표): y를 문단 advance로 리셋 (Task #9)
                        // 표 렌더 높이가 아닌, 일반 문단과 동일한 lh+ls advance 사용
                        let advance = hwpunit_to_px(seg.line_height + seg.line_spacing, self.dpi).max(0.0);
                        y_offset = tac_table_y_before + advance;
                    }
                }
                let comp = composed.get(para_index);
                let ps_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
                if let Some(ps) = styles.para_styles.get(ps_id) {
                    if ps.spacing_after > 0.0 {
                        y_offset += ps.spacing_after;
                    }
                }
                return (y_offset, true);
            }
            // ── 같은 문단의 인라인 TAC 표 렌더링 ──
            if !is_tac {
                let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if ci == control_index { continue; }
                    if let Control::Table(inline_t) = ctrl {
                        if inline_t.common.treat_as_char
                            && crate::renderer::height_measurer::is_tac_table_inline(inline_t, seg_width, &para.text, &para.controls)
                        {
                            let mt = measured_tables.iter().find(|m| m.para_index == para_index && m.control_index == ci);
                            let alignment = composed.get(para_index)
                                .map(|c| styles.para_styles.get(c.para_style_id as usize)
                                    .map(|s| s.alignment).unwrap_or(Alignment::Left))
                                .unwrap_or(Alignment::Left);
                            // paragraph_layout에서 계산된 인라인 좌표 사용
                            let inline_pos = tree.get_inline_shape_position(
                                page_content.section_index, para_index, ci);
                            let (inline_x, inline_y) = if let Some((ix, iy)) = inline_pos {
                                (Some(ix), iy)
                            } else {
                                (None, para_y_for_table)
                            };
                            let tac_new_y = self.layout_table(
                                tree, col_node, inline_t,
                                page_content.section_index, styles, col_area, inline_y,
                                bin_data_content, mt, 0,
                                Some((para_index, ci)),
                                alignment, None, 0.0, 0.0, inline_x, None, None,
                            );
                            y_offset = y_offset.max(tac_new_y);
                        }
                    }
                }
            }
        }
        (y_offset, false)
    }

    /// 어울림 배치 표 옆에 빈 리턴 문단을 렌더링
    /// 표는 왼쪽, 문단(하드 리턴)은 오른쪽에 배치
    /// `table_content_offset`: 현재 페이지에서 표시되는 표 콘텐츠의
    /// 어울림 배치 표 옆 문단 렌더링 (텍스트 문단 + 빈 리턴 ↵ 마크)
    ///
    /// table_content_offset: 분할 표에서 이전 페이지에 표시된 행 높이 합 (px)
    #[allow(clippy::too_many_arguments)]
    /// PartialTable PageItem 레이아웃 (layout_column_item에서 분리)
    #[allow(clippy::too_many_arguments)]
    fn layout_partial_table_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        para_index: usize,
        control_index: usize,
        start_row: usize,
        end_row: usize,
        is_continuation: bool,
        split_start_content_offset: f64,
        split_end_content_limit: f64,
        ctx: &ColumnItemCtx,
        mut y_offset: f64,
    ) -> f64 {
        let ColumnItemCtx {
            page_content, paragraphs, composed, styles, bin_data_content,
            measured_tables, col_area, multi_col_width, wrap_around_paras, ..
        } = ctx;
        // ── 분할 표 첫 부분: 호스트 문단 텍스트 렌더링 ──
        if !is_continuation {
            if let Some(para) = paragraphs.get(para_index) {
                let is_tac = para.controls.get(control_index)
                    .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    .unwrap_or(false);
                if !is_tac {
                    let has_real_text = para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                    if has_real_text {
                        if let Some(comp) = composed.get(para_index) {
                            let text_start_line = comp.lines.iter().position(|line| {
                                line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                            });
                            if let Some(start_line) = text_start_line {
                                let text_end_line = comp.lines.iter().rposition(|line| {
                                    line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                                }).map(|i| i + 1).unwrap_or(comp.lines.len());
                                para_start_y.insert(para_index, y_offset);
                                let _text_y_end = self.layout_partial_paragraph(
                                    tree, col_node, para, Some(comp), styles,
                                    col_area, y_offset, start_line, text_end_line,
                                    page_content.section_index, para_index,
                                    *multi_col_width, Some(bin_data_content),
                                );
                            }
                        }
                    }
                }
            }
        }
        let (pt_margin_left, pt_margin_right) = if let Some(para) = paragraphs.get(para_index) {
            let ps = styles.para_styles.get(para.para_shape_id as usize);
            let ml = ps.map(|s| s.margin_left).unwrap_or(0.0);
            let ind = ps.map(|s| s.indent).unwrap_or(0.0);
            let mr = ps.map(|s| s.margin_right).unwrap_or(0.0);
            (if ind > 0.0 { ml + ind } else { ml }, mr)
        } else {
            (0.0, 0.0)
        };
        let pt_mt = measured_tables.iter().find(|mt|
            mt.para_index == para_index && mt.control_index == control_index
        );
        // 비-TAC 자리차지 표에서 vert offset이 있으면 문단 시작 y 전달
        // layout_partial_table 내부에서 vert_offset을 적용하므로 이중 적용 방지
        let pt_y_start = if let Some(para) = paragraphs.get(para_index) {
            if let Some(Control::Table(t)) = para.controls.get(control_index) {
                if !t.common.treat_as_char
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                    && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                    && t.common.vertical_offset > 0
                {
                    para_start_y.get(&para_index).copied().unwrap_or(y_offset)
                } else {
                    y_offset
                }
            } else { y_offset }
        } else { y_offset };
        let pt_y_before = y_offset;
        y_offset = self.layout_partial_table(
            tree, col_node, paragraphs,
            para_index, control_index,
            page_content.section_index, styles, col_area,
            pt_y_start, bin_data_content,
            start_row, end_row, is_continuation,
            split_start_content_offset, split_end_content_limit,
            pt_margin_left, pt_margin_right, pt_mt,
        );
        if let Some(para) = paragraphs.get(para_index) {
            let comp = composed.get(para_index);
            let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
            if let Some(para_style) = styles.para_styles.get(para_style_id) {
                let is_tac = para.controls.get(control_index)
                    .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    .unwrap_or(false);
                if is_tac {
                    if para_style.spacing_after > 0.0 {
                        y_offset += para_style.spacing_after;
                    }
                    let outer_margin_bottom_px = if let Some(Control::Table(t)) = para.controls.get(control_index) {
                        hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
                    } else { 0.0 };
                    if outer_margin_bottom_px > 0.0 {
                        y_offset += outer_margin_bottom_px;
                    }
                } else {
                    if para_style.spacing_after > 0.0 {
                        y_offset += para_style.spacing_after;
                    }
                }
            }
        }
        // ── 분할 표: 어울림 문단 렌더링 ──
        if let Some(para) = paragraphs.get(para_index) {
            if let Some(Control::Table(t)) = para.controls.get(control_index) {
                let pt_is_tac = t.common.treat_as_char;
                let pt_is_square = matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square);
                if !pt_is_tac && pt_is_square && !wrap_around_paras.is_empty() {
                    let wrap_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                    let wrap_sw = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                    let wrap_text_x = col_area.x + hwpunit_to_px(wrap_cs, self.dpi);
                    let wrap_text_width = hwpunit_to_px(wrap_sw, self.dpi);
                    let content_offset = if let Some(mt) = pt_mt {
                        mt.range_height(0, start_row)
                    } else { 0.0 };
                    let tbl_x_right = compute_square_wrap_tbl_x_right(t, col_area, self.dpi);
                    self.layout_wrap_around_paras(
                        tree, col_node, paragraphs, composed, styles, col_area,
                        page_content.section_index, para_index, wrap_around_paras,
                        pt_y_before, y_offset,
                        wrap_text_x, wrap_text_width, content_offset,
                        bin_data_content,
                        Some(tbl_x_right),
                    );
                }
            }
        }
        y_offset
    }

    /// Shape PageItem 레이아웃 (layout_column_item에서 분리)
    #[allow(clippy::too_many_arguments)]
    fn layout_shape_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        para_index: usize,
        control_index: usize,
        ctx: &ColumnItemCtx,
        y_offset: f64,
    ) -> f64 {
        let ColumnItemCtx {
            page_content, paragraphs, composed, styles, bin_data_content,
            layout, col_area, ..
        } = ctx;
        // Task #402: 같은 paragraph 안에 TAC 컨트롤(표/그림/도형) 2개 이상이 서로 다른 line에
        // 배치된 경우, 두 번째 이후의 그림은 paragraph 시작 y가 아니라 진행된 y_offset
        // (선행 TAC 후속 위치)에 그려져야 표와 겹치지 않는다. control_index 이전에 같은
        // paragraph의 TAC 컨트롤이 있고 y_offset이 기존 등록값보다 진행됐으면 갱신한다.
        let has_prior_tac_in_para = paragraphs.get(para_index)
            .map(|p| p.controls.iter().take(control_index).any(|c| match c {
                Control::Table(t) => t.common.treat_as_char,
                Control::Picture(p) => p.common.treat_as_char,
                Control::Shape(s) => s.common().treat_as_char,
                _ => false,
            }))
            .unwrap_or(false);
        if has_prior_tac_in_para {
            let needs_update = para_start_y.get(&para_index)
                .map(|&existing| y_offset > existing + 1.0)
                .unwrap_or(true);
            if needs_update {
                para_start_y.insert(para_index, y_offset);
            }
        } else {
            para_start_y.entry(para_index).or_insert(y_offset);
        }
        let mut result_y = y_offset;
        if let Some(para) = paragraphs.get(para_index) {
            if let Some(ctrl) = para.controls.get(control_index) {
                if let Control::Picture(pic) = ctrl {
                    if pic.common.treat_as_char {
                        let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                        let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                        let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
                        let comp = composed.get(para_index);
                        let para_style_id = comp.map(|c| c.para_style_id as usize)
                            .unwrap_or(para.para_shape_id as usize);
                        let para_style_ref = styles.para_styles.get(para_style_id);
                        let para_alignment = para_style_ref
                            .map(|s| s.alignment)
                            .unwrap_or(Alignment::Left);
                        // Task #347: 첫 줄 effective_margin (hanging indent: indent<0 → first-line은 margin_left만 적용)
                        let para_margin_left = para_style_ref.map(|s| s.margin_left).unwrap_or(0.0);
                        let para_indent = para_style_ref.map(|s| s.indent).unwrap_or(0.0);
                        let effective_margin_left = if para_indent > 0.0 {
                            para_margin_left + para_indent
                        } else {
                            para_margin_left
                        };
                        let para_margin_right = para_style_ref.map(|s| s.margin_right).unwrap_or(0.0);
                        let avail_w = (col_area.width - effective_margin_left - para_margin_right).max(pic_w);
                        let pic_x = match para_alignment {
                            Alignment::Center | Alignment::Distribute =>
                                col_area.x + effective_margin_left + (avail_w - pic_w).max(0.0) / 2.0,
                            Alignment::Right =>
                                col_area.x + effective_margin_left + (avail_w - pic_w).max(0.0),
                            _ => col_area.x + effective_margin_left,
                        };

                        // Task #347: paragraph_layout이 호출되지 않는 빈 문단(텍스트 없음 +
                        // TAC 그림만 있는 경우)에서는 인라인 그림이 누락되어
                        // 박스 프레임 시각이 사라지고 후속 InFrontOfText 표가 위로 겹침.
                        // 호스트 문단에 실제 텍스트가 없으면 여기서 직접 이미지 노드를 생성하고
                        // y_offset을 그림 높이만큼 진행시킨다.
                        let has_real_text = para.text.chars()
                            .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                        // [Task #418/#376] paragraph_layout 의 빈 문단 + TAC Picture 분기에서
                        // 이미 ImageNode 가 emit 되어 inline_shape_position 이 등록된 경우,
                        // 여기서 또 push 하면 이중 emit 이 된다. 등록된 경우 push 를 스킵하고
                        // result_y 만 갱신한다.
                        let already_registered = tree.get_inline_shape_position(
                            page_content.section_index, para_index, control_index,
                        ).is_some();
                        if !has_real_text && !already_registered {
                            let bin_data_id = pic.image_attr.bin_data_id;
                            let image_data = find_bin_data(bin_data_content, bin_data_id)
                                .map(|c| c.data.clone());
                            let crop = {
                                let c = &pic.crop;
                                if c.right > c.left && c.bottom > c.top {
                                    Some((c.left, c.top, c.right, c.bottom))
                                } else {
                                    None
                                }
                            };
                            let original_size_hu = if pic.shape_attr.original_width > 0
                                && pic.shape_attr.original_height > 0
                            {
                                Some((pic.shape_attr.original_width, pic.shape_attr.original_height))
                            } else {
                                None
                            };
                            let img_id = tree.next_id();
                            let img_node = RenderNode::new(
                                img_id,
                                RenderNodeType::Image(ImageNode {
                                    section_index: Some(page_content.section_index),
                                    para_index: Some(para_index),
                                    control_index: Some(control_index),
                                    crop,
                                    original_size_hu,
                                    effect: pic.image_attr.effect,
                                    brightness: pic.image_attr.brightness,
                                    contrast: pic.image_attr.contrast,
                                    ..ImageNode::new(bin_data_id, image_data)
                                }),
                                BoundingBox::new(pic_x, pic_y, pic_w, pic_h),
                            );
                            // Task #347: 같은 문단의 InFrontOfText 표가 이미 렌더되어
                            // col_node.children에 들어있으면 그 앞에 끼워넣어 z-order 보존
                            // (인라인 TAC 그림은 박스 프레임 시각이고 InFrontOfText 표가
                            //  본문 콘텐츠로 그 위에 그려져야 함).
                            let insert_pos = col_node.children.iter().position(|c| {
                                matches!(&c.node_type, RenderNodeType::Table(t)
                                    if t.para_index == Some(para_index))
                            });
                            if let Some(pos) = insert_pos {
                                col_node.children.insert(pos, img_node);
                            } else {
                                col_node.children.push(img_node);
                            }
                            // 후속 InFrontOfText 객체의 para_y 기준이 되도록 위치 등록
                            tree.set_inline_shape_position(
                                page_content.section_index, para_index, control_index, pic_x, pic_y,
                            );
                            // [Task #462] LINE_SEG 의 lh+ls 를 advance 로 사용 — 이미지 박스
                            // 높이만 사용하면 leading + line_spacing 이 누락되어 다음 문단이
                            // 그림 바로 아래에 붙음. max(pic_h) 는 LINE_SEG 가 비정상적으로
                            // 작은 경우의 안전장치.
                            let line_advance = para.line_segs.first()
                                .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
                                .unwrap_or(pic_h);
                            result_y = pic_y + line_advance.max(pic_h);
                        } else if !has_real_text && already_registered {
                            // [Task #418/#376] paragraph_layout 가 이미 emit 함 — push 스킵, result_y 만 갱신
                            // [Task #462] 동일하게 LINE_SEG 기반 advance 사용
                            let line_advance = para.line_segs.first()
                                .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
                                .unwrap_or(pic_h);
                            result_y = pic_y + line_advance.max(pic_h);
                        }

                        if let Some(ref caption) = pic.caption {
                            use crate::model::shape::CaptionDirection;
                            let caption_spacing = hwpunit_to_px(caption.spacing as i32, self.dpi);
                            let caption_h = self.calculate_caption_height(&pic.caption, styles);
                            let cap_y = match caption.direction {
                                CaptionDirection::Bottom => pic_y + pic_h + caption_spacing,
                                CaptionDirection::Top => pic_y,
                                _ => pic_y + pic_h + caption_spacing,
                            };
                            if caption.direction == CaptionDirection::Top {
                                let dy = caption_h + caption_spacing;
                                Self::offset_inline_image_y(col_node, para_index, control_index, dy);
                            }
                            let cell_ctx = CellContext {
                                parent_para_index: para_index,
                                path: vec![CellPathEntry {
                                    control_index,
                                    cell_index: 0,
                                    cell_para_index: 0,
                                    text_direction: 0,
                                }],
                            };
                            self.layout_caption(
                                tree, col_node, caption, styles, col_area,
                                pic_x, pic_w, cap_y,
                                &mut self.auto_counter.borrow_mut(),
                                Some(cell_ctx),
                            );
                        }
                    } else {
                        let is_paper_based = (pic.common.vert_rel_to == VertRelTo::Paper || pic.common.vert_rel_to == VertRelTo::Page)
                            && (pic.common.horz_rel_to == HorzRelTo::Paper || pic.common.horz_rel_to == HorzRelTo::Page);
                        if is_paper_based {
                            let mut temp_parent = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Column(0),
                                BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
                            );
                            let paper_area = LayoutRect {
                                x: 0.0, y: 0.0,
                                width: layout.page_width,
                                height: layout.page_height,
                            };
                            let _ = self.layout_body_picture(
                                tree, &mut temp_parent, pic,
                                &paper_area, col_area, &layout.body_area, &paper_area,
                                bin_data_content, styles, Alignment::Left, 0.0,
                                page_content.section_index, para_index, control_index,
                            );
                            for child in temp_parent.children.drain(..) {
                                paper_images.push(child);
                            }
                        } else {
                            let comp = composed.get(para_index);
                            let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
                            let alignment = styles.para_styles.get(para_style_id)
                                .map(|s| s.alignment)
                                .unwrap_or(Alignment::Left);
                            let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
                            let pic_container = LayoutRect {
                                x: col_area.x, y: pic_y,
                                width: col_area.width,
                                height: col_area.height - (pic_y - col_area.y),
                            };
                            result_y = self.layout_body_picture(
                                tree, col_node, pic,
                                &pic_container, col_area, &layout.body_area,
                                &LayoutRect { x: 0.0, y: 0.0, width: layout.page_width, height: layout.page_height },
                                bin_data_content, styles, alignment, pic_y,
                                page_content.section_index, para_index, control_index,
                            );
                        }
                    }
                }
            }
        }
        result_y
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_wrap_around_paras(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        section_index: usize,
        table_para_index: usize,
        wrap_around_paras: &[super::pagination::WrapAroundPara],
        table_y_start: f64,
        table_y_end: f64,
        wrap_text_x: f64,
        wrap_text_width: f64,
        table_content_offset: f64,
        bin_data_content: &[BinDataContent],
        // Task #463: 인라인 floating 표(예: 인용 따옴표 ｢｣)의 우측 끝 x 좌표.
        // wrap host paragraph 의 외곽선이 이 표 위치까지 둘러싸도록 box 너비를
        // 확장하기 위해 caller 에서 계산하여 전달한다. None 이면 box 미확장.
        tbl_x_right: Option<f64>,
    ) {
        // 이 표에 연관된 어울림 문단만 필터링
        let related: Vec<_> = wrap_around_paras.iter()
            .filter(|wp| wp.table_para_index == table_para_index)
            .collect();

        // 표 문단의 LINE_SEG에서 기준 vertical_pos
        let table_para = match paragraphs.get(table_para_index) {
            Some(p) => p,
            None => return,
        };
        let table_seg = match table_para.line_segs.first() {
            Some(s) => s,
            None => return,
        };
        let table_base_vpos = table_seg.vertical_pos;

        // 어울림 텍스트 영역
        // Task #463: wrap_text_x 는 LINE_SEG.column_start 기반으로 paragraph
        // margin_left 를 이미 포함하지만, layout_composed_paragraph 가 col_area.x 에
        // margin_left 를 한 번 더 더하기 때문에 wrap host 텍스트가 한 단계 더
        // 들여쓰기 됨 (학생3 wrap host 가 학생1 보다 +margin_left 만큼 우측으로 밀림).
        // wrap_area.x 를 margin_left 만큼 좌측으로 보정하고 width 도 그만큼 확장.
        // (inner_pad 는 외곽선 안쪽 여백으로 wrap_cs 와 무관하므로 보정 대상 아님)
        let host_para_style = composed.get(table_para_index)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize));
        let host_margin_left = host_para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let host_margin_right = host_para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let wrap_area = LayoutRect {
            x: wrap_text_x - host_margin_left,
            y: col_area.y,
            width: wrap_text_width + host_margin_left + host_margin_right,
            height: col_area.height,
        };

        // 호스트 문단(표 문단) 텍스트를 어울림 영역에 렌더링
        let has_host_text = table_para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
        if table_content_offset == 0.0 {
            if has_host_text {
                if let Some(comp) = composed.get(table_para_index) {
                    let text_start_line = comp.lines.iter().position(|line| {
                        line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                    });
                    if let Some(start_line) = text_start_line {
                        // 호스트 본문의 모든 텍스트 줄을 wrap 영역에 렌더링
                        // (Task #295: 자가 wrap host의 다중 줄 누락 수정)
                        let text_end_line = comp.lines.iter().rposition(|line| {
                            line.runs.iter().any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                        }).map(|i| i + 1).unwrap_or(comp.lines.len());
                        // Task #463: wrap host 의 외곽선은 원래 col_area 너비로 그려야
                        // 인라인 floating 표(인용 따옴표 ｢｣ 등)를 박스가 둘러쌈. tbl_x_right
                        // 가 col_area 우측을 넘으면 그 위치까지 박스 너비를 확장한다.
                        let prev_override = self.border_box_override.get();
                        let extended_width = match tbl_x_right {
                            Some(tx) if tx > col_area.x + col_area.width =>
                                tx - col_area.x,
                            _ => col_area.width,
                        };
                        self.border_box_override.set(Some((col_area.x, extended_width)));
                        self.layout_partial_paragraph(
                            tree, col_node, table_para, Some(comp), styles,
                            &wrap_area, table_y_start, start_line, text_end_line,
                            section_index, table_para_index, None, Some(bin_data_content),
                        );
                        self.border_box_override.set(prev_override);
                        // 어울림 문단은 항상 ↵ 표시 필요 — 부분 렌더링 시 is_para_end 강제 설정
                        force_para_end_on_last_run(col_node);
                    }
                }
            } else {
                // 호스트 문단에 텍스트 없음 (빈 문단 + 표): ↵ 마크 렌더링
                let seg = table_para.line_segs.first();
                let line_height = seg.map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi)).unwrap_or(13.3);
                let font_size = seg.map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi)).unwrap_or(13.3);
                let baseline = font_size * 0.8;
                let line_id = tree.next_id();
                let line_node = RenderNode::new(
                    line_id,
                    RenderNodeType::TextLine(TextLineNode::new(line_height, font_size)),
                    BoundingBox::new(wrap_text_x, table_y_start, font_size, line_height),
                );
                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: String::new(),
                        style: TextStyle {
                            font_family: "바탕".to_string(),
                            font_size,
                            color: 0x000000,
                            ..Default::default()
                        },
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: None,
                        para_index: Some(table_para_index),
                        char_start: None,
                        cell_context: None,
                        is_para_end: true,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(wrap_text_x, table_y_start, 0.0, line_height),
                );
                let mut line_container = line_node;
                line_container.children.push(run_node);
                col_node.children.push(line_container);
            }
        }

        if related.is_empty() {
            return;
        }

        // 어울림 텍스트 영역: col_area를 cs/sw 기반으로 조정
        let wrap_area = LayoutRect {
            x: wrap_text_x,
            y: col_area.y,
            width: wrap_text_width,
            height: col_area.height,
        };

        for wp in &related {
            let para = match paragraphs.get(wp.para_index) {
                Some(p) => p,
                None => continue,
            };
            let seg = match para.line_segs.first() {
                Some(s) => s,
                None => continue,
            };
            // 어울림 문단의 표 내 vpos 오프셋 → px
            let vpos_offset = seg.vertical_pos - table_base_vpos;
            let abs_y_in_table = crate::renderer::hwpunit_to_px(vpos_offset, self.dpi);

            // 현재 페이지에서의 y
            let para_y = table_y_start + (abs_y_in_table - table_content_offset);

            // 현재 페이지의 표 y 범위 내에서만 렌더링
            if para_y < table_y_start - 1.0 || para_y >= table_y_end {
                continue;
            }

            if wp.has_text {
                // 텍스트 문단: composed paragraph를 사용하여 어울림 영역에 렌더링
                let comp = composed.get(wp.para_index);
                // 다중 LINE_SEG: wrap 영역에 해당하는 첫 줄만 렌더링
                let end_line = if comp.map(|c| c.lines.len()).unwrap_or(1) > 1 {
                    1
                } else {
                    comp.map(|c| c.lines.len()).unwrap_or(1)
                };
                self.layout_partial_paragraph(
                    tree, col_node, para, comp, styles,
                    &wrap_area, para_y, 0, end_line,
                    section_index, wp.para_index, None, Some(bin_data_content),
                );
                // 어울림 문단은 항상 ↵ 표시 필요
                force_para_end_on_last_run(col_node);
            } else {
                // 빈 리턴 문단: ↵ 마크 렌더링
                let line_height = crate::renderer::hwpunit_to_px(seg.line_height, self.dpi);
                // 문단의 글자 모양에서 실제 폰트 크기 추출
                let font_size = {
                    let cs_id = para.char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0);
                    styles.char_styles.get(cs_id as usize)
                        .map(|cs| cs.font_size)
                        .filter(|fs| *fs > 0.0)
                        .unwrap_or(13.3)
                };
                let mark_x = wrap_text_x;

                let line_id = tree.next_id();
                let line_node = RenderNode::new(
                    line_id,
                    RenderNodeType::TextLine(TextLineNode::new(line_height, font_size)),
                    BoundingBox::new(mark_x, para_y, font_size, line_height),
                );

                let run_id = tree.next_id();
                let baseline = font_size * 0.8;
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: String::new(),
                        style: TextStyle {
                            font_family: "바탕".to_string(),
                            font_size,
                            color: 0x000000,
                            ..Default::default()
                        },
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: None,
                        para_index: Some(wp.para_index),
                        char_start: None,
                        cell_context: None,
                        is_para_end: true,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(mark_x, para_y, 0.0, line_height),
                );

                let mut line_container = line_node;
                line_container.children.push(run_node);
                col_node.children.push(line_container);
            }
        }
    }

    /// 글상자(Shape) 2차 패스: z-order 정렬 후 렌더링.
    #[allow(clippy::too_many_arguments)]
    fn layout_column_shapes_pass(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        col_content: &ColumnContent,
        page_content: &PageContent,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        layout: &PageLayoutInfo,
        col_area: &LayoutRect,
        para_start_y: &std::collections::HashMap<usize, f64>,
    ) {
        let mut shape_render_items: Vec<(i32, usize, usize, f64, Alignment)> = Vec::new();
        for item in &col_content.items {
            if let PageItem::Shape { para_index, control_index } = item {
                let para_y = para_start_y.get(para_index).copied().unwrap_or(col_area.y);
                let comp = composed.get(*para_index);
                let para_style_id = if let Some(para) = paragraphs.get(*para_index) {
                    comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize)
                } else {
                    0
                };
                let alignment = styles.para_styles.get(para_style_id)
                    .map(|s| s.alignment)
                    .unwrap_or(Alignment::Left);
                let z_order = paragraphs.get(*para_index)
                    .and_then(|p| p.controls.get(*control_index))
                    .map(|ctrl| match ctrl {
                        Control::Shape(shape) => shape.z_order(),
                        Control::Table(table) => table.common.z_order,
                        _ => 0,
                    })
                    .unwrap_or(0);
                shape_render_items.push((z_order, *para_index, *control_index, para_y, alignment));
            }
        }
        shape_render_items.sort_by_key(|item| item.0);

        let overflow_map = self.scan_textbox_overflow(paragraphs, &shape_render_items);

        for (_, para_index, control_index, para_y, alignment) in shape_render_items {
            let ctrl = paragraphs.get(para_index)
                .and_then(|p| p.controls.get(control_index));
            let is_paper_based = ctrl
                .map(|ctrl| {
                    let common = match ctrl {
                        Control::Shape(s) => Some(s.common()),
                        Control::Table(t) => Some(&t.common),
                        _ => None,
                    };
                    common.map(|c| {
                        matches!(c.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page)
                        || matches!(c.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
                    }).unwrap_or(false)
                })
                .unwrap_or(false);
            let is_table_control = ctrl.map(|c| matches!(c, Control::Table(_))).unwrap_or(false);

            let paper_area = LayoutRect {
                x: 0.0, y: 0.0,
                width: layout.page_width,
                height: layout.page_height,
            };

            if is_table_control {
                // InFrontOfText/BehindText 표: paper 기준 절대 위치에 렌더링
                if let Some(Control::Table(table)) = paragraphs.get(para_index)
                    .and_then(|p| p.controls.get(control_index))
                {
                    let mut temp_parent = RenderNode::new(
                        tree.next_id(),
                        RenderNodeType::Column(0),
                        BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
                    );
                    self.layout_table(
                        tree, &mut temp_parent, table,
                        page_content.section_index, styles, col_area, para_y,
                        bin_data_content, None, 0,
                        Some((para_index, control_index)),
                        alignment, None, 0.0, 0.0, None, None, None,
                    );
                    for child in temp_parent.children.drain(..) {
                        paper_images.push(child);
                    }
                }
            } else if is_paper_based {
                let mut temp_parent = RenderNode::new(
                    tree.next_id(),
                    RenderNodeType::Column(0),
                    BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
                );
                self.layout_shape(
                    tree,
                    &mut temp_parent,
                    paragraphs,
                    para_index,
                    control_index,
                    page_content.section_index,
                    styles,
                    col_area,
                    &layout.body_area,
                    &paper_area,
                    para_y,
                    alignment,
                    bin_data_content,
                    &overflow_map,
                );
                for child in temp_parent.children.drain(..) {
                    paper_images.push(child);
                }
            } else {
                self.layout_shape(
                    tree,
                    col_node,
                    paragraphs,
                    para_index,
                    control_index,
                    page_content.section_index,
                    styles,
                    col_area,
                    &layout.body_area,
                    &paper_area,
                    para_y,
                    alignment,
                    bin_data_content,
                    &overflow_map,
                );
            }
        }
    }

    /// treat_as_char 이미지의 x 좌표를 텍스트 위치 기반으로 계산한다.
    ///
    /// h_offset=0인 HWP 파일에서 올바른 인라인 이미지 위치를 결정하기 위해
    /// 문단의 텍스트 시뮬레이션으로 해당 제어 문자 위치의 x를 계산한다.
    fn compute_tac_pic_x(
        &self,
        para: &Paragraph,
        comp: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        control_index: usize,
    ) -> f64 {
        use crate::document_core::find_control_text_positions;

        let positions = find_control_text_positions(para);
        let ctrl_text_pos = positions.get(control_index).copied().unwrap_or(0);

        // margin_left를 미리 계산 (text_pos=0 early return에도 사용)
        let para_style_id_for_ml = comp.map(|c| c.para_style_id as usize).unwrap_or(0);
        let margin_left = styles.para_styles.get(para_style_id_for_ml)
            .map(|s| s.margin_left).unwrap_or(0.0);
        // x_base: 텍스트가 시작되는 절대 x 위치 (문단 첫 글자 위치)
        let x_base = col_area.x + margin_left;

        // text_pos=0 이면 문단 첫 글자 위치(margin_left 포함)에서 시작
        if ctrl_text_pos == 0 {
            return x_base;
        }

        let comp = match comp {
            Some(c) => c,
            None => return x_base,
        };
        let para_style = styles.para_styles.get(comp.para_style_id as usize);
        let tab_width = para_style.map(|s| s.default_tab_width).unwrap_or(48.0);
        let tab_stops = para_style.map(|s| s.tab_stops.clone()).unwrap_or_default();
        let auto_tab_right = para_style.map(|s| s.auto_tab_right).unwrap_or(false);
        let available_width = col_area.width - margin_left;

        // ctrl_text_pos 이전에 있는 treat_as_char 컨트롤(text_pos > 0)의 너비 목록
        let mut preceding_tac: Vec<(usize, f64)> = para.controls.iter().enumerate()
            .filter_map(|(ci, ctrl)| {
                if ci >= control_index { return None; }
                let tp = positions.get(ci).copied().unwrap_or(0);
                if tp == 0 || tp >= ctrl_text_pos { return None; }
                let w = match ctrl {
                    Control::Picture(p) if p.common.treat_as_char => {
                        hwpunit_to_px(p.common.width as i32, self.dpi)
                    }
                    Control::Shape(s) if s.common().treat_as_char => {
                        hwpunit_to_px(s.common().width as i32, self.dpi)
                    }
                    _ => return None,
                };
                Some((tp, w))
            })
            .collect();
        preceding_tac.sort_by_key(|(tp, _)| *tp);

        // 첫 번째 줄의 텍스트 런을 순회하며 ctrl_text_pos까지의 x 누적
        let first_line = match comp.lines.first() {
            Some(l) => l,
            None => return x_base,
        };

        let mut est_x = 0.0f64; // x_base로부터의 상대 오프셋
        let mut char_idx: usize = 0;
        let mut tac_pos = 0usize;

        'outer: for run in &first_line.runs {
            let mut ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
            ts.default_tab_width = tab_width;
            ts.tab_stops = tab_stops.clone();
            ts.auto_tab_right = auto_tab_right;
            ts.available_width = available_width;

            for ch in run.text.chars() {
                // 현재 char_idx 위치에 삽입된 preceding tac 컨트롤 너비 추가
                while tac_pos < preceding_tac.len() && preceding_tac[tac_pos].0 <= char_idx {
                    est_x += preceding_tac[tac_pos].1;
                    tac_pos += 1;
                }
                if char_idx >= ctrl_text_pos {
                    break 'outer;
                }
                ts.line_x_offset = est_x;
                if ch == '\t' {
                    let (tp, _, _) = find_next_tab_stop(
                        est_x, &ts.tab_stops, ts.default_tab_width, ts.auto_tab_right, ts.available_width,
                    );
                    est_x = tp;
                } else {
                    est_x += estimate_text_width(&ch.to_string(), &ts);
                }
                char_idx += 1;
            }
        }

        x_base + est_x
    }
}

/// TAC 표 앞의 선행 텍스트(주로 공백) 폭을 계산한다.
///
/// `composed.lines[0]` 의 runs 중 target TAC 이전 문자 범위의 폭을 합산.
/// TAC 문단에 `PageItem::FullParagraph` 가 발행되지 않아 `paragraph_layout`
/// 가 호출되지 않는 경우(선행 공백만 있는 TAC 표 등)에 `layout_table_item`
/// 에서 표 inline x 좌표를 복원하기 위해 사용한다.
/// Task #463: 인라인 wrap=Square floating 표의 우측 끝 x 좌표 계산.
/// `table_layout::compute_table_x_position` 의 depth=0 + Column-relative
/// 경로와 동일한 공식을 사용하여, paragraph border box 가 표를 둘러쌀 수
/// 있도록 한다. 인용 따옴표 ｢｣ 처럼 col_area 우측을 horizontal_offset 만큼
/// 넘는 표를 정확히 처리한다.
fn compute_square_wrap_tbl_x_right(
    t: &crate::model::table::Table,
    col_area: &LayoutRect,
    dpi: f64,
) -> f64 {
    use crate::model::shape::HorzAlign;
    let tbl_w = crate::renderer::hwpunit_to_px(t.common.width as i32, dpi);
    let h_offset = crate::renderer::hwpunit_to_px(t.common.horizontal_offset as i32, dpi);
    let tbl_x = match t.common.horz_align {
        // table_layout.rs:966 와 동일: ref_x + (ref_w - table_width) - h_offset.
        // 이후 inline_x_override 경로(line 924-925)에서 +h_offset 가산되어
        // 최종 x = ref_x + (ref_w - table_width). h_offset 효과는 상쇄됨.
        // 그러나 실제 렌더된 좌표(empirical: 526.93) 는 ref_x+(ref_w-tw)+h_offset 임.
        // 여기서는 tbl_inline_x(line 2218)와 일관되게 단순 우측정렬 후
        // h_offset 가산식을 사용한다.
        HorzAlign::Right | HorzAlign::Outside =>
            col_area.x + col_area.width - tbl_w + h_offset,
        HorzAlign::Center =>
            col_area.x + (col_area.width - tbl_w) / 2.0 + h_offset,
        _ => col_area.x + h_offset,
    };
    tbl_x + tbl_w
}

fn compute_tac_leading_width(
    composed: &ComposedParagraph,
    target_control_index: usize,
    styles: &ResolvedStyleSet,
) -> f64 {
    let Some(first_line) = composed.lines.first() else { return 0.0; };

    // target TAC 이 composed.tac_controls 에 있으면 해당 위치까지 합산.
    // 없으면(블록 취급: 너비 ≥ 90% seg_width 등 is_tac_table_inline 이 false 인 경우)
    // 선행 텍스트는 line 0 전체로 간주하고 모든 run 폭 합산.
    let tac_pos_opt = composed.tac_controls.iter()
        .find(|(_, _, ci)| *ci == target_control_index)
        .map(|(pos, _, _)| *pos);

    let mut char_pos = first_line.char_start;
    let mut width = 0.0;
    for run in &first_line.runs {
        let run_len = run.text.chars().count();
        let style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
        match tac_pos_opt {
            Some(tac_pos) if char_pos + run_len <= tac_pos => {
                width += estimate_text_width(&run.text, &style);
                char_pos += run_len;
            }
            Some(tac_pos) if char_pos < tac_pos => {
                let partial_len = tac_pos - char_pos;
                let partial: String = run.text.chars().take(partial_len).collect();
                width += estimate_text_width(&partial, &style);
                break;
            }
            Some(_) => break,
            None => {
                // block 취급 TAC: 전체 run 합산
                width += estimate_text_width(&run.text, &style);
                char_pos += run_len;
            }
        }
    }
    width
}

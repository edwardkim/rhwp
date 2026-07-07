//! 레이아웃 엔진 (Layout Engine)
//!
//! 페이지 분할 결과를 받아 각 요소의 정확한 위치와 크기를 계산하고
//! 렌더 트리(PageRenderTree)를 생성한다.

use super::composer::{compose_paragraph, effective_text_for_metrics, ComposedParagraph};
use super::float_placement::{
    horizontal_range, is_para_topbottom_float, signed_hwpunit, FloatLaneSet, FloatPlacementContext,
};
use super::font_metrics_data;
use super::height_cursor::HeightCursor;
use super::height_measurer::{fit_measured_table_to_declared_height, MeasuredTable};
use super::page_layout::{LayoutRect, PageLayoutInfo};
use super::pagination::{
    ColumnContent, EndnoteParaSource, FootnoteRef, FootnoteSource, PageContent, PageItem,
};
use super::render_tree::*;
use super::style_resolver::ResolvedStyleSet;
use super::{
    format_number, hwpunit_to_px, px_to_hwpunit, ArrowStyle, AutoNumberCounter, LineStyle,
    NumberFormat as NumFmt, PathCommand, ShapeStyle, StrokeDash, TextStyle, DEFAULT_DPI,
};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::footnote::{FootnoteShape, NumberFormat};
use crate::model::header_footer::MasterPage;
use crate::model::page::{PageBorderBasis, PageBorderFill};
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    Caption, CaptionDirection, CommonObjAttr, HorzAlign, HorzRelTo, ShapeObject, TextWrap,
    VertAlign, VertRelTo,
};
use crate::model::style::{
    Alignment, BorderLine, BorderLineType, HeadType, Numbering, UnderlineType,
};
use crate::model::table::VerticalAlign;

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
    /// [Task #604 R3] anchor ↔ wrap text 매칭 메타데이터 (typeset 출력 → layout 소비)
    wrap_anchors: &'a std::collections::HashMap<usize, super::pagination::WrapAnchorRef>,
}

const ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU: i32 = 1984;

fn effective_tac_segment_width_hu(para: &Paragraph, fallback_width_hu: i32) -> i32 {
    let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
    if seg_width > 0 {
        seg_width
    } else {
        fallback_width_hu.max(0)
    }
}

#[derive(Clone)]
struct PagePreviewImage {
    mime: &'static str,
    data: Vec<u8>,
}

fn para_border_is_visible(border: &BorderLine) -> bool {
    !matches!(border.line_type, BorderLineType::None)
}

fn para_border_same_stroke(a: &BorderLine, b: &BorderLine) -> bool {
    a.line_type == b.line_type && a.width == b.width && a.color == b.color
}

fn para_border_can_use_rect_stroke(
    borders: &[BorderLine; 4],
    skip_top: bool,
    skip_bottom: bool,
) -> bool {
    borders.iter().all(para_border_is_visible)
        // Rectangle stroke 는 dash 정보를 표현하지 못하므로 점선/파선 문단 테두리는
        // 면별 LineNode 경로로 보내야 한컴의 선 모양과 일치한다.
        && borders
            .iter()
            .all(|border| matches!(border.line_type, BorderLineType::Solid))
        && borders[1..]
            .iter()
            .all(|border| para_border_same_stroke(&borders[0], border))
        && !skip_top
        && !skip_bottom
}

/// `Square/어울림` 그림이 문단 중간부터 본문을 감싸는 경우, HWP5는
/// `LINE_SEG`에서 그림 옆으로 좁아지는 첫 줄의 `vertical_pos`를 저장한다.
/// 개체 자체도 그 줄의 top에 맞춰야 한컴의 “서로 자리를 침범하지 않는”
/// 어울림 배치가 된다.
///
/// 일부 HWP5 원본은 문단 첫 줄의 `vertical_pos`가 0이 아니라 페이지/구역
/// 흐름 기준 누적값이다. 이때 좁아지는 줄의 raw vpos를 그대로 문단 y에
/// 더하면 `para_y + absolute_vpos`가 되어 그림이 페이지 하단 밖으로 밀린다.
/// 따라서 그림 배치에는 문단 첫 줄 대비 상대 delta만 사용한다.
fn square_wrap_first_narrow_line_vpos_px(
    para: &Paragraph,
    col_area: &LayoutRect,
    dpi: f64,
) -> Option<f64> {
    if para.line_segs.len() < 2 {
        return None;
    }
    let col_w_hu = px_to_hwpunit(col_area.width, dpi);
    let first_wrap_idx = para
        .line_segs
        .iter()
        .position(|seg| seg.is_in_wrap_zone(col_w_hu))?;
    if first_wrap_idx == 0 {
        return None;
    }
    let has_full_width_before = para.line_segs[..first_wrap_idx]
        .iter()
        .any(|seg| !seg.is_in_wrap_zone(col_w_hu) && seg.segment_width > 0);
    if !has_full_width_before {
        return None;
    }
    let base_vpos = para.line_segs.first()?.vertical_pos;
    let narrow_vpos = para.line_segs[first_wrap_idx].vertical_pos;
    if narrow_vpos < base_vpos {
        return None;
    }
    Some(hwpunit_to_px(narrow_vpos - base_vpos, dpi))
}

fn table_has_detached_para_flow_object(table: &crate::model::table::Table) -> bool {
    table
        .cells
        .iter()
        .flat_map(|cell| cell.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .any(|ctrl| match ctrl {
            Control::Picture(pic) => {
                !pic.common.treat_as_char
                    && !pic.common.flow_with_text
                    && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                    && matches!(pic.common.vert_rel_to, VertRelTo::Para)
            }
            Control::Shape(shape) => {
                let common = shape.common();
                !common.treat_as_char
                    && !common.flow_with_text
                    && matches!(common.text_wrap, TextWrap::TopAndBottom)
                    && matches!(common.vert_rel_to, VertRelTo::Para)
            }
            _ => false,
        })
}

type ParaFloatLanes = std::collections::HashMap<usize, FloatLaneSet>;

#[derive(Debug, Clone, Copy)]
struct VisibleFloatExclusion {
    /// visible host 문단의 양수 offset 자리차지 표가 후속 본문을 밀어내야 하는 y 구간.
    top: f64,
    bottom: f64,
    /// 이 zone 을 만든 표가 앵커된 host 문단 index. 같은 문단의 텍스트(섹션 제목)는
    /// 자기 표가 만든 zone 에 밀리면 안 된다 — 한컴은 제목을 문단 앵커(표 위)에 두고
    /// 양수 offset 표를 그 아래에 둔다. consume 시 self-owned zone 을 skip 하는 데 쓴다.
    owner_para: usize,
}

fn render_node_contains_text_for_para(node: &RenderNode, para_index: usize) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(para_index) {
            return true;
        }
    }
    node.children
        .iter()
        .any(|child| render_node_contains_text_for_para(child, para_index))
}

fn insert_before_para_text(parent: &mut RenderNode, para_index: usize, mut nodes: Vec<RenderNode>) {
    if nodes.is_empty() {
        return;
    }
    if let Some(pos) = parent
        .children
        .iter()
        .position(|child| render_node_contains_text_for_para(child, para_index))
    {
        for (offset, node) in nodes.drain(..).enumerate() {
            parent.children.insert(pos + offset, node);
        }
    } else {
        parent.children.extend(nodes);
    }
}

fn page_item_is_treat_as_char_picture_only(item: &PageItem, paragraphs: &[Paragraph]) -> bool {
    let para_index = match item {
        PageItem::FullParagraph { para_index }
        | PageItem::PartialParagraph { para_index, .. }
        | PageItem::Table { para_index, .. }
        | PageItem::PartialTable { para_index, .. }
        | PageItem::Shape { para_index, .. } => *para_index,
        PageItem::EndnoteSeparator { .. } => return false,
    };
    paragraphs
        .get(para_index)
        .map(|para| {
            para.text.trim().is_empty()
                && para.controls.iter().any(|ctrl| match ctrl {
                    Control::Picture(pic) => pic.common.treat_as_char,
                    Control::Shape(shape) => shape.common().treat_as_char,
                    _ => false,
                })
        })
        .unwrap_or(false)
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
    pub fn outermost_control(&self) -> usize {
        self.path[0].control_index
    }
    /// 최외곽 표의 셀 인덱스
    pub fn outermost_cell(&self) -> usize {
        self.path[0].cell_index
    }
    /// 최외곽 표의 셀 문단 인덱스
    pub fn outermost_cell_para(&self) -> usize {
        self.path[0].cell_para_index
    }
    /// 최내곽 레벨의 엔트리
    pub fn innermost(&self) -> &CellPathEntry {
        self.path.last().unwrap()
    }
    /// 텍스트 방향 (최내곽 기준)
    pub fn text_direction(&self) -> u8 {
        self.innermost().text_direction
    }

    /// (cell_index, cell_para_index, outer_table_control_index) — 최내곽 entry 의 3 필드.
    /// ImageNode / RectangleNode 등의 cell context 3 필드 매핑 boilerplate 통합용.
    /// path 가 비어있으면 (None, None, None).
    pub fn last_image_indices(&self) -> (Option<usize>, Option<usize>, Option<usize>) {
        match self.path.last() {
            Some(e) => (
                Some(e.cell_index),
                Some(e.cell_para_index),
                Some(e.control_index),
            ),
            None => (None, None, None),
        }
    }
}

fn para_has_visible_text(para: &Paragraph) -> bool {
    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
}

fn para_has_non_whitespace_text(para: &Paragraph) -> bool {
    para.text
        .chars()
        .any(|c| c > '\u{001F}' && c != '\u{FFFC}' && !c.is_whitespace())
}

fn para_line_spacing_px(para: &Paragraph, dpi: f64) -> f64 {
    para.line_segs
        .last()
        .filter(|seg| seg.line_spacing > 0)
        .map(|seg| hwpunit_to_px(seg.line_spacing, dpi))
        .unwrap_or(0.0)
}

fn has_following_non_positive_visible_float(para: &Paragraph, control_index: usize) -> bool {
    para.controls
        .iter()
        .skip(control_index + 1)
        .any(|ctrl| match ctrl {
            Control::Table(table) => {
                is_para_topbottom_float(&table.common)
                    && signed_hwpunit(table.common.vertical_offset) <= 0
            }
            _ => false,
        })
}

fn para_has_visible_inline_control(para: &Paragraph) -> bool {
    para.controls.iter().any(|ctrl| match ctrl {
        Control::Picture(pic) => pic.common.treat_as_char,
        Control::Shape(shape) => shape.common().treat_as_char,
        Control::Table(table) => table.common.treat_as_char,
        Control::Equation(eq) => eq.common.treat_as_char,
        Control::Form(_) => true,
        _ => false,
    })
}

fn para_is_empty_topbottom_table_anchor(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, Control::Table(t) if is_para_topbottom_float(&t.common)))
}

fn inline_equation_count(para: &Paragraph) -> usize {
    para.controls
        .iter()
        .filter(|ctrl| matches!(ctrl, Control::Equation(eq) if eq.common.treat_as_char))
        .count()
}

fn same_endnote_control(a: &EndnoteParaSource, b: &EndnoteParaSource) -> bool {
    a.section_index == b.section_index
        && a.para_index == b.para_index
        && a.control_index == b.control_index
}

fn para_large_tac_picture_or_shape_height_px(para: &Paragraph, dpi: f64) -> Option<f64> {
    para.controls
        .iter()
        .filter_map(|ctrl| match ctrl {
            Control::Picture(pic) if pic.common.treat_as_char => Some(
                hwpunit_to_px(pic.common.height as i32, dpi)
                    .max(hwpunit_to_px(pic.shape_attr.current_height as i32, dpi)),
            ),
            Control::Shape(shape) if shape.common().treat_as_char => {
                Some(hwpunit_to_px(shape.common().height as i32, dpi))
            }
            _ => None,
        })
        .reduce(f64::max)
}

fn endnote_question_number(para: &Paragraph) -> Option<u16> {
    let text = para.text.trim_start().strip_prefix('문')?;
    let digits: String = text.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn textless_non_tac_topbottom_object_tail_advance_px(
    para: &Paragraph,
    control_index: usize,
    dpi: f64,
) -> Option<f64> {
    if para_has_visible_text(para) {
        return None;
    }
    match para.controls.get(control_index)? {
        Control::Picture(pic)
            if !pic.common.treat_as_char
                && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                && matches!(pic.common.vert_rel_to, VertRelTo::Para) =>
        {
            para.line_segs
                .first()
                .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, dpi).max(0.0))
        }
        Control::Shape(shape)
            if !shape.common().treat_as_char
                && matches!(shape.common().text_wrap, TextWrap::TopAndBottom)
                && matches!(shape.common().vert_rel_to, VertRelTo::Para) =>
        {
            Some(hwpunit_to_px(shape.common().margin.bottom as i32, dpi).max(0.0))
        }
        _ => None,
    }
}

fn compact_endnote_title_gap_after_single_equation_tail(
    prev_para: &Paragraph,
    current_para: &Paragraph,
    prev_content_bottom_y: f64,
    y_offset: f64,
    prev_endnote_title_gap_px: f64,
    item_ordinal: usize,
    dpi: f64,
) -> Option<f64> {
    let current_is_endnote_question_title = endnote_question_number(current_para).is_some();
    if item_ordinal > 13
        || prev_endnote_title_gap_px < 50.0
        || !current_is_endnote_question_title
        || inline_equation_count(prev_para) != 1
    {
        return None;
    }

    let consumed_gap = y_offset - prev_content_bottom_y;
    if consumed_gap < prev_endnote_title_gap_px * 0.70 {
        return None;
    }

    let prev_seg = prev_para
        .line_segs
        .iter()
        .rev()
        .find(|seg| seg.segment_width > 0)
        .or_else(|| prev_para.line_segs.last())?;
    let current_first_vpos = current_para.line_segs.first()?.vertical_pos;
    let saved_gap_px = hwpunit_to_px(
        (current_first_vpos - (prev_seg.vertical_pos + prev_seg.line_height)).max(0),
        dpi,
    );
    if saved_gap_px >= prev_endnote_title_gap_px * 0.70
        && saved_gap_px <= prev_endnote_title_gap_px * 1.20
    {
        // 저장 vpos가 20mm급 미주 사이 간격 자체를 이미 표현하는 경계는
        // 단일 수식 tail 압축 대상으로 보지 않는다.
        return None;
    }

    // 페이지/단 첫머리로 이어진 미주 tail 뒤의 단일 수식 줄은 한컴/PDF에서
    // 20mm gap 전체를 다시 열지 않는다. 저장 vpos가 크게 튄 경우만 기본 7mm
    // 흐름 몫을 남기고, 일반 단일 수식 tail은 실제 수식 하단에 붙여 시작한다.
    let target_gap = if saved_gap_px > prev_endnote_title_gap_px * 1.50 {
        prev_endnote_title_gap_px * 0.35
    } else {
        0.0
    };
    let target_y = prev_content_bottom_y + target_gap;
    (target_y + 4.0 < y_offset).then_some(target_y)
}

fn para_has_visible_textless_float_shape_item(
    page_content: &PageContent,
    para: &Paragraph,
    para_index: usize,
) -> bool {
    if para_has_visible_text(para) || para_has_visible_inline_control(para) {
        return false;
    }

    para.controls
        .iter()
        .enumerate()
        .any(|(control_index, ctrl)| {
            let is_float_shape = match ctrl {
                Control::Picture(pic) => !pic.common.treat_as_char,
                Control::Shape(shape) => !shape.common().treat_as_char,
                _ => false,
            };
            is_float_shape
                && page_content.column_contents.iter().any(|cc| {
                    cc.items.iter().any(|it| {
                        matches!(
                            it,
                            PageItem::Shape {
                                para_index: pi,
                                control_index: ci,
                            } if *pi == para_index && *ci == control_index
                        )
                    })
                })
        })
}

fn textless_infront_para_host_requires_line_advance(para: &Paragraph) -> bool {
    if para_has_visible_text(para) {
        return false;
    }

    para.controls.iter().any(|ctrl| match ctrl {
        Control::Picture(pic) => {
            let cm = &pic.common;
            !cm.treat_as_char
                && matches!(cm.text_wrap, TextWrap::InFrontOfText)
                && matches!(cm.vert_rel_to, VertRelTo::Para)
        }
        Control::Shape(shape) => {
            let cm = shape.common();
            !cm.treat_as_char
                && matches!(cm.text_wrap, TextWrap::InFrontOfText)
                && (matches!(cm.vert_rel_to, VertRelTo::Para)
                    || (matches!(cm.vert_rel_to, VertRelTo::Paper)
                        && shape.drawing().and_then(|d| d.text_box.as_ref()).is_some()))
        }
        _ => false,
    })
}

fn paragraph_line_advance_px(
    para: &Paragraph,
    composed: Option<&ComposedParagraph>,
    dpi: f64,
) -> f64 {
    let advance_hu: i32 = composed
        .map(|comp| {
            comp.lines
                .iter()
                .map(|line| line.line_height + line.line_spacing)
                .sum()
        })
        .unwrap_or_else(|| {
            para.line_segs
                .iter()
                .map(|seg| seg.line_height + seg.line_spacing)
                .sum()
        });

    hwpunit_to_px(advance_hu.max(0), dpi)
}

fn square_wrap_table_line_anchor_y(
    para: &Paragraph,
    table: &crate::model::table::Table,
    para_y: f64,
    dpi: f64,
) -> Option<f64> {
    if table.common.treat_as_char
        || !matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::Square
        )
        || !matches!(table.common.vert_rel_to, VertRelTo::Para)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || !matches!(
            table.common.horz_align,
            HorzAlign::Right | HorzAlign::Outside
        )
        || !para_has_visible_text(para)
        || para.line_segs.len() < 2
    {
        return None;
    }

    let first = para.line_segs.first()?;
    let max_width = para
        .line_segs
        .iter()
        .map(|seg| seg.segment_width)
        .max()
        .unwrap_or(0);
    if max_width <= 0 {
        return None;
    }

    let table_width = signed_hwpunit(table.common.width).max(0);
    let min_reduction = (table_width / 3).max(256);
    let anchor = para.line_segs.iter().skip(1).find(|seg| {
        if seg.vertical_pos < first.vertical_pos {
            return false;
        }
        let width_reduced =
            max_width > seg.segment_width && max_width - seg.segment_width >= min_reduction;
        let start_shifted = seg.column_start != first.column_start;
        width_reduced || start_shifted
    })?;

    Some(para_y + hwpunit_to_px(anchor.vertical_pos - first.vertical_pos, dpi))
}

pub(crate) const ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX: f64 = 24.0;
const ENDNOTE_COLUMN_BOTTOM_OVERFLOW_LOG_TOLERANCE_PX: f64 = 48.0;
const ENDNOTE_EQUATION_TAIL_LINE_BOX_OVERFLOW_LOG_TOLERANCE_PX: f64 = 68.0;
const ZERO_ENDNOTE_COLUMN_BOTTOM_OVERFLOW_LOG_TOLERANCE_PX: f64 = 33.0;

pub(crate) fn is_tolerated_endnote_column_bottom_bleed(
    is_endnote_flow: bool,
    content_bottom: f64,
    col_bottom: f64,
) -> bool {
    is_tolerated_endnote_column_bottom_bleed_with_limit(
        is_endnote_flow,
        content_bottom,
        col_bottom,
        ENDNOTE_COLUMN_BOTTOM_OVERFLOW_LOG_TOLERANCE_PX,
    )
}

fn is_tolerated_endnote_column_bottom_bleed_with_limit(
    is_endnote_flow: bool,
    content_bottom: f64,
    col_bottom: f64,
    log_tolerance_px: f64,
) -> bool {
    // 한컴은 compact 미주 하단에서 마지막 줄을 본문 하단보다 약간 아래,
    // 페이지 테두리 안쪽 여백에 남기기도 한다. 이 경우 줄을 다음 쪽으로
    // 넘기면 시각 분기가 틀어지므로, 작은 bleed는 page overflow로 보지 않는다.
    // 9pt 미주에서 수식/빈 TAC guide가 섞인 문단은 line box가 실제 ink보다
    // 크게 계산되어 40px대까지 내려가기도 한다. 조판 분기 기준은 기존 24px를
    // 유지하고 렌더 overflow 로그만 더 넓게 본다.
    is_endnote_flow
        && content_bottom > col_bottom
        && content_bottom <= col_bottom + log_tolerance_px
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
    /// 구역 인덱스 (0-based). [Task #1046] reflow hint 키 = (section_index, para_index).
    pub section_index: usize,
    /// 문단 인덱스 (구역-로컬)
    pub para_index: usize,
    /// 요소 종류
    pub item_type: &'static str,
    /// [Task #1046] 이 항목이 단의 첫 항목인가. true 면 다음 페이지로 이월해도 또 넘침
    /// (본문보다 큰 단일 항목 = page-larger) → reflow 대상 아님.
    pub is_first_in_column: bool,
    /// 요소의 실제 Y 좌표 (배치 후)
    pub element_y: f64,
    /// 단 영역 하단 Y 좌표
    pub column_bottom: f64,
    /// 초과량 (px)
    pub overflow_px: f64,
}

impl std::fmt::Display for LayoutOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LAYOUT_OVERFLOW: page={}, sec={}, col={}, para={}, type={}, first={}, y={:.1}, bottom={:.1}, overflow={:.1}px",
            self.page_index, self.section_index, self.column_index, self.para_index,
            self.item_type, self.is_first_in_column, self.element_y, self.column_bottom, self.overflow_px)
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

/// 빈 TopAndBottom 표 host 문단의 조판부호를 표 시작 위치에 직접 그린다.
fn push_empty_para_end_mark(
    tree: &mut PageRenderTree,
    col_node: &mut RenderNode,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    section_index: usize,
    para_index: usize,
    x: f64,
    y: f64,
    dpi: f64,
) {
    let char_shape_id = para
        .char_shape_id_at(0)
        .or_else(|| para.char_shapes.first().map(|cs| cs.char_shape_id));
    let mut style = char_shape_id
        .map(|id| resolved_to_text_style(styles, id, 0))
        .unwrap_or_default();
    let line_height = para
        .line_segs
        .first()
        .map(|seg| hwpunit_to_px(seg.line_height, dpi))
        .unwrap_or_else(|| style.font_size.max(13.3));

    if style.font_size <= 0.0 {
        style.font_size = line_height.max(13.3);
    }
    if style.font_family.is_empty() {
        style.font_family = "바탕".to_string();
    }

    let font_size = style.font_size.max(1.0);
    let line_height = line_height.max(font_size);
    let baseline = ensure_min_baseline(font_size * 0.8, font_size);
    let line_id = tree.next_id();
    let mut line_node = RenderNode::new(
        line_id,
        RenderNodeType::TextLine(TextLineNode::new(line_height, font_size)),
        BoundingBox::new(x, y, font_size, line_height),
    );

    let run_id = tree.next_id();
    let run_node = RenderNode::new(
        run_id,
        RenderNodeType::TextRun(TextRunNode {
            text: String::new(),
            style,
            char_shape_id,
            para_shape_id: Some(para.para_shape_id),
            section_index: Some(section_index),
            para_index: Some(para_index),
            char_start: Some(0),
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
        BoundingBox::new(x, y, 0.0, line_height),
    );
    line_node.children.push(run_node);
    col_node.children.push(line_node);
}

/// [Task #1027 Stage A] VPOS_CORR 의 보정 목표 y(end_y) 계산 + 클램프(순수).
/// 렌더러(layout)와 페이지네이터(typeset)가 동일 측정을 쓰도록 추출한 공용 함수.
///
/// vpos_end/base 로 보정 목표를 구하고, 렌더러와 동일한 자가검증을 적용한다:
/// - 본문 영역 내(`col_area_y ..= col_area_y+height`)
/// - 단계당 ≤8px 백워드(`MAX_BACKWARD_PX`)
/// - stale table-host(TopAndBottom+vert=Para Table) forward jump 가드(>100px)
///
/// 반환: `(end_y, applied)`. `applied==true` 이면 호출자는 y 를 `end_y` 로 갱신.
pub(crate) fn vpos_corrected_end_y(
    is_page_path: bool,
    col_anchor_y: f64,
    col_area_y: f64,
    col_area_height: f64,
    vpos_end: i32,
    base: i32,
    curr_sb: f64,
    y_offset: f64,
    curr_has_topbottom_para_table: bool,
    skip_spacing_before_prededuct: bool,
    allow_large_backward: bool,
    dpi: f64,
) -> (f64, bool) {
    // [Task #412] page_path: col_anchor_y 기준, lazy_path: col_area_y 기준.
    let anchor = if is_page_path {
        col_anchor_y
    } else {
        col_area_y
    };
    let raw_end_y = anchor + hwpunit_to_px(vpos_end - base, dpi);
    let end_y = if skip_spacing_before_prededuct {
        // HWP3-origin HWP5 변환본은 parser 단계에서 paragraph vpos에서 spacing_before를
        // 이미 분리한다. 여기서 다시 sb_N을 사전 차감하면 문단 사이 간격이 사라져
        // sample16 p3의 3mm 격자 기준보다 본문이 위로 붙는다.
        raw_end_y
    } else {
        raw_end_y - curr_sb
    }
    .max(col_area_y);
    // [Task #643] 단계당 백워드 허용폭 8px.
    const MAX_BACKWARD_PX: f64 = 8.0;
    // [Task #874 #8] stale/inflated vpos forward jump 가드.
    const MAX_TABLE_HOST_FORWARD_PX: f64 = 100.0;
    let stale_table_host_vpos =
        curr_has_topbottom_para_table && end_y > y_offset + MAX_TABLE_HOST_FORWARD_PX;
    let backward_ok = end_y >= y_offset - MAX_BACKWARD_PX || allow_large_backward;
    let applied = end_y >= col_area_y
        && end_y <= col_area_y + col_area_height
        && backward_ok
        && !stale_table_host_vpos;
    (end_y, applied)
}

/// [Task #1027 Stage B] 문단이 vpos 보정을 무효화하는 overlay 개체를 포함하는지.
/// 글앞으로/글뒤로(InFrontOfText/BehindText) 또는 위아래(TopAndBottom)+vert=Para 인
/// 비-TAC Shape/Picture 는 vpos 에 개체 높이가 포함되어 과대하므로, 다음 항목의 vpos
/// 보정 base 산출에서 이 문단을 제외(bypass)한다. tac=true 는 LINE_SEG 에 통합되므로
/// 제외 대상 아님(#539). 렌더러·페이지네이터 공유.
pub(crate) fn para_has_overlay_shape(para: &Paragraph) -> bool {
    use crate::model::shape::{TextWrap, VertRelTo};
    para.controls.iter().any(|c| match c {
        Control::Shape(s) => {
            let cm = s.common();
            if cm.treat_as_char {
                return false;
            }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para)
                    && !cm.treat_as_char)
        }
        Control::Picture(pic) => {
            let cm = &pic.common;
            if cm.treat_as_char {
                return false;
            }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para))
        }
        _ => false,
    })
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
    para_border_ranges:
        std::cell::RefCell<Vec<(u16, f64, f64, f64, f64, f64, f64, bool, bool, usize)>>,
    /// 문단 외곽선 box geometry override (Task #463): wrap=Square 호스트 문단의
    /// 텍스트는 좁은 wrap_area 에서 layout 되지만, 외곽선은 원래 col_area 의
    /// 전체 너비로 그려야 PDF 와 일치한다 (인라인 floating 표를 박스가 둘러쌈).
    /// `layout_wrap_around_paras` 가 호출 직전에 Some(원래 col_area.x, col_area.width)
    /// 로 설정하고, 호출 직후 None 으로 복원한다.
    border_box_override: std::cell::Cell<Option<(f64, f64)>>,
    /// 레이아웃 검증 결과: 경계 초과 목록
    layout_overflows: std::cell::RefCell<Vec<LayoutOverflow>>,
    /// [Task #1046 Stage 3 Class B/C/D] 직전 렌더한 항목(표/문단)의 실제 콘텐츠 하단(y).
    /// 표 뒤/문단 끝에 더해지는 trailing 간격(줄간격/spacing_after/outer_margin)이
    /// 포함된 y_offset 과 달리, 콘텐츠(표 행/마지막 텍스트 줄)가 실제로 점유한 마지막
    /// y 다. overflow 검출이 페이지 바닥의 후행 간격을 콘텐츠 초과로 오판하지 않도록
    /// 이 값으로 비교한다(페이지네이터의 trailing_ls 정책 #359/#404 와 정합). 항목
    /// 디스패치마다 NaN 으로 리셋되고 표/문단 렌더에서만 설정된다.
    last_item_content_bottom: std::cell::Cell<f64>,
    /// 직전 항목의 마지막 미주 줄이 공백 텍스트 + 수식만 가진 tail line-box 인지 여부.
    /// 이런 줄은 실제 ink보다 line box가 훨씬 커져 item overflow 로그만 남을 수 있다.
    last_item_endnote_equation_tail_line_box: std::cell::Cell<bool>,
    /// 빈 줄 감추기로 높이 0 처리된 문단 인덱스 집합
    hidden_empty_paras: std::cell::RefCell<std::collections::HashSet<usize>>,
    /// [Task #1755] 지연 이월 표의 host 텍스트 줄이 typeset 에서 이월 전 쪽에
    /// PartialParagraph 로 pre-emit 된 문단 집합 — 마지막 fragment 뒤 host 렌더 억제.
    pre_emitted_host_paras: std::cell::RefCell<std::collections::HashSet<usize>>,
    /// [#2015] pre-emit 한 host 텍스트 높이(px) — vert_offset 이중계상 보정용.
    pre_emitted_host_heights: std::cell::RefCell<std::collections::HashMap<usize, f64>>,
    /// 렌더용 가상 미주 문단 시작 인덱스
    endnote_para_base: std::cell::Cell<usize>,
    /// 가상 미주 문단별 원본 위치
    endnote_para_sources: std::cell::RefCell<Vec<EndnoteParaSource>>,
    /// [Task #1246] 현재 섹션 미주의 between-notes 마진(HWPUNIT, 0=미적용). HeightCursor 가 미주
    /// 사이 min-gap 보정(gap 부족 시 끌어올림)에 사용한다. 섹션 렌더 셋업마다 갱신.
    endnote_between_notes_hu: std::cell::Cell<i32>,
    /// 현재 섹션 미주의 정규화된 "구분선 위" 마진(HWPUNIT).
    endnote_separator_above_hu: std::cell::Cell<i32>,
    /// 현재 섹션 미주의 정규화된 "구분선 아래" 마진(HWPUNIT).
    endnote_separator_below_hu: std::cell::Cell<i32>,
    /// 현재 활성 필드 위치 — 안내문 렌더링 스킵용
    /// (section_idx, para_idx, control_idx, cell_path)
    /// cell_path: 셀 내 필드일 경우 Some(Vec<(ctrl, cell, para)>)
    active_field:
        std::cell::RefCell<Option<(usize, usize, usize, Option<Vec<(usize, usize, usize)>>)>>,
    /// 조판부호 표시 여부
    show_control_codes: std::cell::Cell<bool>,
    /// 현재 페이지 용지 너비 (표 HorzRelTo::Paper 위치 계산용)
    current_paper_width: std::cell::Cell<f64>,
    /// 현재 페이지 본문 영역 (표 HorzRelTo::Page / VertRelTo::Page 위치 계산용)
    /// (x, y, width, height). 미설정 시 (0, 0, 0, 0) — 호출부에서 col_area로 폴백.
    current_body_area: std::cell::Cell<(f64, f64, f64, f64)>,
    /// HWP3-origin HWP5 변환본 여부.
    is_hwp3_variant: std::cell::Cell<bool>,
    /// HWP3 원본 및 HWP3-origin HWP5 변환본의 본문 흐름 spacing_before 보정 여부.
    use_hwp3_origin_flow_spacing_before: std::cell::Cell<bool>,
    /// [Task #1147 v2] HWPX 원본 여부 — 빈 앵커 TopAndBottom 비-TAC 표 직후 갭을
    /// typeset (host_line_spacing=0) 과 동일하게 0 으로 억제하기 위한 트리거.
    is_hwpx_source: std::cell::Cell<bool>,
    /// [Task #1728 v2] RowBreak 셀-내 continuation 조각의 첫 가시 문단에서만 set.
    /// 이 문단은 셀-상단(is_column_top)이고 셀-상대 인덱스>0 이지만, 한컴은 앞 간격
    /// (spacing_before)을 유지하므로 column-top 트림을 우회해 전량 적용한다.
    keep_continuation_column_top_spacing_before: std::cell::Cell<bool>,
    /// HWPX `Preview/PrvImage.png` 원본. HMapsi OLE처럼 일반 preview stream이 없는
    /// legacy 객체의 제한적 첫 페이지 fallback에 사용한다.
    hwpx_page_preview: std::cell::RefCell<Option<PagePreviewImage>>,
    /// [Task #1949] 셀 콘텐츠 유닛(cell_units) 메모이제이션. 거대 셀(수천 문단·중첩표)이
    /// RowBreak 로 여러 페이지에 걸칠 때, 각 페이지의 컷 판정이 같은 셀의 units 를
    /// 반복 재계산해 O(pages×cell) 로 폭증한다. cell_units 는 (cell,table,styles)의 순수
    /// 함수이고 셀 폭·콘텐츠는 렌더 중 불변이므로 셀 포인터를 키로 캐시한다. 문서
    /// 재조판(paginate) 경계에서 clear 하여 다른 IR 의 포인터 재사용을 방지한다.
    cell_units_cache: std::cell::RefCell<
        std::collections::HashMap<usize, std::rc::Rc<Vec<table_layout::CellUnit>>>,
    >,
}

mod border_rendering;
mod paragraph_layout;
mod picture_footnote;
mod shape_layout;
mod table_cell_content;
mod table_layout;
mod table_partial;
mod text_measurement;
mod utils;

pub(crate) use paragraph_layout::ensure_min_baseline;
pub(crate) use text_measurement::{
    compute_char_positions, estimate_text_width, estimate_text_width_unrounded,
    extract_tab_leaders_with_extended, find_next_tab_stop, is_cjk_char, resolved_to_text_style,
    split_into_clusters,
};
// [Task #826] map_pua_bullet_char 는 통합 테스트 (tests/issue_826.rs) 에서 직접 검증
// (PUA substitution 매핑 정합) — pub 노출.
pub(crate) use border_rendering::{
    body_page_border_outset, border_line_visual_span, border_width_to_px, create_border_line_nodes,
};
pub use paragraph_layout::map_pua_bullet_char;
pub(crate) use utils::{
    drawing_to_line_style, drawing_to_shape_style, find_bin_data, format_page_number,
    layout_rect_to_bbox, picture_display_size_hu, picture_flow_frame_size_hu, resolve_numbering_id,
};

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;

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
            last_item_content_bottom: std::cell::Cell::new(f64::NAN),
            last_item_endnote_equation_tail_line_box: std::cell::Cell::new(false),
            hidden_empty_paras: std::cell::RefCell::new(std::collections::HashSet::new()),
            pre_emitted_host_paras: std::cell::RefCell::new(std::collections::HashSet::new()),
            pre_emitted_host_heights: std::cell::RefCell::new(std::collections::HashMap::new()),
            endnote_para_base: std::cell::Cell::new(usize::MAX),
            endnote_para_sources: std::cell::RefCell::new(Vec::new()),
            endnote_between_notes_hu: std::cell::Cell::new(0),
            endnote_separator_above_hu: std::cell::Cell::new(0),
            endnote_separator_below_hu: std::cell::Cell::new(0),
            active_field: std::cell::RefCell::new(None),
            show_control_codes: std::cell::Cell::new(false),
            current_paper_width: std::cell::Cell::new(0.0),
            current_body_area: std::cell::Cell::new((0.0, 0.0, 0.0, 0.0)),
            is_hwp3_variant: std::cell::Cell::new(false),
            use_hwp3_origin_flow_spacing_before: std::cell::Cell::new(false),
            keep_continuation_column_top_spacing_before: std::cell::Cell::new(false),
            is_hwpx_source: std::cell::Cell::new(false),
            hwpx_page_preview: std::cell::RefCell::new(None),
            cell_units_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// [Task #1949] 셀 단위 레이아웃 캐시를 비운다. 문서 재조판 등 IR 이 바뀌는
    /// 경계에서 호출해 포인터 키 재사용으로 인한 오재사용을 방지한다.
    pub fn clear_layout_caches(&self) {
        self.cell_units_cache.borrow_mut().clear();
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

    pub(crate) fn is_body_flow_col_area(&self, col_area: &LayoutRect) -> bool {
        let (_, body_y, _, body_h) = self.current_body_area.get();
        body_h > 0.0 && (col_area.y - body_y).abs() < 1.0 && (col_area.height - body_h).abs() < 1.0
    }

    fn object_stable_index(para_index: usize, control_index: usize) -> u32 {
        ((para_index.min(u16::MAX as usize) as u32) << 16)
            | control_index.min(u16::MAX as usize) as u32
    }

    fn render_layer_from_common(
        common: &CommonObjAttr,
        para_index: usize,
        control_index: usize,
    ) -> RenderLayerInfo {
        RenderLayerInfo::new(
            Some(common.text_wrap),
            common.z_order,
            Self::object_stable_index(para_index, control_index),
        )
    }

    fn render_layer_from_control(
        control: &Control,
        para_index: usize,
        control_index: usize,
    ) -> Option<RenderLayerInfo> {
        match control {
            Control::Shape(shape) => Some(Self::render_layer_from_common(
                shape.common(),
                para_index,
                control_index,
            )),
            Control::Picture(picture) => Some(Self::render_layer_from_common(
                &picture.common,
                para_index,
                control_index,
            )),
            Control::Table(table) => Some(Self::render_layer_from_common(
                &table.common,
                para_index,
                control_index,
            )),
            Control::Equation(equation) => Some(Self::render_layer_from_common(
                &equation.common,
                para_index,
                control_index,
            )),
            _ => None,
        }
    }

    fn control_common_attr(control: &Control) -> Option<&CommonObjAttr> {
        match control {
            Control::Shape(shape) => Some(shape.common()),
            Control::Picture(picture) => Some(&picture.common),
            Control::Table(table) => Some(&table.common),
            Control::Equation(equation) => Some(&equation.common),
            _ => None,
        }
    }

    fn master_background_common_attr(control: &Control) -> Option<&CommonObjAttr> {
        match control {
            Control::Shape(shape) => Some(shape.common()),
            Control::Picture(picture) => Some(&picture.common),
            _ => None,
        }
    }

    fn render_layer_from_master_control(
        &self,
        control: &Control,
        para_index: usize,
        control_index: usize,
        paper_area: &LayoutRect,
        body_area: &LayoutRect,
    ) -> Option<RenderLayerInfo> {
        let common = Self::control_common_attr(control)?;
        let mut layer = Self::render_layer_from_common(common, para_index, control_index);
        if Self::master_background_common_attr(control).is_some_and(|common| {
            self.is_master_paper_background_control(common, paper_area, body_area)
        }) {
            layer.text_wrap = Some(TextWrap::BehindText);
        }
        Some(layer)
    }

    fn is_master_paper_background_control(
        &self,
        common: &CommonObjAttr,
        paper_area: &LayoutRect,
        body_area: &LayoutRect,
    ) -> bool {
        if !matches!(common.text_wrap, TextWrap::InFrontOfText) {
            return false;
        }
        if !matches!(common.horz_rel_to, HorzRelTo::Paper)
            || !matches!(common.vert_rel_to, VertRelTo::Paper)
        {
            return false;
        }

        let (width, height) = self.resolve_object_size(common, paper_area, body_area, paper_area);
        let (x, y) = self.compute_object_position(
            common,
            width,
            height,
            paper_area,
            paper_area,
            body_area,
            paper_area,
            paper_area.y,
            Alignment::Left,
        );

        let near_origin = (x - paper_area.x).abs() <= 1.0 && (y - paper_area.y).abs() <= 1.0;
        let covers_paper = width >= paper_area.width * 0.95 && height >= paper_area.height * 0.95;
        near_origin && covers_paper
    }

    fn push_layered_paper_children(
        paper_images: &mut Vec<RenderNode>,
        temp_parent: &mut RenderNode,
        layer: RenderLayerInfo,
    ) {
        for mut child in temp_parent.children.drain(..) {
            child.set_layer(layer);
            paper_images.push(child);
        }
    }

    fn render_layer_plane(layer: Option<RenderLayerInfo>) -> u8 {
        match layer.and_then(|layer| layer.text_wrap) {
            Some(TextWrap::BehindText) => 1,
            Some(TextWrap::InFrontOfText) => 3,
            _ => 2,
        }
    }

    /// 종이 기준 렌더 노드의 정렬키 `(plane, z_order, stable_index)`.
    /// 레이아웃 쿼리(`get_page_control_layout_native`)가 컨트롤별 plane/zOrder/stableIndex 를
    /// 프런트 히트테스트에 노출할 때 재사용한다(렌더 정렬과 단일 진실 원천 유지). [Task #1280 v2]
    pub(crate) fn paper_node_sort_key(node: &RenderNode) -> (u8, i32, u32) {
        let layer = node.layer;
        let (z_order, stable_index) = layer
            .map(|layer| (layer.z_order, layer.stable_index))
            .unwrap_or((0, node.id));

        (Self::render_layer_plane(layer), z_order, stable_index)
    }

    fn sort_paper_render_nodes(paper_images: &mut [RenderNode]) {
        paper_images.sort_by_key(Self::paper_node_sort_key);
    }

    /// 빈 줄 감추기 문단 집합 설정
    pub fn set_hidden_empty_paras(&self, paras: &std::collections::HashSet<usize>) {
        *self.hidden_empty_paras.borrow_mut() = paras.clone();
    }

    /// [Task #1755] 이월 전 쪽에 host 텍스트가 pre-emit 된 문단 집합 설정
    pub fn set_pre_emitted_host_paras(&self, paras: &std::collections::HashSet<usize>) {
        *self.pre_emitted_host_paras.borrow_mut() = paras.clone();
    }

    /// [#2015] pre-emit 된 host 텍스트 높이 맵 설정 (vert_offset 이중계상 보정용)
    pub fn set_pre_emitted_host_heights(&self, heights: &std::collections::HashMap<usize, f64>) {
        *self.pre_emitted_host_heights.borrow_mut() = heights.clone();
    }

    /// 렌더용 가상 미주 문단과 원본 Endnote 내부 문단의 매핑을 설정한다.
    pub fn set_endnote_para_sources(&self, base: usize, sources: &[EndnoteParaSource]) {
        self.endnote_para_base.set(base);
        *self.endnote_para_sources.borrow_mut() = sources.to_vec();
    }

    /// [Task #1236] 이 미주 문단의 다음 렌더 문단이 **같은 미주(문제)** 내 연속 문단인지.
    ///
    /// 같은 미주 연속이면 다줄 미주 문단의 마지막 줄에도 trailing 줄간격을 적용해야
    /// 풀이 본문 줄간격이 균일해진다(다줄 문단 다음 줄간격이 좁아지는 #1236 증상 해소).
    /// 미주의 마지막 문단(=다음이 새 문제 = between-notes margin 적용)이면 false 를 반환해
    /// 문제-사이 간격(7mm 등) 중복 가산을 막는다.
    fn endnote_para_has_same_endnote_successor(&self, para_index: usize) -> bool {
        let base = self.endnote_para_base.get();
        let Some(local_idx) = para_index.checked_sub(base) else {
            return false;
        };
        let sources = self.endnote_para_sources.borrow();
        match (sources.get(local_idx), sources.get(local_idx + 1)) {
            (Some(cur), Some(next)) => {
                cur.section_index == next.section_index
                    && cur.para_index == next.para_index
                    && cur.control_index == next.control_index
            }
            _ => false,
        }
    }

    /// [Task #1246] 현재 섹션 미주의 between-notes 마진(HU)을 설정한다(섹션 렌더 셋업마다 호출).
    /// HeightCursor 가 미주 사이 min-gap 보정에 사용. 0 = 미적용.
    pub fn set_endnote_between_notes_hu(&self, between_notes_hu: i32) {
        self.endnote_between_notes_hu.set(between_notes_hu.max(0));
    }

    /// 현재 섹션 미주의 정규화된 "미주 모양" 여백을 설정한다.
    pub fn set_endnote_shape_margins_hu(
        &self,
        separator_above_hu: i32,
        between_notes_hu: i32,
        separator_below_hu: i32,
    ) {
        self.endnote_separator_above_hu
            .set(separator_above_hu.max(0));
        self.endnote_between_notes_hu.set(between_notes_hu.max(0));
        self.endnote_separator_below_hu
            .set(separator_below_hu.max(0));
    }

    pub(crate) fn current_endnote_zero_spacing_profile(&self) -> bool {
        self.endnote_separator_above_hu.get() == 0
            && self.endnote_between_notes_hu.get() == 0
            && self.endnote_separator_below_hu.get() == 0
    }

    fn current_endnote_zero_between_large_separator_profile(&self) -> bool {
        self.endnote_between_notes_hu.get() == 0
            && self.endnote_separator_above_hu.get() > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
            && self.endnote_separator_below_hu.get() > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
    }

    fn endnote_para_source_for(&self, para_index: usize) -> Option<EndnoteParaSource> {
        let base = self.endnote_para_base.get();
        let local_idx = para_index.checked_sub(base)?;
        self.endnote_para_sources.borrow().get(local_idx).cloned()
    }

    pub(crate) fn is_tolerated_current_endnote_bottom_bleed(
        &self,
        is_endnote_flow: bool,
        content_bottom: f64,
        col_bottom: f64,
        equation_tail_line_box: bool,
    ) -> bool {
        let log_tolerance_px = if self.current_endnote_zero_spacing_profile() {
            ZERO_ENDNOTE_COLUMN_BOTTOM_OVERFLOW_LOG_TOLERANCE_PX
        } else if equation_tail_line_box {
            ENDNOTE_EQUATION_TAIL_LINE_BOX_OVERFLOW_LOG_TOLERANCE_PX
        } else {
            ENDNOTE_COLUMN_BOTTOM_OVERFLOW_LOG_TOLERANCE_PX
        };
        is_tolerated_endnote_column_bottom_bleed_with_limit(
            is_endnote_flow,
            content_bottom,
            col_bottom,
            log_tolerance_px,
        )
    }

    fn note_ref_for_endnote_equation(
        &self,
        para_index: usize,
        inner_control_index: usize,
    ) -> Option<NoteControlRef> {
        let base = self.endnote_para_base.get();
        let local_idx = para_index.checked_sub(base)?;
        let src = self.endnote_para_sources.borrow().get(local_idx)?.clone();
        Some(NoteControlRef {
            kind: "endnote".to_string(),
            section_index: src.section_index,
            para_index: src.para_index,
            control_index: src.control_index,
            note_para_index: src.note_para_index,
            inner_control_index,
        })
    }

    /// 번호 상태를 초기화한다.
    pub fn reset_numbering_state(&self) {
        self.numbering_state.borrow_mut().reset();
    }

    pub fn set_hwp3_variant(&self, enabled: bool) {
        self.is_hwp3_variant.set(enabled);
        self.use_hwp3_origin_flow_spacing_before.set(enabled);
    }

    pub fn set_hwp3_origin_flow_spacing_before(&self, enabled: bool) {
        self.use_hwp3_origin_flow_spacing_before.set(enabled);
    }

    /// [Task #1147 v2] HWPX 원본 소스 표시.
    pub fn set_hwpx_source(&self, enabled: bool) {
        self.is_hwpx_source.set(enabled);
    }

    /// HWPX page preview 이미지를 렌더 fallback용으로 설정한다.
    pub fn set_hwpx_page_preview(&self, data: Option<&[u8]>) {
        *self.hwpx_page_preview.borrow_mut() = data.and_then(|bytes| {
            if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                Some(PagePreviewImage {
                    mime: "image/png",
                    data: bytes.to_vec(),
                })
            } else if bytes.starts_with(b"GIF8") {
                Some(PagePreviewImage {
                    mime: "image/gif",
                    data: bytes.to_vec(),
                })
            } else {
                None
            }
        });
    }

    /// 이미 렌더된 인라인 이미지 노드의 y 좌표를 dy만큼 이동 (캡션 Top 보정)
    fn offset_inline_image_y(
        node: &mut RenderNode,
        para_index: usize,
        control_index: usize,
        dy: f64,
    ) {
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
        self.numbering_state
            .borrow_mut()
            .advance(numbering_id, level, None);
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
    pub fn set_active_field(
        &self,
        info: Option<(usize, usize, usize, Option<Vec<(usize, usize, usize)>>)>,
    ) {
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
        page_border_fill: Option<&PageBorderFill>,
        outline_numbering_id: u16,
        wrap_around_paras: &[super::pagination::WrapAroundPara],
    ) -> PageRenderTree {
        let layout = &page_content.layout;
        let mut tree = PageRenderTree::new(
            page_content.page_index,
            layout.page_width,
            layout.page_height,
        );

        // 페이지 배경 (감추기 설정 시 건너뜀)
        let hide_fill = page_content
            .page_hide
            .as_ref()
            .map(|ph| ph.hide_fill)
            .unwrap_or(false);
        if !hide_fill {
            self.build_page_background(
                &mut tree,
                layout,
                page_border_fill,
                styles,
                bin_data_content,
            );
        }

        // 쪽 테두리선 (감추기 설정 시 건너뜀)
        let hide_border = page_content
            .page_hide
            .as_ref()
            .map(|ph| ph.hide_border)
            .unwrap_or(false);
        if !hide_border {
            self.build_page_borders(&mut tree, layout, page_border_fill, styles);
        }

        // 바탕쪽 (감추기 설정 시 건너뜀)
        let hide_master = page_content
            .page_hide
            .as_ref()
            .map(|ph| ph.hide_master_page)
            .unwrap_or(false);
        if !hide_master {
            self.build_master_page(
                &mut tree,
                active_master_page,
                layout,
                composed,
                styles,
                bin_data_content,
                page_content.section_index,
                page_content.page_number,
            );
        }

        // 머리말 (감추기 설정 시 건너뜀)
        let hide_header = page_content
            .page_hide
            .as_ref()
            .map(|ph| ph.hide_header)
            .unwrap_or(false);
        if !hide_header {
            self.build_header(
                &mut tree,
                page_content,
                header_paragraphs,
                composed,
                styles,
                layout,
                bin_data_content,
                page_border_fill,
            );
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
            &mut tree,
            &mut body_node,
            &mut paper_images,
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            measured_tables,
            layout,
            outline_numbering_id,
            wrap_around_paras,
        );

        // 단 구분선은 build_columns 내부의 emit_zone_column_separators 가 zone(또는
        // page layout 폴백)별 콘텐츠 높이로 그린다. 과거 page-level build_column_separators
        // 는 body 전체높이를 고정으로 그려 부분 페이지에서 구분선이 과도하게 길었고,
        // zone emit 과 이중 렌더되어 [Task #1333 v2] 에서 제거되었다.

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
            let body_bottom = body_bbox.y + body_bbox.height;
            let max_bottom = body_bottom + 10.0;
            if clip.y + clip.height > max_bottom {
                clip.height = max_bottom - clip.y;
            }
            body_node.node_type = RenderNodeType::Body {
                clip_rect: Some(clip),
            };
        }

        tree.root.children.push(body_node);

        Self::sort_paper_render_nodes(&mut paper_images);

        // [Task #604 Stage 6] 용지 기준 개체: body 위 z-layer 로 배치 (한컴 변환 메커니즘
        // 정합). Task #1197 부터 Picture/Table/Shape 공통 layer 메타데이터로 같은
        // text-wrap/z-order 축을 보존한다.
        for img_node in paper_images {
            tree.root.children.push(img_node);
        }

        // 각주 영역
        self.build_footnote_area(
            &mut tree,
            page_content,
            paragraphs,
            footnote_shape,
            styles,
            layout,
        );

        // 꼬리말 + 쪽 번호 (감추기 설정 시 건너뜀)
        let hide_footer = page_content
            .page_hide
            .as_ref()
            .map(|ph| ph.hide_footer)
            .unwrap_or(false);
        let mut footer_node = if !hide_footer {
            self.build_footer(
                &mut tree,
                page_content,
                footer_paragraphs,
                composed,
                styles,
                layout,
                bin_data_content,
            )
        } else {
            let fid = tree.next_id();
            RenderNode::new(
                fid,
                RenderNodeType::Footer,
                layout_rect_to_bbox(&layout.footer_area),
            )
        };
        self.build_page_number(
            &mut tree,
            &mut footer_node,
            page_content,
            layout,
            page_border_fill,
        );
        tree.root.children.push(footer_node);

        tree
    }

    /// 머리말/꼬리말 문단을 해당 영역에 레이아웃한다.
    /// [Task #825] `outer_section_index` + `outer_hf_ref` — 머리말/꼬리말 그림 클릭
    /// hit-test marker (Some 일 때 ImageNode 에 전파). None 이면 기존 동작 (그림 미선택).
    #[allow(clippy::too_many_arguments)]
    fn layout_header_footer_paragraphs(
        &self,
        tree: &mut PageRenderTree,
        area_node: &mut RenderNode,
        hf_paragraphs: &[Paragraph],
        _composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        table_area: Option<&LayoutRect>,
        page_index: u32,
        page_number: u32,
        bin_data_content: &[BinDataContent],
        outer_section_index: Option<usize>,
        outer_hf_ref: Option<crate::renderer::render_tree::HeaderFooterImageRef>,
        is_header: bool,
    ) {
        let mut y_offset = area.y;
        for (i, para) in hf_paragraphs.iter().enumerate() {
            // 테이블 컨트롤이 있으면 테이블 렌더링
            let has_table = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            let has_shape = para.controls.iter().any(|c| matches!(c, Control::Shape(_)));
            let has_picture = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Picture(_)));
            if has_table {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(t) = ctrl {
                        let alignment = styles
                            .para_styles
                            .get(para.para_shape_id as usize)
                            .map(|s| s.alignment)
                            .unwrap_or(Alignment::Left);
                        // Task #445: 꼬리말 영역의 wrap=TopAndBottom + vert=Para 표는
                        // 첫 라인의 line_height/2 만큼 아래로 anchor 됨 (HWP 가 line center
                        // 기준으로 표를 배치하는 동작과 일치). 이 보정이 없으면 페이지 번호
                        // 박스가 본문 바닥과 붙어 보이는 문제(Task #445) 발생.
                        // [Issue #924] 머릿말에서는 적용하지 않음 — 표가 header_area 안에 정확히 위치해야 함.
                        // 꼬리말은 Task #445에서 필요하므로 유지.
                        let line_anchor_offset = if !is_header
                            && matches!(
                                t.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                            && i == 0
                        {
                            let lh_hu = para
                                .line_segs
                                .first()
                                .map(|ls| ls.line_height as i32)
                                .unwrap_or(0);
                            hwpunit_to_px(lh_hu, self.dpi) / 2.0
                        } else {
                            0.0
                        };
                        let table_y = y_offset + line_anchor_offset;
                        let table_area = table_area.unwrap_or(area);
                        y_offset = self.layout_table(
                            tree,
                            area_node,
                            t.as_ref(),
                            0,
                            styles,
                            0,
                            table_area,
                            table_y,
                            bin_data_content,
                            None,
                            0,
                            Some((i, ci)),
                            alignment,
                            None,
                            0.0,
                            0.0,
                            None,
                            None,
                            None,
                            false,
                            is_header,
                        );
                    }
                }
            } else if has_picture {
                // Picture 컨트롤이 있는 문단
                let comp = self.compose_header_footer_paragraph(para, page_number);
                if comp.tac_controls.is_empty() {
                    // 머리말/꼬리말 내 Picture: header/footer area 기준 배치
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Picture(pic) = ctrl {
                            if pic.common.treat_as_char {
                                let pic_container = LayoutRect {
                                    x: area.x,
                                    y: y_offset,
                                    width: area.width,
                                    height: area.height - (y_offset - area.y),
                                };
                                // [Task #825] inner para_index = i (hf_paragraphs 안 인덱스),
                                // inner control_index = ci. outer 위치는 outer_hf_ref 보존.
                                self.layout_picture_full(
                                    tree,
                                    area_node,
                                    pic,
                                    &pic_container,
                                    bin_data_content,
                                    Alignment::Left,
                                    outer_section_index,
                                    Some(i),
                                    Some(ci),
                                    outer_hf_ref.clone(),
                                    None, // [Task #1151 v4] cell_ctx: 머리말/꼬리말 path
                                );
                            } else {
                                self.layout_header_footer_picture(
                                    tree,
                                    area_node,
                                    pic,
                                    area,
                                    body_area,
                                    paper_area,
                                    y_offset,
                                    bin_data_content,
                                    outer_section_index,
                                    i,
                                    ci,
                                    outer_hf_ref.clone(),
                                );
                            }
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            y_offset += pic_h;
                        }
                    }
                } else {
                    // TAC Picture: layout_paragraph에서 인라인 배치
                    y_offset = self.layout_paragraph(
                        tree,
                        area_node,
                        para,
                        Some(&comp),
                        styles,
                        area,
                        y_offset,
                        0,
                        usize::MAX - i,
                        None,
                        Some(bin_data_content),
                        None, // 머리말/꼬리말 컨텍스트 — wrap zone 무관
                    );
                }
            } else if has_shape {
                // Shape 컨트롤 렌더링 (머리말/꼬리말 내 글상자 등)
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(_) = ctrl {
                        self.layout_shape(
                            tree,
                            area_node,
                            hf_paragraphs,
                            i,
                            ci,
                            0, // section_index
                            styles,
                            area,
                            body_area,
                            paper_area,
                            y_offset,
                            Alignment::Left,
                            bin_data_content,
                            &std::collections::HashMap::new(),
                            is_header,
                        );
                    }
                }
                // 텍스트도 함께 렌더링
                if !para.text.is_empty() {
                    let comp = self.compose_header_footer_paragraph(para, page_number);
                    y_offset = self.layout_paragraph(
                        tree,
                        area_node,
                        para,
                        Some(&comp),
                        styles,
                        area,
                        y_offset,
                        0,
                        usize::MAX - i,
                        None,
                        None,
                        None, // 머리말/꼬리말 컨텍스트 — wrap zone 무관
                    );
                }
            } else {
                // 일반 텍스트 문단 레이아웃 (필드 마커 치환 포함)
                let comp = self.compose_header_footer_paragraph(para, page_number);
                y_offset = self.layout_paragraph(
                    tree,
                    area_node,
                    para,
                    Some(&comp),
                    styles,
                    area,
                    y_offset,
                    0,
                    usize::MAX - i,
                    None,
                    None,
                    None, // 머리말/꼬리말 컨텍스트 — wrap zone 무관
                );
            }
            if y_offset >= area.y + area.height {
                break;
            }
        }
    }

    fn compose_header_footer_paragraph(
        &self,
        para: &Paragraph,
        page_number: u32,
    ) -> ComposedParagraph {
        let mut comp = compose_paragraph(para);
        self.substitute_hf_field_markers(&mut comp, page_number);
        if para.controls.iter().any(|ctrl| {
            matches!(ctrl, Control::AutoNumber(an)
                if an.number_type == crate::model::control::AutoNumberType::Page)
        }) {
            self.substitute_page_auto_numbers_in_composed(para, &mut comp, page_number);
        }
        comp
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
                if !run.text.contains('\u{0015}')
                    && !run.text.contains('\u{0016}')
                    && !run.text.contains('\u{0017}')
                {
                    new_runs.push(run.clone());
                    continue;
                }
                // 마커가 포함된 런 → 치환 후 분할
                let replaced = run
                    .text
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

    /// `AutoNumber(Page)` 컨트롤의 placeholder 문자만 현재 쪽번호로 치환한다.
    ///
    /// HWPX는 `<hp:autoNum numType="PAGE">` 뒤에 `<hp:fwSpace/>` 같은 공백 텍스트를
    /// 같은 문단에 둘 수 있다. 공백 run 전체를 쪽번호로 바꾸면 짝수 머리말처럼
    /// `쪽번호 + 전각공백 + 제목` 구조에서 쪽번호가 두 번 출력될 수 있으므로,
    /// 컨트롤 placeholder 한 글자만 치환한다.
    pub(crate) fn substitute_page_auto_numbers_in_composed(
        &self,
        para: &Paragraph,
        comp: &mut ComposedParagraph,
        page_number: u32,
    ) {
        if page_number == 0 {
            return;
        }

        let page_str = page_number.to_string();

        for line in &mut comp.lines {
            for run in &mut line.runs {
                if run.text.contains('\u{0015}') {
                    run.text = run.text.replace('\u{0015}', &page_str);
                    run.display_text = None;
                }
            }
        }

        let mut positions = self.page_auto_number_placeholder_positions(para);
        positions.sort_unstable();
        positions.dedup();
        for pos in positions.into_iter().rev() {
            Self::replace_composed_char(comp, pos, &page_str);
        }
    }

    fn page_auto_number_placeholder_positions(&self, para: &Paragraph) -> Vec<usize> {
        let ctrl_positions = crate::document_core::helpers::find_control_text_positions(para);
        let text_chars: Vec<char> = para.text.chars().collect();
        let mut positions = Vec::new();
        let mut search_from = 0usize;

        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            if !matches!(
                ctrl,
                Control::AutoNumber(an)
                    if an.number_type == crate::model::control::AutoNumberType::Page
            ) {
                continue;
            }

            let direct_pos = ctrl_positions.get(ctrl_idx).copied().filter(|&pos| {
                Self::is_auto_number_placeholder_at(para, &text_chars, pos)
                    || text_chars
                        .get(pos)
                        .map_or(false, |ch| Self::is_auto_number_placeholder_char(*ch))
            });

            let pos = direct_pos.or_else(|| {
                Self::find_auto_number_placeholder_char(para, &text_chars, search_from)
            });

            if let Some(pos) = pos {
                positions.push(pos);
                search_from = pos.saturating_add(1);
            }
        }

        positions
    }

    fn is_auto_number_placeholder_char(ch: char) -> bool {
        ch == '\u{0015}' || ch.is_whitespace()
    }

    fn is_auto_number_placeholder_at(para: &Paragraph, text_chars: &[char], idx: usize) -> bool {
        if !text_chars
            .get(idx)
            .map_or(false, |ch| Self::is_auto_number_placeholder_char(*ch))
        {
            return false;
        }

        let Some(&current) = para.char_offsets.get(idx) else {
            return false;
        };
        let next = para
            .char_offsets
            .get(idx.saturating_add(1))
            .copied()
            .unwrap_or_else(|| para.char_count.saturating_sub(1));

        next.saturating_sub(current) >= 8
    }

    fn find_auto_number_placeholder_char(
        para: &Paragraph,
        text_chars: &[char],
        search_from: usize,
    ) -> Option<usize> {
        let preferred = text_chars
            .iter()
            .enumerate()
            .skip(search_from)
            .find(|(idx, _)| Self::is_auto_number_placeholder_at(para, text_chars, *idx))
            .map(|(idx, _)| idx);

        preferred.or_else(|| {
            if !para.char_offsets.is_empty() {
                return text_chars
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(idx, ch)| {
                        *idx >= search_from && Self::is_auto_number_placeholder_char(**ch)
                    })
                    .map(|(idx, _)| idx);
            }
            text_chars
                .iter()
                .enumerate()
                .skip(search_from)
                .find(|(_, ch)| Self::is_auto_number_placeholder_char(**ch))
                .map(|(idx, _)| idx)
        })
    }

    fn replace_composed_char(
        comp: &mut ComposedParagraph,
        abs_pos: usize,
        replacement: &str,
    ) -> bool {
        for line in &mut comp.lines {
            let mut run_start = line.char_start;
            for run in &mut line.runs {
                let run_len = run.text.chars().count();
                let run_end = run_start + run_len;
                if abs_pos >= run_start && abs_pos < run_end {
                    let rel_pos = abs_pos - run_start;
                    let mut chars = run.text.chars();
                    let before: String = chars.by_ref().take(rel_pos).collect();
                    let _ = chars.next();
                    let after: String = chars.collect();
                    run.text = format!("{before}{replacement}{after}");
                    run.display_text = None;
                    return true;
                }
                run_start = run_end;
            }
        }
        false
    }

    /// 페이지 배경 노드를 생성하여 tree에 추가한다.
    fn build_page_background(
        &self,
        tree: &mut PageRenderTree,
        layout: &PageLayoutInfo,
        page_border_fill: Option<&PageBorderFill>,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
    ) {
        let (page_bg_color, page_bg_gradient, page_bg_image) = if let Some(pbf) = page_border_fill {
            if pbf.border_fill_id > 0 {
                let bf_idx = (pbf.border_fill_id - 1) as usize;
                if let Some(bs) = styles.border_styles.get(bf_idx) {
                    let img = bs.image_fill.as_ref().and_then(|img_fill| {
                        find_bin_data(bin_data_content, img_fill.bin_data_id).map(|c| {
                            PageBackgroundImage {
                                data: c.data.clone(),
                                fill_mode: img_fill.fill_mode,
                                brightness: img_fill.brightness,
                                contrast: img_fill.contrast,
                                effect: img_fill.effect,
                            }
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

        let fill_area = page_border_fill
            .map(|pbf| (pbf.attr >> 3) & 0x03)
            .unwrap_or(0);
        let bg_bbox = match fill_area {
            1 => BoundingBox::new(
                layout.body_area.x,
                layout.body_area.y,
                layout.body_area.width,
                layout.body_area.height,
            ),
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
    /// 쪽 번호 배치 보정용 — 쪽 번호 baseline 의 y 좌표 (px).
    ///
    /// **body 기준 테두리일 때만** Some 을 반환한다. body 기준 테두리는
    /// 본문을 감싸므로 한컴은 쪽 번호를 본문(테두리) 아래 꼬리말 영역에 둔다.
    /// 한컴 정답지(sample16) 실측: 쪽 번호는 꼬리말 영역(footer_area)
    /// *세로 중앙* 에 담겨 출력된다 (테두리 아래로 흘러나가지 않음).
    /// paper 기준 테두리는 종이 전체를 감싸 쪽 번호가 테두리 *안쪽* 에 오며
    /// (aift.hwp Task #634), 이 경우 보정하지 않고 None.
    fn footer_page_number_y(
        &self,
        layout: &PageLayoutInfo,
        footer_area: &LayoutRect,
        font_size: f64,
    ) -> f64 {
        // [Task #1728] 자동 쪽번호 세로 위치: HWP 실측상 glyph 은 body_bottom(footer_area.y) 에서
        // margin_footer/2 + ~10px 아래에 온다(gc/ktx/aift 3문서 1~2px 정합). 종전 공식은
        // footer_area.height(= margin_bottom)/2 를 써서, margin_footer ≠ margin_bottom 인 문서
        // (margin_footer=0 인 giant cell, margin_footer≠margin_bottom 인 KTX)를 7~18px 낮게 놓았다.
        // margin_footer = page_height - footer_area.bottom.
        let center_y = if footer_area.height > 0.5 {
            let margin_footer =
                (layout.page_height - (footer_area.y + footer_area.height)).max(0.0);
            footer_area.y + margin_footer / 2.0
        } else {
            (footer_area.y + layout.page_height) / 2.0
        };
        center_y + font_size / 3.0
    }

    fn page_number_baseline_y(
        &self,
        layout: &PageLayoutInfo,
        page_border_fill: Option<&PageBorderFill>,
        font_size: f64,
    ) -> Option<f64> {
        let pbf = page_border_fill.filter(|p| p.border_fill_id > 0)?;
        let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
        if paper_based {
            return None;
        }
        // 꼬리말 영역 세로 중앙 baseline (기존 footer 중앙 공식과 동일).
        Some(self.footer_page_number_y(layout, &layout.footer_area, font_size))
    }

    fn build_page_borders(
        &self,
        tree: &mut PageRenderTree,
        layout: &PageLayoutInfo,
        page_border_fill: Option<&PageBorderFill>,
        styles: &ResolvedStyleSet,
    ) {
        if let Some(pbf) = page_border_fill.filter(|p| p.border_fill_id > 0) {
            let bf_idx = (pbf.border_fill_id - 1) as usize;
            if let Some(bs) = styles.border_styles.get(bf_idx) {
                // 외곽선 위치 기준: PageBorderFill.basis (PaperBased/BodyBased).
                // 회귀 history:
                //   - task877: paper_based = (attr & 0x01) != 0 — sample16 정합, 시험지 회귀
                //   - #920: paper_based = (attr & 0x01) == 0 — 시험지 정합, sample16 회귀
                //   - #952: paper_based = true 전역 — 당시 모든 sample 정합 판정
                //   - #987: bfid 정정 + attr 존중 — 변환본 logo overlap 회귀 (#1006)
                // 정답: PageBorderFill.basis 를 직접 따른다.
                // HWP3 원본은 쪽 기준(BodyBased), HWP5/HWPX는 저장된 UI 기준에 따라
                // PaperBased/BodyBased를 분리한다.
                // 또한 머리말 conditional clip 제거 (그림 이동 시 외곽선 shrink 회귀 해소),
                // 꼬리말 clip 은 유지 (페이지 번호 외곽선 안쪽 회귀 해소 — PR #1011).
                // [Task #1029] PR #1003 cherry-pick `--theirs` 충돌 해소로 본 로직이
                // PR #987 attr 비트 해석으로 revert 되어 HWP3 native (attr=0) 만
                // body-edge 로 좁아진 시각 회귀 발생 — 본 task 에서 PR #1011 상태 복원.
                let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
                if std::env::var("RHWP_DEBUG_PAGE_BORDER").is_ok() {
                    eprintln!(
                        "PAGE_BORDER: attr=0x{:08x} bit0={} bit1={} bit2={} paper_based={} bfid={} spacing(L={},R={},T={},B={})",
                        pbf.attr, pbf.attr & 0x01, (pbf.attr >> 1) & 0x01, (pbf.attr >> 2) & 0x01,
                        paper_based, pbf.border_fill_id,
                        pbf.spacing_left, pbf.spacing_right, pbf.spacing_top, pbf.spacing_bottom,
                    );
                }
                let borders = &bs.borders;
                let (base_x, base_y, base_w, base_h) = if paper_based {
                    (0.0, 0.0, layout.page_width, layout.page_height)
                } else {
                    (
                        layout.body_area.x,
                        layout.body_area.y,
                        layout.body_area.width,
                        layout.body_area.height,
                    )
                };

                let sp_l = hwpunit_to_px(pbf.spacing_left as i32, self.dpi);
                let sp_r = hwpunit_to_px(pbf.spacing_right as i32, self.dpi);
                let sp_t = hwpunit_to_px(pbf.spacing_top as i32, self.dpi);
                let sp_b = hwpunit_to_px(pbf.spacing_bottom as i32, self.dpi);
                let (out_l, out_r, out_t, out_b) = if paper_based {
                    (0.0, 0.0, 0.0, 0.0)
                } else {
                    (
                        body_page_border_outset(&borders[0]),
                        body_page_border_outset(&borders[1]),
                        body_page_border_outset(&borders[2]),
                        0.0,
                    )
                };
                // 종이 기준: 종이 가장자리에서 안쪽(+)으로 spacing
                // 쪽 기준: 본문 영역에서 바깥쪽(-)으로 spacing + 선 묶음 폭만큼 확장
                // 단 하단은 footer/쪽번호 영역과 맞닿으므로 한컴처럼 spacing까지만
                // 반영한다. 상단/좌우 outset은 Stage 29 로고 정합을 유지한다.
                let (bx, by, bw, bh) = if paper_based {
                    (
                        base_x + sp_l,
                        base_y + sp_t,
                        base_w - sp_l - sp_r,
                        base_h - sp_t - sp_b,
                    )
                } else {
                    (
                        base_x - sp_l - out_l,
                        base_y - sp_t - out_t,
                        base_w + sp_l + sp_r + out_l + out_r,
                        base_h + sp_t + sp_b + out_t + out_b,
                    )
                };

                let top_nodes = create_border_line_nodes(tree, &borders[2], bx, by, bx + bw, by);
                for n in top_nodes {
                    tree.root.children.push(n);
                }
                let bottom_nodes =
                    create_border_line_nodes(tree, &borders[3], bx, by + bh, bx + bw, by + bh);
                for n in bottom_nodes {
                    tree.root.children.push(n);
                }
                let left_nodes = create_border_line_nodes(tree, &borders[0], bx, by, bx, by + bh);
                for n in left_nodes {
                    tree.root.children.push(n);
                }
                let right_nodes =
                    create_border_line_nodes(tree, &borders[1], bx + bw, by, bx + bw, by + bh);
                for n in right_nodes {
                    tree.root.children.push(n);
                }
            }
        }
    }

    fn header_table_area_from_page_border(
        &self,
        layout: &PageLayoutInfo,
        page_border_fill: Option<&PageBorderFill>,
    ) -> Option<LayoutRect> {
        let pbf = page_border_fill.filter(|p| p.border_fill_id > 0)?;
        if !matches!(pbf.basis, PageBorderBasis::PaperBased) {
            return None;
        }

        let left = hwpunit_to_px(pbf.spacing_left as i32, self.dpi);
        let right = layout.page_width - hwpunit_to_px(pbf.spacing_right as i32, self.dpi);
        if right <= left {
            return None;
        }

        let mut area = layout.header_area;
        area.x = left;
        area.width = right - left;
        Some(area)
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
        self.build_master_page(
            tree,
            active_master_page,
            layout,
            composed,
            styles,
            bin_data_content,
            section_index,
            page_number,
        );
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
            if !mp.paragraphs.is_empty() {
                let previous_page_number = self.current_page_number.get();
                // HWPX masterPage@pageNumber/hasNumRef is not a reliable signal to suppress
                // inline autoNum(PAGE) controls. exam_social.hwpx uses pageNumber=0 and
                // hasNumRef=0 even though the bottom master-page table contains the visible
                // page number. Header duplicate page numbers are handled at the AutoNumber
                // placeholder level instead of disabling master-page numbering wholesale.
                self.current_page_number.set(page_number);
                let mp_id = tree.next_id();
                let paper_area = LayoutRect {
                    x: 0.0,
                    y: 0.0,
                    width: layout.page_width,
                    height: layout.page_height,
                };
                let body_area = &layout.body_area;
                // HWP/HWPX에서 `PAGE` 기준은 물리 용지(PAPER)가 아니라 본문 영역 기준이다.
                // 바탕쪽은 본문보다 먼저 렌더링되므로 표 위치 계산용 현재 페이지 context를
                // 여기서 명시적으로 채워야 `vertRelTo=PAGE`, `horzRelTo=PAGE`가 올바르게 동작한다.
                self.current_paper_width.set(layout.page_width);
                self.current_body_area.set((
                    body_area.x,
                    body_area.y,
                    body_area.width,
                    body_area.height,
                ));
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
                            let layer = self.render_layer_from_master_control(
                                ctrl,
                                pi,
                                ci,
                                &paper_area,
                                body_area,
                            );
                            let mut temp_parent = layer.map(|_| {
                                RenderNode::new(
                                    0,
                                    RenderNodeType::MasterPage,
                                    layout_rect_to_bbox(&paper_area),
                                )
                            });
                            let target_node = temp_parent.as_mut().unwrap_or(&mut mp_node);
                            match ctrl {
                                Control::Shape(_) | Control::Equation(_) => {
                                    self.layout_shape(
                                        tree,
                                        target_node,
                                        &mp.paragraphs,
                                        pi,
                                        ci,
                                        section_index,
                                        styles,
                                        &paper_area,
                                        body_area,
                                        &paper_area,
                                        paper_area.y,
                                        Alignment::Left,
                                        bin_data_content,
                                        &std::collections::HashMap::new(),
                                        false,
                                    );
                                }
                                Control::Picture(pic) => {
                                    let (pic_w, pic_h) = self.resolve_object_size(
                                        &pic.common,
                                        &paper_area,
                                        body_area,
                                        &paper_area,
                                    );
                                    let (pic_x, pic_y) = self.compute_object_position(
                                        &pic.common,
                                        pic_w,
                                        pic_h,
                                        &paper_area,
                                        &paper_area,
                                        body_area,
                                        &paper_area,
                                        paper_area.y,
                                        Alignment::Left,
                                    );
                                    let pic_area = super::layout::LayoutRect {
                                        x: pic_x,
                                        y: pic_y,
                                        width: pic_w,
                                        height: pic_h,
                                    };
                                    let mut positioned = (**pic).clone();
                                    positioned.common.horizontal_offset = 0;
                                    positioned.common.vertical_offset = 0;
                                    positioned.common.horz_rel_to = HorzRelTo::Para;
                                    positioned.common.vert_rel_to = VertRelTo::Para;
                                    positioned.common.horz_align = HorzAlign::Left;
                                    positioned.common.vert_align = VertAlign::Top;
                                    self.layout_picture(
                                        tree,
                                        target_node,
                                        &positioned,
                                        &pic_area,
                                        bin_data_content,
                                        Alignment::Left,
                                        Some(section_index),
                                        None,
                                        None,
                                        None, // [Task #1151 v4] cell_ctx: 바탕쪽
                                    );
                                }
                                Control::Table(t) => {
                                    let alignment = styles
                                        .para_styles
                                        .get(para.para_shape_id as usize)
                                        .map(|s| s.alignment)
                                        .unwrap_or(Alignment::Left);
                                    // 바탕쪽 표: PAPER 기준은 paper_area, PAGE 기준은 위에서 설정한
                                    // current_body_area를 통해 본문 영역으로 계산된다.
                                    self.layout_table(
                                        tree,
                                        target_node,
                                        t,
                                        section_index,
                                        styles,
                                        0,
                                        &paper_area,
                                        0.0,
                                        bin_data_content,
                                        None,
                                        0,
                                        Some((pi, ci)),
                                        alignment,
                                        None,
                                        0.0,
                                        0.0,
                                        None,
                                        None,
                                        None,
                                        false,
                                        false,
                                    );
                                }
                                _ => {}
                            }
                            if let (Some(layer), Some(temp_parent)) = (layer, temp_parent.as_mut())
                            {
                                Self::push_layered_paper_children(
                                    &mut mp_node.children,
                                    temp_parent,
                                    layer,
                                );
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
                            let vpos_y =
                                paper_area.y + hwpunit_to_px(first_ls.vertical_pos, self.dpi);
                            if vpos_y > mp_y_offset {
                                mp_y_offset = vpos_y;
                            }
                        }
                        mp_y_offset = self.layout_paragraph(
                            tree,
                            &mut mp_node,
                            para,
                            Some(&comp),
                            styles,
                            &paper_area,
                            mp_y_offset,
                            0,
                            usize::MAX - pi,
                            None,
                            None,
                            None, // 바탕쪽 컨텍스트 — wrap zone 무관
                        );
                    } else {
                        // 빈 문단: LINE_SEG vpos로 y 위치 갱신
                        if let Some(first_ls) = para.line_segs.first() {
                            let vpos_y =
                                paper_area.y + hwpunit_to_px(first_ls.vertical_pos, self.dpi);
                            let lh = hwpunit_to_px(first_ls.line_height, self.dpi);
                            let ls = hwpunit_to_px(first_ls.line_spacing, self.dpi);
                            mp_y_offset = (vpos_y + lh + ls).max(mp_y_offset);
                        }
                    }
                }
                // Hancom prepares master-page furniture through object order, not raw XML
                // order: text-wrap plane, zOrder, then stable source index.
                Self::sort_paper_render_nodes(&mut mp_node.children);
                tree.root.children.push(mp_node);
                self.current_page_number.set(previous_page_number);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_header_footer_picture(
        &self,
        tree: &mut PageRenderTree,
        area_node: &mut RenderNode,
        pic: &crate::model::image::Picture,
        area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        para_y: f64,
        bin_data_content: &[BinDataContent],
        outer_section_index: Option<usize>,
        inner_para_index: usize,
        inner_control_index: usize,
        outer_hf_ref: Option<crate::renderer::render_tree::HeaderFooterImageRef>,
    ) {
        let rotation = pic.shape_attr.rotation_angle.rem_euclid(360);
        let uses_rotated_frame = rotation != 0
            && pic.shape_attr.current_width > 0
            && pic.shape_attr.current_height > 0
            && pic.common.width > 0
            && pic.common.height > 0;
        let (pic_width_hu, pic_height_hu) = if uses_rotated_frame {
            (
                pic.shape_attr.current_width as i32,
                pic.shape_attr.current_height as i32,
            )
        } else {
            picture_display_size_hu(pic)
        };
        let frame_width = if uses_rotated_frame {
            hwpunit_to_px(pic.common.width as i32, self.dpi)
        } else {
            hwpunit_to_px(pic_width_hu, self.dpi)
        };
        let frame_height = if uses_rotated_frame {
            hwpunit_to_px(pic.common.height as i32, self.dpi)
        } else {
            hwpunit_to_px(pic_height_hu, self.dpi)
        };

        let (frame_x, frame_y) = self.compute_object_position(
            &pic.common,
            frame_width,
            frame_height,
            area,
            area,
            body_area,
            paper_area,
            para_y,
            Alignment::Left,
        );
        let mut positioned = pic.clone();
        positioned.common.horizontal_offset = 0;
        positioned.common.vertical_offset = 0;
        positioned.common.horz_rel_to = HorzRelTo::Para;
        positioned.common.vert_rel_to = VertRelTo::Para;
        positioned.common.horz_align = HorzAlign::Left;
        positioned.common.vert_align = VertAlign::Top;
        let pic_container = LayoutRect {
            x: frame_x,
            y: frame_y,
            width: frame_width,
            height: frame_height,
        };
        self.layout_picture_full(
            tree,
            area_node,
            &positioned,
            &pic_container,
            bin_data_content,
            Alignment::Left,
            outer_section_index,
            Some(inner_para_index),
            Some(inner_control_index),
            outer_hf_ref,
            None,
        );
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
        page_border_fill: Option<&PageBorderFill>,
    ) {
        self.current_page_number.set(page_content.page_number);
        let header_id = tree.next_id();
        let mut header_node = RenderNode::new(
            header_id,
            RenderNodeType::Header,
            layout_rect_to_bbox(&layout.header_area),
        );
        // 감추기 플래그가 설정된 페이지는 머리말 내용을 렌더링하지 않음
        let hidden = self
            .hidden_header_footer
            .borrow()
            .contains(&(page_content.page_index, true));
        if !hidden {
            if let Some(hf_ref) = &page_content.active_header {
                if let Some(para) = paragraphs.get(hf_ref.para_index) {
                    if let Some(ctrl) = para.controls.get(hf_ref.control_index) {
                        if let Control::Header(header) = ctrl {
                            let header_table_area =
                                self.header_table_area_from_page_border(layout, page_border_fill);
                            // [Task #825] 머리말 그림 hit-test marker.
                            let outer_ref = crate::renderer::render_tree::HeaderFooterImageRef {
                                outer_para_index: hf_ref.para_index,
                                outer_control_index: hf_ref.control_index,
                                kind: crate::renderer::render_tree::HeaderFooterKind::Header,
                            };
                            self.layout_header_footer_paragraphs(
                                tree,
                                &mut header_node,
                                &header.paragraphs,
                                composed,
                                styles,
                                &layout.header_area,
                                &layout.body_area,
                                &LayoutRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: layout.page_width,
                                    height: layout.page_height,
                                },
                                header_table_area.as_ref(),
                                page_content.page_index,
                                page_content.page_number,
                                bin_data_content,
                                Some(hf_ref.source_section_index),
                                Some(outer_ref),
                                true,
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
        // [Task #825] 머리말 안 모든 ImageNode 에 header_footer_ref 부여 + 인덱스 정규화.
        // TAC 인라인 picture 는 layout_paragraph 경로에서 para_index = usize::MAX - i 로
        // 인코딩되어 ImageNode 에 저장되므로, 본 후처리로 inner para idx 회복.
        if let Some(hf_ref) = &page_content.active_header {
            let outer_ref = crate::renderer::render_tree::HeaderFooterImageRef {
                outer_para_index: hf_ref.para_index,
                outer_control_index: hf_ref.control_index,
                kind: crate::renderer::render_tree::HeaderFooterKind::Header,
            };
            Self::propagate_header_footer_ref(
                &mut header_node,
                &outer_ref,
                hf_ref.source_section_index,
            );
        }
        tree.root.children.push(header_node);
    }

    /// [Task #825] header/footer 노드 안 모든 ImageNode 에 header_footer_ref 부여
    /// + para_index 정규화 (usize::MAX - i → i).
    fn propagate_header_footer_ref(
        node: &mut RenderNode,
        outer_ref: &crate::renderer::render_tree::HeaderFooterImageRef,
        section_index: usize,
    ) {
        if let RenderNodeType::Image(img) = &mut node.node_type {
            // TAC 경로 인코딩 회복: para_index 가 MAX 근처면 usize::MAX - i 로 저장된 것.
            if let Some(pi) = img.para_index {
                if pi >= usize::MAX - 1024 {
                    img.para_index = Some(usize::MAX - pi);
                }
            }
            img.section_index = Some(section_index);
            img.header_footer_ref = Some(outer_ref.clone());
        }
        for child in node.children.iter_mut() {
            Self::propagate_header_footer_ref(child, outer_ref, section_index);
        }
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
        let hidden = self
            .hidden_header_footer
            .borrow()
            .contains(&(page_content.page_index, false));
        if !hidden {
            if let Some(hf_ref) = &page_content.active_footer {
                if let Some(para) = paragraphs.get(hf_ref.para_index) {
                    if let Some(ctrl) = para.controls.get(hf_ref.control_index) {
                        if let Control::Footer(footer) = ctrl {
                            // [Task #825] 꼬리말 그림 hit-test marker.
                            let outer_ref = crate::renderer::render_tree::HeaderFooterImageRef {
                                outer_para_index: hf_ref.para_index,
                                outer_control_index: hf_ref.control_index,
                                kind: crate::renderer::render_tree::HeaderFooterKind::Footer,
                            };
                            self.layout_header_footer_paragraphs(
                                tree,
                                &mut footer_node,
                                &footer.paragraphs,
                                composed,
                                styles,
                                &layout.footer_area,
                                &layout.body_area,
                                &LayoutRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: layout.page_width,
                                    height: layout.page_height,
                                },
                                None,
                                page_content.page_index,
                                page_content.page_number,
                                bin_data_content,
                                Some(hf_ref.source_section_index),
                                Some(outer_ref),
                                false,
                            );
                        }
                    }
                }
            }
        }
        Self::expand_bbox_to_children(&mut footer_node);
        Self::disable_cell_clip_recursive(&mut footer_node);
        // [Task #825] 꼬리말 안 모든 ImageNode 에 header_footer_ref 부여 + 인덱스 정규화.
        if let Some(hf_ref) = &page_content.active_footer {
            let outer_ref = crate::renderer::render_tree::HeaderFooterImageRef {
                outer_para_index: hf_ref.para_index,
                outer_control_index: hf_ref.control_index,
                kind: crate::renderer::render_tree::HeaderFooterKind::Footer,
            };
            Self::propagate_header_footer_ref(
                &mut footer_node,
                &outer_ref,
                hf_ref.source_section_index,
            );
        }
        footer_node
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
                &page_content.footnotes,
                paragraphs,
                footnote_shape,
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
        page_border_fill: Option<&PageBorderFill>,
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
                page_content.page_number,
                pnp.format,
                pnp.prefix_char,
                pnp.suffix_char,
                pnp.dash_char,
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
                7 | 8 => {
                    if is_odd_page {
                        target_area.x + target_area.width - text_width
                    } else {
                        target_area.x
                    }
                }
                // 안쪽: 홀수쪽→왼쪽, 짝수쪽→오른쪽
                9 | 10 => {
                    if is_odd_page {
                        target_area.x
                    } else {
                        target_area.x + target_area.width - text_width
                    }
                }
                _ => target_area.x + (target_area.width - text_width) / 2.0,
            };

            // 기본: target_area(머리말/꼬리말) 세로 중앙.
            // 단 꼬리말 위치 + body 기준 쪽 테두리가 *이 페이지에 실제로
            // 그려질 때* 한컴은 쪽 번호를 꼬리말 영역 하단(= 용지 하단에서
            // margin_footer 만큼 위)에 배치한다 (Task #987 Stage 5).
            // 쪽 테두리 없거나 paper 기준이거나 hide_border 인 페이지는
            // 기존 중앙 로직 유지 → 회귀 격리.
            let border_drawn = !page_content
                .page_hide
                .as_ref()
                .map(|ph| ph.hide_border)
                .unwrap_or(false);
            let is_footer = !matches!(pnp.position, 1..=3 | 7 | 9);
            let footer_center = if is_footer {
                self.footer_page_number_y(layout, target_area, font_size)
            } else {
                target_area.y + target_area.height / 2.0 + font_size / 3.0
            };
            // body 기준 테두리 + 테두리 실제 그려질 때만 footer_area 중앙으로
            // 보정 (target_area 가 footer_area 와 다를 수 있는 경우 정합).
            // 그 외(paper 기준/테두리 없음/hide_border)는 기존 footer_center.
            let y = if is_footer {
                self.page_number_baseline_y(layout, page_border_fill, font_size)
                    .filter(|_| border_drawn)
                    .unwrap_or(footer_center)
            } else {
                footer_center
            };

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
        // [Task #866 v2 Stage 3] zone 별 단 구분선 렌더용. 페이지 내 다단 zone 의 ColumnDef
        // (예: pi=2 의 2단/구분선 type=7) 를 반영하고, [Task #1333] 이후 단 구분선은
        // 이 zone emit 경로 하나에서 그린다.
        let mut prev_zone_layout_for_sep: Option<PageLayoutInfo> = None;
        let mut prev_zone_sep_y_start: f64 = 0.0;
        // [Task #853/#866] 직전 zone 의 "디자인 spacing"(1단 ColumnDef 의 `간격`, 다단은 0).
        // 한컴은 zone 전환 시 (이전 zone 디자인 spacing /2)+(새 zone /2) 만큼 세로 여백을
        // 둔다(shortcut.hwp 1쪽 헤더 띠 ColumnDef 간격=10mm → 제목↔헤더 5mm, 헤더↔본문 5mm).
        // pagination 측 process_multicolumn_break 의 동작과 동일 시멘틱.
        let design_spacing_of = |para_idx: usize| -> f64 {
            paragraphs
                .get(para_idx)
                .and_then(|p| {
                    p.controls.iter().find_map(|c| match c {
                        Control::ColumnDef(cd) if cd.column_count.max(1) <= 1 => {
                            Some(hwpunit_to_px(cd.spacing as i32, self.dpi))
                        }
                        Control::ColumnDef(_) => Some(0.0),
                        _ => None,
                    })
                })
                .unwrap_or(0.0)
        };
        let mut prev_zone_design_px: f64 = 0.0;
        let mut prev_zone_was_solo: bool = false;
        // [Task #866 v3 Stage 1] 직전 zone 이 헤더 띠(TAC wrap=TopAndBottom 표만 보유) 였으면
        // solo_zone_pad 의 leaving 분기를 제외 (typeset.rs::leaving_is_header_band 와 동일).
        let mut prev_zone_was_header_band: bool = false;

        // 다단 레이아웃: body_area 전체에 걸치는 TopAndBottom 개체의 예약 높이
        // (한 단에만 할당되더라도 모든 단에 적용)
        let body_wide_reserved: Vec<(usize, f64)> = if page_content.column_contents.len() > 1 {
            self.calculate_body_wide_shape_reserved(
                paragraphs,
                &page_content.column_contents,
                &layout.body_area,
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
                // 직전 zone 의 단 구분선 emit (있다면).
                if let Some(pz) = prev_zone_layout_for_sep.take() {
                    self.emit_zone_column_separators(
                        tree,
                        body_node,
                        &pz,
                        prev_zone_sep_y_start,
                        prev_zone_y_end,
                    );
                }
                // 새 zone 의 디자인 spacing = 이 zone 첫 paragraph 의 ColumnDef `간격`(1단 한정).
                let new_zone_first_para = col_content.items.first().and_then(|it| match it {
                    PageItem::FullParagraph { para_index }
                    | PageItem::PartialParagraph { para_index, .. }
                    | PageItem::Table { para_index, .. }
                    | PageItem::PartialTable { para_index, .. }
                    | PageItem::Shape { para_index, .. } => Some(*para_index),
                    PageItem::EndnoteSeparator { .. } => None,
                });
                let new_zone_design = new_zone_first_para
                    .map(|pi| design_spacing_of(pi))
                    .unwrap_or(0.0);
                // [Task #866 v2 Stage 2/4] pagination 측 solo_zone_pad 와 동일:
                //   (1) 1단/간격=0 zone 진입·이탈, (2) [단나누기] 로 시작한 새 zone → +20px.
                // [Task #874 Case 5 v4] solo_zero 인정 범위를 spacing ≤ 1mm (283 HU) 까지
                // 확장. shortcut.hwp 의 `<스타일에서>` (pi=148) 등 일부 `<...>` 소제목 zone 은
                // ColumnDef 가 1단/spacing=1mm 으로 정의되어 있어 strict == 0 검사를 통과 못
                // 했고, 결과적으로 solo_zone_pad +16 이 누락되어 페이지 4 본문 (스타일 적용
                // 등) 줄간격이 1.92 px 까지 좁아짐. typeset.rs::tac_band_extra 가 < 4.0 px 까지
                // 인정하는 것과 동일 시멘틱.
                let new_zone_is_solo_zero = new_zone_first_para
                    .and_then(|pi| {
                        paragraphs.get(pi).map(|p| {
                            p.controls.iter().any(|c| {
                                matches!(c,
                        Control::ColumnDef(cd) if cd.column_count.max(1) <= 1 && cd.spacing <= 283)
                            })
                        })
                    })
                    .unwrap_or(false);
                // [Task #874 Case 5 v4] solo_zero leaving 인정 범위도 1mm (3.8 px) 까지 확장.
                let prev_zone_is_solo_zero = prev_zone_design_px < 4.0 && prev_zone_was_solo;
                let column_break_new_band = new_zone_first_para
                    .and_then(|pi| paragraphs.get(pi))
                    .map(|p| p.column_type == crate::model::paragraph::ColumnBreakType::Column)
                    .unwrap_or(false);
                let solo_zone_pad = if new_zone_is_solo_zero
                    || (prev_zone_is_solo_zero && !prev_zone_was_header_band)
                    || column_break_new_band
                {
                    hwpunit_to_px(1200, self.dpi)
                } else {
                    0.0
                };
                if col_content.zone_y_offset > 0.0 {
                    current_zone_start_y = prev_zone_y_end
                        + prev_zone_design_px / 2.0
                        + new_zone_design / 2.0
                        + solo_zone_pad;
                } else {
                    current_zone_start_y = 0.0;
                }
                prev_zone_design_px = new_zone_design;
                prev_zone_was_solo = new_zone_is_solo_zero
                    || (new_zone_design > 0.5
                        && new_zone_first_para
                            .and_then(|pi| paragraphs.get(pi))
                            .map(|p| {
                                p.controls.iter().any(|c| {
                                    matches!(c,
                            Control::ColumnDef(cd) if cd.column_count.max(1) <= 1)
                                })
                            })
                            .unwrap_or(false));
                last_zone_y_offset = col_content.zone_y_offset;
                // 본 zone 이 다단 + 구분선 보유 시 종료 시점에 emit 하기 위해 기록.
                // [Task #1333] zone emit(emit_zone_column_separators)이 단 구분선의 단일
                // 경로다. zone_layout=None(초기 단정의·연속 페이지)은 unwrap_or(layout)로
                // page layout 을 따르며, 콘텐츠가 채워진 높이까지만 구분선을 그린다(한컴 정합).
                // 꽉 찬 페이지는 콘텐츠≈body 라 전체 높이로, 부분 페이지(섹션 끝 등)는 콘텐츠
                // 하단까지만 그려진다. body 초과분은 emit_zone_column_separators 가 캡한다.
                if zone_layout.column_areas.len() >= 2 && zone_layout.separator_type > 0 {
                    prev_zone_layout_for_sep = Some(zone_layout.clone());
                    prev_zone_sep_y_start = current_zone_start_y.max(zone_layout.body_area.y);
                } else {
                    prev_zone_layout_for_sep = None;
                }
            }

            let col_area = if current_zone_start_y > col_area_base.y {
                LayoutRect {
                    x: col_area_base.x,
                    y: current_zone_start_y,
                    width: col_area_base.width,
                    height: (col_area_base.y + col_area_base.height - current_zone_start_y)
                        .max(0.0),
                }
            } else {
                *col_area_base
            };

            let (col_node, y_offset) = self.build_single_column(
                tree,
                paper_images,
                col_content,
                page_content,
                paragraphs,
                composed,
                styles,
                bin_data_content,
                measured_tables,
                layout,
                zone_layout,
                &col_area,
                outline_numbering_id,
                wrap_around_paras,
                &body_wide_reserved,
            );

            // [Task #874 Case 5] solo-single zone (1단 ColumnDef + 1 paragraph) leaving 시
            // 마지막 paragraph 의 trailing line_spacing 을 prev_zone_y_end 에 포함하지
            // 않는다. zone 간 gap 은 design_spacing/2 + solo_zone_pad 가 담당하므로
            // trailing_ls 까지 더하면 이중 가산. 한컴 PDF 측정 (shortcut.hwp 1쪽):
            // 본문 첫 줄 top 195.3 px (Hancom) vs 210.7 px (rhwp pre) = +15.4 px
            // (≈11.5pt) 넓다. 제목 paragraph 의 trailing_ls 16 px 이 y_offset 에 포함되어
            // 다음 zone(헤더 띠 + 본문) 을 일괄 하향. zone 내부 paragraph 의 trailing_ls 는
            // 영향 없음 (y_offset 누적 자체는 유지).
            //
            // 적용 조건 (모두 만족):
            // - prev_zone_was_solo: 현재 zone 이 solo (1단 ColumnDef) — 다단 본문 zone
            //   leaving 에는 미적용 (페이지 4 "개체 모양 복사" → `<스타일에서>` 전환의
            //   본문 paragraph 줄간격이 좁아지는 사용자 피드백).
            // - last paragraph 가 TAC 헤더 띠/ `<...>` solo 가 아닐 것 — pi=81/pi=127
            //   형식의 ls=480/600 HU 는 한컴 의도 간격이므로 보존.
            let last_para_idx = col_content.items.last().and_then(|it| match it {
                PageItem::FullParagraph { para_index }
                | PageItem::PartialParagraph { para_index, .. }
                | PageItem::Table { para_index, .. } => Some(*para_index),
                _ => None,
            });
            let last_para = last_para_idx.and_then(|pi| paragraphs.get(pi));
            let last_is_tac_band = last_para
                .map(|p| p.controls.iter().any(|c| matches!(c,
                    Control::Table(t) if t.common.treat_as_char
                        && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom))))
                .unwrap_or(false);
            let last_is_solo_text = last_para
                .map(|p| {
                    p.controls.iter().any(|c| {
                        matches!(c,
                    Control::ColumnDef(cd) if cd.column_count.max(1) <= 1 && cd.spacing == 0)
                    }) && p.text.trim_start().starts_with('<')
                })
                .unwrap_or(false);
            let apply_trailing_ls_subtract =
                prev_zone_was_solo && !last_is_tac_band && !last_is_solo_text;
            let last_para_trailing_ls = if apply_trailing_ls_subtract {
                last_para
                    .and_then(|p| p.line_segs.last())
                    .map(|ls| hwpunit_to_px(ls.line_spacing, self.dpi))
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let y_offset_no_trailing = (y_offset - last_para_trailing_ls).max(0.0);

            if y_offset_no_trailing > prev_zone_y_end {
                prev_zone_y_end = y_offset_no_trailing;
            }
            // [Task #866] 헤더 띠 zone (wrap=위아래 인 글자처럼-취급 표 보유 + 1단 ColumnDef
            // 간격=0) 의 leaving 시 zone 아래 band 가산 + header_band flag 갱신.
            //
            // 이력:
            // - 초기 (#866): `prev_zone_y_end += band` 전체 가산 (≈31px) — 페이지 6 (Table-only
            //   pi=210) 형식 정합, 그러나 페이지 2·3 (PartialParagraph + Table pi=36/81) 형식
            //   에서는 본문 첫 줄 +30pt 넓다 (사용자 피드백).
            // - #874 Case 1: 전체 제거 — 페이지 2·3 -8~-16pt 좁다 over-correction.
            // - #874 Case 1 v2 (현재): **items 수로 분기**.
            //     items==1 (Table only, pi=210 형식, 페이지 6 헤더 띠 zone): y_offset 이
            //       표 높이만 advance 하므로 표 본체 + outer_margin 만큼 추가 가산 필요.
            //     items>1 (PartialParagraph + Table, pi=36/81 형식, 페이지 2·3): y_offset 이
            //       text 라인 + 표 라인 까지 advance 한 상태 — band 추가 가산은 이중 가산.
            prev_zone_was_header_band = false;
            if let Some(last_para_idx) = col_content.items.last().and_then(|it| match it {
                PageItem::Table { para_index, .. } => Some(*para_index),
                _ => None,
            }) {
                if let Some(p) = paragraphs.get(last_para_idx) {
                    let cd_gap_zero = if p
                        .controls
                        .iter()
                        .any(|c| matches!(c, Control::ColumnDef(_)))
                    {
                        p.controls.iter().any(|c| matches!(c,
                            Control::ColumnDef(cd) if cd.column_count.max(1) <= 1 && cd.spacing == 0))
                    } else {
                        (0..last_para_idx)
                            .rev()
                            .find_map(|i| {
                                paragraphs.get(i).and_then(|pp| {
                                    pp.controls.iter().find_map(|c| match c {
                                        Control::ColumnDef(cd) => {
                                            Some(cd.column_count.max(1) <= 1 && cd.spacing <= 283)
                                        }
                                        _ => None,
                                    })
                                })
                            })
                            .unwrap_or(false)
                    };
                    if cd_gap_zero {
                        if let Some(band) = p.controls.iter().find_map(|c| match c {
                            Control::Table(t)
                                if t.common.treat_as_char
                                    && matches!(
                                        t.common.text_wrap,
                                        crate::model::shape::TextWrap::TopAndBottom
                                    ) =>
                            {
                                Some(
                                    hwpunit_to_px(t.common.height as i32, self.dpi)
                                        + hwpunit_to_px(t.outer_margin_top as i32, self.dpi)
                                        + hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi),
                                )
                            }
                            _ => None,
                        }) {
                            // Table-only zone (페이지 6 pi=210 형식): 전체 band 가산.
                            //   y_offset 이 표 높이만 advance → outer_margin 까지 추가 필요.
                            // PartialParagraph + Table zone (페이지 2·3 pi=36/81 형식):
                            //   y_offset 이 text 라인 + 표 라인 까지 advance — 일부 중복.
                            //   half (band/2) 가산으로 측정 정합 (페이지 2 +3.8, 페이지 3 -5.6).
                            if col_content.items.len() == 1 {
                                prev_zone_y_end += band;
                            } else {
                                prev_zone_y_end += band / 2.0;
                            }
                            prev_zone_was_header_band = true;
                        }
                    }
                }
            }
            body_node.children.push(col_node);
        }

        // 마지막 zone 의 단 구분선 emit.
        if let Some(pz) = prev_zone_layout_for_sep.take() {
            self.emit_zone_column_separators(
                tree,
                body_node,
                &pz,
                prev_zone_sep_y_start,
                prev_zone_y_end,
            );
        }
    }

    /// [Task #866 v2 Stage 3] 단일 zone 의 단 구분선을 emit.
    /// zone_layout 기준 + y 범위 인자로 단 구분선을 그린다.
    fn emit_zone_column_separators(
        &self,
        tree: &mut PageRenderTree,
        body_node: &mut RenderNode,
        zone_layout: &PageLayoutInfo,
        y_start: f64,
        y_end: f64,
    ) {
        // [Task #1333 v2] 콘텐츠 높이(y_end)가 body 영역 하단을 넘으면 하단에서 자른다.
        // 꽉 찬 페이지에서 prev_zone_y_end 가 trailing 간격 등으로 body 를 초과해 구분선이
        // 페이지 밖까지 그려지던 결함(예: 대상문서 p22 105%) 정정. 부분 페이지(콘텐츠 < body
        // 하단)와 sub-page zone 은 영향 없음.
        let body_bottom = zone_layout.body_area.y + zone_layout.body_area.height;
        let y_end = y_end.min(body_bottom);
        if zone_layout.column_areas.len() < 2 || zone_layout.separator_type == 0 || y_end <= y_start
        {
            return;
        }
        let line_width = border_width_to_px(zone_layout.separator_width).max(0.5);
        let dash = match zone_layout.separator_type {
            2 => StrokeDash::Dash,
            3 => StrokeDash::Dot,
            4 => StrokeDash::DashDot,
            5 => StrokeDash::DashDotDot,
            6 => StrokeDash::Dash,
            7 => StrokeDash::Dot,
            _ => StrokeDash::Solid,
        };
        for i in 0..zone_layout.column_areas.len() - 1 {
            let left = &zone_layout.column_areas[i];
            let right = &zone_layout.column_areas[i + 1];
            let sep_x = (left.x + left.width + right.x) / 2.0;
            let sep_id = tree.next_id();
            let sep_node = RenderNode::new(
                sep_id,
                RenderNodeType::Line(LineNode::new(
                    sep_x,
                    y_start,
                    sep_x,
                    y_end,
                    LineStyle {
                        color: zone_layout.separator_color,
                        width: line_width,
                        dash,
                        ..Default::default()
                    },
                )),
                BoundingBox::new(
                    sep_x - line_width / 2.0,
                    y_start,
                    line_width,
                    y_end - y_start,
                ),
            );
            body_node.children.push(sep_node);
        }
    }

    /// 단일 단의 콘텐츠를 레이아웃한다.
    #[allow(clippy::too_many_arguments)]
    /// [Task #1363 v3 옵션 3] 미주 단의 전 items 를 scratch 로 **1회 순차 레이아웃**해 정확한
    /// 렌더 단 bottom(px, col_area 상대)을 반환한다. per-para 고립 측정 + HeightCursor 시뮬의
    /// 컨텍스트 의존·순차 상호작용(vpos forward-jump ↔ trailing) 발산을 회피한다 — 렌더
    /// 코드(`build_single_column`) 자체로 측정하므로 sim==render 가 구조적으로 보장된다.
    ///
    /// `items`/`paragraphs`/`composed` 는 호출부에서 단 items 만 추출해 **로컬 0-기반 재색인**해
    /// 전달한다. `col_area` 는 상대 프레임(`y=0`)으로 둔다. 표/그림 개체는 measured_tables/
    /// bin_data 없이 측정(미주 단은 텍스트/수식 지배 — 표 미주는 근사). numbering/overflow 등
    /// 상태는 매 호출 새 scratch 엔진이라 격리된다([[tech_endnote_overflow_nonmonotonic_gate]]).
    pub(crate) fn measure_endnote_column_bottom(
        &self,
        items: Vec<PageItem>,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        start_height: f64,
        section_index: usize,
        between_notes_hu: i32,
    ) -> f64 {
        self.endnote_between_notes_hu.set(between_notes_hu);
        // 로컬 paras 는 전부 미주 para(0-기반 재색인). `endnote_para_base=0` 으로 미주 vpos
        // 정규화 경로(`endnote_line_vpos_base`: para_index >= base)를 활성화한다 — 미설정 시
        // usize::MAX 라 정규화가 꺼져 para 의 절대 파일-vpos 가 그대로 새어 단독 측정이
        // 폭발한다(수식 para 35px→13721px).
        self.endnote_para_base.set(0);
        let layout_info = PageLayoutInfo {
            page_width: col_area.width,
            page_height: col_area.y + col_area.height,
            header_area: *col_area,
            body_area: *col_area,
            column_areas: vec![*col_area],
            footnote_area: *col_area,
            footer_area: *col_area,
            dpi: self.dpi,
            separator_type: 0,
            separator_width: 0,
            separator_color: 0,
            pagination_tolerance_px: 0.0,
        };
        let col_content = ColumnContent {
            column_index: 0,
            start_height,
            endnote_flow: true,
            items,
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
            wrap_anchors: std::collections::HashMap::new(),
        };
        let page_content = PageContent {
            page_index: 0,
            page_number: 0,
            section_index,
            layout: layout_info.clone(),
            column_contents: Vec::new(),
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
        };
        let mut tree = PageRenderTree::new(0, col_area.width, col_area.y + col_area.height);
        let mut paper_images: Vec<RenderNode> = Vec::new();
        let (_node, y_offset) = self.build_single_column(
            &mut tree,
            &mut paper_images,
            &col_content,
            &page_content,
            paragraphs,
            composed,
            styles,
            &[],
            &[],
            &layout_info,
            &layout_info,
            col_area,
            0,
            &[],
            &[],
        );
        // y_offset 은 col_area 절대 프레임의 단 콘텐츠 bottom. 호출부가 `current_height`
        // (=col_area.y 가 단 시작) 프레임과 정합하도록 그대로 반환한다.
        y_offset
    }

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
        self.current_body_area
            .set((ba.x, ba.y, ba.width, ba.height));

        // 문단 테두리 범위 수집 초기화
        self.para_border_ranges.borrow_mut().clear();

        // TopAndBottom 글상자/표/이미지의 앵커 문단별 예약 높이 목록
        let mut shape_reserved = self.calculate_shape_reserved_heights(
            paragraphs,
            &col_content.items,
            col_area,
            &layout.body_area,
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
        let allow_negative_visual_start = col_content.endnote_flow
            && col_content.start_height < -0.5
            && col_content
                .items
                .first()
                .map(|item| page_item_is_treat_as_char_picture_only(item, paragraphs))
                .unwrap_or(false);
        let start_shift = if allow_negative_visual_start {
            col_content.start_height.min(0.0)
        } else {
            0.0
        };
        let visual_col_y = col_area.y + start_shift;
        let visual_col_height = col_area.height - start_shift;
        let mut y_offset = visual_col_y;
        // [미주 구분선 아래 여백] 구분선 직후 첫 미주 본문 y 를 구분선의 belowLine 바닥
        // (layout_endnote_separator_item 반환 new_y)으로 floor 한다. 저장 vpos 가 구분선
        // 위(문서끝 재배치된 미주)여도 구분선 아래로 내려 텍스트 가림을 막는다. 정상 vpos
        // (구역끝 미주)는 이미 new_y 이상이라 max 로 불변.
        let mut endnote_sep_body_floor: Option<f64> = None;
        // body_area 전체에 걸치는 개체: 단 시작 y_offset을 개체 하단 아래로 초기화
        for &(_, bottom_y) in body_wide_reserved {
            if bottom_y > y_offset {
                y_offset = bottom_y;
            }
        }
        // [Task #901 Stage 8/10] TopAndBottom flow-around: anchor paragraph 의 text 가 picture
        // 위에 fit 가능하면 pre-jump skip, render 후 post-jump 적용.
        // (bottom_y, anchor_first_vpos) — Stage 10: vpos_lazy_base 를 anchor first vpos 로
        // 직접 설정. file vpos 가 anchor 첫 줄 vpos 기준 누적이므로, lazy_base 를 anchor
        // 첫 줄 vpos 로 두면 후속 paragraph 의 end_y = col_anchor_y + (vpos_end - first_vpos)
        // → file 의 누적 vpos 가 정확히 visual y 로 매핑됨.
        let mut pending_topbottom_post_jump: Option<(f64, i32)> = None;
        // [Task #412] vpos 보정 anchor: 첫 PageItem 이 실제 렌더링되는 y_offset.
        // body_wide_reserved 푸시 후의 y_offset 이 첫 항목의 vpos(=base) 에 대응됨.
        // 이를 anchor 로 사용해야 vpos→y 변환이 정확함 (col_area.y 는 단 영역 top
        // 으로 vpos=0 이 아니라 vpos=base 도 아닌 일반적으로 어긋난 값).
        let col_anchor_y = y_offset;

        let mut para_start_y: std::collections::HashMap<usize, f64> =
            std::collections::HashMap::new();
        let mut para_float_lanes: ParaFloatLanes = std::collections::HashMap::new();
        let mut visible_float_exclusions: Vec<VisibleFloatExclusion> = Vec::new();
        // [Task #1151 v9 결함 D] paragraph 단위 inline picture 가로 분배 cursor state.
        // 같은 paragraph 의 sibling tac=true picture 들이 가로로 inline 분배 (한컴 native 정합).
        let mut para_inline_state: std::collections::HashMap<
            usize,
            super::layout::paragraph_layout::ParaInlineState,
        > = std::collections::HashMap::new();

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
        let mut prev_tac_seg_applied = false;
        let mut tac_seg_applied_para: Option<usize> = None;
        let mut prev_endnote_title_gap_px = 0.0;
        let mut prev_endnote_title_gap_from_continued_partial = false;
        let mut pending_textless_equation_tail_gap_restore: Option<(EndnoteParaSource, f64)> = None;

        // 고정값 줄간격 TAC 표 병행 (Task #9): 표 하단 비교용
        let mut fix_table_start_y: f64 = 0.0;
        let mut fix_table_visual_h: f64 = 0.0;
        let mut fix_overlay_active = false;

        // vpos 보정을 위한 페이지 기준 vpos 계산
        // 페이지 첫 항목의 vpos를 기준점으로 삼아 모든 페이지에서 vpos 보정 적용
        let vpos_page_base_init: Option<i32> = col_content.items.first().and_then(|item| {
            match item {
                PageItem::FullParagraph { para_index } => paragraphs
                    .get(*para_index)
                    .and_then(|p| p.line_segs.first())
                    .map(|seg| seg.vertical_pos),
                PageItem::PartialParagraph {
                    para_index,
                    start_line,
                    ..
                } => paragraphs
                    .get(*para_index)
                    .and_then(|p| p.line_segs.get(*start_line))
                    .map(|seg| seg.vertical_pos),
                PageItem::Table { para_index, .. } => paragraphs
                    .get(*para_index)
                    .and_then(|p| p.line_segs.first())
                    .map(|seg| seg.vertical_pos),
                // PartialTable/Shape: 지연 보정 사용
                _ => None,
            }
        });
        // [Task #1027 Stage C] inter-item VPOS_CORR 상태머신을 HeightCursor 로 캡슐화.
        // vpos_page_base/lazy_base, prev_layout_para, prev_item_was_partial_table(#991:
        // 분할 표 직후 첫 문단은 sequential 신뢰)를 보유하며 항목 사이 vpos 보정을 위임.
        let mut hcursor = HeightCursor::new(
            self.dpi,
            visual_col_y,
            visual_col_height,
            col_anchor_y,
            vpos_page_base_init,
            self.use_hwp3_origin_flow_spacing_before.get(),
            false,
            col_content.endnote_flow && col_content.start_height < -0.5,
            col_content.endnote_flow,
        );
        hcursor.suppress_hwpx_stale_forward = self.is_hwpx_source.get();
        // [Task #1246] 미주 흐름 컬럼에만 between-notes 마진(HU)을 주입 → HeightCursor 가 새 미주
        // 제목 forward 흐름의 min-gap 보정에 사용. 본문 컬럼은 0 (무영향).
        if col_content.endnote_flow {
            hcursor.endnote_between_notes_hu = self.endnote_between_notes_hu.get();
        }

        // 1차 패스: 표, 문단, 텍스트 렌더링 (글상자 제외)
        for (item_ordinal, item) in col_content.items.iter().enumerate() {
            // vpos 기반 y_offset 보정
            let item_para = match item {
                PageItem::FullParagraph { para_index } => *para_index,
                PageItem::PartialParagraph { para_index, .. } => *para_index,
                PageItem::Table { para_index, .. } => *para_index,
                PageItem::PartialTable { para_index, .. } => *para_index,
                PageItem::Shape { para_index, .. } => *para_index,
                PageItem::EndnoteSeparator { .. } => {
                    // [미주 구분선 위치 — 한컴 정합] 직전 본문 문단 마지막 줄의 trailing
                    // 줄간격(line_spacing)은 한컴이 note 영역에 포함하지 않는다. rhwp 문단
                    // advance 는 포함하므로 그만큼 note 영역(구분선+본문)을 위로 올린다.
                    let trailing_ls_px = col_content.items[..item_ordinal]
                        .iter()
                        .rev()
                        .find_map(|it| match it {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. } => paragraphs
                                .get(*para_index)
                                .and_then(|p| p.line_segs.last())
                                .filter(|ls| ls.line_spacing > 0)
                                .map(|ls| hwpunit_to_px(ls.line_spacing, self.dpi)),
                            _ => None,
                        })
                        .unwrap_or(0.0);
                    y_offset = (y_offset - trailing_ls_px).max(0.0);
                    let (new_y, _) = self.layout_column_item(
                        tree,
                        &mut col_node,
                        paper_images,
                        &mut para_start_y,
                        &mut para_float_lanes,
                        &mut visible_float_exclusions,
                        &mut para_inline_state,
                        item,
                        page_content,
                        paragraphs,
                        composed,
                        styles,
                        bin_data_content,
                        measured_tables,
                        layout,
                        col_area,
                        outline_numbering_id,
                        multi_col_width,
                        y_offset,
                        prev_tac_seg_applied,
                        wrap_around_paras,
                        &col_content.wrap_anchors,
                    );
                    y_offset = new_y;
                    endnote_sep_body_floor = Some(new_y);
                    continue;
                }
            };
            // [Task #901 Stage 8/10] post-jump 적용: 직전 anchor paragraph 가 flow-around 로
            // 그림 위에 렌더된 경우 후속 paragraph 의 y_offset 을 picture bottom 으로 jump
            // + vpos_lazy_base 를 anchor first_vpos 로 set → file vpos 누적이 visual y 와 정합.
            // Shape/Table item 은 skip (vpos_lazy_base reset 회피).
            let item_is_paragraph = matches!(
                item,
                PageItem::FullParagraph { .. } | PageItem::PartialParagraph { .. }
            );
            if item_is_paragraph {
                if let Some((bottom_y, anchor_first_vpos)) = pending_topbottom_post_jump.take() {
                    // y_offset 이 bottom_y 보다 작으면 jump (예: iris Shape pre-jump 미적용 케이스)
                    if bottom_y > y_offset {
                        y_offset = bottom_y;
                    }
                    // vpos_lazy_base 는 항상 set (anchor first_vpos 기준 후속 paragraph 정합)
                    hcursor.vpos_lazy_base = Some(anchor_first_vpos);
                    hcursor.vpos_page_base = None;
                }
            }
            // TopAndBottom 글상자: 앵커 문단에 도달하면 y_offset을 글상자 하단 아래로 점프
            let mut shape_jumped = false;
            for &(anchor_pi, bottom_y) in &shape_reserved {
                if item_para == anchor_pi && bottom_y > y_offset {
                    // [Task #901 Stage 8] flow-around 시도: anchor 의 text height 가 picture 위
                    // 영역 (col_area.y ~ picture_top_y) 에 fit 가능하면 pre-jump skip.
                    use crate::model::shape::TextWrap;
                    let anchor_para = &paragraphs[anchor_pi];
                    let picture_top_y_opt: Option<f64> =
                        anchor_para.controls.iter().find_map(|c| {
                            let common = match c {
                                Control::Picture(pic) if !pic.common.treat_as_char => {
                                    Some(&pic.common)
                                }
                                Control::Shape(s) if !s.common().treat_as_char => Some(s.common()),
                                Control::Table(t) if !t.common.treat_as_char => Some(&t.common),
                                _ => None,
                            }?;
                            if !matches!(common.text_wrap, TextWrap::TopAndBottom) {
                                return None;
                            }
                            let (_bot, top) =
                                self.calc_shape_bottom_y(common, col_area, &layout.body_area);
                            Some(top)
                        });
                    let text_height = composed
                        .get(anchor_pi)
                        .map(|comp| {
                            comp.lines
                                .iter()
                                .map(|line| {
                                    crate::renderer::hwpunit_to_px(
                                        line.line_height + line.line_spacing,
                                        self.dpi,
                                    )
                                })
                                .sum::<f64>()
                        })
                        .unwrap_or(f64::MAX);
                    let fits_above = picture_top_y_opt
                        .map(|top_y| text_height + 4.0 <= (top_y - y_offset))
                        .unwrap_or(false);
                    if fits_above {
                        // Stage 11: 후속 paragraph item 의 first vpos 를 peek 하여 base 직접 계산.
                        // base = next_para_vpos - (bottom_y - col_area.y) * scale^-1
                        // → end_y for next para = bottom_y (iris 직하 정합).
                        let next_para_vpos: Option<i32> = col_content
                            .items
                            .iter()
                            .skip_while(|it| match it {
                                PageItem::FullParagraph { para_index } => *para_index != anchor_pi,
                                PageItem::PartialParagraph { para_index, .. } => {
                                    *para_index != anchor_pi
                                }
                                _ => true,
                            })
                            .skip(1) // anchor item 자체 skip
                            .find_map(|it| match it {
                                PageItem::FullParagraph { para_index }
                                | PageItem::PartialParagraph { para_index, .. } => paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.line_segs.first())
                                    .map(|s| s.vertical_pos),
                                _ => None,
                            });
                        let base_for_post = if let Some(npv) = next_para_vpos {
                            let visual_diff_hu =
                                ((bottom_y - col_area.y) / self.dpi * 7200.0).round() as i32;
                            npv - visual_diff_hu
                        } else {
                            anchor_para
                                .line_segs
                                .first()
                                .map(|s| s.vertical_pos)
                                .unwrap_or(0)
                        };
                        pending_topbottom_post_jump = Some((bottom_y, base_for_post));
                    } else {
                        y_offset = bottom_y;
                        shape_jumped = true;
                    }
                }
            }

            let current_is_endnote_question_title = col_content.endnote_flow
                && paragraphs
                    .get(item_para)
                    .map(|p| p.text.trim_start().starts_with('문'))
                    .unwrap_or(false);
            let current_endnote_source = if col_content.endnote_flow {
                self.endnote_para_source_for(item_para)
            } else {
                None
            };
            if current_is_endnote_question_title {
                if let (Some((pending_source, delta)), Some(current_source)) = (
                    pending_textless_equation_tail_gap_restore.as_ref(),
                    current_endnote_source.as_ref(),
                ) {
                    if !same_endnote_control(pending_source, current_source) {
                        // textless equation tail 뒤 제목에서 생략한 logical gap은
                        // 해당 미주의 본문까지는 적용하지 않는다. 다음 미주 제목을
                        // 만날 때만 vpos base에 복원해 후속 문항이 같이 당겨지지
                        // 않게 한다.
                        hcursor.shift_vpos_base_for_rendered_delta(*delta);
                        pending_textless_equation_tail_gap_restore = None;
                    }
                }
            }
            let y_before_vpos = y_offset;
            let prev_item_content_bottom_y = if item_ordinal > 0 {
                let content_bottom_y = self.last_item_content_bottom.get();
                content_bottom_y.is_finite().then_some(content_bottom_y)
            } else {
                None
            };
            hcursor.prev_item_content_bottom_y = prev_item_content_bottom_y;
            if !shape_jumped && (!prev_tac_seg_applied || current_is_endnote_question_title) {
                // [Task #1027 Stage C] inter-item VPOS_CORR 보정을 HeightCursor 에 위임 (동작 동일).
                // 이전 문단 overlay-shape/분할표 bypass, page/lazy base 산출, sb 차감,
                // ≤8px 백워드 클램프를 모두 캡슐화 (Stage A/B 함수 결합). 렌더러·페이지네이터 공유.
                y_offset = hcursor.vpos_adjust(y_offset, item_para, paragraphs, styles);
            } // !shape_jumped
            let current_title_tail_backtracked =
                current_is_endnote_question_title && y_offset < y_before_vpos - 32.0;
            let current_large_gap_title_compacted_by_cursor = current_is_endnote_question_title
                && col_content.endnote_flow
                && self.endnote_between_notes_hu.get() > 3000
                && y_offset < y_before_vpos - 0.5;
            let current_line_height_px = paragraphs
                .get(item_para)
                .and_then(|p| p.line_segs.first())
                .map(|seg| hwpunit_to_px(seg.line_height.max(0), self.dpi))
                .unwrap_or(0.0);
            let endnote_title_direct_bottom_fit = current_is_endnote_question_title
                && col_content.endnote_flow
                && current_line_height_px > 0.0
                && y_offset + current_line_height_px > col_area.y + col_area.height + 0.5
                && y_offset <= col_area.y + col_area.height + 80.0;
            if endnote_title_direct_bottom_fit {
                // TAC/수식 직후에는 prev_tac_seg_applied 때문에 HeightCursor 보정이
                // 생략될 수 있다. 그래도 새 문항 제목 1줄이 단 하단 안쪽에 들어가면
                // 한컴/PDF처럼 제목 tail만 현재 단에 남긴다.
                y_offset = (col_area.y + col_area.height - current_line_height_px - 7.0)
                    .max(col_area.y)
                    .min(y_offset);
            }
            let endnote_title_bottom_fit_applied = current_is_endnote_question_title
                && current_line_height_px > 0.0
                && y_offset < y_before_vpos - 0.5
                && y_before_vpos + current_line_height_px > col_area.y + col_area.height + 0.5
                && y_offset + current_line_height_px <= col_area.y + col_area.height + 0.5;
            let mut compacted_equation_tail_title_gap = false;
            let compact_single_equation_tail_gap_profile = self.endnote_between_notes_hu.get() > 0
                && self.endnote_between_notes_hu.get() <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU;
            if compact_single_equation_tail_gap_profile
                && current_is_endnote_question_title
                && col_content.endnote_flow
                && !endnote_title_direct_bottom_fit
                && !endnote_title_bottom_fit_applied
            {
                if let (Some(prev_pi), Some(prev_content_bottom_y), Some(current_para)) = (
                    hcursor.prev_layout_para,
                    prev_item_content_bottom_y,
                    paragraphs.get(item_para),
                ) {
                    if let Some(prev_para) = paragraphs.get(prev_pi) {
                        if let Some(compacted_y) =
                            compact_endnote_title_gap_after_single_equation_tail(
                                prev_para,
                                current_para,
                                prev_content_bottom_y,
                                y_offset,
                                prev_endnote_title_gap_px,
                                item_ordinal,
                                self.dpi,
                            )
                        {
                            y_offset = compacted_y.max(col_area.y);
                            hcursor.vpos_page_base = None;
                            hcursor.vpos_lazy_base = None;
                            compacted_equation_tail_title_gap = true;
                        }
                    }
                }
            }
            // [Task #1355] 미주 제목 saved-vpos 점프에 의한 gap 이중계상 정정.
            // 직전 미주 콘텐츠의 trailing line-spacing 이 흐름에 "미주 사이" gap 을 이미
            // 만들었는데(flow_advance ≈ gap), 제목의 saved LINE_SEG vpos 가 직전 bottom 보다
            // 크게 점프(원본에서 단/쪽 경계를 건넌 미주)하면 vpos_adjust 가 saved 기준으로 gap
            // 을 한 번 더 더해 제목 앞 여백이 약 2배가 된다(예: p18 문30 → 문24 답안 본문 초과).
            // 이때만 제목을 흐름 위치(y_before_vpos)로 되돌려 gap 을 한 번만 남긴다.
            // saved-vpos 점프가 작은 일반 순차 미주(2022_oct q19 등)는 vpos_adjust 가 정답
            // 이므로 제외 — flow_advance 만으로는 양자 시그니처가 동일(둘 다 ≈gap)해 구분 불가,
            // saved-vpos 점프량(원본 단/쪽 경계 신호)으로 구분한다.
            if current_is_endnote_question_title
                && col_content.endnote_flow
                && !compacted_equation_tail_title_gap
                && !endnote_title_direct_bottom_fit
                && !endnote_title_bottom_fit_applied
                && !current_title_tail_backtracked
                && prev_endnote_title_gap_px > 0.0
                && y_offset > y_before_vpos + 4.0
            {
                let cur_first_vpos = paragraphs
                    .get(item_para)
                    .and_then(|p| p.line_segs.first())
                    .map(|s| s.vertical_pos);
                let prev_last_bottom_vpos = hcursor
                    .prev_layout_para
                    .and_then(|pi| paragraphs.get(pi))
                    .and_then(|p| p.line_segs.last())
                    .map(|s| s.vertical_pos + s.line_height);
                let saved_delta_hu = match (cur_first_vpos, prev_last_bottom_vpos) {
                    (Some(cf), Some(pb)) => cf - pb,
                    _ => 0,
                };
                // 이중계상은 직전 미주 문단이 "수식 전용(보이는 텍스트 없음)" tail 일 때만
                // 발생한다(수식 tail 의 trailing line-spacing 인플레이션 + saved-vpos 점프).
                // 직전이 텍스트 문단이면 vpos_adjust 가 정답이므로 제외(2022_sep q15,
                // 2022_oct q29 회귀 방지).
                let prev_is_textless = hcursor
                    .prev_layout_para
                    .and_then(|pi| paragraphs.get(pi))
                    .map(|p| !para_has_visible_text(p))
                    .unwrap_or(false);
                if let Some(prev_bottom) = prev_item_content_bottom_y {
                    let flow_advance = y_before_vpos - prev_bottom;
                    if prev_is_textless
                        && flow_advance >= prev_endnote_title_gap_px * 0.9
                        && flow_advance <= prev_endnote_title_gap_px * 1.25
                        && saved_delta_hu > 5000
                    {
                        y_offset = y_before_vpos;
                        hcursor.vpos_page_base = None;
                        hcursor.vpos_lazy_base = None;
                        compacted_equation_tail_title_gap = true;
                    }
                }
            }
            if current_is_endnote_question_title
                && col_content.endnote_flow
                && !endnote_title_direct_bottom_fit
                && !endnote_title_bottom_fit_applied
                && !compacted_equation_tail_title_gap
            {
                let section_between_notes_gap_px =
                    hwpunit_to_px(self.endnote_between_notes_hu.get(), self.dpi);
                let zero_between_large_separator_profile =
                    self.current_endnote_zero_between_large_separator_profile();
                let effective_endnote_title_gap_px = if zero_between_large_separator_profile {
                    section_between_notes_gap_px
                } else if prev_endnote_title_gap_px >= 50.0 {
                    prev_endnote_title_gap_px
                } else {
                    section_between_notes_gap_px
                };
                let previous_item_para_index = item_ordinal
                    .checked_sub(1)
                    .and_then(|idx| col_content.items.get(idx))
                    .and_then(|prev_item| match prev_item {
                        PageItem::FullParagraph { para_index }
                        | PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                        _ => None,
                    });
                if let (Some(prev_pi), Some(prev_content_bottom_y)) = (
                    previous_item_para_index.or(hcursor.prev_layout_para),
                    prev_item_content_bottom_y,
                ) {
                    if let Some(prev_para) = paragraphs.get(prev_pi) {
                        let prev_has_textless_equation_tail = inline_equation_count(prev_para) > 0
                            && !para_has_visible_text(prev_para);
                        if prev_has_textless_equation_tail && effective_endnote_title_gap_px >= 50.0
                        {
                            let saved_head_gap_px = paragraphs
                                .get(item_para)
                                .and_then(|current_para| {
                                    let current_source = self.endnote_para_source_for(item_para)?;
                                    let prev_source = self.endnote_para_source_for(prev_pi)?;
                                    if current_source.note_para_index != 0
                                        || same_endnote_control(&current_source, &prev_source)
                                    {
                                        return None;
                                    }
                                    let is_last_column = (col_content.column_index as usize + 1)
                                        >= zone_layout.column_areas.len().max(1);
                                    let visible_separator_large_between_profile =
                                        self.endnote_between_notes_hu.get() > 3000
                                            && self.endnote_separator_above_hu.get()
                                                <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                                            && self.endnote_separator_below_hu.get()
                                                <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU;
                                    if !is_last_column || !visible_separator_large_between_profile {
                                        return None;
                                    }
                                    let mut saw_visible_body_before_large_tac = false;
                                    let mut current_head_has_large_tac = false;
                                    for (next_pi, next_para) in
                                        paragraphs.iter().enumerate().skip(item_para + 1).take(24)
                                    {
                                        let Some(next_source) =
                                            self.endnote_para_source_for(next_pi)
                                        else {
                                            continue;
                                        };
                                        if !(same_endnote_control(&current_source, &next_source)
                                            && next_source.note_para_index
                                                > current_source.note_para_index
                                            && next_source.note_para_index
                                                <= current_source.note_para_index + 8)
                                        {
                                            continue;
                                        }
                                        if !para_has_visible_text(next_para)
                                            && para_large_tac_picture_or_shape_height_px(
                                                next_para, self.dpi,
                                            )
                                            .is_some_and(|height| height >= 80.0)
                                            && saw_visible_body_before_large_tac
                                        {
                                            current_head_has_large_tac = true;
                                            break;
                                        }
                                        if para_has_visible_text(next_para) {
                                            saw_visible_body_before_large_tac = true;
                                        }
                                    }
                                    if !current_head_has_large_tac {
                                        return None;
                                    }
                                    let prev_seg = prev_para.line_segs.last()?;
                                    let current_first =
                                        current_para.line_segs.first()?.vertical_pos;
                                    let prev_content_bottom =
                                        prev_seg.vertical_pos + prev_seg.line_height;
                                    let saved_gap_hu = (current_first - prev_content_bottom).max(0);
                                    if saved_gap_hu <= 0 {
                                        return None;
                                    }
                                    let saved_gap_px = hwpunit_to_px(saved_gap_hu, self.dpi);
                                    (saved_gap_px >= 24.0)
                                        .then_some(saved_gap_px.min(effective_endnote_title_gap_px))
                                })
                                .unwrap_or(0.0);
                            let target_y = if saved_head_gap_px > 0.0 {
                                y_offset + saved_head_gap_px
                            } else {
                                prev_content_bottom_y + effective_endnote_title_gap_px
                            };
                            if y_offset + 1.0 < target_y {
                                let delta = target_y - y_offset;
                                y_offset = target_y;
                                hcursor.shift_vpos_base_for_rendered_delta(delta);
                                if saved_head_gap_px > 0.0 {
                                    compacted_equation_tail_title_gap = true;
                                }
                            }
                        }
                    }
                }
            }
            let compact_endnote_title_gap_already_compacted = current_is_endnote_question_title
                && (hcursor.last_compacted_endnote_title_gap || compacted_equation_tail_title_gap);
            let suppress_zero_between_large_separator_title_gap = self
                .current_endnote_zero_between_large_separator_profile()
                && prev_endnote_title_gap_px >= 50.0;
            let textless_equation_tail_gap_already_visible = current_is_endnote_question_title
                && col_content.endnote_flow
                && prev_endnote_title_gap_px > 0.0
                && !prev_endnote_title_gap_from_continued_partial
                && y_before_vpos > col_area.y + col_area.height * 0.65
                && hcursor
                    .prev_layout_para
                    .and_then(|prev_pi| {
                        let prev_para = paragraphs.get(prev_pi)?;
                        let prev_content_bottom_y = prev_item_content_bottom_y?;
                        let prev_has_textless_equation_tail = inline_equation_count(prev_para) > 0
                            && !para_has_visible_text(prev_para);
                        let required_gap_y = prev_content_bottom_y + prev_endnote_title_gap_px;
                        (prev_has_textless_equation_tail && y_offset + 0.5 >= required_gap_y)
                            .then_some(())
                    })
                    .is_some();
            let should_preserve_endnote_title_gap = current_is_endnote_question_title
                && prev_endnote_title_gap_px > 0.0
                && !endnote_title_direct_bottom_fit
                && !endnote_title_bottom_fit_applied
                && !compact_endnote_title_gap_already_compacted
                && !suppress_zero_between_large_separator_title_gap
                && !textless_equation_tail_gap_already_visible
                && !current_title_tail_backtracked
                && !current_large_gap_title_compacted_by_cursor
                && (prev_endnote_title_gap_from_continued_partial
                    || y_offset > y_before_vpos + 0.5);
            if textless_equation_tail_gap_already_visible {
                let min_y = y_before_vpos + prev_endnote_title_gap_px;
                if y_offset < min_y {
                    if let Some(source) = current_endnote_source.clone() {
                        pending_textless_equation_tail_gap_restore =
                            Some((source, min_y - y_offset));
                    }
                }
            }
            if should_preserve_endnote_title_gap {
                // Compact 미주에서 다음 문제 제목이 오면 LINE_SEG의 절대 vpos가
                // 현재 쪽/단 기준과 어긋나 직전 미주 내용 뒤의 "미주 사이"
                // 간격이 사라질 수 있다. 직전 paragraph 조각의 trailing
                // line_spacing을 공통 gap으로 보존하고, 후속 항목도 같은 기준을
                // 따르도록 vpos base를 함께 이동한다.
                // 다만 단 하단부 textless equation tail은 직전 content bottom 기준
                // gap이 이미 충분한 경우가 있다. 이때 logical flow 기준으로 다시
                // 보존하면 다음 문항 영역을 침범하므로 여기서 한 번 더 얹지 않는다.
                let min_y = y_before_vpos + prev_endnote_title_gap_px;
                if y_offset < min_y {
                    let delta = min_y - y_offset;
                    y_offset = min_y;
                    hcursor.shift_vpos_base_for_rendered_delta(delta);
                }
            }
            let current_vpos_rewinds_from_prev = hcursor
                .prev_layout_para
                .and_then(|prev_pi| {
                    let prev_first = paragraphs
                        .get(prev_pi)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)?;
                    let curr_first = paragraphs
                        .get(item_para)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)?;
                    Some(curr_first < prev_first)
                })
                .unwrap_or(false);

            let next_is_endnote_question_title = col_content.endnote_flow
                && col_content
                    .items
                    .get(item_ordinal + 1)
                    .and_then(|next_item| match next_item {
                        PageItem::FullParagraph { para_index }
                        | PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                        _ => None,
                    })
                    .and_then(|pi| paragraphs.get(pi))
                    .map(|p| p.text.trim_start().starts_with('문'))
                    .unwrap_or(false);
            if (matches!(
                item,
                PageItem::PartialParagraph { start_line, .. } if *start_line > 0
            ) && !next_is_endnote_question_title)
                || current_vpos_rewinds_from_prev
            {
                // 이어지는 partial paragraph는 이전 쪽/단에서 시작한 문단의 나머지다.
                // 다음 항목과의 간격은 원본 절대 vpos 차이가 아니라 현재 쪽의 순차 y를
                // 따라야 한다. 그렇지 않으면 3-09월 10쪽 첫 수식 뒤 문8)이 크게 밀린다.
                //
                // compact endnote 에서는 같은 쪽/단 안에서도 다음 미주 묶음이 낮은 vpos 로
                // 되감기는 구간이 있다(3-09월 p17/p18). 되감긴 문단을 다음 문단의 기준으로
                // 계속 쓰면 다음 줄이 위로 끌려가 겹치므로 여기서 vpos 기준을 끊는다.
                hcursor.prev_layout_para = None;
                hcursor.vpos_page_base = None;
                hcursor.vpos_lazy_base = None;
            } else {
                hcursor.prev_layout_para = Some(item_para);
            }

            // Percent 전환: 표 하단과 비교 (Task #9)
            if fix_overlay_active {
                let is_fixed = paragraphs
                    .get(item_para)
                    .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
                    .map(|ps| ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed)
                    .unwrap_or(false);
                // [Task #716] 빈 paragraph (text_len=0 또는 control 문자/object placeholder
                // 만 존재) 는 시각적으로 invisible. fix_overlay push 가 적용되어도
                // 보이는 차이가 없는 반면 y_offset 만 (table_bottom - y_offset) 만큼
                // 누적되어 forward drift 의 누적 원인이 된다 (page 1 LAYOUT_OVERFLOW
                // 의 99.3%: pi=1 +8 px + pi=3 +12 px). Task #9 의 push 의도(텍스트
                // paragraph 가 TAC 표 위에 침범하지 않도록 보호) 는 그대로 유지하고,
                // 빈 paragraph 는 push 대상에서 제외한다. fix_overlay_active 는 유지하여
                // 후속 비-empty paragraph 가 push 대상이 될 수 있게 한다.
                let is_empty_para = paragraphs
                    .get(item_para)
                    .map(|p| {
                        p.text.is_empty()
                            || p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
                    })
                    .unwrap_or(false);
                if !is_fixed && !is_empty_para {
                    let table_bottom = fix_table_start_y + fix_table_visual_h;
                    if y_offset < table_bottom {
                        y_offset = table_bottom;
                    }
                    fix_overlay_active = false;
                }
            }

            if item_is_paragraph && !visible_float_exclusions.is_empty() {
                visible_float_exclusions.retain(|zone| y_offset < zone.bottom - 0.5);
                // [Task #1794] 잉크-겹침 프로브를 HWP5 소스에도 적용 — 자리차지 표의
                // exclusion zone 과 문단 첫 줄 잉크가 겹치면 소스 포맷과 무관하게 표
                // 아래로 밀어야 한다 (seoul_0765: HWPX 직파스와 HWP5 재파스의 표 앵커
                // 95.16px 갈림). HWPX 전용 게이트는 도입 당시 blast radius 제한이었다.
                let item_probe_height = {
                    match item {
                        PageItem::FullParagraph { para_index } => paragraphs
                            .get(*para_index)
                            .and_then(|p| {
                                p.line_segs.first().map(|seg| {
                                    let line_height = if p.controls.is_empty()
                                        && seg.text_height > 0
                                        && seg.text_height < seg.line_height
                                    {
                                        seg.text_height
                                    } else {
                                        seg.line_height
                                    };
                                    // [Task #1789] line_spacing 은 겹침 판정에서 제외 —
                                    // 잉크가 zone 위에 들어가는 줄을 spacing 포함분 수 px
                                    // 겹침으로 표 아래로 밀면 한컴 저장 flow(36385142 pi8
                                    // vpos=34925 = 표 위 유지)와 어긋난다 (최대 345px).
                                    hwpunit_to_px(line_height, self.dpi)
                                })
                            })
                            .unwrap_or(0.0),
                        PageItem::PartialParagraph {
                            para_index,
                            start_line,
                            ..
                        } => paragraphs
                            .get(*para_index)
                            .and_then(|p| {
                                p.line_segs.get(*start_line).map(|seg| {
                                    let line_height = if p.controls.is_empty()
                                        && seg.text_height > 0
                                        && seg.text_height < seg.line_height
                                    {
                                        seg.text_height
                                    } else {
                                        seg.line_height
                                    };
                                    // [Task #1789] line_spacing 은 겹침 판정에서 제외 —
                                    // 잉크가 zone 위에 들어가는 줄을 spacing 포함분 수 px
                                    // 겹침으로 표 아래로 밀면 한컴 저장 flow(36385142 pi8
                                    // vpos=34925 = 표 위 유지)와 어긋난다 (최대 345px).
                                    hwpunit_to_px(line_height, self.dpi)
                                })
                            })
                            .unwrap_or(0.0),
                        _ => 0.0,
                    }
                };
                let mut jump_to = y_offset;
                for zone in &visible_float_exclusions {
                    // [Issue #1549] 자기 문단에 앵커된 float 표는 그 문단의 텍스트(제목)를
                    // 밀어내지 않는다 — 제목은 앵커(표 위)에 남아야 한다. owner 가 다른 후속
                    // 문단은 그대로 표 아래로 밀린다.
                    if zone.owner_para == item_para {
                        continue;
                    }
                    let starts_in_zone = jump_to + 0.5 >= zone.top && jump_to < zone.bottom;
                    let overlaps_zone = item_probe_height > 0.0
                        && jump_to < zone.top
                        && jump_to + item_probe_height > zone.top + 0.5;
                    if starts_in_zone || overlaps_zone {
                        jump_to = jump_to.max(zone.bottom);
                    }
                }
                if jump_to > y_offset + 0.5 {
                    let delta = jump_to - y_offset;
                    y_offset = jump_to;
                    hcursor.shift_vpos_base_for_rendered_delta(delta);
                }
            }

            let _dbg_tac = std::env::var("RHWP_DEBUG_TAC_CURSOR").is_ok();
            let _y_in = y_offset;
            let _item_desc = if _dbg_tac {
                match item {
                    PageItem::FullParagraph { para_index } => format!("FullPara pi={}", para_index),
                    PageItem::PartialParagraph { para_index, .. } => {
                        format!("PartialPara pi={}", para_index)
                    }
                    PageItem::Table {
                        para_index,
                        control_index,
                    } => format!("Table pi={} ci={}", para_index, control_index),
                    PageItem::PartialTable {
                        para_index,
                        control_index,
                        ..
                    } => format!("PartialTable pi={} ci={}", para_index, control_index),
                    PageItem::Shape {
                        para_index,
                        control_index,
                        ..
                    } => format!("Shape pi={} ci={}", para_index, control_index),
                    PageItem::EndnoteSeparator { .. } => "EndnoteSeparator".to_string(),
                }
            } else {
                String::new()
            };
            // [Task #1046 Stage 3 Class B] 표 콘텐츠 하단 기록을 항목마다 리셋 —
            // 표 항목 렌더에서만 설정되므로, 비-표 항목/다른 표에 stale 값이 새지 않는다.
            self.last_item_content_bottom.set(f64::NAN);
            self.last_item_endnote_equation_tail_line_box.set(false);
            let zero_between_shape_tail_margin_px = match item {
                PageItem::Shape {
                    para_index,
                    control_index,
                } if col_content.endnote_flow
                    && item_ordinal == 0
                    && self.current_endnote_zero_between_large_separator_profile() =>
                {
                    let current_source = self.endnote_para_source_for(*para_index);
                    let next_para_index =
                        col_content
                            .items
                            .get(item_ordinal + 1)
                            .and_then(|it| match it {
                                PageItem::FullParagraph { para_index }
                                | PageItem::PartialParagraph { para_index, .. }
                                | PageItem::Table { para_index, .. }
                                | PageItem::PartialTable { para_index, .. }
                                | PageItem::Shape { para_index, .. } => Some(*para_index),
                                PageItem::EndnoteSeparator { .. } => None,
                            });
                    let next_is_new_question = next_para_index
                        .and_then(|next_pi| {
                            let next_para = paragraphs.get(next_pi)?;
                            let next_source = self.endnote_para_source_for(next_pi)?;
                            let current_source = current_source.as_ref()?;
                            let same_note = current_source.section_index
                                == next_source.section_index
                                && current_source.para_index == next_source.para_index
                                && current_source.control_index == next_source.control_index;
                            (endnote_question_number(next_para).is_some() && !same_note)
                                .then_some(())
                        })
                        .is_some();
                    if next_is_new_question {
                        paragraphs
                            .get(*para_index)
                            .and_then(|para| {
                                textless_non_tac_topbottom_object_tail_advance_px(
                                    para,
                                    *control_index,
                                    self.dpi,
                                )
                            })
                            .unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            };
            // 구분선 직후 첫 미주 본문: belowLine 바닥(new_y)으로 floor.
            if let Some(floor) = endnote_sep_body_floor.take() {
                y_offset = y_offset.max(floor);
            }
            let (mut new_y, was_tac) = self.layout_column_item(
                tree,
                &mut col_node,
                paper_images,
                &mut para_start_y,
                &mut para_float_lanes,
                &mut visible_float_exclusions,
                &mut para_inline_state,
                item,
                page_content,
                paragraphs,
                composed,
                styles,
                bin_data_content,
                measured_tables,
                layout,
                col_area,
                outline_numbering_id,
                multi_col_width,
                y_offset,
                prev_tac_seg_applied,
                wrap_around_paras,
                &col_content.wrap_anchors,
            );
            if zero_between_shape_tail_margin_px > 0.0 {
                // 미주 사이 0에서 직전 미주의 마지막 수식 tail을 앞 단에 남기고
                // 비TAC 그림만 다음 단으로 넘긴 경우, 한컴은 그림 뒤 bottom margin을
                // 새 문항 앞 빈 줄처럼 소비하지 않는다.
                new_y = (new_y - zero_between_shape_tail_margin_px).max(_y_in);
            }
            if _dbg_tac {
                eprintln!(
                    "TAC_CURSOR  {} y_in={:.1} y_out={:.1} dy={:.1} was_tac={}",
                    _item_desc,
                    _y_in,
                    new_y,
                    new_y - _y_in,
                    was_tac,
                );
            }
            if col_content.endnote_flow && std::env::var("RHWP_EN_SSOT_DEBUG").is_ok() {
                eprintln!(
                    "EN_RENDER pi={} y_in_rel={:.1} y_out_rel={:.1} dy={:.1} col_h={:.1}",
                    item_para,
                    _y_in - col_area.y,
                    new_y - col_area.y,
                    new_y - _y_in,
                    col_area.height,
                );
            }
            y_offset = new_y;
            if was_tac {
                tac_seg_applied_para = Some(item_para);
            }
            // A TAC segment is a paragraph-level line-segment condition, not a property
            // of whichever PageItem happened to be emitted last for that paragraph.
            // PR #1088 may render a para-relative float after a TAC table; keep the
            // next-paragraph vpos guard active until we leave that host paragraph.
            prev_tac_seg_applied = was_tac || tac_seg_applied_para == Some(item_para);
            // [Task #991] 다음 반복의 vpos 보정용 — 직전 항목이 분할 표였는지 기록.
            hcursor.prev_item_was_partial_table = matches!(item, PageItem::PartialTable { .. });
            let mut next_endnote_title_gap_from_continued_partial = false;
            prev_endnote_title_gap_px = if col_content.endnote_flow {
                match item {
                    PageItem::FullParagraph { para_index } => paragraphs
                        .get(*para_index)
                        .and_then(|p| p.line_segs.last())
                        // [Task #1257] line_spacing>1000 이 주입된 between-notes 갭 마커. 직전
                        // 미주가 tall 줄(수식)로 끝나도 갭은 보존해야 하므로 line_height 제한 제거
                        // (문26 lh=2070·문29 lh=6897 케이스가 갭 0 으로 떨어지던 원인).
                        .filter(|seg| seg.line_spacing > 1000)
                        .map(|seg| hwpunit_to_px(seg.line_spacing.max(0), self.dpi))
                        .unwrap_or(0.0),
                    PageItem::PartialParagraph {
                        para_index,
                        start_line,
                        end_line,
                    } if *start_line > 0 => paragraphs
                        .get(*para_index)
                        .and_then(|p| p.line_segs.get(end_line.saturating_sub(1)))
                        .map(|seg| {
                            next_endnote_title_gap_from_continued_partial = true;
                            hwpunit_to_px(seg.line_spacing.max(0), self.dpi)
                        })
                        .unwrap_or(0.0),
                    _ => 0.0,
                }
            } else {
                0.0
            };
            prev_endnote_title_gap_from_continued_partial =
                next_endnote_title_gap_from_continued_partial;

            // 고정값 줄간격 TAC 표 병행 (Task #9)
            if was_tac {
                if let Some(para) = paragraphs.get(item_para) {
                    if let Some(seg) = para.line_segs.first() {
                        if seg.line_spacing < 0 {
                            // 표 시작 y와 시각적 높이 저장 (Percent 전환 시 비교용)
                            let ps = styles.para_styles.get(para.para_shape_id as usize);
                            let sa = ps.map(|s| s.spacing_after).unwrap_or(0.0);
                            fix_table_start_y = y_offset
                                - hwpunit_to_px(seg.line_height + seg.line_spacing, self.dpi)
                                    .max(0.0)
                                - sa;
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
            let is_table_or_shape = matches!(
                item,
                PageItem::Table { .. } | PageItem::PartialTable { .. } | PageItem::Shape { .. }
            );
            let is_para_float_table = if let PageItem::Table {
                para_index,
                control_index,
            } = item
            {
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
            // 예외 2 (#1898): 실제 텍스트 줄에 통합된 글자처럼(tac) 인라인 개체의
            // Shape 항목은 흐름에 독립 높이를 만들지 않는다(호스트 LINE_SEG 는 텍스트
            // 줄 높이, 본 항목 dy=0). 기준점을 초기화하면 다음 문단 vpos_adjust 가
            // lazy_base 재산출에서 trailing-ls bridge 를 다시 적용해, 불릿 그림 문단
            // 마다 렌더 y 가 줄간격 1회분씩 과대 전진한다 (36388711 p9: layout
            // 33.1px vs 렌더 44.8px, 한컴 32.9px). 단, 텍스트 없는 tac-전용 문단
            // (LINE_SEG lh = 개체 높이, sample16 pi=71 RFP 박스)은 종전대로 초기화 —
            // 그 lh 는 개체 높이를 포함해 vpos 누적과 순차 y 의 drift 근거가 맞다.
            let is_inline_tac_object = if let PageItem::Shape {
                para_index,
                control_index,
            } = item
            {
                paragraphs
                    .get(*para_index)
                    .map(|p| {
                        para_has_visible_text(p)
                            && p.controls
                                .get(*control_index)
                                .map(|c| match c {
                                    Control::Picture(pic) => pic.common.treat_as_char,
                                    Control::Shape(shape) => shape.common().treat_as_char,
                                    Control::Equation(eq) => eq.common.treat_as_char,
                                    _ => false,
                                })
                                .unwrap_or(false)
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            if was_tac || (is_table_or_shape && !is_para_float_table && !is_inline_tac_object) {
                hcursor.vpos_page_base = None;
                hcursor.vpos_lazy_base = None;
            }

            // [Task #1046 Stage 1] 렌더러 항목별 y_offset 진행 로그 (페이지네이터 cur_h 대조).
            if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
                eprintln!(
                    "LAYOUT_Y: page={} sec={} ord={} pi={} y_after={:.1} (body_top={:.1})",
                    page_content.page_index,
                    page_content.section_index,
                    item_ordinal,
                    item_para,
                    y_offset,
                    col_area.y,
                );
            }

            // 자가 검증: 배치 후 y_offset이 단 영역 하단을 초과하는지 확인
            let col_bottom = col_area.y + col_area.height;
            let tolerance = 2.0; // 반올림 오차 허용 (2px)
                                 // [Task #1046 Stage 3 Class B] 표 항목은 표 뒤 trailing 간격(host 문단 줄간격/
                                 // spacing_after)이 더해진 y_offset 대신 실제 콘텐츠 하단으로 초과를 판정한다.
                                 // 페이지 바닥의 후행 간격은 다음 항목이 다음 페이지로 가므로 시각적 초과가
                                 // 아니다(문단 trailing_ls 정책 #359/#404 의 표 대응). 표가 아니거나 콘텐츠
                                 // 하단 미기록(NaN)이면 종전대로 y_offset 사용.
            let check_y = match item {
                PageItem::Table { .. }
                | PageItem::PartialTable { .. }
                | PageItem::FullParagraph { .. }
                | PageItem::PartialParagraph { .. }
                | PageItem::Shape { .. } => {
                    let cb = self.last_item_content_bottom.get();
                    if cb.is_finite() {
                        cb
                    } else {
                        y_offset
                    }
                }
                _ => y_offset,
            };
            // 마지막 continuation 직전 항목도 미주 꼬리로 본다. 작은 bottom
            // bleed는 draw overflow가 없으면 한컴식 하단 배치 허용 범위다.
            let same_endnote_successor = match item {
                PageItem::FullParagraph { para_index }
                | PageItem::PartialParagraph { para_index, .. } => {
                    self.endnote_para_has_same_endnote_successor(*para_index)
                }
                _ => false,
            };
            let is_endnote_tail_item = col_content.endnote_flow
                && (item_ordinal + 1 == col_content.items.len()
                    || (item_ordinal + 2 == col_content.items.len()
                        && current_is_endnote_question_title)
                    || (item_ordinal + 2 == col_content.items.len()
                        && matches!(
                            col_content.items.get(item_ordinal + 1),
                            Some(PageItem::PartialParagraph { .. })
                        ))
                    || same_endnote_successor);
            let is_zero_spacing_endnote_item =
                col_content.endnote_flow && self.current_endnote_zero_spacing_profile();
            let tolerated_endnote_bottom_bleed = self.is_tolerated_current_endnote_bottom_bleed(
                is_endnote_tail_item || is_zero_spacing_endnote_item,
                check_y,
                col_bottom,
                self.last_item_endnote_equation_tail_line_box.get(),
            );
            if check_y > col_bottom + tolerance && !tolerated_endnote_bottom_bleed {
                let (item_type, para_idx) = match item {
                    PageItem::FullParagraph { para_index } => ("FullParagraph", *para_index),
                    PageItem::PartialParagraph { para_index, .. } => {
                        ("PartialParagraph", *para_index)
                    }
                    PageItem::Table { para_index, .. } => ("Table", *para_index),
                    PageItem::PartialTable { para_index, .. } => ("PartialTable", *para_index),
                    PageItem::Shape { para_index, .. } => ("Shape", *para_index),
                    PageItem::EndnoteSeparator { .. } => ("EndnoteSeparator", usize::MAX),
                };
                self.record_overflow(LayoutOverflow {
                    page_index: page_content.page_index,
                    section_index: page_content.section_index,
                    column_index: col_content.column_index as usize,
                    para_index: para_idx,
                    item_type,
                    is_first_in_column: item_ordinal == 0,
                    element_y: check_y,
                    column_bottom: col_bottom,
                    overflow_px: check_y - col_bottom,
                });
            }
        }

        // 2차 패스: 글상자(Shape) z-order 정렬 후 렌더링
        self.layout_column_shapes_pass(
            tree,
            &mut col_node,
            paper_images,
            col_content,
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            layout,
            col_area,
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
                    let any_w = bs
                        .borders
                        .iter()
                        .any(|b| !matches!(b.line_type, BorderLineType::None));
                    if any_w {
                        Some((top.line_type, top.width, top.color))
                    } else {
                        None
                    }
                };
                // 그룹 튜플: (bf_id, x, y_start, w, y_end, top_inset, bottom_inset,
                //              is_partial_start, is_partial_end, first_para_idx, last_para_idx)
                let mut groups: Vec<(u16, f64, f64, f64, f64, f64, f64, bool, bool, usize, usize)> =
                    Vec::new();
                for &(
                    bf_id,
                    x,
                    y_start,
                    w,
                    y_end,
                    top_inset,
                    bottom_inset,
                    is_partial_start,
                    is_partial_end,
                    para_idx,
                ) in ranges.iter()
                {
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
                            last.10 = para_idx; // last_para_idx 갱신
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
                    groups.push((
                        bf_id,
                        x,
                        y_start,
                        w,
                        y_end,
                        top_inset,
                        bottom_inset,
                        is_partial_start,
                        is_partial_end,
                        para_idx,
                        para_idx,
                    ));
                }

                // Task #468: cross-column 박스 연속 검출.
                // sequential 인접 paragraph 가 같은 stroke_sig 면 박스가 다른 컬럼/페이지로 이어진 것.
                // [Task #471] bf_id 비교가 아닌 stroke_sig 비교 — 머지(Task #321 v6)가 visual
                // stroke 기준으로 동작하므로 그룹의 g.0 bf_id 는 첫 range 의 bf_id 만 보존됨.
                // 그룹의 visual sig 와 인접 paragraph 의 visual sig 비교가 정확.
                for g in groups.iter_mut() {
                    let bf_id = g.0;
                    if bf_id == 0 {
                        continue;
                    }
                    let first_pi = g.9;
                    let last_pi = g.10;
                    let group_sig = stroke_sig(bf_id);
                    if group_sig.is_none() {
                        continue;
                    }

                    let para_bf = |pi: usize| -> u16 {
                        composed
                            .get(pi)
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
                    if g.2 < col_top {
                        g.2 = col_top;
                    }
                    if g.4 > col_bot {
                        g.4 = col_bot;
                    }
                }
                groups.retain(|g| g.4 > g.2);

                let groups_len = groups.len();
                for (
                    gi,
                    (
                        bf_id,
                        x,
                        y_start,
                        w,
                        y_end,
                        top_inset,
                        bottom_inset,
                        is_partial_start,
                        is_partial_end,
                        _,
                        _,
                    ),
                ) in groups.clone().into_iter().enumerate()
                {
                    let height = y_end - y_start;
                    if height <= 0.0 {
                        continue;
                    }
                    // 인접한 다른 border 그룹 (간격 < 4px) 과는 inset 충돌 회피.
                    let prev_touches = gi > 0 && (y_start - groups[gi - 1].4) < 4.0;
                    let next_touches = gi + 1 < groups_len && (groups[gi + 1].2 - y_end) < 4.0;
                    let idx = (bf_id as usize).saturating_sub(1);
                    let border_style = styles.border_styles.get(idx);
                    let fill_color = border_style.and_then(|bs| bs.fill_color);
                    let borders = border_style.map(|bs| bs.borders);
                    let stroke_width = borders
                        .map(|borders| {
                            borders
                                .iter()
                                .filter(|border| para_border_is_visible(border))
                                .map(|border| {
                                    super::layout::border_rendering::border_width_to_px(
                                        border.width,
                                    )
                                })
                                .fold(0.0, f64::max)
                        })
                        .unwrap_or(0.0);
                    // Task #321 v6: ParaShape::border_spacing 정식 반영 + stroke 있을 때 default 2px 최소.
                    // 인접 border 그룹과 충돌 방지를 위해 인접 경계는 inset 0.
                    const DEFAULT_MIN_INSET: f64 = 2.0;
                    let top_pad = if stroke_width > 0.0 && !prev_touches {
                        top_inset.max(DEFAULT_MIN_INSET)
                    } else {
                        top_inset
                    };
                    let bot_pad = if stroke_width > 0.0 && !next_touches {
                        bottom_inset.max(DEFAULT_MIN_INSET)
                    } else {
                        bottom_inset
                    };
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
                    let can_use_rect_stroke = borders
                        .map(|borders| {
                            para_border_can_use_rect_stroke(&borders, skip_top, skip_bottom)
                        })
                        .unwrap_or(false);
                    if can_use_rect_stroke {
                        // 기존 경로: 단일 Rectangle (fill + 4면 stroke)
                        let stroke_border =
                            borders.expect("can_use_rect_stroke requires borders")[0];
                        let rect_id = tree.next_id();
                        let rect_node = RenderNode::new(
                            rect_id,
                            RenderNodeType::Rectangle(super::render_tree::RectangleNode::new(
                                0.0,
                                super::ShapeStyle {
                                    fill_color,
                                    stroke_color: Some(stroke_border.color),
                                    stroke_width:
                                        super::layout::border_rendering::border_width_to_px(
                                            stroke_border.width,
                                        ),
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
                        let mut push_border_line =
                            |border: &BorderLine, x1: f64, y1: f64, x2: f64, y2: f64| {
                                if !para_border_is_visible(border) {
                                    return;
                                }
                                let nodes =
                                    super::layout::border_rendering::create_border_line_nodes(
                                        tree, border, x1, y1, x2, y2,
                                    );
                                for node in nodes {
                                    col_node.children.insert(0, node);
                                }
                            };
                        let x_left = x;
                        let x_right = x + w;
                        let y_top = rect_y;
                        let y_bot = rect_y + rect_h;
                        if let Some(borders) = borders {
                            push_border_line(&borders[0], x_left, y_top, x_left, y_bot);
                            push_border_line(&borders[1], x_right, y_top, x_right, y_bot);
                            if !skip_top {
                                push_border_line(&borders[2], x_left, y_top, x_right, y_top);
                            }
                            if !skip_bottom {
                                push_border_line(&borders[3], x_left, y_bot, x_right, y_bot);
                            }
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
        para_float_lanes: &mut ParaFloatLanes,
        visible_float_exclusions: &mut Vec<VisibleFloatExclusion>,
        // [Task #1151 v9 결함 D] sibling TAC picture 가로 분배 cursor state.
        para_inline_state: &mut std::collections::HashMap<
            usize,
            super::layout::paragraph_layout::ParaInlineState,
        >,
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
        wrap_anchors: &std::collections::HashMap<usize, super::pagination::WrapAnchorRef>,
    ) -> (f64, bool) {
        let ctx = ColumnItemCtx {
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            measured_tables,
            layout,
            col_area,
            outline_numbering_id,
            multi_col_width,
            prev_tac_seg_applied,
            wrap_around_paras,
            wrap_anchors,
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
                                tree,
                                col_node,
                                para,
                                Some(comp),
                                styles,
                                col_area,
                                y_offset,
                                page_content.section_index,
                                *para_index,
                                multi_col_width,
                                Some(bin_data_content),
                                ctx.wrap_anchors.get(para_index),
                            );
                        }
                    }
                    return (y_offset, false);
                }
                if let Some(para) = paragraphs.get(*para_index) {
                    if para_has_visible_textless_float_shape_item(page_content, para, *para_index) {
                        // 빈 non-TAC 그림/도형 host 문단은 바로 뒤 Shape PageItem 이 실제
                        // 개체를 렌더한다. 여기서 layout_paragraph 를 태우면 보이지 않는
                        // 빈 줄이 저장 vpos 기준으로 페이지 밖에 기록되어 overflow 오탐이 난다.
                        para_start_y.entry(*para_index).or_insert(y_offset);
                        if textless_infront_para_host_requires_line_advance(para) {
                            // HWPX 글앞으로 도장처럼 문단 기준으로 붙는 host 는 빈
                            // 텍스트를 그리지 않더라도 한컴처럼 줄 진행량은 예약한다.
                            // BehindText 배경 그림은 기존 비예약 경로를 유지한다.
                            let advance = paragraph_line_advance_px(
                                para,
                                composed.get(*para_index),
                                self.dpi,
                            );
                            return (y_offset + advance, false);
                        }
                        return (y_offset, false);
                    }

                    let seg_width = effective_tac_segment_width_hu(
                        para,
                        px_to_hwpunit(col_area.width, self.dpi),
                    );
                    let has_block_table = para.controls.iter()
                        .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char
                            || (t.common.treat_as_char
                                && !crate::renderer::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls))));
                    if has_block_table {
                        if para_is_empty_topbottom_table_anchor(para) {
                            // 빈 기본 표 host 문단은 별도 빈 줄로 소비하지 않는다.
                            // 표 PageItem 렌더 시 같은 y에 문단부호를 얹어 한컴처럼
                            // 첫 조판부호가 표와 겹쳐 보이게 한다.
                            para_start_y.entry(*para_index).or_insert(y_offset);
                            return (y_offset, false);
                        }

                        let comp = composed.get(*para_index);
                        let para_style_id = comp
                            .map(|c| c.para_style_id as usize)
                            .unwrap_or(para.para_shape_id as usize);
                        if let Some(para_style) = styles.para_styles.get(para_style_id) {
                            // 번호 카운터 전진 (후속 문단의 번호 연속성 유지)
                            // Bullet은 카운터를 사용하지 않으므로 제외
                            if para_style.head_type == HeadType::Outline
                                || para_style.head_type == HeadType::Number
                            {
                                let nid = resolve_numbering_id(
                                    para_style.head_type,
                                    para_style.numbering_id,
                                    outline_numbering_id,
                                );
                                if nid > 0 {
                                    self.numbering_state.borrow_mut().advance(
                                        nid,
                                        para_style.para_level,
                                        para.numbering_restart,
                                    );
                                }
                            }
                            if para_style.spacing_before > 0.0 {
                                y_offset += para_style.spacing_before;
                            }
                        }
                        // 어울림 표 호스트 문단의 텍스트는 layout_wrap_around_paras에서 처리
                        let is_wrap_host = para.controls.iter().any(|c| {
                            if let Control::Table(t) = c {
                                !t.common.treat_as_char
                                    && matches!(
                                        t.common.text_wrap,
                                        crate::model::shape::TextWrap::Square
                                    )
                            } else {
                                false
                            }
                        });
                        // 블록 표/도형 외에 실제 텍스트가 있는지 확인
                        // (예: [선][선][표][표]참고문헌 → 표 아래에 텍스트 렌더링 필요)
                        let has_real_text = !is_wrap_host
                            && para
                                .text
                                .chars()
                                .any(|c| c > '\u{001F}' && c != '\u{FFFC}' && !c.is_whitespace());
                        if has_real_text {
                            if let Some(comp) = comp {
                                // 컨트롤 전용 줄(runs가 모두 제어문자)을 건너뛰고 텍스트 줄부터 렌더링
                                let text_start_line = comp.lines.iter().position(|line| {
                                    line.runs.iter().any(|r| {
                                        r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                    })
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
                                        ctx.wrap_anchors.get(para_index),
                                    );
                                }
                            }
                        }
                        return (y_offset, false);
                    }

                    let has_inline_tables = para.controls.iter()
                        .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char
                            && crate::renderer::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls)));

                    // [Task #565] 인라인 표 + 다른 인라인 컨트롤(수식/treat_as_char Picture/Shape)
                    // 이 같이 있는 문단은 layout_inline_table_paragraph 가 인라인 수식 등을
                    // 처리하지 않아 shape_layout fallback (col_area.x, para_y) 으로 9개 수식이
                    // 동일 좌표에 겹친다 (exam_science.hwp 12/15/18/19번). 일반
                    // layout_paragraph 로 보내 인라인 표 + 인라인 수식이 같은 line/x 체계
                    // (run_tacs / inline_x) 로 정상 배치되도록 한다.
                    let has_other_inline_ctrls = para.controls.iter().any(|c| match c {
                        Control::Equation(_) => true,
                        Control::Picture(p) => p.common.treat_as_char,
                        Control::Shape(s) => s.common().treat_as_char,
                        _ => false,
                    });

                    if has_inline_tables && !has_other_inline_ctrls {
                        // 인라인 표 문단도 번호 카운터 전진 필요
                        self.apply_paragraph_numbering(
                            composed.get(*para_index),
                            para,
                            styles,
                            outline_numbering_id,
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
                            comp,
                            para,
                            styles,
                            outline_numbering_id,
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
                            ctx.wrap_anchors.get(para_index),
                        );
                    }
                    // TAC Shape 높이 보정: 문단에 TAC Shape(개체묶기 등)가 있으면
                    // Shape 높이가 문단 텍스트 높이보다 클 수 있으므로 y_offset을 보정.
                    // LINE_SEG lh가 Shape+캡션+간격을 모두 포함하므로 max(Shape.height, lh)를 사용.
                    // 보정 시 원래 문단 간격(spacing_after)도 유지한다.
                    {
                        let has_tac_shape = para
                            .controls
                            .iter()
                            .any(|c| matches!(c, Control::Shape(s) if s.common().treat_as_char));
                        if has_tac_shape {
                            // LINE_SEG lh = 이미지+캡션+간격 전체 높이
                            let seg_lh: f64 = para
                                .line_segs
                                .iter()
                                .map(|seg| hwpunit_to_px(seg.line_height, self.dpi))
                                .fold(0.0f64, f64::max);
                            let shape_max_h: f64 = para
                                .controls
                                .iter()
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
                                    let spacing = styles
                                        .para_styles
                                        .get(para.para_shape_id as usize)
                                        .map(|s| s.spacing_after)
                                        .unwrap_or(0.0);
                                    y_offset = shape_bottom + spacing;
                                }
                            }
                        }
                    }
                    // 각주 위첨자: footnote_positions가 있으면 인라인으로 이미 처리됨
                    let has_inline_fn = composed
                        .get(*para_index)
                        .map(|c| !c.footnote_positions.is_empty())
                        .unwrap_or(false);
                    if !has_inline_fn {
                        self.add_footnote_superscripts(tree, col_node, para, styles);
                    }
                }
            }
            PageItem::PartialParagraph {
                para_index,
                start_line,
                end_line,
            } => {
                if let Some(para) = paragraphs.get(*para_index) {
                    // Task #318: wrap=Square 표 호스트 문단의 텍스트는
                    // layout_wrap_around_paras (자가 wrap 경로) 가 처리한다. PartialParagraph
                    // 측에서 같은 paragraph 를 layout_partial_paragraph 로 다시 호출하면
                    // 호스트 텍스트 + 인라인 수식이 중복 emit 됨 (#301 회귀).
                    // FullParagraph 경로 (`is_wrap_host` 가드, layout.rs:1639) 와 동일한 처리.
                    let is_wrap_host = para.controls.iter().any(|c| {
                        if let Control::Table(t) = c {
                            !t.common.treat_as_char
                                && matches!(
                                    t.common.text_wrap,
                                    crate::model::shape::TextWrap::Square
                                )
                        } else {
                            false
                        }
                    });
                    if is_wrap_host {
                        return (y_offset, false);
                    }

                    // TAC 블록 표 문단의 post-text PP: 텍스트가 공백만이면 건너뜀
                    // (Table PageItem에서 이미 y_offset이 결정됨)
                    if prev_tac_seg_applied {
                        let seg_width = effective_tac_segment_width_hu(
                            para,
                            px_to_hwpunit(col_area.width, self.dpi),
                        );
                        let has_tac_block = para.controls.iter().any(|c| {
                            matches!(c, Control::Table(t) if t.common.treat_as_char
                                && !crate::renderer::height_measurer::is_tac_table_inline(
                                    t, seg_width, &para.text, &para.controls))
                        });
                        if has_tac_block {
                            let pp_text_only_ws = if let Some(comp) = composed.get(*para_index) {
                                comp.lines[*start_line..*end_line].iter().all(|line| {
                                    line.runs.iter().all(|r| {
                                        r.text.chars().all(|c| {
                                            c.is_whitespace() || c <= '\u{001F}' || c == '\u{FFFC}'
                                        })
                                    })
                                })
                            } else {
                                false
                            };
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
                            composed.get(*para_index),
                            para,
                            styles,
                            outline_numbering_id,
                        );
                        // numbered가 있으면 composed 업데이트는 불가하므로
                        // layout_partial_paragraph에 직접 전달
                        numbered.or_else(|| composed.get(*para_index).cloned())
                    } else {
                        composed.get(*para_index).cloned()
                    };
                    // [Issue #677] 같은 paragraph 의 TAC 표를 선행한 PP 는 y_offset 이
                    // 이미 표 바닥까지 누적된 상태로 진입한다. 그러나 HWP IR 는 line 1 의
                    // lh 에 표 높이를 인코딩 (table 가 line 1 안의 인라인 객체) 하므로
                    // PP 의 y 를 LineSeg.vpos 정합 위치로 리셋하지 않으면 표 높이만큼
                    // 이중 누적 (LAYOUT_OVERFLOW). 조건 가드 3개로 좁게 발동:
                    //   1) start_line > 0 (문단 첫 PP 미적용)
                    //   2) para 가 TAC 표 보유 (treat_as_char=true)
                    //   3) para_start_y 등록 (Table item 선행 처리됨 → 같은 column)
                    let pp_y_in = if *start_line > 0
                        && para
                            .controls
                            .iter()
                            .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                        && para_start_y.contains_key(para_index)
                    {
                        if let (Some(seg), Some(seg0), Some(para_top)) = (
                            para.line_segs.get(*start_line),
                            para.line_segs.first(),
                            para_start_y.get(para_index).copied(),
                        ) {
                            para_top + hwpunit_to_px(seg.vertical_pos - seg0.vertical_pos, self.dpi)
                        } else {
                            y_offset
                        }
                    } else {
                        y_offset
                    };
                    let pp_y_out = self.layout_partial_paragraph(
                        tree,
                        col_node,
                        para,
                        comp.as_ref(),
                        styles,
                        col_area,
                        pp_y_in,
                        *start_line,
                        *end_line,
                        page_content.section_index,
                        *para_index,
                        None,
                        Some(bin_data_content),
                        ctx.wrap_anchors.get(para_index),
                    );
                    // y_offset 누적: Table item 의 누적값 (표 바닥) 과 PP 자연 종료값
                    // 중 최대로 갱신. 표 + 라인 영역의 시각 바닥을 후속 item 에 정확 전파.
                    y_offset = y_offset.max(pp_y_out);
                }
            }
            PageItem::Table {
                para_index,
                control_index,
            } => {
                return self.layout_table_item(
                    tree,
                    col_node,
                    paper_images,
                    para_start_y,
                    para_float_lanes,
                    visible_float_exclusions,
                    *para_index,
                    *control_index,
                    &ctx,
                    y_offset,
                );
            }
            PageItem::PartialTable {
                para_index,
                control_index,
                start_row,
                end_row,
                is_continuation,
                start_cut,
                end_cut,
                is_block_split,
            } => {
                y_offset = self.layout_partial_table_item(
                    tree,
                    col_node,
                    para_start_y,
                    *para_index,
                    *control_index,
                    *start_row,
                    *end_row,
                    *is_continuation,
                    start_cut,
                    end_cut,
                    *is_block_split,
                    &ctx,
                    y_offset,
                );
            }
            PageItem::Shape {
                para_index,
                control_index,
            } => {
                y_offset = self.layout_shape_item(
                    tree,
                    col_node,
                    paper_images,
                    para_start_y,
                    para_inline_state,
                    *para_index,
                    *control_index,
                    &ctx,
                    y_offset,
                );
            }
            PageItem::EndnoteSeparator {
                separator_length,
                margin_above,
                margin_below,
                line_type,
                line_width,
                color,
            } => {
                y_offset = self.layout_endnote_separator_item(
                    tree,
                    col_node,
                    ctx.col_area,
                    y_offset,
                    *separator_length,
                    *margin_above,
                    *margin_below,
                    *line_type,
                    *line_width,
                    *color,
                );
            }
        }
        (y_offset, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_endnote_separator_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        col_area: &LayoutRect,
        mut y_offset: f64,
        separator_length: i32,
        margin_above: i16,
        margin_below: i16,
        line_type: u8,
        line_width_raw: u8,
        color: crate::model::ColorRef,
    ) -> f64 {
        y_offset += hwpunit_to_px(margin_above as i32, self.dpi);
        let has_separator = line_type != 0 && line_width_raw != 0;
        let line_width = if has_separator {
            let line_width = border_width_to_px(line_width_raw).max(0.5);
            let sep_length = if separator_length > 0 {
                hwpunit_to_px(separator_length as i32, self.dpi).min(col_area.width)
            } else {
                col_area.width / 3.0
            };
            let line_id = tree.next_id();
            let line_node = RenderNode::new(
                line_id,
                RenderNodeType::Line(LineNode::new(
                    col_area.x,
                    y_offset,
                    col_area.x + sep_length,
                    y_offset,
                    LineStyle {
                        color,
                        width: line_width,
                        dash: StrokeDash::Solid,
                        ..Default::default()
                    },
                )),
                BoundingBox::new(
                    col_area.x,
                    y_offset - line_width / 2.0,
                    sep_length,
                    line_width,
                ),
            );
            col_node.children.push(line_node);
            line_width
        } else {
            0.0
        };
        y_offset + line_width + hwpunit_to_px(margin_below as i32, self.dpi)
    }

    /// Table PageItem 레이아웃 (layout_column_item에서 분리)
    #[allow(clippy::too_many_arguments)]
    fn layout_table_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        para_float_lanes: &mut ParaFloatLanes,
        visible_float_exclusions: &mut Vec<VisibleFloatExclusion>,
        para_index: usize,
        control_index: usize,
        ctx: &ColumnItemCtx,
        mut y_offset: f64,
    ) -> (f64, bool) {
        let ColumnItemCtx {
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            measured_tables,
            layout,
            col_area,
            outline_numbering_id,
            multi_col_width,
            prev_tac_seg_applied,
            wrap_around_paras,
            wrap_anchors,
            ..
        } = ctx;
        // 표 앵커 문단의 y 위치 등록
        // TAC 표: 이전 TAC가 y_offset을 진행시킨 경우 갱신 (같은 문단 TAC+블록 구조)
        // 비-TAC 표: 문단 시작 y를 유지 (각 표가 독립적으로 vert offset 기준 배치)
        let is_current_tac = paragraphs
            .get(para_index)
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
            let is_tac = para
                .controls
                .get(control_index)
                .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                .unwrap_or(false);
            let is_current_empty_para_float = para
                .controls
                .get(control_index)
                .map(|c| {
                    matches!(
                        c,
                        Control::Table(t)
                            if is_para_topbottom_float(&t.common) && !para_has_visible_text(para)
                    )
                })
                .unwrap_or(false);
            let is_current_visible_para_float = para
                .controls
                .get(control_index)
                .map(|c| {
                    matches!(
                        c,
                        Control::Table(t)
                            if is_para_topbottom_float(&t.common)
                                && para_has_non_whitespace_text(para)
                    )
                })
                .unwrap_or(false);
            let is_first_empty_para_float_control = is_current_empty_para_float
                && para.controls.iter().position(|c| {
                    matches!(
                        c,
                        Control::Table(t)
                            if is_para_topbottom_float(&t.common)
                                && !para_has_visible_text(para)
                    )
                }) == Some(control_index);
            // ── 표 위 간격 ──
            {
                let comp = composed.get(para_index);
                let ps_id = comp
                    .map(|c| c.para_style_id as usize)
                    .unwrap_or(para.para_shape_id as usize);
                let is_column_top = (y_offset - col_area.y).abs() < 1.0;
                if is_tac {
                    if !prev_tac_seg_applied {
                        let outer_margin_top_px =
                            if let Some(Control::Table(t)) = para.controls.get(control_index) {
                                hwpunit_to_px(t.outer_margin_top as i32, self.dpi)
                            } else {
                                0.0
                            };
                        if !is_column_top {
                            let spacing_before = styles
                                .para_styles
                                .get(ps_id)
                                .map(|ps| ps.spacing_before)
                                .unwrap_or(0.0);
                            if spacing_before > 0.0 {
                                y_offset += spacing_before;
                            }
                        }
                        if outer_margin_top_px > 0.0 {
                            y_offset += outer_margin_top_px;
                        }
                    }
                } else if !is_current_empty_para_float {
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
                let host_is_not_square =
                    if let Some(Control::Table(ht)) = para.controls.get(control_index) {
                        !matches!(ht.common.text_wrap, crate::model::shape::TextWrap::Square)
                    } else {
                        true
                    };
                if host_is_not_square {
                    let has_real_text =
                        para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                    if has_real_text {
                        if let Some(comp) = composed.get(para_index) {
                            let text_start_line = comp.lines.iter().position(|line| {
                                line.runs.iter().any(|r| {
                                    r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                })
                            });
                            if let Some(start_line) = text_start_line {
                                let text_end_line = comp
                                    .lines
                                    .iter()
                                    .rposition(|line| {
                                        line.runs.iter().any(|r| {
                                            r.text
                                                .chars()
                                                .any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                        })
                                    })
                                    .map(|i| i + 1)
                                    .unwrap_or(comp.lines.len());
                                para_start_y.insert(para_index, y_offset);
                                let _text_y_end = self.layout_partial_paragraph(
                                    tree,
                                    col_node,
                                    para,
                                    Some(comp),
                                    styles,
                                    col_area,
                                    y_offset,
                                    start_line,
                                    text_end_line,
                                    page_content.section_index,
                                    para_index,
                                    *multi_col_width,
                                    Some(bin_data_content),
                                    wrap_anchors.get(&para_index),
                                );
                            }
                        }
                    }
                }
            }
            // ── 표 레이아웃 ──
            let mut tac_seg_applied = false;
            let tac_table_y_before = y_offset; // Task #9: 표 렌더 전 y 보존
            let mut para_float_lane_info: Option<(f64, f64, f64, f64, f64)> = None;
            if let Some(Control::Table(t)) = para.controls.get(control_index) {
                let raw_mt = measured_tables
                    .iter()
                    .find(|mt| mt.para_index == para_index && mt.control_index == control_index);
                let fitted_visible_mt = if is_current_visible_para_float {
                    raw_mt.map(|measured| {
                        fit_measured_table_to_declared_height(measured, t, self.dpi)
                    })
                } else {
                    None
                };
                let mt = fitted_visible_mt.as_ref().or(raw_mt);
                let para_style = styles.para_styles.get(para.para_shape_id as usize);
                let alignment = para_style.map(|s| s.alignment).unwrap_or(Alignment::Left);
                let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let indent = para_style.map(|s| s.indent).unwrap_or(0.0);
                let effective_margin = if indent > 0.0 {
                    margin_left + indent
                } else {
                    margin_left
                };
                let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let table_y_before = y_offset;
                let tbl_is_square =
                    matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square);
                // インラインTAC表: paragraph_layoutで計算された位置を使用
                let inline_pos = if is_tac {
                    tree.get_inline_shape_position(
                        page_content.section_index,
                        para_index,
                        control_index,
                        None,
                    )
                } else {
                    None
                };
                // [Task #1470 Stage 2] paragraph_layout가 인라인 TAC 표를 이미
                // 렌더하고 좌표를 등록한 경우, PageItem 표 경로에서는 본문 흐름
                // advance만 보존하고 같은 컨트롤을 다시 그리지 않는다.
                let tac_already_rendered_inline = is_tac && inline_pos.is_some();
                let tbl_inline_x = if let Some((ix, _)) = inline_pos {
                    Some(ix)
                } else if !is_tac
                    && tbl_is_square
                    && matches!(t.common.horz_rel_to, crate::model::shape::HorzRelTo::Para)
                {
                    // [Issue #480 / #590] horz_rel_to=Para 인 Square wrap 표만 paragraph 영역
                    // (col_area + margin) 기준으로 정렬. horz_rel_to=Column/Page/Paper 는
                    // compute_table_x_position 의 기본 분기에서 명세대로 처리한다.
                    // (Task #295: halign=Right 표가 좌측에 잘못 배치되는 문제 수정)
                    let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
                    let area_x = col_area.x + effective_margin;
                    let area_w = (col_area.width - effective_margin - margin_right).max(0.0);
                    let x = match t.common.horz_align {
                        crate::model::shape::HorzAlign::Right
                        | crate::model::shape::HorzAlign::Outside => {
                            area_x + (area_w - tbl_w).max(0.0)
                        }
                        crate::model::shape::HorzAlign::Center => {
                            area_x + (area_w - tbl_w).max(0.0) / 2.0
                        }
                        _ => area_x,
                    };
                    Some(x)
                } else if is_tac {
                    // TAC 문단에 PageItem::FullParagraph 가 발행되지 않아
                    // paragraph_layout 가 호출되지 않는 케이스(선행 공백만 있는 TAC 표 등):
                    // composed.lines[0] 의 runs 에서 TAC 이전 텍스트 폭을 직접
                    // 합산해 표 x 좌표에 반영한다. inline_shape_position 미세팅 상태에서
                    // 기본값 col_area.x(body_left) 으로 붕괴되는 현상 방지.
                    // [Issue #842 #2] 문단이 여러 줄이고 line 0 에 *실제 텍스트*(필러/공백/
                    // 오브젝트마커가 아닌 가시 글자)가 있으면 — 예: line 0 = "파일" 텍스트 +
                    // line 1 = 자체 줄의 헤더 바 표 — 표는 line 0 텍스트 *다음* 이 아니라
                    // 자체 줄 좌측에서 시작하므로 leading = 0. line 0 이 HWP TAC 필러(U+F081C)/
                    // 공백뿐인 경우(예: 복학원서.hwp pi=16, 한컴이 표 폭만큼 필러를 채워
                    // 줄바꿈시킨 케이스)는 종전대로 compute_tac_leading_width 사용.
                    // "실제 텍스트" 판정은 alphanumeric(한글 음절·라틴·숫자·한자 등 Letter/Number)
                    // 만 인정 — HWP TAC 필러(U+F081C 등 PUA), 공백, 오브젝트마커는 PUA/공백이라
                    // 자동 제외된다 (복학원서.hwp pi=16 line 0 = U+F081C/U+F012B 필러 99개 → 제외).
                    let line0_has_real_text = composed
                        .get(para_index)
                        .map(|c| {
                            c.lines.len() > 1
                                && c.lines
                                    .first()
                                    .map(|l0| {
                                        l0.runs
                                            .iter()
                                            .any(|r| r.text.chars().any(|ch| ch.is_alphanumeric()))
                                    })
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    let leading = if line0_has_real_text {
                        0.0
                    } else {
                        composed
                            .get(para_index)
                            .map(|c| compute_tac_leading_width(c, control_index, styles))
                            .unwrap_or(0.0)
                    };
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
                let tac_detached_line_shift =
                    if is_tac && inline_pos.is_none() && table_has_detached_para_flow_object(t) {
                        para.line_segs
                            .first()
                            .filter(|seg| seg.vertical_pos > 0)
                            .map(|seg| hwpunit_to_px(seg.vertical_pos, self.dpi))
                            .unwrap_or(0.0)
                    } else {
                        0.0
                    };
                // vert=Paper로 body_area 위에 배치되는 표
                // 본문 영역 외부(머리말/꼬리말 자리)에 그려지는 페이지/페이퍼 앵커 TopAndBottom 표는
                // 본문 흐름의 y_offset을 진행시키지 않고 out-of-flow로 paper_images에 렌더한다.
                // (Task #295: vert=Page valign=Bottom 푸터 표가 좌단 y_offset을 본문 하단으로
                //  끌어올려 후속 콘텐츠를 깨뜨리는 문제 수정 — Paper만 다루던 기존 분기를 Page까지 확장)
                let renders_outside_body = !is_tac
                    && matches!(
                        t.common.vert_rel_to,
                        crate::model::shape::VertRelTo::Paper
                            | crate::model::shape::VertRelTo::Page
                    )
                    && matches!(
                        t.common.text_wrap,
                        crate::model::shape::TextWrap::TopAndBottom
                    )
                    && {
                        let tbl_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                        let v_off = hwpunit_to_px(t.common.vertical_offset as i32, self.dpi);
                        let tbl_y = match t.common.vert_align {
                            crate::model::shape::VertAlign::Top
                            | crate::model::shape::VertAlign::Inside => v_off,
                            crate::model::shape::VertAlign::Center => {
                                (layout.page_height - tbl_h) / 2.0 + v_off
                            }
                            crate::model::shape::VertAlign::Bottom
                            | crate::model::shape::VertAlign::Outside => {
                                layout.page_height - tbl_h - v_off
                            }
                        };
                        // 표 상단이 본문 위(머리말)이거나, 표 하단이 본문 아래(꼬리말)에 걸치는 경우
                        let body_bottom = layout.body_area.y + layout.body_area.height;
                        tbl_y < layout.body_area.y || tbl_y + tbl_h > body_bottom
                    };
                if is_current_empty_para_float && !renders_outside_body {
                    let width_px = hwpunit_to_px(signed_hwpunit(t.common.width), self.dpi);
                    if width_px > 0.0 {
                        let placement_ctx = FloatPlacementContext::new(**col_area)
                            .with_body_area(layout.body_area)
                            .with_paper_width(layout.page_width)
                            .with_host_margins(effective_margin, margin_right);
                        let (x_start, x_end) =
                            horizontal_range(&t.common, width_px, placement_ctx, self.dpi);
                        let v_offset_px =
                            hwpunit_to_px(signed_hwpunit(t.common.vertical_offset), self.dpi);
                        let raw_top = (para_y_for_table + v_offset_px).max(para_y_for_table);
                        let lane_top = para_float_lanes
                            .entry(para_index)
                            .or_default()
                            .pushed_top(x_start, x_end, raw_top);
                        para_float_lane_info = Some((x_start, x_end, raw_top, lane_top, y_offset));
                    }
                }
                let mut table_visual_shift = 0.0;
                let mut table_y_end = y_offset;
                if renders_outside_body {
                    let tmp_id = tree.next_id();
                    let mut tmp_node = RenderNode::new(
                        tmp_id,
                        RenderNodeType::Column(0),
                        layout_rect_to_bbox(&layout.body_area),
                    );
                    let _table_y_end = self.layout_table(
                        tree,
                        &mut tmp_node,
                        t,
                        page_content.section_index,
                        styles,
                        *outline_numbering_id,
                        &layout.body_area,
                        y_offset,
                        bin_data_content,
                        mt,
                        0,
                        Some((para_index, control_index)),
                        alignment,
                        None,
                        effective_margin,
                        margin_right,
                        tbl_inline_x,
                        None,
                        Some(para_y_for_table),
                        false,
                        false,
                    );
                    let layer =
                        Self::render_layer_from_common(&t.common, para_index, control_index);
                    Self::push_layered_paper_children(paper_images, &mut tmp_node, layer);
                } else {
                    let square_anchor_y = if !is_tac && tbl_is_square {
                        square_wrap_table_line_anchor_y(para, t, para_y_for_table, self.dpi)
                    } else {
                        None
                    };
                    let visible_outer_top_px = if is_current_visible_para_float {
                        hwpunit_to_px(t.outer_margin_top as i32, self.dpi)
                    } else {
                        0.0
                    };
                    let table_y_start = if let Some((_, _, _, lane_top, _)) = para_float_lane_info {
                        lane_top
                    } else if let Some((_, iy)) = inline_pos {
                        iy
                    } else if is_current_visible_para_float {
                        let v_off =
                            hwpunit_to_px(signed_hwpunit(t.common.vertical_offset), self.dpi);
                        if self.is_hwpx_source.get() && v_off <= 0.0 {
                            let flow_at_para_start =
                                (table_y_before - para_y_for_table).abs() < 0.5;
                            table_y_before
                                + if flow_at_para_start {
                                    visible_outer_top_px
                                } else {
                                    0.0
                                }
                                + v_off.max(0.0)
                        } else if v_off < 0.0 {
                            para_y_for_table + visible_outer_top_px + v_off
                        } else {
                            para_y_for_table
                        }
                    } else if let Some(anchor_y) = square_anchor_y {
                        table_visual_shift = (anchor_y - y_offset).max(0.0);
                        anchor_y
                    } else if tac_detached_line_shift > 0.0 {
                        y_offset + tac_detached_line_shift
                    } else {
                        y_offset
                    };
                    // [Issue #1535] visible-host co-anchored float 표도 문단(아래 visible_float_exclusions
                    // 소비부)과 동일하게 선행 float 표가 점유한 세로 영역을 벗어나 배치되어야 한다.
                    // 표 배치는 기존에 exclusion 을 push 만 하고 consult 하지 않아, 연속 문단의
                    // float 표가 앞 문단 float 표 위에 겹쳐 그려졌다. 표의 자연 상단
                    // (para_y + outer_margin + v_offset, = compute_table_y_position 의 raw_y)이
                    // 활성 exclusion 영역 안에서 시작하면 그 영역 하단으로 내려 시작점을 끌어올린다.
                    // compute_table_y_position 이 raw_y.max(y_start) 로 클램프하므로 table_y_start
                    // (= y_start)를 올리면 표가 해당 영역 아래로 밀린다.
                    let table_y_start = if is_para_topbottom_float(&t.common)
                        && !visible_float_exclusions.is_empty()
                    {
                        let v_off =
                            hwpunit_to_px(signed_hwpunit(t.common.vertical_offset), self.dpi);
                        // 빈-host float 은 base table_y_start(=흐름 위치)가 곧 자연 상단이고,
                        // visible host 는 para_y+outer+v_off 가 자연 상단이다. 둘 중 큰 값을
                        // 자연 상단으로 보아 선행 exclusion 밴드 안에서 시작하면 그 하단으로 내린다.
                        let natural_top = table_y_start
                            .max(para_y_for_table + visible_outer_top_px + v_off.max(0.0));
                        let mut floor = table_y_start;
                        for zone in visible_float_exclusions.iter() {
                            if natural_top + 0.5 >= zone.top && natural_top < zone.bottom {
                                // 빈-host(text 없는) float 은 자기 offset 이 선행 exclusion 에
                                // 흡수되어 표끼리 붙는다. zone 하단 아래로 그 offset 만큼 띄워
                                // 복원한다(한컴: 표-표 간격 = 후행 표 offset). visible host 는
                                // part 3(host-title-line)이 간격을 처리하므로 여기선 zone 하단만.
                                let restore = if is_current_visible_para_float {
                                    0.0
                                } else {
                                    v_off.max(0.0)
                                };
                                floor = floor.max(zone.bottom + restore);
                            }
                        }
                        floor
                    } else {
                        table_y_start
                    };
                    // [Issue #1549] visible-host 의 양수 offset float 표는 host 텍스트(섹션 제목)가
                    // line 0 으로 그려지는 줄 *아래*에 와야 한다(한컴: 제목 위, 표 아래). 제목과 표는
                    // 같은 문단 앵커를 공유하고 선행 float exclusion 으로 함께 같은 y 까지 밀리므로,
                    // 작은 offset 은 흡수되어 둘이 겹친다. 제목이 그려지는 위치(선행 exclusion 아래로
                    // 밀린 para 흐름) + 제목 줄높이 아래로 표를 내려 겹침을 막는다. 큰 offset 으로 표가
                    // 이미 더 아래면 max 라 영향 없다.
                    let table_y_start = if is_current_visible_para_float
                        && signed_hwpunit(t.common.vertical_offset) > 0
                    {
                        let host_line_px = para
                            .line_segs
                            .first()
                            .map(|s| hwpunit_to_px(s.line_height, self.dpi))
                            .unwrap_or(0.0);
                        let mut title_flow_y = para_y_for_table;
                        for zone in visible_float_exclusions.iter() {
                            if title_flow_y + 0.5 >= zone.top && title_flow_y < zone.bottom {
                                title_flow_y = title_flow_y.max(zone.bottom);
                            }
                        }
                        table_y_start.max(title_flow_y + host_line_px + visible_outer_top_px)
                    } else {
                        table_y_start
                    };
                    let allow_para_top_bleed = is_current_visible_para_float
                        && signed_hwpunit(t.common.vertical_offset) < 0;
                    let table_visual_height = mt
                        .map(|m| m.total_height)
                        .filter(|h| *h > 0.0)
                        .unwrap_or_else(|| hwpunit_to_px(t.common.height as i32, self.dpi));
                    let table_visual_end = if tac_already_rendered_inline {
                        table_y_start + table_visual_height
                    } else {
                        self.layout_table(
                            tree,
                            col_node,
                            t,
                            page_content.section_index,
                            styles,
                            *outline_numbering_id,
                            col_area,
                            table_y_start,
                            bin_data_content,
                            mt,
                            0,
                            Some((para_index, control_index)),
                            alignment,
                            None,
                            effective_margin,
                            margin_right,
                            tbl_inline_x,
                            None,
                            Some(para_y_for_table + visible_outer_top_px),
                            allow_para_top_bleed,
                            false,
                        )
                    };
                    if is_tac {
                        let marker_x = tbl_inline_x.unwrap_or(col_area.x + effective_margin);
                        tree.set_inline_shape_position(
                            page_content.section_index,
                            para_index,
                            control_index,
                            None,
                            marker_x,
                            table_y_start,
                        );
                    }
                    if is_first_empty_para_float_control && !is_tac {
                        let marker_x = tbl_inline_x.unwrap_or(col_area.x + effective_margin);
                        // FullParagraph에서 빈 줄 진행을 생략한 대신, 표와 같은 줄에
                        // host 문단부호를 렌더링한다. 표 뒤 빈 문단은 그대로 남아
                        // 아래쪽 탈출 위치를 제공한다.
                        push_empty_para_end_mark(
                            tree,
                            col_node,
                            para,
                            styles,
                            page_content.section_index,
                            para_index,
                            marker_x,
                            table_y_start,
                            self.dpi,
                        );
                    }
                    table_y_end = table_visual_end;
                    // [Task #1841] 자리차지(TopAndBottom) 표 아래에서 host 본문이 재개될 때
                    // 표의 바깥 여백 bottom 을 띄운다 (한글 실측: 표 하단→첫 줄 gap =
                    // rhwp 10.2pt + outer_bottom 8.5pt = 한글 18.7pt, 결재문서 헤더 표
                    // 852HU 계열 — 동작소방서 36385142/관악소방서 36389312).
                    let visible_outer_bottom_px = if is_current_visible_para_float {
                        hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
                    } else {
                        0.0
                    };
                    y_offset = if is_current_visible_para_float {
                        if signed_hwpunit(t.common.vertical_offset) > 0 {
                            table_y_before
                        } else if self.is_hwpx_source.get() {
                            let following_non_positive =
                                has_following_non_positive_visible_float(para, control_index);
                            let inter_float_gap = if following_non_positive {
                                para_line_spacing_px(para, self.dpi)
                            } else {
                                0.0
                            };
                            table_visual_end + inter_float_gap + visible_outer_bottom_px
                        } else {
                            table_y_before.max(table_visual_end + visible_outer_bottom_px)
                        }
                    } else if table_visual_shift > 0.0 {
                        (table_visual_end - table_visual_shift).max(table_y_before)
                    } else {
                        table_visual_end
                    };
                    if is_current_visible_para_float
                        && signed_hwpunit(t.common.vertical_offset) > 0
                        && table_visual_height > 0.0
                    {
                        let table_visual_top = table_visual_end - table_visual_height;
                        if table_visual_end > table_visual_top + 0.5 {
                            // exclusion 하단을 표의 outer_margin_bottom 만큼 늘려, 다음 섹션
                            // 제목(이 zone 을 consult)이 표 아래로 그 여백만큼 띄워지게 한다
                            // (한컴: 섹션 표와 다음 섹션 제목 사이 간격 = 표 아래 외곽여백).
                            let margin_bottom_px =
                                hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi);
                            visible_float_exclusions.push(VisibleFloatExclusion {
                                top: table_visual_top,
                                bottom: table_visual_end + margin_bottom_px,
                                owner_para: para_index,
                            });
                        }
                    }
                }
                // [Task #1046 Stage 3 Class B] 표 실제 콘텐츠 하단 기록 — 이후 더해지는
                // 표 뒤 trailing 간격(tac 줄간격/표 아래 간격)을 제외한 값. overflow 검출이
                // 페이지 바닥의 후행 간격을 콘텐츠 초과로 오판하지 않도록 한다.
                self.last_item_content_bottom.set(table_y_end);
                // [Task #1046 Stage 3 Class B 진단] 통째 표 렌더 분해 — 표 시작/끝,
                // para 시작, host before. 동작 불변(게이트).
                if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
                    eprintln!(
                        "WHOLE_TABLE_Y: pi={} sec={} tac={} table_y_start={:.1} table_y_end={:.1} table_h={:.1} para_y={:.1} table_y_before={:.1}",
                        para_index, page_content.section_index, is_tac,
                        if let Some((_, iy)) = inline_pos { iy } else { table_y_before },
                        table_y_end,
                        table_y_end - (if let Some((_, iy)) = inline_pos { iy } else { table_y_before }),
                        para_y_for_table, table_y_before,
                    );
                }
                // ── TAC 표: 줄간격 처리 ──
                // layout_table 반환값(표 하단)에 line_spacing을 더하여 다음 표 시작 y 결정
                if is_tac {
                    let seg_idx = control_index;
                    let tac_count_total = para
                        .controls
                        .iter()
                        .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                        .count();
                    let tac_idx_current = para
                        .controls
                        .iter()
                        .take(control_index + 1)
                        .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                        .count();
                    // TAC 표 사이에 non-TAC 표가 있는지 확인
                    let has_non_tac_between = para
                        .controls
                        .iter()
                        .skip(control_index + 1)
                        .take_while(|c| !matches!(c, Control::Table(t) if t.common.treat_as_char))
                        .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char));
                    if tac_idx_current < tac_count_total && !has_non_tac_between {
                        // 다음 TAC가 있으면: vpos 차이분만 추가 (= line_spacing)
                        // 이후 tac_seg_applied 경로의 line_spacing 추가를 스킵하기 위해
                        // 여기서 직접 return (spacing_after/line_spacing 이중 적용 방지)
                        if let (Some(seg), Some(next_seg)) =
                            (para.line_segs.get(seg_idx), para.line_segs.get(seg_idx + 1))
                        {
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
                            let max_correction =
                                hwpunit_to_px(seg.line_spacing * 2 + 1000, self.dpi);
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
                let table_is_square =
                    matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square);
                if !is_tac && table_is_square {
                    let wrap_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                    let wrap_sw = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                    let wrap_text_x = col_area.x + hwpunit_to_px(wrap_cs, self.dpi);
                    let wrap_text_width = hwpunit_to_px(wrap_sw, self.dpi);
                    // [Task #1745] 텍스트 혼합 anchor: 후속 어울림 문단 띠는 표 geometry 로.
                    let (strip_x, strip_width) =
                        crate::renderer::text_anchor_square_table_strip(para)
                            .map(|(cs, sw)| {
                                (
                                    col_area.x + hwpunit_to_px(cs, self.dpi),
                                    hwpunit_to_px(sw, self.dpi),
                                )
                            })
                            .unwrap_or((wrap_text_x, wrap_text_width));
                    // Task #463: 인라인 floating 표 우측 x 계산 (paragraph border box 확장용).
                    // table_layout::compute_table_x_position 와 동일 공식.
                    let tbl_x_right = compute_square_wrap_tbl_x_right(t, col_area, self.dpi);
                    self.layout_wrap_around_paras(
                        tree,
                        col_node,
                        paragraphs,
                        composed,
                        styles,
                        col_area,
                        page_content.section_index,
                        para_index,
                        wrap_around_paras,
                        table_y_before,
                        y_offset,
                        wrap_text_x,
                        wrap_text_width,
                        strip_x,
                        strip_width,
                        true,
                        0.0,
                        bin_data_content,
                        Some(tbl_x_right),
                    );
                    // [#1218] 어울림(Square) 호스트 본문이 표보다 길면 커서를 본문 하단까지
                    // 전진시킨다. 그렇지 않으면 다음 단락이 표 하단(=현재 y_offset)에서 시작해
                    // 표보다 아래로 내려온 본문 줄과 겹친다(3-09월_교육_통합_2023 4쪽 문26).
                    // 본문 ≤ 표 인 기존 다수 케이스는 host_text_bottom ≤ y_offset 이라 불변.
                    if let Some(comp) = composed.get(para_index) {
                        let mut text_h = 0.0;
                        let mut last_ls = 0.0;
                        for line in &comp.lines {
                            let lh = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            text_h += lh + ls;
                            last_ls = ls;
                        }
                        // 마지막 줄의 trailing line_spacing 은 본문 하단에서 제외(height_for_fit 정합).
                        let host_text_bottom = table_y_before + (text_h - last_ls).max(0.0);
                        if host_text_bottom > y_offset {
                            y_offset = host_text_bottom;
                        }
                    }
                }
            }
            // ── 표 아래 간격 ──
            // out-of-flow로 그려진 표(머리말/꼬리말 자리)는 본문 흐름 간격을 추가하지 않는다.
            let is_outside_body = if let Some(Control::Table(t)) = para.controls.get(control_index)
            {
                !t.common.treat_as_char
                    && matches!(
                        t.common.vert_rel_to,
                        crate::model::shape::VertRelTo::Paper
                            | crate::model::shape::VertRelTo::Page
                    )
                    && matches!(
                        t.common.text_wrap,
                        crate::model::shape::TextWrap::TopAndBottom
                    )
                    && {
                        let tbl_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                        let v_off = hwpunit_to_px(t.common.vertical_offset as i32, self.dpi);
                        let tbl_y = match t.common.vert_align {
                            crate::model::shape::VertAlign::Top
                            | crate::model::shape::VertAlign::Inside => v_off,
                            crate::model::shape::VertAlign::Center => {
                                (layout.page_height - tbl_h) / 2.0 + v_off
                            }
                            crate::model::shape::VertAlign::Bottom
                            | crate::model::shape::VertAlign::Outside => {
                                layout.page_height - tbl_h - v_off
                            }
                        };
                        let body_bottom = layout.body_area.y + layout.body_area.height;
                        tbl_y < layout.body_area.y || tbl_y + tbl_h > body_bottom
                    }
            } else {
                false
            };
            if !tac_seg_applied && !is_outside_body && !is_current_visible_para_float {
                let comp = composed.get(para_index);
                let para_style_id = comp
                    .map(|c| c.para_style_id as usize)
                    .unwrap_or(para.para_shape_id as usize);
                if let Some(para_style) = styles.para_styles.get(para_style_id) {
                    if para_style.spacing_after > 0.0 {
                        y_offset += para_style.spacing_after;
                    }
                }
                // [Task #1147 v2] 빈 앵커 TopAndBottom 비-TAC 표는 다음
                // 항목이 일반 문단일 때 host_line_spacing=0 으로 맞춘다. 단, [Task
                // #1133] 다음 항목도 빈 앵커 TopAndBottom 표이면 해당 line_spacing 이
                // 표-표 사이 간격이므로 HWP처럼 보존한다.
                let next_is_empty_topbottom_table_anchor = paragraphs
                    .get(para_index + 1)
                    .map(para_is_empty_topbottom_table_anchor)
                    .unwrap_or(false);
                let suppress_empty_anchor_spacing =
                    is_current_empty_para_float && !next_is_empty_topbottom_table_anchor;
                if let Some(seg) = para.line_segs.last() {
                    let gap = if suppress_empty_anchor_spacing {
                        0
                    } else if is_current_empty_para_float {
                        seg.line_spacing.max(0)
                    } else if seg.line_spacing > 0 {
                        seg.line_spacing
                    } else {
                        seg.line_height
                    };
                    if gap > 0 {
                        y_offset += hwpunit_to_px(gap, self.dpi);
                    }
                }
            }
            if let Some((x_start, x_end, raw_top, lane_top, global_y_before)) = para_float_lane_info
            {
                let reserved_height = (y_offset - lane_top).max(0.0);
                let lanes = para_float_lanes.entry(para_index).or_default();
                lanes.place(x_start, x_end, raw_top, reserved_height);
                let lane_flow_bottom = if is_current_empty_para_float {
                    // Empty-anchor TopAndBottom tables can encode a visual
                    // vertical offset separately from the flow height measured
                    // by pagination. Keep the table painted at lane_top, but
                    // advance following items by the reserved table height only.
                    global_y_before + reserved_height
                } else {
                    lanes.max_bottom()
                };
                y_offset = global_y_before.max(lane_flow_bottom);
            }
            if tac_seg_applied {
                // [hwpdf cycle#3 — 폴백 한정] control_index 는 컨트롤 배열 인덱스지 줄
                // 인덱스가 아니다. 표 앞 비가시 컨트롤(SectionDef/ColumnDef/책갈피 등)은
                // 줄 seg 를 만들지 않아 get(control_index)=None 이 되며, 이때는 표가 곧
                // 호스트 줄이므로 마지막(=유일) seg 의 줄간격을 적용해야 후속 본문이
                // 위로 당겨지지 않는다.
                //
                // 단, 표 앞에 가시 개체(그림/그리기/표)가 있으면 그 개체들은 별도
                // 경로로 배치되고 다음 문단 위치가 파일 lineseg(vpos=lh+sp 포함)로 이미
                // 확정되므로, 여기서 줄간격을 다시 더하면 이중 적용된다(test_521: 이메일
                // 박스 TAC 표 앞 그림 2개 → 한컴 간격은 호스트 줄간격 제외, gap≈20px).
                // → 폴백은 표 앞 컨트롤이 전부 비가시일 때로 한정한다.
                let only_invisible_before_tac = para.controls
                    [..control_index.min(para.controls.len())]
                    .iter()
                    .all(|c| {
                        !matches!(
                            c,
                            Control::Table(_) | Control::Picture(_) | Control::Shape(_)
                        )
                    });
                let host_seg = para.line_segs.get(control_index).or_else(|| {
                    if only_invisible_before_tac {
                        para.line_segs.last()
                    } else {
                        None
                    }
                });
                if let Some(seg) = host_seg {
                    if seg.line_spacing > 0 {
                        y_offset += hwpunit_to_px(seg.line_spacing, self.dpi);
                    } else if seg.line_spacing < 0 {
                        // 음수 ls (Fixed 줄간격 TAC 표): y를 문단 advance로 리셋 (Task #9)
                        // 표 렌더 높이가 아닌, 일반 문단과 동일한 lh+ls advance 사용
                        let advance =
                            hwpunit_to_px(seg.line_height + seg.line_spacing, self.dpi).max(0.0);
                        y_offset = tac_table_y_before + advance;
                    }
                }
                let comp = composed.get(para_index);
                let ps_id = comp
                    .map(|c| c.para_style_id as usize)
                    .unwrap_or(para.para_shape_id as usize);
                if let Some(ps) = styles.para_styles.get(ps_id) {
                    if ps.spacing_after > 0.0 {
                        y_offset += ps.spacing_after;
                    }
                }
                // [Task #521] TAC 표 outer_margin_bottom 적용 (한컴 명세 정합).
                // layout_partial_table_item:2642-2647 와 동일 처리. lh = cell_h +
                // outer_margin_bottom 으로 한컴이 정의하므로, layout_table 가
                // cell_h 만 advance 한 후 outer_margin_bottom 을 별도 적용해야
                // 다음 paragraph 가 정합 (exam_eng p2 18번 ① 위치 -8 px shortfall).
                let outer_margin_bottom_px =
                    if let Some(Control::Table(t)) = para.controls.get(control_index) {
                        hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
                    } else {
                        0.0
                    };
                if outer_margin_bottom_px > 0.0 {
                    y_offset += outer_margin_bottom_px;
                }
                return (y_offset, true);
            }
            // ── 같은 문단의 인라인 TAC 표 렌더링 ──
            if !is_tac {
                let seg_width =
                    effective_tac_segment_width_hu(para, px_to_hwpunit(col_area.width, self.dpi));
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if ci == control_index {
                        continue;
                    }
                    if let Control::Table(inline_t) = ctrl {
                        if inline_t.common.treat_as_char
                            && crate::renderer::height_measurer::is_tac_table_inline(
                                inline_t,
                                seg_width,
                                &para.text,
                                &para.controls,
                            )
                        {
                            let mt = measured_tables
                                .iter()
                                .find(|m| m.para_index == para_index && m.control_index == ci);
                            let alignment = composed
                                .get(para_index)
                                .map(|c| {
                                    styles
                                        .para_styles
                                        .get(c.para_style_id as usize)
                                        .map(|s| s.alignment)
                                        .unwrap_or(Alignment::Left)
                                })
                                .unwrap_or(Alignment::Left);
                            // paragraph_layout에서 계산된 인라인 좌표 사용
                            let inline_pos = tree.get_inline_shape_position(
                                page_content.section_index,
                                para_index,
                                ci,
                                None,
                            );
                            let (inline_x, inline_y) = if let Some((ix, iy)) = inline_pos {
                                (Some(ix), iy)
                            } else {
                                (None, para_y_for_table)
                            };
                            let tac_new_y = self.layout_table(
                                tree,
                                col_node,
                                inline_t,
                                page_content.section_index,
                                styles,
                                *outline_numbering_id,
                                col_area,
                                inline_y,
                                bin_data_content,
                                mt,
                                0,
                                Some((para_index, ci)),
                                alignment,
                                None,
                                0.0,
                                0.0,
                                inline_x,
                                None,
                                None,
                                false,
                                false,
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
        start_cut: &[usize],
        end_cut: &[usize],
        is_block_split: bool,
        ctx: &ColumnItemCtx,
        mut y_offset: f64,
    ) -> f64 {
        let ColumnItemCtx {
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            measured_tables,
            col_area,
            outline_numbering_id,
            multi_col_width,
            wrap_around_paras,
            wrap_anchors,
            ..
        } = ctx;
        let defer_visible_rowbreak_host_text = paragraphs.get(para_index).and_then(|para| {
            para.controls
                .get(control_index)
                .and_then(|ctrl| match ctrl {
                    Control::Table(t)
                        if !t.common.treat_as_char
                            && is_para_topbottom_float(&t.common)
                            && matches!(
                                t.page_break,
                                crate::model::table::TablePageBreak::RowBreak
                            )
                            && para_has_non_whitespace_text(para) =>
                    {
                        Some(t.row_count as usize)
                    }
                    _ => None,
                })
        });
        // [Task #1755] typeset 이 host 텍스트 줄을 이월 전 쪽에 PartialParagraph 로
        // pre-emit 한 문단은 fragment 쪽 host 렌더(첫 부분/마지막 뒤 모두)를 억제한다.
        let host_pre_emitted = self.pre_emitted_host_paras.borrow().contains(&para_index);
        let render_deferred_rowbreak_host_text_after = !host_pre_emitted
            && defer_visible_rowbreak_host_text.is_some_and(|row_count| {
                is_continuation && end_cut.is_empty() && end_row >= row_count
            });
        // ── 분할 표 첫 부분: 호스트 문단 텍스트 렌더링 ──
        if !is_continuation && defer_visible_rowbreak_host_text.is_none() && !host_pre_emitted {
            if let Some(para) = paragraphs.get(para_index) {
                let is_tac = para
                    .controls
                    .get(control_index)
                    .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    .unwrap_or(false);
                if !is_tac {
                    let has_real_text =
                        para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                    if has_real_text {
                        if let Some(comp) = composed.get(para_index) {
                            let text_start_line = comp.lines.iter().position(|line| {
                                line.runs.iter().any(|r| {
                                    r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                })
                            });
                            if let Some(start_line) = text_start_line {
                                let text_end_line = comp
                                    .lines
                                    .iter()
                                    .rposition(|line| {
                                        line.runs.iter().any(|r| {
                                            r.text
                                                .chars()
                                                .any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                        })
                                    })
                                    .map(|i| i + 1)
                                    .unwrap_or(comp.lines.len());
                                para_start_y.insert(para_index, y_offset);
                                let _text_y_end = self.layout_partial_paragraph(
                                    tree,
                                    col_node,
                                    para,
                                    Some(comp),
                                    styles,
                                    col_area,
                                    y_offset,
                                    start_line,
                                    text_end_line,
                                    page_content.section_index,
                                    para_index,
                                    *multi_col_width,
                                    Some(bin_data_content),
                                    wrap_anchors.get(&para_index),
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
        let pt_mt = measured_tables
            .iter()
            .find(|mt| mt.para_index == para_index && mt.control_index == control_index);
        // 비-TAC 자리차지 표에서 vert offset이 있으면 문단 시작 y 전달.
        // layout_partial_table 내부에서 vert_offset을 적용하므로 이중 적용 방지.
        // [Task #712] HwpUnit=u32 이라 `vertical_offset > 0` 가드는 음수 비트표현
        // (예: -1796 HU = 4294965500u32) 도 통과시킴. signed 비교로 정정.
        let pt_y_start = if let Some(para) = paragraphs.get(para_index) {
            if let Some(Control::Table(t)) = para.controls.get(control_index) {
                if !t.common.treat_as_char
                    && matches!(
                        t.common.text_wrap,
                        crate::model::shape::TextWrap::TopAndBottom
                    )
                    && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                    && (t.common.vertical_offset as i32) > 0
                {
                    para_start_y.get(&para_index).copied().unwrap_or(y_offset)
                } else {
                    y_offset
                }
            } else {
                y_offset
            }
        } else {
            y_offset
        };
        let pt_y_before = y_offset;
        y_offset = self.layout_partial_table(
            tree,
            col_node,
            paragraphs,
            para_index,
            control_index,
            page_content.section_index,
            styles,
            *outline_numbering_id,
            col_area,
            pt_y_start,
            bin_data_content,
            start_row,
            end_row,
            is_continuation,
            start_cut,
            end_cut,
            is_block_split,
            pt_margin_left,
            pt_margin_right,
            pt_mt,
            false,
        );
        if render_deferred_rowbreak_host_text_after {
            if let Some(para) = paragraphs.get(para_index) {
                let has_real_text = para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                if has_real_text {
                    if let Some(comp) = composed.get(para_index) {
                        let text_start_line = comp.lines.iter().position(|line| {
                            line.runs
                                .iter()
                                .any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                        });
                        if let Some(start_line) = text_start_line {
                            let text_end_line = comp
                                .lines
                                .iter()
                                .rposition(|line| {
                                    line.runs.iter().any(|r| {
                                        r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                    })
                                })
                                .map(|i| i + 1)
                                .unwrap_or(comp.lines.len());
                            para_start_y.insert(para_index, y_offset);
                            y_offset = self.layout_partial_paragraph(
                                tree,
                                col_node,
                                para,
                                Some(comp),
                                styles,
                                col_area,
                                y_offset,
                                start_line,
                                text_end_line,
                                page_content.section_index,
                                para_index,
                                *multi_col_width,
                                Some(bin_data_content),
                                wrap_anchors.get(&para_index),
                            );
                        }
                    }
                }
            }
        }
        // [Task #1046 Stage 3 Class B/C] 분할 표 실제 콘텐츠 하단 기록 — 이후 더해지는
        // spacing_after/outer_margin_bottom(표 뒤 trailing 간격) 제외. overflow 검출이
        // 페이지 바닥의 후행 간격을 콘텐츠 초과로 오판하지 않도록 한다.
        self.last_item_content_bottom.set(y_offset);
        if let Some(para) = paragraphs.get(para_index) {
            let comp = composed.get(para_index);
            let para_style_id = comp
                .map(|c| c.para_style_id as usize)
                .unwrap_or(para.para_shape_id as usize);
            if let Some(para_style) = styles.para_styles.get(para_style_id) {
                let is_tac = para
                    .controls
                    .get(control_index)
                    .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    .unwrap_or(false);
                if is_tac {
                    if para_style.spacing_after > 0.0 {
                        y_offset += para_style.spacing_after;
                    }
                    let outer_margin_bottom_px =
                        if let Some(Control::Table(t)) = para.controls.get(control_index) {
                            hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
                        } else {
                            0.0
                        };
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
                let pt_is_square =
                    matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square);
                if !pt_is_tac && pt_is_square && !wrap_around_paras.is_empty() {
                    let wrap_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                    let wrap_sw = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                    let wrap_text_x = col_area.x + hwpunit_to_px(wrap_cs, self.dpi);
                    let wrap_text_width = hwpunit_to_px(wrap_sw, self.dpi);
                    // [Task #1745] 텍스트 혼합 anchor: 후속 어울림 문단 띠는 표 geometry 로,
                    // 호스트 텍스트는 이미 "분할 표 첫 부분" 경로에서 렌더됨 → 이중 렌더 방지.
                    let strip = crate::renderer::text_anchor_square_table_strip(para);
                    let (strip_x, strip_width) = strip
                        .map(|(cs, sw)| {
                            (
                                col_area.x + hwpunit_to_px(cs, self.dpi),
                                hwpunit_to_px(sw, self.dpi),
                            )
                        })
                        .unwrap_or((wrap_text_x, wrap_text_width));
                    let content_offset = if let Some(mt) = pt_mt {
                        mt.range_height(0, start_row)
                    } else {
                        0.0
                    };
                    let tbl_x_right = compute_square_wrap_tbl_x_right(t, col_area, self.dpi);
                    self.layout_wrap_around_paras(
                        tree,
                        col_node,
                        paragraphs,
                        composed,
                        styles,
                        col_area,
                        page_content.section_index,
                        para_index,
                        wrap_around_paras,
                        pt_y_before,
                        y_offset,
                        wrap_text_x,
                        wrap_text_width,
                        strip_x,
                        strip_width,
                        strip.is_none(),
                        content_offset,
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
    #[allow(clippy::too_many_arguments)]
    fn layout_shape_item(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        // [Task #1151 v9 결함 D] sibling TAC picture 가로 분배 cursor state.
        para_inline_state: &mut std::collections::HashMap<
            usize,
            super::layout::paragraph_layout::ParaInlineState,
        >,
        para_index: usize,
        control_index: usize,
        ctx: &ColumnItemCtx,
        y_offset: f64,
    ) -> f64 {
        let ColumnItemCtx {
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            layout,
            col_area,
            wrap_around_paras,
            ..
        } = ctx;
        // Task #402: 같은 paragraph 안에 TAC 컨트롤(표/그림/도형) 2개 이상이 서로 다른 line에
        // 배치된 경우, 두 번째 이후의 그림은 paragraph 시작 y가 아니라 진행된 y_offset
        // (선행 TAC 후속 위치)에 그려져야 표와 겹치지 않는다. control_index 이전에 같은
        // paragraph의 TAC 컨트롤이 있고 y_offset이 기존 등록값보다 진행됐으면 갱신한다.
        //
        // [Task #1151 v9 결함 D] sibling TAC picture 만 있는 경우 (= 가로 분배 시나리오) 는
        // para_start_y 갱신 X — picture 의 y 는 line_top_y 로 동일 유지. has_prior_tac 의
        // 종류를 Table/Shape vs Picture 로 분리하여 picture-only 시퀀스에서 y 진행을 차단.
        let has_prior_non_picture_tac = paragraphs
            .get(para_index)
            .map(|p| {
                p.controls.iter().take(control_index).any(|c| match c {
                    Control::Table(t) => t.common.treat_as_char,
                    Control::Shape(s) => s.common().treat_as_char,
                    _ => false,
                })
            })
            .unwrap_or(false);
        let has_prior_tac_picture = paragraphs
            .get(para_index)
            .map(|p| {
                p.controls.iter().take(control_index).any(|c| match c {
                    Control::Picture(p) => p.common.treat_as_char,
                    _ => false,
                })
            })
            .unwrap_or(false);
        if has_prior_non_picture_tac {
            // 선행 TAC Table/Shape 가 있는 경우만 진행된 y_offset 으로 갱신.
            let needs_update = para_start_y
                .get(&para_index)
                .map(|&existing| y_offset > existing + 1.0)
                .unwrap_or(true);
            if needs_update {
                para_start_y.insert(para_index, y_offset);
            }
        } else if !has_prior_tac_picture {
            // 첫 picture (선행 TAC picture 도 Table/Shape 도 없음): paragraph 시작 y 등록.
            para_start_y.entry(para_index).or_insert(y_offset);
        }
        // 선행 picture 만 있는 경우 (has_prior_tac_picture && !has_prior_non_picture_tac):
        // para_start_y 의 기존 값 유지 — 가로 분배의 첫 picture y 와 동일.
        let mut result_y = y_offset;
        if let Some(para) = paragraphs.get(para_index) {
            if let Some(ctrl) = para.controls.get(control_index) {
                if let Control::Picture(pic) = ctrl {
                    if pic.common.treat_as_char {
                        let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi).max(
                            hwpunit_to_px(pic.shape_attr.current_height as i32, self.dpi),
                        );
                        let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                        // 같은 paragraph 의 sibling wrap=TopAndBottom 개체(tac=false)가
                        // 차지하는 vertical 영역만큼 picture y 보정.
                        let sibling_reserved_hu =
                            super::layout::paragraph_layout::calc_sibling_topandbottom_reserved_hu(
                                &para.controls,
                            );
                        let sibling_reserved_px = hwpunit_to_px(sibling_reserved_hu, self.dpi);

                        // [Task #1151 v9 결함 D] sibling TAC picture 시퀀스 위치 판별.
                        // 한컴 native 정합: 동일 paragraph 안 sibling tac=true picture 들이
                        // 가로로 inline 분배 (inline glyph 처럼).
                        let tac_pic_seq =
                            super::layout::paragraph_layout::collect_sibling_tac_picture_widths_px(
                                &para.controls,
                                self.dpi,
                            );
                        let position_in_seq =
                            tac_pic_seq.iter().position(|(ci, _)| *ci == control_index);
                        let is_single_pic = tac_pic_seq.len() == 1;
                        let is_first_in_seq = position_in_seq == Some(0);
                        let is_subsequent_in_seq = position_in_seq.map(|p| p > 0).unwrap_or(false);
                        let is_last_in_seq = position_in_seq
                            .map(|p| p == tac_pic_seq.len() - 1)
                            .unwrap_or(false);

                        // pic_y 결정:
                        // - 단일 picture / 시퀀스 첫 picture: paragraph 시작 y + sibling_reserved
                        //   + 라벨/그림 높이 정합 보정
                        // - 시퀀스 후속 picture: state.line_top_y (pic_x wrap 처리 후 결정 — 아래)
                        // [Task #1151 v9 결함 D fix] pic_y 의 시퀀스 후속 picture 결정은 pic_x
                        // (wrap 처리 포함) 뒤로 옮김. 그 전에는 placeholder 로 default 값 사용.
                        let _ = is_single_pic;
                        let comp = composed.get(para_index);
                        let para_y_for_pic =
                            para_start_y.get(&para_index).copied().unwrap_or(y_offset)
                                + sibling_reserved_px;
                        let default_pic_y = self.compute_tac_picture_shape_y(
                            para,
                            comp,
                            styles,
                            para_y_for_pic,
                            pic_h,
                        );
                        let para_style_id = comp
                            .map(|c| c.para_style_id as usize)
                            .unwrap_or(para.para_shape_id as usize);
                        let para_style_ref = styles.para_styles.get(para_style_id);
                        let para_alignment = para_style_ref
                            .map(|s| s.alignment)
                            .unwrap_or(Alignment::Left);
                        // Task #347: 첫 줄 effective_margin (hanging indent: indent<0 → first-line은 margin_left만 적용)
                        let para_margin_left = para_style_ref.map(|s| s.margin_left).unwrap_or(0.0);
                        let para_indent = para_style_ref.map(|s| s.indent).unwrap_or(0.0);
                        // [Task #534] paragraph_layout 의 effective_margin_left 정합:
                        // visible stroke 보유 + border_spacing[0,1]=0 인 paragraph 는
                        // box_margin_left 를 inner padding 으로 추가 가산 (paragraph_layout.rs
                        // line 711-716 와 동일). wrap_host (Square wrap 표 보유) paragraph 는
                        // paragraph_layout 미호출되어 본 경로만 emit → inner_pad 누락 시
                        // 위치 결함 (예: exam_kor p18 pi=50/56 의 [A]/[B] 표시기 옆 그림).
                        let para_border_fill_id_pre =
                            para_style_ref.map(|s| s.border_fill_id).unwrap_or(0);
                        let has_visible_stroke = if para_border_fill_id_pre > 0 {
                            let idx = (para_border_fill_id_pre as usize).saturating_sub(1);
                            styles
                                .border_styles
                                .get(idx)
                                .map(|bs| {
                                    bs.borders.iter().any(|b| {
                                        !matches!(
                                            b.line_type,
                                            crate::model::style::BorderLineType::None
                                        ) && b.width > 0
                                    })
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        let bs_left_px = para_style_ref.map(|s| s.border_spacing[0]).unwrap_or(0.0);
                        let bs_right_px =
                            para_style_ref.map(|s| s.border_spacing[1]).unwrap_or(0.0);
                        let inner_pad_left =
                            if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
                                para_margin_left
                            } else {
                                0.0
                            };
                        let mut effective_margin_left = if para_indent > 0.0 {
                            para_margin_left + para_indent + inner_pad_left
                        } else {
                            para_margin_left + inner_pad_left
                        };
                        // [Task #534 v2] LINE_SEG.column_start 는 Square wrap 인라인 표/그림이
                        // 좌측에 floating 시 표 영역 이후 텍스트 시작 위치를 HWP IR 가 인코딩.
                        // layout_shape_item 은 col_area.x 그대로 사용 → picture (TAC) 가 표
                        // 영역 위에 겹쳐 표시되는 결함 (예: exam_kor p18 pi=50/56 [A]/[B]
                        // 표시기 + 그림). cs 가 effective_margin_left 보다 크면 cs 우선.
                        let line_seg_cs_px = para
                            .line_segs
                            .first()
                            .map(|s| hwpunit_to_px(s.column_start, self.dpi))
                            .unwrap_or(0.0);
                        if line_seg_cs_px > effective_margin_left {
                            effective_margin_left = line_seg_cs_px;
                        }
                        let para_margin_right =
                            para_style_ref.map(|s| s.margin_right).unwrap_or(0.0);
                        let avail_w =
                            (col_area.width - effective_margin_left - para_margin_right).max(pic_w);
                        // [Task #1151 v9 결함 D] pic_x 결정:
                        // - 단일 picture: 기존 alignment 그대로
                        // - 시퀀스 첫 picture: total_tac_width 기반 alignment + state 초기화
                        // - 시퀀스 후속 picture: state.cursor_x 사용 (가로 누적)
                        let pic_x = if is_subsequent_in_seq {
                            let cur = para_inline_state
                                .get(&para_index)
                                .map(|s| s.cursor_x)
                                .unwrap_or(col_area.x + effective_margin_left);
                            let line_right = col_area.x + effective_margin_left + avail_w;
                            // [Task #1151 v9 Stage 24] line wrap: cursor_x + pic_w > avail 면
                            // 다음 line 으로 wrap (cursor_x reset, line_top_y advance).
                            if cur + pic_w > line_right + 0.5 {
                                if let Some(state) = para_inline_state.get_mut(&para_index) {
                                    state.cursor_x = col_area.x + effective_margin_left;
                                    state.line_top_y += state.line_height;
                                    state.line_height = 0.0;
                                }
                                col_area.x + effective_margin_left
                            } else {
                                cur
                            }
                        } else if is_first_in_seq && !is_single_pic {
                            // 시퀀스 첫 picture: 전체 시퀀스 폭 기반 alignment.
                            let total_tac_width: f64 = tac_pic_seq.iter().map(|(_, w)| w).sum();
                            let align_offset = match para_alignment {
                                Alignment::Center | Alignment::Distribute => {
                                    (avail_w - total_tac_width).max(0.0) / 2.0
                                }
                                Alignment::Right => (avail_w - total_tac_width).max(0.0),
                                _ => 0.0,
                            };
                            col_area.x + effective_margin_left + align_offset
                        } else {
                            // 단일 picture (기존 경로): 기존 alignment 그대로.
                            match para_alignment {
                                Alignment::Center | Alignment::Distribute => {
                                    col_area.x
                                        + effective_margin_left
                                        + (avail_w - pic_w).max(0.0) / 2.0
                                }
                                Alignment::Right => {
                                    col_area.x + effective_margin_left + (avail_w - pic_w).max(0.0)
                                }
                                _ => col_area.x + effective_margin_left,
                            }
                        };

                        // [Task #1151 v9 결함 D fix] pic_y 의 시퀀스 후속 picture 결정 —
                        // pic_x wrap 처리 후 갱신된 state.line_top_y 사용 (wrap 시 진행됨).
                        let pic_y = if is_subsequent_in_seq {
                            para_inline_state
                                .get(&para_index)
                                .map(|s| s.line_top_y)
                                .unwrap_or(default_pic_y)
                        } else {
                            default_pic_y
                        };

                        // Task #347: paragraph_layout이 호출되지 않는 빈 문단(텍스트 없음 +
                        // TAC 그림만 있는 경우)에서는 인라인 그림이 누락되어
                        // 박스 프레임 시각이 사라지고 후속 InFrontOfText 표가 위로 겹침.
                        // 호스트 문단에 실제 텍스트가 없으면 여기서 직접 이미지 노드를 생성하고
                        // y_offset을 그림 높이만큼 진행시킨다.
                        let has_real_text =
                            para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                        let has_full_para_item = page_content.column_contents.iter().any(|cc| {
                            cc.items.iter().any(|it| {
                                matches!(
                                    it,
                                    PageItem::FullParagraph { para_index: pi }
                                        if *pi == para_index
                                )
                            })
                        });
                        // [Task #418/#376] paragraph_layout 의 빈 문단 + TAC Picture 분기에서
                        // 이미 ImageNode 가 emit 되어 inline_shape_position 이 등록된 경우,
                        // 여기서 또 push 하면 이중 emit 이 된다. 등록된 경우 push 를 스킵하고
                        // result_y 만 갱신한다.
                        //
                        // [Task #1452 Stage 6] FullParagraph 가 있는 문단은 paragraph_layout 이
                        // TAC 그림을 줄 안에 배치한다. 위치 등록이 아직 안 보이는 순서여도
                        // Shape fallback 을 그리면 같은 투명 그림이 한 번 더 합성된다.
                        let registered_inline_pos = tree.get_inline_shape_position(
                            page_content.section_index,
                            para_index,
                            control_index,
                            None,
                        );
                        let already_registered = registered_inline_pos.is_some();
                        let effective_pic_y = registered_inline_pos
                            .map(|(_, registered_y)| registered_y)
                            .unwrap_or(pic_y);
                        // paragraph_layout 이 이미 emit 한 인라인 그림은 실제 bbox 높이와 같은
                        // common.height 기준으로 content bottom 을 판정한다.
                        let effective_pic_h = if already_registered {
                            hwpunit_to_px(pic.common.height as i32, self.dpi)
                        } else {
                            pic_h
                        };
                        // [Task #1151 v9 결함 D] state 갱신 — 가로 분배 cursor 누적.
                        // 첫 picture 시 line_top_y / cursor_x 초기화. 후속 picture 마다 cursor_x 가산.
                        if !is_single_pic {
                            let entry = para_inline_state.entry(para_index).or_insert(
                                super::layout::paragraph_layout::ParaInlineState {
                                    cursor_x: pic_x + pic_w,
                                    line_top_y: pic_y,
                                    line_height: pic_h,
                                },
                            );
                            if is_subsequent_in_seq {
                                entry.cursor_x = pic_x + pic_w;
                                entry.line_height = entry.line_height.max(pic_h);
                            } else {
                                // 첫 picture: 초기화 (기존 값 덮어쓰기)
                                entry.cursor_x = pic_x + pic_w;
                                entry.line_top_y = pic_y;
                                entry.line_height = pic_h;
                            }
                        }

                        if !already_registered && !has_full_para_item {
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
                                Some((
                                    pic.shape_attr.original_width,
                                    pic.shape_attr.original_height,
                                ))
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
                                    opacity: pic.image_attr.opacity(),
                                    transform: utils::extract_shape_transform(&pic.shape_attr),
                                    // [Issue #1167] wrap 모드 보존 — SVG plane multi-pass z-order
                                    // 판별에 사용 (BehindText 워터마크가 본문 뒤로). PaintOp
                                    // 경로(skia/canvaskit)는 별도로 image.text_wrap 을 set 하므로 무관.
                                    text_wrap: Some(pic.common.text_wrap),
                                    external_path: pic.image_attr.external_path.clone(),
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
                                page_content.section_index,
                                para_index,
                                control_index,
                                None,
                                pic_x,
                                pic_y,
                            );
                            if !has_real_text {
                                // [Task #462] LINE_SEG 의 lh+ls 를 advance 로 사용 — 이미지 박스
                                // 높이만 사용하면 leading + line_spacing 이 누락되어 다음 문단이
                                // 그림 바로 아래에 붙음. max(pic_h) 는 LINE_SEG 가 비정상적으로
                                // 작은 경우의 안전장치.
                                // [Task #1151 v9 결함 D] 가로 분배 시퀀스의 중간 picture 는 result_y
                                // 진행 안 함 (y_offset 유지). 시퀀스 마지막 picture 또는 단일 picture
                                // 에서만 advance — 다음 paragraph 가 가로 분배 영역 아래로 진행.
                                if !has_full_para_item {
                                    let line_advance = para
                                        .line_segs
                                        .first()
                                        .map(|ls| {
                                            hwpunit_to_px(
                                                ls.line_height + ls.line_spacing,
                                                self.dpi,
                                            )
                                        })
                                        .unwrap_or(pic_h);
                                    if is_single_pic || is_last_in_seq {
                                        // 시퀀스 마지막: state 의 line_height (시퀀스 최대 height) 기반 advance
                                        let line_top_y = para_inline_state
                                            .get(&para_index)
                                            .map(|s| s.line_top_y)
                                            .unwrap_or(pic_y);
                                        let line_height = para_inline_state
                                            .get(&para_index)
                                            .map(|s| s.line_height)
                                            .unwrap_or(pic_h);
                                        result_y = line_top_y + line_advance.max(line_height);
                                    }
                                    // 중간 picture: result_y = y_offset (그대로 유지, line 4527 의 default)
                                }
                            }
                        } else if !has_real_text && !has_full_para_item {
                            // [Task #418/#376] paragraph_layout 가 이미 emit 함 — push 스킵, result_y 만 갱신
                            // [Task #462] 동일하게 LINE_SEG 기반 advance 사용
                            // [Task #1151 v9 결함 D] 가로 분배 시퀀스의 중간 picture 는 result_y
                            // 진행 안 함 (y_offset 유지). 시퀀스 마지막 picture 또는 단일 picture
                            // 에서만 advance — 다음 paragraph 가 가로 분배 영역 아래로 진행.
                            let line_advance = para
                                .line_segs
                                .first()
                                .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
                                .unwrap_or(pic_h);
                            if is_single_pic || is_last_in_seq {
                                // 시퀀스 마지막: state 의 line_height (시퀀스 최대 height) 기반 advance
                                let line_top_y = para_inline_state
                                    .get(&para_index)
                                    .map(|s| s.line_top_y)
                                    .unwrap_or(pic_y);
                                let line_height = para_inline_state
                                    .get(&para_index)
                                    .map(|s| s.line_height)
                                    .unwrap_or(pic_h);
                                result_y = line_top_y + line_advance.max(line_height);
                            }
                            // 중간 picture: result_y = y_offset (그대로 유지, line 4527 의 default)
                        }

                        let mut pic_content_bottom = effective_pic_y + effective_pic_h;
                        if let Some(ref caption) = pic.caption {
                            use crate::model::shape::CaptionDirection;
                            let caption_spacing = hwpunit_to_px(caption.spacing as i32, self.dpi);
                            let caption_h = self.calculate_caption_height(&pic.caption, styles);
                            // [Task #864 Stage E v2] paragraph_layout 가 inline TAC image 를
                            // baseline-aligned (y = pic_y + baseline - pic_h) 위치에 emit
                            // 함. caption 은 image 바로 아래 (image_bottom = pic_y + baseline)
                            // 에 위치해야 함. 기존 pic_y + pic_h 사용 시 image 영역 안에
                            // 그려져 가려짐.
                            let baseline_px = para
                                .line_segs
                                .first()
                                .map(|ls| hwpunit_to_px(ls.baseline_distance, self.dpi))
                                .unwrap_or(effective_pic_h);
                            let image_bottom = effective_pic_y + baseline_px.max(effective_pic_h);
                            let cap_y = match caption.direction {
                                CaptionDirection::Bottom => image_bottom + caption_spacing,
                                CaptionDirection::Top => effective_pic_y,
                                _ => image_bottom + caption_spacing,
                            };
                            if caption.direction == CaptionDirection::Top {
                                let dy = caption_h + caption_spacing;
                                Self::offset_inline_image_y(
                                    col_node,
                                    para_index,
                                    control_index,
                                    dy,
                                );
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
                                tree,
                                col_node,
                                caption,
                                styles,
                                col_area,
                                pic_x,
                                pic_w,
                                cap_y,
                                &mut self.auto_counter.borrow_mut(),
                                bin_data_content,
                                Some(cell_ctx),
                            );
                            // [Task #864 Stage F] caption 이 차지한 영역까지 result_y 진행.
                            // 미진행 시 다음 paragraph 가 caption 위에 그려져 겹침
                            // (HWP3 sample14 page 4 "Visual Block을 이용한 대소문자 변경"
                            // 가 본문 "먼저 원하는 구간을..." 와 겹침). Bottom 만 진행 (Top
                            // 은 위에서 offset_inline_image_y 로 image 전체를 밀어서 처리).
                            //
                            // [Task #957] 빈 caption (text 없음 + controls 없음) 은 SVG 에 invisible.
                            // pic_y = para_start_y[para_idx] 가 has_prior_tac 로 인해 후속 위치로
                            // 갱신되면 image_bottom = pic_y + pic_h 가 페이지 바깥 위치로 계산되어
                            // result_y 가 phantom +caption_h 만큼 누적 → 후속 paragraph 가 다음
                            // 페이지로 밀림 (sample16 page 18 pi=394 ci=1 "그림" 의 empty caption
                            // 으로 +430.6px advance). 빈 caption 은 advance skip.
                            let caption_is_empty = caption.paragraphs.iter().all(|p| {
                                p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
                                    && p.controls.is_empty()
                            });
                            if !caption_is_empty
                                && matches!(caption.direction, CaptionDirection::Bottom)
                            {
                                let cap_bottom = cap_y + caption_h;
                                if cap_bottom > result_y {
                                    result_y = cap_bottom;
                                }
                                pic_content_bottom = pic_content_bottom.max(cap_bottom);
                            }
                        }
                        let prev_bottom = self.last_item_content_bottom.get();
                        self.last_item_content_bottom
                            .set(if prev_bottom.is_finite() {
                                prev_bottom.max(pic_content_bottom)
                            } else {
                                pic_content_bottom
                            });
                    } else {
                        let is_paper_based = (pic.common.vert_rel_to == VertRelTo::Paper
                            || pic.common.vert_rel_to == VertRelTo::Page)
                            && (pic.common.horz_rel_to == HorzRelTo::Paper
                                || pic.common.horz_rel_to == HorzRelTo::Page);
                        if is_paper_based {
                            let mut temp_parent = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Column(0),
                                BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
                            );
                            let paper_area = LayoutRect {
                                x: 0.0,
                                y: 0.0,
                                width: layout.page_width,
                                height: layout.page_height,
                            };
                            let _ = self.layout_body_picture(
                                tree,
                                &mut temp_parent,
                                pic,
                                &paper_area,
                                col_area,
                                &layout.body_area,
                                &paper_area,
                                bin_data_content,
                                styles,
                                Alignment::Left,
                                0.0,
                                page_content.section_index,
                                para_index,
                                control_index,
                                false,
                            );
                            let layer = Self::render_layer_from_common(
                                &pic.common,
                                para_index,
                                control_index,
                            );
                            Self::push_layered_paper_children(
                                paper_images,
                                &mut temp_parent,
                                layer,
                            );
                        } else {
                            let comp = composed.get(para_index);
                            let para_style_id = comp
                                .map(|c| c.para_style_id as usize)
                                .unwrap_or(para.para_shape_id as usize);
                            let alignment = styles
                                .para_styles
                                .get(para_style_id)
                                .map(|s| s.alignment)
                                .unwrap_or(Alignment::Left);
                            let para_base_y =
                                para_start_y.get(&para_index).copied().unwrap_or(y_offset);
                            let pic_y = if matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::Square
                            ) && matches!(
                                pic.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            ) {
                                square_wrap_first_narrow_line_vpos_px(para, col_area, self.dpi)
                                    .map(|dy| para_base_y + dy)
                                    .unwrap_or(para_base_y)
                            } else {
                                para_base_y
                            };
                            let pic_container = LayoutRect {
                                x: col_area.x,
                                y: pic_y,
                                width: col_area.width,
                                height: col_area.height - (pic_y - col_area.y),
                            };
                            let saved_y_offset = y_offset;
                            // [Task #1079] 파일 vpos 가 이미 그림 공간을 반영(그림 para 줄 앞
                            // gap ≥ 그림 높이)하면 그림 높이 추가 진행 생략(typeset pushdown
                            // 게이트와 동일 조건). #409 계열(gap 작음)은 현행 유지.
                            let vpos_accounts_for_height = para_index > 0 && {
                                const PUSHDOWN_GAP_TOL_PX: f64 = 8.0;
                                let obj_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                let v_cur = paragraphs[para_index]
                                    .line_segs
                                    .first()
                                    .map(|s| s.vertical_pos);
                                let prev_end = paragraphs[para_index - 1]
                                    .line_segs
                                    .last()
                                    .map(|s| s.vertical_pos + s.line_height);
                                match (v_cur, prev_end) {
                                    (Some(vc), Some(pe)) if vc > pe => {
                                        hwpunit_to_px((vc - pe) as i32, self.dpi)
                                            >= obj_h - PUSHDOWN_GAP_TOL_PX
                                    }
                                    _ => false,
                                }
                            };
                            result_y = self.layout_body_picture(
                                tree,
                                col_node,
                                pic,
                                &pic_container,
                                col_area,
                                &layout.body_area,
                                &LayoutRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: layout.page_width,
                                    height: layout.page_height,
                                },
                                bin_data_content,
                                styles,
                                alignment,
                                pic_y,
                                page_content.section_index,
                                para_index,
                                control_index,
                                vpos_accounts_for_height,
                            );
                            // layout_body_picture needs the host paragraph y for Para-relative
                            // positioning, but InFront pictures must not rewind the
                            // already-advanced text flow cursor back to that paragraph y.
                            //
                            // Keep BehindText on the legacy non-advancing path. HWP5 files such
                            // as samples/복학원서.hwp use an empty first paragraph with a
                            // BehindText logo; preserving the advanced cursor there inserts an
                            // extra line-height before the following table.
                            if matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::InFrontOfText
                            ) {
                                result_y = saved_y_offset;
                            }
                            // [Task #959] horz_rel_to=Column 의 picture 가 col_area 우측을
                            // 초과하는 위치에 emit 되면 한컴 viewer 는 column flow 에
                            // reservation 하지 않음. rhwp 는 cursor 를 picture height 만큼
                            // advance → 후속 paragraph 처짐.
                            // (3-11월_실전_통합_2022 page 1 우측 단 pi=69 picture
                            //  pic_emit_x=767 > col_right=759 → +274px advance → 문9 처짐)
                            // Picture 의 좌측 edge (x) 가 col_area 우측을 초과하면 advance skip.
                            if matches!(pic.common.horz_rel_to, HorzRelTo::Column) {
                                // [Issue #1230] emit 폭 판정은 layout_body_picture 와 동일한
                                // 프레임 크기를 사용해야 skip 판정이 실제 배치와 일치한다.
                                let (pic_width_hu, _) = picture_flow_frame_size_hu(pic);
                                let pic_width_px = hwpunit_to_px(pic_width_hu, self.dpi);
                                let h_offset_px =
                                    hwpunit_to_px(pic.common.horizontal_offset as i32, self.dpi);
                                let pic_emit_x = match pic.common.horz_align {
                                    crate::model::shape::HorzAlign::Left
                                    | crate::model::shape::HorzAlign::Inside => {
                                        col_area.x + h_offset_px
                                    }
                                    crate::model::shape::HorzAlign::Center => {
                                        col_area.x
                                            + (col_area.width - pic_width_px) / 2.0
                                            + h_offset_px
                                    }
                                    crate::model::shape::HorzAlign::Right
                                    | crate::model::shape::HorzAlign::Outside => {
                                        col_area.x + col_area.width - pic_width_px - h_offset_px
                                    }
                                };
                                if pic_emit_x >= col_area.x + col_area.width {
                                    result_y = saved_y_offset;
                                }
                            }
                            // [Task #683] 빈 paragraph (텍스트 없음) + Para-relative TopAndBottom
                            // 그림 (caption 없음) 의 layout 진행량 보정. 한컴 한글 2022 PDF 는
                            // 그림 다음에 paragraph 의 line baseline 1줄(line_height + line_spacing)
                            // 을 추가 진행하나 rhwp 기본 layout 은 image_height 만 진행하여
                            // cluster 거리가 1 line 부족 (pr-149.hwp 18864 HU vs 17280 HU 결함).
                            if matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            ) && matches!(
                                pic.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            ) && pic.caption.is_none()
                            {
                                let has_visible_text =
                                    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                                if !has_visible_text {
                                    let line_advance = para
                                        .line_segs
                                        .first()
                                        .map(|ls| {
                                            hwpunit_to_px(
                                                ls.line_height + ls.line_spacing,
                                                self.dpi,
                                            )
                                        })
                                        .unwrap_or(0.0);
                                    result_y += line_advance;
                                }
                            }
                            // Square wrap + Para-relative: 그림 높이로 column y를 밀지 않는다.
                            // 텍스트는 그림 옆에 segment_width로 제어되어 흐르므로
                            // 후속 문단은 앵커 단락 직후(shape item y_offset)부터 시작해야 한다.
                            // layout_body_picture의 y_offset은 pic_y(=단락 시작 y)이므로
                            // 반환값이 para_start_y로 거슬러 올라감 — 이를 shape item y로 복원.
                            if matches!(pic.common.text_wrap, crate::model::shape::TextWrap::Square)
                                && matches!(
                                    pic.common.vert_rel_to,
                                    crate::model::shape::VertRelTo::Para
                                )
                            {
                                result_y = y_offset;
                            }
                            // [Task #525] Picture Square wrap 의 호스트 paragraph 텍스트는
                            // 정상 PageItem::FullParagraph 경로 (layout_composed_paragraph 의
                            // has_picture_shape_square_wrap 분기, paragraph_layout.rs:822/973)
                            // 가 LINE_SEG.cs/sw 기반으로 그림 옆 (좁은) + 그림 아래 (넓은)
                            // 모두 처리. Task #604 Stage 2 의 wrap_anchors 메타데이터 채널
                            // 로 FullParagraph path 가 cs offset 을 정확히 적용하므로 별도 호출 불필요.
                        }
                    }
                } else if let Control::Shape(shape) = ctrl {
                    let common = shape.common();
                    if common.treat_as_char {
                        let has_real_text =
                            para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                        let registered_inline_pos = tree.get_inline_shape_position(
                            page_content.section_index,
                            para_index,
                            control_index,
                            None,
                        );
                        let already_registered = registered_inline_pos.is_some();

                        if !has_real_text {
                            let shape_w = hwpunit_to_px(common.width as i32, self.dpi);
                            let shape_h = hwpunit_to_px(common.height as i32, self.dpi);
                            // [Task #990] 해당 문단에 PageItem::FullParagraph 가
                            // 발행되었으면(빈 문단이 호스트인 RFP 형) layout_paragraph
                            // 가 이미 LINE_SEG advance 를 마쳤으므로, Shape 항목은
                            // 글상자를 호스트 문단 시작(para_start)에 배치하고 재진행
                            // 하지 않는다 — 이중 가산 방지(Task #974 c3e32151 회귀).
                            // FullParagraph 항목이 없으면(선행 표 등에 이어 붙은
                            // Shape, 예: hy-001 pi=27) Task #974 동작을 유지한다.
                            let has_full_para_item =
                                page_content.column_contents.iter().any(|cc| {
                                    cc.items.iter().any(|it| {
                                        matches!(
                                            it,
                                            PageItem::FullParagraph { para_index: pi }
                                                if *pi == para_index
                                        )
                                    })
                                });
                            let para_start =
                                para_start_y.get(&para_index).copied().unwrap_or(y_offset);
                            let shape_y = if let Some((_, registered_y)) = registered_inline_pos {
                                registered_y
                            } else if has_full_para_item {
                                para_start
                            } else {
                                y_offset
                            };

                            if !already_registered {
                                let comp = composed.get(para_index);
                                let para_style_id = comp
                                    .map(|c| c.para_style_id as usize)
                                    .unwrap_or(para.para_shape_id as usize);
                                let para_style_ref = styles.para_styles.get(para_style_id);
                                let para_alignment = para_style_ref
                                    .map(|s| s.alignment)
                                    .unwrap_or(Alignment::Left);
                                let para_margin_left =
                                    para_style_ref.map(|s| s.margin_left).unwrap_or(0.0);
                                let para_indent = para_style_ref.map(|s| s.indent).unwrap_or(0.0);
                                let para_margin_right =
                                    para_style_ref.map(|s| s.margin_right).unwrap_or(0.0);
                                let effective_margin_left = if para_indent > 0.0 {
                                    para_margin_left + para_indent
                                } else {
                                    para_margin_left
                                };
                                let avail_w =
                                    (col_area.width - effective_margin_left - para_margin_right)
                                        .max(shape_w);
                                let shape_x = match para_alignment {
                                    Alignment::Center | Alignment::Distribute => {
                                        col_area.x
                                            + effective_margin_left
                                            + (avail_w - shape_w).max(0.0) / 2.0
                                    }
                                    Alignment::Right => {
                                        col_area.x
                                            + effective_margin_left
                                            + (avail_w - shape_w).max(0.0)
                                    }
                                    _ => col_area.x + effective_margin_left,
                                };

                                tree.set_inline_shape_position(
                                    page_content.section_index,
                                    para_index,
                                    control_index,
                                    None,
                                    shape_x,
                                    shape_y,
                                );
                            }

                            // [Task #990] FullParagraph 항목이 없는 경우에만
                            // LINE_SEG 1회분을 진행한다. FullParagraph 가 있으면
                            // 이미 진행되었으므로 result_y(=y_offset)를 유지한다.
                            if !has_full_para_item {
                                let line_advance = para
                                    .line_segs
                                    .first()
                                    .map(|ls| {
                                        hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi)
                                    })
                                    .unwrap_or(shape_h);
                                result_y = shape_y + line_advance.max(shape_h);
                            }
                            let prev_bottom = self.last_item_content_bottom.get();
                            let shape_bottom = shape_y + shape_h;
                            self.last_item_content_bottom
                                .set(if prev_bottom.is_finite() {
                                    prev_bottom.max(shape_bottom)
                                } else {
                                    shape_bottom
                                });
                        }
                    } else if !common.treat_as_char
                        && matches!(
                            common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        )
                        && matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                    {
                        // [Issue #1156] 비-TAC 자리차지(TopAndBottom) 객체(차트 OLE 등):
                        // 자리차지 객체는 본문 텍스트를 위/아래로 밀어내므로, 후속 콘텐츠
                        // 시작 y(result_y)를 객체 높이 + 아래 여백만큼 진행시켜 텍스트가
                        // 객체 영역과 겹치지 않게 한다. (typeset.rs Stage 2 의 current_height
                        // 가산과 layout 정합 — 단 이동 후 단 시작 y_offset 기준.)
                        let shape_h = hwpunit_to_px(common.height as i32, self.dpi);
                        let margin_bottom = hwpunit_to_px(common.margin.bottom as i32, self.dpi);
                        let advance = shape_h + margin_bottom;
                        if y_offset + advance > result_y {
                            result_y = y_offset + advance;
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
        // [Task #1745] 후속 어울림 문단(WrapAroundPara)이 놓일 wrap 띠. 표 단독 anchor
        // 는 wrap_text_x/width 와 동일. 텍스트 혼합 anchor 는 표 geometry 로 도출한
        // 띠(text_anchor_square_table_strip) — host 영역(전폭)과 분리된다.
        strip_x: f64,
        strip_width: f64,
        // [Task #1745] 호스트 문단 텍스트 렌더 여부. 분할 표의 텍스트 혼합 anchor 는
        // 호스트 텍스트가 이미 일반 경로(분할 표 첫 부분)에서 렌더되므로 false 로
        // 이중 렌더를 막는다.
        render_host_text: bool,
        table_content_offset: f64,
        bin_data_content: &[BinDataContent],
        // Task #463: 인라인 floating 표(예: 인용 따옴표 ｢｣)의 우측 끝 x 좌표.
        // wrap host paragraph 의 외곽선이 이 표 위치까지 둘러싸도록 box 너비를
        // 확장하기 위해 caller 에서 계산하여 전달한다. None 이면 box 미확장.
        tbl_x_right: Option<f64>,
    ) {
        // 이 표에 연관된 어울림 문단만 필터링
        let related: Vec<_> = wrap_around_paras
            .iter()
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
        let host_para_style = composed
            .get(table_para_index)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize));
        let host_margin_left = host_para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let host_margin_right = host_para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let wrap_area = LayoutRect {
            x: wrap_text_x - host_margin_left,
            y: col_area.y,
            width: wrap_text_width + host_margin_left + host_margin_right,
            height: col_area.height,
        };
        // [Task #1745] 후속 어울림 문단용 띠 영역 (표 단독 anchor 는 wrap_area 와 동일).
        let strip_area = LayoutRect {
            x: strip_x - host_margin_left,
            y: col_area.y,
            width: strip_width + host_margin_left + host_margin_right,
            height: col_area.height,
        };

        // 호스트 문단(표 문단) 텍스트를 어울림 영역에 렌더링
        let has_host_text = table_para
            .text
            .chars()
            .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
        if table_content_offset == 0.0 && render_host_text {
            if has_host_text {
                if let Some(comp) = composed.get(table_para_index) {
                    let text_start_line = comp.lines.iter().position(|line| {
                        line.runs
                            .iter()
                            .any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                    });
                    if let Some(start_line) = text_start_line {
                        // 호스트 본문의 모든 텍스트 줄을 wrap 영역에 렌더링
                        // (Task #295: 자가 wrap host의 다중 줄 누락 수정)
                        let text_end_line = comp
                            .lines
                            .iter()
                            .rposition(|line| {
                                line.runs.iter().any(|r| {
                                    r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                })
                            })
                            .map(|i| i + 1)
                            .unwrap_or(comp.lines.len());
                        // Task #463: wrap host 의 외곽선은 원래 col_area 너비로 그려야
                        // 인라인 floating 표(인용 따옴표 ｢｣ 등)를 박스가 둘러쌈. tbl_x_right
                        // 가 col_area 우측을 넘으면 그 위치까지 박스 너비를 확장한다.
                        let prev_override = self.border_box_override.get();
                        let extended_width = match tbl_x_right {
                            Some(tx) if tx > col_area.x + col_area.width => tx - col_area.x,
                            _ => col_area.width,
                        };
                        self.border_box_override
                            .set(Some((col_area.x, extended_width)));
                        self.layout_partial_paragraph(
                            tree,
                            col_node,
                            table_para,
                            Some(comp),
                            styles,
                            &wrap_area,
                            table_y_start,
                            start_line,
                            text_end_line,
                            section_index,
                            table_para_index,
                            None,
                            Some(bin_data_content),
                            None, // 표 호스트 어울림 문단 — 별도 wrap_anchor 메커니즘
                        );
                        self.border_box_override.set(prev_override);
                        // 어울림 문단은 항상 ↵ 표시 필요 — 부분 렌더링 시 is_para_end 강제 설정
                        force_para_end_on_last_run(col_node);
                    }
                }
            } else {
                // 호스트 문단에 텍스트 없음 (빈 문단 + 표): ↵ 마크 렌더링
                let seg = table_para.line_segs.first();
                let line_height = seg
                    .map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi))
                    .unwrap_or(13.3);
                let font_size = seg
                    .map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi))
                    .unwrap_or(13.3);
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
                // 어울림 문단의 전체 줄 렌더링.
                // 표 어울림: 각 WrapAroundPara가 별도 1-줄 문단이므로 all_lines=1.
                // 그림 어울림: 하나의 WrapAroundPara에 여러 줄이 포함될 수 있어 전체 렌더링.
                let end_line = comp.map(|c| c.lines.len()).unwrap_or(1);
                self.layout_partial_paragraph(
                    tree,
                    col_node,
                    para,
                    comp,
                    styles,
                    &strip_area,
                    para_y,
                    0,
                    end_line,
                    section_index,
                    wp.para_index,
                    None,
                    Some(bin_data_content),
                    None, // 표 호스트 어울림 문단 — 별도 wrap_anchor 메커니즘
                );
                // 어울림 문단은 항상 ↵ 표시 필요
                force_para_end_on_last_run(col_node);
            } else {
                // 빈 리턴 문단: ↵ 마크 렌더링
                let line_height = crate::renderer::hwpunit_to_px(seg.line_height, self.dpi);
                // 문단의 글자 모양에서 실제 폰트 크기 추출
                let font_size = {
                    let cs_id = para
                        .char_shapes
                        .first()
                        .map(|cs| cs.char_shape_id)
                        .unwrap_or(0);
                    styles
                        .char_styles
                        .get(cs_id as usize)
                        .map(|cs| cs.font_size)
                        .filter(|fs| *fs > 0.0)
                        .unwrap_or(13.3)
                };
                let mark_x = strip_x;

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
            if let PageItem::Shape {
                para_index,
                control_index,
            } = item
            {
                let para_y = para_start_y.get(para_index).copied().unwrap_or(col_area.y);
                let comp = composed.get(*para_index);
                let para_style_id = if let Some(para) = paragraphs.get(*para_index) {
                    comp.map(|c| c.para_style_id as usize)
                        .unwrap_or(para.para_shape_id as usize)
                } else {
                    0
                };
                let alignment = styles
                    .para_styles
                    .get(para_style_id)
                    .map(|s| s.alignment)
                    .unwrap_or(Alignment::Left);
                let z_order = paragraphs
                    .get(*para_index)
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
            let ctrl = paragraphs
                .get(para_index)
                .and_then(|p| p.controls.get(control_index));
            let is_paper_based = ctrl
                .map(|ctrl| {
                    let common = match ctrl {
                        Control::Shape(s) => Some(s.common()),
                        Control::Table(t) => Some(&t.common),
                        _ => None,
                    };
                    common
                        .map(|c| {
                            matches!(c.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page)
                                || matches!(c.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            let is_table_control = ctrl
                .map(|c| matches!(c, Control::Table(_)))
                .unwrap_or(false);
            let is_tac_picture_shape = matches!(
                ctrl,
                Some(Control::Shape(shape))
                    if shape.common().treat_as_char
                        && matches!(shape.as_ref(), ShapeObject::Picture(_))
            );

            let paper_area = LayoutRect {
                x: 0.0,
                y: 0.0,
                width: layout.page_width,
                height: layout.page_height,
            };

            if is_table_control {
                // InFrontOfText/BehindText 표: paper 기준 절대 위치에 렌더링
                if let Some(Control::Table(table)) = paragraphs
                    .get(para_index)
                    .and_then(|p| p.controls.get(control_index))
                {
                    let mut temp_parent = RenderNode::new(
                        tree.next_id(),
                        RenderNodeType::Column(0),
                        BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
                    );
                    self.layout_table(
                        tree,
                        &mut temp_parent,
                        table,
                        page_content.section_index,
                        styles,
                        0,
                        col_area,
                        para_y,
                        bin_data_content,
                        None,
                        0,
                        Some((para_index, control_index)),
                        alignment,
                        None,
                        0.0,
                        0.0,
                        None,
                        None,
                        None,
                        false,
                        false,
                    );
                    let layer =
                        Self::render_layer_from_common(&table.common, para_index, control_index);
                    Self::push_layered_paper_children(paper_images, &mut temp_parent, layer);
                }
            } else if is_tac_picture_shape {
                let mut temp_parent = RenderNode::new(
                    tree.next_id(),
                    RenderNodeType::Column(0),
                    col_node.bbox.clone(),
                );
                if let (Some(para), Some(Control::Shape(shape))) =
                    (paragraphs.get(para_index), ctrl)
                {
                    let common = shape.common();
                    let comp = composed.get(para_index);
                    let base_x =
                        self.compute_tac_pic_x(para, comp, styles, col_area, control_index);
                    let inline_x =
                        base_x + hwpunit_to_px(signed_hwpunit(common.horizontal_offset), self.dpi);
                    let shape_attr = shape.shape_attr();
                    let shape_h = hwpunit_to_px(common.height as i32, self.dpi)
                        .max(hwpunit_to_px(shape_attr.current_height as i32, self.dpi));
                    let inline_y = self
                        .compute_tac_picture_shape_y(para, comp, styles, para_y, shape_h)
                        + hwpunit_to_px(signed_hwpunit(common.vertical_offset), self.dpi);
                    tree.set_inline_shape_position(
                        page_content.section_index,
                        para_index,
                        control_index,
                        None,
                        inline_x,
                        inline_y,
                    );
                }
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
                    false,
                );
                insert_before_para_text(
                    col_node,
                    para_index,
                    temp_parent.children.drain(..).collect(),
                );
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
                    false,
                );
                if let Some(layer) = ctrl.and_then(|ctrl| match ctrl {
                    Control::Shape(shape) => Some(Self::render_layer_from_common(
                        shape.common(),
                        para_index,
                        control_index,
                    )),
                    Control::Table(table) => Some(Self::render_layer_from_common(
                        &table.common,
                        para_index,
                        control_index,
                    )),
                    _ => None,
                }) {
                    Self::push_layered_paper_children(paper_images, &mut temp_parent, layer);
                } else {
                    paper_images.append(&mut temp_parent.children);
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
                    false,
                );
            }
            // [Task #525] 비-TAC Picture/Shape Square wrap 의 어울림 문단 렌더링은
            // layout_shape_item:3106 (PageItem::Shape 처리 시) 에서 수행. 본 패스에서
            // 별도 호출은 동일 paragraph 의 wrap-around 텍스트가 두 다른 col_w 정렬로
            // distinct x 위치에 중복 emit 되어 (광범위 시각 결함, 7 샘플 37 페이지 영향)
            // 제거. Task #604 Stage 2 의 wrap_anchors 메타데이터 채널로 FullParagraph
            // path 가 cs offset 을 정확히 적용하므로 별도 호출 불필요.
        }
    }

    /// `Control::Shape(ShapeObject::Picture)` 형태의 글자처럼 취급 그림은
    /// paragraph_layout 이 직접 ImageNode 를 만들지 않고 shape pass 에서 그린다.
    /// 한컴은 라벨 텍스트와 그림 높이가 같은 LINE_SEG에 있을 때 라벨 한 줄을 먼저
    /// 배치한 뒤 그림을 아래에 둔다. raw line y 그대로 배치하면 라벨이 그림에 가려진다.
    fn compute_tac_picture_shape_y(
        &self,
        _para: &Paragraph,
        comp: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        para_y: f64,
        shape_height_px: f64,
    ) -> f64 {
        let Some(comp) = comp else {
            return para_y;
        };
        let mut offset_y = 0.0;
        for line in &comp.lines {
            let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
            let line_spacing_px = hwpunit_to_px(line.line_spacing, self.dpi);
            let max_fs = line
                .runs
                .iter()
                .map(|r| {
                    let ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                    if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    }
                })
                .fold(0.0f64, f64::max);
            if (raw_lh - shape_height_px).abs() <= 4.0 && raw_lh > max_fs * 2.0 {
                let runs_all_whitespace = line.runs.iter().all(|r| r.text.trim().is_empty());
                let label_extra = if !runs_all_whitespace {
                    max_fs + line_spacing_px.max(0.0)
                } else {
                    0.0
                };
                return para_y + offset_y + label_extra;
            }
            offset_y += raw_lh + line_spacing_px;
        }
        para_y
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
        let margin_left = styles
            .para_styles
            .get(para_style_id_for_ml)
            .map(|s| s.margin_left)
            .unwrap_or(0.0);
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
        let mut preceding_tac: Vec<(usize, f64)> = para
            .controls
            .iter()
            .enumerate()
            .filter_map(|(ci, ctrl)| {
                if ci >= control_index {
                    return None;
                }
                let tp = positions.get(ci).copied().unwrap_or(0);
                if tp == 0 || tp >= ctrl_text_pos {
                    return None;
                }
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
                        est_x,
                        &ts.tab_stops,
                        ts.default_tab_width,
                        ts.auto_tab_right,
                        ts.available_width,
                    );
                    est_x = tp;
                } else {
                    // [Task #555] PUA 옛한글 char 은 자모 시퀀스 폭으로 측정.
                    use super::pua_oldhangul::map_pua_old_hangul;
                    let metric_str: String = if let Some(jamos) = map_pua_old_hangul(ch) {
                        jamos.iter().copied().collect()
                    } else {
                        ch.to_string()
                    };
                    est_x += estimate_text_width(&metric_str, &ts);
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
        HorzAlign::Right | HorzAlign::Outside => col_area.x + col_area.width - tbl_w + h_offset,
        HorzAlign::Center => col_area.x + (col_area.width - tbl_w) / 2.0 + h_offset,
        _ => col_area.x + h_offset,
    };
    tbl_x + tbl_w
}

fn compute_tac_leading_width(
    composed: &ComposedParagraph,
    target_control_index: usize,
    styles: &ResolvedStyleSet,
) -> f64 {
    let Some(first_line) = composed.lines.first() else {
        return 0.0;
    };

    // target TAC 이 composed.tac_controls 에 있으면 해당 위치까지 합산.
    // 없으면(블록 취급: 너비 ≥ 90% seg_width 등 is_tac_table_inline 이 false 인 경우)
    // 선행 텍스트는 line 0 전체로 간주하고 모든 run 폭 합산.
    let tac_pos_opt = composed
        .tac_controls
        .iter()
        .find(|(_, _, ci)| *ci == target_control_index)
        .map(|(pos, _, _)| *pos);

    let mut char_pos = first_line.char_start;
    let mut width = 0.0;
    for run in &first_line.runs {
        let run_len = run.text.chars().count();
        let style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
        // [Task #555] PUA 옛한글 변환 후 폰트 매트릭스는 자모 시퀀스 기준.
        let effective_full = effective_text_for_metrics(run);
        match tac_pos_opt {
            Some(tac_pos) if char_pos + run_len <= tac_pos => {
                width += estimate_text_width(effective_full, &style);
                char_pos += run_len;
            }
            Some(tac_pos) if char_pos < tac_pos => {
                let partial_len = tac_pos - char_pos;
                // partial 추출은 run.text 기준 (인덱싱 불변성). 이후 PUA 변환 적용.
                let partial: String = run.text.chars().take(partial_len).collect();
                let partial_display: String = partial
                    .chars()
                    .flat_map(|ch| {
                        use super::pua_oldhangul::map_pua_old_hangul;
                        if let Some(jamos) = map_pua_old_hangul(ch) {
                            jamos.iter().copied().collect::<Vec<_>>()
                        } else {
                            vec![ch]
                        }
                    })
                    .collect();
                width += estimate_text_width(&partial_display, &style);
                break;
            }
            Some(_) => break,
            None => {
                // block 취급 TAC: 전체 run 합산
                width += estimate_text_width(effective_full, &style);
                char_pos += run_len;
            }
        }
    }
    width
}

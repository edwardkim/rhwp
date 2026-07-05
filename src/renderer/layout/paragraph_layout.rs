//! 문단 레이아웃 (인라인 표, 문단 전체/부분, composed/raw) + 번호 매기기

use super::super::composer::{
    compose_paragraph, effective_text_for_metrics, ComposedParagraph, ComposedTextRun,
};
use super::super::height_measurer::MeasuredTable;
use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{
    format_number, hwpunit_to_px, px_to_hwpunit, AutoNumberCounter, NumberFormat as NumFmt,
    ShapeStyle, TabStop, TextStyle,
};
use super::border_rendering::create_border_line_nodes;
use super::text_measurement::{
    compute_char_positions, estimate_text_width, extract_tab_leaders_with_extended,
    find_next_tab_stop, resolved_to_text_style,
};
use super::utils::{
    expand_numbering_format, extract_shape_transform, find_bin_data,
    numbering_format_to_number_format, picture_display_size_hu, resolve_numbering_id,
};
use super::{CellContext, LayoutEngine};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertRelTo};
use crate::model::style::{Alignment, HeadType, LineSpacingType, Numbering, UnderlineType};

const CAPTION_CELL_SENTINEL: usize = 65534;

/// `RHWP_LAYOUT_DEBUG=1` 로 활성화되는 layout 디버그 로깅 여부.
/// Phase 1 (#517) — 본질 정정 (#467/#491/#496) 시 결함 측정·재현 자동화에 사용.
#[inline]
pub(crate) fn layout_debug_enabled() -> bool {
    std::env::var("RHWP_LAYOUT_DEBUG")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// lineseg baseline_distance를 폰트 어센트 기준으로 보정한다.
/// CENTER 문단 수직정렬 등으로 baseline이 50% 이하로 설정된 경우,
/// 텍스트 어센트(~80%)가 줄 박스 밖으로 넘치지 않도록 보장한다.
pub(crate) fn ensure_min_baseline(raw_baseline: f64, max_font_size: f64) -> f64 {
    if max_font_size <= 0.0 {
        return raw_baseline;
    }
    let min_baseline = max_font_size * 0.8;
    raw_baseline.max(min_baseline)
}

fn paragraph_active_text_style(
    styles: &ResolvedStyleSet,
    para: Option<&Paragraph>,
    char_offset: usize,
) -> (TextStyle, Option<u32>) {
    let char_shape_id = para
        .and_then(|p| p.char_shape_id_at(char_offset))
        .or_else(|| para.and_then(|p| p.char_shapes.first().map(|cs| cs.char_shape_id)));

    if let Some(id) = char_shape_id {
        (resolved_to_text_style(styles, id, 0), Some(id))
    } else {
        (resolved_to_text_style(styles, 0, 0), None)
    }
}

fn numbering_marker_text_style(
    styles: &ResolvedStyleSet,
    para: Option<&Paragraph>,
    first_run: Option<&ComposedTextRun>,
) -> TextStyle {
    if let Some(run) = first_run {
        resolved_to_text_style(styles, run.char_style_id, run.lang_index)
    } else {
        paragraph_active_text_style(styles, para, 0).0
    }
}

fn para_float_horz_intersects_column(
    common: &CommonObjAttr,
    width_hu: i32,
    col_area: &LayoutRect,
    dpi: f64,
) -> bool {
    if !matches!(common.horz_rel_to, HorzRelTo::Column | HorzRelTo::Para) {
        return true;
    }

    let width_px = hwpunit_to_px(width_hu, dpi);
    let h_offset_px = hwpunit_to_px(common.horizontal_offset as i32, dpi);
    let left = match common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => col_area.x + h_offset_px,
        HorzAlign::Center => col_area.x + (col_area.width - width_px) / 2.0 + h_offset_px,
        HorzAlign::Right | HorzAlign::Outside => {
            col_area.x + col_area.width - width_px - h_offset_px
        }
    };
    let right = left + width_px;

    right > col_area.x + 0.5 && left < col_area.x + col_area.width - 0.5
}

fn has_para_topbottom_float_affecting_column(
    para: Option<&Paragraph>,
    col_area: &LayoutRect,
    dpi: f64,
) -> bool {
    para.map(|p| {
        p.controls.iter().any(|ctrl| match ctrl {
            Control::Picture(pic) => {
                !pic.common.treat_as_char
                    && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                    && matches!(pic.common.vert_rel_to, VertRelTo::Para)
                    && {
                        let (width_hu, _) = picture_display_size_hu(pic);
                        para_float_horz_intersects_column(&pic.common, width_hu, col_area, dpi)
                    }
            }
            Control::Shape(shape) => {
                let common = shape.common();
                !common.treat_as_char
                    && matches!(common.text_wrap, TextWrap::TopAndBottom)
                    && matches!(common.vert_rel_to, VertRelTo::Para)
                    && para_float_horz_intersects_column(common, common.width as i32, col_area, dpi)
            }
            _ => false,
        })
    })
    .unwrap_or(false)
}

fn tac_picture_or_shape_height_for_line(
    para: Option<&Paragraph>,
    raw_line_height: f64,
    dpi: f64,
) -> Option<f64> {
    let para = para?;
    para.controls.iter().find_map(|ctrl| {
        let height_hu = match ctrl {
            Control::Picture(pic) if pic.common.treat_as_char => pic.common.height as i32,
            Control::Shape(shape) if shape.common().treat_as_char => {
                let common_h = shape.common().height as i32;
                let current_h = shape.shape_attr().current_height as i32;
                common_h.max(current_h)
            }
            _ => return None,
        };
        let height = hwpunit_to_px(height_hu, dpi);
        if height > 8.0 && raw_line_height + 4.0 >= height && raw_line_height <= height + 8.0 {
            Some(height)
        } else {
            None
        }
    })
}

fn is_treat_as_char_equation_control(ctrl: Option<&Control>) -> bool {
    matches!(ctrl, Some(Control::Equation(eq)) if eq.common.treat_as_char)
}

fn is_caption_cell_context(cell_ctx: Option<&CellContext>) -> bool {
    cell_ctx
        .and_then(|ctx| ctx.path.last())
        .is_some_and(|entry| entry.cell_index == CAPTION_CELL_SENTINEL)
}

fn composed_line_char_end(comp: &ComposedParagraph, line_idx: usize) -> usize {
    if let Some(next) = comp.lines.get(line_idx + 1) {
        return next.char_start;
    }
    let Some(line) = comp.lines.get(line_idx) else {
        return 0;
    };
    line.char_start
        + line
            .runs
            .iter()
            .map(|run| run.text.chars().count())
            .sum::<usize>()
        + usize::from(line.has_line_break)
}

fn char_pos_in_line(pos: usize, start: usize, end: usize) -> bool {
    if end > start {
        pos >= start && pos < end
    } else {
        pos == start
    }
}

fn line_has_tac_control(comp: &ComposedParagraph, line_idx: usize) -> bool {
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let start = line.char_start;
    let end = comp
        .lines
        .get(line_idx + 1)
        .map(|next| next.char_start)
        .unwrap_or(usize::MAX);
    comp.tac_controls
        .iter()
        .any(|(pos, _, _)| char_pos_in_line(*pos, start, end))
}

fn line_has_strict_tac_control(
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> bool {
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let start = line.char_start;
    let end = composed_line_char_end(comp, line_idx);
    end > start
        && tac_offsets_px
            .iter()
            .any(|(pos, _, _)| *pos >= start && *pos < end)
}

fn line_has_strict_equation_tac_control(
    para: Option<&Paragraph>,
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> bool {
    let Some(para) = para else {
        return false;
    };
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let start = line.char_start;
    let end = composed_line_char_end(comp, line_idx);
    end > start
        && tac_offsets_px.iter().any(|(pos, _, ci)| {
            *pos >= start && *pos < end && is_treat_as_char_equation_control(para.controls.get(*ci))
        })
}

fn line_is_leading_empty_equation_tac_guide(
    para: Option<&Paragraph>,
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> bool {
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let Some(next) = comp.lines.get(line_idx + 1) else {
        return false;
    };
    line.runs.is_empty()
        && line.char_start == next.char_start
        && !line_has_strict_tac_control(comp, tac_offsets_px, line_idx)
        && line_has_strict_equation_tac_control(para, comp, tac_offsets_px, line_idx + 1)
}

/// [#1925 추출] `layout_empty_runs_line` 줄-스코프 스칼라 입력 묶음.
#[derive(Clone, Copy)]
struct EmptyRunsLineVars {
    alignment: crate::model::style::Alignment,
    available_width: f64,
    effective_col_x: f64,
    effective_margin_left: f64,
    x_start: f64,
    /// 이 줄 끝의 문서 char 좌표 (원본: char_offset)
    line_char_end: usize,
    y: f64,
    baseline: f64,
    raw_lh: f64,
    runs_all_whitespace: bool,
    max_fs: f64,
    line_spacing_px: f64,
    /// para_topbottom_line_vpos_base.is_some()
    has_topbottom_vpos_base: bool,
    is_last_line_of_para: bool,
    defer_empty_line_control_marker: bool,
    line_flow_height: f64,
    section_index: usize,
    para_index: usize,
}
/// [#1925 추출] `estimate_line_run_widths` 결과 — est 사전 폭 추정 산출물.
struct LineWidthEst {
    /// 추정 종료 x (초기값 기준 누적 점유 폭 계산용)
    est_x: f64,
    /// 추정에 포함된 tac 개체 폭 합
    included_tac_width: f64,
}
fn tac_offsets_for_line(
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> Vec<(usize, f64, usize)> {
    let Some(line) = comp.lines.get(line_idx) else {
        return Vec::new();
    };
    let start = line.char_start;
    let end = composed_line_char_end(comp, line_idx);
    tac_offsets_px
        .iter()
        .copied()
        .filter(|(pos, _, _)| char_pos_in_line(*pos, start, end))
        .collect()
}

fn repeated_empty_tac_line_offset(
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> Option<Vec<(usize, f64, usize)>> {
    let line = comp.lines.get(line_idx)?;
    if !line.runs.is_empty() {
        return None;
    }

    let start = line.char_start;
    let repeated_empty_line_count = comp
        .lines
        .iter()
        .filter(|candidate| candidate.runs.is_empty() && candidate.char_start == start)
        .count();
    if repeated_empty_line_count <= 1 {
        return None;
    }

    let line_ordinal = comp
        .lines
        .iter()
        .take(line_idx)
        .filter(|candidate| candidate.runs.is_empty() && candidate.char_start == start)
        .count();
    let line_tac_sequence = tac_offsets_px
        .iter()
        .copied()
        .filter(|(pos, _, _)| *pos >= start && *pos < start + repeated_empty_line_count)
        .collect::<Vec<_>>();

    // 텍스트 없는 HWP 문단은 LINE_SEG 여러 줄이 같은 text_start 를 가질 수 있다.
    // 이때 TAC 개수와 빈 줄 수가 정확히 맞으면 한 줄에 하나씩 순서대로 배정한다.
    if line_tac_sequence.len() == repeated_empty_line_count {
        line_tac_sequence
            .get(line_ordinal)
            .copied()
            .map(|offset| vec![offset])
    } else {
        None
    }
}

fn tac_picture_or_shape_height_px(ctrl: &Control, dpi: f64) -> Option<f64> {
    let height_hu = match ctrl {
        Control::Picture(pic) if pic.common.treat_as_char => pic.common.height as i32,
        Control::Shape(shape) if shape.common().treat_as_char => {
            let common_h = shape.common().height as i32;
            let current_h = shape.shape_attr().current_height as i32;
            common_h.max(current_h)
        }
        _ => return None,
    };
    Some(hwpunit_to_px(height_hu, dpi))
}

fn note_number_format_from_hwp_code(code: u8) -> NumFmt {
    match code {
        0 => NumFmt::Digit,
        1 => NumFmt::CircledDigit,
        2 => NumFmt::RomanUpper,
        3 => NumFmt::RomanLower,
        4 => NumFmt::LatinUpper,
        5 => NumFmt::LatinLower,
        8 => NumFmt::HangulGaNaDa,
        12 => NumFmt::HangulNumber,
        13 => NumFmt::HanjaNumber,
        _ => NumFmt::Digit,
    }
}

fn note_decoration_char(value: u16) -> Option<char> {
    if value == 0 {
        None
    } else {
        char::from_u32(value as u32).filter(|ch| *ch != '\0')
    }
}

fn format_note_marker_text(
    number: u16,
    number_shape: u32,
    before_decoration_letter: u16,
    after_decoration_letter: u16,
) -> String {
    let number = format_number(number, note_number_format_from_hwp_code(number_shape as u8));
    let prefix = note_decoration_char(before_decoration_letter)
        .map(|ch| ch.to_string())
        .unwrap_or_default();
    let suffix = note_decoration_char(after_decoration_letter)
        .unwrap_or(')')
        .to_string();
    format!("{}{}{}", prefix, number, suffix)
}

fn note_marker_text_from_control(ctrl: Option<&Control>, fallback_number: u16) -> String {
    match ctrl {
        Some(Control::Footnote(footnote)) => format_note_marker_text(
            fallback_number,
            footnote.number_shape,
            footnote.before_decoration_letter,
            footnote.after_decoration_letter,
        ),
        Some(Control::Endnote(endnote)) => format_note_marker_text(
            fallback_number,
            endnote.number_shape,
            endnote.before_decoration_letter,
            endnote.after_decoration_letter,
        ),
        _ => format!("{})", fallback_number),
    }
}

fn is_leading_endnote_marker_rendered_as_prefix(
    para: Option<&Paragraph>,
    control_index: usize,
    line_idx: usize,
    start_line: usize,
    marker_pos: usize,
    line_char_start: usize,
) -> bool {
    line_idx == start_line
        && start_line == 0
        && marker_pos == line_char_start
        && matches!(
            para.and_then(|p| p.controls.get(control_index)),
            Some(Control::Endnote(_))
        )
}

fn line_tac_picture_or_shape_height(
    para: Option<&Paragraph>,
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
    dpi: f64,
) -> Option<f64> {
    let para = para?;
    tac_offsets_for_line(comp, tac_offsets_px, line_idx)
        .iter()
        .find_map(|(_, _, ci)| {
            para.controls
                .get(*ci)
                .and_then(|ctrl| tac_picture_or_shape_height_px(ctrl, dpi))
        })
}

fn text_line_is_picture_lead_in(
    para: Option<&Paragraph>,
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
    raw_lh: f64,
    max_fs: f64,
    dpi: f64,
) -> bool {
    if max_fs <= 0.0 || raw_lh <= max_fs * 2.0 {
        return false;
    }
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    if line.runs.iter().all(|run| run.text.trim().is_empty())
        || line_tac_picture_or_shape_height(para, comp, tac_offsets_px, line_idx, dpi).is_some()
    {
        return false;
    }
    let Some(next) = comp.lines.get(line_idx + 1) else {
        return false;
    };
    if !next.runs.iter().all(|run| run.text.trim().is_empty()) {
        return false;
    }
    line_tac_picture_or_shape_height(para, comp, tac_offsets_px, line_idx + 1, dpi)
        .map(|height| (raw_lh - height).abs() <= 8.0)
        .unwrap_or(false)
}

fn has_treat_as_char_picture_or_shape(para: Option<&Paragraph>) -> bool {
    para.map(|para| {
        para.controls.iter().any(|ctrl| {
            matches!(
                ctrl,
                Control::Picture(pic) if pic.common.treat_as_char
            ) || matches!(
                ctrl,
                Control::Shape(shape) if shape.common().treat_as_char
            )
        })
    })
    .unwrap_or(false)
}

fn is_blank_spacer_line(
    para: Option<&Paragraph>,
    is_endnote_virtual_para: bool,
    runs_all_whitespace: bool,
    line_tac_offsets: &[(usize, f64, usize)],
) -> bool {
    if !runs_all_whitespace || !line_tac_offsets.is_empty() {
        return false;
    }
    is_endnote_virtual_para || para.map(|p| p.controls.is_empty()).unwrap_or(false)
}

fn is_equation_only_tac_line(
    para: Option<&Paragraph>,
    runs_all_whitespace: bool,
    line_tac_offsets: &[(usize, f64, usize)],
) -> bool {
    let Some(para) = para else {
        return false;
    };
    runs_all_whitespace
        && !line_tac_offsets.is_empty()
        && line_tac_offsets
            .iter()
            .all(|(_, _, ci)| is_treat_as_char_equation_control(para.controls.get(*ci)))
}

fn tac_picture_label_extra_px(
    runs_all_whitespace: bool,
    raw_line_height: f64,
    reserved_picture_height: Option<f64>,
    max_font_size: f64,
    line_spacing_px: f64,
) -> f64 {
    let Some(pic_h) = reserved_picture_height else {
        return 0.0;
    };
    if runs_all_whitespace || max_font_size <= 0.0 {
        return 0.0;
    }
    if (raw_line_height - pic_h).abs() > 4.0 || raw_line_height <= max_font_size * 2.0 {
        return 0.0;
    }
    max_font_size + line_spacing_px.max(0.0)
}

fn tac_picture_label_extra_for_line(
    _cell_ctx: Option<&CellContext>,
    runs_all_whitespace: bool,
    raw_line_height: f64,
    reserved_picture_height: Option<f64>,
    max_font_size: f64,
    line_spacing_px: f64,
) -> f64 {
    // #1352/#1486: "TAC picture + 실제 텍스트" 줄은 한컴 PDF 기준
    // picture와 텍스트가 같은 세로 위치에 놓인다. label 보정은 TAC-only 라인에만 남긴다.
    if !runs_all_whitespace {
        return 0.0;
    }
    tac_picture_label_extra_px(
        runs_all_whitespace,
        raw_line_height,
        reserved_picture_height,
        max_font_size,
        line_spacing_px,
    )
}

/// run 이 `\t` 로 끝날 때, 그 마지막 `\t` 가 cross-run 우측/가운데 탭으로 동작해야 하는지 판정한다.
///
/// HWP 본문 탭에는 두 가지 정보원이 있다:
/// - `tab_extended` (inline tab): `ext[2]` 고바이트 = 탭 종류 (1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL)
/// - `TabDef` (문단 모양의 탭 정의): 절대 위치 + type/fill
///
/// inline 이 커버하는 `\t` 는 inline 의 종류가 우선이며, LEFT 이면 cross-run 재배치 없음.
/// inline 이 비었거나 `\t` 인덱스를 초과하는 경우에만 `find_next_tab_stop` 기반 TabDef 폴백으로 판정한다.
///
/// 반환 `Some((tab_pos, tab_type, fill_type))` 은 `pending_right_tab_*` 에 그대로 대입 가능 (tab_type ∈ {1, 2}).
/// fill_type 은 호출 측에서 리더(점선/실선/파선 등) 가 있는 RIGHT 탭을 단 우측 끝으로 보정하는 용도.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_last_tab_pending(
    run_text: &str,
    last_inline_idx: usize,
    tab_extended: &[[u16; 7]],
    text_style: &TextStyle,
    tab_stops: &[TabStop],
    tab_width: f64,
    auto_tab_right: bool,
    available_width: f64,
) -> Option<(f64, u8, u8)> {
    // 1) inline_tabs 가 마지막 \t 를 커버하는 경우: ext[2] 고바이트로 종류 판정
    if last_inline_idx < tab_extended.len() {
        let inline_type = ((tab_extended[last_inline_idx][2] >> 8) & 0xFF) as u8;
        match inline_type {
            // 1=LEFT (explicit), 0=unspecified → cross-run pending 없음 (본 수정의 핵심)
            0 | 1 => return None,
            // 2=RIGHT, 3=CENTER → TabDef 기반 위치 계산으로 폴스루
            2 | 3 => {}
            // 미지 값 (4=DECIMAL 등) → 보수적으로 LEFT 취급
            _ => return None,
        }
    }

    // 2) inline 이 LEFT 아님 (RIGHT/CENTER) 또는 inline 없음 → TabDef find_next_tab_stop 으로 판정
    let last_tab_byte = run_text.rfind('\t')?;
    let text_before = &run_text[..last_tab_byte];
    let w_before = estimate_text_width(text_before, text_style);
    let abs_before = text_style.line_x_offset + w_before;
    let tw = if tab_width > 0.0 { tab_width } else { 48.0 };
    let (tp, tt, ft) =
        find_next_tab_stop(abs_before, tab_stops, tw, auto_tab_right, available_width);
    if tt == 1 || tt == 2 {
        Some((tp, tt, ft))
    } else {
        None
    }
}

/// 우측/가운데 탭 정렬 단위의 폭(px).
///
/// 탭 직후 run(`start`)부터 `\t` 를 포함하지 않는 연속 run 들의 `estimate_text_width` 합산.
/// composer(`split_runs_by_lang` / `split_by_char_shapes`)가 char-shape·스크립트 경계로 run 을
/// 쪼개므로(예: `"Ctrl+(회색)5"` → `["Ctrl+(", "회색)", "5"]`), 탭 직후 한 개 run 폭만 쓰면
/// 나머지 run 이 탭스톱 우측으로 흘러넘친다 (Issue #842, 결함 #4).
#[allow(clippy::too_many_arguments)]
pub(crate) fn right_tab_block_width(
    runs: &[crate::renderer::composer::ComposedTextRun],
    start: usize,
    styles: &ResolvedStyleSet,
    default_tab_width: f64,
    tab_stops: &[TabStop],
    auto_tab_right: bool,
    available_width: f64,
) -> f64 {
    let mut w = 0.0;
    for r in runs.iter().skip(start) {
        if r.text.contains('\t') {
            break;
        }
        if let Some(_ov) = &r.char_overlap {
            let chars: Vec<char> = r.text.chars().collect();
            let fs = {
                let ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                if ts.font_size > 0.0 {
                    ts.font_size
                } else {
                    12.0
                }
            };
            w += fs * crate::renderer::composer::char_overlap_advance_units(&chars) as f64;
            continue;
        }
        let mut ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
        ts.default_tab_width = default_tab_width;
        ts.tab_stops = tab_stops.to_vec();
        ts.auto_tab_right = auto_tab_right;
        ts.available_width = available_width;
        // [Task #874] text_start_offset 은 right_tab_block_width 가 측정만 하므로
        // 영향 없음 — 0 그대로.
        w += estimate_text_width(effective_text_for_metrics(r), &ts);
    }
    w
}

impl LayoutEngine {
    pub(crate) fn layout_inline_table_paragraph(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        y_start: f64,
        section_index: usize,
        para_index: usize,
        bin_data_content: &[BinDataContent],
        measured_tables: &[MeasuredTable],
    ) -> f64 {
        use crate::model::control::Control;

        // 1. 문단 스타일 조회
        let para_style_id = composed
            .map(|c| c.para_style_id as usize)
            .unwrap_or(para.para_shape_id as usize);
        let para_style = styles.para_styles.get(para_style_id);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let spacing_before = crate::renderer::hwp3_variant_flow_spacing_before(
            para_style.map(|s| s.spacing_before).unwrap_or(0.0),
            self.use_hwp3_origin_flow_spacing_before.get(),
        );
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);
        let alignment = para_style.map(|s| s.alignment).unwrap_or(Alignment::Left);

        // 2. treat_as_char 표 목록과 폭 수집
        let inline_tables: Vec<(usize, &crate::model::table::Table)> = para
            .controls
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if let Control::Table(t) = c {
                    if t.common.treat_as_char {
                        return Some((i, t.as_ref()));
                    }
                }
                None
            })
            .collect();
        let flow_anchor_y = y_start + spacing_before;
        let has_detached_para_object = inline_tables.iter().any(|(_, table)| {
            table
                .cells
                .iter()
                .flat_map(|cell| cell.paragraphs.iter())
                .flat_map(|p| p.controls.iter())
                .any(|ctrl| match ctrl {
                    Control::Picture(pic) => {
                        !pic.common.treat_as_char
                            && !pic.common.flow_with_text
                            && matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(
                                pic.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            )
                    }
                    Control::Shape(shape) => {
                        let common = shape.common();
                        !common.treat_as_char
                            && !common.flow_with_text
                            && matches!(
                                common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                    }
                    _ => false,
                })
        });
        let inline_table_line_shift = if has_detached_para_object {
            para.line_segs
                .first()
                .filter(|seg| seg.vertical_pos > 0)
                .map(|seg| hwpunit_to_px(seg.vertical_pos, self.dpi))
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let y = flow_anchor_y + inline_table_line_shift;
        let table_para_y = if inline_table_line_shift > 0.0 {
            Some(flow_anchor_y)
        } else {
            None
        };

        // [Task #517 Stage 1] RHWP_LAYOUT_DEBUG 진단 로깅
        if layout_debug_enabled() {
            eprintln!(
                "LAYOUT_INLINE_TABLE_PARA: pi={} sec={} col_x={:.1} col_w={:.1} y_start={:.1} y={:.1} sb={:.1} sa={:.1} ml={:.1} mr={:.1} align={:?} ls_count={} tables={}",
                para_index, section_index, col_area.x, col_area.width, y_start, y,
                spacing_before, spacing_after, margin_left, margin_right, alignment,
                para.line_segs.len(), inline_tables.len(),
            );
            for (li, seg) in para.line_segs.iter().enumerate() {
                eprintln!(
                    "  LAYOUT_LS[{}]: vpos={} lh={} ls={} bl={} text_start={} sw={}",
                    li,
                    seg.vertical_pos,
                    seg.line_height,
                    seg.line_spacing,
                    seg.baseline_distance,
                    seg.text_start,
                    seg.segment_width,
                );
            }
            for (ti, (ci, tbl)) in inline_tables.iter().enumerate() {
                eprintln!(
                    "  LAYOUT_INLINE_TBL[{}]: ctrl_idx={} rows={} cols={} w={} h={} vert={:?} horz={:?} wrap={:?}",
                    ti, ci, tbl.row_count, tbl.col_count,
                    tbl.common.width, tbl.common.height,
                    tbl.common.vert_align, tbl.common.horz_align, tbl.common.text_wrap,
                );
            }
        }

        // 3. char_offsets 갭 분석으로 텍스트 세그먼트 분할
        // 확장 컨트롤은 8 UTF-16 코드 유닛을 차지
        let text_chars: Vec<char> = para.text.chars().collect();
        let offsets = &para.char_offsets;

        // 텍스트 세그먼트 분리: 갭이 8 이상이면 컨트롤 위치
        let mut segments: Vec<(usize, usize)> = Vec::new(); // (start_char_idx, end_char_idx)

        // 선행 컨트롤 감지: 첫 텍스트 문자 앞에 컨트롤이 있으면 빈 세그먼트 추가
        // 확장 컨트롤은 8 UTF-16 유닛을 차지하므로, offsets[0] / 8 = 선행 컨트롤 수
        if !offsets.is_empty() && offsets[0] >= 8 {
            let num_leading = (offsets[0] / 8) as usize;
            let tables_to_prepend = num_leading.min(inline_tables.len());
            for _ in 0..tables_to_prepend {
                segments.push((0, 0)); // 빈 세그먼트 → 표가 텍스트 앞에 배치됨
            }
        }

        let mut seg_start = 0;
        for i in 1..offsets.len() {
            let prev_char_utf16_len = if text_chars[i - 1] >= '\u{10000}' {
                2u32
            } else {
                1
            };
            let gap = offsets[i] - offsets[i - 1];
            if gap > prev_char_utf16_len + 4 {
                // 갭에 컨트롤이 있음
                segments.push((seg_start, i));
                seg_start = i;
            }
        }
        segments.push((seg_start, text_chars.len()));

        // 배치 순서: segment[0], table[0], segment[1], table[1], ...
        // 선행 컨트롤이 있으면: empty_seg, table[0], text_seg, table[1], ...

        // 4. 각 요소의 폭 계산
        // 4a. 표 폭 계산
        let table_widths: Vec<f64> = inline_tables
            .iter()
            .map(|(_, t)| {
                // col_widths로부터 table_width 계산
                let col_count = t.col_count as usize;
                let cell_spacing = hwpunit_to_px(t.cell_spacing as i32, self.dpi);
                let mut col_widths = vec![0.0f64; col_count];
                for cell in &t.cells {
                    let c = cell.col as usize;
                    let span = cell.col_span.max(1) as usize;
                    if c + span <= col_count {
                        let w = hwpunit_to_px(cell.width as i32, self.dpi);
                        if span == 1 {
                            if w > col_widths[c] {
                                col_widths[c] = w;
                            }
                        }
                    }
                }
                let total: f64 = col_widths.iter().sum::<f64>()
                    + cell_spacing * (col_count.saturating_sub(1) as f64);
                total
            })
            .collect();

        // 4b. 텍스트 세그먼트 폭 계산
        let char_style_id = para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id as u32)
            .unwrap_or(0);

        let seg_widths: Vec<f64> = segments
            .iter()
            .map(|(s, e)| {
                let seg_text: String = text_chars[*s..*e].iter().collect();
                if seg_text.is_empty() {
                    return 0.0;
                }
                // 세그먼트 내 char_shape 변경을 고려한 폭 계산
                let mut total = 0.0;
                for ch_idx in *s..*e {
                    // 해당 문자의 char_shape 찾기
                    let utf16_pos = offsets[ch_idx];
                    let cs_id = para
                        .char_shapes
                        .iter()
                        .rev()
                        .find(|cs| cs.start_pos <= utf16_pos)
                        .map(|cs| cs.char_shape_id as u32)
                        .unwrap_or(char_style_id);
                    let ch = map_pua_bullet_char(text_chars[ch_idx]);
                    let lang = super::super::style_resolver::detect_lang_category(ch);
                    let ts = resolved_to_text_style(styles, cs_id, lang);
                    total += estimate_text_width(&ch.to_string(), &ts);
                }
                total
            })
            .collect();

        // 5. 총 폭과 정렬 계산
        let total_width: f64 = seg_widths.iter().sum::<f64>() + table_widths.iter().sum::<f64>();
        let available_width = col_area.width - margin_left - margin_right;
        let start_x = match alignment {
            Alignment::Center | Alignment::Distribute => {
                col_area.x + margin_left + (available_width - total_width).max(0.0) / 2.0
            }
            Alignment::Right => col_area.x + margin_left + (available_width - total_width).max(0.0),
            _ => col_area.x + margin_left,
        };

        // 6. 줄 높이 계산 (line_seg 기반)
        // line_seg[0]은 표를 포함한 줄 (표 높이 반영), line_seg[1]은 텍스트 줄
        let line_height = if let Some(ls) = para.line_segs.first() {
            hwpunit_to_px(ls.line_height, self.dpi)
        } else {
            hwpunit_to_px(400, self.dpi)
        };
        let line_spacing = if let Some(ls) = para.line_segs.first() {
            hwpunit_to_px(ls.line_spacing, self.dpi)
        } else {
            0.0
        };
        // 폰트 어센트 보정용: 문단 내 최대 폰트 크기
        let para_max_font_size = {
            let default_cs = para
                .char_shapes
                .first()
                .map(|cs| cs.char_shape_id as u32)
                .unwrap_or(0);
            let ts = resolved_to_text_style(styles, default_cs, 0);
            if ts.font_size > 0.0 {
                ts.font_size
            } else {
                12.0
            }
        };
        let baseline_dist = if let Some(ls) = para.line_segs.first() {
            ensure_min_baseline(
                hwpunit_to_px(ls.baseline_distance, self.dpi),
                para_max_font_size,
            )
        } else {
            line_height * 0.8
        };
        // 텍스트 줄(표 아래) 전용 메트릭: line_seg[1]이 있으면 사용
        let text_line_baseline = if let Some(ls) = para.line_segs.get(1) {
            ensure_min_baseline(
                hwpunit_to_px(ls.baseline_distance, self.dpi),
                para_max_font_size,
            )
        } else {
            baseline_dist
        };
        let text_line_height = if let Some(ls) = para.line_segs.get(1) {
            hwpunit_to_px(ls.line_height, self.dpi)
        } else {
            line_height
        };
        let text_line_spacing = if let Some(ls) = para.line_segs.get(1) {
            hwpunit_to_px(ls.line_spacing, self.dpi)
        } else {
            line_spacing
        };

        // 7. 가로 배치: 텍스트 세그먼트와 표를 순차 배치
        let right_margin = col_area.x + col_area.width - margin_right;
        let line_start_x = col_area.x + margin_left;
        // 텍스트 줄바꿈 시 줄 높이: line_seg[0]은 표 높이를 포함하므로
        // line_seg[1]이 있으면 사용 (텍스트 줄 높이), 없으면 baseline_dist 기반
        let line_step = if para.line_segs.len() > 1 {
            let ls = &para.line_segs[1];
            hwpunit_to_px(ls.line_height, self.dpi) + hwpunit_to_px(ls.line_spacing, self.dpi)
        } else if let Some(ls) = para.line_segs.first() {
            hwpunit_to_px(ls.line_height, self.dpi) + hwpunit_to_px(ls.line_spacing, self.dpi)
        } else {
            baseline_dist * 1.5
        };

        // [Task #518 Phase 2] LINE_SEG 기반 줄 나눔 위치 결정:
        // ls[1..] 의 text_start (raw UTF-16 위치, controls 포함) 를 char index 로 변환.
        // char_offsets[i] = text_chars[i] 의 원본 UTF-16 위치 → char_offsets[i] >= ts 인 첫 i 가 break.
        //
        // 이전: ctrl_gap 을 paragraph 전체 controls 합으로 over-subtract → controls 가 있는
        // paragraph 에서 saturating 0 으로 항상 break 미감지 (#496 케이스).
        // 이전: ls[1] 만 사용. 다중 줄 paragraph 에서 ls[2..] 무시 → dynamic reflow.
        let line_break_char_indices: Vec<usize> =
            if para.line_segs.len() > 1 && !para.char_offsets.is_empty() {
                let mut indices: Vec<usize> = Vec::new();
                for ls in para.line_segs.iter().skip(1) {
                    let ts = ls.text_start as u32;
                    // char_offsets[i] >= ts 인 첫 i (= text_chars 의 break 위치)
                    let char_idx = para
                        .char_offsets
                        .iter()
                        .position(|&off| off >= ts)
                        .unwrap_or(text_chars.len());
                    if char_idx > 0 && char_idx <= text_chars.len() {
                        // 단조 증가 보장 (이전 break 보다 큰 경우에만 추가)
                        if indices.last().map(|&prev| char_idx > prev).unwrap_or(true) {
                            indices.push(char_idx);
                        }
                    }
                }
                indices
            } else {
                Vec::new()
            };
        if layout_debug_enabled() {
            eprintln!(
                "  LAYOUT_BREAK_INDICES: pi={} indices={:?} (from ls[1..])",
                para_index, line_break_char_indices,
            );
        }

        let mut inline_x = start_x;
        let mut current_y = y;
        let mut table_idx = 0;
        let mut max_table_bottom = y; // 표의 최대 하단 y (표 높이를 줄 높이로 사용하기 위함)
        let mut wrapped_below_table = false; // 텍스트가 표 아래로 줄바꿈되었는지
                                             // [Task #518] 다음 break 인덱스 (line_break_char_indices 안에서)
        let mut next_break: usize = 0;

        for (s, e) in &segments {
            // 텍스트 세그먼트 렌더링 (줄바꿈 지원)
            if *s < *e {
                let seg_text: String = text_chars[*s..*e].iter().collect();
                if !seg_text.is_empty() {
                    // 문자별로 처리하며 줄바꿈 판단
                    let mut run_start = *s;
                    let mut line_run_start = *s; // 현재 줄 run의 시작
                    let mut line_run_x = inline_x; // 현재 줄 run의 x 시작
                    let mut current_cs_id = {
                        let utf16_pos = offsets[*s];
                        para.char_shapes
                            .iter()
                            .rev()
                            .find(|cs| cs.start_pos <= utf16_pos)
                            .map(|cs| cs.char_shape_id as u32)
                            .unwrap_or(char_style_id)
                    };

                    for ch_idx in *s..*e {
                        // 각주 마커 삽입: 현재 문자 위치에 각주가 있으면 먼저 run flush + FootnoteMarker 노드 삽입
                        if let Some(&(_, fn_num, fn_ctrl_idx)) = composed.and_then(|c| {
                            c.footnote_positions
                                .iter()
                                .find(|&&(pos, _, _)| pos == ch_idx)
                        }) {
                            // 현재까지 누적된 run 출력
                            if ch_idx > line_run_start {
                                let run_text: String =
                                    text_chars[line_run_start..ch_idx].iter().collect();
                                let first_lang = super::super::style_resolver::detect_lang_category(
                                    text_chars[line_run_start],
                                );
                                let run_ts =
                                    resolved_to_text_style(styles, current_cs_id, first_lang);
                                let run_width = estimate_text_width(&run_text, &run_ts);
                                let run_bbox_h = if wrapped_below_table {
                                    text_line_baseline
                                } else {
                                    baseline_dist
                                };
                                let run_id = tree.next_id();
                                let run_node = RenderNode::new(
                                    run_id,
                                    RenderNodeType::TextRun(TextRunNode {
                                        text: run_text,
                                        style: run_ts,
                                        char_shape_id: Some(current_cs_id),
                                        para_shape_id: Some(para_style_id as u16),
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        char_start: Some(line_run_start),
                                        cell_context: None,
                                        is_para_end: false,
                                        is_line_break_end: false,
                                        rotation: 0.0,
                                        is_vertical: false,
                                        char_overlap: None,
                                        border_fill_id: styles
                                            .char_styles
                                            .get(current_cs_id as usize)
                                            .map(|cs| cs.border_fill_id)
                                            .unwrap_or(0),
                                        baseline: run_bbox_h,
                                        field_marker: FieldMarkerType::None,
                                    }),
                                    BoundingBox::new(line_run_x, current_y, run_width, run_bbox_h),
                                );
                                col_node.children.push(run_node);
                                inline_x += run_width;
                                line_run_x = inline_x;
                                line_run_start = ch_idx;
                            }
                            // FootnoteMarker 노드 삽입 (위첨자로 렌더링됨)
                            let fn_text = note_marker_text_from_control(
                                para.controls.get(fn_ctrl_idx),
                                fn_num,
                            );
                            let base_ts = resolved_to_text_style(styles, current_cs_id, 0);
                            let sup_font_size = (base_ts.font_size * 0.55).max(7.0);
                            let sup_ts = TextStyle {
                                font_size: sup_font_size,
                                font_family: base_ts.font_family.clone(),
                                ..Default::default()
                            };
                            let sup_w = estimate_text_width(&fn_text, &sup_ts);
                            let run_bbox_h = if wrapped_below_table {
                                text_line_baseline
                            } else {
                                baseline_dist
                            };
                            let marker_id = tree.next_id();
                            let marker_node = RenderNode::new(
                                marker_id,
                                RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
                                    number: fn_num,
                                    text: fn_text,
                                    base_font_size: base_ts.font_size,
                                    font_family: base_ts.font_family.clone(),
                                    color: base_ts.color,
                                    section_index,
                                    para_index,
                                    control_index: fn_ctrl_idx,
                                }),
                                BoundingBox::new(inline_x, current_y, sup_w, run_bbox_h),
                            );
                            col_node.children.push(marker_node);
                            inline_x += sup_w;
                            line_run_x = inline_x;
                        }

                        let utf16_pos = offsets[ch_idx];
                        let cs_id = para
                            .char_shapes
                            .iter()
                            .rev()
                            .find(|cs| cs.start_pos <= utf16_pos)
                            .map(|cs| cs.char_shape_id as u32)
                            .unwrap_or(char_style_id);

                        let ch = text_chars[ch_idx];
                        let lang = super::super::style_resolver::detect_lang_category(ch);
                        let ts = resolved_to_text_style(styles, cs_id, lang);
                        let ch_w = estimate_text_width(&ch.to_string(), &ts);

                        // char_shape 변경 또는 줄바꿈 시 누적된 run을 출력
                        // [Task #518] LINE_SEG 기반 줄 나눔: ls[1..] 의 text_start 위치 모두 사용.
                        // break 가 모두 소진되거나 미존재 시 right_margin 동적 reflow 로 fallback.
                        let need_wrap = if next_break < line_break_char_indices.len()
                            && ch_idx >= line_break_char_indices[next_break]
                        {
                            next_break += 1;
                            true
                        } else {
                            inline_x + ch_w > right_margin + 0.5 && inline_x > line_start_x + 1.0
                        };
                        let cs_changed = cs_id != current_cs_id;

                        // 줄바꿈된 텍스트의 BoundingBox 높이: 표 줄 vs 텍스트 줄
                        let run_bbox_h = if wrapped_below_table {
                            text_line_baseline
                        } else {
                            baseline_dist
                        };

                        if (cs_changed || need_wrap) && ch_idx > line_run_start {
                            // 누적된 run 출력
                            let run_text: String =
                                text_chars[line_run_start..ch_idx].iter().collect();
                            let first_lang = super::super::style_resolver::detect_lang_category(
                                text_chars[line_run_start],
                            );
                            let run_ts = resolved_to_text_style(styles, current_cs_id, first_lang);
                            let run_width = estimate_text_width(&run_text, &run_ts);

                            let run_id = tree.next_id();
                            let run_node = RenderNode::new(
                                run_id,
                                RenderNodeType::TextRun(TextRunNode {
                                    text: run_text,
                                    style: run_ts,
                                    char_shape_id: Some(current_cs_id),
                                    para_shape_id: Some(para_style_id as u16),
                                    section_index: Some(section_index),
                                    para_index: Some(para_index),
                                    char_start: Some(line_run_start),
                                    cell_context: None,
                                    is_para_end: false,
                                    is_line_break_end: false,
                                    rotation: 0.0,
                                    is_vertical: false,
                                    char_overlap: None,
                                    border_fill_id: styles
                                        .char_styles
                                        .get(current_cs_id as usize)
                                        .map(|cs| cs.border_fill_id)
                                        .unwrap_or(0),
                                    baseline: run_bbox_h,
                                    field_marker: FieldMarkerType::None,
                                }),
                                BoundingBox::new(line_run_x, current_y, run_width, run_bbox_h),
                            );
                            col_node.children.push(run_node);
                            line_run_start = ch_idx;
                            line_run_x = inline_x;
                        }

                        if need_wrap {
                            // 줄바꿈: 표 아래로 넘어가는 경우 표 하단 기준 배치
                            if !wrapped_below_table && max_table_bottom > y {
                                // 첫 번째 줄바꿈 시 표 아래로 이동
                                // HWP: 표 너비로 인한 텍스트 오버플로우에는 줄간격 미적용
                                // (텍스트만의 오버플로우에는 줄간격 적용)
                                current_y = max_table_bottom;
                                wrapped_below_table = true;
                            } else {
                                current_y += line_step;
                            }
                            inline_x = line_start_x;
                            line_run_x = inline_x;
                        }

                        current_cs_id = cs_id;
                        inline_x += ch_w;
                    }

                    // 남은 run의 BoundingBox 높이
                    let remaining_bbox_h = if wrapped_below_table {
                        text_line_baseline
                    } else {
                        baseline_dist
                    };

                    // 남은 run 출력
                    if line_run_start < *e {
                        let run_text: String = text_chars[line_run_start..*e].iter().collect();
                        let first_lang = super::super::style_resolver::detect_lang_category(
                            text_chars[line_run_start],
                        );
                        let run_ts = resolved_to_text_style(styles, current_cs_id, first_lang);
                        let run_width = estimate_text_width(&run_text, &run_ts);

                        let run_id = tree.next_id();
                        let run_node = RenderNode::new(
                            run_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: run_text,
                                style: run_ts,
                                char_shape_id: Some(current_cs_id),
                                para_shape_id: Some(para_style_id as u16),
                                section_index: Some(section_index),
                                para_index: Some(para_index),
                                char_start: Some(line_run_start),
                                cell_context: None,
                                is_para_end: false,
                                is_line_break_end: false,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: styles
                                    .char_styles
                                    .get(current_cs_id as usize)
                                    .map(|cs| cs.border_fill_id)
                                    .unwrap_or(0),
                                baseline: remaining_bbox_h,
                                field_marker: FieldMarkerType::None,
                            }),
                            BoundingBox::new(line_run_x, current_y, run_width, remaining_bbox_h),
                        );
                        col_node.children.push(run_node);
                    }
                }
            }

            // 텍스트 세그먼트 뒤의 표 배치
            // 표 하단 = 베이스라인 + outer_margin_bottom
            if table_idx < inline_tables.len() {
                let (ctrl_idx, tbl) = &inline_tables[table_idx];
                let mt = measured_tables
                    .iter()
                    .find(|mt| mt.para_index == para_index && mt.control_index == *ctrl_idx);
                let tw = table_widths[table_idx];
                let tbl_h = mt
                    .map(|m| m.total_height)
                    .unwrap_or_else(|| hwpunit_to_px(tbl.common.height as i32, self.dpi));
                let om_bottom = hwpunit_to_px(tbl.outer_margin_bottom as i32, self.dpi);
                let tbl_y = (current_y + baseline_dist + om_bottom - tbl_h).max(current_y);

                let table_bottom = self.layout_table(
                    tree,
                    col_node,
                    tbl,
                    section_index,
                    styles,
                    0,
                    col_area,
                    tbl_y,
                    bin_data_content,
                    mt,
                    0,
                    Some((para_index, *ctrl_idx)),
                    Alignment::Left,
                    None,
                    0.0,
                    0.0,
                    Some(inline_x),
                    None,
                    table_para_y,
                    false,
                    false,
                );
                if table_bottom > max_table_bottom {
                    max_table_bottom = table_bottom;
                }

                inline_x += tw;
                table_idx += 1;
            }
        }

        // 후행 표 (텍스트 세그먼트보다 표가 더 많은 경우)
        while table_idx < inline_tables.len() {
            let (ctrl_idx, tbl) = &inline_tables[table_idx];
            let mt = measured_tables
                .iter()
                .find(|mt| mt.para_index == para_index && mt.control_index == *ctrl_idx);
            let tw = table_widths[table_idx];
            let tbl_h = mt
                .map(|m| m.total_height)
                .unwrap_or_else(|| hwpunit_to_px(tbl.common.height as i32, self.dpi));
            let om_bottom = hwpunit_to_px(tbl.outer_margin_bottom as i32, self.dpi);
            let tbl_y = (current_y + baseline_dist + om_bottom - tbl_h).max(current_y);

            let table_bottom = self.layout_table(
                tree,
                col_node,
                tbl,
                section_index,
                styles,
                0,
                col_area,
                tbl_y,
                bin_data_content,
                mt,
                0,
                Some((para_index, *ctrl_idx)),
                Alignment::Left,
                None,
                0.0,
                0.0,
                Some(inline_x),
                None,
                table_para_y,
                false,
                false,
            );
            if table_bottom > max_table_bottom {
                max_table_bottom = table_bottom;
            }

            inline_x += tw;
            table_idx += 1;
        }

        // 텍스트가 줄바꿈된 경우 텍스트 하단 고려
        // 줄바꿈된 텍스트는 텍스트 줄 높이 기준, 아니면 표 줄 높이 기준
        let text_bottom = if wrapped_below_table {
            current_y + text_line_height + line_spacing
        } else {
            current_y + line_height + line_spacing
        };
        // 표와 텍스트 중 더 큰 하단을 사용
        let effective_line_bottom = max_table_bottom
            .max(text_bottom)
            .max(y + line_height + line_spacing);
        effective_line_bottom + spacing_after
    }

    /// 문단 전체를 레이아웃하여 단 노드에 추가
    pub(crate) fn layout_paragraph(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        y_start: f64,
        section_index: usize,
        para_index: usize,
        multi_col_width_hu: Option<i32>,
        bin_data_content: Option<&[BinDataContent]>,
        wrap_anchor: Option<&crate::renderer::pagination::WrapAnchorRef>,
    ) -> f64 {
        let end_line = composed
            .map(|c| c.lines.len())
            .unwrap_or(para.line_segs.len());
        self.layout_partial_paragraph(
            tree,
            col_node,
            para,
            composed,
            styles,
            col_area,
            y_start,
            0,
            end_line,
            section_index,
            para_index,
            multi_col_width_hu,
            bin_data_content,
            wrap_anchor,
        )
    }

    /// 문단 일부를 레이아웃하여 단 노드에 추가
    pub(crate) fn layout_partial_paragraph(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        y_start: f64,
        start_line: usize,
        end_line: usize,
        section_index: usize,
        para_index: usize,
        multi_col_width_hu: Option<i32>,
        bin_data_content: Option<&[BinDataContent]>,
        wrap_anchor: Option<&crate::renderer::pagination::WrapAnchorRef>,
    ) -> f64 {
        if let Some(comp) = composed {
            // [Task #1042 Stage 6b] 본문 paragraph 의 line_segs.empty case 의 wrap 정합 —
            // compose_lines fallback (CHARS_PER_LINE=45 heuristic) 결과를 column inner width
            // 기반으로 re-split. cell paragraph (Stage 6a 의 height_measurer 호출) 와 동일
            // recompose path 사용.
            let recomposed: Option<ComposedParagraph> = if para.line_segs.is_empty() {
                let para_style = styles.para_styles.get(comp.para_style_id as usize);
                let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let column_inner_width = (col_area.width - margin_l - margin_r).max(0.0);
                if column_inner_width > 0.0 {
                    let mut cloned = comp.clone();
                    crate::renderer::composer::recompose_for_cell_width(
                        &mut cloned,
                        para,
                        column_inner_width,
                        styles,
                    );
                    Some(cloned)
                } else {
                    None
                }
            } else {
                None
            };
            let comp_ref = recomposed.as_ref().unwrap_or(comp);
            let end_line_adjusted = end_line.min(comp_ref.lines.len()).max(start_line);
            return self.layout_composed_paragraph(
                tree,
                col_node,
                comp_ref,
                styles,
                col_area,
                y_start,
                start_line,
                end_line_adjusted,
                section_index,
                para_index,
                None,
                false,
                false,
                0.0,
                multi_col_width_hu,
                Some(para),
                bin_data_content,
                wrap_anchor,
            );
        }

        // ComposedParagraph 없는 경우 기존 방식 fallback
        self.layout_raw_paragraph(
            tree, col_node, para, col_area, y_start, start_line, end_line,
        )
    }

    /// ComposedParagraph를 사용한 레이아웃
    /// `is_last_cell_para`: 셀 내 마지막 문단이면 true (마지막 줄의 trailing line_spacing 제외)
    /// `suppress_column_top_vpos_fallback`: caller가 첫 줄 vpos를 이미 y에 반영한
    /// 경우 true. 글상자 내부 문단처럼 LINE_SEG.vertical_pos 기반으로 선배치한 뒤
    /// 다시 column-top fallback을 적용하면 y가 이중 보정된다.
    /// `multi_col_width_hu`: 다단 문서에서 현재 단 너비(HWPUNIT). Some이면 segment_width 불일치 줄 건너뜀.
    /// `para`: 원본 문단 (treat_as_char 이미지 인라인 렌더링에 사용)
    /// `bin_data_content`: 이미지 데이터 (treat_as_char 이미지 인라인 렌더링에 사용)
    pub(crate) fn layout_composed_paragraph(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        composed: &ComposedParagraph,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        y_start: f64,
        start_line: usize,
        end_line: usize,
        section_index: usize,
        para_index: usize,
        cell_ctx: Option<CellContext>,
        suppress_column_top_vpos_fallback: bool,
        is_last_cell_para: bool,
        first_line_x_offset: f64,
        multi_col_width_hu: Option<i32>,
        para: Option<&Paragraph>,
        bin_data_content: Option<&[BinDataContent]>,
        wrap_anchor: Option<&crate::renderer::pagination::WrapAnchorRef>,
    ) -> f64 {
        let mut y = y_start;
        let end = end_line.min(composed.lines.len());

        // 문단 스타일에서 여백 및 정렬 정보
        let para_style = styles.para_styles.get(composed.para_style_id as usize);
        let box_margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let box_margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let indent = para_style.map(|s| s.indent).unwrap_or(0.0);

        // [Task #547] paragraph margin_left/right 는 텍스트 좌/우 inset 으로 한 번만
        // 적용. Task #544 후 box outline = col_area (margin 미적용) 이므로 박스 안
        // 좌측 여백 = box_margin_left (PDF 한컴 2010 정합).
        // 이전 코드는 paragraph border + border_spacing=0 인 경우 inner_pad_left =
        // box_margin_left 로 한 번 더 더해 이중 inset 부작용 발생 (Task #544 전 박스도
        // margin 적용했을 때만 의미가 있던 분기).
        let margin_left = box_margin_left;
        let margin_right = box_margin_right;
        let alignment = para_style
            .map(|s| s.alignment)
            .unwrap_or(Alignment::Justify);
        let spacing_before = crate::renderer::hwp3_variant_flow_spacing_before(
            para_style.map(|s| s.spacing_before).unwrap_or(0.0),
            self.use_hwp3_origin_flow_spacing_before.get(),
        );
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);
        // [Task #874 Case 3] `<...>` 단독 paragraph 의 paragraph-level extra spacing 제거.
        // typeset.rs::format_paragraph 측 동일 제거 — solo_zone_pad (zone 전환 패딩) 만 유지.
        let tab_width = para_style.map(|s| s.default_tab_width).unwrap_or(0.0);
        let tab_stops = para_style.map(|s| s.tab_stops.clone()).unwrap_or_default();
        let auto_tab_right = para_style.map(|s| s.auto_tab_right).unwrap_or(false);

        // [Task #489] 비-TAC Picture/Shape with wrap=Square 보유 여부.
        // 한컴은 어울림 그림이 있는 paragraph 의 LINE_SEG.cs/sw 를 그림 너비만큼 좁혀
        // 인코딩한다. 표 Square wrap (#362/#439/#463) 은 caller 가 col_area 를 좁혀
        // wrap_area 로 우회하지만, Picture/Shape 는 호스트 paragraph 와 같은 paragraph
        // 에 anchor 되므로 별도 우회 경로가 없다. 이 플래그가 true 면 줄별 루프에서
        // LINE_SEG.cs/sw 를 effective col_x/col_width 로 사용한다.
        let has_picture_shape_square_wrap = para
            .map(|p| {
                p.controls.iter().any(|c| {
                    let common_opt = match c {
                        Control::Picture(pic) if !pic.common.treat_as_char => Some(&pic.common),
                        Control::Shape(s) if !s.common().treat_as_char => Some(s.common()),
                        _ => None,
                    };
                    common_opt
                        .map(|cm| matches!(cm.text_wrap, TextWrap::Square))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        // [Task #1209 Stage5] 비-TAC `자리차지(TopAndBottom)` 개체가 같은 문단에
        // 있으면 한컴은 LINE_SEG.vertical_pos 로 각 줄의 실제 흐름 위치를 저장한다.
        // 첫 줄 vpos 만 한 번 더하는 fallback 으로는 “텍스트-그림-텍스트”처럼
        // 한 문단 안에서 그림 위/아래로 흐름이 갈라지는 케이스를 처리할 수 없다.
        let has_para_topbottom_float =
            has_para_topbottom_float_affecting_column(para, col_area, self.dpi);
        let col_area_w_hu = px_to_hwpunit(col_area.width, self.dpi);

        // treat_as_char 컨트롤의 px 폭 목록 (절대 char 위치, px 폭, control_index) — 정렬 보장
        let tac_offsets_px: Vec<(usize, f64, usize)> = {
            let mut v: Vec<(usize, f64, usize)> = composed
                .tac_controls
                .iter()
                .map(|(pos, w_hu, ci)| (*pos, hwpunit_to_px(*w_hu, self.dpi), *ci))
                .collect();
            v.sort_by_key(|(p, _, _)| *p);
            v
        };
        // 문단 배경색: border_fill_id 조회
        let para_border_fill_id = para_style.map(|s| s.border_fill_id).unwrap_or(0);
        let para_fill_color = if para_border_fill_id > 0 {
            let idx = (para_border_fill_id as usize).saturating_sub(1);
            styles.border_styles.get(idx).and_then(|bs| bs.fill_color)
        } else {
            None
        };

        // 문단 앞 간격 (첫 줄일 때만)
        // 단/페이지의 맨 처음 문단(column-top)은 spacing_before 를 통째 적용하면 한컴보다
        // 아래로 밀리므로 종전엔 0 으로 버렸다. 다만 섹션의 첫 문단(para_index==0, 예: 제목)은
        // 한컴 PDF 가 LINE_SEG.vertical_pos(실제 렌더한 첫 줄 흐름 위치)만큼 앞 간격을 두므로
        // (제목: spacing_before=52.9px 이지만 vertical_pos=26.5px), 그 경우 spacing_before 를
        // LINE_SEG.vertical_pos 로 상한 클램프해 적용한다. 페이지 break 후 이어진 column-top
        // (para_index>0)은 종전대로 0. (Task #853)
        let is_column_top = (y - col_area.y).abs() < 1.0;
        // [Task #1728 v2] RowBreak 셀-내 continuation 조각의 첫 가시 문단은 셀-상단이지만
        // (is_column_top) 셀-상대 인덱스>0 이라 아래 para_index==0 클램프 분기에도 못 든다.
        // 한컴은 이 첫 문단의 앞 간격(spacing_before)을 유지하므로, 토글이 켜진 이 문단만
        // column-top 이 아닌 것처럼 spacing_before 를 전량 적용한다.
        let keep_continuation_spacing_before =
            self.keep_continuation_column_top_spacing_before.get();
        if start_line == 0 && spacing_before > 0.0 {
            if !is_column_top || keep_continuation_spacing_before {
                y += spacing_before;
            } else if para_index == 0 && !suppress_column_top_vpos_fallback {
                let vpos0_px = para
                    .and_then(|p| p.line_segs.first())
                    .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
                    .unwrap_or(0.0);
                y += spacing_before.min(vpos0_px.max(0.0));
            } else if !suppress_column_top_vpos_fallback {
                // [Task #1811] 쪽 상단(para_index>0) 문단도 저장 첫 줄 vpos 가 증거다 —
                // 한컴이 앞 간격을 유지한 문서는 쪽-상대 vpos ≈ spacing_before 로 저장되고
                // (task1750 샘플 p2: sb=700HU, vpos=700 — 트림 시 페이지 전체가 5pt 위로
                // 밀려 visual sweep 이중상), 트림한 문서는 vpos=0 이다. #853 의
                // para_index==0 클램프를 저장 증거 기반으로 일반화하되, 누적축 vpos
                // 인코딩(vpos ≫ sb)은 쪽-상대 증거가 아니므로 종전(트림) 유지.
                let vpos0_px = para
                    .and_then(|p| p.line_segs.first())
                    .filter(|ls| ls.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
                    .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
                    .unwrap_or(0.0);
                if vpos0_px > 0.0 && vpos0_px <= spacing_before + 0.5 {
                    y += vpos0_px;
                }
            }
        }
        // [Task #1012] paragraph 첫 line vpos > 0 인데 spacing_before=0 으로
        // 위 블록 진입 안한 경우 (test-image.hwp page 1: TopAndBottom Picture)
        // line_seg.vpos 를 직접 y 에 가산하여 텍스트가 wrap shape 아래로
        // 위치하도록 함. wrap 메커니즘이 별도로 처리하지 못하는 case 의
        // fallback. start_line==0 + column-top + para_index==0 으로 한정.
        if start_line == 0
            && spacing_before == 0.0
            && is_column_top
            && para_index == 0
            && !has_para_topbottom_float
            && !suppress_column_top_vpos_fallback
        {
            let vpos0_px = para
                .and_then(|p| p.line_segs.first())
                .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
                .unwrap_or(0.0);
            if vpos0_px > 0.0 {
                y += vpos0_px;
            }
        }

        // 문단 전체에서 모든 라인의 runs가 비어있는지 확인
        // (텍스트 없이 TAC 이미지만 있는 문단)
        //
        // [Issue #1945] `start_line` 은 PartialTable/Partial 이월 경로에서 인자로
        // 전달되며 `end`(= end_line.min(lines.len())) 와 독립 계산이라, 이월 루프가
        // 조판 라인 수를 넘겨 `start_line > end`(또는 > lines.len())가 되면 직접
        // 슬라이스가 패닉했다(실문서 크래시). 아래 렌더 루프(`for line_idx in
        // start_line..end`)는 빈 범위를 안전히 처리하므로, 여기서도 `get()` 으로
        // 방어해 범위 밖이면 "가시 run 없음"(vacuously true)으로 본다.
        let all_runs_empty = composed
            .lines
            .get(start_line..end)
            .map_or(true, |slice| slice.iter().all(|l| l.runs.is_empty()));

        // 개요 번호/글머리표 마커 폭 사전 계산 (첫 줄 가용폭 차감용)
        let numbering_width = if start_line == 0 {
            if let Some(ref num_text) = composed.numbering_text {
                let num_style = numbering_marker_text_style(
                    styles,
                    para,
                    composed.lines.first().and_then(|l| l.runs.first()),
                );
                estimate_text_width(num_text, &num_style)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // 배경/테두리 렌더링을 위한 시작 위치 기록
        // 문단 경계 = 이전 문단 끝 = y_start (spacing_before 적용 전)
        let bg_y_start = if para_border_fill_id > 0 { y_start } else { y };
        let bg_insert_idx = col_node.children.len();

        // start_line까지의 누적 문자 오프셋 계산 (편집용 문서 좌표)
        let mut char_offset: usize = 0;
        for li in 0..start_line.min(composed.lines.len()) {
            for run in &composed.lines[li].runs {
                char_offset += run.text.chars().count();
            }
            // 강제 줄바꿈(\n)은 run 텍스트에서 제거되었으므로 별도 가산
            if composed.lines[li].has_line_break {
                char_offset += 1;
            }
        }

        // [Issue #926] Endnote 인라인 마커 — 첫 줄 앞에 일반 텍스트로 emit
        // 한컴에서 미주 마커는 위첨자가 아닌 본문 크기 텍스트로 표시
        let mut endnote_marker_x_advance = 0.0f64;
        if start_line == 0 {
            if let Some(p) = para {
                let ctrl_positions = p.control_text_positions();
                let first_line_char_start = composed
                    .lines
                    .first()
                    .map(|line| line.char_start)
                    .unwrap_or(0);
                for (ctrl_idx, ctrl) in p.controls.iter().enumerate() {
                    if let Control::Endnote(en) = ctrl {
                        let Some(marker_pos) = ctrl_positions.get(ctrl_idx).copied() else {
                            continue;
                        };
                        if !is_leading_endnote_marker_rendered_as_prefix(
                            para,
                            ctrl_idx,
                            0,
                            start_line,
                            marker_pos,
                            first_line_char_start,
                        ) {
                            continue;
                        }
                        let marker_text =
                            format!("{} ", note_marker_text_from_control(Some(ctrl), en.number));
                        let first_cs_id = p
                            .char_shapes
                            .first()
                            .map(|cs| cs.char_shape_id as usize)
                            .unwrap_or(0);
                        let ts = resolved_to_text_style(styles, first_cs_id as u32, 0);
                        let marker_w = estimate_text_width(&marker_text, &ts);
                        let marker_y = y
                            + spacing_before
                            + hwpunit_to_px(
                                composed
                                    .lines
                                    .first()
                                    .map(|l| l.baseline_distance)
                                    .unwrap_or(0),
                                self.dpi,
                            );
                        let marker_x = col_area.x + margin_left + indent;
                        let marker_id = tree.next_id();
                        let marker_node = RenderNode::new(
                            marker_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: marker_text,
                                style: ts,
                                char_shape_id: Some(first_cs_id as u32),
                                para_shape_id: Some(composed.para_style_id),
                                section_index: Some(section_index),
                                para_index: Some(para_index),
                                char_start: Some(0),
                                cell_context: None,
                                is_para_end: false,
                                is_line_break_end: false,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: 0,
                                baseline: hwpunit_to_px(
                                    composed
                                        .lines
                                        .first()
                                        .map(|l| l.baseline_distance)
                                        .unwrap_or(0),
                                    self.dpi,
                                ),
                                field_marker: FieldMarkerType::None,
                            }),
                            BoundingBox::new(
                                marker_x,
                                y + spacing_before,
                                marker_w,
                                hwpunit_to_px(
                                    composed.lines.first().map(|l| l.line_height).unwrap_or(0),
                                    self.dpi,
                                ),
                            ),
                        );
                        col_node.children.push(marker_node);
                        endnote_marker_x_advance += marker_w;
                    }
                }
            }
        }

        let endnote_line_vpos_base: Option<(i32, f64)> = {
            let base = self.endnote_para_base.get();
            if cell_ctx.is_none() && para_index >= base && end > start_line + 1 {
                para.and_then(|p| {
                    let base_line_idx = if line_is_leading_empty_equation_tac_guide(
                        Some(p),
                        composed,
                        &tac_offsets_px,
                        start_line,
                    ) {
                        start_line + 1
                    } else {
                        start_line
                    };
                    let range = p.line_segs.get(base_line_idx..end)?;
                    if range
                        .windows(2)
                        .all(|w| w[1].vertical_pos >= w[0].vertical_pos)
                    {
                        range.first().map(|seg| (seg.vertical_pos, y))
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        };
        let para_topbottom_line_vpos_base: Option<(i32, f64)> = {
            if cell_ctx.is_none() && has_para_topbottom_float {
                para.and_then(|p| {
                    let range = p.line_segs.get(start_line..end)?;
                    if range.iter().any(|seg| seg.vertical_pos > 0)
                        && range
                            .windows(2)
                            .all(|w| w[1].vertical_pos >= w[0].vertical_pos)
                    {
                        let base_vpos = if start_line == 0 {
                            0
                        } else {
                            range.first().map(|seg| seg.vertical_pos).unwrap_or(0)
                        };
                        Some((base_vpos, y))
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        };
        let mut endnote_line_vpos_y_end: Option<f64> = None;
        let mut endnote_auto_wrap_y_end: Option<f64> = None;
        let mut prev_line_reserved_tac_picture_height: Option<f64> = None;
        for line_idx in start_line..end {
            let comp_line = &composed.lines[line_idx];
            let mut current_line_reserved_tac_picture_height: Option<f64> = None;
            let mut endnote_used_auto_wrap_y = false;
            if let (Some((base_vpos, base_y)), Some(seg)) = (
                endnote_line_vpos_base,
                para.and_then(|p| p.line_segs.get(line_idx)),
            ) {
                let vpos_y = base_y + hwpunit_to_px(seg.vertical_pos - base_vpos, self.dpi);
                if let Some(prev) = endnote_auto_wrap_y_end {
                    if prev > vpos_y + 0.5 {
                        y = prev;
                        endnote_used_auto_wrap_y = true;
                    } else {
                        y = vpos_y;
                        endnote_auto_wrap_y_end = None;
                    }
                } else {
                    y = vpos_y;
                }
            } else if let (Some((base_vpos, base_y)), Some(seg)) = (
                para_topbottom_line_vpos_base,
                para.and_then(|p| p.line_segs.get(line_idx)),
            ) {
                y = base_y + hwpunit_to_px(seg.vertical_pos - base_vpos, self.dpi);
            }

            // 다단 필터링: segment_width가 현재 단 너비와 불일치하면 건너뜀
            if let Some(col_w) = multi_col_width_hu {
                if comp_line.segment_width > 0 && (comp_line.segment_width - col_w).abs() > 200 {
                    // char_offset만 진행하고 렌더링 건너뜀
                    for run in &comp_line.runs {
                        char_offset += run.text.chars().count();
                    }
                    if comp_line.has_line_break {
                        char_offset += 1;
                    }
                    continue;
                }
            }

            // 최대 폰트 크기 계산 (line_height 최솟값 보정에도 사용)
            let max_fs = comp_line
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
            let mut line_tac_offsets = tac_offsets_for_line(composed, &tac_offsets_px, line_idx);
            if let Some(offsets) =
                repeated_empty_tac_line_offset(composed, &tac_offsets_px, line_idx)
            {
                line_tac_offsets = offsets;
            }
            let runs_all_whitespace = comp_line.runs.iter().all(|r| r.text.trim().is_empty());
            let mut line_tac_offsets_for_width = line_tac_offsets.clone();
            if cell_ctx.is_some()
                && alignment == Alignment::Right
                && runs_all_whitespace
                && composed.lines.get(line_idx + 1).is_none()
            {
                let has_strict_inline_tac_table = para
                    .map(|p| {
                        line_tac_offsets.iter().any(|(_, _, ci)| {
                            matches!(p.controls.get(*ci), Some(Control::Table(t)) if t.common.treat_as_char)
                        })
                    })
                    .unwrap_or(false);
                if has_strict_inline_tac_table {
                    let line_end = composed_line_char_end(composed, line_idx);
                    if line_end > comp_line.char_start {
                        for (pos, tac_w, ci) in tac_offsets_px.iter().copied() {
                            if pos == line_end
                                && !line_tac_offsets_for_width
                                    .iter()
                                    .any(|(_, _, existing_ci)| *existing_ci == ci)
                                && para
                                    .and_then(|p| p.controls.get(ci))
                                    .is_some_and(|ctrl| {
                                        matches!(ctrl, Control::Table(t) if t.common.treat_as_char)
                                    })
                            {
                                line_tac_offsets_for_width.push((pos, tac_w, ci));
                            }
                        }
                    }
                }
            }
            let empty_tac_guide_line = comp_line.runs.is_empty() && !line_tac_offsets.is_empty();
            // LineSeg.line_height는 HWP에서 줄간격이 이미 반영된 값.
            // PARA_LINE_SEG가 없는 폴백(400 HWPUNIT=5.333px) 등 line_height가 폰트 크기보다 작으면,
            // ParaShape의 줄간격 설정(line_spacing_type + line_spacing)으로 올바른 줄 높이를 계산한다.
            let raw_lh = hwpunit_to_px(comp_line.line_height, self.dpi);
            let text_before_picture_line = text_line_is_picture_lead_in(
                para,
                composed,
                &tac_offsets_px,
                line_idx,
                raw_lh,
                max_fs,
                self.dpi,
            );
            let (line_height, line_spacing_px) = {
                let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                let ls_type = para_style
                    .map(|s| s.line_spacing_type)
                    .unwrap_or(LineSpacingType::Percent);
                let raw_text_height = para
                    .and_then(|p| p.line_segs.get(line_idx))
                    .map(|seg| hwpunit_to_px(seg.text_height, self.dpi))
                    .unwrap_or(0.0);
                let is_plain_text_para = para.map(|p| p.controls.is_empty()).unwrap_or(false);
                let use_stored_text_height =
                    is_plain_text_para && (self.is_hwpx_source.get() || cell_ctx.is_none());
                crate::renderer::corrected_line_metrics_for_source(
                    raw_lh,
                    raw_text_height,
                    hwpunit_to_px(comp_line.line_spacing, self.dpi),
                    max_fs,
                    ls_type,
                    ls_val,
                    use_stored_text_height,
                )
            };
            // 인라인 Shape(글상자)가 있는 줄: line_height에 Shape 높이가 포함됨
            // Shape는 별도 패스에서 para_y 기준으로 렌더링되므로,
            // 텍스트의 y와 line_height를 폰트 기반으로 보정하여 baseline 정렬
            let has_tac_shape = !tac_offsets_px.is_empty()
                && para
                    .map(|p| {
                        tac_offsets_px.iter().any(|(_, _, ci)| {
                            p.controls
                                .get(*ci)
                                .map(|c| matches!(c, Control::Shape(_)))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);
            let empty_tac_guide_has_explicit_shape_height = empty_tac_guide_line
                && para
                    .map(|p| {
                        line_tac_offsets.iter().any(|(_, _, ci)| {
                            p.controls.get(*ci).is_some_and(|ctrl| match ctrl {
                                Control::Shape(shape) if shape.common().treat_as_char => {
                                    shape.shape_attr().current_height > shape.common().height
                                }
                                _ => false,
                            })
                        })
                    })
                    .unwrap_or(false);
            let (line_height, baseline) = if text_before_picture_line {
                let font_lh = max_fs.max(1.0);
                let font_bl = max_fs * 0.85;
                (font_lh, ensure_min_baseline(font_bl, max_fs))
            } else if has_tac_shape
                && !empty_tac_guide_has_explicit_shape_height
                && (cell_ctx.is_none() || max_fs > 0.0)
                && raw_lh > max_fs * 1.5
            {
                // Shape와 텍스트가 같은 줄에 있으면 Shape 높이가 line_height에 포함된다.
                // [#1842] 셀 내부에서 max_fs=0(텍스트 없는 tac-전용 줄)이면 이 보정의
                // 전제("Shape 와 텍스트의 baseline 정렬")가 성립하지 않는다 — 종전에는
                // raw_lh > 0*1.5 가 항상 참이라 font_lh=0 으로 퇴화해, 셀 내부
                // tac 묶음 전용 문단의 저장 lh(예: 3401HU)가 소실되고 후속 블록이
                // 통째로 당겨졌다 (3114781 p2 −33pt, 한글 2022 오라클 정합 확인).
                // 본문(cell_ctx 없음)은 reserved/skip-advance 보상 기계가 이 축소값을
                // 전제로 한컴 정합을 이미 이루고 있어(sample16 issue_1116 한컴 핀)
                // 종전 동작을 유지한다.
                let font_lh = max_fs * 1.2; // 폰트 크기의 120%
                let font_bl = max_fs * 0.85;
                (font_lh, ensure_min_baseline(font_bl, max_fs))
            } else {
                (
                    line_height,
                    ensure_min_baseline(
                        hwpunit_to_px(comp_line.baseline_distance, self.dpi),
                        max_fs,
                    ),
                )
            };
            // 들여쓰기/내어쓰기: 문단 여백은 무조건 적용
            // - 보통(ind=0): 모든 줄 margin_left
            // - 들여쓰기(ind>0): 첫줄 margin_left+indent, 다음줄 margin_left
            // - 내어쓰기(ind<0): 첫줄 margin_left, 다음줄 margin_left+|indent|
            let line_indent =
                crate::renderer::equation_tac_flow::paragraph_line_indent(indent, line_idx);
            let effective_margin_left = margin_left + line_indent;

            // [Task #489] Picture/Shape Square wrap (어울림) 시 LINE_SEG.cs/sw 적용.
            // 한컴이 인코딩한 정답값을 그대로 사용 (휴리스틱 없음).
            // 표 Square wrap 케이스는 caller 가 col_area 를 이미 wrap_area 로 좁혀
            // 호출하므로 segment_width ≈ col_area_w_hu → 조건 미발동 (회귀 차단).
            // 200 HU 임계값은 paragraph_layout 의 multi-col filter 와 동일 (페이지네이션 노이즈 제거).
            //
            // [Task #568] 인라인 TAC 표(treat_as_char=true) 가 있는 줄도 동일 처리.
            // HWP 는 인라인 TAC 표가 있는 줄의 segment_width 를 표 폭 + 잔여로 좁게
            // 인코딩한다 (wrap=TopAndBottom 영향). col_area.width 로 잡으면
            // Justify slack 이 과대 산출되어 선두 공백이 80 px/space 로 부풀어 표를
            // 우측으로 민다 (exam_science.hwp pi=61 12번 응답: +175 px 편위).
            let line_has_inline_tac_table = !tac_offsets_px.is_empty()
                && para
                    .map(|p| {
                        line_tac_offsets.iter().any(|(_, _, ci)| {
                            matches!(p.controls.get(*ci),
                            Some(Control::Table(t)) if t.common.treat_as_char)
                        })
                    })
                    .unwrap_or(false);

            // [Task #568] 임계값에 column_start 포함 — 실제 가용 line 폭은 (sw + cs).
            // 단락 들여쓰기를 LINE_SEG.column_start 로 인코딩한 paragraph 의
            // 정상 라인은 (sw + cs) ≈ col_w_hu 이므로 새 분기 미진입.
            // Picture/Shape Square wrap 은 cs=0 이라 기존 동작과 동일.
            let line_avail_hu = comp_line
                .segment_width
                .saturating_add(comp_line.column_start);
            // [Task #901] cs > 0 + sw < col_w 인 경우도 effective_col_x 적용.
            // pic2.hwp paragraph 0 의 ls[1] (cs=39123 sw=3397, avail=col_w) 같은
            // wrap zone 우측 영역 case 의 X 위치 정합 — paragraph 0 의 한글 char
            // ("우/리/나/라") 가 그림 사이/우측 좁은 영역에 그려져야 함.
            // 기존 조건 `avail < col_w - 200` 만으로는 avail == col_w 인 wrap zone
            // 라인이 분기 미진입 → col_area.x 좌측에 잘못 그려짐.
            let cs_significant = comp_line.column_start > 0
                && comp_line.segment_width > 0
                && comp_line.segment_width < col_area_w_hu;
            // [Task #1440] anchor 매칭이 없는 후속 body 문단이라도 LINE_SEG 자체가
            // 단 폭보다 확연히 좁은 wrap zone 을 보존하면 그 저장 폭을 따른다.
            // 정상 들여쓰기 계열은 cs+sw ~= col_w 이므로 제외한다.
            //
            // LineSeg cs/sw 만으로 wrap zone 을 판정하면 paragraph border 박스의 내부
            // inset도 그림 어울림으로 오인된다(#547 passage box, #1440 6쪽 지문 박스).
            // anchor 메타데이터가 없는 fallback 보정은 같은 문단 안에서 실제로 좁은 줄과
            // 넓은 줄이 섞인 precomputed picture-wrap 흐름에만 제한한다.
            let para_has_mixed_segment_widths = para
                .map(|p| {
                    let mut min_sw = i32::MAX;
                    let mut max_sw = 0;
                    for seg in p.line_segs.iter().filter(|seg| seg.segment_width > 0) {
                        min_sw = min_sw.min(seg.segment_width);
                        max_sw = max_sw.max(seg.segment_width);
                    }
                    min_sw != i32::MAX && max_sw.saturating_sub(min_sw) > 1000
                })
                .unwrap_or(false);
            let precomputed_body_wrap_line = cell_ctx.is_none()
                && para_has_mixed_segment_widths
                && comp_line.segment_width > 0
                && line_avail_hu < col_area_w_hu - 200
                && para
                    .and_then(|p| p.line_segs.get(line_idx))
                    .map(|seg| seg.is_in_wrap_zone(col_area_w_hu))
                    .unwrap_or(false);
            let (effective_col_x, effective_col_w) = if (has_picture_shape_square_wrap
                || line_has_inline_tac_table
                || precomputed_body_wrap_line)
                && comp_line.segment_width > 0
                && (line_avail_hu < col_area_w_hu - 200 || cs_significant)
            {
                let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
                let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
                (col_area.x + cs_px, sw_px)
            } else {
                (col_area.x, col_area.width)
            };

            // 인라인 Shape가 있는 줄: 텍스트 y를 Shape 하단 baseline에 맞춤
            let text_y = if has_tac_shape
                && !empty_tac_guide_has_explicit_shape_height
                && raw_lh > max_fs * 1.5
            {
                // raw_lh는 Shape 높이 포함 원본 줄 높이, line_height는 폰트 기반 보정 높이
                // 텍스트를 Shape 하단 근처로 이동 (Shape 높이 - 폰트 줄 높이)
                y + (raw_lh - line_height).max(0.0)
            } else {
                y
            };
            // Task #332 Stage 4b: clamp 제거. 단 하단을 초과하는 줄은 그대로 그린다
            // (시각 경계 약간 넘김 허용). 기존의 `text_y = col_bottom - line_height`
            // 클램프는 여러 overflow 줄을 같은 y 에 piling 해 글자 겹침을 만들었으나,
            // 클램프 없이 원래 y 에 그리면 piling 자체가 발생하지 않는다. 콘텐츠 손실
            // (stop drawing) 도 발생하지 않으며, drift 의 본질적 해결은 Stage 5 에서.
            let col_bottom = col_area.y + col_area.height;
            let line_visual_bottom = text_y + line_height;
            let is_body_flow_col_area = self.is_body_flow_col_area(col_area);
            let is_endnote_virtual_para = para_index >= self.endnote_para_base.get();
            let blank_spacer_line = is_blank_spacer_line(
                para,
                is_endnote_virtual_para,
                runs_all_whitespace,
                &line_tac_offsets,
            );
            let equation_only_endnote_tail_line = is_body_flow_col_area
                && cell_ctx.is_none()
                && is_endnote_virtual_para
                && line_idx + 1 >= end
                && is_equation_only_tac_line(para, runs_all_whitespace, &line_tac_offsets);
            let tolerated_endnote_bottom_bleed = self.is_tolerated_current_endnote_bottom_bleed(
                is_body_flow_col_area && cell_ctx.is_none() && is_endnote_virtual_para,
                line_visual_bottom,
                col_bottom,
                equation_only_endnote_tail_line,
            );
            if is_body_flow_col_area
                && cell_ctx.is_none()
                && line_visual_bottom > col_bottom + 0.5
                && !blank_spacer_line
                && !tolerated_endnote_bottom_bleed
            {
                eprintln!(
                    "LAYOUT_OVERFLOW_DRAW: section={} pi={} line={} y={:.1} col_bottom={:.1} overflow={:.1}px",
                    section_index, para_index, line_idx,
                    line_visual_bottom, col_bottom, line_visual_bottom - col_bottom,
                );
            }
            // [Task #604 R3] wrap_anchor 가 있으면 본 문단은 anchor 그림/표 옆 wrap text.
            // 각 라인의 LineSeg cs(column_start)/sw(segment_width)를 x 오프셋/너비로 적용.
            // typeset 의 wrap_around state machine 매칭 결과 (ColumnContent.wrap_anchors)
            // 가 layout 에 전달되어 본 분기가 동작.
            //
            // [Task #722] inter-image-text gap 보정 — 한컴 viewer 는 anchor image 의
            // outer margin_right (HU) 만큼 cs 에 더해 text 시작 x 결정. sw 에서 동일량
            // 차감하여 가용 폭 정합. WrapAnchorRef.anchor_image_margin_right 활용.
            let (line_cs_offset, line_avail_w_override) = if let Some(anchor) = wrap_anchor {
                let seg = para.and_then(|p| p.line_segs.get(line_idx));
                let cs = seg.map(|s| s.column_start as i32).unwrap_or(0);
                let sw = seg.map(|s| s.segment_width as i32).unwrap_or(0);
                let mr = anchor.anchor_image_margin_right;
                let cs_px = crate::renderer::hwpunit_to_px(cs + mr, self.dpi);
                let sw_px = if sw > 0 {
                    Some(crate::renderer::hwpunit_to_px((sw - mr).max(0), self.dpi))
                } else {
                    None
                };
                (cs_px, sw_px)
            } else {
                (0.0, None)
            };

            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine({
                    let vpos = para
                        .and_then(|p| p.line_segs.get(line_idx))
                        .map(|ls| ls.vertical_pos)
                        .unwrap_or(0);
                    TextLineNode::with_para_vpos(
                        line_height,
                        baseline,
                        section_index,
                        para_index,
                        line_idx as u32,
                        vpos,
                    )
                }),
                BoundingBox::new(
                    // [Task #604 R3] wrap_anchor 가 있으면 line_cs_offset 사용 (col_area.x 기준),
                    // 아니면 Task #489 effective_col_x 사용. 두 경로 중복 적용 방지.
                    if wrap_anchor.is_some() {
                        col_area.x + effective_margin_left + line_cs_offset
                    } else {
                        effective_col_x + effective_margin_left
                    },
                    text_y,
                    line_avail_w_override
                        .unwrap_or(effective_col_w - effective_margin_left - margin_right),
                    line_height,
                ),
            );

            let inline_offset = if line_idx == start_line {
                first_line_x_offset + endnote_marker_x_advance
            } else {
                0.0
            };
            // 번호/글머리표 마커: 모든 줄에서 마커 폭만큼 가용폭 차감 (행잉 인덴트)
            let num_offset = if numbering_width > 0.0 {
                numbering_width
            } else {
                0.0
            };
            let available_width = line_avail_w_override
                .map(|w| w - inline_offset - num_offset)
                .unwrap_or(
                    effective_col_w
                        - effective_margin_left
                        - margin_right
                        - inline_offset
                        - num_offset,
                );
            // [Task #1472] IR indent 를 full 로 되돌리면서(parser/mod.rs) 미주 TAC 수식
            // available_width 의 effective indent 를 불변 유지: 변환본은 scale 을 절반으로.
            // (종전: IR(half)×2.0=full → 현재: IR(full)×1.0=full)
            let equation_indent_scale = (if cell_ctx.is_some() { 1.0 } else { 2.0 })
                * if self.is_hwp3_variant.get() { 0.5 } else { 1.0 };
            let equation_first_effective_margin_left =
                crate::renderer::equation_tac_flow::paragraph_effective_margin_left_with_indent_scale(
                    margin_left,
                    indent,
                    0,
                    equation_indent_scale,
                );
            let equation_continuation_effective_margin_left =
                crate::renderer::equation_tac_flow::paragraph_effective_margin_left_with_indent_scale(
                    margin_left,
                    indent,
                    1,
                    equation_indent_scale,
                );
            let equation_first_available_width = line_avail_w_override
                .map(|w| w - inline_offset - num_offset)
                .unwrap_or(
                    effective_col_w
                        - equation_first_effective_margin_left
                        - margin_right
                        - inline_offset
                        - num_offset,
                );
            let equation_continuation_available_width = line_avail_w_override
                .map(|w| w - inline_offset - num_offset)
                .unwrap_or(
                    effective_col_w
                        - equation_continuation_effective_margin_left
                        - margin_right
                        - inline_offset
                        - num_offset,
                );
            let equation_tac_line_flow =
                crate::renderer::equation_tac_flow::compute_equation_only_tac_line_flow(
                    para,
                    composed,
                    &tac_offsets_px,
                    line_idx,
                    if cell_ctx.is_some() {
                        f64::INFINITY
                    } else {
                        equation_first_available_width
                    },
                    if cell_ctx.is_some() {
                        f64::INFINITY
                    } else {
                        equation_continuation_available_width
                    },
                );
            let equation_tac_extra_rows = equation_tac_line_flow
                .as_ref()
                .map(|flow| flow.extra_rows)
                .unwrap_or(0);
            let line_flow_height =
                line_height + equation_tac_extra_rows as f64 * (line_height + line_spacing_px);
            let render_line_flow_height =
                if cell_ctx.is_none() && para_index >= self.endnote_para_base.get() {
                    // 미주 lineSeg의 행 진행값이 실제 TextLine bbox보다 작으면 단일 줄 미주가
                    // 서로 겹친다. Pagination은 별도 압축 흐름을 쓰더라도 렌더 y 진행은
                    // 실제 그려진 줄 높이를 최소값으로 보존한다.
                    line_flow_height.max(max_fs).max(line_node.bbox.height)
                } else {
                    line_flow_height
                };
            let render_line_spacing_px =
                if cell_ctx.is_none() && para_index >= self.endnote_para_base.get() {
                    // 비가시 구분선/0mm 미주는 pagination과 render가 같은 압축 spacing을
                    // 써야 단 하단 클리핑이 생기지 않는다. 다만 과한 음수값은 글자 겹침을
                    // 만들 수 있으므로 실제 glyph 높이의 10% 범위로 제한한다.
                    if line_spacing_px < 0.0 {
                        line_spacing_px.max(-render_line_flow_height * 0.10)
                    } else {
                        line_spacing_px
                    }
                } else {
                    line_spacing_px
                };
            if equation_tac_extra_rows > 0 {
                line_node.bbox.height = line_flow_height;
                if let RenderNodeType::TextLine(ref mut text_line) = line_node.node_type {
                    text_line.line_height = line_flow_height;
                }
            }

            // 텍스트 정렬을 위한 전체 줄 폭 계산 (자연 폭, 추가 간격 미포함)
            // treat_as_char 이미지 폭도 포함하여 정확한 폭 산출
            // [Task #604 Stage 2] wrap_anchor 가 있는 줄: line_cs_offset 을 est_x 기준점에
            // 포함 (line_x_offset 은 col_area.x 기준 상대좌표).
            let est_x_start = effective_margin_left + line_cs_offset + inline_offset;
            let line_width_est = self.estimate_line_run_widths(
                comp_line,
                composed,
                para,
                styles,
                &tab_stops,
                tab_width,
                auto_tab_right,
                &line_tac_offsets_for_width,
                effective_margin_left,
                available_width,
                start_line,
                line_idx,
                est_x_start,
            );
            let est_x = line_width_est.est_x;
            let included_tac_width_in_est = line_width_est.included_tac_width;
            // 교차 run 탭으로 인한 역방향 이동이 있을 수 있으므로
            // est_x 차이로 정확한 점유 폭을 계산
            let mut total_text_width = (est_x - est_x_start).max(0.0);
            // TAC 이미지/Shape 폭이 est_x에 미포함된 경우 별도 추가
            // (이미지가 텍스트 끝 위치에 있으면 run 범위 필터에서 제외됨)
            //
            // [Task #1219] 줄-경계 정규 집합 line_tac_offsets 로 통일.
            // 기존 `pos <= line_end` 는 줄 끝 위치(다음 줄 선두) 수식을 포함하는
            // 동일 결함을 가졌다. line_tac_offsets 는 이미 줄-범위 집합이므로 폭만 합산.
            let total_tac_width_in_line: f64 =
                line_tac_offsets_for_width.iter().map(|(_, w, _)| w).sum();
            let missing_tac_width = (total_tac_width_in_line - included_tac_width_in_est).max(0.0);
            if missing_tac_width > 0.0 && total_text_width < total_tac_width_in_line {
                total_text_width += missing_tac_width;
            }
            let is_last_line_of_para = line_idx == end - 1 && end == composed.lines.len();

            // 정렬별 간격 분배 계산
            let has_forced_break = comp_line.has_line_break;
            // 머리말/꼬리말은 내부 문단 인덱스를 `usize::MAX - i`로 넘긴다.
            // HWP3 머리말 단일 줄 Justify도 한컴처럼 머리말 폭까지 공간을 벌려야 한다.
            let is_header_footer_para = para_index >= usize::MAX - 1024;
            let needs_justify = alignment == Alignment::Justify
                && (!is_last_line_of_para || is_header_footer_para)
                && !has_forced_break;
            let needs_distribute = alignment == Alignment::Distribute
                || (alignment == Alignment::Split && !is_last_line_of_para && !has_forced_break);

            let has_tabs = comp_line.runs.iter().any(|r| r.text.contains('\t'));
            let total_char_count: usize = comp_line
                .runs
                .iter()
                .map(|r| r.text.chars().filter(|c| *c != '\t').count())
                .sum();
            let suppress_cell_overflow_spacing =
                cell_ctx.is_some() && total_text_width > available_width * 1.15;

            // Task #352: 라인 내 dash leader (3+ 연속 '-') 글자 수 카운트.
            // visible_count 까지의 chars 에서만 카운트 (후행 공백 제외).
            let count_dash_leaders = |chars: &[char]| -> usize {
                let mut count = 0;
                let n = chars.len();
                let mut i = 0;
                while i < n {
                    if chars[i] == '-' {
                        let mut j = i;
                        while j < n && chars[j] == '-' {
                            j += 1;
                        }
                        let run_len = j - i;
                        if run_len >= 3 {
                            count += run_len;
                        }
                        i = j;
                    } else {
                        i += 1;
                    }
                }
                count
            };

            let (extra_word_sp, extra_char_sp, extra_dash_sp) = if needs_justify {
                // 양쪽 정렬: 후행 공백 제외한 내부 공백에 분배
                let all_chars: Vec<char> =
                    comp_line.runs.iter().flat_map(|r| r.text.chars()).collect();
                let trailing_spaces = all_chars.iter().rev().take_while(|c| **c == ' ').count();
                let visible_count = all_chars.len() - trailing_spaces;
                let interior_spaces = all_chars[..visible_count]
                    .iter()
                    .filter(|c| **c == ' ')
                    .count();
                let leader_dashes = count_dash_leaders(&all_chars[..visible_count]);
                if interior_spaces > 0 {
                    // 후행 공백 폭 계산
                    let trailing_width = if trailing_spaces > 0 {
                        if let Some(last_run) = comp_line.runs.last() {
                            let mut ts = resolved_to_text_style(
                                styles,
                                last_run.char_style_id,
                                last_run.lang_index,
                            );
                            ts.default_tab_width = tab_width;
                            let trailing_str: String = " ".repeat(trailing_spaces);
                            estimate_text_width(&trailing_str, &ts)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    let effective_used = total_text_width - trailing_width;
                    let slack = available_width - effective_used;
                    if leader_dashes > 0 && slack > 0.0 {
                        // Task #352: 라인에 dash leader 가 있고 슬랙이 양수면
                        // dash 가 흡수 (PDF elastic leader 동작 모방). 공백·일반
                        // 글자 자연 폭 유지.
                        (0.0, 0.0, slack / leader_dashes as f64)
                    } else if suppress_cell_overflow_spacing && slack < 0.0 {
                        // 셀 내부 폭이 글자 자연 폭보다 작아도 한컴처럼 글자를 압축하지 않는다.
                        // 줄바꿈은 LINE_SEG/리플로우가 결정하고, 그린 글자는 셀 경계에서만 클리핑한다.
                        (0.0, 0.0, 0.0)
                    } else {
                        // 양쪽 정렬: 단어 간격 분배 (또는 음수 슬랙 시 압축)
                        let raw_ews = slack / interior_spaces as f64;
                        let space_base_w = estimate_text_width(
                            " ",
                            &resolved_to_text_style(
                                styles,
                                comp_line.runs[0].char_style_id,
                                comp_line.runs[0].lang_index,
                            ),
                        );
                        let min_ews = -(space_base_w * 0.5);
                        (raw_ews.max(min_ews), 0.0, 0.0)
                    }
                } else if total_char_count > 1 {
                    // 양쪽 정렬이지만 공백 없음 (일본어/숫자 등):
                    let slack = available_width - total_text_width;
                    if leader_dashes > 0 && slack > 0.0 {
                        (0.0, 0.0, slack / leader_dashes as f64)
                    } else if suppress_cell_overflow_spacing && slack < 0.0 {
                        // 셀의 좁은 내부 폭은 줄바꿈 기준일 뿐, 숫자/문자를 수평 압축하지 않는다.
                        (0.0, 0.0, 0.0)
                    } else {
                        let raw = slack / total_char_count as f64;
                        let avg_char_w = total_text_width / total_char_count as f64;
                        let min_sp = -avg_char_w * 0.5;
                        (0.0, raw.max(min_sp), 0.0)
                    }
                } else {
                    (0.0, 0.0, 0.0)
                }
            } else if needs_distribute && total_char_count > 1 {
                // 배분/나눔 정렬: 모든 글자에 균등 분배
                let raw = (available_width - total_text_width) / total_char_count as f64;
                if suppress_cell_overflow_spacing && raw < 0.0 {
                    (0.0, 0.0, 0.0)
                } else {
                    let avg_char_w = total_text_width / total_char_count as f64;
                    let min_sp = -avg_char_w * 0.5;
                    (0.0, raw.max(min_sp), 0.0)
                }
            } else if total_text_width > available_width && total_char_count > 1 && !has_tabs {
                // 비정렬(왼쪽/오른쪽/가운데) 텍스트가 오버플로우할 때 글자 간격 압축
                if suppress_cell_overflow_spacing {
                    (0.0, 0.0, 0.0)
                } else {
                    let raw = (available_width - total_text_width) / total_char_count as f64;
                    let avg_char_w = total_text_width / total_char_count as f64;
                    let min_sp = -avg_char_w * 0.5;
                    (0.0, raw.max(min_sp), 0.0)
                }
            } else if cell_ctx.is_some()
                && total_char_count > 1
                && !has_tabs
                && alignment != Alignment::Left
                && total_text_width < available_width
                && total_text_width > 0.0
                && comp_line.runs.iter().any(|r| {
                    let ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                    ts.letter_spacing < -0.01
                })
                && {
                    // 자연 폭(letter_spacing=0)이 셀 inner 폭보다 커야만 "문서가
                    // 셀에 맞추기 위해 음수 자간으로 압축했던" 케이스로 간주. 그렇지
                    // 않으면 음수 자간은 장식적 의도이므로 기존 동작(natural width
                    // 그대로, 좌우 여백 유지)을 유지한다.
                    let natural_w: f64 = comp_line
                        .runs
                        .iter()
                        .map(|r| {
                            let mut ts =
                                resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                            ts.default_tab_width = tab_width;
                            ts.letter_spacing = 0.0;
                            estimate_text_width(&r.text, &ts)
                        })
                        .sum();
                    natural_w > available_width
                }
            {
                // 표 셀 내부 underflow: HWP 편집기가 자연 폭이 셀을 넘는 텍스트를
                // 음수 자간으로 셀 폭에 맞춰 저장했으므로, 재렌더 시 우리 폰트
                // 메트릭으로 좁게 측정되더라도 셀 폭을 채우도록 자간을 양수로 보정.
                //
                // narrow glyph per-char 클램프가 개입하면 선형 분배와 실제 렌더 폭이
                // 어긋나므로 수렴 반복으로 보정한다.
                let mut extra = (available_width - total_text_width) / total_char_count as f64;
                for _ in 0..3 {
                    let mut measured = 0.0f64;
                    for r in &comp_line.runs {
                        let mut ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                        ts.default_tab_width = tab_width;
                        ts.extra_char_spacing = extra;
                        measured += estimate_text_width(&r.text, &ts);
                    }
                    let delta = available_width - measured;
                    if delta.abs() < 0.5 {
                        break;
                    }
                    extra += delta / total_char_count as f64;
                }
                (0.0, extra, 0.0)
            } else {
                (0.0, 0.0, 0.0)
            };

            let line_plain_text: String = comp_line.runs.iter().map(|r| r.text.as_str()).collect();
            let is_answer_sheet_number_label =
                cell_ctx.is_some() && line_plain_text.trim() == "수험번호";
            // [Task #1308 CI follow-up / #1256 regression]
            // 본문/미주 흐름의 TAC-only 줄은 저장된 LINE_SEG x 흐름을 따라야 한다.
            // 빈 TextRun 이 있는 수식-only 문단은 일반 정렬 경로로 들어오므로,
            // Distribute/Center 의 잔여 폭 중앙 오프셋을 적용하면 한컴과 달리 수식 블록이
            // 열 안쪽으로 밀린다. 표 셀 안 수식은 기존처럼 셀 정렬을 따른다.
            let non_cell_tac_only_line = cell_ctx.is_none()
                && !line_tac_offsets_for_width.is_empty()
                && line_plain_text.trim().is_empty();

            // 셀 overflow/underflow 분기로 자간 보정된 경우 정렬 기준 폭은 실제 렌더 폭이어야 함.
            // 특히 #1285 답안지 `수험번호` 라벨은 음수 자간으로 압축된 텍스트를 자연 폭 기준으로
            // 정렬하면 압축 후 남은 폭만큼 왼쪽에 붙는다. 일반 셀은 기존 단순 보정 경로를 유지한다.
            let effective_text_width = if is_answer_sheet_number_label
                && extra_char_sp.abs() > 0.001
                && cell_ctx.is_some()
                && !needs_justify
                && !needs_distribute
                && total_char_count > 1
                && !has_tabs
            {
                comp_line
                    .runs
                    .iter()
                    .map(|r| {
                        let mut ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                        ts.default_tab_width = tab_width;
                        ts.tab_stops = tab_stops.clone();
                        ts.auto_tab_right = auto_tab_right;
                        ts.available_width = available_width;
                        ts.text_start_offset = effective_margin_left;
                        ts.inline_tabs = composed.tab_extended.clone();
                        ts.extra_char_spacing = extra_char_sp;
                        if r.char_overlap.is_some() {
                            let fs = if ts.font_size > 0.0 {
                                ts.font_size
                            } else {
                                12.0
                            };
                            let chars: Vec<char> = r.text.chars().collect();
                            fs * crate::renderer::composer::char_overlap_advance_units(&chars)
                                as f64
                        } else {
                            estimate_text_width(effective_text_for_metrics(r), &ts)
                        }
                    })
                    .sum()
            } else if extra_char_sp > 0.0
                && cell_ctx.is_some()
                && !needs_justify
                && !needs_distribute
                && total_char_count > 1
            {
                total_text_width + extra_char_sp * total_char_count as f64
            } else {
                total_text_width
            };

            // [Task #1285] 답안지 머리말의 `수험번호` 라벨은
            // 파일상 ParaShape가 Center로 들어오더라도 한컴 출력에서는 셀 오른쪽에
            // 붙어 보인다. 기존 중앙 정렬 셀을 흔들지 않도록 해당 라벨에만 적용한다.
            let center_packed_cell_label_as_right = is_answer_sheet_number_label
                && alignment == Alignment::Center
                && !has_tabs
                && line_node.bbox.width <= 110.0
                && effective_text_width >= line_node.bbox.width * 0.75;

            // 비첫줄에서 번호 마커 오프셋 (첫 줄은 마커 렌더링이 x를 전진시킴)
            let num_x_offset = if num_offset > 0.0 && !(line_idx == start_line && start_line == 0) {
                num_offset
            } else {
                0.0
            };
            // [Task #604 R3] wrap_anchor 가 있으면 col_area.x + line_cs_offset 기준,
            // 아니면 effective_col_x (Task #489) 기준.
            let x_base = if wrap_anchor.is_some() {
                col_area.x + effective_margin_left + line_cs_offset
            } else {
                effective_col_x + effective_margin_left
            };
            let x_start = match alignment {
                Alignment::Center => {
                    let align_offset = if center_packed_cell_label_as_right {
                        (available_width - effective_text_width).max(0.0)
                    } else if non_cell_tac_only_line {
                        0.0
                    } else {
                        (available_width - effective_text_width).max(0.0) / 2.0
                    };
                    x_base + inline_offset + num_x_offset + align_offset
                }
                Alignment::Distribute if !needs_distribute || total_char_count <= 1 => {
                    let align_offset = if non_cell_tac_only_line {
                        0.0
                    } else {
                        (available_width - effective_text_width).max(0.0) / 2.0
                    };
                    x_base + inline_offset + num_x_offset + align_offset
                }
                Alignment::Right => {
                    x_base
                        + inline_offset
                        + num_x_offset
                        + (available_width - effective_text_width).max(0.0)
                }
                _ => x_base + inline_offset + num_x_offset, // Left, Justify, Split, Distribute(분배중)
            };

            // TextRun 노드 생성
            // 선행 공백은 x좌표 오프셋으로 처리하여 SVG 뷰어의 폰트 메트릭과 무관하게 정렬
            let mut x = x_start;

            // 개요 번호/글머리표: 첫 줄에서 별도 TextRunNode로 렌더링 (char_start: None)
            if line_idx == start_line && start_line == 0 {
                if let Some(ref num_text) = composed.numbering_text {
                    let num_style =
                        numbering_marker_text_style(styles, para, comp_line.runs.first());
                    let num_width = estimate_text_width(num_text, &num_style);
                    let num_id = tree.next_id();
                    let num_node = RenderNode::new(
                        num_id,
                        RenderNodeType::TextRun(TextRunNode {
                            text: num_text.clone(),
                            style: num_style,
                            char_shape_id: None,
                            para_shape_id: Some(composed.para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: None, // 문서 좌표에 포함되지 않음
                            cell_context: cell_ctx.clone(),
                            is_para_end: false,
                            is_line_break_end: false,
                            rotation: 0.0,
                            is_vertical: false,
                            char_overlap: None,
                            border_fill_id: 0,
                            baseline,
                            field_marker: FieldMarkerType::None,
                        }),
                        BoundingBox::new(x, y, num_width, line_height),
                    );
                    line_node.children.push(num_node);
                    x += num_width;
                }
            }

            // char_offset→x 매핑 (필드 마커 위치 계산용)
            let mut char_x_map: Vec<(usize, f64)> = Vec::new();
            char_x_map.push((comp_line.char_start, x));

            // 조판부호 모드: 인라인 도형 마커 위치 수집
            let show_ctrl = self.show_control_codes.get();
            let shape_markers: Vec<(usize, String)> = if show_ctrl {
                if let Some(ref pa) = para {
                    let ctrl_positions =
                        crate::document_core::helpers::find_logical_control_positions(pa);
                    pa.controls
                        .iter()
                        .enumerate()
                        .filter_map(|(ci, ctrl)| {
                            let pos = ctrl_positions.get(ci).copied().unwrap_or(0);
                            match ctrl {
                                Control::Shape(s) => Some((pos, format!("[{}]", s.shape_name()))),
                                Control::Picture(_) => Some((pos, "[그림]".to_string())),
                                Control::Table(t) if t.common.treat_as_char => {
                                    Some((pos, "[표]".to_string()))
                                }
                                Control::PageHide(_) => Some((pos, "[감추기]".to_string())),
                                Control::PageNumberPos(_) => {
                                    Some((pos, "[쪽 번호 위치]".to_string()))
                                }
                                Control::Header(h) => {
                                    let apply = match h.apply_to {
                                        crate::model::header_footer::HeaderFooterApply::Both => {
                                            "양 쪽"
                                        }
                                        crate::model::header_footer::HeaderFooterApply::Even => {
                                            "짝수 쪽"
                                        }
                                        crate::model::header_footer::HeaderFooterApply::Odd => {
                                            "홀수 쪽"
                                        }
                                    };
                                    Some((pos, format!("[머리말({})]", apply)))
                                }
                                Control::Footer(f) => {
                                    let apply = match f.apply_to {
                                        crate::model::header_footer::HeaderFooterApply::Both => {
                                            "양 쪽"
                                        }
                                        crate::model::header_footer::HeaderFooterApply::Even => {
                                            "짝수 쪽"
                                        }
                                        crate::model::header_footer::HeaderFooterApply::Odd => {
                                            "홀수 쪽"
                                        }
                                    };
                                    Some((pos, format!("[꼬리말({})]", apply)))
                                }
                                Control::Footnote(_) => Some((pos, "[각주]".to_string())),
                                Control::Endnote(_) => Some((pos, "[미주]".to_string())),
                                Control::NewNumber(_) => Some((pos, "[새 번호]".to_string())),
                                Control::Bookmark(bm) => {
                                    Some((pos, format!("[책갈피:{}]", bm.name)))
                                }
                                _ => None,
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // 각주 마커 위치 수집
            let fn_positions: &[(usize, u16, usize)] = &composed.footnote_positions;
            let mut fn_marker_inserted = vec![false; fn_positions.len()];

            let mut pending_right_tab_render: Option<(f64, u8, u8)> = None;
            let mut pending_right_leader_digit_render = false;
            let is_last_run_of_line = |idx: usize| idx == comp_line.runs.len() - 1;
            let mut run_char_pos = comp_line.char_start;
            // 이미 삽입한 도형 마커 추적
            let mut shape_marker_inserted = vec![false; shape_markers.len()];
            // cross-run 탭 감지용 inline_tabs(composed.tab_extended) 커서 — Task #290
            let mut inline_tab_cursor_render: usize = 0;
            for (run_idx, run) in comp_line.runs.iter().enumerate() {
                // 조판부호: 이 run 시작 위치 이전의 도형 마커를 먼저 삽입
                for (smi, (spos, stext)) in shape_markers.iter().enumerate() {
                    if !shape_marker_inserted[smi] && *spos <= run_char_pos {
                        shape_marker_inserted[smi] = true;
                        let base_style =
                            resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                        let mut ms = base_style;
                        ms.color = 0x0000FF; // BGR: 빨간색
                        ms.font_size *= 0.55;
                        let mw = estimate_text_width(stext, &ms);
                        let mid = tree.next_id();
                        let mn = RenderNode::new(
                            mid,
                            RenderNodeType::TextRun(TextRunNode {
                                text: stext.clone(),
                                style: ms,
                                char_shape_id: None,
                                para_shape_id: Some(composed.para_style_id),
                                section_index: Some(section_index),
                                para_index: Some(para_index),
                                char_start: None,
                                cell_context: cell_ctx.clone(),
                                is_para_end: false,
                                is_line_break_end: false,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: 0,
                                baseline,
                                field_marker: FieldMarkerType::ShapeMarker(*spos),
                            }),
                            BoundingBox::new(x, y, mw, line_height),
                        );
                        line_node.children.push(mn);
                        x += mw;
                    }
                }
                let mut text_style =
                    resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                text_style.default_tab_width = tab_width;
                text_style.tab_stops = tab_stops.clone();
                text_style.auto_tab_right = auto_tab_right;
                text_style.available_width = available_width;
                text_style.text_start_offset = effective_margin_left;
                text_style.inline_tabs = composed.tab_extended.clone();
                if pending_right_leader_digit_render {
                    if run.text.trim().is_empty() {
                        pending_right_leader_digit_render = true;
                    } else {
                        if run.text.trim().chars().all(|ch| ch.is_ascii_digit()) {
                            if let Some(tab) = tab_stops
                                .iter()
                                .rev()
                                .find(|tab| tab.tab_type == 1 && tab.fill_type != 0)
                            {
                                let digit_w = estimate_text_width(run.text.trim(), &text_style);
                                let target =
                                    if composed.tab_extended.is_empty() && available_width > 0.0 {
                                        effective_margin_left + available_width
                                    } else {
                                        tab.position
                                    };
                                let gap = if composed.tab_extended.is_empty() {
                                    0.0
                                } else {
                                    text_style.font_size * 0.25
                                };
                                x = col_area.x + target - gap - digit_w;
                            }
                        }
                        pending_right_leader_digit_render = false;
                    }
                }
                // 교차 run 오른쪽/가운데 탭: 이전 run이 \t로 끝났고
                // 해당 탭이 오른쪽/가운데 탭이면 이 run을 역방향으로 이동
                if let Some((tab_pos, tab_type, fill_type)) = pending_right_tab_render.take() {
                    // [Task #279] 공백만 있는 run 은 right/center tab 정렬 단위가 아니다.
                    // 한컴 목차의 장제목 케이스: "Ⅰ. 사업개요\t" + " " + "3" 으로 run 분리되며,
                    // " " run 에 right tab 을 적용하면 페이지번호 "3" 이 effective_pos 보다
                    // 공백 폭만큼 우측으로 밀려 소제목 정렬과 어긋난다. 공백 only run 은 정렬을
                    // 건너뛰고 pending 을 다음 의미있는 run 으로 carry-over.
                    if (tab_type == 1 || tab_type == 2) && run.text.trim().is_empty() {
                        // carry-over: 공백 run 은 정렬 단위가 아님. leader 보정도 다음 run 시점으로
                        // 위임 (그 시점의 leader-bearing TextRun 검색이 \t 가진 진짜 leader run 을 찾음).
                        pending_right_tab_render = Some((tab_pos, tab_type, fill_type));
                    } else {
                        text_style.line_x_offset = x - col_area.x;
                        // [Task #279] 리더(fill_type ≠ 0) 가 있는 RIGHT 탭은 "이 줄 우측 끝까지" 의미.
                        // 한컴은 TabDef.position 을 절대 좌표로 신뢰하지 않고 리더 도트의 시멘틱
                        // (= 단/셀 콘텐츠 영역 우측 끝까지 채움) 으로 재해석한다.
                        // 셀 안 문단에서는 col_area 가 이미 cell padding 적용된 inner_area 이므로
                        // `effective_margin_left + available_width` 가 inner 우측 끝.
                        // tab_pos (HWP 저장값) 이 inner 우측 끝을 초과하면 셀 padding_right 침범이므로 강제 클램핑.
                        // [Task #874] auto_tab_right (fill_type=0) 도 effective_margin_left 변환 필요.
                        let effective_pos = if tab_type == 1 {
                            effective_margin_left
                                + (if fill_type != 0 {
                                    available_width
                                } else {
                                    tab_pos
                                })
                        } else {
                            tab_pos
                        };
                        // [Issue #842 #4] 탭 다음 콘텐츠가 여러 composed run 으로 쪼개진 경우
                        // (스크립트·char-shape 경계, 예 "Ctrl+(회색)5") 전체 블록 폭 기준 정렬.
                        let next_w = right_tab_block_width(
                            &comp_line.runs,
                            run_idx,
                            styles,
                            tab_width,
                            &tab_stops,
                            auto_tab_right,
                            available_width,
                        );
                        match tab_type {
                            1 => {
                                x = col_area.x + effective_pos - next_w;
                            }
                            2 => {
                                x = col_area.x + effective_pos - next_w / 2.0;
                            }
                            _ => {}
                        }
                        // [Task #279] 직전 run 의 leader 끝 위치를 페이지번호 시작 x 직전까지 단축.
                        // 한컴은 페이지번호 폭에 따라 리더 길이가 달라지도록 조판한다 (한 자리 vs
                        // 두 자리 페이지번호의 leader 끝점이 다름). cross-run RIGHT 정렬 후
                        // tab_leaders 가 있는 직전 TextRun 을 거슬러 찾아 마지막 항목 end_x 를 보정.
                        // 공백 only run carry-over 케이스 대비 — 가장 마지막 TextRun 이 공백 run 이고
                        // leader 가 없으면 그 이전 (\t 가진 leader-bearing) TextRun 을 찾음.
                        if let Some(prev_run_node) = line_node.children.iter_mut().rev().find(|n| {
                            if let RenderNodeType::TextRun(tr) = &n.node_type {
                                !tr.style.tab_leaders.is_empty()
                            } else {
                                false
                            }
                        }) {
                            let prev_bbox_x = prev_run_node.bbox.x;
                            if let RenderNodeType::TextRun(prev_text_run) =
                                &mut prev_run_node.node_type
                            {
                                let space_gap = if text_style.font_size > 0.0 {
                                    text_style.font_size * 0.25
                                } else {
                                    3.0
                                };
                                for leader in &mut prev_text_run.style.tab_leaders {
                                    let new_end_x =
                                        (x - prev_bbox_x - space_gap).max(leader.start_x);
                                    if new_end_x < leader.end_x {
                                        leader.end_x = new_end_x;
                                    }
                                }
                            }
                        }
                    } // end else (non-blank run)
                }
                text_style.line_x_offset = x - col_area.x;
                text_style.extra_word_spacing = extra_word_sp;
                text_style.extra_char_spacing = extra_char_sp;
                text_style.extra_dash_advance = extra_dash_sp;
                // [Task #874 #2] composer lang split (예: "F3→Alt+I" → "F3"/"→"/"Alt+I")
                // 으로 auto_tab_right post-tab 콘텐츠가 후속 run 으로 흩어진 경우, 현재
                // run 내부 seg_w 만으로는 우측 정렬 위치가 어긋남. 후속 run 합산을 미리
                // 계산해 text_style.right_tab_block_width_override 로 주입한다.
                if auto_tab_right && run.text.contains('\t') && run_idx + 1 < comp_line.runs.len() {
                    let tab_byte = run.text.rfind('\t').unwrap();
                    let post_tab: String = run.text[tab_byte + '\t'.len_utf8()..].to_string();
                    let no_more_tabs_after_in_run = !post_tab.contains('\t');
                    let no_tabs_in_subsequent = comp_line
                        .runs
                        .iter()
                        .skip(run_idx + 1)
                        .all(|r| !r.text.contains('\t'));
                    if no_more_tabs_after_in_run && no_tabs_in_subsequent {
                        let mut ts_measure = text_style.clone();
                        ts_measure.right_tab_block_width_override = None;
                        let post_tab_w = estimate_text_width(&post_tab, &ts_measure);
                        let subsequent_w = right_tab_block_width(
                            &comp_line.runs,
                            run_idx + 1,
                            styles,
                            tab_width,
                            &tab_stops,
                            auto_tab_right,
                            available_width,
                        );
                        text_style.right_tab_block_width_override = Some(post_tab_w + subsequent_w);
                    }
                }
                let run_border_fill_id = styles
                    .char_styles
                    .get(run.char_style_id as usize)
                    .map(|cs| cs.border_fill_id)
                    .unwrap_or(0);
                let full_width = if run.char_overlap.is_some() {
                    // 글자겹침: 한 컨트롤은 payload 글자 수와 무관하게 1글자 폭.
                    let fs = if text_style.font_size > 0.0 {
                        text_style.font_size
                    } else {
                        12.0
                    };
                    let chars: Vec<char> = run.text.chars().collect();
                    fs * crate::renderer::composer::char_overlap_advance_units(&chars) as f64
                } else {
                    estimate_text_width(effective_text_for_metrics(run), &text_style)
                };
                // 탭 리더 계산: 탭이 포함된 run에서 채움 기호 정보 추출
                // inline_tabs를 일시 제거하여 tab_stops 기반 위치 계산과 일관되게 함
                if has_tabs && run.text.contains('\t') {
                    let saved_inline_tabs = std::mem::take(&mut text_style.inline_tabs);
                    let positions = compute_char_positions(&run.text, &text_style);
                    text_style.inline_tabs = saved_inline_tabs;
                    text_style.tab_leaders = extract_tab_leaders_with_extended(
                        &run.text,
                        &positions,
                        &text_style,
                        &composed.tab_extended,
                    );
                }
                // 교차 run 오른쪽/가운데 탭 감지 — Task #290:
                // inline_tabs(composed.tab_extended) 가 LEFT 를 명시하면 cross-run pending 을 설정하지 않는다.
                // [Task #279] trailing 공백 (\t 뒤에 따라오는 ' ') 도 허용 — 목차 소제목의
                // 들여쓰기 문단에서 한컴이 "\t " 형태로 저장하는 케이스가 있음.
                let trimmed_end_r = run
                    .text
                    .trim_end_matches(|c: char| c == ' ' || c == '\u{2007}');
                if has_tabs && trimmed_end_r.ends_with('\t') {
                    let run_tab_count = run.text.chars().filter(|c| *c == '\t').count();
                    if run_tab_count > 0 {
                        let last_inline_idx = inline_tab_cursor_render + run_tab_count - 1;
                        pending_right_tab_render = resolve_last_tab_pending(
                            &run.text,
                            last_inline_idx,
                            &composed.tab_extended,
                            &text_style,
                            &tab_stops,
                            tab_width,
                            auto_tab_right,
                            available_width,
                        );
                    }
                }
                if has_tabs
                    && run.text.contains('\t')
                    && run
                        .text
                        .rsplit_once('\t')
                        .map(|(_, after)| after.trim().is_empty())
                        .unwrap_or(false)
                    && tab_stops
                        .iter()
                        .any(|tab| tab.tab_type == 1 && tab.fill_type != 0)
                {
                    pending_right_leader_digit_render = true;
                }
                let run_char_count = if run.char_overlap.is_some() {
                    // 글자겹침(CharOverlap)은 HWP char_offset 공간에서 1개 위치만 차지
                    let chars: Vec<char> = run.text.chars().collect();
                    crate::renderer::composer::char_overlap_advance_units(&chars)
                } else {
                    run.text.chars().count()
                };
                let run_char_end = run_char_pos + run_char_count;
                let is_last_run = is_last_line_of_para && is_last_run_of_line(run_idx);
                let is_line_break = comp_line.has_line_break && is_last_run_of_line(run_idx);

                // treat_as_char 분기점: run 내 이미지 위치 목록 (rel_pos, width_px, control_index)
                // 마지막 run에서는 run_char_end 위치의 TAC도 포함 (문단 끝 수식/그림)
                // [Task #960] has_line_break line 의 마지막 run 도 run_char_end 위치 의 TAC
                // 포함. HWP3 의 char_offsets gap 분석으로 매핑된 control 위치가 `\n` 문자
                // 에 떨어지면 (예: 시험지 page 2 pi=117 의 cases formula at position 30 =
                // `\n` 위치), 그 line 의 chars range [start, end) 에서 end 가 `\n` 위치
                // 이므로 누락. has_line_break line 의 마지막 run 의 end position 도 TAC
                // 포함하면 line 의 정확한 위치에 inline emit.
                //
                // 다만 다음 LineSeg/ComposedLine 이 같은 char 위치에서 시작하면
                // 그 boundary TAC 는 다음 줄의 시작 글자처럼 취급해야 한다. 현재 줄에서도
                // end TAC 로 허용하면 미주 수식이 이전 줄 끝과 다음 줄 시작에 중복 emit 되어
                // 같은 수식이 겹친다.
                let next_line_starts_at_run_end = composed
                    .lines
                    .get(line_idx + 1)
                    .is_some_and(|next| next.char_start == run_char_end);
                let allow_end_tac = (is_last_run
                    || (comp_line.has_line_break && is_last_run_of_line(run_idx)))
                    && !next_line_starts_at_run_end;
                let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px
                    .iter()
                    .filter(|(pos, _, _)| {
                        *pos >= run_char_pos
                            && (*pos < run_char_end || (allow_end_tac && *pos == run_char_end))
                    })
                    .map(|(pos, w, ci)| (pos - run_char_pos, *w, *ci))
                    .collect();

                // [Task #960] env-gated TAC line-mapping 추적
                if std::env::var("RHWP_DEBUG_PARA_TAC").is_ok() && !tac_offsets_px.is_empty() {
                    eprintln!("  TAC_LINE pi={} line_idx={} run_idx={} run_char_pos={} run_char_end={} y={:.1} lh={:.1} ls={:.1} raw_lh={:.1} baseline={:.1} run_tacs={:?}",
                        para_index, line_idx, run_idx, run_char_pos, run_char_end, y, line_height, line_spacing_px, raw_lh, baseline, run_tacs);
                }

                if run_tacs.is_empty() {
                    // tac 없음: 기존 렌더링 경로
                    // 선행 공백 분리
                    let leading_spaces: String =
                        run.text.chars().take_while(|c| *c == ' ').collect();
                    let content = run.text.trim_start_matches(' ');

                    // 글자 테두리/배경: bbox 계산용 run_x, run_w
                    let (run_x, run_w) = if !leading_spaces.is_empty() && !content.is_empty() {
                        let sw = estimate_text_width(&leading_spaces, &text_style);
                        (x + sw, estimate_text_width(content, &text_style))
                    } else {
                        (x, full_width)
                    };

                    // 글자 배경 사각형 (텍스트 앞에 삽입)
                    if run_border_fill_id > 0 {
                        let bf_idx = (run_border_fill_id as usize).saturating_sub(1);
                        if let Some(bs) = styles.border_styles.get(bf_idx) {
                            if let Some(fill_color) = bs.fill_color {
                                let rect_id = tree.next_id();
                                let rect_node = RenderNode::new(
                                    rect_id,
                                    RenderNodeType::Rectangle(RectangleNode::new(
                                        0.0,
                                        ShapeStyle {
                                            fill_color: Some(fill_color),
                                            stroke_color: None,
                                            stroke_width: 0.0,
                                            ..Default::default()
                                        },
                                        None,
                                    )),
                                    BoundingBox::new(run_x, y, run_w, line_height),
                                );
                                line_node.children.push(rect_node);
                            }
                        }
                    }

                    // 형광펜 배경 사각형 (RangeTag type=2)
                    if let Some(p) = para {
                        if !p.range_tags.is_empty() {
                            let char_w = if run_char_count > 0 {
                                run_w / run_char_count as f64
                            } else {
                                0.0
                            };
                            for rt in &p.range_tags {
                                let rt_type = (rt.tag >> 24) & 0xFF;
                                if rt_type != 2 {
                                    continue;
                                }
                                let rt_start = rt.start as usize;
                                let rt_end = rt.end as usize;
                                // run과 RangeTag가 겹치는 문자 범위
                                let overlap_start = rt_start.max(run_char_pos);
                                let overlap_end = rt_end.min(run_char_end);
                                if overlap_start >= overlap_end {
                                    continue;
                                }
                                let hl_color = rt.tag & 0x00FFFFFF;
                                let hl_x = run_x + (overlap_start - run_char_pos) as f64 * char_w;
                                let hl_w = (overlap_end - overlap_start) as f64 * char_w;
                                let rect_id = tree.next_id();
                                let rect_node = RenderNode::new(
                                    rect_id,
                                    RenderNodeType::Rectangle(RectangleNode::new(
                                        0.0,
                                        ShapeStyle {
                                            fill_color: Some(hl_color),
                                            stroke_color: None,
                                            stroke_width: 0.0,
                                            ..Default::default()
                                        },
                                        None,
                                    )),
                                    BoundingBox::new(hl_x, y, hl_w, line_height),
                                );
                                line_node.children.push(rect_node);
                            }
                        }
                    }

                    let mut fn_split_extra = 0.0f64; // 각주 마커 삽입으로 인한 추가 폭
                    {
                        // run 내 각주 위치 수집 (run 내 상대 위치, 각주 번호, fn_positions 인덱스, control 인덱스)
                        // 마지막 run에서는 run_char_end 위치의 각주도 포함 (문단 끝 각주)
                        let is_last = is_last_run_of_line(run_idx);
                        let run_fn_markers: Vec<(usize, u16, usize, usize)> = fn_positions
                            .iter()
                            .enumerate()
                            .filter_map(|(fni, &(fpos, fnum, ctrl_idx))| {
                                if is_leading_endnote_marker_rendered_as_prefix(
                                    para,
                                    ctrl_idx,
                                    line_idx,
                                    start_line,
                                    fpos,
                                    comp_line.char_start,
                                ) {
                                    // 미주는 첫 줄 앞에 본문 크기 번호를 별도 TextRun으로 이미 그린다.
                                    // 같은 위치의 위첨자 마커를 다시 만들면 `문26)`처럼 제목이 중복된다.
                                    fn_marker_inserted[fni] = true;
                                    return None;
                                }
                                let in_range = fpos >= run_char_pos
                                    && (fpos < run_char_end || (is_last && fpos == run_char_end));
                                if !fn_marker_inserted[fni] && in_range {
                                    Some((fpos - run_char_pos, fnum, fni, ctrl_idx))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if run_fn_markers.is_empty() {
                            // 각주 없음: 기존 방식으로 전체 TextRun 생성
                            let run_id = tree.next_id();
                            let run_node = RenderNode::new(
                                run_id,
                                RenderNodeType::TextRun(TextRunNode {
                                    text: run.text.clone(),
                                    style: text_style,
                                    char_shape_id: Some(run.char_style_id),
                                    para_shape_id: Some(composed.para_style_id),
                                    section_index: Some(section_index),
                                    para_index: Some(para_index),
                                    char_start: Some(char_offset),
                                    cell_context: cell_ctx.clone(),
                                    is_para_end: is_last_run,
                                    is_line_break_end: is_line_break,
                                    rotation: 0.0,
                                    is_vertical: false,
                                    char_overlap: run.char_overlap.clone(),
                                    border_fill_id: run_border_fill_id,
                                    baseline,
                                    field_marker: FieldMarkerType::None,
                                }),
                                BoundingBox::new(x, y, full_width, line_height),
                            );
                            line_node.children.push(run_node);
                        } else {
                            // 각주 있음: run을 각주 위치에서 분할하여 TextRun + FootnoteMarker 교차 생성
                            let run_chars: Vec<char> = run.text.chars().collect();
                            let mut seg_start = 0usize; // run 내 상대 문자 인덱스
                            let mut sub_x = x;
                            let mut sub_char_offset = char_offset;

                            for &(rel_pos, fnum, fni, ctrl_idx) in &run_fn_markers {
                                fn_marker_inserted[fni] = true;
                                // 각주 앞 텍스트 세그먼트
                                if rel_pos > seg_start {
                                    let seg_text: String =
                                        run_chars[seg_start..rel_pos].iter().collect();
                                    let seg_w = estimate_text_width(&seg_text, &text_style);
                                    let seg_id = tree.next_id();
                                    let seg_node = RenderNode::new(
                                        seg_id,
                                        RenderNodeType::TextRun(TextRunNode {
                                            text: seg_text,
                                            style: text_style.clone(),
                                            char_shape_id: Some(run.char_style_id),
                                            para_shape_id: Some(composed.para_style_id),
                                            section_index: Some(section_index),
                                            para_index: Some(para_index),
                                            char_start: Some(sub_char_offset),
                                            cell_context: cell_ctx.clone(),
                                            is_para_end: false,
                                            is_line_break_end: false,
                                            rotation: 0.0,
                                            is_vertical: false,
                                            char_overlap: None,
                                            border_fill_id: run_border_fill_id,
                                            baseline,
                                            field_marker: FieldMarkerType::None,
                                        }),
                                        BoundingBox::new(sub_x, y, seg_w, line_height),
                                    );
                                    line_node.children.push(seg_node);
                                    sub_x += seg_w;
                                    sub_char_offset += rel_pos - seg_start;
                                }
                                // FootnoteMarker 노드
                                let fn_text = note_marker_text_from_control(
                                    para.and_then(|p| p.controls.get(ctrl_idx)),
                                    fnum,
                                );
                                let base_ts = &text_style;
                                let sup_size = (base_ts.font_size * 0.55).max(7.0);
                                let sup_ts = TextStyle {
                                    font_size: sup_size,
                                    font_family: base_ts.font_family.clone(),
                                    color: base_ts.color,
                                    ..Default::default()
                                };
                                let sup_w = estimate_text_width(&fn_text, &sup_ts);
                                let fid = tree.next_id();
                                let fn_node = RenderNode::new(
                                    fid,
                                    RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
                                        number: fnum,
                                        text: fn_text,
                                        base_font_size: base_ts.font_size,
                                        font_family: base_ts.font_family.clone(),
                                        color: base_ts.color,
                                        section_index,
                                        para_index,
                                        control_index: ctrl_idx,
                                    }),
                                    BoundingBox::new(sub_x, y, sup_w, line_height),
                                );
                                line_node.children.push(fn_node);
                                sub_x += sup_w;
                                fn_split_extra += sup_w;
                                seg_start = rel_pos;
                            }
                            // 마지막 세그먼트 (각주 뒤 나머지 텍스트)
                            if seg_start < run_chars.len() {
                                let seg_text: String = run_chars[seg_start..].iter().collect();
                                let seg_w = estimate_text_width(&seg_text, &text_style);
                                let seg_id = tree.next_id();
                                let seg_node = RenderNode::new(
                                    seg_id,
                                    RenderNodeType::TextRun(TextRunNode {
                                        text: seg_text,
                                        style: text_style,
                                        char_shape_id: Some(run.char_style_id),
                                        para_shape_id: Some(composed.para_style_id),
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        char_start: Some(sub_char_offset),
                                        cell_context: cell_ctx.clone(),
                                        is_para_end: is_last_run,
                                        is_line_break_end: is_line_break,
                                        rotation: 0.0,
                                        is_vertical: false,
                                        char_overlap: run.char_overlap.clone(),
                                        border_fill_id: run_border_fill_id,
                                        baseline,
                                        field_marker: FieldMarkerType::None,
                                    }),
                                    BoundingBox::new(sub_x, y, seg_w, line_height),
                                );
                                line_node.children.push(seg_node);
                            }
                        }
                    }

                    // 글자 테두리선 (텍스트 뒤에 삽입)
                    if run_border_fill_id > 0 {
                        let bf_idx = (run_border_fill_id as usize).saturating_sub(1);
                        if let Some(bs) = styles.border_styles.get(bf_idx) {
                            let bx = run_x;
                            let by = y;
                            let bw = run_w;
                            let bh = line_height;
                            // borders[0]=left, [1]=right, [2]=top, [3]=bottom
                            let border_pairs: [(f64, f64, f64, f64, usize); 4] = [
                                (bx, by, bx, by + bh, 0),           // left
                                (bx + bw, by, bx + bw, by + bh, 1), // right
                                (bx, by, bx + bw, by, 2),           // top
                                (bx, by + bh, bx + bw, by + bh, 3), // bottom
                            ];
                            for (lx1, ly1, lx2, ly2, bi) in border_pairs {
                                let nodes = create_border_line_nodes(
                                    tree,
                                    &bs.borders[bi],
                                    lx1,
                                    ly1,
                                    lx2,
                                    ly2,
                                );
                                for n in nodes {
                                    line_node.children.push(n);
                                }
                            }
                        }
                    }

                    x += full_width + fn_split_extra;
                } else {
                    // tac 있음: 분기점마다 하위 텍스트 런 생성 (이미지는 layout.rs에서 별도 렌더링)
                    let run_chars: Vec<char> = run.text.chars().collect();
                    let mut seg_start = 0usize;
                    let mut sub_char_offset = char_offset;

                    // [Task #455] 외부 문단 본문 텍스트는 글상자 유무와 무관하게 항상 렌더한다.
                    // 글상자(TextBox) 자체와 그 내부 텍스트("개화" 같은)는
                    // shape_layout 이 inline_shape_position 을 보고 별도 패스에서 렌더하므로 중복되지 않는다.

                    for &(tac_rel, tac_w, tac_ci) in &run_tacs {
                        // tac 앞 텍스트 세그먼트 렌더링
                        if seg_start < tac_rel {
                            let seg_text: String = run_chars[seg_start..tac_rel].iter().collect();
                            let mut seg_style = text_style.clone();
                            seg_style.line_x_offset = x - col_area.x;
                            // 탭 리더 계산
                            if has_tabs && seg_text.contains('\t') {
                                let positions = compute_char_positions(&seg_text, &seg_style);
                                seg_style.tab_leaders = extract_tab_leaders_with_extended(
                                    &seg_text,
                                    &positions,
                                    &seg_style,
                                    &composed.tab_extended,
                                );
                            }
                            let seg_w = estimate_text_width(&seg_text, &seg_style);
                            let seg_char_count = tac_rel - seg_start;
                            {
                                let sub_run_id = tree.next_id();
                                let sub_run_node = RenderNode::new(
                                    sub_run_id,
                                    RenderNodeType::TextRun(TextRunNode {
                                        text: seg_text,
                                        style: seg_style,
                                        char_shape_id: Some(run.char_style_id),
                                        para_shape_id: Some(composed.para_style_id),
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        char_start: Some(sub_char_offset),
                                        cell_context: cell_ctx.clone(),
                                        is_para_end: false,
                                        is_line_break_end: false,
                                        rotation: 0.0,
                                        is_vertical: false,
                                        char_overlap: run.char_overlap.clone(),
                                        border_fill_id: run_border_fill_id,
                                        baseline,
                                        field_marker: FieldMarkerType::None,
                                    }),
                                    BoundingBox::new(x, y, seg_w, line_height),
                                );
                                line_node.children.push(sub_run_node);
                            }
                            x += seg_w;
                            sub_char_offset += seg_char_count;
                        }
                        // 인라인 이미지 렌더링: 텍스트 흐름 순서에 맞게 이 위치에서 직접 렌더링
                        if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                            if let Some(ctrl) = p.controls.get(tac_ci) {
                                if let Control::Picture(pic) = ctrl {
                                    let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    // LINE_SEG vpos가 TopAndBottom 흐름 위치를 이미 담고 있으면
                                    // sibling 예약 높이를 다시 더하지 않는다.
                                    let sibling_reserved_px =
                                        if para_topbottom_line_vpos_base.is_some() {
                                            0.0
                                        } else {
                                            hwpunit_to_px(
                                                calc_sibling_topandbottom_reserved_hu(&p.controls),
                                                self.dpi,
                                            )
                                        };
                                    if raw_lh + 4.0 >= pic_h {
                                        current_line_reserved_tac_picture_height = Some(pic_h);
                                    }
                                    let label_extra = tac_picture_label_extra_for_line(
                                        cell_ctx.as_ref(),
                                        runs_all_whitespace,
                                        raw_lh,
                                        current_line_reserved_tac_picture_height,
                                        max_fs,
                                        line_spacing_px,
                                    );
                                    let base_img_y = if label_extra > 0.0 {
                                        y + label_extra
                                    } else {
                                        (y + baseline - pic_h).max(y)
                                    };
                                    let img_y = base_img_y + sibling_reserved_px;
                                    let bin_data_id = pic.image_attr.bin_data_id;
                                    let image_data =
                                        find_bin_data(bdc, bin_data_id).map(|c| c.data.clone());
                                    let crop = {
                                        let c = &pic.crop;
                                        if c.right > c.left
                                            && c.bottom > c.top
                                            && (c.left != 0
                                                || c.top != 0
                                                || c.right != 0
                                                || c.bottom != 0)
                                        {
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
                                    // [Task #1151 v7 항목 7] ImageNode 생성 helper 통합.
                                    let img_node = make_picture_image_node(
                                        tree,
                                        pic,
                                        section_index,
                                        para_index,
                                        tac_ci,
                                        cell_ctx.as_ref(),
                                        crop,
                                        original_size_hu,
                                        bin_data_id,
                                        image_data,
                                        BoundingBox::new(x, img_y, tac_w, pic_h),
                                    );
                                    line_node.children.push(img_node);
                                    // [Task #864 Stage G] inline TAC picture 의 위치 등록.
                                    // layout.rs 의 TAC inline branch (line ~2906) 가
                                    // already_registered 체크로 중복 emit 방지하나, 기존
                                    // paragraph_layout 은 picture 에 대해 register 누락
                                    // → layout.rs branch 가 또 emit 하여 동일 picture 가
                                    // 두 위치 (top-aligned + baseline-aligned) 에 그려짐.
                                    // HWP3 sample14 에서 caption 이 duplicate image 에 가려져
                                    // 보이지 않던 결함 정정.
                                    tree.set_inline_shape_position(
                                        section_index,
                                        para_index,
                                        tac_ci,
                                        cell_ctx.as_ref(),
                                        x,
                                        img_y,
                                    );
                                }
                            }
                        }
                        // 인라인 Shape(글상자) 렌더링: 텍스트 흐름 순서에 맞게 배치
                        // Shape 내부의 텍스트/테두리를 직접 렌더링하고, 별도 Shape 패스에서는 스킵
                        if let Some(p) = para {
                            if let Some(Control::Shape(shape)) = p.controls.get(tac_ci) {
                                let common = shape.common();
                                let shape_h_hu = (common.height as i32)
                                    .max(shape.shape_attr().current_height as i32);
                                let shape_h = hwpunit_to_px(shape_h_hu, self.dpi);
                                if raw_lh + 4.0 >= shape_h {
                                    current_line_reserved_tac_picture_height = Some(shape_h);
                                }
                                let label_extra = tac_picture_label_extra_for_line(
                                    cell_ctx.as_ref(),
                                    runs_all_whitespace,
                                    raw_lh,
                                    current_line_reserved_tac_picture_height,
                                    max_fs,
                                    line_spacing_px,
                                );
                                let shape_y = if label_extra > 0.0 {
                                    y + label_extra
                                } else {
                                    (y + baseline - shape_h).max(y)
                                };
                                // 인라인 좌표 등록 → shape_layout.rs에서 이 Shape를 스킵
                                tree.set_inline_shape_position(
                                    section_index,
                                    para_index,
                                    tac_ci,
                                    cell_ctx.as_ref(),
                                    x,
                                    shape_y,
                                );
                            }
                        }
                        // 인라인 수식: 직접 EquationNode로 렌더링
                        if let Some(p) = para {
                            if let Some(Control::Equation(eq)) = p.controls.get(tac_ci) {
                                // 수식 스크립트 → AST → 레이아웃 → SVG 조각
                                let tokens =
                                    crate::renderer::equation::tokenizer::tokenize(&eq.script);
                                let ast = crate::renderer::equation::parser::EqParser::new(tokens)
                                    .parse();
                                let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                                let layout_box =
                                    crate::renderer::equation::layout::EqLayout::new(font_size_px)
                                        .layout(&ast);
                                let color_str =
                                    crate::renderer::equation::svg_render::eq_color_to_svg(
                                        eq.color,
                                    );
                                let svg_content =
                                    crate::renderer::equation::svg_render::render_equation_svg(
                                        &layout_box,
                                        &color_str,
                                        font_size_px,
                                    );
                                // HWP 저장 높이를 우선 사용 (한컴 조판 결과 기준)
                                let hwp_eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                                let eq_h = if hwp_eq_h > 0.0 {
                                    hwp_eq_h
                                } else {
                                    layout_box.height
                                };
                                // 텍스트와 섞인 인라인 수식뿐 아니라 공백 run 안의 TAC 수식도
                                // baseline을 맞춘다. 수식 renderer는 bbox 높이로 세로 스케일하지
                                // 않으므로 y에 직접 붙이면 큰 루트/분수 수식이 아래 줄을 덮는다.
                                let eq_y = if cell_ctx.is_none()
                                    && comp_line.runs.iter().all(|r| {
                                        !r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                    }) {
                                    y + baseline - layout_box.baseline
                                } else {
                                    (y + baseline - layout_box.baseline).max(y)
                                };
                                let (eq_cell_idx, eq_cell_para_idx) =
                                    if let Some(ref ctx) = cell_ctx {
                                        (
                                            Some(ctx.path[0].cell_index),
                                            Some(ctx.path[0].cell_para_index),
                                        )
                                    } else {
                                        (None, None)
                                    };
                                let note_ref = if cell_ctx.is_none() {
                                    self.note_ref_for_endnote_equation(para_index, tac_ci)
                                } else {
                                    None
                                };
                                let eq_node = RenderNode::new(
                                    tree.next_id(),
                                    RenderNodeType::Equation(
                                        crate::renderer::render_tree::EquationNode {
                                            svg_content,
                                            layout_box,
                                            color_str,
                                            color: eq.color,
                                            font_size: font_size_px,
                                            section_index: note_ref
                                                .as_ref()
                                                .map(|r| r.section_index)
                                                .or(Some(section_index)),
                                            para_index: if let Some(ref ctx) = cell_ctx {
                                                Some(ctx.parent_para_index)
                                            } else {
                                                Some(para_index)
                                            },
                                            control_index: if let Some(ref ctx) = cell_ctx {
                                                Some(ctx.path[0].control_index)
                                            } else {
                                                Some(tac_ci)
                                            },
                                            cell_index: eq_cell_idx,
                                            cell_para_index: eq_cell_para_idx,
                                            note_ref,
                                        },
                                    ),
                                    BoundingBox::new(x, eq_y, tac_w, eq_h),
                                );
                                line_node.children.push(eq_node);
                                // 인라인 좌표 등록 → shape_layout에서 이 수식을 스킵
                                tree.set_inline_shape_position(
                                    section_index,
                                    para_index,
                                    tac_ci,
                                    cell_ctx.as_ref(),
                                    x,
                                    eq_y,
                                );
                            }
                        }
                        // 인라인 TAC 표: 텍스트 흐름 위치에 직접 렌더링
                        // 표 하단 = 베이스라인 + outer_margin_bottom
                        if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                            if let Some(Control::Table(t)) = p.controls.get(tac_ci) {
                                let raw_seg_width =
                                    p.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                                let seg_width = if raw_seg_width > 0 {
                                    raw_seg_width
                                } else {
                                    px_to_hwpunit(col_area.width, self.dpi)
                                };
                                let should_render_inline = cell_ctx.is_some()
                                    || crate::renderer::height_measurer::is_tac_table_inline(
                                        t,
                                        seg_width,
                                        &p.text,
                                        &p.controls,
                                    );
                                let already_rendered = tree
                                    .get_inline_shape_position(
                                        section_index,
                                        para_index,
                                        tac_ci,
                                        cell_ctx.as_ref(),
                                    )
                                    .is_some();
                                if t.common.treat_as_char
                                    && should_render_inline
                                    && !already_rendered
                                {
                                    let table_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                                    let om_bottom =
                                        hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi);
                                    let table_y = (y + baseline + om_bottom - table_h).max(y);
                                    self.layout_table(
                                        tree,
                                        col_node,
                                        t,
                                        section_index,
                                        styles,
                                        0,
                                        col_area,
                                        table_y,
                                        bdc,
                                        None,
                                        0,
                                        Some((para_index, tac_ci)),
                                        alignment,
                                        None,
                                        0.0,
                                        0.0,
                                        Some(x),
                                        None,
                                        None,
                                        false,
                                        false,
                                    );
                                    // 스킵 마커 등록 (별도 Table PageItem에서 중복 렌더 방지)
                                    tree.set_inline_shape_position(
                                        section_index,
                                        para_index,
                                        tac_ci,
                                        cell_ctx.as_ref(),
                                        x,
                                        table_y,
                                    );
                                }
                            }
                        }
                        // 인라인 양식 개체 렌더링
                        if let Some(p) = para {
                            if let Some(Control::Form(f)) = p.controls.get(tac_ci) {
                                let form_h = hwpunit_to_px(f.height as i32, self.dpi);
                                let form_y = (y + baseline - form_h).max(y);
                                // 셀 내부인 경우 cell_location 채우기
                                let cell_location = cell_ctx.as_ref().map(|ctx| {
                                    let e = &ctx.path[0];
                                    (
                                        ctx.parent_para_index,
                                        e.control_index,
                                        e.cell_index,
                                        e.cell_para_index,
                                    )
                                });
                                let form_node = RenderNode::new(
                                    tree.next_id(),
                                    RenderNodeType::FormObject(FormObjectNode {
                                        form_type: f.form_type,
                                        caption: f.caption.clone(),
                                        text: f.text.clone(),
                                        fore_color: form_color_to_css(f.fore_color),
                                        back_color: form_color_to_css(f.back_color),
                                        value: f.value,
                                        enabled: f.enabled,
                                        section_index,
                                        para_index,
                                        control_index: tac_ci,
                                        name: f.name.clone(),
                                        cell_location,
                                    }),
                                    BoundingBox::new(x, form_y, tac_w, form_h),
                                );
                                line_node.children.push(form_node);
                            }
                        }
                        // tac 폭만큼 x 전진
                        x += tac_w;
                        sub_char_offset += 1;
                        seg_start = tac_rel;
                    }

                    // 마지막 tac 이후 텍스트 세그먼트 렌더링
                    let remaining: String = run_chars[seg_start..].iter().collect();
                    if !remaining.is_empty() {
                        let mut seg_style = text_style.clone();
                        seg_style.line_x_offset = x - col_area.x;
                        if has_tabs && remaining.contains('\t') {
                            let positions = compute_char_positions(&remaining, &seg_style);
                            seg_style.tab_leaders = extract_tab_leaders_with_extended(
                                &remaining,
                                &positions,
                                &seg_style,
                                &composed.tab_extended,
                            );
                        }
                        let seg_w = estimate_text_width(&remaining, &seg_style);
                        {
                            let sub_run_id = tree.next_id();
                            let sub_run_node = RenderNode::new(
                                sub_run_id,
                                RenderNodeType::TextRun(TextRunNode {
                                    text: remaining,
                                    style: seg_style,
                                    char_shape_id: Some(run.char_style_id),
                                    para_shape_id: Some(composed.para_style_id),
                                    section_index: Some(section_index),
                                    para_index: Some(para_index),
                                    char_start: Some(sub_char_offset),
                                    cell_context: cell_ctx.clone(),
                                    is_para_end: is_last_run,
                                    is_line_break_end: is_line_break,
                                    rotation: 0.0,
                                    is_vertical: false,
                                    char_overlap: run.char_overlap.clone(),
                                    border_fill_id: run_border_fill_id,
                                    baseline,
                                    field_marker: FieldMarkerType::None,
                                }),
                                BoundingBox::new(x, y, seg_w, line_height),
                            );
                            line_node.children.push(sub_run_node);
                        }
                        x += seg_w;
                    } else if is_last_run {
                        // 마지막 run이 tac로 끝나는 경우: 빈 TextRun으로 is_para_end 표시
                        let mut seg_style = text_style.clone();
                        seg_style.line_x_offset = x - col_area.x;
                        let sub_run_id = tree.next_id();
                        let sub_run_node = RenderNode::new(
                            sub_run_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: String::new(),
                                style: seg_style,
                                char_shape_id: Some(run.char_style_id),
                                para_shape_id: Some(composed.para_style_id),
                                section_index: Some(section_index),
                                para_index: Some(para_index),
                                char_start: Some(sub_char_offset),
                                cell_context: cell_ctx.clone(),
                                is_para_end: true,
                                is_line_break_end: is_line_break,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: 0,
                                baseline,
                                field_marker: FieldMarkerType::None,
                            }),
                            BoundingBox::new(x, y, 0.0, line_height),
                        );
                        line_node.children.push(sub_run_node);
                    }
                    // x는 이미 sub-run 루프에서 갱신됨 (x += full_width 생략)
                }

                char_offset += run_char_count;
                run_char_pos = run_char_end;
                inline_tab_cursor_render += run.text.chars().filter(|c| *c == '\t').count();
                char_x_map.push((char_offset, x));
            }

            // 조판부호: 텍스트 뒤에 위치한 미삽입 도형 마커 추가
            for (smi, (spos, stext)) in shape_markers.iter().enumerate() {
                if !shape_marker_inserted[smi] {
                    shape_marker_inserted[smi] = true;
                    let base_style = resolved_to_text_style(styles, 0, 0);
                    let mut ms = base_style;
                    ms.color = 0x0000FF;
                    ms.font_size *= 0.55;
                    let mw = estimate_text_width(stext, &ms);
                    let mid = tree.next_id();
                    let mn = RenderNode::new(
                        mid,
                        RenderNodeType::TextRun(TextRunNode {
                            text: stext.clone(),
                            style: ms,
                            char_shape_id: None,
                            para_shape_id: Some(composed.para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: None,
                            cell_context: cell_ctx.clone(),
                            is_para_end: false,
                            is_line_break_end: false,
                            rotation: 0.0,
                            is_vertical: false,
                            char_overlap: None,
                            border_fill_id: 0,
                            baseline,
                            field_marker: FieldMarkerType::ShapeMarker(*spos),
                        }),
                        BoundingBox::new(x, y, mw, line_height),
                    );
                    line_node.children.push(mn);
                    x += mw;
                }
            }

            // run 루프 종료 후, run 범위 밖(pos >= run_char_pos)의 미매칭 TAC 이미지 렌더링
            if !comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
                if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                    let line_start_char = comp_line.char_start;
                    let line_end_char = line_start_char
                        + comp_line
                            .runs
                            .iter()
                            .map(|r| r.text.chars().count())
                            .sum::<usize>();
                    for &(tac_pos, tac_w, tac_ci) in &tac_offsets_px {
                        if tac_pos <= run_char_pos || tac_pos > line_end_char {
                            continue; // run 범위 내/끝 또는 미래 줄 TAC: 여기서 처리하지 않음
                        }
                        if let Some(ctrl) = p.controls.get(tac_ci) {
                            if let Control::Picture(pic) = ctrl {
                                let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                if raw_lh + 4.0 >= pic_h {
                                    current_line_reserved_tac_picture_height = Some(pic_h);
                                }
                                let img_y = (y + baseline - pic_h).max(y);
                                let bin_data_id = pic.image_attr.bin_data_id;
                                let image_data =
                                    find_bin_data(bdc, bin_data_id).map(|c| c.data.clone());
                                let crop = {
                                    let c = &pic.crop;
                                    if c.right > c.left
                                        && c.bottom > c.top
                                        && (c.left != 0
                                            || c.top != 0
                                            || c.right != 0
                                            || c.bottom != 0)
                                    {
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
                                // [Task #1151 v7 항목 7] ImageNode 생성 helper 통합.
                                let img_node = make_picture_image_node(
                                    tree,
                                    pic,
                                    section_index,
                                    para_index,
                                    tac_ci,
                                    cell_ctx.as_ref(),
                                    crop,
                                    original_size_hu,
                                    bin_data_id,
                                    image_data,
                                    BoundingBox::new(x, img_y, tac_w, pic_h),
                                );
                                line_node.children.push(img_node);
                                x += tac_w;
                            }
                        }
                    }
                }
            }

            // 빈 문단(runs 없음)에서 tac 양식 개체 렌더링
            if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
                if let Some(p) = para {
                    for &(_tac_pos, tac_w, tac_ci) in &tac_offsets_px {
                        if let Some(Control::Form(f)) = p.controls.get(tac_ci) {
                            let form_h = hwpunit_to_px(f.height as i32, self.dpi);
                            let form_y = (y + baseline - form_h).max(y);
                            let cell_location = cell_ctx.as_ref().map(|ctx| {
                                let e = &ctx.path[0];
                                (
                                    ctx.parent_para_index,
                                    e.control_index,
                                    e.cell_index,
                                    e.cell_para_index,
                                )
                            });
                            let form_node = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::FormObject(FormObjectNode {
                                    form_type: f.form_type,
                                    caption: f.caption.clone(),
                                    text: f.text.clone(),
                                    fore_color: form_color_to_css(f.fore_color),
                                    back_color: form_color_to_css(f.back_color),
                                    value: f.value,
                                    enabled: f.enabled,
                                    section_index,
                                    para_index,
                                    control_index: tac_ci,
                                    name: f.name.clone(),
                                    cell_location,
                                }),
                                BoundingBox::new(x, form_y, tac_w, form_h),
                            );
                            line_node.children.push(form_node);
                            x += tac_w;
                        }
                    }
                }
            }

            let defer_empty_line_control_marker = comp_line.runs.is_empty()
                && !tac_offsets_px.is_empty()
                && equation_tac_line_flow.is_some();

            // runs가 비어있으면 빈 TextRun 생성 (빈 셀 편집용)
            if comp_line.runs.is_empty() {
                self.layout_empty_runs_line(
                    tree,
                    &mut line_node,
                    comp_line,
                    composed,
                    para,
                    bin_data_content,
                    styles,
                    &cell_ctx,
                    &line_tac_offsets,
                    EmptyRunsLineVars {
                        alignment,
                        available_width,
                        effective_col_x,
                        effective_margin_left,
                        x_start,
                        line_char_end: char_offset,
                        y,
                        baseline,
                        raw_lh,
                        runs_all_whitespace,
                        max_fs,
                        line_spacing_px,
                        has_topbottom_vpos_base: para_topbottom_line_vpos_base.is_some(),
                        is_last_line_of_para,
                        defer_empty_line_control_marker,
                        line_flow_height,
                        section_index,
                        para_index,
                    },
                    &mut current_line_reserved_tac_picture_height,
                );
            }

            // [Task #287] 빈 runs 줄의 TAC 수식 인라인 처리
            // 큰 디스플레이 수식이 자체 LINE_SEG 를 가질 때 comp_line.runs 가 비어있는데,
            // run 루프가 돌지 않아 수식이 인라인 경로로 렌더되지 않고 shape_layout display
            // 경로로 떨어져 col_area.y 에 고정되던 문제를 해결한다.
            if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
                let line_start_char = comp_line.char_start;
                let line_end_char = composed
                    .lines
                    .get(line_idx + 1)
                    .map(|l| l.char_start)
                    .unwrap_or(usize::MAX);
                let tac_on_line = |k: usize, pos: usize| -> bool {
                    if let Some(ref flow) = equation_tac_line_flow {
                        flow.row_for_tac(k).is_some()
                    } else {
                        pos >= line_start_char && pos < line_end_char
                    }
                };
                let tac_row_for = |k: usize| -> usize {
                    equation_tac_line_flow
                        .as_ref()
                        .and_then(|flow| flow.row_for_tac(k))
                        .unwrap_or(0)
                };
                // [Task #490] 셀에 텍스트 없이 수식만 있을 때는 셀 ParaShape alignment 를
                // 따라야 한다. 단, [Task #1245] 본문/미주 수식-only 줄은 저장된 LINE_SEG
                // 흐름을 따라야 하며 문단 alignment 를 다시 적용하면 열 안에서 중앙으로 밀린다.
                // [Task #489] effective_col_x 적용 (Picture+Square wrap LINE_SEG cs/sw 좁은 영역).
                let mut row_tac_widths = vec![0.0f64; equation_tac_extra_rows + 1];
                for (k, (pos, w, _)) in tac_offsets_px.iter().enumerate() {
                    if tac_on_line(k, *pos) {
                        let row = tac_row_for(k).min(row_tac_widths.len() - 1);
                        row_tac_widths[row] += *w;
                    }
                }
                let line_tac_width: f64 = row_tac_widths.iter().sum();
                let align_offset = if cell_ctx.is_some() {
                    match alignment {
                        Alignment::Center | Alignment::Distribute => {
                            (available_width - line_tac_width).max(0.0) / 2.0
                        }
                        Alignment::Right => (available_width - line_tac_width).max(0.0),
                        _ => 0.0,
                    }
                } else {
                    0.0
                };
                // Empty-run TAC-only lines still belong to the visual line flow.
                // Therefore paragraph margins and first-line/hanging indent must
                // use the same x origin as ordinary TextLine nodes.
                let row_base_x = |row: usize| -> f64 {
                    let visual_line_idx = equation_tac_line_flow
                        .as_ref()
                        .map(|flow| flow.visual_line_idx_for_row(row))
                        .unwrap_or(line_idx + row);
                    let row_effective_margin_left =
                        crate::renderer::equation_tac_flow::paragraph_effective_margin_left_with_indent_scale(
                            margin_left,
                            indent,
                            visual_line_idx,
                            // [Task #1472] 변환본은 effective indent 불변 위해 scale 절반.
                            (if equation_tac_line_flow.is_some() && cell_ctx.is_none() {
                                2.0
                            } else {
                                1.0
                            }) * if self.is_hwp3_variant.get() { 0.5 } else { 1.0 },
                        );
                    effective_col_x + row_effective_margin_left
                };
                let mut row_inline_x: Vec<f64> = (0..=equation_tac_extra_rows)
                    .map(|row| {
                        let row_width = row_tac_widths.get(row).copied().unwrap_or(0.0);
                        let row_align_offset = if cell_ctx.is_some() {
                            match alignment {
                                Alignment::Center | Alignment::Distribute => {
                                    (available_width - row_width).max(0.0) / 2.0
                                }
                                Alignment::Right => (available_width - row_width).max(0.0),
                                _ => 0.0,
                            }
                        } else {
                            align_offset
                        };
                        row_base_x(row) + row_align_offset
                    })
                    .collect();
                let zero_endnote_boundary_result_shift = if cell_ctx.is_none()
                    && self.current_endnote_zero_spacing_profile()
                    && para_index >= self.endnote_para_base.get()
                    && !self.endnote_para_has_same_endnote_successor(para_index)
                    && line_idx + 1 >= end
                    && equation_tac_extra_rows == 0
                    && line_tac_offsets.len() == 1
                    && comp_line.runs.is_empty()
                    && y + line_height > col_bottom - 20.0
                    && line_tac_offsets.iter().all(|(_, _, ci)| {
                        para.is_some_and(|p| {
                            matches!(
                                p.controls.get(*ci),
                                Some(Control::Equation(eq))
                                    if eq.common.treat_as_char && eq.common.height <= 1200
                            )
                        })
                    }) {
                    // 0/0/0 미주에서는 새 미주 제목이 바로 뒤따르는 작은 결과식 tail이
                    // 저장 LINE_SEG 하단에 놓이면 제목과 순서가 뒤집혀 보일 수 있다.
                    // 물리 흐름은 유지하고 마지막 작은 수식 표시만 한 줄 위 결과 위치로 붙인다.
                    ((line_height + line_spacing_px) * 2.0).clamp(24.0, 42.0)
                } else {
                    0.0
                };
                for (tac_k, &(tac_pos, tac_w, tac_ci)) in tac_offsets_px.iter().enumerate() {
                    if !tac_on_line(tac_k, tac_pos) {
                        continue;
                    }
                    if let Some(p) = para {
                        if let Some(Control::Equation(eq)) = p.controls.get(tac_ci) {
                            let tokens = crate::renderer::equation::tokenizer::tokenize(&eq.script);
                            let ast =
                                crate::renderer::equation::parser::EqParser::new(tokens).parse();
                            let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                            let layout_box =
                                crate::renderer::equation::layout::EqLayout::new(font_size_px)
                                    .layout(&ast);
                            let color_str =
                                crate::renderer::equation::svg_render::eq_color_to_svg(eq.color);
                            let svg_content =
                                crate::renderer::equation::svg_render::render_equation_svg(
                                    &layout_box,
                                    &color_str,
                                    font_size_px,
                                );
                            let hwp_eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                            let eq_h = if hwp_eq_h > 0.0 {
                                hwp_eq_h
                            } else {
                                layout_box.height
                            };
                            let tac_row = tac_row_for(tac_k).min(row_inline_x.len() - 1);
                            let row_y = (y + tac_row as f64 * (line_height + line_spacing_px)
                                - zero_endnote_boundary_result_shift)
                                .max(col_area.y);
                            let inline_x = row_inline_x[tac_row];
                            let eq_y = if cell_ctx.is_some() {
                                (row_y + baseline - layout_box.baseline).max(row_y)
                            } else {
                                row_y + baseline - layout_box.baseline
                            };
                            let (eq_cell_idx, eq_cell_para_idx) = if let Some(ref ctx) = cell_ctx {
                                (
                                    Some(ctx.path[0].cell_index),
                                    Some(ctx.path[0].cell_para_index),
                                )
                            } else {
                                (None, None)
                            };
                            let note_ref = if cell_ctx.is_none() {
                                self.note_ref_for_endnote_equation(para_index, tac_ci)
                            } else {
                                None
                            };
                            let eq_node = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Equation(
                                    crate::renderer::render_tree::EquationNode {
                                        svg_content,
                                        layout_box,
                                        color_str,
                                        color: eq.color,
                                        font_size: font_size_px,
                                        section_index: note_ref
                                            .as_ref()
                                            .map(|r| r.section_index)
                                            .or(Some(section_index)),
                                        para_index: if let Some(ref ctx) = cell_ctx {
                                            Some(ctx.parent_para_index)
                                        } else {
                                            Some(para_index)
                                        },
                                        control_index: if let Some(ref ctx) = cell_ctx {
                                            Some(ctx.path[0].control_index)
                                        } else {
                                            Some(tac_ci)
                                        },
                                        cell_index: eq_cell_idx,
                                        cell_para_index: eq_cell_para_idx,
                                        note_ref,
                                    },
                                ),
                                BoundingBox::new(inline_x, eq_y, tac_w, eq_h),
                            );
                            line_node.children.push(eq_node);
                            tree.set_inline_shape_position(
                                section_index,
                                para_index,
                                tac_ci,
                                cell_ctx.as_ref(),
                                inline_x,
                                eq_y,
                            );
                            row_inline_x[tac_row] += tac_w;
                        }
                    }
                }

                if defer_empty_line_control_marker
                    && (is_last_line_of_para || comp_line.has_line_break)
                    && !row_inline_x.is_empty()
                {
                    let marker_row = row_tac_widths
                        .iter()
                        .enumerate()
                        .rev()
                        .find_map(|(row, width)| if *width > 0.0 { Some(row) } else { None })
                        .unwrap_or(0)
                        .min(row_inline_x.len() - 1);
                    let marker_x = row_inline_x[marker_row];
                    let marker_y = y + marker_row as f64 * (line_height + line_spacing_px);
                    let marker_id = tree.next_id();
                    let marker_style = paragraph_active_text_style(styles, para, char_offset).0;
                    let marker_node = RenderNode::new(
                        marker_id,
                        RenderNodeType::TextRun(TextRunNode {
                            text: String::new(),
                            style: marker_style,
                            char_shape_id: None,
                            para_shape_id: Some(composed.para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: None,
                            cell_context: cell_ctx.clone(),
                            is_para_end: is_last_line_of_para,
                            is_line_break_end: comp_line.has_line_break,
                            rotation: 0.0,
                            is_vertical: false,
                            char_overlap: None,
                            border_fill_id: 0,
                            baseline,
                            field_marker: FieldMarkerType::None,
                        }),
                        BoundingBox::new(marker_x, marker_y, 0.0, line_height),
                    );
                    line_node.children.push(marker_node);
                }
            }

            // ClickHere 필드 처리: 안내문 + 조판부호 마커 — #1925 추출
            if let Some(p) = para {
                x += self.layout_click_here_and_bookmark_markers(
                    tree,
                    &mut line_node,
                    p,
                    comp_line,
                    &char_x_map,
                    styles,
                    composed.para_style_id,
                    section_index,
                    para_index,
                    &cell_ctx,
                    char_offset,
                    x,
                    y,
                    line_height,
                    baseline,
                );
            }

            // 강제 줄바꿈(\n)이 이 줄에서 제거되었으므로 char_offset에 1을 더하여
            // 다음 줄의 TextRun.char_start가 올바른 문서 좌표를 가리키도록 한다.
            if comp_line.has_line_break {
                char_offset += 1;
            }

            let following_text_xs: Vec<f64> = line_node
                .children
                .iter()
                .filter_map(|child| {
                    if let RenderNodeType::TextRun(tr) = &child.node_type {
                        if !tr.text.trim().is_empty() {
                            return Some(child.bbox.x);
                        }
                    }
                    None
                })
                .collect();
            for child in &mut line_node.children {
                if let RenderNodeType::TextRun(tr) = &mut child.node_type {
                    if tr.style.tab_leaders.is_empty() {
                        continue;
                    }
                    let space_gap = if tr.style.font_size > 0.0 {
                        tr.style.font_size * 0.25
                    } else {
                        3.0
                    };
                    for leader in &mut tr.style.tab_leaders {
                        let abs_start = child.bbox.x + leader.start_x;
                        if let Some(next_x) = following_text_xs
                            .iter()
                            .copied()
                            .filter(|x| *x > abs_start + 0.5)
                            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        {
                            let new_end_x = (next_x - child.bbox.x - space_gap).max(leader.start_x);
                            if new_end_x < leader.end_x {
                                leader.end_x = new_end_x;
                            }
                        }
                    }
                }
            }

            col_node.children.push(line_node);
            // 줄간격 적용:
            //   - 셀 내 마지막 문단의 마지막 줄: trailing line_spacing 제외
            //     (셀 높이 모델은 trailing 미포함, 셀 내부와 정합)
            //   - 그 외 모든 줄(본문 단락의 마지막 줄 포함): trailing line_spacing 가산
            //     pagination/engine.rs 의 current_height 누적(para_height = sum(lh+ls))
            //     과 정합. (Task #452: 이전 #332 의 layout-only trailing 제외 →
            //     pagination 과 1 ls drift 발생 → 회복)
            let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
            // [Task #901 Stage 5/6] wrap zone paragraph 의 empty-runs / whitespace-only
            // line 은 y advance 건너뜀.
            // pic2.hwp paragraph 0 case: 8 line_segs (4 visible "우/리/나/라" + 4 empty
            // phantom lines for wrap zone 의 다른 column). 추가로 첫 idx=0 은 cs=24470
            // (LEFT narrow wrap zone) 의 공백 한 글자만 가짐 — 한컴 viewer 가 wrap zone
            // 좌측 영역에 텍스트 미배치한 결과. has_picture_shape_square_wrap 게이트로
            // wrap zone 호스트 paragraph 만 영향.
            if !runs_all_whitespace
                && !text_before_picture_line
                && current_line_reserved_tac_picture_height.is_none()
            {
                current_line_reserved_tac_picture_height =
                    tac_picture_or_shape_height_for_line(para, raw_lh, self.dpi);
                if current_line_reserved_tac_picture_height.is_none()
                    && has_treat_as_char_picture_or_shape(para)
                    && max_fs > 0.0
                    && raw_lh > max_fs * 2.0
                {
                    current_line_reserved_tac_picture_height = Some(raw_lh);
                }
            }
            let tac_picture_label_extra = tac_picture_label_extra_for_line(
                cell_ctx.as_ref(),
                runs_all_whitespace,
                raw_lh,
                current_line_reserved_tac_picture_height,
                max_fs,
                line_spacing_px,
            );
            // Square wrap host 의 빈 guide 줄은 advance 를 건너뛰지만, 같은 줄에
            // TAC 수식/개체가 있으면 실제 콘텐츠 줄이므로 높이를 보존한다.
            let skip_advance_empty_wrap = has_picture_shape_square_wrap
                && runs_all_whitespace
                && !line_has_tac_control(composed, line_idx);
            // 촘촘한 미주 수식 문단에는 다음 줄과 char_start가 같은 선행
            // 퇴화 LINE_SEG가 들어오는 경우가 있다. 해당 줄 자체에는 TAC가
            // 없으며, 한컴은 첫 수식 앞에 이 안내 줄 높이를 예약하지 않는다.
            let skip_advance_empty_tac_lead = cell_ctx.is_none()
                && !tac_offsets_px.is_empty()
                && line_is_leading_empty_equation_tac_guide(
                    para,
                    composed,
                    &tac_offsets_px,
                    line_idx,
                );
            let skip_advance_empty_tac_picture = runs_all_whitespace
                && current_line_reserved_tac_picture_height.is_none()
                && prev_line_reserved_tac_picture_height
                    .map(|pic_h| (raw_lh - pic_h).abs() <= 4.0)
                    .unwrap_or(false);
            let skip_advance_empty_line = skip_advance_empty_wrap
                || skip_advance_empty_tac_picture
                || skip_advance_empty_tac_lead;
            // RHWP_DEBUG_PARA_TAC="95,96,97" — 대상 pi 를 콤마 목록으로 지정 (빈 값/all=전체).
            if std::env::var("RHWP_DEBUG_PARA_TAC").is_ok_and(|v| {
                v.is_empty()
                    || v == "all"
                    || v.split(',').any(|t| t.trim().parse() == Ok(para_index))
            }) {
                eprintln!(
                    "  TAC_ADV pi={} line_idx={} y={:.1} raw_lh={:.1} lh={:.1} ls={:.1} label_extra={:.1} whitespace={} cur_pic={:?} prev_pic={:?} skip_wrap={} skip_pic={} skip={}",
                    para_index,
                    line_idx,
                    y,
                    raw_lh,
                    line_height,
                    line_spacing_px,
                    tac_picture_label_extra,
                    runs_all_whitespace,
                    current_line_reserved_tac_picture_height,
                    prev_line_reserved_tac_picture_height,
                    skip_advance_empty_wrap,
                    skip_advance_empty_tac_picture,
                    skip_advance_empty_line,
                );
            }
            // [Task #1046 Stage 3 Class D] 본문 문단(셀 밖)의 콘텐츠 하단(=현재 줄 텍스트
            // 바닥, trailing 줄간격/spacing_after 제외) 기록. overflow 검출이 페이지 바닥
            // 후행 줄간격을 콘텐츠 초과로 오판하지 않도록 한다(페이지네이터의 마지막 줄
            // trailing_ls 허용 #359/#404 와 정합). 매 줄 갱신 → 마지막 렌더 줄 값이 남는다.
            if cell_ctx.is_none() && !skip_advance_empty_line {
                let content_bottom = if blank_spacer_line {
                    y
                } else {
                    y + render_line_flow_height
                };
                self.last_item_content_bottom.set(content_bottom);
                if equation_only_endnote_tail_line && content_bottom > col_bottom {
                    self.last_item_endnote_equation_tail_line_box.set(true);
                }
            }
            if endnote_line_vpos_base.is_some() {
                let line_bottom = if skip_advance_empty_line {
                    y
                } else {
                    // [Task #1236] 다줄 미주 문단의 마지막 줄: 다음 문단이 **같은 미주**
                    // 연속이면 trailing 줄간격을 포함해 풀이 줄간격을 균일하게 한다
                    // (간헐적 좁아짐 해소). 미주 마지막 문단(=문제 경계)이면 0 유지해
                    // between-notes margin 과 중복 가산되지 않게 한다.
                    let trailing = if line_idx + 1 < end
                        || self.endnote_para_has_same_endnote_successor(para_index)
                    {
                        render_line_spacing_px
                    } else {
                        0.0
                    };
                    y + render_line_flow_height + trailing + tac_picture_label_extra
                };
                let next_y = endnote_line_vpos_y_end
                    .map(|prev| prev.max(line_bottom))
                    .unwrap_or(line_bottom);
                endnote_line_vpos_y_end = Some(next_y);
                if equation_tac_extra_rows > 0 || endnote_used_auto_wrap_y {
                    endnote_auto_wrap_y_end = Some(line_bottom);
                }
                y = next_y;
            } else if is_cell_last_line && cell_ctx.is_some() {
                y += line_flow_height;
            } else if skip_advance_empty_line {
                // no advance
            } else {
                y += render_line_flow_height + render_line_spacing_px + tac_picture_label_extra;
            }
            prev_line_reserved_tac_picture_height = current_line_reserved_tac_picture_height;
        }

        // 문단 테두리/배경 범위 수집 (build_single_column에서 연속 그룹으로 병합 렌더링)
        // margin_left/margin_right를 반영하여 박스 위치·폭 조정.
        // Task #463: 셀 안 단락은 본문 큐에 leakage 하지 않도록 cell_ctx 게이팅.
        // 셀 외곽선은 별도 경로(table_layout/border_rendering)에서 처리되므로
        // 본문 단락의 연속 외곽선 merge 가 셀 단락 좌표/시그니처에 의해 깨지지 않게 한다.
        if para_border_fill_id > 0 && cell_ctx.is_none() {
            let bg_height = y - bg_y_start;
            if bg_height > 0.0 {
                // margin_left/margin_right는 이미 px 단위 (style_resolver에서 변환됨)
                // border_spacing[2]/[3] (top/bottom) 을 inset 으로 전달 — 병합 그룹의 첫/마지막 range 에서만 적용됨.
                let top_inset = para_style.map(|s| s.border_spacing[2]).unwrap_or(0.0);
                let bottom_inset = para_style.map(|s| s.border_spacing[3]).unwrap_or(0.0);
                // 컬럼/페이지 wrap 시 inner edge 미렌더링용 partial 플래그
                let is_partial_start = start_line > 0;
                let is_partial_end = end < composed.lines.len();
                // Task #463: wrap=Square 호스트 문단의 텍스트는 좁은 wrap_area 에서
                // 렌더링되지만 외곽선은 원래 col_area 너비로 그려야 floating 표를
                // 박스가 둘러쌈. layout_wrap_around_paras 가 override 를 설정.
                // override 가 활성된 경우(wrap host), 박스 우측은 floating 표의 끝
                // 까지 확장된 width 그대로 사용 — margin_right 차감하지 않는다
                // (그렇지 않으면 표가 박스 밖으로 다시 튀어나옴).
                // [Task #544] paragraph margin_left/right 는 텍스트 inset 으로만 사용,
                // 박스 outline 좌표는 col_area 전체 (PDF 정합). wrap=Square 호스트
                // (border_box_override) 케이스는 layout_wrap_around_paras 가 설정한
                // override 좌표 그대로 사용 (margin 미적용).
                let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
                    (ox, ow)
                } else {
                    (col_area.x, col_area.width)
                };
                self.para_border_ranges.borrow_mut().push((
                    para_border_fill_id,
                    box_x,
                    bg_y_start,
                    box_w,
                    y,
                    top_inset,
                    bottom_inset,
                    is_partial_start,
                    is_partial_end,
                    para_index,
                ));
            }
        }

        // 문단 뒤 간격 (spacing_after)
        if spacing_after > 0.0 && end == composed.lines.len() {
            y += spacing_after;
        }

        // ComposedLine이 없으면 기본 높이 + 빈 TextRun 생성 (편집용)
        if composed.lines.is_empty() && start_line == 0 {
            let default_height = hwpunit_to_px(400, self.dpi);
            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::with_para(
                    default_height,
                    default_height * 0.8,
                    section_index,
                    para_index,
                )),
                BoundingBox::new(col_area.x, y, col_area.width, default_height),
            );

            // 빈 문단에도 TextRun 노드를 생성하여 캐럿 위치 제공
            let run_id = tree.next_id();
            let (text_style, char_shape_id) =
                paragraph_active_text_style(styles, para, char_offset);
            let run_node = RenderNode::new(
                run_id,
                RenderNodeType::TextRun(TextRunNode {
                    text: String::new(),
                    style: text_style,
                    char_shape_id,
                    para_shape_id: Some(composed.para_style_id),
                    section_index: Some(section_index),
                    para_index: Some(para_index),
                    char_start: Some(char_offset),
                    cell_context: cell_ctx.clone(),
                    is_para_end: true,
                    is_line_break_end: false,
                    rotation: 0.0,
                    is_vertical: false,
                    char_overlap: None,
                    border_fill_id: 0,
                    baseline: default_height * 0.85,
                    field_marker: FieldMarkerType::None,
                }),
                BoundingBox::new(col_area.x, y, col_area.width, default_height),
            );
            line_node.children.push(run_node);

            col_node.children.push(line_node);
            y += default_height;
        }

        y
    }

    /// [#1925 추출] ClickHere 필드 처리(안내문, [누름틀 시작/끝] 조판부호 마커)와
    /// 책갈피 조판부호 마커. char_x_map 보간으로 필드 위치의 x 좌표를 계산해
    /// 마커 노드를 삽입하고, 마커 폭만큼 기존 노드를 오른쪽으로 shift 한다.
    /// 반환값 accumulated_shift 는 caller 가 라인 커서 x 에 가산한다.
    #[allow(clippy::too_many_arguments)]
    fn layout_click_here_and_bookmark_markers(
        &self,
        tree: &mut PageRenderTree,
        line_node: &mut RenderNode,
        p: &Paragraph,
        comp_line: &crate::renderer::composer::ComposedLine,
        char_x_map: &[(usize, f64)],
        styles: &ResolvedStyleSet,
        para_style_id: u16,
        section_index: usize,
        para_index: usize,
        cell_ctx: &Option<CellContext>,
        line_char_end: usize,
        x: f64,
        y: f64,
        line_height: f64,
        baseline: f64,
    ) -> f64 {
        // line_char_end: 파라미터로 수령 (원본: char_offset)
        let line_char_start = comp_line.char_start;
        let active = self.active_field.borrow();
        let ctrl_codes = self.show_control_codes.get();

        // char_x_map에서 특정 char_idx에 해당하는 x 좌표를 보간 계산
        let find_x_for_char = |target: usize| -> f64 {
            for i in 0..char_x_map.len().saturating_sub(1) {
                let (c0, x0) = char_x_map[i];
                let (c1, x1) = char_x_map[i + 1];
                if target >= c0 && target <= c1 {
                    if c1 == c0 {
                        return x0;
                    }
                    let ratio = (target - c0) as f64 / (c1 - c0) as f64;
                    return x0 + ratio * (x1 - x0);
                }
            }
            char_x_map.last().map(|&(_, xv)| xv).unwrap_or(x)
        };

        // 마커 삽입 정보 수집 (오른쪽→왼쪽 순으로 shift 처리)
        struct MarkerInsert {
            marker_x: f64,
            marker_w: f64,
            node: RenderNode,
        }
        let mut markers: Vec<MarkerInsert> = Vec::new();

        for fr in &p.field_ranges {
            if let Some(Control::Field(field)) = p.controls.get(fr.control_idx) {
                if field.field_type != crate::model::control::FieldType::ClickHere {
                    continue;
                }
                let is_empty = fr.start_char_idx == fr.end_char_idx;
                let start_in_line =
                    fr.start_char_idx >= line_char_start && fr.start_char_idx <= line_char_end;
                let end_in_line =
                    fr.end_char_idx >= line_char_start && fr.end_char_idx <= line_char_end;

                if !start_in_line && !end_in_line {
                    continue;
                }

                let is_active = if let Some((af_sec, af_para, af_ctrl, ref af_cell)) = *active {
                    if af_sec != section_index || af_para != para_index || af_ctrl != fr.control_idx
                    {
                        false
                    } else {
                        // cell_path 전체 일치 확인
                        match (af_cell, cell_ctx) {
                            (None, None) => true,
                            (Some(af_path), Some(ctx)) => {
                                // af_path와 ctx.path의 (control_index, cell_index) 쌍이 모두 일치해야 함
                                af_path.len() == ctx.path.len()
                                    && af_path.iter().zip(ctx.path.iter()).all(
                                        |(&(ac, ax, _ap), entry)| {
                                            ac == entry.control_index && ax == entry.cell_index
                                        },
                                    )
                            }
                            _ => false,
                        }
                    }
                } else {
                    false
                };

                let base_run = comp_line.runs.last().or(comp_line.runs.first());
                let base_style = if let Some(run) = base_run {
                    resolved_to_text_style(styles, run.char_style_id, run.lang_index)
                } else {
                    resolved_to_text_style(styles, 0, 0)
                };

                // [누름틀 시작] 마커 — fr.start_char_idx 위치에 삽입
                if ctrl_codes && start_in_line {
                    let mut marker_style = base_style.clone();
                    marker_style.color = 0x0066CC; // BGR: 주황색 (#CC6600)
                    marker_style.font_size *= 0.55;
                    let marker_text = "[누름틀 시작]";
                    let marker_w = estimate_text_width(marker_text, &marker_style);
                    let marker_x = find_x_for_char(fr.start_char_idx);
                    let m_id = tree.next_id();
                    let m_node = RenderNode::new(
                        m_id,
                        RenderNodeType::TextRun(TextRunNode {
                            text: marker_text.to_string(),
                            style: marker_style,
                            char_shape_id: None,
                            para_shape_id: Some(para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: None,
                            cell_context: cell_ctx.clone(),
                            is_para_end: false,
                            is_line_break_end: false,
                            rotation: 0.0,
                            is_vertical: false,
                            char_overlap: None,
                            border_fill_id: 0,
                            baseline,
                            field_marker: FieldMarkerType::FieldBegin,
                        }),
                        BoundingBox::new(marker_x, y, marker_w, line_height),
                    );
                    markers.push(MarkerInsert {
                        marker_x,
                        marker_w,
                        node: m_node,
                    });
                }

                // 빈 필드 커서 앵커: getCursorRect가 필드 시작 위치를 찾을 수 있도록
                // char_start를 설정한 zero-width 노드 삽입
                if is_empty && start_in_line {
                    let anchor_x = find_x_for_char(fr.start_char_idx);
                    let anchor_id = tree.next_id();
                    let anchor_node = RenderNode::new(
                        anchor_id,
                        RenderNodeType::TextRun(TextRunNode {
                            text: String::new(),
                            style: base_style.clone(),
                            char_shape_id: None,
                            para_shape_id: Some(para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: Some(fr.start_char_idx),
                            cell_context: cell_ctx.clone(),
                            is_para_end: false,
                            is_line_break_end: false,
                            rotation: 0.0,
                            is_vertical: false,
                            char_overlap: None,
                            border_fill_id: 0,
                            baseline,
                            field_marker: FieldMarkerType::None,
                        }),
                        BoundingBox::new(anchor_x, y, 0.0, line_height),
                    );
                    markers.push(MarkerInsert {
                        marker_x: anchor_x,
                        marker_w: 0.0,
                        node: anchor_node,
                    });
                }

                // 빈 필드 안내문 (활성 필드가 아닐 때만)
                if is_empty && !is_active && start_in_line {
                    if let Some(guide) = field.guide_text() {
                        let mut guide_style = base_style.clone();
                        guide_style.color = 0x0000FF; // BGR: 빨간색
                        guide_style.italic = true;
                        let guide_width = estimate_text_width(guide, &guide_style);
                        // 안내문은 [누름틀 시작] 마커 뒤에 위치
                        let guide_x = find_x_for_char(fr.start_char_idx);
                        let guide_id = tree.next_id();
                        let guide_node = RenderNode::new(
                            guide_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: guide.to_string(),
                                style: guide_style,
                                char_shape_id: None,
                                para_shape_id: Some(para_style_id),
                                section_index: Some(section_index),
                                para_index: Some(para_index),
                                char_start: None,
                                cell_context: cell_ctx.clone(),
                                is_para_end: false,
                                is_line_break_end: false,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: 0,
                                baseline,
                                field_marker: FieldMarkerType::None,
                            }),
                            BoundingBox::new(guide_x, y, guide_width, line_height),
                        );
                        markers.push(MarkerInsert {
                            marker_x: guide_x,
                            marker_w: guide_width,
                            node: guide_node,
                        });
                    }
                }

                // [누름틀 끝] 마커 — fr.end_char_idx 위치에 삽입
                if ctrl_codes && end_in_line {
                    let mut marker_style = base_style.clone();
                    marker_style.color = 0x0066CC; // BGR: 주황색
                    marker_style.font_size *= 0.55;
                    let marker_text = "[누름틀 끝]";
                    let marker_w = estimate_text_width(marker_text, &marker_style);
                    let marker_x = find_x_for_char(fr.end_char_idx);
                    let m_id = tree.next_id();
                    let m_node = RenderNode::new(
                        m_id,
                        RenderNodeType::TextRun(TextRunNode {
                            text: marker_text.to_string(),
                            style: marker_style,
                            char_shape_id: None,
                            para_shape_id: Some(para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: None,
                            cell_context: cell_ctx.clone(),
                            is_para_end: false,
                            is_line_break_end: false,
                            rotation: 0.0,
                            is_vertical: false,
                            char_overlap: None,
                            border_fill_id: 0,
                            baseline,
                            field_marker: FieldMarkerType::FieldEnd,
                        }),
                        BoundingBox::new(marker_x, y, marker_w, line_height),
                    );
                    markers.push(MarkerInsert {
                        marker_x,
                        marker_w,
                        node: m_node,
                    });
                }
            }
        }

        // 책갈피 조판부호 마커
        if ctrl_codes {
            let ctrl_positions = crate::document_core::helpers::find_logical_control_positions(p);
            for (ci, ctrl) in p.controls.iter().enumerate() {
                if let Control::Bookmark(_bm) = ctrl {
                    let char_pos = ctrl_positions.get(ci).copied().unwrap_or(0);
                    if char_pos >= line_char_start && char_pos <= line_char_end {
                        let base_run = comp_line.runs.last().or(comp_line.runs.first());
                        let bm_base_style = if let Some(run) = base_run {
                            resolved_to_text_style(styles, run.char_style_id, run.lang_index)
                        } else {
                            resolved_to_text_style(styles, 0, 0)
                        };
                        let mut marker_style = bm_base_style;
                        marker_style.color = 0x0000FF; // BGR: 빨간색 (#FF0000)
                        marker_style.font_size *= 0.55;
                        let marker_text = "[책갈피]".to_string();
                        let marker_w = estimate_text_width(&marker_text, &marker_style);
                        let marker_x = find_x_for_char(char_pos);
                        let m_id = tree.next_id();
                        let m_node = RenderNode::new(
                            m_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: marker_text,
                                style: marker_style,
                                char_shape_id: None,
                                para_shape_id: Some(para_style_id),
                                section_index: Some(section_index),
                                para_index: Some(para_index),
                                char_start: None,
                                cell_context: cell_ctx.clone(),
                                is_para_end: false,
                                is_line_break_end: false,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: 0,
                                baseline,
                                field_marker: FieldMarkerType::None,
                            }),
                            BoundingBox::new(marker_x, y, marker_w, line_height),
                        );
                        markers.push(MarkerInsert {
                            marker_x,
                            marker_w,
                            node: m_node,
                        });
                    }
                }
            }
        }

        // 도형 조판부호 마커는 텍스트 런 루프 내에서 직접 처리됨 (MarkerInsert 불사용)

        // 마커를 왼쪽부터 삽입하면서, 각 마커 뒤의 기존 노드와 이후 마커를 오른쪽으로 shift
        // zero-width 앵커(커서 위치용)는 shift하지 않고 원래 위치 유지
        markers.sort_by(|a, b| {
            a.marker_x
                .partial_cmp(&b.marker_x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut accumulated_shift = 0.0_f64;
        for mi in 0..markers.len() {
            let mw = markers[mi].marker_w;
            if mw == 0.0 {
                // zero-width 앵커: shift 없이 원래 위치 유지
                continue;
            }
            let shift_x = markers[mi].marker_x + accumulated_shift;
            // 기존 children 중 이 마커 위치 이후의 노드를 오른쪽으로 shift
            for child in line_node.children.iter_mut() {
                if child.bbox.x >= shift_x {
                    child.bbox.x += mw;
                }
            }
            // 이미 삽입된 마커도 shift (이전 마커 중 이 위치 이후에 있는 것)
            // → accumulated_shift로 처리됨
            markers[mi].node.bbox.x = shift_x;
            accumulated_shift += mw;
        }
        // 모든 마커 노드를 children에 추가
        for mi in markers {
            line_node.children.push(mi.node);
        }
        accumulated_shift
    }

    /// [#1925 추출] 정렬/탭 계산용 est 사전 폭 추정 run 패스.
    /// 렌더 노드를 만들지 않고 run 폭·탭 진행만 시뮬레이션해, 줄의 점유 폭
    /// (`est_x`)과 추정에 포함된 tac 개체 폭 합(`included_tac_width`)을 구한다.
    #[allow(clippy::too_many_arguments)]
    fn estimate_line_run_widths(
        &self,
        comp_line: &crate::renderer::composer::ComposedLine,
        composed: &ComposedParagraph,
        para: Option<&Paragraph>,
        styles: &ResolvedStyleSet,
        tab_stops: &[TabStop],
        tab_width: f64,
        auto_tab_right: bool,
        line_tac_offsets_for_width: &[(usize, f64, usize)],
        effective_margin_left: f64,
        available_width: f64,
        start_line: usize,
        line_idx: usize,
        est_x_init: f64,
    ) -> LineWidthEst {
        let mut est_x = est_x_init;
        let mut pending_right_tab_est: Option<(f64, u8, u8)> = None;
        let mut pending_right_leader_digit_est = false;
        let mut run_char_pos_est = comp_line.char_start;
        let mut included_tac_width_in_est = 0.0f64;
        // cross-run 탭 감지용 inline_tabs(composed.tab_extended) 커서 — Task #290
        let mut inline_tab_cursor_est: usize = 0;
        for (run_idx_est, run) in comp_line.runs.iter().enumerate() {
            let run_char_count_est = if run.char_overlap.is_some() {
                let chars: Vec<char> = run.text.chars().collect();
                crate::renderer::composer::char_overlap_advance_units(&chars)
            } else {
                run.text.chars().count()
            };
            let run_char_end_est = run_char_pos_est + run_char_count_est;
            let mut ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
            ts.default_tab_width = tab_width;
            ts.tab_stops = tab_stops.to_vec();
            ts.auto_tab_right = auto_tab_right;
            ts.available_width = available_width;
            ts.text_start_offset = effective_margin_left;
            ts.inline_tabs = composed.tab_extended.clone();
            if pending_right_leader_digit_est {
                if run.text.trim().is_empty() {
                    pending_right_leader_digit_est = true;
                } else {
                    if run.text.trim().chars().all(|ch| ch.is_ascii_digit()) {
                        if let Some(tab) = tab_stops
                            .iter()
                            .rev()
                            .find(|tab| tab.tab_type == 1 && tab.fill_type != 0)
                        {
                            let digit_w = estimate_text_width(run.text.trim(), &ts);
                            let target =
                                if composed.tab_extended.is_empty() && available_width > 0.0 {
                                    effective_margin_left + available_width
                                } else {
                                    tab.position
                                };
                            let gap = if composed.tab_extended.is_empty() {
                                0.0
                            } else {
                                ts.font_size * 0.25
                            };
                            est_x = target - gap - digit_w;
                        }
                    }
                    pending_right_leader_digit_est = false;
                }
            }
            // 교차 run 오른쪽/가운데 탭: 이 run의 시작 위치를 역방향으로 조정
            if let Some((tab_pos, tab_type, fill_type)) = pending_right_tab_est.take() {
                // [Task #279] 공백만 있는 run 은 right/center tab 정렬 단위가 아니다.
                // (장제목 케이스: " " 단독 run → carry-over)
                if (tab_type == 1 || tab_type == 2) && run.text.trim().is_empty() {
                    pending_right_tab_est = Some((tab_pos, tab_type, fill_type));
                } else {
                    ts.line_x_offset = est_x;
                    // [Task #279] 리더(fill_type ≠ 0) 가 있는 RIGHT 탭은 "이 줄 우측 끝까지" 의미.
                    // 셀 안 문단에서는 col_area 가 이미 cell padding 적용된 inner_area 이므로
                    // `effective_margin_left + available_width` 가 inner 우측 끝.
                    // [Task #874] auto_tab_right 의 tab_pos = available_width (text-start
                    // 상대). RIGHT 탭은 모두 col-start 좌표계로 변환 시 effective_margin_left
                    // 더해야 함. 종전엔 fill_type ≠ 0 만 변환되어 leader 없는 auto_right tab
                    // (shortcut.hwp 인쇄/개체 모양 복사 등) 가 ~27 px 왼쪽으로 밀려 렌더됨.
                    let effective_pos = if tab_type == 1 {
                        effective_margin_left
                            + (if fill_type != 0 {
                                available_width
                            } else {
                                tab_pos
                            })
                    } else {
                        tab_pos
                    };
                    // [Issue #842 #4] 탭 다음 콘텐츠가 여러 composed run 으로 쪼개진 경우
                    // (스크립트·char-shape 경계) 전체 블록 폭 기준으로 정렬해야 마지막 글자가
                    // 탭스톱에 맞는다. (선행 공백 run "예 16" 케이스도 합산에 포함되어 동작 유지.)
                    let run_w = right_tab_block_width(
                        &comp_line.runs,
                        run_idx_est,
                        styles,
                        tab_width,
                        &tab_stops,
                        auto_tab_right,
                        available_width,
                    );
                    match tab_type {
                        1 => {
                            est_x = effective_pos - run_w;
                        }
                        2 => {
                            est_x = effective_pos - run_w / 2.0;
                        }
                        _ => {}
                    }
                }
            }
            // 글자겹침 run: PUA 다자리 숫자는 1글자 폭, 그 외는 font_size * char_count
            if run.char_overlap.is_some() {
                let fs = if ts.font_size > 0.0 {
                    ts.font_size
                } else {
                    12.0
                };
                let chars: Vec<char> = run.text.chars().collect();
                let w = fs * crate::renderer::composer::char_overlap_advance_units(&chars) as f64;
                est_x += w;
                run_char_pos_est = run_char_end_est;
                inline_tab_cursor_est += run.text.chars().filter(|c| *c == '\t').count();
                continue;
            }
            // treat_as_char 분기점 처리: run 내 tac 위치에서 이미지 폭 삽입
            // 마지막 run에서는 run_char_end 위치의 TAC도 포함
            //
            // [Task #1219] TAC 소스를 줄-경계 정규 집합 `line_tac_offsets`
            // (= tac_offsets_for_line, 렌더 경로와 동일한 `pos < 다음 줄 시작`
            // 엄격 미만 규칙)로 통일한다. 전역 tac_offsets_px 를 run 경계로
            // 재필터링하면 줄 끝 위치(== 다음 줄 선두)의 수식이 현재 줄 폭에
            // 오포함되어(문26 라인0 에 다음 줄 `a₁=b₁=1` 55px) 거짓 오버플로우
            // → 본문 한글 압축이 발생했다. line_tac_offsets 는 이미 줄-범위로
            // 필터링되어 있으므로 run 범위 필터만 적용한다.
            //
            // [Task #1285] 단, 오른쪽 정렬된 셀 안에서 `TAC 표 + 공백 + TAC 표`가
            // 같은 마지막 줄에 놓이는 경우 두 번째 TAC 표는 run 끝 위치(pos == end)에
            // 기록된다. 일반 줄 경계 판정에는 포함하지 않고, 위에서 좁게 만든
            // line_tac_offsets_for_width 에만 넣어 부모 줄 오른쪽 정렬 폭을 맞춘다.
            let run_chars_est: Vec<char> = run.text.chars().collect();
            let mut seg_start_est = 0usize;
            let is_last_run_est_tac = run_char_end_est
                >= comp_line
                    .runs
                    .iter()
                    .map(|r| r.text.chars().count())
                    .sum::<usize>()
                    + comp_line.char_start;
            for &(tac_abs_pos, tac_w, _) in
                line_tac_offsets_for_width.iter().filter(|(pos, _, _)| {
                    *pos >= run_char_pos_est
                        && (*pos < run_char_end_est
                            || (is_last_run_est_tac && *pos == run_char_end_est))
                })
            {
                let tac_rel = tac_abs_pos - run_char_pos_est;
                if seg_start_est < tac_rel {
                    let seg: String = run_chars_est[seg_start_est..tac_rel].iter().collect();
                    ts.line_x_offset = est_x;
                    est_x += estimate_text_width(&seg, &ts);
                }
                est_x += tac_w;
                included_tac_width_in_est += tac_w;
                seg_start_est = tac_rel;
            }
            // 마지막 세그먼트 처리
            let remaining_est: String = run_chars_est[seg_start_est..].iter().collect();
            ts.line_x_offset = est_x;
            // [Task #874 #2] composer lang split (예: "F3→Alt+I" → "F3"/"→"/"Alt+I")
            // 으로 auto_tab_right post-tab 콘텐츠가 후속 run 으로 흩어진 경우, 현재
            // run 내부 seg_w 만으로는 우측 정렬 위치가 어긋남. 후속 run 합산을 미리
            // 계산해 ts.right_tab_block_width_override 로 주입한다.
            if auto_tab_right
                && remaining_est.contains('\t')
                && run_idx_est + 1 < comp_line.runs.len()
            {
                let tab_byte = remaining_est.rfind('\t').unwrap();
                let post_tab: String = remaining_est[tab_byte + '\t'.len_utf8()..].to_string();
                let no_more_tabs_after_in_run = !post_tab.contains('\t');
                let no_tabs_in_subsequent = comp_line
                    .runs
                    .iter()
                    .skip(run_idx_est + 1)
                    .all(|r| !r.text.contains('\t'));
                if no_more_tabs_after_in_run && no_tabs_in_subsequent {
                    let mut ts_measure = ts.clone();
                    ts_measure.right_tab_block_width_override = None;
                    let post_tab_w = estimate_text_width(&post_tab, &ts_measure);
                    let subsequent_w = right_tab_block_width(
                        &comp_line.runs,
                        run_idx_est + 1,
                        styles,
                        tab_width,
                        &tab_stops,
                        auto_tab_right,
                        available_width,
                    );
                    ts.right_tab_block_width_override = Some(post_tab_w + subsequent_w);
                }
            }
            if !remaining_est.is_empty() {
                est_x += estimate_text_width(&remaining_est, &ts);
            }
            // run이 \t로 끝나면 다음 run에 오른쪽/가운데 탭 조정 필요 — Task #290:
            // inline_tabs(composed.tab_extended) 가 LEFT 를 명시하면 cross-run pending 을 설정하지 않는다.
            // [Task #279] trailing 공백 (\t 뒤에 따라오는 ' ') 도 허용 — 목차 소제목의
            // 들여쓰기 문단에서 한컴이 "\t " 형태로 저장하는 케이스가 있음.
            let trimmed_end = run
                .text
                .trim_end_matches(|c: char| c == ' ' || c == '\u{2007}');
            if trimmed_end.ends_with('\t') {
                let run_tab_count = run.text.chars().filter(|c| *c == '\t').count();
                if run_tab_count > 0 {
                    let last_inline_idx = inline_tab_cursor_est + run_tab_count - 1;
                    pending_right_tab_est = resolve_last_tab_pending(
                        &run.text,
                        last_inline_idx,
                        &composed.tab_extended,
                        &ts,
                        &tab_stops,
                        tab_width,
                        auto_tab_right,
                        available_width,
                    );
                }
            }
            if run.text.contains('\t')
                && run
                    .text
                    .rsplit_once('\t')
                    .map(|(_, after)| after.trim().is_empty())
                    .unwrap_or(false)
                && tab_stops
                    .iter()
                    .any(|tab| tab.tab_type == 1 && tab.fill_type != 0)
            {
                pending_right_leader_digit_est = true;
            }
            // 각주 마커 폭: run 내에 각주가 있으면 마커 위첨자 폭 추가
            let is_last_run_est = run_char_end_est
                >= comp_line
                    .runs
                    .iter()
                    .map(|r| r.text.chars().count())
                    .sum::<usize>()
                    + comp_line.char_start;
            for &(fpos, fnum, ctrl_idx) in composed.footnote_positions.iter() {
                // [Task #1219 Stage 1b] 선두 미주 마커는 endnote_marker_x_advance
                // 가 풀사이즈 선두 마커로 렌더하고 그 폭을 inline_offset 에 이미
                // 반영했다(available_width 에서 차감). 렌더 경로는 이 미주의 인라인
                // 위첨자를 그리지 않으므로(문26 "공" x=78=선두 마커 끝), 측정에서도
                // est_x 에 위첨자 폭을 더하면 이중 계상 → 거짓 오버플로우.
                // start_line==0 의 미주(= endnote_marker_x_advance 처리 대상)는 제외.
                let is_leading_endnote_marker = is_leading_endnote_marker_rendered_as_prefix(
                    para,
                    ctrl_idx,
                    line_idx,
                    start_line,
                    fpos,
                    comp_line.char_start,
                );
                if is_leading_endnote_marker {
                    continue;
                }
                if fpos >= run_char_pos_est
                    && (fpos < run_char_end_est || (is_last_run_est && fpos == run_char_end_est))
                {
                    let fn_text = note_marker_text_from_control(
                        para.and_then(|p| p.controls.get(ctrl_idx)),
                        fnum,
                    );
                    let sup_size = (ts.font_size * 0.55).max(7.0);
                    let sup_ts = TextStyle {
                        font_size: sup_size,
                        font_family: ts.font_family.clone(),
                        ..Default::default()
                    };
                    est_x += estimate_text_width(&fn_text, &sup_ts);
                }
            }
            run_char_pos_est = run_char_end_est;
            inline_tab_cursor_est += run.text.chars().filter(|c| *c == '\t').count();
        }
        LineWidthEst {
            est_x,
            included_tac_width: included_tac_width_in_est,
        }
    }

    /// [#1925 추출] runs 가 비어있는 줄 처리 — 빈 TextRun 생성(빈 셀 편집용)과
    /// 셀 외부 빈 줄의 treat_as_char 이미지/Shape 인라인 렌더링.
    #[allow(clippy::too_many_arguments)]
    fn layout_empty_runs_line(
        &self,
        tree: &mut PageRenderTree,
        line_node: &mut RenderNode,
        comp_line: &crate::renderer::composer::ComposedLine,
        composed: &ComposedParagraph,
        para: Option<&Paragraph>,
        bin_data_content: Option<&[BinDataContent]>,
        styles: &ResolvedStyleSet,
        cell_ctx: &Option<CellContext>,
        line_tac_offsets: &[(usize, f64, usize)],
        vars: EmptyRunsLineVars,
        current_line_reserved_tac_picture_height: &mut Option<f64>,
    ) {
        let mut empty_line_mark_x = vars.x_start;
        let mut empty_line_logical_end = vars.line_char_end;
        // runs가 없는 빈 줄에서 treat_as_char 이미지 렌더링
        // 테이블 셀 내부에서는 table_layout.rs가 layout_picture로 이미 처리하므로 스킵.
        // 셀 외부에서 해당 줄 범위에 걸린 TAC만 여기서 렌더링.
        let empty_line_tac_allowed =
            cell_ctx.is_none() || is_caption_cell_context(cell_ctx.as_ref());
        if empty_line_tac_allowed && !line_tac_offsets.is_empty() {
            if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                // TAC 이미지 전체 폭 계산 후 문단 정렬 적용
                let total_tac_width: f64 = line_tac_offsets.iter().map(|(_, w, _)| *w).sum();
                let align_offset = match vars.alignment {
                    Alignment::Center | Alignment::Distribute => {
                        (vars.available_width - total_tac_width).max(0.0) / 2.0
                    }
                    Alignment::Right => (vars.available_width - total_tac_width).max(0.0),
                    _ => 0.0, // Left, Justify
                };
                let mut img_x = vars.effective_col_x + vars.effective_margin_left + align_offset;
                for &(_, tac_w, tac_ci) in line_tac_offsets {
                    if let Some(ctrl) = p.controls.get(tac_ci) {
                        // [Issue #476] 빈 문단 + 인라인 Shape: inline_pos 등록 후 shape_layout 이 그리도록 위임.
                        // 등록하지 않으면 layout_shape 가 inline_pos=None 으로 받아 fallback 위치에 그리거나,
                        // #476 의 fallback 차단 분기로 박스가 누락된다.
                        if let Control::Shape(shape) = ctrl {
                            let common = shape.common();
                            let shape_h_hu = (common.height as i32)
                                .max(shape.shape_attr().current_height as i32);
                            let shape_h = hwpunit_to_px(shape_h_hu, self.dpi);
                            let shape_y = (vars.y + vars.baseline - shape_h).max(vars.y);
                            tree.set_inline_shape_position(
                                vars.section_index,
                                vars.para_index,
                                tac_ci,
                                cell_ctx.as_ref(),
                                img_x,
                                shape_y,
                            );
                            img_x += tac_w;
                            empty_line_mark_x = img_x;
                            empty_line_logical_end += 1;
                            continue;
                        }
                        if let Control::Picture(pic) = ctrl {
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            // LINE_SEG vpos가 TopAndBottom 흐름 위치를 이미 담고 있으면
                            // sibling 예약 높이를 다시 더하지 않는다.
                            let sibling_reserved_px = if vars.has_topbottom_vpos_base {
                                0.0
                            } else {
                                hwpunit_to_px(
                                    calc_sibling_topandbottom_reserved_hu(&p.controls),
                                    self.dpi,
                                )
                            };
                            if vars.raw_lh + 4.0 >= pic_h {
                                *current_line_reserved_tac_picture_height = Some(pic_h);
                            }
                            let label_extra = tac_picture_label_extra_for_line(
                                cell_ctx.as_ref(),
                                vars.runs_all_whitespace,
                                vars.raw_lh,
                                *current_line_reserved_tac_picture_height,
                                vars.max_fs,
                                vars.line_spacing_px,
                            );
                            let base_img_y = if label_extra > 0.0 {
                                vars.y + label_extra
                            } else {
                                (vars.y + vars.baseline - pic_h).max(vars.y)
                            };
                            let img_y = base_img_y + sibling_reserved_px;
                            let bin_data_id = pic.image_attr.bin_data_id;
                            let image_data =
                                find_bin_data(bdc, bin_data_id).map(|c| c.data.clone());
                            let crop = {
                                let c = &pic.crop;
                                if c.right > c.left
                                    && c.bottom > c.top
                                    && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0)
                                {
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
                            // [Task #1151 v7 항목 7] ImageNode 생성 helper 통합.
                            let img_node = make_picture_image_node(
                                tree,
                                pic,
                                vars.section_index,
                                vars.para_index,
                                tac_ci,
                                cell_ctx.as_ref(),
                                crop,
                                original_size_hu,
                                bin_data_id,
                                image_data,
                                BoundingBox::new(img_x, img_y, tac_w, pic_h),
                            );
                            line_node.children.push(img_node);
                            // [Task #418/#376] layout_shape_item 의 Task #347 분기 (빈 문단 +
                            // TAC Picture 직접 emit) 와 이중 렌더링되지 않도록 인라인 위치를
                            // 등록한다. layout_shape_item 은 등록된 경우 push 를 스킵한다.
                            tree.set_inline_shape_position(
                                vars.section_index,
                                vars.para_index,
                                tac_ci,
                                cell_ctx.as_ref(),
                                img_x,
                                img_y,
                            );
                            img_x += tac_w;
                            empty_line_mark_x = img_x;
                            empty_line_logical_end += 1;
                        }
                    }
                }
            }
        }

        let run_id = tree.next_id();
        let (text_style, char_shape_id) =
            paragraph_active_text_style(styles, para, vars.line_char_end);
        let run_node = RenderNode::new(
            run_id,
            RenderNodeType::TextRun(TextRunNode {
                text: String::new(),
                style: text_style,
                char_shape_id,
                para_shape_id: Some(composed.para_style_id),
                section_index: Some(vars.section_index),
                para_index: Some(vars.para_index),
                char_start: Some(empty_line_logical_end),
                cell_context: cell_ctx.clone(),
                is_para_end: vars.is_last_line_of_para && !vars.defer_empty_line_control_marker,
                is_line_break_end: comp_line.has_line_break
                    && !vars.defer_empty_line_control_marker,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: vars.baseline,
                field_marker: FieldMarkerType::None,
            }),
            BoundingBox::new(
                empty_line_mark_x,
                vars.y,
                if empty_line_mark_x > vars.x_start {
                    0.0
                } else {
                    vars.available_width
                },
                vars.line_flow_height,
            ),
        );
        line_node.children.push(run_node);
    }

    /// 원본 문단 데이터로 레이아웃 (ComposedParagraph 없는 경우 fallback)
    pub(crate) fn layout_raw_paragraph(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        para: &Paragraph,
        col_area: &LayoutRect,
        y_start: f64,
        start_line: usize,
        end_line: usize,
    ) -> f64 {
        let mut y = y_start;
        let end = end_line.min(para.line_segs.len());

        for line_idx in start_line..end {
            let line_seg = &para.line_segs[line_idx];
            let line_height = hwpunit_to_px(line_seg.line_height, self.dpi);
            let baseline = ensure_min_baseline(
                hwpunit_to_px(line_seg.baseline_distance, self.dpi),
                line_height * 0.8, // fallback: 줄 높이 기반 최소 어센트
            );

            // Task #332 Stage 4b: clamp 제거, overflow 그대로 그림 (piling 차단)
            let col_bottom = col_area.y + col_area.height;
            if self.is_body_flow_col_area(col_area) && y + line_height > col_bottom + 0.5 {
                eprintln!(
                    "LAYOUT_OVERFLOW_DRAW: line={} y={:.1} col_bottom={:.1} overflow={:.1}px (fast path)",
                    line_idx, y + line_height, col_bottom, y + line_height - col_bottom,
                );
            }
            let y_clamped = y;
            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(line_height, baseline)),
                BoundingBox::new(col_area.x, y_clamped, col_area.width, line_height),
            );

            if !para.text.is_empty() && line_idx == start_line {
                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: para.text.clone(),
                        style: TextStyle::default(),
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: None,
                        para_index: None,
                        char_start: None,
                        cell_context: None,
                        is_para_end: line_idx == end - 1,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline: line_height * 0.85,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(col_area.x, y_clamped, col_area.width, line_height),
                );
                line_node.children.push(run_node);
            }

            col_node.children.push(line_node);
            // 줄간격 적용: line_height에 line_spacing 추가
            let line_spacing_px = hwpunit_to_px(line_seg.line_spacing, self.dpi);
            y += line_height + line_spacing_px;
        }

        if para.line_segs.is_empty() {
            let default_height = hwpunit_to_px(400, self.dpi);
            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(default_height, default_height * 0.8)),
                BoundingBox::new(col_area.x, y, col_area.width, default_height),
            );

            if !para.text.is_empty() {
                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: para.text.clone(),
                        style: TextStyle::default(),
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
                        baseline: default_height * 0.8,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(col_area.x, y, col_area.width, default_height),
                );
                line_node.children.push(run_node);
            }

            col_node.children.push(line_node);
            y += default_height;
        }

        y
    }

    pub(crate) fn apply_paragraph_numbering(
        &self,
        composed: Option<&ComposedParagraph>,
        para: &Paragraph,
        styles: &ResolvedStyleSet,
        outline_numbering_id: u16,
    ) -> Option<ComposedParagraph> {
        let para_style = styles.para_styles.get(para.para_shape_id as usize)?;

        let head_text = match para_style.head_type {
            HeadType::None => return None,
            HeadType::Outline | HeadType::Number => {
                let numbering_id = resolve_numbering_id(
                    para_style.head_type,
                    para_style.numbering_id,
                    outline_numbering_id,
                );
                let level = para_style.para_level;
                if numbering_id == 0 {
                    return None;
                }
                let numbering = styles.numberings.get((numbering_id - 1) as usize)?;

                let counters = self.numbering_state.borrow_mut().advance(
                    numbering_id,
                    level,
                    para.numbering_restart,
                );
                let start_numbers = numbering.level_start_numbers;

                let level_idx = (level as usize).min(6);
                let format_str = &numbering.level_formats[level_idx];
                if format_str.is_empty() {
                    return None;
                }

                let text =
                    expand_numbering_format(format_str, &counters, numbering, &start_numbers);
                if text.is_empty() {
                    return None;
                }
                let has_distance = numbering
                    .heads
                    .get(level_idx)
                    .map(|h| h.text_distance > 0)
                    .unwrap_or(false);
                if has_distance {
                    format!("{} ", text)
                } else {
                    text
                }
            }
            HeadType::Bullet => {
                // Bullet: numbering_id(1-based)로 Bullet 참조
                let bullet_id = para_style.numbering_id;
                if bullet_id == 0 {
                    return None;
                }
                let bullet = styles.bullets.get((bullet_id - 1) as usize)?;
                // U+FFFF는 이미지 글머리표 표시자 — 문자 렌더링 불가, 건너뜀
                if bullet.bullet_char == '\u{FFFF}' {
                    return None;
                }
                // PUA 문자(0xF000~0xF0FF)를 표준 Unicode로 매핑
                // HWP는 Symbol 폰트 문자를 PUA(0xF000+code)로 저장
                let bullet_ch = map_pua_bullet_char(bullet.bullet_char);
                // 글머리 기호 + 본문과의 거리(text_distance)에 따른 간격
                if bullet.text_distance > 0 {
                    format!("{} ", bullet_ch)
                } else {
                    format!("{}", bullet_ch)
                }
            }
        };

        // 번호 텍스트를 별도 필드에 저장 (첫 run에 prepend하지 않음)
        // 렌더링 시 별도 TextRunNode로 생성하여 char_offset에 영향을 주지 않는다.
        let comp = composed?;
        let mut modified = comp.clone();
        modified.numbering_text = Some(head_text);

        Some(modified)
    }

    /// 조합된 문단의 텍스트에 AutoNumber를 적용한다.
    pub(crate) fn apply_auto_numbers_to_composed(
        &self,
        composed: &mut ComposedParagraph,
        para: &Paragraph,
        _counter: &mut super::AutoNumberCounter, // 더 이상 사용하지 않음 (파싱 시 할당됨)
    ) {
        // AutoNumber 컨트롤이 있는지 확인
        for ctrl in &para.controls {
            if let Control::AutoNumber(an) = ctrl {
                // 파싱 시점에 할당된 번호를 번호 형식에 맞게 변환 + 장식 문자 적용
                let num_fmt = NumFmt::from_hwp_format(an.format);
                let num_str = format_number(an.assigned_number, num_fmt);
                let num_str = if an.prefix_char != '\0' || an.suffix_char != '\0' {
                    format!(
                        "{}{}{}",
                        if an.prefix_char != '\0' {
                            an.prefix_char.to_string()
                        } else {
                            String::new()
                        },
                        num_str,
                        if an.suffix_char != '\0' {
                            an.suffix_char.to_string()
                        } else {
                            String::new()
                        },
                    )
                } else {
                    num_str
                };

                // 각 줄의 텍스트에서 AutoNumber 위치를 찾아 번호로 대체
                // HWP5/HWPX/HWP3 공통: 공백 두 개("  ") 패턴 탐색
                for line in &mut composed.lines {
                    for run in &mut line.runs {
                        if let Some(pos) = run.text.find("  ") {
                            run.text = format!(
                                "{}{}{}",
                                &run.text[..pos + 1],
                                num_str,
                                &run.text[pos + 1..]
                            );
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// paragraph 의 sibling controls 중 `wrap=TopAndBottom` +
/// `treat_as_char=false` 인 개체가 차지하는 vertical 영역 (HWPUNIT) 합산.
///
/// 한컴 layout 정합 (`mydocs/tech/topandbottom_table_inline_picture_layout.md` H1):
/// 같은 paragraph 의 sibling tac picture 가 표 아래 영역에 그려지도록 picture
/// 의 y 위치 보정값을 계산한다. 예약 개체가 없으면 0 반환 (회귀 0 보장).
///
/// 합산 공식:
/// - 표: `common.height + outer_margin_top + outer_margin_bottom`
/// - 그림/도형: `common.height + common.margin.top + common.margin.bottom`
pub(crate) fn calc_sibling_topandbottom_reserved_hu(
    controls: &[crate::model::control::Control],
) -> i32 {
    use crate::model::control::Control;
    use crate::model::shape::TextWrap;
    controls
        .iter()
        .map(|c| match c {
            Control::Table(t)
                if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
                    && !t.common.treat_as_char =>
            {
                t.common.height as i32 + t.outer_margin_top as i32 + t.outer_margin_bottom as i32
            }
            Control::Picture(p)
                if matches!(p.common.text_wrap, TextWrap::TopAndBottom)
                    && !p.common.treat_as_char =>
            {
                p.common.height as i32 + p.common.margin.top as i32 + p.common.margin.bottom as i32
            }
            Control::Shape(s)
                if matches!(s.common().text_wrap, TextWrap::TopAndBottom)
                    && !s.common().treat_as_char =>
            {
                let common = s.common();
                common.height as i32 + common.margin.top as i32 + common.margin.bottom as i32
            }
            _ => 0,
        })
        .sum()
}

/// [Task #1151 v7 항목 7] paragraph_layout 의 3 곳에서 반복되던 ImageNode 생성
/// boilerplate 통합 (cell_ctx → 3 필드 + outer paragraph idx 노출 + picture 의
/// effect/brightness/contrast/text_wrap/transform 매핑). picture_footnote 의
/// `layout_picture_full` 가 본문/머리말/꼬리말 path 의 진입점 helper 인 것과 짝.
#[allow(clippy::too_many_arguments)]
fn make_picture_image_node(
    tree: &mut PageRenderTree,
    pic: &crate::model::image::Picture,
    section_index: usize,
    para_index: usize,
    ctrl_idx: usize,
    cell_ctx: Option<&CellContext>,
    crop: Option<(i32, i32, i32, i32)>,
    original_size_hu: Option<(u32, u32)>,
    bin_data_id: u16,
    image_data: Option<Vec<u8>>,
    bbox: BoundingBox,
) -> RenderNode {
    let (cei, cpi, otci) = cell_ctx
        .map(|c| c.last_image_indices())
        .unwrap_or((None, None, None));
    let para_for_image = cell_ctx.map(|c| c.parent_para_index).unwrap_or(para_index);
    let img_id = tree.next_id();
    RenderNode::new(
        img_id,
        RenderNodeType::Image(ImageNode {
            section_index: Some(section_index),
            para_index: Some(para_for_image),
            control_index: Some(ctrl_idx),
            cell_index: cei,
            cell_para_index: cpi,
            outer_table_control_index: otci,
            // [Task #1161] 전체 다단계 경로 보존(스칼라는 위 innermost 투영).
            cell_context: cell_ctx.cloned(),
            crop,
            original_size_hu,
            effect: pic.image_attr.effect,
            brightness: pic.image_attr.brightness,
            contrast: pic.image_attr.contrast,
            opacity: pic.image_attr.opacity(),
            text_wrap: Some(pic.common.text_wrap),
            transform: extract_shape_transform(&pic.shape_attr),
            external_path: pic.image_attr.external_path.clone(),
            ..ImageNode::new(bin_data_id, image_data)
        }),
        bbox,
    )
}

/// [Task #1151 v9 결함 D] paragraph 의 sibling TAC picture 들의 (control_idx, width_px)
/// 시퀀스 수집 (시점순). layout_shape_item 의 가로 분배 cursor / alignment 계산용.
///
/// 한컴 native 정합: 동일 paragraph 안 sibling tac=true picture 들이 가로로 inline
/// 분배 (inline glyph 처럼). 첫 picture 시점에 전체 시퀀스 폭을 알아야 alignment
/// (center / right) 의 시작 x 가 정확히 계산되므로 pre-scan helper 가 필요.
pub(crate) fn collect_sibling_tac_picture_widths_px(
    controls: &[crate::model::control::Control],
    dpi: f64,
) -> Vec<(usize, f64)> {
    use crate::model::control::Control;
    controls
        .iter()
        .enumerate()
        .filter_map(|(ci, c)| match c {
            Control::Picture(p) if p.common.treat_as_char => {
                Some((ci, hwpunit_to_px(p.common.width as i32, dpi)))
            }
            _ => None,
        })
        .collect()
}

/// [Task #1151 v9 결함 D] paragraph 단위 inline picture 의 가로 분배 cursor 상태.
/// layout_shape_item 이 같은 paragraph 의 sibling TAC picture 들을 순서대로 처리할 때
/// HashMap<para_index, ParaInlineState> 에 보관하여 가로 누적 + line wrap 처리.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParaInlineState {
    /// 다음 picture 의 x 시작점 (paper-relative px)
    pub cursor_x: f64,
    /// 현재 line 의 y (= first picture 의 pic_y, 가로 분배 시 유지)
    pub line_top_y: f64,
    /// 현재 line 의 최대 picture height (line wrap 임계 + 다음 line advance 용)
    pub line_height: f64,
}

#[cfg(test)]
mod issue_1151_v3_helper_tests {
    //! Issue #1151 v3/#1459: sibling TopAndBottom 예약 높이 helper 단위 검증.
    //!
    //! 한컴 정합: wrap=TopAndBottom + tac=false 인 개체가 vertical 영역
    //! reservation 으로 합산된다. TAC 개체와 Square wrap 은 제외한다.

    use super::calc_sibling_topandbottom_reserved_hu;
    use crate::model::control::Control;
    use crate::model::image::Picture;
    use crate::model::shape::{CommonObjAttr, TextWrap};
    use crate::model::table::Table;

    fn make_table(width: u32, height: u32, wrap: TextWrap, tac: bool) -> Table {
        Table {
            common: CommonObjAttr {
                width,
                height,
                text_wrap: wrap,
                treat_as_char: tac,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            ..Default::default()
        }
    }

    #[test]
    fn topandbottom_table_reserved_single() {
        // scenario-a-after.hwp 의 표: 13630×12498, outer_margin (top=283, bottom=283).
        // 합산 = 12498 + 283 + 283 = 13064 HU.
        let table = make_table(13630, 12498, TextWrap::TopAndBottom, false);
        let controls = vec![Control::Table(Box::new(table))];
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 13064);
    }

    #[test]
    fn topandbottom_table_reserved_none_when_no_table() {
        let controls: Vec<Control> = vec![];
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 0);
    }

    #[test]
    fn topandbottom_table_reserved_excludes_tac_table() {
        let table = make_table(13630, 12498, TextWrap::TopAndBottom, true); // tac=true 제외
        let controls = vec![Control::Table(Box::new(table))];
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 0);
    }

    #[test]
    fn topandbottom_table_reserved_excludes_square_wrap() {
        let table = make_table(13630, 12498, TextWrap::Square, false); // wrap=Square 제외
        let controls = vec![Control::Table(Box::new(table))];
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 0);
    }

    #[test]
    fn topandbottom_reserved_includes_non_tac_picture_control() {
        let mut pic = Picture::default();
        pic.common.text_wrap = TextWrap::TopAndBottom;
        pic.common.treat_as_char = false;
        pic.common.height = 7733;
        pic.common.margin.top = 100;
        pic.common.margin.bottom = 200;
        let controls = vec![Control::Picture(Box::new(pic))];
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 8033);
    }

    #[test]
    fn topandbottom_reserved_excludes_tac_picture_control() {
        let mut pic = Picture::default();
        pic.common.text_wrap = TextWrap::TopAndBottom;
        pic.common.treat_as_char = true;
        pic.common.height = 7733;
        let controls = vec![Control::Picture(Box::new(pic))];
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 0);
    }

    #[test]
    fn topandbottom_table_reserved_sums_multiple_tables() {
        let t1 = make_table(13630, 10000, TextWrap::TopAndBottom, false);
        let t2 = make_table(13630, 5000, TextWrap::TopAndBottom, false);
        let controls = vec![Control::Table(Box::new(t1)), Control::Table(Box::new(t2))];
        // (10000 + 283 + 283) + (5000 + 283 + 283) = 10566 + 5566 = 16132
        assert_eq!(calc_sibling_topandbottom_reserved_hu(&controls), 16132);
    }
}

#[cfg(test)]
mod issue_1151_v9_helper_tests {
    //! [Task #1151 v9 결함 D] collect_sibling_tac_picture_widths_px helper 단위 검증.

    use super::collect_sibling_tac_picture_widths_px;
    use crate::model::control::Control;
    use crate::model::image::Picture;
    use crate::model::shape::CommonObjAttr;
    use crate::model::table::Table;

    fn make_pic(width: u32, height: u32, tac: bool) -> Picture {
        Picture {
            common: CommonObjAttr {
                width,
                height,
                treat_as_char: tac,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn empty_controls_returns_empty() {
        assert!(collect_sibling_tac_picture_widths_px(&[], 96.0).is_empty());
    }

    #[test]
    fn collects_single_tac_picture() {
        // 5670 HU @ 96 dpi = 5670 * 96 / 7200 = 75.6 px
        let controls = vec![Control::Picture(Box::new(make_pic(5670, 5670, true)))];
        let result = collect_sibling_tac_picture_widths_px(&controls, 96.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 0);
        assert!((result[0].1 - 75.6).abs() < 0.01);
    }

    #[test]
    fn collects_multiple_tac_pictures_in_order() {
        let controls = vec![
            Control::Picture(Box::new(make_pic(3000, 3000, true))),
            Control::Picture(Box::new(make_pic(4500, 4500, true))),
        ];
        let result = collect_sibling_tac_picture_widths_px(&controls, 96.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 1);
    }

    #[test]
    fn skips_non_tac_picture() {
        // tac=false 인 picture (floating) 는 가로 분배 대상 아님 — 제외.
        let controls = vec![
            Control::Picture(Box::new(make_pic(3000, 3000, false))),
            Control::Picture(Box::new(make_pic(4500, 4500, true))),
        ];
        let result = collect_sibling_tac_picture_widths_px(&controls, 96.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 1); // 두 번째 (tac=true) 만
    }

    #[test]
    fn skips_table_and_other_controls() {
        // Table / Shape 는 가로 분배 대상 아님 (Picture 만).
        let controls = vec![
            Control::Table(Box::default()),
            Control::Picture(Box::new(make_pic(5670, 5670, true))),
            Control::Picture(Box::new(make_pic(5670, 5670, true))),
        ];
        let result = collect_sibling_tac_picture_widths_px(&controls, 96.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[1].0, 2);
    }

    #[test]
    fn realistic_v1_scenario_1x1_table_two_tac_pictures() {
        // 사용자 시연 정확 재현: [Table(tac=false), Pic1(tac=true), Pic2(tac=true)]
        let controls = vec![
            Control::Table(Box::default()),
            Control::Picture(Box::new(make_pic(5670, 5670, true))),
            Control::Picture(Box::new(make_pic(5670, 5670, true))),
        ];
        let result = collect_sibling_tac_picture_widths_px(&controls, 96.0);
        assert_eq!(result.len(), 2);
        let total_width: f64 = result.iter().map(|(_, w)| w).sum();
        assert!((total_width - 151.2).abs() < 0.01); // 75.6 + 75.6
    }
}

/// HWP PUA 문자를 표준 Unicode 로 매핑.
///
/// 두 영역 분기 — Task #509 정답지 매핑 표 정합:
///
/// **Basic PUA (0xF020~0xF0FF)** — Wingdings 폰트 PUA 영역.
///   기준: Wingdings 폰트 → Unicode 매핑 (alanwood.net/demos/wingdings.html).
///   HWP 글머리표는 Wingdings 폰트 문자를 PUA(0xF000+code)로 저장.
///
/// **Supplementary PUA-A (0xF02B0~0xF02FF)** — 한컴 자체 PUA 영역.
///   원문자 (①~⑳, U+2460~U+2473) 와 · (U+00B7) 등을 본 영역에 저장.
///   Task #509 의 한컴 PDF 정답지 시각 검증으로 매핑 확정.
///
/// **Supplementary PUA-A 저영역 (0xF0000~0xF00CF)** — 한컴 자체 PUA 저영역.
///   요약형 문항 화살표 등 시각 마커. Task #588 의 한컴 PDF 임베디드 폰트
///   글리프 외곽 분석 + 정답지 시각 검증으로 매핑 확정.
pub fn map_pua_bullet_char(ch: char) -> char {
    let code = ch as u32;

    // Supplementary PUA-A 저영역 — 한컴 자체 영역 (Task #588 한컴 정답지 정합)
    if (0xF0000..=0xF00CF).contains(&code) {
        return match code {
            // exam_eng.hwp p7 #40 요약형 문항 글상자 사이 화살표.
            // 한컴 PDF (HCRBatang 임베디드 폰트) 글리프 외곽 분석:
            //   stem 35% × arrowhead 100% × solid filled (1 contour, 7 pts) → ↓
            0xF003B => '\u{2193}', // ↓ DOWNWARDS ARROW
            _ => ch,
        };
    }

    // Supplementary PUA-A — 한컴 자체 영역 (Task #509 한컴 정답지 정합)
    if (0xF02B0..=0xF02FF).contains(&code) {
        return match code {
            // 캡스톤 F-1 (2026-05-16): U+F02B1~F02C4 사각 안 숫자 한컴 자체 PUA 글리프.
            // 한글 2024 복사 + PowerShell 디코딩으로 "사각 안 1" = 0xF02B1 확정.
            // 이전 표준 U+2460-U+2473 매핑 (Task #509 mel-001 영역) 은 fallback chain 효과
            // 못 받음 — 매핑 결과 표준 ① 가 1순위 폰트 (맑은 고딕 등) 의 원 안 글리프로
            // 즉시 렌더링 (글리프 단위 fallback 작동 안 함). raw PUA passthrough +
            // generic_fallback() 의 함초롬바탕 확장B 등이 PUA 영역 글리프 (사각 안) 매칭.
            // 두 대상 파일 (HWPX 스마트행정팀, HWP 공직기강) 모두 같은 PUA, 한컴 동일 글리프.
            //
            // KTX 회귀 origin — 한컴 PDF 시각 = · (Middle dot), ★ 아님
            // (작업지시자 정정 — 이전 ★ U+2605 매핑은 잘못)
            0xF02EF => '\u{00B7}', // · Middle dot
            _ => ch,
        };
    }

    // Supplementary PUA-A — 한컴 책괄호 / 예시 마커 (Task #528 exam_kor p17)
    // exam_kor p17 측정: F0854/F0855 각 33회 (책 제목 둘러싸기), F00DA 2회
    if (0xF00D0..=0xF09FF).contains(&code) {
        return match code {
            // 책괄호 (한국어 도서 제목) — 용비어천가, 석보상절, 월인천강지곡 등
            0xF0854 => '\u{300A}', // 《 LEFT DOUBLE ANGLE BRACKET
            0xF0855 => '\u{300B}', // 》 RIGHT DOUBLE ANGLE BRACKET
            // 예시 마커 — `(F00DA 단풍 철 : 철 성분)` 패턴 — 한컴 PDF 시각 검증 필요
            0xF00DA => '\u{25B8}', // ▸ BLACK SMALL TRIANGLE (잠정, 시각 판정 후 정정)
            // [Task #826] HWP3 한컴 PUA 그래픽 라인 (PR #753 후속 — johab.rs:65,67).
            // 한컴 함초롬 폰트는 PUA glyph 보유, rhwp-studio 번들 폰트 (오픈 라이선스)
            // 부재 → render-time substitution. 측정/렌더링 양쪽 자동 적용.
            // sample11.hwp 머리말/꼬리말 가로선 패턴 (각 85+ 회) 시각 정합.
            0xF080F => '\u{2501}', // ━ BOX DRAWINGS HEAVY HORIZONTAL (한컴 — 굵은 가로선)
            // [Task #1692 Stage 9] HWP3 관계도 계열 선문자.
            // 한컴은 U+F0811/F0817/F081A를 자체 글리프로 이어진 선처럼 렌더한다.
            // 공개 폰트 경로에서는 .notdef 두부가 나오므로 대응 가능한 box drawing으로 낮춘다.
            0xF0811 => '\u{250C}', // ┌ BOX DRAWINGS LIGHT DOWN AND RIGHT
            0xF0817 => '\u{2514}', // └ BOX DRAWINGS LIGHT UP AND RIGHT
            0xF081A => '\u{2500}', // ─ BOX DRAWINGS LIGHT HORIZONTAL
            0xF0827 => '\u{25A0}', // ■ BLACK SQUARE (한컴 — 잠정, 시각 판정 후 조정)
            _ => ch,
        };
    }

    if !(0xF020..=0xF0FF).contains(&code) {
        return ch;
    }
    let w = (code - 0xF000) as u8;
    match w {
        // 도형/기호 (0x6C~0x7E)
        0x6C => '\u{25CF}', // ● Black circle
        0x6D => '\u{25CF}', // ● (Lower right shadowed white circle → 근사값)
        0x6E => '\u{25A0}', // ■ Black square
        0x6F => '\u{25A1}', // □ White square
        0x70 => '\u{25A1}', // □ (Bold white square → 근사값)
        0x71 => '\u{25A1}', // □ (Lower right shadowed → 근사값)
        0x72 => '\u{25A1}', // □ (Upper right shadowed → 근사값)
        0x73 => '\u{2B27}', // ⬧ Black medium lozenge
        0x74 => '\u{29EB}', // ⧫ Black lozenge
        0x75 => '\u{25C6}', // ◆ Black diamond
        0x76 => '\u{2756}', // ❖ Black diamond minus white X
        0x77 => '\u{2B25}', // ⬥ Black medium diamond
        // 체크/별/점 (0x9E~0xAF)
        0x9E => '\u{00B7}', // · Middle dot
        0x9F => '\u{2022}', // • Bullet
        // [Task #509] 0xA0 → · U+00B7 (Middle dot) — 한컴 PDF 정답지 시각 정합.
        // ▪ U+25AA (Black small square) 영역 아님 (synam-001 사용 영역).
        0xA0 => '\u{00B7}', // · Middle dot
        0xA1 => '\u{26AA}', // ⚪ Medium white circle
        0xA2 => '\u{25CB}', // ○ (Heavy large circle → 근사값)
        0xA3 => '\u{25CB}', // ○ (Very heavy white circle → 근사값)
        0xA4 => '\u{25C9}', // ◉ Fisheye
        0xA5 => '\u{25CE}', // ◎ Bullseye
        0xA7 => '\u{25AA}', // ▪ Black small square
        0xA8 => '\u{25FB}', // ◻ White medium square
        0xAA => '\u{2726}', // ✦ Black four pointed star
        0xAB => '\u{2605}', // ★ Black star
        0xAC => '\u{2736}', // ✶ Six pointed black star
        0xAD => '\u{2734}', // ✴ Eight pointed black star
        0xAE => '\u{2739}', // ✹ Twelve pointed black star
        // 손 모양 (0x45~0x48)
        0x45 => '\u{261C}', // ☜ White left pointing index
        0x46 => '\u{261E}', // ☞ White right pointing index
        0x47 => '\u{261D}', // ☝ White up pointing index
        0x48 => '\u{261F}', // ☟ White down pointing index
        // 체크마크 (0xFB~0xFE)
        0xFB => '\u{2717}', // ✗ Ballot X (근사값)
        0xFC => '\u{2714}', // ✔ Heavy check mark
        0xFD => '\u{2612}', // ☒ Ballot box with X (근사값)
        0xFE => '\u{2611}', // ☑ Ballot box with check (근사값)
        // 화살표 (0xEF~0xF8)
        // [Task #509] 0xE8 → ➔ U+2794 (Heavy wide-headed rightwards arrow) —
        // 한컴 PDF 정답지 시각 정합. ➤ U+27A4 (Black rightwards) 와 글리프 형태
        // 차이 — 한컴은 wide-headed arrow 영역.
        0xE8 => '\u{2794}', // ➔ Heavy wide-headed rightwards arrow
        0xEF => '\u{21E6}', // ⇦ Leftwards white arrow
        0xF0 => '\u{21E8}', // ⇨ Rightwards white arrow
        0xF1 => '\u{21E7}', // ⇧ Upwards white arrow
        0xF2 => '\u{21E9}', // ⇩ Downwards white arrow
        // 기타 자주 쓰이는 기호
        0x22 => '\u{2702}', // ✂ Black scissors
        0x36 => '\u{231B}', // ⌛ Hourglass
        0x4A => '\u{263A}', // ☺ White smiling face
        0x4E => '\u{2620}', // ☠ Skull and crossbones
        0x52 => '\u{263C}', // ☼ White sun with rays
        0x54 => '\u{2744}', // ❄ Snowflake
        0x58 => '\u{2720}', // ✠ Maltese cross
        0x59 => '\u{2721}', // ✡ Star of David
        // 매핑 없는 PUA 문자는 원본 유지
        _ => ch,
    }
}

/// HWP COLORREF (0x00BBGGRR) → CSS 색상 문자열 변환
fn form_color_to_css(color: u32) -> String {
    let b = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let r = color & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod pua_mapping_tests {
    use super::map_pua_bullet_char;

    #[test]
    fn supplementary_pua_a_passthrough_for_boxed_digits() {
        // 캡스톤 F-1 (2026-05-16): U+F02B1~F02C4 사각 안 숫자 한컴 자체 PUA — raw
        // passthrough (이전 ①~⑳ 표준 매핑은 fallback chain 효과 못 받아 NG). 시스템
        // 한컴 폰트 (함초롬바탕 확장B 등) 가 PUA 영역에서 사각 글리프 렌더링.
        for cp in 0xF02B1..=0xF02C4 {
            let ch = char::from_u32(cp).unwrap();
            assert_eq!(
                map_pua_bullet_char(ch),
                ch,
                "U+{:05X} should passthrough",
                cp
            );
        }
    }

    #[test]
    fn supplementary_pua_a_maps_middle_dot() {
        // [Task #509] U+F02EF → U+00B7 · Middle dot (KTX p10 표 회귀 origin)
        // 한컴 PDF 시각 정답지: dot (·) — ★ 가 아님 (작업지시자 정정)
        assert_eq!(map_pua_bullet_char('\u{F02EF}'), '\u{00B7}');
    }

    #[test]
    fn basic_pua_arrow_e8() {
        // [Task #509] U+0F0E8 → U+2794 ➔ (Heavy wide-headed rightwards arrow,
        // 한컴 PDF 정답지 시각 정합)
        assert_eq!(map_pua_bullet_char('\u{F0E8}'), '\u{2794}');
    }

    #[test]
    fn supplementary_pua_a_unmapped_returns_original() {
        // 매핑 표 외 영역은 원본 유지
        assert_eq!(map_pua_bullet_char('\u{F0500}'), '\u{F0500}');
    }

    #[test]
    fn basic_pua_outside_range_returns_original() {
        // 0xF020~0xF0FF 외 Basic PUA 는 원본 유지 (예: U+0F53A 한글 "흔")
        assert_eq!(map_pua_bullet_char('\u{F53A}'), '\u{F53A}');
    }

    #[test]
    fn supplementary_pua_a_low_range_maps_down_arrow() {
        // [Task #588] U+F003B → U+2193 ↓ (DOWNWARDS ARROW)
        // exam_eng.hwp p7 #40 요약형 문항 글상자 사이 화살표.
        // 한컴 PDF (HCRBatang) 임베디드 폰트 글리프 외곽 분석으로 확정.
        assert_eq!(map_pua_bullet_char('\u{F003B}'), '\u{2193}');
    }

    #[test]
    fn supplementary_pua_a_low_range_unmapped_returns_original() {
        // [Task #588] 0xF0000~0xF00CF 영역의 매핑 표 외 코드포인트는 원본 유지
        // (예: U+F0090 — img-start-001.hwp 1건, 별도 task 후보)
        assert_eq!(map_pua_bullet_char('\u{F0090}'), '\u{F0090}');
        assert_eq!(map_pua_bullet_char('\u{F0000}'), '\u{F0000}');
        assert_eq!(map_pua_bullet_char('\u{F00CF}'), '\u{F00CF}');
    }
}

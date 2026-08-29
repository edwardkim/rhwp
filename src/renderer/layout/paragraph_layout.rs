//! 문단 레이아웃 (인라인 표, 문단 전체/부분, composed/raw) + 번호 매기기

use super::super::composer::{
    compose_paragraph, effective_text_for_metrics, ComposedLine, ComposedParagraph, ComposedTextRun,
};
use super::super::height_measurer::MeasuredTable;
use super::super::kerning::{
    ExactFontSlot, KerningLayoutSession, KerningRunMeasurementDisposition,
};
use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{
    format_number, hwpunit_to_px, px_to_hwpunit, AutoNumberCounter, NumberFormat as NumFmt,
    ShapeStyle, TabStop, TextStyle,
};
use super::border_rendering::create_border_line_nodes;
use super::text_measurement::{
    compute_char_positions, estimate_text_width, estimate_text_width_unrounded,
    extract_tab_leaders_with_extended, find_next_tab_stop, resolved_to_text_style,
};
use super::utils::{
    expand_numbering_format, extract_shape_transform, find_bin_data_bytes,
    numbering_format_to_number_format, picture_display_size_hu, resolve_numbering_id,
};
use super::{CellContext, LayoutEngine};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::{
    CaptionDirection, CommonObjAttr, HorzAlign, HorzRelTo, ShapeObject, TextWrap, VertRelTo,
};
use crate::model::style::{Alignment, HeadType, LineSpacingType, Numbering, UnderlineType};
use crate::model::table::Table;

const CAPTION_CELL_SENTINEL: usize = 65534;

/// 최종 emitted text run에서만 exact pair positions를 게시한다.
///
/// `fallback_width`는 K0의 기존 반올림·field projection·줄-말미 공백 회수 계약을
/// 그대로 보존한다. exact pair가 실제 적용된 경우에만 최종 positions의 끝값을
/// bbox/다음 run advance로 사용한다.
#[allow(clippy::too_many_arguments)]
fn emitted_run_layout_positions(
    session: &mut KerningLayoutSession<'_>,
    slot: ExactFontSlot,
    replay_text: &str,
    style: &TextStyle,
    fallback_width: f64,
    trailing_space_count: usize,
    eligible: bool,
) -> (f64, Option<Vec<f64>>) {
    if !eligible
        || !style.kerning
        || replay_text.is_empty()
        || session.source_handle(slot).is_none()
    {
        return (fallback_width, None);
    }

    let mut base_positions = compute_char_positions(replay_text, style);
    let scalar_count = replay_text.chars().count();
    let trailing_space_count = trailing_space_count.min(scalar_count);
    if trailing_space_count > 0 && style.extra_word_spacing != 0.0 {
        let trailing_start = scalar_count - trailing_space_count;
        for (index, position) in base_positions
            .iter_mut()
            .enumerate()
            .skip(trailing_start + 1)
        {
            *position -= style.extra_word_spacing * (index - trailing_start) as f64;
        }
    }

    let base_font_size = if style.font_size > 0.0 {
        style.font_size
    } else {
        12.0
    };
    let effective_font_size_px = if style.superscript || style.subscript {
        base_font_size * crate::renderer::SCRIPT_FONT_SCALE
    } else {
        base_font_size
    };
    let width_ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    let measurement = session.measure_run(
        slot,
        replay_text,
        true,
        base_positions,
        effective_font_size_px,
        width_ratio,
    );
    if measurement.disposition != KerningRunMeasurementDisposition::PairAdjusted {
        return (fallback_width, None);
    }
    let Some(positions) = measurement.pair_adjusted_positions else {
        return (fallback_width, None);
    };
    let Some(width) = positions.last().copied() else {
        return (fallback_width, None);
    };
    if !width.is_finite() || width < 0.0 {
        return (fallback_width, None);
    }
    (width, Some(positions))
}

/// Q2-D4-B 최초 lane의 문단 전체 feature detection이다. 버전이나 파일 형식은
/// 보지 않고 최종 composed surface와 현재 style capability만 판정한다.
fn horizontal_shaping_initial_lane_preflight(
    composed: &ComposedParagraph,
    para: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    start_line: usize,
    end_line: usize,
    alignment: Alignment,
    para_border_fill_id: u16,
) -> bool {
    let (Some(para), Some(context), Some(outcome)) = (
        para,
        styles.horizontal_shaping_context.as_ref(),
        composed.horizontal_shaping.as_ref(),
    ) else {
        return false;
    };
    let (Some(line), Some(final_line)) = (composed.lines.first(), outcome.lines.first()) else {
        return false;
    };
    let Some(run) = line.runs.first() else {
        return false;
    };
    let Some(target) = final_line.target_runs.first() else {
        return false;
    };
    let style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
    let scalar_count = run.text.chars().count();
    let style_surface_supported = !style.bold
        && !style.italic
        && style.font_size.is_finite()
        && style.font_size > 0.0
        && style.font_size <= 4_096.0
        && style.ratio.is_finite()
        && style.ratio > 0.0
        && style.ratio < 0.999
        && style.letter_spacing.abs() <= f64::EPSILON
        && style.underline == UnderlineType::None
        && !style.strikethrough
        && style.outline_type == 0
        && style.shadow_type == 0
        && !style.emboss
        && !style.engrave
        && !style.superscript
        && !style.subscript
        && style.emphasis_dot == 0
        && crate::model::color::char_shade(style.shade_color).is_none();
    let para_style_supported = styles
        .para_styles
        .get(composed.para_style_id as usize)
        .is_some_and(|style| {
            style.alignment == Alignment::Left
                && style.border_fill_id == 0
                && style.condense_min_space == 0
                && !style.auto_tab_right
        });
    let raw_vertical_positioning_is_zero = target
        .measurement
        .applied
        .glyphs
        .iter()
        .all(|glyph| glyph.y_offset == 0 && glyph.y_advance == 0);

    start_line == 0
        && end_line >= composed.lines.len()
        && composed.lines.len() == 1
        && line.runs.len() == 1
        && !line.has_line_break
        && line.char_start == 0
        && composed.numbering_text.is_none()
        && composed.inline_controls.is_empty()
        && composed.tac_controls.is_empty()
        && composed.footnote_positions.is_empty()
        && composed.tab_extended.is_empty()
        && para.controls.is_empty()
        && para.range_tags.is_empty()
        && para.field_ranges.is_empty()
        && para.orphan_field_ends.is_empty()
        && !para.text.chars().any(|character| {
            matches!(character, '\t' | '\n' | '\r' | '\u{fffc}') || character.is_control()
        })
        && para.text == run.text
        && !run.text.is_empty()
        && run.display_text.is_none()
        && run.char_overlap.is_none()
        && run.footnote_marker.is_none()
        && alignment == Alignment::Left
        && para_border_fill_id == 0
        && style_surface_supported
        && para_style_supported
        && outcome.lines.len() == 1
        && final_line.scalar_start == 0
        && final_line.scalar_end == scalar_count
        && final_line.target_runs.len() == 1
        && target.scalar_start == 0
        && target.scalar_end == scalar_count
        && target.measurement.code_point_count == scalar_count
        && target.measurement.registry_generation == context.registry_generation()
        && raw_vertical_positioning_is_zero
}

/// Mapping, exact-source certification, page attach가 모두 성공한 뒤에만
/// measurement advance를 반환한다. None이면 호출자는 K1/K0 legacy 경로를 그대로 탄다.
fn attach_horizontal_shaping_initial_lane(
    tree: &mut PageLayoutContext,
    composed: &ComposedParagraph,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    run: &ComposedTextRun,
    node_id: NodeId,
    scalar_start: usize,
    origin_x_px: f64,
) -> Option<f64> {
    let outcome = composed.horizontal_shaping.as_ref()?;
    let context = styles.horizontal_shaping_context.as_ref()?;
    let mapped = crate::renderer::shaping_composition::map_horizontal_shaping_emitted_run(
        outcome,
        crate::renderer::shaping_composition::HorizontalShapingEmittedRunCandidate {
            node_id,
            paragraph_text: &para.text,
            emitted_text: &run.text,
            scalar_start,
            origin_x_px,
            layout_positions_present: false,
            display_projection_present: false,
            horizontal_ltr_bidi0: true,
            has_field_or_note_split: false,
            has_char_overlap: false,
            has_border_or_background: false,
            has_decoration: false,
        },
    )
    .ok()?;
    let decision = crate::renderer::shaping_composition::certify_horizontal_shaping_mapped_run(
        context, &mapped,
    )
    .ok()?;
    tree.attach_horizontal_shaping_sidecar(mapped.node_id, mapped.range, decision)
        .ok()?;
    Some(mapped.bbox_width_px)
}

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

/// 인라인으로 이미 분류된 TAC 표의 줄바꿈 여부만 판단한다.
///
/// 인라인 분류 자체는 상류 게이트 `height_measurer::is_tac_table_inline_in_para`
/// (앵커 양쪽 실제 텍스트 요구, 더 엄격)가 담당하므로, 여기서는 위치·폭 조건만 본다.
fn should_wrap_middle_anchored_table(
    control_position: Option<usize>,
    text_len: usize,
    occupied_width: f64,
    table_footprint: f64,
    line_width: f64,
) -> bool {
    // [#4370] 끝 앵커(position == text_len)도 포함한다 — 본문 텍스트 뒤에 붙은
    // tac 표가 남은 줄 폭에 안 들어가면 페이지 우측 밖으로 방출되던 결함.
    control_position.is_some_and(|position| position > 0 && position <= text_len)
        && occupied_width > 1.0
        && occupied_width + table_footprint > line_width + 0.5
}

/// 선행 inline TAC 표의 Bottom caption이 첫 저장 줄을 소유하고, 표 뒤의 첫 visible
/// 문자가 두 번째 저장 줄에서 시작하는 좁은 HWP5 계약인지 판정한다.
///
/// `LINE_SEG.text_start`는 extended control의 8 UTF-16 unit을 포함한다. 따라서 선행
/// 표 하나 뒤의 첫 글자 offset과 `line_segs[1].text_start`가 모두 8이면, visible text
/// 관점의 break index는 0이다. 일반 문단에서 index 0을 허용하면 저장 정보가 불충분한
/// control 문단까지 강제 개행할 수 있으므로 아래 구조가 모두 입증될 때만 보존한다.
fn preserves_stored_first_visible_break_after_bottom_caption_table(para: &Paragraph) -> bool {
    let Some(&first_visible_offset) = para.char_offsets.first() else {
        return false;
    };
    // [#5961] `first_visible_offset` 은 `char_offsets` 값이라 HWP5 축이다. 저장
    // `text_start` 는 출처에 따라 더 짧은 축일 수 있으므로 올려서 견준다.
    if first_visible_offset != 8
        || para.text.is_empty()
        || para.line_segs.first().map(|ls| ls.text_start) != Some(0)
        || para.line_segs.len() < 2
        || para.line_seg_text_start(1) != first_visible_offset
    {
        return false;
    }

    let control_positions = para.control_text_positions();
    let mut leading_controls = para
        .controls
        .iter()
        .enumerate()
        .filter(|(control_index, _)| control_positions.get(*control_index) == Some(&0));
    let Some((_, Control::Table(table))) = leading_controls.next() else {
        return false;
    };
    // first_visible_offset == 8은 저장 stream의 선행 extended control이 정확히 하나라는
    // 뜻이다. IR에서도 owner를 하나로 확정해 다른 co-anchored control에는 확장하지 않는다.
    if leading_controls.next().is_some() {
        return false;
    }

    let has_bottom_caption = table.caption.as_ref().is_some_and(|caption| {
        caption.direction == CaptionDirection::Bottom && !caption.paragraphs.is_empty()
    });
    let segment_width = para
        .line_segs
        .first()
        .map(|ls| ls.segment_width)
        .unwrap_or_default();

    table.common.treat_as_char
        && has_bottom_caption
        && segment_width > 0
        && crate::renderer::height_measurer::is_tac_table_inline_in_para(table, segment_width, para)
}

/// inline TAC 문단의 저장 `LINE_SEG` 시작점을 visible character index로 변환한다.
///
/// 보통 index 0은 실질적인 개행이 아니므로 제외한다. 단,
/// [`preserves_stored_first_visible_break_after_bottom_caption_table`]가 소유권을 증명하면
/// 두 번째 저장 줄의 index 0만 보존한다.
pub(super) fn inline_table_stored_line_break_char_indices(para: &Paragraph) -> Vec<usize> {
    if para.line_segs.len() <= 1 || para.char_offsets.is_empty() {
        return Vec::new();
    }

    let text_len = para.text.chars().count();
    let preserves_first_visible_break =
        preserves_stored_first_visible_break_after_bottom_caption_table(para);
    let mut indices = Vec::new();
    for (line_index, _line_seg) in para.line_segs.iter().enumerate().skip(1) {
        // [#5961] `char_offsets` 는 HWP5 축이므로 저장 `text_start` 를 같은 자로 올린다.
        let seg_start = para.line_seg_text_start(line_index);
        let char_idx = para
            .char_offsets
            .iter()
            .position(|&offset| offset >= seg_start)
            .unwrap_or(text_len);
        let is_owned_first_visible_break =
            line_index == 1 && char_idx == 0 && preserves_first_visible_break;
        if (char_idx > 0 || is_owned_first_visible_break)
            && char_idx <= text_len
            && indices
                .last()
                .map(|&previous| char_idx > previous)
                .unwrap_or(true)
        {
            indices.push(char_idx);
        }
    }
    indices
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

/// 저장 LINE_SEG 없는 실제 빈 문단의 한컴 줄 metrics를 복원한다.
///
/// `compose_paragraph()` 는 렌더러 내부 안내용 400HU 줄을 남기지만, HWP5 원본의
/// 빈 문단 높이는 그 값이 아니라 글자 모양과 ParaShape 줄간격에서 결정된다.
/// HWP3 변환본만 기존 page-count 계약을 위해 작은 글꼴 cap을 유지한다.
fn empty_no_lineseg_paragraph_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
    hwp3_legacy_caps: bool,
    dpi: f64,
) -> Option<(f64, f64, f64)> {
    if !para.text.trim().is_empty()
        || !para.controls.is_empty()
        || !para.line_segs.is_empty()
        || para.char_count == 0
    {
        return None;
    }
    let char_shape_id = para
        .char_shape_id_at(0)
        .or_else(|| para.char_shapes.first().map(|shape| shape.char_shape_id))?
        as usize;
    let char_style = styles.char_styles.get(char_shape_id)?;
    let font_size = char_style.font_size;
    if font_size <= 0.0 {
        return None;
    }
    if hwp3_legacy_caps {
        let small_empty_para_max_font = hwpunit_to_px(1000, dpi);
        if font_size > small_empty_para_max_font + 0.1 {
            return None;
        }
        let meaningful_empty_para_min_font = hwpunit_to_px(800, dpi);
        if !char_style.bold && font_size < meaningful_empty_para_min_font - 0.1 {
            return None;
        }
    }
    let line_spacing = para_style.map(|style| style.line_spacing).unwrap_or(160.0);
    let line_spacing_type = para_style
        .map(|style| style.line_spacing_type)
        .unwrap_or(LineSpacingType::Percent);
    let (line_height, line_spacing_px) = crate::renderer::corrected_line_metrics(
        0.0,
        0.0,
        font_size,
        line_spacing_type,
        line_spacing,
    );
    Some((line_height, line_spacing_px, font_size))
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

fn is_treat_as_char_equation_control(ctrl: Option<&Control>) -> bool {
    matches!(ctrl, Some(Control::Equation(eq)) if eq.common.treat_as_char)
}

fn is_caption_cell_context(cell_ctx: Option<&CellContext>) -> bool {
    cell_ctx
        .and_then(|ctx| ctx.path.last())
        .is_some_and(|entry| entry.cell_index == CAPTION_CELL_SENTINEL)
}

/// HWP5 원본 LineSeg가 저장한 column-relative 줄 시작점을 일반 본문 줄에 적용한다.
///
/// ParaShape의 margin/indent는 재조판 기본값이고, 원본 LineSeg.column_start는 해당
/// 줄의 확정 좌표다. 다만 cs+sw가 단 너비와 같은 일반 줄에만 적용한다. 그림 어울림,
/// 표 셀, 합성 LineSeg는 각각 별도 좌표계를 사용하므로 caller가 `eligible=false`로
/// 제외해 column_start가 이중 적용되지 않게 한다.
fn authoritative_stored_line_start_px(
    styled_margin_left: f64,
    line_seg: Option<&LineSeg>,
    column_width_hu: i32,
    dpi: f64,
    eligible: bool,
) -> f64 {
    let Some(line_seg) = line_seg else {
        return styled_margin_left;
    };
    let full_width_line = line_seg.column_start > 0
        && line_seg.segment_width > 0
        && line_seg
            .column_start
            .saturating_add(line_seg.segment_width)
            .saturating_sub(column_width_hu)
            .abs()
            <= 200;
    let authoritative = line_seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0;
    if !eligible || !authoritative || !full_width_line {
        return styled_margin_left;
    }

    styled_margin_left.max(hwpunit_to_px(line_seg.column_start, dpi))
}

/// HWP5의 저장 `column_start`를 권위로 해석할 수 있는 출처 경계.
///
/// HWP5-origin HWPX는 컨테이너만 HWPX일 뿐 저장 LINE_SEG는 HWP5 원본의 것이므로
/// 원본 HWP5와 같은 계약을 쓴다. 원본 HWPX까지 넓히면 별도 저장 계약을 침범한다.
fn uses_hwp5_stored_line_start_profile(
    profile: crate::model::provenance::LayoutCompatibilityProfile,
) -> bool {
    profile.hwp5_stored_pagination_layout()
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

/// [#5727] 이 TAC 위치를 **앞선 빈 composed 줄**이 이미 소유하는가.
///
/// 저장 lineseg 가 TAC 개체에 자기 줄을 배정하면(제어문자만 담아 텍스트 범위가
/// 비는 줄) 그 빈 줄과 다음 줄의 `char_start` 가 같은 텍스트 인덱스로 붕괴한다
/// (제어문자는 `text` 에 없고 `char_offsets` 갭으로만 남는다). 이때 다음 줄이
/// 그 TAC 를 다시 집으면 개체가 다음 줄로 끌려 내려가고, 그 줄 텍스트는 개체
/// 폭만큼 오른쪽에서 시작한다 — 156732636 로고 칸 실측: `노동부` 가 저장
/// horzpos=0 인데 +172px(로고 폭)에서 시작. 빈 줄 소유 TAC 는 다음 줄 귀속에서
/// 제외한다.
fn tac_owned_by_prior_empty_line(comp: &ComposedParagraph, line_idx: usize, pos: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    // 현재 줄이 **텍스트 줄**일 때만 적용 — 빈 줄 연쇄(빈 문단 + TAC 여러 개,
    // 59043 p12 실측)는 기존 반복-빈-줄 기제가 소유를 배정하므로 건드리지 않는다.
    if line.char_start != pos || line.runs.is_empty() {
        return false;
    }
    comp.lines
        .get(line_idx - 1)
        .is_some_and(|prev| prev.runs.is_empty() && prev.char_start == pos)
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
    /// [#5727] composed 줄 인덱스 — 경계 TAC 자기-줄 판정에 사용.
    line_idx: usize,
}
/// [#2003] run 방출 루프의 줄-간 캐리오버 묶음 (Copy 스칼라 9종) — 값 전달 + 반환.
#[derive(Clone, Copy)]
struct RunEmitState {
    x: f64,
    y: f64,
    char_offset: usize,
    run_char_pos: usize,
    inline_tab_cursor_render: usize,
    pending_right_tab_render: Option<(f64, u8, u8)>,
    pending_right_leader_digit_render: bool,
    current_line_reserved_tac_picture_height: Option<f64>,
}

/// [#2067] TAC 그림 배치의 줄-스코프 스칼라 입력 묶음.
#[derive(Clone, Copy)]
struct TacPictureLineVars {
    run_char_pos: usize,
    x: f64,
    y: f64,
    baseline: f64,
    raw_lh: f64,
    section_index: usize,
    para_index: usize,
}

/// [#2067] 빈 runs 줄 TAC 수식 인라인 배치의 줄-스코프 스칼라 입력 묶음.
#[derive(Clone, Copy)]
struct EquationTacLineVars {
    line_idx: usize,
    /// 이 문단에서 배치하는 마지막 줄 인덱스 상한 (원본: end)
    line_end: usize,
    alignment: crate::model::style::Alignment,
    available_width: f64,
    margin_left: f64,
    indent: f64,
    effective_col_x: f64,
    y: f64,
    baseline: f64,
    line_height: f64,
    line_spacing_px: f64,
    col_area_y: f64,
    col_bottom: f64,
    /// 이 줄 끝의 문서 char 좌표 (원본: char_offset)
    line_char_end: usize,
    is_last_line_of_para: bool,
    defer_empty_line_control_marker: bool,
    equation_tac_extra_rows: usize,
    /// [Task #1472] hwp3 변환본 indent scale 배율 — 소스분기는 caller 유지.
    hwp3_indent_scale: f64,
    section_index: usize,
    para_index: usize,
}

/// [#2003] run 방출 루프의 줄-스코프 읽기 스칼라 묶음.
#[derive(Clone, Copy)]
struct RunEmitVars {
    baseline: f64,
    raw_lh: f64,
    alignment: crate::model::style::Alignment,
    auto_tab_right: bool,
    available_width: f64,
    effective_margin_left: f64,
    end: usize,
    extra_char_sp: f64,
    extra_dash_sp: f64,
    extra_word_sp: f64,
    has_tabs: bool,
    horizontal_shaping_initial_lane: bool,
    is_last_line_of_para: bool,
    line_height: f64,
    line_idx: usize,
    line_spacing_px: f64,
    max_fs: f64,
    runs_all_whitespace: bool,
    renders_synthetic_wrap_trailing_space: bool,
    start_line: usize,
    tab_width: f64,
    section_index: usize,
    para_index: usize,
}

// [#2510] 종전 `receipt_date_stamp_shift_px`(#2020) 제거 — 접수증 ㊞ 를
// "한컴 공백 0.42em" 가정으로 −21px 이동시키던 보정. 실측(무신축 래더)
// space=0.505em 균일이라 가정이 허구였고, 실체는 구 HY 테이블의 글자
// 과대폭(+20px)을 도장 위치에서만 상쇄하던 것 — #2430 실측 메트릭 교정으로
// 불필요·유해(㊞ 오라클 −15px)해져 제거. 제거+교정 시 ㊞ = 오라클 +6.2px,
// issue_2020 도장 정렬 핀 4/4 유지 (PR #2510 코멘트 5017316669 실측).

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
        .filter(|(pos, _, _)| {
            char_pos_in_line(*pos, start, end)
                // [#5727] 앞선 빈 줄(개체 자기 줄)이 소유한 경계 TAC 는 제외
                && !tac_owned_by_prior_empty_line(comp, line_idx, *pos)
        })
        .collect()
}

/// 정렬 폭 산정에 사용할 줄 단위 TAC 집합.
///
/// 기본 줄 범위는 [`tac_offsets_for_line`]과 동일하게 엄격한 반열림 구간이다. 다만
/// 실제 렌더 경로(`emit_line_runs`)는 문단 마지막 run 또는 명시 줄바꿈의 마지막 run
/// 끝에 놓인 TAC를 현재 줄에 방출한다. 그 TAC를 폭 계산에서 제외하면 Center/Right
/// 정렬의 시작점만 그림 폭만큼 어긋난다 (#3257).
///
/// 다음 composed line이 정확히 같은 run 끝 위치에서 시작하면 그 TAC는 다음 줄 선두다.
/// #1219의 줄 경계 수식 중복·폭 오포함을 막기 위해 이 경우에는 추가하지 않는다.
/// [#5820 → Issue #6173] 오른쪽/가운데 정렬이 폭에서 제외할 **줄 말미 공백** 폭 (px).
///
/// 말미 공백이 서로 다른 글꼴·글자 크기의 run 경계를 넘을 수 있으므로, 전체 공백을
/// 마지막 run 의 style 로 재측정하지 않고 뒤에서부터 각 run 의 실제 style 폭을 더한다.
///
/// **[Issue #6173] 자리차지(TAC) 개체 앞 공백은 말미 공백이 아니다.** 인라인 개체는
/// run 을 쪼개지 않고 run 안 char 위치에 놓이므로 `[그림A][공백4][그림B][공백2]` 가
/// 공백 6칸짜리 run **하나**로 합성된다. run 만 보고 뒤에서 공백을 세면 그림 사이 4칸까지
/// 말미로 걷어내 오른쪽 앵커가 그만큼(26.7px) 우측으로 밀리고, 마지막 그림이 글상자
/// 우단을 넘어 잘린다(156740495 2쪽). 줄의 **마지막 개체 위치 뒤** 공백만 말미다.
///
/// - `last_inline_object_pos`: 이 줄이 소유한 TAC 개체 중 마지막 것의 절대 char 위치.
///   개체가 없으면 `None` — 종전 동작 그대로.
/// - `stop_on_underline`: 밑줄 친 말미 공백에서 멈춘다(가운데 정렬 전용 규칙).
pub(crate) fn trailing_space_width_after_last_inline_object(
    line: &ComposedLine,
    last_inline_object_pos: Option<usize>,
    styles: &ResolvedStyleSet,
    stop_on_underline: bool,
) -> f64 {
    let run_chars = |r: &crate::renderer::composer::ComposedTextRun| -> usize {
        if r.char_overlap.is_some() {
            let chars: Vec<char> = r.text.chars().collect();
            crate::renderer::composer::char_overlap_advance_units(&chars)
        } else {
            r.text.chars().count()
        }
    };
    let mut run_end_pos = line.char_start + line.runs.iter().map(run_chars).sum::<usize>();
    let mut width = 0.0;
    for run in line.runs.iter().rev() {
        let run_char_count = run_chars(run);
        let run_start_pos = run_end_pos.saturating_sub(run_char_count);
        let mut trailing_spaces = run.text.chars().rev().take_while(|c| *c == ' ').count();
        if let Some(obj_pos) = last_inline_object_pos {
            // 마지막 개체 뒤로 자른다 — 개체 자리 이전 공백은 콘텐츠다.
            let floor = obj_pos.max(run_start_pos);
            trailing_spaces = trailing_spaces.min(run_end_pos.saturating_sub(floor));
        }
        if trailing_spaces == 0 {
            break;
        }
        let ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
        if stop_on_underline && ts.underline != crate::renderer::UnderlineType::None {
            break;
        }
        width += estimate_text_width(&" ".repeat(trailing_spaces), &ts);
        if trailing_spaces != run_char_count {
            break;
        }
        run_end_pos = run_start_pos;
    }
    width
}

fn tac_offsets_for_line_width(
    comp: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> Vec<(usize, f64, usize)> {
    let mut offsets = tac_offsets_for_line(comp, tac_offsets_px, line_idx);
    let Some(line) = comp.lines.get(line_idx) else {
        return offsets;
    };
    if line.runs.is_empty() {
        return offsets;
    }

    let run_end = line.char_start
        + line
            .runs
            .iter()
            .map(|run| run.text.chars().count())
            .sum::<usize>();
    let is_last_line = comp.lines.get(line_idx + 1).is_none();
    let next_starts_at_run_end = comp
        .lines
        .get(line_idx + 1)
        .is_some_and(|next| next.char_start == run_end);
    let emits_trailing_tac = (is_last_line || line.has_line_break) && !next_starts_at_run_end;
    if !emits_trailing_tac {
        return offsets;
    }

    for offset @ (pos, _, _) in tac_offsets_px.iter().copied() {
        if pos == run_end && !offsets.iter().any(|(_, _, ci)| *ci == offset.2) {
            offsets.push(offset);
        }
    }
    offsets
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
    // TAC가 빈 줄보다 적으면 앞 줄부터 하나씩만 귀속하고, 나머지 guide 줄에는
    // 이미 귀속한 개체를 되풀이해 그리지 않는다. TAC 수와 빈 줄 수가 정확히
    // 같은 기존 사례도 같은 순서 배정으로 보존된다.
    if !line_tac_sequence.is_empty() && line_tac_sequence.len() <= repeated_empty_line_count {
        // 후보가 모자란 뒤쪽 guide 줄도 `Some(vec![])`으로 명시해야 한다. `None`을
        // 반환하면 호출자가 기본 줄-범위 집합으로 되돌아가 같은 TAC를 재배정한다.
        Some(
            line_tac_sequence
                .get(line_ordinal)
                .copied()
                .into_iter()
                .collect(),
        )
    } else {
        None
    }
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
                .and_then(|ctrl| crate::renderer::tac_object_flow_height_px(ctrl, dpi))
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

/// [Issue #6179] 오른쪽 탭 뒤에 오는 **자리차지(TAC) 개체**까지 포함한 정렬 블록 폭.
///
/// `auto_tab_right` 오른쪽 탭은 "탭 뒤 블록의 **오른쪽 변**을 우단에 맞춘다"는 뜻이고,
/// 그 되밀기 폭은 `text_measurement` 가 탭 뒤 **글자**만 재서 구한다. 그런데 run 은
/// TAC 개체 위치에서 조각으로 쪼개져 측정되므로, 탭 바로 뒤가 개체면 측정 대상 조각에
/// 남는 글자가 없어 되밀기 폭이 0 이 된다 → 개체의 **왼쪽** 변이 우단에 놓여, 개체는
/// 정확히 제 폭만큼 우측(용지 밖)으로 밀린다.
///
/// 여기서 탭 뒤 잔여 글자 폭 + 탭 뒤 TAC 개체 폭을 합해
/// `right_tab_block_width_override` 로 주입한다. 탭 뒤에 또 탭이 있으면
/// (`has_more_tabs_after`) 측정 쪽이 override 를 쓰지 않으므로 `None` 을 돌려준다.
///
/// - `run_chars`: run 전체 문자열 (조각이 아니라 run 단위 — 탭 뒤 잔여가 다음 조각에
///   있을 수 있다)
/// - `tab_rel`: run 안 마지막 탭의 문자 인덱스
/// - `run_tacs`: run 안 TAC 목록 `(rel_pos, width_px, control_index)`
fn right_tab_block_width_with_tac(
    run_chars: &[char],
    tab_rel: usize,
    run_tacs: &[(usize, f64, usize)],
    style: &TextStyle,
) -> Option<f64> {
    if run_chars[tab_rel + 1..].contains(&'\t') {
        return None;
    }
    let tac_w: f64 = run_tacs
        .iter()
        .filter(|(rel, _, _)| *rel > tab_rel)
        .map(|(_, w, _)| *w)
        .sum();
    if tac_w <= 0.0 {
        return None;
    }
    let tail: String = run_chars[tab_rel + 1..].iter().collect();
    let mut ts = style.clone();
    ts.right_tab_block_width_override = None;
    Some(estimate_text_width(&tail, &ts) + tac_w)
}

/// [Task #2067] 정렬(양쪽/배분/나눔)·오버플로우·셀 underflow 에 따른 여분 간격 계산.
/// 반환 = (extra_word_sp, extra_char_sp, extra_dash_sp). Task #352 dash leader 분배 포함.
#[allow(clippy::too_many_arguments)]
fn compute_line_extra_spacing(
    comp_line: &ComposedLine,
    styles: &ResolvedStyleSet,
    alignment: Alignment,
    in_cell: bool,
    needs_justify: bool,
    justify_spaces_only: bool,
    needs_distribute: bool,
    has_tabs: bool,
    renders_synthetic_wrap_trailing_space: bool,
    suppress_cell_overflow_spacing: bool,
    total_char_count: usize,
    total_text_width: f64,
    available_width: f64,
    tab_width: f64,
) -> (f64, f64, f64) {
    // 음수 자간은 마지막 글자의 advance도 줄이지만 실제 glyph 잉크 폭은 줄이지 않는다.
    // 나눔정렬에서 advance만 셀 끝에 맞추면 정상 폭으로 그린 마지막 glyph가 clip을
    // 넘어가므로, 마지막 가시 글자의 음수 자간만 시각 점유 폭에 되돌린다.
    let trailing_glyph_ink_overhang = || -> f64 {
        for run in comp_line.runs.iter().rev() {
            if let Some(last_visible) = run.text.chars().rev().find(|c| *c != ' ') {
                if last_visible == '\t' || last_visible == '\u{FFFC}' {
                    return 0.0;
                }
                let mut with_spacing =
                    resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                with_spacing.default_tab_width = tab_width;
                if with_spacing.letter_spacing >= 0.0 {
                    return 0.0;
                }
                let glyph = last_visible.to_string();
                let spaced_width = estimate_text_width(&glyph, &with_spacing);
                with_spacing.letter_spacing = 0.0;
                let ink_advance = estimate_text_width(&glyph, &with_spacing);
                return (ink_advance - spaced_width).max(0.0);
            }
        }
        0.0
    };

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

    // [#5830] 양쪽정렬 배분 대상이 아닌 줄(문단 마지막 줄·강제 줄바꿈 줄)의 dash leader.
    //
    // 종전에는 이 줄들에서 슬랙 자체가 계산되지 않아 `char_width_decision` 의 leader
    // 클램프 `min(자연폭, font_size * 0.3)` 에 머물렀다 — 한글 2022 정본 대비 폭 절반.
    //
    // 정본(86712 규제영향분석서 p34·p35, PDF 글리프 원점 실측)의 마지막 줄 규칙:
    //   - 여백이 충분하면 dash 는 **자연 폭**으로 그린다 (p35 10자·18자 런 = 8.00pt
    //     = 0.571em, 오른쪽 여백에 닿지 않고 끝난다 — 무한 신장이 아니다).
    //   - 여백이 그보다 좁으면 **여백까지만** 좁힌다 (p34 10자 런 = 7.00pt = 0.499em,
    //     끝점이 정확히 여백 x≈530pt).
    // 즉 마지막 줄에서는 leader 클램프를 **자연 폭 한도 안에서 슬랙만큼** 되돌린다.
    // needs_justify 줄의 기존 탄력 흡수(Task #352, 여백까지 확장)는 그대로다.
    //
    // 정렬이 여백까지 채우는 종류(Justify·Split)일 때만 연다 — 왼쪽/가운데 정렬의
    // 짧은 dash 는 저자가 의도한 길이일 수 있다.
    let last_line_leader_fill = if !needs_justify
        && matches!(alignment, Alignment::Justify | Alignment::Split)
    {
        let all_chars: Vec<char> = comp_line.runs.iter().flat_map(|r| r.text.chars()).collect();
        let trailing_spaces = all_chars.iter().rev().take_while(|c| **c == ' ').count();
        let visible_count = all_chars.len() - trailing_spaces;
        let leader_dashes = count_dash_leaders(&all_chars[..visible_count]);
        if leader_dashes > 0 {
            // 클램프가 깎아낸 폭 = 자연 advance − min(자연, 0.3em). 단독 '-' 는 3+ 연속
            // leader 가 아니므로 estimate_text_width 가 클램프 없는 자연 폭을 돌려준다.
            let per_dash_restore = comp_line
                .runs
                .iter()
                .find(|r| {
                    let chars: Vec<char> = r.text.chars().collect();
                    (0..chars.len()).any(|i| {
                        chars[i] == '-' && chars[i..].iter().take_while(|c| **c == '-').count() >= 3
                    })
                })
                .map(|r| {
                    let mut ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                    ts.default_tab_width = tab_width;
                    let natural = estimate_text_width("-", &ts);
                    (natural - natural.min(ts.font_size * 0.3)).max(0.0)
                })
                .unwrap_or(0.0);
            let trailing_width = if trailing_spaces > 0 {
                if let Some(last_run) = comp_line.runs.last() {
                    let mut ts =
                        resolved_to_text_style(styles, last_run.char_style_id, last_run.lang_index);
                    ts.default_tab_width = tab_width;
                    estimate_text_width(&" ".repeat(trailing_spaces), &ts)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let slack = available_width - (total_text_width - trailing_width);
            // 슬랙이 없으면(이미 꽉 찬 줄) 아래 기존 분기(오버플로우 압축 등)로 흘린다.
            let extra = (slack / leader_dashes as f64).min(per_dash_restore);
            (extra > 0.0).then_some(extra)
        } else {
            None
        }
    } else {
        None
    };

    if needs_justify {
        // 양쪽 정렬: 후행 공백 제외한 내부 공백에 분배.
        //
        // [#5899] 공백은 **그려지는 텍스트**로 센다. `extra_word_spacing` 은
        // text_measurement 가 표시 텍스트의 공백마다 붙이므로, 모델 텍스트(`run.text`)
        // 로 세면 분모(내부 공백 수)와 실제 적용 대상이 어긋난다. 머리말/꼬리말
        // 쪽번호 필드는 #3216 규약대로 모델 1자(공백 placeholder)를 유지하고
        // `display_text` 만 번호로 바꾸므로, 모델로 세면 `… Inc.` + 공백 75개로
        // **끝나는 줄**로 보여 슬랙이 내부 공백 2개에만 나뉜다. 그 여분(262.9px)이
        // 표시 텍스트의 공백 76개 전부에 붙어 쪽번호가 종이 밖 x≈20,163px 로
        // 밀려났다. 폭(`total_text_width`)·글자수(`total_char_count`)는 이미 표시
        // 텍스트 기준이라 여기만 축이 달랐다.
        let all_chars: Vec<char> = comp_line
            .runs
            .iter()
            .flat_map(|r| effective_text_for_metrics(r).chars())
            .collect();
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
                    let mut ts =
                        resolved_to_text_style(styles, last_run.char_style_id, last_run.lang_index);
                    ts.default_tab_width = tab_width;
                    let trailing_str: String = " ".repeat(trailing_spaces);
                    estimate_text_width(&trailing_str, &ts)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let split_ink_overhang = if alignment == Alignment::Split {
                trailing_glyph_ink_overhang()
            } else {
                0.0
            };
            // A fresh soft-wrap consumes the separator before starting the next
            // row, but LineSeg can encode only the next row's start. Composition
            // therefore leaves that separator at the end of this row. The run
            // painter advances every preserved space, so its distribution must
            // account for that slot and its natural width; otherwise a corrected
            // break immediately before a word can push the trailing separator
            // beyond the line box (#4956).
            let rendered_space_slots = if renders_synthetic_wrap_trailing_space {
                interior_spaces + trailing_spaces
            } else {
                interior_spaces
            };
            let effective_used = if renders_synthetic_wrap_trailing_space {
                total_text_width + split_ink_overhang
            } else {
                total_text_width - trailing_width + split_ink_overhang
            };
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
                let raw_ews = slack / rendered_space_slots as f64;
                let space_base_w = estimate_text_width(
                    " ",
                    &resolved_to_text_style(
                        styles,
                        comp_line.runs[0].char_style_id,
                        comp_line.runs[0].lang_index,
                    ),
                );
                let min_ews = -(space_base_w * 0.5);
                let ews = raw_ews.max(min_ews);
                // [Task #2189] 저장 줄바꿈(LINE_SEG) 셀에서 대체 폰트 advance 가 한컴
                // 실폰트보다 넓으면 공백 -50% 클램프만으로는 잔여 초과가 남아 우측
                // 테두리에서 클리핑된다. 공백-없는 분기와 동일하게 잔여 음수 슬랙을
                // 자간으로 흡수한다 (narrow glyph 역진은 #229 per-char 클램프가 방어).
                let leftover = slack - ews * rendered_space_slots as f64;
                let ecs = if in_cell && leftover < 0.0 && total_char_count > 1 && !has_tabs {
                    let avg_char_w = total_text_width / total_char_count as f64;
                    let min_ecs = -avg_char_w * 0.5;
                    let mut ecs = (leftover / total_char_count as f64).max(min_ecs);
                    // narrow glyph per-char 클램프가 음수 자간 기여 일부를 되돌리므로
                    // 선형 1회 분배로는 부족하다 — underflow 확장과 동일하게 실효 폭
                    // 재측정으로 수렴 반복한다.
                    let measure_with = |ecs: f64| -> f64 {
                        let mut measured = 0.0f64;
                        for r in &comp_line.runs {
                            let mut ts =
                                resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                            ts.default_tab_width = tab_width;
                            ts.extra_word_spacing = ews;
                            ts.extra_char_spacing = ecs;
                            measured += estimate_text_width(&r.text, &ts);
                        }
                        if trailing_spaces > 0 {
                            if let Some(last_run) = comp_line.runs.last() {
                                let mut ts = resolved_to_text_style(
                                    styles,
                                    last_run.char_style_id,
                                    last_run.lang_index,
                                );
                                ts.default_tab_width = tab_width;
                                ts.extra_word_spacing = ews;
                                ts.extra_char_spacing = ecs;
                                measured -= estimate_text_width(&" ".repeat(trailing_spaces), &ts);
                            }
                        }
                        measured
                    };
                    for _ in 0..3 {
                        let delta = available_width - measure_with(ecs);
                        if delta.abs() < 0.5 {
                            break;
                        }
                        ecs = (ecs + delta / total_char_count as f64).max(min_ecs);
                    }
                    ecs.min(0.0)
                } else {
                    0.0
                };
                (ews, ecs, 0.0)
            }
        } else if total_char_count > 1 {
            // 양쪽 정렬이지만 공백 없음 (일본어/숫자 등):
            let slack = available_width - total_text_width;
            if justify_spaces_only && slack > 0.0 {
                // [#4516] 머리말/꼬리말 예외로만 justify 된 마지막 줄은 한컴처럼
                // **공백만** 벌린다. 공백 없는 줄(영문 문서번호 등)에 양수 slack 을
                // 자간으로 살포하면 글자가 전체 폭으로 흩어지므로 자연 폭 유지.
                (0.0, 0.0, 0.0)
            } else if leader_dashes > 0 && slack > 0.0 {
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
    } else if let Some(extra_dash) = last_line_leader_fill {
        // [#5830] 마지막 줄·강제 줄바꿈 줄의 dash leader 채움.
        (0.0, 0.0, extra_dash)
    } else if needs_distribute && total_char_count > 1 {
        // [#4657] 배분 정렬: 남는 폭을 글자 **사이**(N-1곳)에 균등 분배.
        // extra_char_spacing 은 각 글자 advance 뒤에 붙으므로 마지막 glyph 의
        // 잉크 오른쪽 끝은 `W + (N-1)·extra` — N 으로 나누면 짧은 줄일수록
        // 마지막 글자가 slack/N 만큼 안쪽으로 밀려 문단마다 오른쪽 끝이
        // 어긋난다(한컴은 줄 길이와 무관하게 오른쪽 끝을 문단 폭에 맞춘다).
        // 말미 공백은 보이는 글자가 아니므로 분배 대상과 기준 폭에서 제외한다.
        let trailing_spaces = comp_line
            .runs
            .iter()
            .rev()
            .flat_map(|r| r.text.chars().rev())
            .take_while(|c| *c == ' ')
            .count();
        let visible_count = total_char_count.saturating_sub(trailing_spaces);
        if visible_count <= 1 {
            (0.0, 0.0, 0.0)
        } else {
            let trailing_width = if trailing_spaces > 0 {
                if let Some(last_run) = comp_line.runs.last() {
                    let mut ts =
                        resolved_to_text_style(styles, last_run.char_style_id, last_run.lang_index);
                    ts.default_tab_width = tab_width;
                    estimate_text_width(&" ".repeat(trailing_spaces), &ts)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let visible_width = total_text_width - trailing_width;
            let raw = (available_width - visible_width) / (visible_count - 1) as f64;
            if suppress_cell_overflow_spacing && raw < 0.0 {
                (0.0, 0.0, 0.0)
            } else {
                let avg_char_w = visible_width / visible_count as f64;
                let min_sp = -avg_char_w * 0.5;
                (0.0, raw.max(min_sp), 0.0)
            }
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
    } else if in_cell
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
                    let mut ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
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
    }
}

/// `한글 97 안내문` 머리말의 한컴 회사명 PUA 여섯 글자와 뒤따르는 inline
/// logo 그림은, HWPX상 `DISTRIBUTE_SPACE` 문단 하나로 저장돼 있다.
///
/// 한컴은 회사명 내부 글자에는 나눔 자간을 넣지 않고, 그 뒤 공백 하나로 logo를
/// 오른쪽에 보낸다. 일반 `DISTRIBUTE_SPACE` 규칙(모든 글자에 자간 분배)을 그대로
/// 적용하면 PUA가 공개 글꼴에서 tofu로 보일 뿐 아니라, 표준 한글로 투영한 뒤에도
/// `한 글 과 컴 퓨 터`처럼 흩어진다. header/footer 내부 문단은 원문 문단 번호를
/// 재사용하므로 호출부의 header 플래그에 의존하지 않고, 확인된 **완전한** 원문
/// 시퀀스와 정렬값만으로 좁게 판별한다.
fn is_hancom_company_pua_logo_line(comp_line: &ComposedLine, alignment: Alignment) -> bool {
    // OWPML `DISTRIBUTE_SPACE`는 parser에서 `Split`(공백에만 나눔)으로
    // 보존한다. `Distribute`는 글자마다 나눔인 별도 값이다.
    if alignment != Alignment::Split {
        return false;
    }

    let raw: String = comp_line.runs.iter().map(|run| run.text.as_str()).collect();
    let company_pua = "\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4}";
    raw == format!("{company_pua} ")
}

/// 문단 정렬이 현재 줄의 공백 폭을 끝까지 배분해야 하는지 판정한다.
///
/// `Justify`는 마지막 줄과 강제 줄바꿈 줄을 제외하지만, HWP5 `Split`
/// (HWPX `DISTRIBUTE_SPACE`, 한컴 UI의 나눔 정렬)은 문단의 마지막 줄까지
/// 공백에 배분한다. 강제 줄바꿈 줄의 기존 억제 동작은 유지한다.
fn needs_word_distribution(
    alignment: Alignment,
    is_last_line_of_para: bool,
    is_header_footer_para: bool,
    has_forced_break: bool,
) -> bool {
    match alignment {
        Alignment::Split => !has_forced_break,
        Alignment::Justify => (!is_last_line_of_para || is_header_footer_para) && !has_forced_break,
        _ => false,
    }
}

/// [Task #2067] 조판부호 모드의 인라인 컨트롤 마커 라벨 수집 — (논리 위치, 라벨).
fn collect_shape_marker_labels(show_ctrl: bool, para: Option<&Paragraph>) -> Vec<(usize, String)> {
    if show_ctrl {
        if let Some(ref pa) = para {
            let ctrl_positions = pa.logical_control_positions();
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
                        Control::PageNumberPos(_) => Some((pos, "[쪽 번호 위치]".to_string())),
                        Control::Header(h) => {
                            let apply = match h.apply_to {
                                crate::model::header_footer::HeaderFooterApply::Both => "양 쪽",
                                crate::model::header_footer::HeaderFooterApply::Even => "짝수 쪽",
                                crate::model::header_footer::HeaderFooterApply::Odd => "홀수 쪽",
                            };
                            Some((pos, format!("[머리말({})]", apply)))
                        }
                        Control::Footer(f) => {
                            let apply = match f.apply_to {
                                crate::model::header_footer::HeaderFooterApply::Both => "양 쪽",
                                crate::model::header_footer::HeaderFooterApply::Even => "짝수 쪽",
                                crate::model::header_footer::HeaderFooterApply::Odd => "홀수 쪽",
                            };
                            Some((pos, format!("[꼬리말({})]", apply)))
                        }
                        Control::Footnote(_) => Some((pos, "[각주]".to_string())),
                        Control::Endnote(_) => Some((pos, "[미주]".to_string())),
                        Control::NewNumber(_) => Some((pos, "[새 번호]".to_string())),
                        Control::Bookmark(bm) => Some((pos, format!("[책갈피:{}]", bm.name))),
                        _ => None,
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

impl LayoutEngine {
    /// [#5729] 저장 줄 밴드가 정확히 `om_top + 선언높이 + om_bottom` 인 TAC 표는
    /// 한글이 표 상단을 **줄 상단 + om_top** 에 앉힌다 (156505870 4표 실측:
    /// 밴드 5195=283+4629+283 등 전부 일치). 종전 baseline-하단 정렬은 측정
    /// 높이 흔들림이 그대로 y 오차가 되어 이중 괘선 사이가 4.3px 벌어지고
    /// 글자가 위 괘선을 뚫었다. 밴드 증거가 없으면 None(종전 경로).
    fn tac_table_stored_outer_band_top(
        &self,
        para: &Paragraph,
        tbl: &crate::model::table::Table,
        current_y: f64,
    ) -> Option<f64> {
        if !Self::tac_stored_band_is_outer_box(para, tbl) {
            return None;
        }
        Some(current_y + hwpunit_to_px(tbl.outer_margin_top as i32, self.dpi))
    }

    /// [#5729] 호스트 줄의 저장 밴드가 정확히 `om_top + 선언높이 + om_bottom`
    /// 인가 — 참이면 한글은 표 상단을 줄 상단 + om_top 에 앉힌다.
    pub(crate) fn tac_stored_band_is_outer_box(
        para: &Paragraph,
        tbl: &crate::model::table::Table,
    ) -> bool {
        let om_top_hu = i64::from(tbl.outer_margin_top);
        let om_bottom_hu = i64::from(tbl.outer_margin_bottom);
        if om_top_hu <= 0 || om_bottom_hu <= 0 {
            return false;
        }
        let declared = i64::from(tbl.common.height.min(i32::MAX as u32));
        if declared <= 0 {
            return false;
        }
        let Some(ls) = para.line_segs.first() else {
            return false;
        };
        if ls.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 {
            return false;
        }
        (i64::from(ls.line_height) - (om_top_hu + declared + om_bottom_hu)).abs() <= 8
    }

    pub(crate) fn layout_inline_table_paragraph(
        &self,
        tree: &mut PageLayoutContext,
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
        // [Issue #3396] 한글은 TAC 표를 "outMargin 포함 폭의 문자"로 배치한다 —
        // 정렬/전진 폭에는 outMargin 좌/우가 포함되고, 괘선(테두리)은
        // pen + outMargin.left 에 그려진다 (오라클 실측: 156678235 p1/p5/p7).
        let table_om_px: Vec<(f64, f64)> = inline_tables
            .iter()
            .map(|(_, t)| {
                (
                    hwpunit_to_px(t.outer_margin_left as i32, self.dpi),
                    hwpunit_to_px(t.outer_margin_right as i32, self.dpi),
                )
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

        // 5. 총 폭과 정렬 계산 (TAC 표는 outMargin 좌/우 포함 폭 — Issue #3396)
        let total_width: f64 = seg_widths.iter().sum::<f64>()
            + table_widths.iter().sum::<f64>()
            + table_om_px.iter().map(|(l, r)| l + r).sum::<f64>();
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
        //
        // [#6078] 단, 그 순서는 **가정이 아니라 조회**여야 한다. HWP3 국세청 납세담보
        // 확인서는 반대로 저장한다 — `ls[0] lh=1300`(제목 텍스트 줄), `ls[1] lh=67616`
        // (표 줄). 0/1 을 고정하면 `￼` 자리표시 조각이 **표 줄의 baseline**(57473HU
        // =766.3px)을 텍스트 줄 높이로 받아 문단 바닥을 표 높이만큼 한 번 더 밀고,
        // 뒤 문단(용지 규격 줄)이 용지 밖(+827px)으로 나가 소실된다. 표가 실제로 속한
        // seg 는 `control_line_seg_index` 가 안다.
        //
        // 판별은 **기하**로 한다 — 표를 담을 수 있는 줄 높이를 가진 seg 가 표 줄이다.
        // (`control_line_seg_index` 는 선행 컨트롤에서 0 대신 1 을 돌려준다: 컨트롤이
        // 문자 0 이고 첫 글자 offset 이 8 이면 `p >= start_txt` 가 0 >= 0 으로 참이 된다.
        // 정책연구용역사업 중간진도보고서 pi=428 이 그 형상 — 표 h=13956 이 ls[0]
        // lh=16086 에 담기는데 seg 1(lh=1000)을 표 줄로 오인해 되레 깨진다.)
        let table_seg_index = inline_tables
            .first()
            .and_then(|(_, tbl)| {
                let need = tbl.common.height;
                para.line_segs
                    .iter()
                    .enumerate()
                    .filter(|(_, seg)| seg.line_height as u32 >= need)
                    .map(|(idx, _)| idx)
                    .next()
            })
            .unwrap_or(0);
        let text_seg_index = (0..para.line_segs.len()).find(|idx| *idx != table_seg_index);
        let table_seg = para.line_segs.get(table_seg_index);
        let text_seg = text_seg_index.and_then(|idx| para.line_segs.get(idx));
        let line_height = if let Some(ls) = table_seg {
            hwpunit_to_px(ls.line_height, self.dpi)
        } else {
            hwpunit_to_px(400, self.dpi)
        };
        let line_spacing = if let Some(ls) = table_seg {
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
        let baseline_dist = if let Some(ls) = table_seg {
            ensure_min_baseline(
                hwpunit_to_px(ls.baseline_distance, self.dpi),
                para_max_font_size,
            )
        } else {
            line_height * 0.8
        };
        // 텍스트 줄(표 아래) 전용 메트릭: 표 줄이 아닌 seg 가 있으면 사용
        let text_line_baseline = if let Some(ls) = text_seg {
            ensure_min_baseline(
                hwpunit_to_px(ls.baseline_distance, self.dpi),
                para_max_font_size,
            )
        } else {
            baseline_dist
        };
        let text_line_height = if let Some(ls) = text_seg {
            hwpunit_to_px(ls.line_height, self.dpi)
        } else {
            line_height
        };
        let text_line_spacing = if let Some(ls) = text_seg {
            hwpunit_to_px(ls.line_spacing, self.dpi)
        } else {
            line_spacing
        };

        // 7. 가로 배치: 텍스트 세그먼트와 표를 순차 배치
        let right_margin = col_area.x + col_area.width - margin_right;
        let line_start_x = col_area.x + margin_left;
        // 텍스트 줄바꿈 시 줄 높이: line_seg[0]은 표 높이를 포함하므로
        // line_seg[1]이 있으면 사용 (텍스트 줄 높이), 없으면 baseline_dist 기반
        let line_step = if let Some(ls) = text_seg {
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
        let line_break_char_indices = inline_table_stored_line_break_char_indices(para);
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
        let control_positions = para.control_text_positions();

        for (s, e) in &segments {
            // 텍스트 세그먼트 렌더링 (줄바꿈 지원)
            if *s < *e {
                let seg_text: String = text_chars[*s..*e].iter().collect();
                if !seg_text.is_empty() {
                    // 문자별로 처리하며 줄바꿈 판단
                    let run_start = *s;
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
                                        layout_positions: None,
                                        display_text: None,
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
                                    layout_positions: None,
                                    display_text: None,
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
                                layout_positions: None,
                                display_text: None,
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
                let table_footprint = tw.max(
                    hwpunit_to_px(tbl.common.width as i32, self.dpi)
                        + hwpunit_to_px(
                            tbl.outer_margin_left as i32 + tbl.outer_margin_right as i32,
                            self.dpi,
                        ),
                );
                let table_wrapped = should_wrap_middle_anchored_table(
                    control_positions.get(*ctrl_idx).copied(),
                    text_chars.len(),
                    inline_x - line_start_x,
                    table_footprint,
                    right_margin - line_start_x,
                );
                if table_wrapped {
                    current_y += line_step;
                    inline_x = line_start_x;
                }
                let (om_left, om_right) = table_om_px[table_idx];
                let om_bottom = hwpunit_to_px(tbl.outer_margin_bottom as i32, self.dpi);
                let tbl_y = self
                    .tac_table_stored_outer_band_top(para, tbl, current_y)
                    .unwrap_or_else(|| {
                        (current_y + baseline_dist + om_bottom - tbl_h).max(current_y)
                    });

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
                    Some(inline_x + om_left),
                    None,
                    table_para_y,
                    None,
                    false,
                    false,
                    false,
                );
                if table_bottom > max_table_bottom {
                    max_table_bottom = table_bottom;
                }

                if table_wrapped {
                    current_y = table_bottom;
                    inline_x = line_start_x;
                    wrapped_below_table = true;
                } else {
                    inline_x += tw + om_left + om_right;
                }
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
            let (om_left, om_right) = table_om_px[table_idx];
            let tbl_h = mt
                .map(|m| m.total_height)
                .unwrap_or_else(|| hwpunit_to_px(tbl.common.height as i32, self.dpi));
            let om_bottom = hwpunit_to_px(tbl.outer_margin_bottom as i32, self.dpi);
            let tbl_y = self
                .tac_table_stored_outer_band_top(para, tbl, current_y)
                .unwrap_or_else(|| (current_y + baseline_dist + om_bottom - tbl_h).max(current_y));

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
                Some(inline_x + om_left),
                None,
                table_para_y,
                None,
                false,
                false,
                false,
            );
            if table_bottom > max_table_bottom {
                max_table_bottom = table_bottom;
            }

            inline_x += tw + om_left + om_right;
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
        tree: &mut PageLayoutContext,
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
            styles.hwp3_variant && self.endnote_para_source_for(para_index).is_none(),
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
        tree: &mut PageLayoutContext,
        col_node: &mut RenderNode,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        hwp3_body_reflow: bool,
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
            let recomposed: Option<ComposedParagraph> = {
                let para_style = styles.para_styles.get(comp.para_style_id as usize);
                let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let column_inner_width = (col_area.width - margin_l - margin_r).max(0.0);
                // 문단 상자는 편집 경로(`DocumentCore::reflow_paragraph`)의 가용 폭과
                // 같아야 한다 — 한 문단이 어느 경로로 왔는지에 따라 다른 폭을 갖지
                // 않게 한다(typeset 의 동일 산출과 맞춘다). 들여쓰기/내어쓰기는 이
                // 상자 **안에서** `layout_paragraph_in_frame` 의 indent_px 가 적용한다.
                // `body_for_style`, not `body` — see the note in `typeset.rs`.
                let paragraph_box = crate::renderer::composer::ParagraphBox::body_for_style(
                    col_area.width,
                    para_style,
                    self.dpi,
                );
                // NO_LS 와 저장분할 both go to the frame: no stored record means
                // the rebuild case outright, and the frame's fill owns it.
                if column_inner_width > 0.0 {
                    crate::renderer::composer::recompose_stored_lines_in_frame_with_known_square_band(
                        comp,
                        para,
                        paragraph_box,
                        column_inner_width,
                        styles,
                        self.dpi,
                        self.profile.get().legacy_hwp3_stored_geometry(),
                        crate::renderer::composer::StoredRowMissPolicy::Reflow,
                        &self.body_float_carve_evidence.borrow(),
                        wrap_anchor.is_some(),
                    )
                } else {
                    None
                }
            };
            let comp_ref = recomposed.as_ref().unwrap_or(comp);
            // [#2279] 전체-문단 요청(start=0, end=원본 줄수 이상)은 재래핑 후 줄수로
            // 확장한다. 종전에는 재래핑이 줄수를 늘린 문단(45자 폴백 3줄 → 실폭 4줄,
            // 86712 pi=22)에서 원본 줄수로 클램프되어 마지막 줄이 렌더에서 소실됐다
            // (측정 4줄 fit vs 렌더 3줄 — maintainer PR #2284 리뷰 p10 픽셀 하락과
            // 정합). 분할(partial) 요청의 라인 범위는 종전 클램프 유지.
            let end_line_adjusted = if start_line == 0 && end_line >= comp.lines.len() {
                comp_ref.lines.len()
            } else {
                end_line.min(comp_ref.lines.len()).max(start_line)
            };
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
    /// [Task #2067] run 루프 종료 후, run 범위 밖(pos >= run_char_pos)의 미매칭
    /// TAC 이미지 배치. 갱신된 x 를 반환한다.
    /// HWP `treat_as_char` 그림도 일반 개체와 같이 size criterion을 해석한다.
    /// HWP5의 `PAPER`/`PAGE` 값은 HWPUNIT가 아니라 기준 영역의 1/100 % 단위다.
    /// 이 경로에서 원시 `common.width`를 HWPUNIT으로 바꾸면 42520(=425.20%) 같은
    /// 그림을 42.52 mm로 축소해 렌더한다.
    pub(crate) fn resolve_inline_picture_size(
        &self,
        picture: &crate::model::image::Picture,
        col_area: &LayoutRect,
    ) -> (f64, f64) {
        let (body_x, body_y, body_w, body_h) = self.current_body_area.get();
        let body_area = if body_w > 0.0 && body_h > 0.0 {
            LayoutRect {
                x: body_x,
                y: body_y,
                width: body_w,
                height: body_h,
            }
        } else {
            *col_area
        };
        let paper_area = LayoutRect {
            x: 0.0,
            y: 0.0,
            width: self.current_paper_width.get().max(col_area.width),
            height: self.current_page_height.get().max(col_area.height),
        };

        self.resolve_object_size(&picture.common, col_area, &body_area, &paper_area)
    }

    #[allow(clippy::too_many_arguments)]
    fn place_unmatched_line_tac_pictures(
        &self,
        tree: &mut PageLayoutContext,
        line_node: &mut RenderNode,
        comp_line: &ComposedLine,
        para: Option<&Paragraph>,
        bin_data_content: Option<&[BinDataContent]>,
        tac_offsets_px: &[(usize, f64, usize)],
        col_area: &LayoutRect,
        cell_ctx: Option<&CellContext>,
        reserved_tac_picture_height: &mut Option<f64>,
        v: TacPictureLineVars,
    ) -> f64 {
        let TacPictureLineVars {
            run_char_pos,
            mut x,
            y,
            baseline,
            raw_lh,
            section_index,
            para_index,
        } = v;
        if !comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
            if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                let line_start_char = comp_line.char_start;
                let line_end_char = line_start_char
                    + comp_line
                        .runs
                        .iter()
                        .map(|r| r.text.chars().count())
                        .sum::<usize>();
                for &(tac_pos, tac_w, tac_ci) in tac_offsets_px {
                    if tac_pos <= run_char_pos || tac_pos > line_end_char {
                        continue; // run 범위 내/끝 또는 미래 줄 TAC: 여기서 처리하지 않음
                    }
                    if let Some(ctrl) = p.controls.get(tac_ci) {
                        if let Control::Picture(pic) = ctrl {
                            let (_, pic_h) = self.resolve_inline_picture_size(pic, col_area);
                            if raw_lh + 4.0 >= pic_h {
                                *reserved_tac_picture_height = Some(pic_h);
                            }
                            let img_y = (y + baseline - pic_h).max(y);
                            let bin_data_id = pic.image_attr.bin_data_id;
                            let image_data = find_bin_data_bytes(bdc, bin_data_id);
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
                            let original_size_hu = pic.crop_reference_size();
                            // [Task #1151 v7 항목 7] ImageNode 생성 helper 통합.
                            let img_node = make_picture_image_node(
                                tree,
                                pic,
                                section_index,
                                para_index,
                                tac_ci,
                                cell_ctx,
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
        x
    }

    /// [Task #2067] 빈 문단(runs 없음)의 TAC 양식 개체 배치. 갱신된 x 를 반환한다.
    #[allow(clippy::too_many_arguments)]
    fn place_empty_line_tac_forms(
        &self,
        tree: &mut PageLayoutContext,
        line_node: &mut RenderNode,
        comp_line: &ComposedLine,
        para: Option<&Paragraph>,
        tac_offsets_px: &[(usize, f64, usize)],
        cell_ctx: Option<&CellContext>,
        mut x: f64,
        y: f64,
        baseline: f64,
        section_index: usize,
        para_index: usize,
    ) -> f64 {
        if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
            if let Some(p) = para {
                for &(_tac_pos, tac_w, tac_ci) in tac_offsets_px {
                    if let Some(Control::Form(f)) = p.controls.get(tac_ci) {
                        let form_h = hwpunit_to_px(f.height as i32, self.dpi);
                        let form_y = (y + baseline - form_h).max(y);
                        let cell_location = cell_ctx.and_then(|ctx| {
                            ctx.path.first().map(|e| {
                                (
                                    ctx.parent_para_index,
                                    e.control_index,
                                    e.cell_index,
                                    e.cell_para_index,
                                )
                            })
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
        x
    }

    /// [Task #2067 / 원본 Task #287] 빈 runs 줄의 TAC 수식 인라인 배치.
    /// 큰 디스플레이 수식이 자체 LINE_SEG 를 가질 때 comp_line.runs 가 비어있는데,
    /// run 루프가 돌지 않아 수식이 인라인 경로로 렌더되지 않고 shape_layout display
    /// 경로로 떨어져 col_area.y 에 고정되던 문제를 해결한다.
    #[allow(clippy::too_many_arguments)]
    fn place_empty_line_inline_equations(
        &self,
        tree: &mut PageLayoutContext,
        line_node: &mut RenderNode,
        comp_line: &ComposedLine,
        composed: &ComposedParagraph,
        para: Option<&Paragraph>,
        styles: &ResolvedStyleSet,
        cell_ctx: &Option<CellContext>,
        tac_offsets_px: &[(usize, f64, usize)],
        line_tac_offsets: &[(usize, f64, usize)],
        equation_tac_line_flow: &Option<crate::renderer::equation_tac_flow::EquationTacLineFlow>,
        v: EquationTacLineVars,
    ) {
        if !comp_line.runs.is_empty() || tac_offsets_px.is_empty() {
            return;
        }
        let EquationTacLineVars {
            line_idx,
            line_end: end,
            alignment,
            available_width,
            margin_left,
            indent,
            effective_col_x,
            y,
            baseline,
            line_height,
            line_spacing_px,
            col_area_y,
            col_bottom,
            line_char_end,
            is_last_line_of_para,
            defer_empty_line_control_marker,
            equation_tac_extra_rows,
            hwp3_indent_scale,
            section_index,
            para_index,
        } = v;
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
        // [#5583] 본문 흐름의 수식-only 줄도 문단 정렬을 따른다 — 단 저장 LINE_SEG 가 줄 시작
        // 위치를 담고 있으면(그 값이 권위다) 종전대로 저장 흐름을 쓴다.
        //
        // 종전에는 비-셀이면 무조건 0.0 이라 가운데 정렬 문단의 수식이 단 왼쪽 끝에 붙었다
        // (3252633 국가유산수리 감리대가 기준 2·3쪽: 저장 cs=0 sw=48188 인데 수식 x=75.6 =
        // 본문 좌측, 가운데라면 269.6). `column_start > 0` 인 줄은 한컴이 흐름 x 를 적어 둔
        // 경우이므로 #1256/#1308 계약대로 그 값을 존중한다.
        let align_offset = if cell_ctx.is_some() || comp_line.column_start == 0 {
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
                        }) * hwp3_indent_scale,
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
                    let ast = crate::renderer::equation::parser::EqParser::new(tokens).parse();
                    let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                    let layout_box =
                        crate::renderer::equation::layout::EqLayout::new(font_size_px).layout(&ast);
                    let color_str =
                        crate::renderer::equation::svg_render::eq_color_to_svg(eq.color);
                    let svg_content = crate::renderer::equation::svg_render::render_equation_svg(
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
                        .max(col_area_y);
                    let inline_x = row_inline_x[tac_row];
                    let eq_y = if cell_ctx.is_some() {
                        (row_y + baseline - layout_box.baseline).max(row_y)
                    } else {
                        row_y + baseline - layout_box.baseline
                    };
                    let (eq_cell_idx, eq_cell_para_idx) = if let Some(ref ctx) = cell_ctx {
                        (
                            ctx.path.first().map(|e| e.cell_index),
                            ctx.path.first().map(|e| e.cell_para_index),
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
                        RenderNodeType::Equation(crate::renderer::render_tree::EquationNode {
                            svg_content,
                            layout_box,
                            color_str,
                            color: eq.color,
                            script: eq.script.clone(),
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
                                ctx.path.first().map(|e| e.control_index).or(Some(tac_ci))
                            } else {
                                Some(tac_ci)
                            },
                            cell_index: eq_cell_idx,
                            cell_para_index: eq_cell_para_idx,
                            note_ref,
                        }),
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
            let marker_style = paragraph_active_text_style(styles, para, line_char_end).0;
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
                    layout_positions: None,
                    display_text: None,
                }),
                BoundingBox::new(marker_x, marker_y, 0.0, line_height),
            );
            line_node.children.push(marker_node);
        }
    }

    pub(crate) fn layout_composed_paragraph(
        &self,
        tree: &mut PageLayoutContext,
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
        // [#4968 R4D-1] 한 문단의 모든 최종 emitted run이 같은 registry
        // generation과 per-face parse cache를 소비한다.
        let mut kerning_layout_session = self.exact_font_layout_session();

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
        let has_ole_shape_square_wrap = para
            .map(|p| {
                p.controls.iter().any(|c| {
                    matches!(
                        c,
                        Control::Shape(shape)
                            if matches!(shape.as_ref(), ShapeObject::Ole(_))
                                && !shape.common().treat_as_char
                                && matches!(shape.common().text_wrap, TextWrap::Square)
                    )
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
                .map(|(pos, w_hu, ci)| {
                    let width = para
                        .and_then(|p| p.controls.get(*ci))
                        .and_then(|ctrl| match ctrl {
                            Control::Picture(pic) => {
                                Some(self.resolve_inline_picture_size(pic, col_area).0)
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| hwpunit_to_px(*w_hu, self.dpi));
                    (*pos, width, *ci)
                })
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
        let horizontal_shaping_initial_lane = horizontal_shaping_initial_lane_preflight(
            composed,
            para,
            styles,
            start_line,
            end,
            alignment,
            para_border_fill_id,
        );

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
        // [#5601] 셀 저장-앵커 스냅 문단 — 호출자가 para_y 에서 spacing_before 를
        // 미리 뺐으므로 column-top 트림과 무관하게 전량 재가산해야 vpos 와 맞는다.
        let reapply_snap_spacing_before = self.reapply_snap_anchored_spacing_before.replace(false);
        if start_line == 0 && spacing_before > 0.0 {
            if !is_column_top || keep_continuation_spacing_before || reapply_snap_spacing_before {
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
                                layout_positions: None,
                                display_text: None,
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
                        // [#3637] 기준은 **단 상단**이다 (원점 0).
                        //
                        // `LINE_SEG.vertical_pos` 는 문단 기준이 아니라 쪽(단) 상단 기준
                        // 누적 절대값이다 — 같은 쪽에서 pi=5 → 13949, pi=17 → 58149 로
                        // 문단을 가로질러 단조 증가한다. 따라서 줄의 y 는
                        // `단 상단 + vpos` 이지, `흐름 커서 + vpos` 가 아니다.
                        //
                        // 종전에는 `start_line == 0` 일 때 기준 vpos 만 0 으로 두고 기준 y 는
                        // 흐름 커서(`y`)로 두어, 절대값이 커서 위에 **한 번 더** 얹혔다.
                        // 문단이 쪽 상단이면 커서≈0 이라 무해했지만, 쪽 중간 문단이면 자기
                        // vpos 만큼 아래로 밀려 쪽 밖으로 나간다.
                        //
                        // 실측 (해양 모빌리티 보도자료 pi=17):
                        //   단 상단 94.5 + vpos 775.3 = 869.8px 가 정답인데 흐름 커서
                        //   869.8 에 vpos 를 또 더해 1660px 에 그렸다. 쪽 하단 1028px 를
                        //   632px 넘겨 세 줄 93글자가 SVG·PNG 어느 경로에서도 보이지 않았다.
                        //
                        // 기준 y 를 단 상단으로 내리면 첫 줄이 개체 아래로 밀린 경우
                        // (#1459 자리차지 그림 + TAC 그림 스택)도 그 밀림이 vpos 에 이미
                        // 담겨 있어 그대로 재현된다. 첫 줄 vpos 를 기준 삼으면 그 밀림이
                        // 사라져 두 그림이 같은 y 에 겹친다 — 실제로 겪은 회귀다.
                        Some((0, col_area.y))
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
        // [#5711] 마지막으로 그린 줄 상자의 아래 경계. 문단 테두리의 아래 변은 이 값을
        // 따라야 한다 — 줄간격이 음수인 문단에서는 전진값 `y` 가 줄 상자 아래보다 위로
        // 올라가, 테두리가 글자를 가로지른다(3143955 제목: 줄 상자 아래 171.6px, 전진 y
        // 162.0px, 실제로 그려진 선 159.7/163.7).
        let mut last_line_box_bottom: Option<f64> = None;
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

            // 저장 LINE_SEG 없는 실제 빈 문단은 compose의 400HU 안내 줄이 아니라
            // 원래 글자 모양과 줄간격을 사용한다. HeightMeasurer의 동일 보정과
            // 맞춰 pagination과 render의 y advance가 갈라지지 않게 한다.
            let empty_no_lineseg_metrics = if line_idx == 0 {
                para.and_then(|p| {
                    empty_no_lineseg_paragraph_metrics(
                        p,
                        styles,
                        para_style,
                        self.profile.get().hwp3_layout(),
                        self.dpi,
                    )
                })
            } else {
                None
            };

            // 최대 폰트 크기 계산 (line_height 최솟값 보정에도 사용)
            let mut max_fs = comp_line
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
            if let Some((_, _, font_size)) = empty_no_lineseg_metrics {
                max_fs = font_size;
            }
            // [#5854] 통짜 합성 사다리 문서의 빈 문단은 조합 줄에 run 이 하나도 없어
            // 위 fold 가 0 을 준다. 조판(typeset)은 이미 `composed_line_max_font_size`
            // 로 저장 글자모양을 보조 근거로 쓰므로, 렌더도 같은 근거를 써야 두 경로의
            // 줄 진행이 갈라지지 않는다.
            let uniform_filler_ladder = self.uniform_filler_ladder.get();
            if uniform_filler_ladder && max_fs <= 0.0 {
                if let Some(p) = para {
                    max_fs = crate::renderer::composed_line_max_font_size(comp_line, p, styles);
                }
            }
            let mut line_tac_offsets = tac_offsets_for_line(composed, &tac_offsets_px, line_idx);
            if let Some(offsets) =
                repeated_empty_tac_line_offset(composed, &tac_offsets_px, line_idx)
            {
                line_tac_offsets = offsets;
            }
            let runs_all_whitespace = comp_line.runs.iter().all(|r| r.text.trim().is_empty());
            // 정렬 폭은 실제 run 방출과 같은 TAC 귀속을 쓴다. 끝 위치 TAC를 빼면
            // 그림은 그리되 Center/Right 시작점이 그림 폭만큼 우측으로 밀린다 (#3257).
            let line_tac_offsets_for_width =
                tac_offsets_for_line_width(composed, &tac_offsets_px, line_idx);
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
            let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
            let ls_type = para_style
                .map(|s| s.line_spacing_type)
                .unwrap_or(LineSpacingType::Percent);
            let raw_text_height = para
                .and_then(|p| p.line_segs.get(line_idx))
                .map(|seg| hwpunit_to_px(seg.text_height, self.dpi))
                .unwrap_or(0.0);
            let use_stored_text_height = para.map(|p| p.controls.is_empty()).unwrap_or(false)
                && (self.profile.get().hwpx_stored_layout() || cell_ctx.is_none());
            let source_metrics_reflow_eligible = para
                .map(|p| crate::renderer::controls_mark_section_start(&p.controls))
                .unwrap_or(false)
                && self.profile.get().hwpx_stored_layout();
            let source_metrics_reflowed = crate::renderer::source_line_metrics_need_reflow(
                raw_lh,
                raw_text_height,
                max_fs,
                ls_type,
                ls_val,
                source_metrics_reflow_eligible,
            );
            let (line_height, line_spacing_px) = empty_no_lineseg_metrics
                .map(|(line_height, line_spacing_px, _)| (line_height, line_spacing_px))
                .unwrap_or_else(|| {
                    // [#5854] 통짜 합성 사다리는 저장 `line_height` 가 글자 크기보다
                    // 크든 작든 실측이 아니다 — 조판(typeset)과 같은 규칙으로 항상
                    // 글꼴·문단 스타일에서 다시 뽑는다.
                    if uniform_filler_ladder && max_fs > 0.0 && !text_before_picture_line {
                        return crate::renderer::corrected_line_metrics(
                            0.0, 0.0, max_fs, ls_type, ls_val,
                        );
                    }
                    crate::renderer::corrected_line_metrics_for_source(
                        raw_lh,
                        raw_text_height,
                        hwpunit_to_px(comp_line.line_spacing, self.dpi),
                        max_fs,
                        ls_type,
                        ls_val,
                        use_stored_text_height,
                        source_metrics_reflow_eligible,
                    )
                });
            // [#2279 진단] 줄별 pitch 분해 — 동작 불변.
            if let Ok(pat) = std::env::var("RHWP_DIAG_PITCH") {
                if para.map(|p| p.text.contains(&pat)).unwrap_or(false) {
                    eprintln!(
                        "DIAG_PITCH li={} raw_lh={:.2} raw_ls={:.2} max_fs={:.2} -> lh={:.2} ls={:.2} stored_ls_cnt={}",
                        line_idx,
                        raw_lh,
                        hwpunit_to_px(comp_line.line_spacing, self.dpi),
                        max_fs,
                        line_height,
                        line_spacing_px,
                        para.map(|p| p.line_segs.len()).unwrap_or(0),
                    );
                }
            }
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
            // 도형의 흐름 높이가 저장 프레임이 아니라 `current_height` 에서 오는 줄은
            // 저장 줄 높이를 그대로 둔다 (아래 baseline 정렬 축소 제외).
            let empty_tac_guide_has_explicit_shape_height = empty_tac_guide_line
                && para.is_some_and(|p| {
                    line_tac_offsets.iter().any(|(_, _, ci)| {
                        p.controls.get(*ci).is_some_and(|ctrl| match ctrl {
                            Control::Shape(shape) if shape.common().treat_as_char => {
                                shape.flow_height_hu() > shape.common().height as i32
                            }
                            _ => false,
                        })
                    })
                });
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
                // 본문(cell_ctx 없음)에서 max_fs=0 인 도형-전용 줄이 lh=0 으로 접히는 것은
                // **의도된 짝**이다. 짝의 나머지 반쪽은 `layout_column_item` 의 TAC-Shape
                // 높이 바닥값(`renderer/layout.rs:7037-7076`,
                // `para_start + max(seg_lh, shape_max_h)`)이다 — 접힘이 줄 루프의 진행을
                // 바닥값 아래로 눌러, 문단 진행을 바닥값이 지배하게 만든다. 한컴에 맞춰진
                // 숫자는 그 바닥값(꼬리 줄간격을 포함하지 않는 값)이지 이 줄의 lh 가 아니다.
                //
                // 실측 (`samples/hwp3-sample16-hwp5.hwp` 구역0 문단71,
                // `RHWP_DEBUG_PARA_TAC` + `RHWP_DEBUG_TAC_CURSOR`):
                //   TAC_ADV    pi=71 raw_lh=130.2 lh=0.0 ls=10.4   ← 루프 내 진행은 10.4
                //   TAC_CURSOR FullPara pi=71 dy=130.2            ← 바닥값이 문 결과
                // 접힘만 없애면 루프 진행이 raw_lh+ls=140.6 으로 바닥값 130.2 를 넘어
                // 문단이 정확히 `LineSeg.line_spacing`(10.4px) 만큼 밀리고,
                // `tests/issue_1116.rs` 의 한컴 PDF 대조 핀 둘이 그만큼 깨진다.
                //
                // 보상자는 `HeightCursor::vpos_adjust` 가 **아니다** — 그 함수는 이 문단
                // 다음 항목(pi=72)에서 `lazy_base < 0` 으로 조기 반환한다
                // (`RHWP_VPOS_DEBUG` → `VPOS_CORR_SKIP: pi=72 ... lazy_base=-72`).
                // 이 자리의 옛 주석이 "reserved/skip-advance 보상 기계"를 지목해 앞선
                // 조사를 `vpos_adjust` 로 잘못 보냈다 (#4333).
                let font_lh = max_fs * 1.2; // 폰트 크기의 120%
                let font_bl = max_fs * 0.85;
                (font_lh, ensure_min_baseline(font_bl, max_fs))
            } else {
                // [#5825] 퇴화 저장 baseline 클램프 — 기계생성 통계표는 lineseg 에
                // baseline == textheight(하강부 0)를 저장한다(156673604 34쪽 표 두 개:
                // bl=1100=vertsize·spacing=0). 받침이 내려갈 자리가 없어 글자가 아래
                // 괘선을 지나간다. 한글 2022 는 이 값을 무시하고 표준 ascent 로
                // 그린다(실측 12.62px = 0.86×; 같은 문서의 정상 표 저장값도
                // 935 = 0.85×1100). 하강부가 0 인 baseline 만 0.85×textheight 로
                // 되돌리고, 정상 저장 baseline(bl < th)은 그대로 둔다.
                let stored_bl = hwpunit_to_px(comp_line.baseline_distance, self.dpi);
                let stored_bl = if raw_text_height > 0.0 && stored_bl >= raw_text_height - 0.01 {
                    raw_text_height * 0.85
                } else {
                    stored_bl
                };
                (
                    line_height,
                    ensure_min_baseline(
                        crate::renderer::corrected_line_baseline_for_source(
                            stored_bl,
                            max_fs,
                            source_metrics_reflowed,
                        ),
                        max_fs,
                    ),
                )
            };
            // 들여쓰기/내어쓰기: 문단 여백은 무조건 적용
            // - 보통(ind=0): 모든 줄 margin_left
            // - 들여쓰기(ind>0): 첫줄 margin_left+indent, 다음줄 margin_left
            // - 내어쓰기(ind<0): 첫줄 margin_left, 다음줄 margin_left+|indent|
            //
            // [Issue #6190] **저장 LINE_SEG 의 `TAG_INDENTATION`(bit 20)이 정답지다.**
            // 이 비트는 "이 줄에 들여쓰기가 적용됐다"는 한글의 줄별 기록이다. 비트가
            // 꺼진 줄에 우리가 들여쓰기를 얹으면 그 줄과, 그 문단이 호스트하는 표까지
            // 함께 밀린다(156458354 3쪽 `경 력 사 항` +68.1px, 마지막 표는 용지 밖 36px).
            //
            // 한글 통제 실험으로 확인했다 — 같은 문단의 `indent` 만 바꿔 한글로 PDF 를
            // 떠서 재면:
            //
            // | 문서 | ls[0].tag | indent 0→20445 스윕 | 한글 x |
            // |---|---|---|---|
            // | 156458354 pi=28 | `0x60000` (bit20 꺼짐) | 0 · 2000 · 6000 · 10000 · 20445 | **전부 345.60 (불변)** |
            // | 36313646 pi=2 | `0x160000` (bit20 켜짐) | 0 · 660 · 4000 · 10000 · 20445 | 352.77 → 420.89 (**정확히 indent/4 씩**) |
            //
            // 내어쓰기 문단이 `ls[0]=0x60000, ls[1..]=0x160000` 인 것도 같은 의미다 —
            // 내어쓰기는 둘째 줄부터 적용되고, 비트가 줄마다 그것을 기록한다.
            // 합성 사다리(`TAG_IMPLEMENTATION_PROPERTY`)는 이 증언이 없으므로 제외한다.
            //
            // 편집으로 줄 수가 달라진 문단은 저장 사다리가 더는 이 조판을 설명하지
            // 못한다 — 그때는 비트도 낡은 기록이다(#6204 계열). `composed.lines` 와
            // 저장 세그 수가 같을 때만 증언으로 쓴다.
            //
            // **본문 흐름 한정**이다 — 표 셀 안 문단은 한글이 들여쓰기를 적용한다
            // (오라클 7문서: 2777015 · 156548319 · 156658621 · 156428389 등 모두 셀 안
            // Center 문단이고 한글 x 가 `indent/2` 반영값과 0.0~0.4px 로 일치).
            let stored_ladder_covers_lines = cell_ctx.is_none()
                && para.is_some_and(|p| p.line_segs.len() == composed.lines.len());
            let stored_seg_denies_indent = stored_ladder_covers_lines
                && para
                    .and_then(|p| p.line_segs.get(line_idx))
                    .is_some_and(|seg| {
                        seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                            && seg.tag & LineSeg::TAG_INDENTATION == 0
                    });
            let line_indent = if stored_seg_denies_indent {
                0.0
            } else {
                crate::renderer::equation_tac_flow::paragraph_line_indent(indent, line_idx)
            };
            let styled_margin_left = margin_left + line_indent;

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
            let empty_stored_wrap_line = cell_ctx.is_none()
                && para
                    .map(|p| p.text.is_empty() && p.controls.is_empty())
                    .unwrap_or(false)
                && comp_line.column_start > 0
                && comp_line.segment_width > 0
                && comp_line.segment_width < col_area_w_hu;
            // [#5818] 어울림(Square 계열) float 그림이 있는 **셀** 의 줄도 저장
            // cs/sw(한컴이 인코딩한 wrap 배제)를 존중한다. 종전 게이트는 전부
            // cell_ctx.is_none() 이라 셀 줄이 배제를 무시하고 셀 왼끝에서 시작해
            // 로고를 파고들었다(156599239 머리 표: 저장 cs=4037HU=53.8px, 한글
            // 실측 x=151.7 = 셀 콘텐츠 왼끝+cs ↔ rhwp 102.2). 신호는 같은 셀에
            // Square float 가 실재할 때만 켜져(#547 문단 테두리 inset 오인 차단),
            // cs>0 && sw<셀폭 인 줄에 한정한다.
            let cell_square_wrap_stored_line = cell_ctx.is_some()
                && self.cell_has_square_float.get()
                && comp_line.column_start > 0
                && comp_line.segment_width > 0
                && comp_line.segment_width < col_area_w_hu;
            // [#6175] 문단 **전체**가 개체 옆에 들어가면 같은 문단 안에 넓은 줄이
            // 없어 `precomputed_body_wrap_line`(혼합 폭)이 발화하지 않는다. 그때는
            // 문서에 실재하는 같은 세로 band의 어울림 개체가 증거다 — 저장 행이 남긴
            // 결손 폭과 위치를 그 개체가 함께 설명하면 좁음의 출처는 외부 기하다.
            //
            // ⚠ "균일하게 좁다"만으로 켜면 문단 테두리 박스의 inset 을 어울림으로
            // 오인해 #547·#1440 핀이 깨진다(#6129 반증). 판별은 개체 폭과 세로 band 대조다 —
            // 셀의 #5818 계약("같은 셀에 Square float 실재")과 같은 원리의 본문 판.
            // 컴포저의 `stored_rows_require_external_geometry` 가 같은 증거로 저장
            // 행을 지켜 두므로, 두 층이 같은 판정을 공유한다.
            let body_square_wrap_stored_line = cell_ctx.is_none()
                && !para_has_mixed_segment_widths
                && comp_line.column_start == 0
                && comp_line.segment_width > 0
                && {
                    let evidence = self.body_float_carve_evidence.borrow();
                    let missing = col_area_w_hu.saturating_sub(line_avail_hu);
                    !evidence.is_empty()
                        && missing > 1200
                        && para.is_some_and(|paragraph| {
                            evidence.iter().any(|candidate| {
                                candidate.matches_stored_rows(missing, &paragraph.line_segs, 1200)
                            })
                        })
                };
            let uses_stored_segment_geometry = (has_picture_shape_square_wrap
                || line_has_inline_tac_table
                || precomputed_body_wrap_line
                || empty_stored_wrap_line
                || body_square_wrap_stored_line
                || cell_square_wrap_stored_line)
                && comp_line.segment_width > 0
                && (line_avail_hu < col_area_w_hu - 200 || cs_significant);
            let (effective_col_x, effective_col_w) = if uses_stored_segment_geometry {
                let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
                let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
                (col_area.x + cs_px, sw_px)
            } else {
                (col_area.x, col_area.width)
            };
            let profile = self.profile.get();
            let hwp5_stored_line_start_eligible = cell_ctx.is_none()
                && self.is_body_flow_col_area(col_area)
                && matches!(alignment, Alignment::Justify | Alignment::Left)
                && wrap_anchor.is_none()
                && !uses_stored_segment_geometry
                && composed.numbering_text.is_none()
                && para.map(|p| p.controls.is_empty()).unwrap_or(false)
                // [#3837] rhwp 가 HWP5 원본에서 내보낸 HWPX 도 같은 계약이다 — 저장
                // LINE_SEG 가 그 HWP5 의 것이라 `column_start` 가 여전히 권위다. 이 조건이
                // 없으면 왕복만으로 들여쓴 줄이 왼쪽으로 밀린다(1370000-200800015: 저장
                // cs=22677 = 302.4px 가 무시돼 글리프 595개가 그만큼 이동).
                // 원본 HWPX 는 건드리지 않는다 — 그쪽 저장 계약은 별개 축이다.
                && uses_hwp5_stored_line_start_profile(profile);
            // 암호 HWP3의 Square-wrap Picture/Shape 저장 cs/sw는 문단 좌·우 inset까지
            // 포함한 완성 line box다. 여기서 ParaShape margin을 다시 더하거나 빼면
            // 그림과 글자 사이에 여백이 한 번 더 생기고 right edge도 불필요하게 줄어든다.
            // 일반 HWP3/HWP5의 저장 segment 계약은 다르므로 기존 여백 처리를 유지한다.
            let hwp3_password_stored_segment_line_box =
                uses_stored_segment_geometry && self.profile.get().hwp3_password_layout();
            // [#4690] 저장 cs/sw 조각이 문단 여백을 담지 못하면 여백을 적용하지 않는다.
            //
            // 이 경로의 `effective_col_x/w` 는 이미 저장 `cs`/`sw` 가 정한 줄 상자다. 그
            // 상자가 좁은데 `margin_left + line_indent + margin_right` 를 그대로 얹으면
            // 줄이 상자 오른쪽 밖에서 시작하고 폭이 음수가 된다 — 그 줄의 글자는 정상적으로
            // 그려질 수 없다(30098 p3 pi48 L1: x=721.2 폭 −1.6, 문서 전체 18줄. 저장
            // 사다리 값은 x=679.7 폭 38.4). 여백이 담기지 않는다는 것 자체가 그 조각을
            // 완성된 line box 로 읽어야 한다는 신호이므로, 암호 HWP3 경로와 같은 처리를
            // 한다.
            //
            // 이 가드는 **여백이 상자를 넘칠 때만** 발동한다. 여백이 들어가는 정상 어울림
            // 줄은 종전대로 둔다 — `line_indent` 를 이 경로에서 일괄로 빼면 wrap 텍스트가
            // 그림 영역으로 침범한다(#1230 `exam_science` pi=21). 저장 cs 가 내어쓰기를
            // 대체한다는 해석도 성립하지 않는다: 정답지 `pdf/exam_kor-2022.pdf`
            // (Hwp 2022 12.0.0.4426) p5 에서 첫/마지막 lineseg 의 cs 가 둘 다 1130 으로
            // 같은 문단인데도 한/글은 이어지는 줄을 `|indent|` 만큼 들여 그린다
            // (99.12pt ↔ 110.4pt = 132.16px ↔ 147.20px @96dpi). 즉 cs 는 그 줄의 확정
            // 시작점이 아니라 문단 왼쪽 여백이다.
            let stored_segment_line_box_cannot_hold_margins = uses_stored_segment_geometry
                && !hwp3_password_stored_segment_line_box
                && styled_margin_left + margin_right >= effective_col_w;
            let (effective_margin_left, effective_margin_right) =
                if hwp3_password_stored_segment_line_box
                    || stored_segment_line_box_cannot_hold_margins
                {
                    (0.0, 0.0)
                } else {
                    (
                        authoritative_stored_line_start_px(
                            styled_margin_left,
                            para.and_then(|p| p.line_segs.get(line_idx)),
                            col_area_w_hu,
                            self.dpi,
                            hwp5_stored_line_start_eligible,
                        ),
                        margin_right,
                    )
                };

            // [#5598] 내어쓰기가 줄 상자를 한 글자도 못 담을 만큼 먹으면 적용하지 않는다.
            //
            // 좁은 표 칸에서 문단 내어쓰기(|indent|)가 칸 안쪽 폭에 육박하면, 이어지는 줄의
            // 상자가 몇 px 로 무너져 글자가 칸 오른쪽 밖으로 밀려 나간다(2995759 `분류처우위원회
            // 심의ㆍ의결` 칸: 안쪽 폭 107.7px, indent −104.4px → 둘째 줄 상자 x=193.3 w=3.3,
            // `의결` 이 칸 밖). 한글은 같은 문단의 두 줄을 모두 칸 안쪽 폭으로 조판한다
            // (저장 LINE_SEG 두 줄 모두 cs=200 sw=8076).
            //
            // 첫 줄은 내어쓰기의 기준선이므로 건드리지 않고, 이어지는 줄에만 적용한다.
            let effective_margin_left = if line_indent > 0.0 {
                let min_line_w = max_fs.max(1.0);
                let avail = effective_col_w - effective_margin_left - effective_margin_right;
                if avail < min_line_w {
                    margin_left.min(effective_margin_left)
                } else {
                    effective_margin_left
                }
            } else {
                effective_margin_left
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
            // [#3637] 셀 안 줄이 **쪽 본문 하단**을 넘는 경우.
            //
            // 위 진단은 `is_body_flow_col_area && cell_ctx.is_none()` 이라 본문 흐름만
            // 본다. 셀은 `col_area` 가 셀 사각형이라 그 조건이 언제나 거짓이고, 그래서
            // 셀 안에서 쪽 밖으로 나간 글자는 **한 줄도 보고되지 않았다**.
            //
            // 실측: 쪽 밖 글자가 있는 문서 91건 중 8건이 이 침묵 구간이었다
            // (총 2,910자, 최대 471.8px 초과). 텍스트 추출에는 남아 있어 텍스트 diff 로도
            // 안 잡히고, 진단마저 없어 관측 자체가 불가능했다.
            //
            // 기준선 두 가지가 함께 맞아야 오탐이 사라진다.
            //
            // 1. **쪽 하단** (본문 하단 아님). 본문 하단과 쪽 하단 사이는 아래 여백·꼬리말
            //    구간이라 거기 그려진 글자는 실제로 보인다. 본문 하단으로 재면 그 구간이
            //    통째로 오탐이 된다.
            // 2. 줄의 **윗변**(`text_y`). 아랫변으로 재면 마지막 줄 디센더가 경계를 스치는
            //    정상 상태까지 잡는다. 윗변이 이미 쪽 밖이면 그 줄은 **어느 부분도 그려지지
            //    않는다** — 배율·글꼴에 무관한 판정이다.
            //
            // MATCH 대조군 80건 실측: 아랫변 기준은 9건(11%) 오탐, 초과폭이 전부
            // 5.4~23.9px(줄 높이 이내)였다. 윗변으로 바꾸니 7건, 기준을 쪽 하단으로 옮겨야
            // 0 이 된다. 진짜 침묵 구간 8건은 146.0~512.0px 라 어느 기준에서도 남는다.
            if cell_ctx.is_some() && !blank_spacer_line {
                let page_h = self.current_page_height.get();
                if page_h > 0.0 && text_y > page_h + 0.5 {
                    // [#3668] stderr 진단과 같은 조건에서 집계 카운터도 올린다.
                    self.overflow_cell_lines
                        .set(self.overflow_cell_lines.get() + 1);
                    eprintln!(
                        "LAYOUT_OVERFLOW_CELL: section={} pi={} line={} y={:.1} \
                         page_bottom={:.1} overflow={:.1}px",
                        section_index,
                        para_index,
                        line_idx,
                        text_y,
                        page_h,
                        text_y - page_h,
                    );
                    if std::env::var("RHWP_DIAG_OVERFLOW_CELL").is_ok() {
                        eprintln!(
                            "DIAG_OVERFLOW_CELL_CTX: section={} pi={} line={} ctx={cell_ctx:?}",
                            section_index, para_index, line_idx,
                        );
                    }
                }
            }
            // [Task #604 R3] wrap_anchor 가 있으면 본 문단은 anchor 그림/표 옆 wrap text.
            // 각 라인의 LineSeg cs(column_start)/sw(segment_width)를 x 오프셋/너비로 적용.
            // typeset 의 wrap_around state machine 매칭 결과 (ColumnContent.wrap_anchors)
            // 가 layout 에 전달되어 본 분기가 동작.
            //
            // [Task #722] inter-image-text gap 보정 — 한컴 viewer 는 anchor image 의
            // outer margin_right (HU) 만큼 cs 에 더해 text 시작 x 결정. sw 에서 동일량
            // 차감하여 가용 폭 정합. WrapAnchorRef.anchor_image_margin_right 활용.
            //
            // `LineSeg.sw`는 문단의 left/right margin을 포함한 source line box 폭이다.
            // 따라서 일반 stored-segment 경로와 마찬가지로 TextLine bbox의 usable width에서는
            // margin을 빼야 한다. wrap-anchor 경로가 `sw`를 그대로 override하면 hanging
            // indent가 image 쪽으로 한 번 더 돌출한다(HWP5 p127 그림 56 / p156 그림 64).
            let (line_cs_offset, line_avail_w_override) = if let Some(anchor) = wrap_anchor {
                let seg = para.and_then(|p| p.line_segs.get(line_idx));
                let cs = seg.map(|s| s.column_start as i32).unwrap_or(0);
                let sw = seg.map(|s| s.segment_width as i32).unwrap_or(0);
                let mr = anchor.anchor_image_margin_right;
                let cs_px = crate::renderer::hwpunit_to_px(cs + mr, self.dpi);
                let sw_px = if sw > 0 {
                    Some(
                        (crate::renderer::hwpunit_to_px((sw - mr).max(0), self.dpi)
                            - effective_margin_left
                            - effective_margin_right)
                            .max(0.0),
                    )
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
                    line_avail_w_override.unwrap_or(
                        effective_col_w - effective_margin_left - effective_margin_right,
                    ),
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
                        - effective_margin_right
                        - inline_offset
                        - num_offset,
                );
            // [Task #1472] IR indent 를 full 로 되돌리면서(parser/mod.rs) 미주 TAC 수식
            // available_width 의 effective indent 를 불변 유지: 변환본은 scale 을 절반으로.
            // (종전: IR(half)×2.0=full → 현재: IR(full)×1.0=full)
            let equation_indent_scale = (if cell_ctx.is_some() { 1.0 } else { 2.0 })
                * if self.profile.get().hwp3_layout() {
                    0.5
                } else {
                    1.0
                };
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
                        - effective_margin_right
                        - inline_offset
                        - num_offset,
                );
            let equation_continuation_available_width = line_avail_w_override
                .map(|w| w - inline_offset - num_offset)
                .unwrap_or(
                    effective_col_w
                        - equation_continuation_effective_margin_left
                        - effective_margin_right
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
            // Justify와 HWPX DISTRIBUTE_SPACE/HWP5 Split은 모두 공백에 배분하지만,
            // 마지막 줄 규칙은 다르다. Split(나눔 정렬)은 마지막 줄도 영역 끝까지
            // 배분한다. 머리말/꼬리말 Justify 단일 줄도 한컴처럼 공백을 벌린다.
            let is_header_footer_para = para_index >= usize::MAX - 1024;
            let needs_justify = needs_word_distribution(
                alignment,
                is_last_line_of_para,
                is_header_footer_para,
                has_forced_break,
            );
            let needs_distribute = alignment == Alignment::Distribute;
            // [#4516] 머리말/꼬리말 마지막 줄 예외로만 성립한 justify 는 공백에만
            // 배분한다 (공백 없는 줄은 자연 폭 유지).
            let justify_spaces_only = needs_justify
                && alignment == Alignment::Justify
                && is_last_line_of_para
                && is_header_footer_para;

            let has_tabs = comp_line.runs.iter().any(|r| r.text.contains('\t'));
            let renders_synthetic_wrap_trailing_space = !is_last_line_of_para
                && para
                    .and_then(|p| p.line_segs.get(line_idx))
                    .is_some_and(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
                && comp_line
                    .runs
                    .iter()
                    .rev()
                    .find_map(|run| run.text.chars().next_back())
                    == Some(' ');
            // 자간은 **그려지는 글자**에 나눠 붙으므로 폭(`total_text_width`)과 같은
            // 텍스트로 센다. 머리말 필드처럼 모델 1자가 표시 N자면 모델로 세었을 때
            // 글자당 몫이 N배로 부풀어 글자가 흩어진다 (Task #3216).
            let total_char_count: usize = comp_line
                .runs
                .iter()
                .map(|r| {
                    effective_text_for_metrics(r)
                        .chars()
                        .filter(|c| *c != '\t')
                        .count()
                })
                .sum();
            // [Issue #6196] 저장 사다리가 이 셀 문단을 **한 줄**로, 그것도 **셀 안쪽 폭
            // 그대로** 적어 두었으면 "한글이 이 문장을 이 폭에 담았다"는 증언이다.
            // 우리 폰트 메트릭의 자연 폭이 그보다 넓다고 압축을 억제하면 문장 꼬리가
            // 칸 밖으로 나가 잘린다(156543798 4쪽 `우수 내용` 칸 9행 중 7행 소실 —
            // 자연 폭 290~331px vs 저장 줄폭 229.2px).
            let stored_single_line_fits_cell = cell_ctx.is_some()
                && composed.lines.len() == 1
                && para.is_some_and(|p| {
                    p.line_segs.len() == 1
                        && p.line_segs[0].tag
                            & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                            == 0
                        && (hwpunit_to_px(p.line_segs[0].segment_width, self.dpi) - available_width)
                            .abs()
                            <= 2.0
                });
            // [#6389] 저장 사다리 증언의 다줄 일반화. 조합이 저장 줄 수를 그대로
            // 따랐고 모든 저장 줄폭이 셀 열폭 이내면, 한글이 이 내용을 이 폭에
            // 담았다는 증언이다 — 편람 p68 셀은 kopub/no-ttf 오라클 PDF 모두
            // 저장 줄과 문자 단위로 일치하는데, 내장 메트릭 진행폭이 실측(0.83em)
            // 보다 넓어(1.0em) 압축을 억제하면 `○` 문단 줄들이 셀 우측 테두리를
            // +72~85px 넘는다. 열폭보다 넓게 기록된 사다리(병합·재저장 안 된 낡은
            // 캐시)는 증언이 성립하지 않으므로 종전대로 억제(클리핑)한다.
            let stored_ladder_fits_frame = cell_ctx.is_some()
                && para.is_some_and(|p| {
                    !p.line_segs.is_empty()
                        && composed.lines.len() == p.line_segs.len()
                        && p.line_segs.iter().all(|seg| {
                            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                == 0
                                && hwpunit_to_px(seg.segment_width, self.dpi)
                                    <= effective_col_w + 2.0
                        })
                });
            let suppress_cell_overflow_spacing = cell_ctx.is_some()
                && total_text_width > available_width * 1.15
                && !stored_single_line_fits_cell
                && !stored_ladder_fits_frame;
            let is_hancom_company_pua_logo_line =
                is_hancom_company_pua_logo_line(comp_line, alignment);

            let (extra_word_sp, extra_char_sp, extra_dash_sp) = if is_hancom_company_pua_logo_line {
                // 이 줄의 trailing space는 뒤의 treat-as-char logo 그림 앞 공백이다.
                // 회사명 자체에는 자간을 추가하지 않고 이 공백 하나가 남는 폭을 전부
                // 흡수하게 해야 Hancom PDF의 좌측 회사명·우측 logo 배치가 유지된다.
                ((available_width - total_text_width).max(0.0), 0.0, 0.0)
            } else {
                compute_line_extra_spacing(
                    comp_line,
                    styles,
                    alignment,
                    cell_ctx.is_some(),
                    needs_justify,
                    justify_spaces_only,
                    needs_distribute,
                    has_tabs,
                    renders_synthetic_wrap_trailing_space,
                    suppress_cell_overflow_spacing,
                    total_char_count,
                    total_text_width,
                    available_width,
                    tab_width,
                )
            };

            let line_plain_text: String = comp_line.runs.iter().map(|r| r.text.as_str()).collect();
            let is_answer_sheet_number_label =
                cell_ctx.is_some() && line_plain_text.trim() == "수험번호";
            // [Task #1308 CI follow-up / #1256 regression]
            // 본문/미주 흐름의 TAC 수식-only 줄은 저장된 LINE_SEG x 흐름을 따라야 한다.
            // 빈 TextRun 이 있는 수식-only 문단은 일반 정렬 경로로 들어오므로,
            // Distribute/Center 의 잔여 폭 중앙 오프셋을 적용하면 한컴과 달리 수식 블록이
            // 열 안쪽으로 밀린다. 그림/표 TAC는 문단 정렬 폭을 따라야 하며, 표 셀 안 수식은
            // 기존처럼 셀 정렬을 따른다.
            let non_cell_tac_only_line = cell_ctx.is_none()
                && !line_tac_offsets_for_width.is_empty()
                && line_plain_text.trim().is_empty()
                && line_tac_offsets_for_width.iter().any(|(_, _, ci)| {
                    is_treat_as_char_equation_control(para.and_then(|p| p.controls.get(*ci)))
                });

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
            // 한글은 셀 밖 오른쪽/가운데 정렬 폭에서 말미 공백을 제외한다
            // (needs_justify 의 후행 공백 제외와 동일 규칙). 포함하면
            // [그림+말미공백72] 꼬리말이 공백 폭(447px)만큼 왼쪽으로 이탈 —
            // 식약처 보도자료 OPEN 로고 실측(한글 x=607.3). Center 는 30213
            // 의결서 위원 서명 줄 실측(말미 공백 8칸 포함 줄만 한글 대비 43px
            // 좌측 이탈, 한글 PDF x=229.56pt 는 공백 제외 중심). 반례 셋으로
            // 한정한다: ① 셀 내부는 한글이 말미 공백을 포함해 정렬(issue_1285
            // 수험번호 TAC 우단 = 셀 inner 우단 오라클 앵커) — cell_ctx 부재.
            // ② soft-wrap 지점의 줄끝 공백은 포함 — 문단 마지막 줄 한정.
            // ③ TAC 컨트롤이 있는 줄은 공백이 시각적 말미가 아니다 —
            // line_tac_offsets_for_width 비어 있을 때 한정. ④ 전부 공백인
            // 줄(밑줄 친 서명란)과 밑줄 스타일 말미 공백은 보이는 콘텐츠라
            // 유지(issue_157 직선 골든 — 제외하면 우측 클립까지 이탈).
            let center_excludes_trailing_ws = alignment == Alignment::Center
                && cell_ctx.is_none()
                && is_last_line_of_para
                && line_tac_offsets_for_width.is_empty()
                && comp_line
                    .runs
                    .iter()
                    .any(|r| r.text.chars().any(|c| c != ' '));
            // [#5820] 글상자(drawText) 안 문단은 표 셀이 아니다 — 한글은 글상자
            // 안에서도 오른쪽 정렬의 말미 공백을 제외한다(156560092 글상자:
            // [로고A][로고B][공백5] RIGHT 문단 — 한글 로고 우변 여백 4.1px,
            // 포함 시 공백 폭 32.7px 만큼 좌측 이탈). 셀 내부 포함-정렬 계약
            // (issue_1285)은 in_textbox=false 로 그대로 유지된다.
            let right_align_excludes_trailing_ws =
                alignment == Alignment::Right && cell_ctx.as_ref().is_none_or(|c| c.in_textbox);
            let trailing_ws_width =
                if right_align_excludes_trailing_ws || center_excludes_trailing_ws {
                    trailing_space_width_after_last_inline_object(
                        comp_line,
                        line_tac_offsets_for_width
                            .iter()
                            .map(|(pos, _, _)| *pos)
                            .max(),
                        styles,
                        // ④ 밑줄 친 말미 공백은 보이는 콘텐츠 — Center 는 제외 대상에서
                        // 뺀다(Right 는 기존 검증 동작 유지).
                        center_excludes_trailing_ws,
                    )
                } else {
                    0.0
                };
            let x_start = match alignment {
                Alignment::Center => {
                    let align_offset = if center_packed_cell_label_as_right {
                        (available_width - effective_text_width).max(0.0)
                    } else if non_cell_tac_only_line {
                        0.0
                    } else {
                        (available_width - (effective_text_width - trailing_ws_width)).max(0.0)
                            / 2.0
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
                        + (available_width - (effective_text_width - trailing_ws_width)).max(0.0)
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
                            layout_positions: None,
                            display_text: None,
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
            let shape_markers: Vec<(usize, String)> = collect_shape_marker_labels(show_ctrl, para);

            // 각주 마커 위치 수집
            let fn_positions: &[(usize, u16, usize)] = &composed.footnote_positions;
            let mut fn_marker_inserted = vec![false; fn_positions.len()];

            let mut pending_right_tab_render: Option<(f64, u8, u8)> = None;
            let mut pending_right_leader_digit_render = false;
            let mut run_char_pos = comp_line.char_start;
            // 이미 삽입한 도형 마커 추적
            let mut shape_marker_inserted = vec![false; shape_markers.len()];
            // cross-run 탭 감지용 inline_tabs(composed.tab_extended) 커서 — Task #290
            let mut inline_tab_cursor_render: usize = 0;
            let emit_state = self.emit_line_runs(
                tree,
                &mut line_node,
                col_node,
                comp_line,
                composed,
                para,
                bin_data_content,
                styles,
                &cell_ctx,
                &tab_stops,
                &tac_offsets_px,
                &shape_markers,
                fn_positions,
                &mut fn_marker_inserted,
                &mut shape_marker_inserted,
                &mut char_x_map,
                para_topbottom_line_vpos_base,
                col_area,
                &mut kerning_layout_session,
                RunEmitVars {
                    baseline,
                    raw_lh,
                    alignment,
                    auto_tab_right,
                    available_width,
                    effective_margin_left,
                    end,
                    extra_char_sp,
                    extra_dash_sp,
                    extra_word_sp,
                    has_tabs,
                    horizontal_shaping_initial_lane,
                    is_last_line_of_para,
                    line_height,
                    line_idx,
                    line_spacing_px,
                    max_fs,
                    runs_all_whitespace,
                    renders_synthetic_wrap_trailing_space,
                    start_line,
                    tab_width,
                    section_index,
                    para_index,
                },
                RunEmitState {
                    x,
                    y,
                    char_offset,
                    run_char_pos,
                    inline_tab_cursor_render,
                    pending_right_tab_render,
                    pending_right_leader_digit_render,
                    current_line_reserved_tac_picture_height,
                },
            );
            x = emit_state.x;
            y = emit_state.y;
            char_offset = emit_state.char_offset;
            run_char_pos = emit_state.run_char_pos;
            inline_tab_cursor_render = emit_state.inline_tab_cursor_render;
            pending_right_tab_render = emit_state.pending_right_tab_render;
            pending_right_leader_digit_render = emit_state.pending_right_leader_digit_render;
            current_line_reserved_tac_picture_height =
                emit_state.current_line_reserved_tac_picture_height;

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
                            layout_positions: None,
                            display_text: None,
                        }),
                        BoundingBox::new(x, y, mw, line_height),
                    );
                    line_node.children.push(mn);
                    x += mw;
                }
            }

            x = self.place_unmatched_line_tac_pictures(
                tree,
                &mut line_node,
                comp_line,
                para,
                bin_data_content,
                &tac_offsets_px,
                col_area,
                cell_ctx.as_ref(),
                &mut current_line_reserved_tac_picture_height,
                TacPictureLineVars {
                    run_char_pos,
                    x,
                    y,
                    baseline,
                    raw_lh,
                    section_index,
                    para_index,
                },
            );

            x = self.place_empty_line_tac_forms(
                tree,
                &mut line_node,
                comp_line,
                para,
                &tac_offsets_px,
                cell_ctx.as_ref(),
                x,
                y,
                baseline,
                section_index,
                para_index,
            );

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
                    col_area,
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
                        line_idx,
                    },
                    &mut current_line_reserved_tac_picture_height,
                );
            }

            // [Task #287] 빈 runs 줄의 TAC 수식 인라인 처리 — #2067 추출
            self.place_empty_line_inline_equations(
                tree,
                &mut line_node,
                comp_line,
                composed,
                para,
                styles,
                &cell_ctx,
                &tac_offsets_px,
                &line_tac_offsets,
                &equation_tac_line_flow,
                EquationTacLineVars {
                    line_idx,
                    line_end: end,
                    alignment,
                    available_width,
                    margin_left,
                    indent,
                    effective_col_x,
                    y,
                    baseline,
                    line_height,
                    line_spacing_px,
                    col_area_y: col_area.y,
                    col_bottom,
                    line_char_end: char_offset,
                    is_last_line_of_para,
                    defer_empty_line_control_marker,
                    equation_tac_extra_rows,
                    hwp3_indent_scale: if self.profile.get().hwp3_layout() {
                        0.5
                    } else {
                        1.0
                    },
                    section_index,
                    para_index,
                },
            );

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
                    composed
                        .lines
                        .get(line_idx + 1)
                        .is_some_and(|next| next.char_start == char_offset),
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
                current_line_reserved_tac_picture_height = para.and_then(|p| {
                    crate::renderer::line_owning_tac_object_height_px(p, raw_lh, self.dpi)
                });
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
                && !has_ole_shape_square_wrap
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
                last_line_box_bottom = Some(y + line_flow_height);
                y += line_flow_height;
            } else if skip_advance_empty_line {
                // no advance
            } else {
                last_line_box_bottom = Some(y + render_line_flow_height);
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
            // [#5711] 줄간격이 음수인 문단은 전진값 `y` 가 마지막 줄 상자 아래보다 위에
            // 있다. 그 값을 테두리 아래 변으로 쓰면 테두리가 글자를 가로지른다. 다음 문단
            // 시작 y 는 종전대로 두어 문단 간 간격 계약은 바꾸지 않는다.
            let border_bottom = last_line_box_bottom.map_or(y, |bottom| y.max(bottom));
            let bg_height = border_bottom - bg_y_start;
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
                    border_bottom,
                    top_inset,
                    bottom_inset,
                    is_partial_start,
                    is_partial_end,
                    para_index,
                ));
            }
        }

        // ComposedLine이 없으면 빈 TextRun 생성 (편집용). `compose_paragraph()`는
        // 빈 문단에 줄을 만들지 않을 수 있는데, 종전 400HU 고정 advance는
        // pagination의 NO_LS 빈 문단 메트릭과 달라 다음 표/문단을 위로 끌어올렸다.
        // 이 경로도 원래 글자모양·줄간격을 사용해 두 경로를 일치시킨다 (#3820 p81–82).
        if composed.lines.is_empty() && start_line == 0 {
            let (default_height, default_spacing) = para
                .and_then(|p| {
                    empty_no_lineseg_paragraph_metrics(
                        p,
                        styles,
                        para_style,
                        self.profile.get().hwp3_layout(),
                        self.dpi,
                    )
                })
                .map(|(line_height, line_spacing, _)| (line_height, line_spacing))
                .unwrap_or((hwpunit_to_px(400, self.dpi), 0.0));
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
                    layout_positions: None,
                    display_text: None,
                }),
                BoundingBox::new(col_area.x, y, col_area.width, default_height),
            );
            line_node.children.push(run_node);

            col_node.children.push(line_node);
            y += default_height + default_spacing;
        }

        // 문단 뒤 간격 (spacing_after). 빈 composed 문단도 실제 한 줄 advance 뒤에
        // 적용해야 일반 composed 문단과 동일한 순서를 따른다.
        if spacing_after > 0.0 && end == composed.lines.len() {
            y += spacing_after;
        }

        y
    }

    /// [#2003 추출] 줄의 run 방출 루프 — TextRun/글리프/탭/밑줄/인라인 개체 방출.
    /// 줄-간 캐리오버는 `RunEmitState` 값 왕복, 읽기 스칼라는 `RunEmitVars` —
    /// 진입 destructure 로 본문 무변경 이동을 보장한다.
    #[allow(clippy::too_many_arguments)]
    fn emit_line_runs(
        &self,
        tree: &mut PageLayoutContext,
        line_node: &mut RenderNode,
        col_node: &mut RenderNode,
        comp_line: &crate::renderer::composer::ComposedLine,
        composed: &ComposedParagraph,
        para: Option<&Paragraph>,
        bin_data_content: Option<&[BinDataContent]>,
        styles: &ResolvedStyleSet,
        cell_ctx: &Option<CellContext>,
        tab_stops: &[TabStop],
        tac_offsets_px: &[(usize, f64, usize)],
        shape_markers: &[(usize, String)],
        fn_positions: &[(usize, u16, usize)],
        fn_marker_inserted: &mut [bool],
        shape_marker_inserted: &mut [bool],
        char_x_map: &mut Vec<(usize, f64)>,
        para_topbottom_line_vpos_base: Option<(i32, f64)>,
        col_area: &LayoutRect,
        kerning_layout_session: &mut KerningLayoutSession<'_>,
        vars: RunEmitVars,
        st: RunEmitState,
    ) -> RunEmitState {
        let RunEmitVars {
            baseline,
            raw_lh,
            alignment,
            auto_tab_right,
            available_width,
            effective_margin_left,
            end,
            extra_char_sp,
            extra_dash_sp,
            extra_word_sp,
            has_tabs,
            horizontal_shaping_initial_lane,
            is_last_line_of_para,
            line_height,
            line_idx,
            line_spacing_px,
            max_fs,
            runs_all_whitespace,
            renders_synthetic_wrap_trailing_space,
            start_line,
            tab_width,
            section_index,
            para_index,
        } = vars;
        let RunEmitState {
            mut x,
            y,
            mut char_offset,
            mut run_char_pos,
            mut inline_tab_cursor_render,
            mut pending_right_tab_render,
            mut pending_right_leader_digit_render,
            mut current_line_reserved_tac_picture_height,
        } = st;
        let is_last_run_of_line = |idx: usize| idx == comp_line.runs.len() - 1;
        // [#5679] 줄-말미 공백에 배정된 배분 여분(extra_word_sp) 회수분.
        // 배분 몫은 **내부 공백 수**로 나눈다(위 needs_justify 분기의
        // rendered_space_slots) — 줄-말미 공백은 est 의 effective_used 에서도
        // 빠져 있다. 그런데 char_width_decision 은 모든 ' ' 에 여분을 붙이므로,
        // 말미 공백이 여분까지 얹어 그려져 run bbox 와 x 전진이 줄 상자를
        // 여분×말미공백수 만큼 넘는다(10857 p11: '외부 평가전문위원 ' 144.0 vs
        // 줄 122.1 — 가시 글리프는 정확히 줄 끝에서 끝나고 초과 전량이 부풀린
        // 말미 공백). 분모에 말미 공백을 넣는 synthetic-wrap 모드는 제외.
        let line_trailing_space_by_run: Vec<usize> = {
            let mut counts = vec![0usize; comp_line.runs.len()];
            if !renders_synthetic_wrap_trailing_space && extra_word_sp != 0.0 {
                let mut budget = comp_line
                    .runs
                    .iter()
                    .flat_map(|r| effective_text_for_metrics(r).chars())
                    .collect::<Vec<char>>()
                    .iter()
                    .rev()
                    .take_while(|c| **c == ' ')
                    .count();
                for (ri, r) in comp_line.runs.iter().enumerate().rev() {
                    if budget == 0 {
                        break;
                    }
                    let rt = effective_text_for_metrics(r);
                    let chars_total = rt.chars().count();
                    let tail_sp = rt.chars().rev().take_while(|c| *c == ' ').count();
                    let take = tail_sp.min(budget);
                    counts[ri] = take;
                    budget -= take;
                    if take < chars_total {
                        break;
                    }
                }
            }
            counts
        };
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
                            layout_positions: None,
                            display_text: None,
                        }),
                        BoundingBox::new(x, y, mw, line_height),
                    );
                    line_node.children.push(mn);
                    x += mw;
                }
            }
            let mut text_style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
            text_style.default_tab_width = tab_width;
            text_style.tab_stops = tab_stops.to_vec();
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
                        if let RenderNodeType::TextRun(prev_text_run) = &mut prev_run_node.node_type
                        {
                            let space_gap = if text_style.font_size > 0.0 {
                                text_style.font_size * 0.25
                            } else {
                                3.0
                            };
                            for leader in &mut prev_text_run.style.tab_leaders {
                                let new_end_x = (x - prev_bbox_x - space_gap).max(leader.start_x);
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
            } else if run.display_text.is_some()
                && run.text.chars().count() == 1
                && matches!(
                    run.text.chars().next(),
                    Some('\u{0015}' | '\u{0016}' | '\u{0017}' | '\u{2007}')
                )
            {
                // 필드 marker 한 글자와 표시 문자열의 폭이 소수 px일 수 있다. 이 런은
                // 다음 조각과 분리되어 있으므로 정수 반올림을 하면 뒤의 fwSpace/텍스트
                // 앵커가 SVG 실제 glyph advance보다 앞선다 (#3216, #1100). field 런만
                // 비반올림 폭을 써서 모델 한 글자 경계와 표시 끝을 같은 좌표에 둔다.
                estimate_text_width_unrounded(effective_text_for_metrics(run), &text_style)
            } else {
                estimate_text_width(effective_text_for_metrics(run), &text_style)
            };
            // [#5679] 줄-말미 공백의 배분 여분 회수 — 자연 폭은 유지한다(한글도
            // 말미 공백 자체는 줄 상자를 넘길 수 있다). 여분이 음수(압축)여도
            // est 가 말미 공백을 제외했으므로 동일하게 회수한다.
            let full_width =
                full_width - extra_word_sp * line_trailing_space_by_run[run_idx] as f64;
            // 각주/TAC/탭은 최종 TextRun 경계를 추가로 만든다. whole-run pair를
            // 먼저 적용하면 그 경계를 가로지르는 delta가 남으므로, sub-run
            // producer가 연결될 때까지 이 특수 run은 원자적으로 K0로 닫는다.
            let run_char_count_for_boundary = if run.char_overlap.is_some() {
                let chars: Vec<char> = run.text.chars().collect();
                crate::renderer::composer::char_overlap_advance_units(&chars)
            } else {
                run.text.chars().count()
            };
            let run_char_end_for_boundary = run_char_pos + run_char_count_for_boundary;
            let has_tac_boundary = tac_offsets_px.iter().any(|(position, _, _)| {
                *position >= run_char_pos && *position <= run_char_end_for_boundary
            });
            let has_note_boundary = fn_positions.iter().any(|(position, _, _)| {
                *position >= run_char_pos && *position <= run_char_end_for_boundary
            });
            let exact_replay_eligible = run.char_overlap.is_none()
                && !run
                    .text
                    .chars()
                    .any(|character| matches!(character, '\t' | '\n' | '\r'))
                && !has_tac_boundary
                && !has_note_boundary;
            let shaping_candidate = horizontal_shaping_initial_lane
                && run_idx == 0
                && run_char_pos == 0
                && char_offset == 0
                && !has_tabs
                && tac_offsets_px.is_empty()
                && fn_positions.is_empty()
                && shape_markers.is_empty()
                && run_border_fill_id == 0
                && extra_word_sp.abs() <= f64::EPSILON
                && extra_char_sp.abs() <= f64::EPSILON
                && extra_dash_sp.abs() <= f64::EPSILON
                && !renders_synthetic_wrap_trailing_space;
            // NodeId를 먼저 고정하되 attach가 실패하면 같은 id로 legacy TextRun을
            // 만든다. 따라서 실패는 id hole이나 K1 suppression을 남기지 않는다.
            let reserved_shaping_run_id = shaping_candidate.then(|| tree.next_id());
            let shaping_width = reserved_shaping_run_id.and_then(|node_id| {
                para.and_then(|para| {
                    attach_horizontal_shaping_initial_lane(
                        tree,
                        composed,
                        para,
                        styles,
                        run,
                        node_id,
                        run_char_pos,
                        x,
                    )
                })
            });
            let (full_width, layout_positions) = if let Some(shaping_width) = shaping_width {
                (shaping_width, None)
            } else {
                emitted_run_layout_positions(
                    kerning_layout_session,
                    ExactFontSlot::new(run.char_style_id, run.lang_index),
                    effective_text_for_metrics(run),
                    &text_style,
                    full_width,
                    line_trailing_space_by_run[run_idx],
                    exact_replay_eligible,
                )
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
                        // [#5727] 저장 lineseg 가 개체에 배정한 빈 줄이 소유한 경계
                        // TAC 는 다음 줄 run 에 다시 싣지 않는다 — 실으면 개체가 이
                        // 줄로 끌려 내려오고 텍스트가 개체 폭만큼 오른쪽으로 밀린다.
                        && !tac_owned_by_prior_empty_line(composed, line_idx, *pos)
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
                let leading_spaces: String = run.text.chars().take_while(|c| *c == ' ').collect();
                let content = run.text.trim_start_matches(' ');

                // 글자 테두리/배경: bbox 계산용 run_x, run_w
                let (run_x, run_w) = if !leading_spaces.is_empty() && !content.is_empty() {
                    let leading_count = leading_spaces.chars().count();
                    if let Some(positions) = layout_positions.as_deref() {
                        let leading_end = positions.get(leading_count).copied();
                        let run_end = positions.last().copied();
                        if let (Some(leading_end), Some(run_end)) = (leading_end, run_end) {
                            (x + leading_end, run_end - leading_end)
                        } else {
                            let sw = estimate_text_width(&leading_spaces, &text_style);
                            (x + sw, estimate_text_width(content, &text_style))
                        }
                    } else {
                        let sw = estimate_text_width(&leading_spaces, &text_style);
                        (x + sw, estimate_text_width(content, &text_style))
                    }
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
                            let relative_start = overlap_start - run_char_pos;
                            let relative_end = overlap_end - run_char_pos;
                            let exact_range = layout_positions.as_deref().and_then(|positions| {
                                if positions.len() != run.text.chars().count().saturating_add(1) {
                                    return None;
                                }
                                Some((
                                    *positions.get(relative_start)?,
                                    *positions.get(relative_end)?,
                                ))
                            });
                            let (hl_x, hl_w) = if let Some((start, end)) = exact_range {
                                (x + start, end - start)
                            } else {
                                (
                                    run_x + relative_start as f64 * char_w,
                                    (relative_end - relative_start) as f64 * char_w,
                                )
                            };
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
                let mut emitted_text_width = full_width;
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
                        let run_x = x;
                        let run_id = reserved_shaping_run_id.unwrap_or_else(|| tree.next_id());
                        let run_node = RenderNode::new(
                            run_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: run.text.clone(),
                                display_text: run.display_text.clone(),
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
                                layout_positions,
                            }),
                            BoundingBox::new(run_x, y, full_width, line_height),
                        );
                        line_node.children.push(run_node);
                    } else {
                        // 각주 있음: run을 각주 위치에서 분할하여 TextRun + FootnoteMarker 교차 생성
                        let run_chars: Vec<char> = run.text.chars().collect();
                        let mut seg_start = 0usize; // run 내 상대 문자 인덱스
                        let mut sub_x = x;
                        let mut sub_char_offset = char_offset;
                        emitted_text_width = 0.0;

                        for &(rel_pos, fnum, fni, ctrl_idx) in &run_fn_markers {
                            fn_marker_inserted[fni] = true;
                            // 각주 앞 텍스트 세그먼트
                            if rel_pos > seg_start {
                                let seg_text: String =
                                    run_chars[seg_start..rel_pos].iter().collect();
                                let seg_w = estimate_text_width(&seg_text, &text_style);
                                let (seg_w, seg_layout_positions) = emitted_run_layout_positions(
                                    kerning_layout_session,
                                    ExactFontSlot::new(run.char_style_id, run.lang_index),
                                    &seg_text,
                                    &text_style,
                                    seg_w,
                                    0,
                                    run.char_overlap.is_none()
                                        && !seg_text.chars().any(|character| {
                                            matches!(character, '\t' | '\n' | '\r')
                                        }),
                                );
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
                                        layout_positions: seg_layout_positions,
                                        display_text: None,
                                    }),
                                    BoundingBox::new(sub_x, y, seg_w, line_height),
                                );
                                line_node.children.push(seg_node);
                                sub_x += seg_w;
                                emitted_text_width += seg_w;
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
                            let trailing_space_count = line_trailing_space_by_run[run_idx]
                                .min(run_chars.len().saturating_sub(seg_start));
                            let (seg_w, seg_layout_positions) = emitted_run_layout_positions(
                                kerning_layout_session,
                                ExactFontSlot::new(run.char_style_id, run.lang_index),
                                &seg_text,
                                &text_style,
                                seg_w - extra_word_sp * trailing_space_count as f64,
                                trailing_space_count,
                                run.char_overlap.is_none()
                                    && !seg_text
                                        .chars()
                                        .any(|character| matches!(character, '\t' | '\n' | '\r')),
                            );
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
                                    layout_positions: seg_layout_positions,
                                    display_text: None,
                                }),
                                BoundingBox::new(sub_x, y, seg_w, line_height),
                            );
                            line_node.children.push(seg_node);
                            emitted_text_width += seg_w;
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
                            let nodes =
                                create_border_line_nodes(tree, &bs.borders[bi], lx1, ly1, lx2, ly2);
                            for n in nodes {
                                line_node.children.push(n);
                            }
                        }
                    }
                }

                x += emitted_text_width + fn_split_extra;
            } else {
                // tac 있음: 분기점마다 하위 텍스트 런 생성 (이미지는 layout.rs에서 별도 렌더링)
                let run_chars: Vec<char> = run.text.chars().collect();
                let mut seg_start = 0usize;
                let mut sub_char_offset = char_offset;

                // [Task #455] 외부 문단 본문 텍스트는 글상자 유무와 무관하게 항상 렌더한다.
                // 글상자(TextBox) 자체와 그 내부 텍스트("개화" 같은)는
                // shape_layout 이 inline_shape_position 을 보고 별도 패스에서 렌더하므로 중복되지 않는다.

                for &(tac_rel, tac_w, tac_ci) in &run_tacs {
                    // [Issue #3396] 한글은 TAC 표를 "outMargin 포함 폭의 문자"로
                    // 배치한다 — 괘선(테두리)은 pen + outMargin.left 에 그려지고,
                    // 다음 문자는 outMargin.right 뒤에서 시작한다 (오라클 실측:
                    // 156678235 JUSTIFY 표 좌측 괘선 = 흐름 x + om_l). tac_w
                    // (composer 열폭 합)는 측정 경로 공유값이라 여기 렌더 전진에서만
                    // 보정한다. 아래 표 분기에서 채워진다.
                    let mut tac_table_om = (0.0f64, 0.0f64);
                    // tac 앞 텍스트 세그먼트 렌더링
                    if seg_start < tac_rel {
                        let seg_text: String = run_chars[seg_start..tac_rel].iter().collect();
                        let mut seg_style = text_style.clone();
                        seg_style.line_x_offset = x - col_area.x;
                        // [Issue #6179] 이 조각의 마지막 탭 뒤에 TAC 개체가 오면,
                        // 되밀기 폭에 그 개체 폭을 포함시킨다 (조각 경계로 잘려
                        // 측정 쪽에서는 보이지 않는다).
                        if auto_tab_right && seg_text.contains('\t') {
                            let tab_rel = seg_start
                                + run_chars[seg_start..tac_rel]
                                    .iter()
                                    .rposition(|c| *c == '\t')
                                    .expect("seg_text 가 탭을 포함한다");
                            seg_style.right_tab_block_width_override =
                                right_tab_block_width_with_tac(
                                    &run_chars, tab_rel, &run_tacs, &seg_style,
                                );
                        }
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
                        let (seg_w, seg_layout_positions) = emitted_run_layout_positions(
                            kerning_layout_session,
                            ExactFontSlot::new(run.char_style_id, run.lang_index),
                            &seg_text,
                            &seg_style,
                            seg_w,
                            0,
                            run.char_overlap.is_none()
                                && !seg_text
                                    .chars()
                                    .any(|character| matches!(character, '\t' | '\n' | '\r')),
                        );
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
                                    layout_positions: seg_layout_positions,
                                    display_text: None,
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
                                let (_, pic_h) = self.resolve_inline_picture_size(pic, col_area);
                                // LINE_SEG vpos가 TopAndBottom 흐름 위치를 이미 담고 있으면
                                // sibling 예약 높이를 다시 더하지 않는다.
                                let sibling_reserved_px = if para_topbottom_line_vpos_base.is_some()
                                {
                                    0.0
                                } else {
                                    let raw = hwpunit_to_px(
                                        calc_sibling_topandbottom_reserved_hu(&p.controls),
                                        self.dpi,
                                    );
                                    // 줄 y 가 이미 형제 자리차지 예약 아래(최종 좌표)면
                                    // 이중 가산 금지 — host 문단의 꼬리 줄이 저장 vpos
                                    // 스냅으로 표 아래(쪽 하단)에 이미 놓였는데 표 높이
                                    // 를 또 더하면 tac 그림이 줄보다 예약 높이만큼 아래
                                    // (쪽 밖, #6271 실측 y=2113px > 단 하단 1115px)에
                                    // 그려져 소실된다.
                                    if raw > 40.0 && y >= col_area.y + raw - 4.0 {
                                        0.0
                                    } else {
                                        raw
                                    }
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
                                let image_data = find_bin_data_bytes(bdc, bin_data_id);
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
                                let original_size_hu = pic.crop_reference_size();
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
                            let shape_h = hwpunit_to_px(shape.flow_height_hu(), self.dpi);
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
                            let (eq_cell_idx, eq_cell_para_idx) = if let Some(ref ctx) = cell_ctx {
                                (
                                    ctx.path.first().map(|e| e.cell_index),
                                    ctx.path.first().map(|e| e.cell_para_index),
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
                                        script: eq.script.clone(),
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
                                            ctx.path
                                                .first()
                                                .map(|e| e.control_index)
                                                .or(Some(tac_ci))
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
                                || crate::renderer::height_measurer::is_tac_table_inline_in_para(
                                    t, seg_width, p,
                                );
                            let already_rendered = tree
                                .get_inline_shape_position(
                                    section_index,
                                    para_index,
                                    tac_ci,
                                    cell_ctx.as_ref(),
                                )
                                .is_some();
                            if t.common.treat_as_char && should_render_inline {
                                // [Issue #3396] 렌더 여부와 무관하게 이 줄에서 표가
                                // 문자로 취급되면 전진 폭에 outMargin 좌/우를 포함.
                                tac_table_om = (
                                    hwpunit_to_px(t.outer_margin_left as i32, self.dpi),
                                    hwpunit_to_px(t.outer_margin_right as i32, self.dpi),
                                );
                            }
                            if t.common.treat_as_char && should_render_inline && !already_rendered {
                                let table_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                                let om_top = hwpunit_to_px(t.outer_margin_top as i32, self.dpi);
                                let om_bottom =
                                    hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi);
                                // [#3386] 저장 lh 가 표+상하 외곽여백을 수용하는 줄
                                // (한글이 lh = h + om 으로 저장한 표 전용 줄)은 표
                                // 상단 = 줄 상단 + om_top 이 한글 실좌표다 (156678235
                                // p5: 저장 vpos+om_top == 한글 PDF 상단, 종전 baseline
                                // 하단정렬식은 om_top 을 소실해 3.8px 상향). #2220 의
                                // stored_lh_covers_om 과 동일 술어의 px 판.
                                let stored_lh_covers_om = (om_top > 0.0 || om_bottom > 0.0)
                                    && raw_lh >= table_h + om_top + om_bottom - 0.2;
                                let table_y = if stored_lh_covers_om {
                                    y + om_top
                                } else {
                                    (y + baseline + om_bottom - table_h).max(y)
                                };
                                // [Task #2212] 셀 안 인라인 TAC 표는 외곽 셀 경로를
                                // 확장한 2단 cell_context 로 렌더해야 경로 기반 조회
                                // (get_table_cell_bboxes_by_path 등)가 내부 셀을 찾는다.
                                // table_layout 중첩 분기(:3475)와 동일한 확장 규칙 —
                                // 내부 entry 의 cell/cp 는 layout_table 셀 루프가 채운다.
                                let nested_ctx = cell_ctx.as_ref().map(|ctx| {
                                    let mut c = ctx.clone();
                                    c.path.push(crate::renderer::layout::CellPathEntry {
                                        control_index: tac_ci,
                                        cell_index: 0,
                                        cell_para_index: 0,
                                        text_direction: 0,
                                    });
                                    c
                                });
                                let nested_depth = usize::from(cell_ctx.is_some());
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
                                    nested_depth,
                                    Some((para_index, tac_ci)),
                                    alignment,
                                    nested_ctx,
                                    0.0,
                                    0.0,
                                    Some(x + tac_table_om.0),
                                    None,
                                    None,
                                    None,
                                    false,
                                    false,
                                    false,
                                );
                                // 스킵 마커 등록 (별도 Table PageItem에서 중복 렌더 방지)
                                tree.set_inline_shape_position(
                                    section_index,
                                    para_index,
                                    tac_ci,
                                    cell_ctx.as_ref(),
                                    x + tac_table_om.0,
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
                            // 셀 내부인 경우 cell_location 채우기 — 빈 경로면 None
                            let cell_location = cell_ctx.as_ref().and_then(|ctx| {
                                ctx.path.first().map(|e| {
                                    (
                                        ctx.parent_para_index,
                                        e.control_index,
                                        e.cell_index,
                                        e.cell_para_index,
                                    )
                                })
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
                    // tac 폭만큼 x 전진 (+ TAC 표 outMargin 좌/우 — Issue #3396)
                    x += tac_w + tac_table_om.0 + tac_table_om.1;
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
                    let trailing_space_count =
                        line_trailing_space_by_run[run_idx].min(remaining.chars().count());
                    let (seg_w, seg_layout_positions) = emitted_run_layout_positions(
                        kerning_layout_session,
                        ExactFontSlot::new(run.char_style_id, run.lang_index),
                        &remaining,
                        &seg_style,
                        seg_w - extra_word_sp * trailing_space_count as f64,
                        trailing_space_count,
                        run.char_overlap.is_none()
                            && !remaining
                                .chars()
                                .any(|character| matches!(character, '\t' | '\n' | '\r')),
                    );
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
                                layout_positions: seg_layout_positions,
                                display_text: None,
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
                            layout_positions: None,
                            display_text: None,
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
        RunEmitState {
            x,
            y,
            char_offset,
            run_char_pos,
            inline_tab_cursor_render,
            pending_right_tab_render,
            pending_right_leader_digit_render,
            current_line_reserved_tac_picture_height,
        }
    }

    /// [#1925 추출] ClickHere 필드 처리(안내문, [누름틀 시작/끝] 조판부호 마커)와
    /// 책갈피 조판부호 마커. char_x_map 보간으로 필드 위치의 x 좌표를 계산해
    /// 마커 노드를 삽입하고, 마커 폭만큼 기존 노드를 오른쪽으로 shift 한다.
    /// 반환값 accumulated_shift 는 caller 가 라인 커서 x 에 가산한다.
    #[allow(clippy::too_many_arguments)]
    fn layout_click_here_and_bookmark_markers(
        &self,
        tree: &mut PageLayoutContext,
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
        // [#6111] 다음 줄이 이 줄의 끝 문자에서 시작하는가. 그렇다면 그 경계
        // 문자에 걸린 누름틀은 **다음 줄**이 소유한다 — 같은 파일의 TAC 계약
        // (`next_line_starts_at_run_end`)과 같은 규칙이다.
        next_line_starts_at_line_end: bool,
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
        // [#6111] 접힌 안내문의 둘째 조각부터는 마커 shift 대상이 아니다 —
        // shift 를 끝낸 뒤 원래 x 그대로 얹는다.
        let mut wrapped_guide_overlays: Vec<RenderNode> = Vec::new();

        for fr in &p.field_ranges {
            if let Some(Control::Field(field)) = p.controls.get(fr.control_idx) {
                if field.field_type != crate::model::control::FieldType::ClickHere {
                    continue;
                }
                let is_empty = fr.start_char_idx == fr.end_char_idx;
                // [#6111] 줄 경계 문자에 걸린 누름틀은 **다음 줄**이 소유한다.
                //
                // 줄 끝 문자는 다음 줄의 시작 문자이기도 해서, 두 줄이 같은
                // 누름틀을 각자 그렸다 — 56345 7쪽은 빈 누름틀 안내문이 두 줄에
                // 중복되고, 그중 앞 줄은 **배분 정렬된 줄의 마지막 문자**라
                // char_x_map 이 본문 우단(718.6px)을 돌려줘 안내문이 쪽 밖으로
                // 나갔다. 같은 파일의 TAC 계약(`next_line_starts_at_run_end`)과
                // 같은 규칙으로 앞 줄의 소유권을 넘긴다.
                let owns_boundary = !next_line_starts_at_line_end;
                let start_in_line = fr.start_char_idx >= line_char_start
                    && (fr.start_char_idx < line_char_end
                        || (fr.start_char_idx == line_char_end && owns_boundary));
                let end_in_line = fr.end_char_idx >= line_char_start
                    && (fr.end_char_idx < line_char_end
                        || (fr.end_char_idx == line_char_end && owns_boundary));

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
                            layout_positions: None,
                            display_text: None,
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
                            layout_positions: None,
                            display_text: None,
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
                        // 안내문은 [누름틀 시작] 마커 뒤에 위치
                        let guide_x = find_x_for_char(fr.start_char_idx);
                        // [#6111] 긴 안내문을 한 줄로 그리면 본문·용지 밖까지 나간다
                        // (56345 7쪽: 49자 안내문이 x 93.7 → 943.7px, 용지 폭 794).
                        // 한글 편집기는 안내문을 누름틀 줄 상자 안에서 접는다. 안내문은
                        // 흐름에 영향이 없는 편집 전용 표시라(아래 `with_editor_only`),
                        // 접힌 뒤 줄들은 순수 오버레이로 아래에 쌓는다 — 첫 조각만
                        // 마커 shift 폭에 계상한다. 셀 안은 가용 폭 기준이 다르므로
                        // 종전대로 한 줄에 둔다.
                        let (body_x, _, body_w, _) = self.current_body_area.get();
                        let wrap_limit = if cell_ctx.is_none() && body_w > 0.0 {
                            (body_x + body_w - guide_x).max(0.0)
                        } else {
                            0.0
                        };
                        let guide_chunks =
                            split_guide_text_to_width(guide, &guide_style, wrap_limit);
                        let guide_width = guide_chunks
                            .first()
                            .map(|chunk| estimate_text_width(chunk, &guide_style))
                            .unwrap_or(0.0);
                        for (idx, chunk) in guide_chunks.iter().enumerate().skip(1) {
                            let extra_id = tree.next_id();
                            let extra = RenderNode::new(
                                extra_id,
                                RenderNodeType::TextRun(TextRunNode {
                                    text: (*chunk).to_string(),
                                    style: guide_style.clone(),
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
                                    layout_positions: None,
                                    display_text: None,
                                }),
                                BoundingBox::new(
                                    guide_x,
                                    y + line_height * idx as f64,
                                    estimate_text_width(chunk, &guide_style),
                                    line_height,
                                ),
                            )
                            .with_editor_only();
                            wrapped_guide_overlays.push(extra);
                        }
                        let guide = guide_chunks.first().copied().unwrap_or(guide);
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
                                layout_positions: None,
                                display_text: None,
                            }),
                            BoundingBox::new(guide_x, y, guide_width, line_height),
                        );
                        // [#3375] 안내문은 한컴 편집 화면에서만 보이고 인쇄·PDF 에는 나가지
                        // 않는다. 그림 미지정 placeholder(#2225)와 같은 계약이라 같은
                        // `editor_only` 표시를 쓴다 — 흐름 폭에는 영향이 없으므로(별도 마커
                        // 노드) 쪽수·줄바꿈은 프로필과 무관하게 동일하다.
                        let guide_node = guide_node.with_editor_only();
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
                            layout_positions: None,
                            display_text: None,
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
            let ctrl_positions = p.logical_control_positions();
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
                                layout_positions: None,
                                display_text: None,
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
        for overlay in wrapped_guide_overlays {
            line_node.children.push(overlay);
        }
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
            let mut remaining_est: String = run_chars_est[seg_start_est..].iter().collect();
            // TAC 로 쪼개지지 않은 런은 통째로 재므로, 표시 길이가 모델과 다르면
            // **그려지는 글자**로 잰다. 이 자연 폭이 정렬 간격 분배의 기준이라, 모델로
            // 재면 남는 폭이 과대평가돼 글자가 흩어진다 (Task #3216).
            if seg_start_est == 0 && run.display_text.is_some() {
                remaining_est = effective_text_for_metrics(run).to_string();
            }
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
        tree: &mut PageLayoutContext,
        line_node: &mut RenderNode,
        comp_line: &crate::renderer::composer::ComposedLine,
        composed: &ComposedParagraph,
        para: Option<&Paragraph>,
        bin_data_content: Option<&[BinDataContent]>,
        styles: &ResolvedStyleSet,
        cell_ctx: &Option<CellContext>,
        line_tac_offsets: &[(usize, f64, usize)],
        col_area: &LayoutRect,
        vars: EmptyRunsLineVars,
        current_line_reserved_tac_picture_height: &mut Option<f64>,
    ) {
        let mut empty_line_mark_x = vars.x_start;
        let mut empty_line_logical_end = vars.line_char_end;
        // runs가 없는 빈 줄에서 treat_as_char 이미지 렌더링
        // 테이블 셀 내부에서는 table_layout.rs가 layout_picture로 이미 처리하므로 스킵.
        // 셀 외부에서 해당 줄 범위에 걸린 TAC만 여기서 렌더링.
        //
        // [#5727] 예외 — 저장 lineseg 가 TAC 개체에 배정한 자기 줄(빈 줄, 다음
        // 줄과 char_start 동일)은 셀 안에서도 여기서 그린다. 이 줄 소유 TAC 는
        // 다음 줄 run 귀속에서 제외되므로 여기서 그리지 않으면 개체가 사라진다.
        // 이미 등록된 개체는 건너뛰어 다른 경로와의 이중 렌더를 막는다.
        let owns_boundary_tac = composed
            .lines
            .get(vars.line_idx + 1)
            .is_some_and(|next| next.char_start == comp_line.char_start && !next.runs.is_empty());
        let empty_line_tac_allowed =
            cell_ctx.is_none() || is_caption_cell_context(cell_ctx.as_ref()) || owns_boundary_tac;
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
                            let shape_h = hwpunit_to_px(shape.flow_height_hu(), self.dpi);
                            // [#5789] 빈 run 줄은 max_fs=0 이라 vars.baseline 이 0 으로
                            // 접힌다 — TAC 개체는 글자처럼 baseline 에 앉아야 하므로
                            // 저장 줄의 baseline_distance 로 폴백한다 (3143955 이중선:
                            // 줄 상자 top 161.99 ↔ 한글 baseline 182.4, 20.4px 어긋남).
                            let baseline = if vars.baseline > 0.01 {
                                vars.baseline
                            } else {
                                hwpunit_to_px(comp_line.baseline_distance, self.dpi)
                            };
                            let shape_y = (vars.y + baseline - shape_h).max(vars.y);
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
                            // [#5727] 셀 안 경로는 다른 패스가 먼저 그렸을 수 있다 —
                            // 등록된 개체는 건너뛰어 이중 렌더를 막는다.
                            if cell_ctx.is_some()
                                && tree
                                    .get_inline_shape_position(
                                        vars.section_index,
                                        vars.para_index,
                                        tac_ci,
                                        cell_ctx.as_ref(),
                                    )
                                    .is_some()
                            {
                                img_x += tac_w;
                                empty_line_mark_x = img_x;
                                empty_line_logical_end += 1;
                                continue;
                            }
                            let (_, pic_h) = self.resolve_inline_picture_size(pic, col_area);
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
                            let image_data = find_bin_data_bytes(bdc, bin_data_id);
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
                            let original_size_hu = pic.crop_reference_size();
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
                layout_positions: None,
                display_text: None,
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
        tree: &mut PageLayoutContext,
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
                        layout_positions: None,
                        display_text: None,
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
                        layout_positions: None,
                        display_text: None,
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
                // [#3307] 개요 문단이 유효한 정의에 도달하지 못하면 한컴 내장
                // 기본 모양(전 수준 ^N)으로 fallback 한다. NUMBER 는 불변 —
                // 정의 없는 NUMBER 는 종전대로 번호를 그리지 않는다.
                let synthesized_default;
                let numbering = match numbering_id
                    .checked_sub(1)
                    .and_then(|i| styles.numberings.get(i as usize))
                {
                    Some(n) => n,
                    None if para_style.head_type == HeadType::Outline => {
                        synthesized_default =
                            crate::renderer::layout::utils::default_outline_numbering();
                        &synthesized_default
                    }
                    None => return None,
                };

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

                let text = expand_numbering_format(
                    format_str,
                    &counters,
                    numbering,
                    &start_numbers,
                    level_idx,
                );
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
/// 한컴 layout 정합 (`mydocs/tech/investigations/issue-1151/topandbottom_table_inline_picture_layout.md` H1):
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
    tree: &mut PageLayoutContext,
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
mod issue_4370_tac_table_wrap_tests {
    use super::should_wrap_middle_anchored_table;

    /// [#4370] 끝 앵커(텍스트 마지막 문자 뒤) tac 표도 남은 폭 초과 시 wrap 된다.
    #[test]
    fn end_anchored_table_wraps_when_exceeding_line_width() {
        assert!(should_wrap_middle_anchored_table(
            Some(25),
            25,
            300.0,
            480.0,
            567.0
        ));
    }

    #[test]
    fn end_anchored_table_stays_inline_when_it_fits() {
        assert!(!should_wrap_middle_anchored_table(
            Some(25),
            25,
            300.0,
            200.0,
            567.0
        ));
    }

    /// 문단 선두 앵커(position == 0)는 점유 폭이 없으므로 wrap 하지 않는다.
    #[test]
    fn leading_anchor_never_wraps() {
        assert!(!should_wrap_middle_anchored_table(
            Some(0),
            25,
            0.0,
            480.0,
            567.0
        ));
    }

    #[test]
    fn middle_anchor_wrap_preserved() {
        assert!(should_wrap_middle_anchored_table(
            Some(10),
            20,
            120.0,
            480.0,
            567.0
        ));
    }
}

#[cfg(test)]
mod issue_2809_split_alignment_tests {
    use super::{compute_line_extra_spacing, needs_word_distribution};
    use crate::model::style::Alignment;
    use crate::renderer::composer::{ComposedLine, ComposedTextRun};
    use crate::renderer::layout::text_measurement::{estimate_text_width, resolved_to_text_style};
    use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedStyleSet};

    fn split_label_line() -> ComposedLine {
        ComposedLine {
            runs: vec![ComposedTextRun {
                text: "다 같 이".to_string(),
                ..Default::default()
            }],
            line_height: 1120,
            baseline_distance: 952,
            segment_width: 6972,
            column_start: 0,
            line_spacing: 560,
            has_line_break: false,
            char_start: 0,
        }
    }

    #[test]
    fn split_distributes_single_last_line_but_justify_does_not() {
        assert!(needs_word_distribution(
            Alignment::Split,
            true,
            false,
            false
        ));
        assert!(!needs_word_distribution(
            Alignment::Justify,
            true,
            false,
            false
        ));
        assert!(needs_word_distribution(
            Alignment::Justify,
            false,
            false,
            false
        ));
        assert!(!needs_word_distribution(
            Alignment::Split,
            true,
            false,
            true
        ));
    }

    #[test]
    fn split_label_assigns_positive_slack_to_interior_spaces() {
        let line = split_label_line();
        let (extra_word, extra_char, extra_dash) = compute_line_extra_spacing(
            &line,
            &ResolvedStyleSet::default(),
            Alignment::Split,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            5,
            30.0,
            90.0,
            40.0,
        );

        assert!((extra_word - 30.0).abs() < 0.001);
        assert_eq!(extra_char, 0.0);
        assert_eq!(extra_dash, 0.0);

        // A synthetic soft-wrap keeps its consumed separator in the projected
        // run. The renderer advances that final space too, so it participates
        // in the slot count and the complete run stays inside the line box.
        let trailing_line = ComposedLine {
            runs: vec![ComposedTextRun {
                text: "다 같 이 ".to_string(),
                ..Default::default()
            }],
            ..line
        };
        let styles = ResolvedStyleSet::default();
        let text_style = resolved_to_text_style(&styles, 0, 0);
        let natural_width = estimate_text_width("다 같 이 ", &text_style);
        let available_width = natural_width + 60.0;
        let (extra_word, extra_char, extra_dash) = compute_line_extra_spacing(
            &trailing_line,
            &styles,
            Alignment::Justify,
            false,
            true,
            false,
            false,
            false,
            true,
            false,
            6,
            natural_width,
            available_width,
            40.0,
        );
        let mut distributed_style = text_style;
        distributed_style.extra_word_spacing = extra_word;
        assert!(
            (estimate_text_width("다 같 이 ", &distributed_style) - available_width).abs() < 0.001
        );
        assert_eq!(extra_char, 0.0);
        assert_eq!(extra_dash, 0.0);
    }

    #[test]
    fn split_reserves_last_glyph_ink_when_letter_spacing_is_negative() {
        let line = split_label_line();
        let styles = ResolvedStyleSet {
            char_styles: vec![ResolvedCharStyle {
                font_size: 12.0,
                letter_spacing: -6.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let text_style = resolved_to_text_style(&styles, 0, 0);
        let total_text_width = estimate_text_width("다 같 이", &text_style);
        let (extra_word, extra_char, extra_dash) = compute_line_extra_spacing(
            &line,
            &styles,
            Alignment::Split,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            5,
            total_text_width,
            90.0,
            40.0,
        );

        let mut distributed_style = text_style.clone();
        distributed_style.extra_word_spacing = extra_word;
        let advance = estimate_text_width("다 같 이", &distributed_style);
        let mut ink_style = text_style;
        ink_style.letter_spacing = 0.0;
        let trailing_ink_overhang =
            estimate_text_width("이", &ink_style) - estimate_text_width("이", &distributed_style);

        assert!((advance + trailing_ink_overhang - 90.0).abs() < 0.001);
        assert_eq!(extra_char, 0.0);
        assert_eq!(extra_dash, 0.0);
    }

    /// [#4516] 머리말/꼬리말 예외로만 justify 된 마지막 줄: 공백 없는 영문
    /// 문서번호에 양수 slack 을 자간으로 살포하지 않는다 (자연 폭 유지).
    #[test]
    fn footer_last_line_justify_without_spaces_keeps_natural_width() {
        let line = ComposedLine {
            runs: vec![ComposedTextRun {
                text: "RVT-QI-02-03".to_string(),
                ..Default::default()
            }],
            line_height: 1120,
            baseline_distance: 952,
            segment_width: 48188,
            column_start: 0,
            line_spacing: 560,
            has_line_break: false,
            char_start: 0,
        };
        // 꼬리말 마지막 줄 예외 (justify_spaces_only = true): 분배 없음
        let (extra_word, extra_char, extra_dash) = compute_line_extra_spacing(
            &line,
            &ResolvedStyleSet::default(),
            Alignment::Justify,
            false,
            true,
            true,
            false,
            false,
            false,
            false,
            12,
            62.5,
            481.8,
            40.0,
        );
        assert_eq!(extra_word, 0.0);
        assert_eq!(extra_char, 0.0);
        assert_eq!(extra_dash, 0.0);

        // 본문 중간 줄 justify (justify_spaces_only = false): 기존 자간 분배 유지
        let (_, extra_char_mid, _) = compute_line_extra_spacing(
            &line,
            &ResolvedStyleSet::default(),
            Alignment::Justify,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            12,
            62.5,
            481.8,
            40.0,
        );
        assert!(extra_char_mid > 0.0);
    }
}

#[cfg(test)]
mod issue_4657_distribute_alignment_tests {
    use super::compute_line_extra_spacing;
    use crate::model::style::Alignment;
    use crate::renderer::composer::{ComposedLine, ComposedTextRun};
    use crate::renderer::layout::text_measurement::{estimate_text_width, resolved_to_text_style};
    use crate::renderer::style_resolver::ResolvedStyleSet;

    fn line(text: &str) -> ComposedLine {
        ComposedLine {
            runs: vec![ComposedTextRun {
                text: text.to_string(),
                ..Default::default()
            }],
            line_height: 1120,
            baseline_distance: 952,
            segment_width: 6972,
            column_start: 0,
            line_spacing: 560,
            has_line_break: false,
            char_start: 0,
        }
    }

    fn distribute_extra(text: &str, char_count: usize, text_width: f64, avail: f64) -> f64 {
        let (extra_word, extra_char, extra_dash) = compute_line_extra_spacing(
            &line(text),
            &ResolvedStyleSet::default(),
            Alignment::Distribute,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            char_count,
            text_width,
            avail,
            40.0,
        );
        assert_eq!(extra_word, 0.0);
        assert_eq!(extra_dash, 0.0);
        extra_char
    }

    /// 배분 정렬은 남는 폭을 글자 사이(N-1곳)에 나눠, 마지막 glyph 잉크의
    /// 오른쪽 끝(`W + (N-1)·extra`)이 줄 길이와 무관하게 문단 폭에 닿는다.
    #[test]
    fn distribute_fills_full_width_regardless_of_line_length() {
        let avail = 368.0;
        for (text, n, w) in [("문서관리번호 :", 8usize, 92.9), ("기관명 :", 5, 52.5)] {
            let extra = distribute_extra(text, n, w, avail);
            let last_ink_right = w + (n - 1) as f64 * extra;
            assert!(
                (last_ink_right - avail).abs() < 0.001,
                "{text}: 오른쪽 끝 {last_ink_right} != 문단 폭 {avail}"
            );
        }
    }

    /// 말미 공백은 배분 대상이 아니다 — 마지막 보이는 글자가 오른쪽 끝에 닿는다.
    #[test]
    fn distribute_excludes_trailing_spaces() {
        let styles = ResolvedStyleSet::default();
        let mut ts = resolved_to_text_style(&styles, 0, 0);
        ts.default_tab_width = 40.0;
        let space_w = estimate_text_width(" ", &ts);
        let avail = 368.0;
        let visible_w = 52.5;
        let extra = distribute_extra("기관명 : ", 6, visible_w + space_w, avail);
        let last_visible_ink_right = visible_w + 4.0 * extra;
        assert!(
            (last_visible_ink_right - avail).abs() < 0.001,
            "말미 공백 제외 후 오른쪽 끝 {last_visible_ink_right} != {avail}"
        );
    }

    /// 한 글자 + 말미 공백뿐인 줄은 분배하지 않는다 (0-division 가드).
    #[test]
    fn distribute_single_visible_char_keeps_natural_width() {
        let extra = distribute_extra("가  ", 3, 30.0, 368.0);
        assert_eq!(extra, 0.0);
    }
}

#[cfg(test)]
mod issue_3486_hancom_company_pua_alignment_tests {
    use super::is_hancom_company_pua_logo_line;
    use crate::model::style::Alignment;
    use crate::renderer::composer::{ComposedLine, ComposedTextRun};

    fn company_line(text: &str) -> ComposedLine {
        ComposedLine {
            runs: vec![ComposedTextRun {
                text: text.to_string(),
                ..Default::default()
            }],
            line_height: 1_000,
            baseline_distance: 850,
            segment_width: 42_520,
            column_start: 0,
            line_spacing: 500,
            has_line_break: false,
            char_start: 0,
        }
    }

    #[test]
    fn company_pua_logo_line_uses_its_space_not_internal_character_distribution() {
        let line = company_line("\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4} ");
        assert!(is_hancom_company_pua_logo_line(&line, Alignment::Split));
        assert!(
            !is_hancom_company_pua_logo_line(&line, Alignment::Left),
            "나눔 정렬이 아닌 문단에는 보정을 적용하면 안 됨",
        );
        assert!(
            !is_hancom_company_pua_logo_line(
                &company_line("\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4} 본문"),
                Alignment::Split,
            ),
            "회사명 뒤의 logo-gap 공백까지 일치할 때만 보정한다",
        );
    }
}

#[cfg(test)]
mod trailing_tac_width_tests {
    use super::tac_offsets_for_line_width;
    use crate::renderer::composer::{ComposedLine, ComposedParagraph, ComposedTextRun};

    fn line(text: &str, char_start: usize, has_line_break: bool) -> ComposedLine {
        ComposedLine {
            runs: vec![ComposedTextRun {
                text: text.to_string(),
                ..Default::default()
            }],
            line_height: 1_000,
            baseline_distance: 800,
            segment_width: 10_000,
            column_start: 0,
            line_spacing: 0,
            has_line_break,
            char_start,
        }
    }

    fn composed(lines: Vec<ComposedLine>) -> ComposedParagraph {
        ComposedParagraph {
            lines,
            para_style_id: 0,
            inline_controls: Vec::new(),
            numbering_text: None,
            tac_controls: Vec::new(),
            footnote_positions: Vec::new(),
            tab_extended: Vec::new(),
            horizontal_shaping: None,
        }
    }

    #[test]
    fn final_run_trailing_tac_is_included_in_alignment_width() {
        let comp = composed(vec![line("      ", 0, false)]);
        let offsets = tac_offsets_for_line_width(&comp, &[(6, 574.08, 0)], 0);

        assert_eq!(offsets, vec![(6, 574.08, 0)]);
    }

    #[test]
    fn next_line_leading_tac_is_not_back_attributed_to_previous_width() {
        let comp = composed(vec![line("A", 0, false), line("B", 1, false)]);
        let offsets = [(1, 55.0, 0)];

        assert!(tac_offsets_for_line_width(&comp, &offsets, 0).is_empty());
        assert_eq!(tac_offsets_for_line_width(&comp, &offsets, 1), offsets);
    }

    #[test]
    fn forced_break_trailing_tac_stays_with_emitting_line() {
        let comp = composed(vec![line("A", 0, true), line("B", 2, false)]);
        let offsets = [(1, 55.0, 0)];

        assert_eq!(tac_offsets_for_line_width(&comp, &offsets, 0), offsets);
        assert!(tac_offsets_for_line_width(&comp, &offsets, 1).is_empty());
    }
}

#[cfg(test)]
mod issue_2439_lineseg_indent_tests;

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
            // [#5793] 시각 판정 완료 — 한글 2022 는 이중 가로선(제목 밑 이중 밑줄,
            // 반각 6.66px/자)으로 그린다. ■(전각)로 두면 띠가 2배 길어져 제목을
            // 겹친다(1776332, layout-anomaly text-overlap w=213 1위). 이웃
            // 0xF0832 → ═ 와 같은 이중선 계열.
            0xF0827 => '\u{2550}', // ═ BOX DRAWINGS DOUBLE HORIZONTAL
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
/// [#6111] 누름틀 안내문을 가용 폭에 맞춰 조각낸다.
///
/// 안내문은 흐름에 영향이 없는 편집 전용 표시라 조판 줄바꿈 경로를 타지 않는다.
/// 그래서 한 줄로 그리면 본문·용지 밖까지 나간다 — 한글 편집기가 누름틀 줄 상자
/// 안에서 접는 것과 같도록 여기서 폭 기준으로만 자른다. `limit` 이 0 이하면(폭을
/// 알 수 없는 셀 등) 자르지 않는다.
fn split_guide_text_to_width<'a>(guide: &'a str, style: &TextStyle, limit: f64) -> Vec<&'a str> {
    if limit <= 0.0 || estimate_text_width(guide, style) <= limit {
        return vec![guide];
    }
    let mut chunks = Vec::new();
    let mut rest = guide;
    while !rest.is_empty() {
        let mut end = rest.len();
        let mut cut = None;
        for (idx, _) in rest.char_indices().skip(1) {
            if estimate_text_width(&rest[..idx], style) > limit {
                end = idx;
                break;
            }
            cut = Some(idx);
        }
        // 폭 안에 들어가는 마지막 경계까지 자른다. 한 글자도 안 들어가면 한 글자.
        let take = match cut {
            Some(idx) if idx < rest.len() => idx,
            _ => end,
        };
        let take = if take == 0 {
            rest.char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(rest.len())
        } else {
            take
        };
        let (head, tail) = rest.split_at(take);
        chunks.push(head);
        rest = tail;
        if chunks.len() > 64 {
            chunks.push(rest);
            break;
        }
    }
    chunks
}

pub(crate) fn form_color_to_css(color: u32) -> String {
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

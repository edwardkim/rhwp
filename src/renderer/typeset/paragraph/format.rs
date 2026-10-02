//! 문단 구성의 조회·계산 경로. 페이지/단 상태의 변경은 상위 흐름 조정자가 수행한다.
//!
//! 재구성 선택 → 줄 메트릭 → 예산/배치용 결과 조립의 기존 순서를 보존한다.
//! context는 필요한 엔진 관측만 제공하며 전체 TypesetEngine/TypesetState를 받지 않는다.
//! 줄 메트릭의 기존 경험적 분기는 그대로이며 이번 이동이 그 타당성의 새 승인은 아니다.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::ComposedParagraph;
use crate::renderer::float_placement::is_para_topbottom_float;
use crate::renderer::hwpunit_to_px;
use crate::renderer::style_resolver::{ResolvedParaStyle, ResolvedStyleSet};
use crate::renderer::typeset::{
    empty_paragraph_fallback_line_metrics, line_has_tac_control,
    para_is_treat_as_char_picture_only, text_line_is_picture_lead_in,
};

use super::context::ParagraphFormatContext;
use super::metrics::FormattedParagraph;

pub(in crate::renderer::typeset) fn format_paragraph_for_flow(
    ctx: &ParagraphFormatContext<'_>,
    para: &Paragraph,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
    column_width_px: Option<f64>,
    hwp3_body_reflow: bool,
    known_square_band: bool,
) -> FormattedParagraph {
    let para_style_id = composed.map(|c| c.para_style_id as usize).unwrap_or(0);
    let para_style = styles.para_styles.get(para_style_id);

    let recomposed = recompose_for_flow(
        ctx,
        para,
        composed,
        styles,
        para_style,
        column_width_px,
        known_square_band,
    );
    let composed = recomposed.as_ref().or(composed);
    let raw_spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
    let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);

    // [Task #998 실험] spacing_before=0 으로 강제 — 효과 측정용
    // [#2279 실험 전용] RHWP_EXP_BODY_FRESH 시 NO_LS 문단도 sb 를 보존한다
    // (한글 fresh 는 sb 를 가산 — 생성기 사다리 sb-누락 모사 우회 계측).
    let spacing_before = if para.line_segs.is_empty()
        && !para.text.is_empty()
        && std::env::var("RHWP_EXP_BODY_FRESH").is_err()
    {
        0.0
    } else {
        raw_spacing_before
    };
    // [Task #874 Case 3] `<...>` 단독 paragraph 의 paragraph-level extra spacing 제거.
    // 이전 #866 Stage 2 는 paragraph 위·아래 각 +20px (총 +40px) 을 paragraph 자체 height
    // 에 포함시켰으나, typeset 의 zone 전환 패딩(solo_zone_pad +16px enter +16px leave)
    // 이 이미 동일 역할을 담당하므로 이중 패딩이 발생 (한컴 PDF 대비 +48px excess, 4·5쪽
    // 누적 +17~30pt 사용자 피드백). zone 전환 패딩만 유지.

    let (line_heights, line_spacings) =
        resolve_line_metrics(ctx, para, composed, styles, para_style, column_width_px);

    let lines_total: f64 = line_heights
        .iter()
        .zip(line_spacings.iter())
        .map(|(h, s)| h + s)
        .sum();
    // 글자처럼 취급되는 표의 **바깥 여백**은 줄 상자 밖이지만 쪽 예산은 차지한다.
    // 한컴 저장 lineseg 의 vertsize 가 `표 높이 + outMargin(위+아래)` 이다(실측 18/18).
    // 줄 높이 자체에 더하면 그리기 좌표까지 밀려 조판이 무너지므로(실측 82.3%→54.0%),
    // **예산 값(total_height·height_for_fit)에만** 더한다.
    //
    // 🔴 이 가산분은 `tac_outer_margin_v_px` 로 따로 들고 다닌다. HWPX 저장조판의
    // "사다리가 문단 여백을 뺐는가" 지문(아래 `ladder_omits_spacing`)이 **±2px 창**이라,
    // 이 3.7px 가 섞이면 판정이 뒤집혀 cap 승격과 전방 스냅이 통째로 사라진다.
    // 그러면 표가 없는 뒤 문단까지 32px 씩 당겨 올라가 용지 밖으로 밀린다(실측 28.3px).
    //
    // 🔴 **저장 조판이 없는 문단에만** 더한다. 파일에서 열린 문단은 한컴이 계산해 둔
    // `lineseg` 를 그대로 쓰므로 여기서 또 더하면 이중 가산이 되어, 분할 표 꼬리가 페이지를
    // 단독 점유하고 뒤 본문이 밀린다(#2373 핀: 한글 2022 정답지 4쪽 ↔ 가산 시 5쪽).
    // 다시 조판해야 하는 문단(붙여넣기·생성계)에서만 이 몫이 실제로 빠져 있다.
    let tac_outer_margin_v_px: f64 = if crate::renderer::para_has_no_stored_line_segs(para) {
        para.controls
            .iter()
            .filter_map(|ctrl| match ctrl {
                Control::Table(t) if t.common.treat_as_char => {
                    Some(crate::renderer::hwpunit_to_px(
                        i32::from(t.common.margin.top) + i32::from(t.common.margin.bottom),
                        ctx.dpi(),
                    ))
                }
                _ => None,
            })
            .fold(0.0f64, f64::max)
    } else {
        0.0
    };
    let total_height = spacing_before + lines_total + spacing_after + tac_outer_margin_v_px;

    // 적합성 판단용: trailing line_spacing 제외
    let trailing_ls = line_spacings.last().copied().unwrap_or(0.0);
    let height_for_fit = {
        let metric = (total_height - trailing_ls).max(0.0);
        let vpos_metric = if para.controls.is_empty() {
            para.line_segs
                .first()
                .zip(para.line_segs.last())
                .and_then(|(first, last)| {
                    // 끝점만 비교하면 **문단 중간에서 리셋되는 사다리**를 놓친다.
                    // 쪽을 넘나드는 문단은 vpos 가 0 으로 돌아갔다가 다시 오르므로
                    // 첫↔끝은 증가로 보이지만 span 은 무의미하다 (#3751 · 1170000 입법역량
                    // pi=1265: 48000 →[line7 리셋]→ 62400, span 208px 인데 실제
                    // 34줄 1088px — fit 이 208 로 판정해 쪽을 863.9px 넘겼다).
                    // 사다리 전체가 단조 증가일 때만 span 을 쓴다.
                    let monotonic = para
                        .line_segs
                        .windows(2)
                        .all(|w| w[1].vertical_pos >= w[0].vertical_pos);
                    let has_progressing_vpos = para.line_segs.len() <= 1
                        || (monotonic && last.vertical_pos > first.vertical_pos);
                    if !has_progressing_vpos {
                        return None;
                    }
                    let span = last
                        .vertical_pos
                        .saturating_add(last.line_height)
                        .saturating_sub(first.vertical_pos);
                    (span > 0).then_some(hwpunit_to_px(span, ctx.dpi()))
                })
        } else {
            None
        };
        vpos_metric.map(|v| metric.min(v)).unwrap_or(metric)
    };

    // 표 없는 일반 본문에는 배치용 줄 사본을 만들지 않는다.
    let computed_host_lines = recomposed
        .as_ref()
        .filter(|_| {
            para.controls.iter().any(|control| {
                matches!(control,
            Control::Table(table) if is_para_topbottom_float(&table.common))
            })
        })
        .map(|comp| {
            // 높이 보정이 줄 수를 바꾼 다른 owner의 결과를 문자 경계에 억지로 zip하지 않는다.
            if comp.lines.len() != line_heights.len() || comp.lines.len() != line_spacings.len() {
                return Vec::new();
            }
            let mut top = 0.0;
            comp.lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    let resolved = crate::renderer::float_placement::ParagraphHostLine {
                        char_start: line.char_start,
                        top,
                        height: line_heights[i],
                    };
                    top += line_heights[i] + line_spacings[i];
                    resolved
                })
                .collect()
        });
    let tail_line_remaining_width = (|| {
        if !para.controls.iter().any(|control| {
            matches!(control, Control::Table(table) if is_para_topbottom_float(&table.common))
        }) {
            return None;
        }
        let comp = composed?;
        let line = comp.lines.last()?;
        // Tabs need their resolved tab-stop positions, not font advances.
        // Do not manufacture a line miss from an unsupported measurement.
        if line.has_line_break || line.runs.iter().any(|run| run.text.contains('\t')) {
            return None;
        }
        let cw = column_width_px?;
        let style = para_style?;
        let available =
            cw - crate::renderer::equation_tac_flow::paragraph_effective_margin_left(
                style.margin_left,
                style.indent,
                comp.lines.len() - 1,
            ) - style.margin_right;
        let text_width = crate::renderer::composer::estimate_composed_line_width(line, styles);
        let inline_width: f64 = comp
            .tac_controls
            .iter()
            .filter(|(position, _, _)| *position >= line.char_start)
            .map(|(_, width, _)| hwpunit_to_px(*width, ctx.dpi()))
            .sum();
        let remaining = available - text_width - inline_width;
        (available.is_finite() && available > 0.0 && remaining.is_finite()).then_some(remaining)
    })();
    FormattedParagraph {
        tail_line_remaining_width,
        computed_host_lines,
        total_height,
        line_heights,
        line_spacings,
        spacing_before,
        spacing_after,
        height_for_fit,
        tac_outer_margin_v_px,
    }
}

/// 기존 frame 재구성의 선택과 읽기 borrow 범위를 보존한다.
fn recompose_for_flow(
    ctx: &ParagraphFormatContext<'_>,
    para: &Paragraph,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
    para_style: Option<&ResolvedParaStyle>,
    column_width_px: Option<f64>,
    known_square_band: bool,
) -> Option<ComposedParagraph> {
    // [Task #1042 Stage 6c] line_segs.empty paragraph 의 typeset/layout 측정 정합 —
    // paragraph_layout (렌더링 path) 는 Stage 6b 에서 recompose_for_cell_width 로 column
    // 기반 wrap 을 적용하지만, format_paragraph (typeset/measurement path) 는 원본
    // compose_lines fallback (CHARS_PER_LINE=45) 결과로 측정 → 두 path 의 line_count 불일치
    // 발생 (e.g. sample16 변환기 pi=417: typeset 2 lines / layout 1 line, +10.4 px gap).
    // 동일 recompose 를 typeset 측에도 적용해 paragraph height 측정 정합.
    match (composed, column_width_px) {
        (Some(c), Some(cw)) if cw > 0.0 => {
            let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let inner = (cw - margin_l - margin_r).max(0.0);
            // 문단 상자는 편집 경로(`DocumentCore::reflow_paragraph`)의 가용 폭과
            // 같아야 한다 — 한 문단이 어느 경로로 왔는지에 따라 다른 폭을 갖지
            // 않게 한다. 들여쓰기/내어쓰기는 이 상자 **안에서**
            // `layout_paragraph_in_frame` 의 indent_px 가 적용한다.
            // `body_for_style`, not `body`: the list-origin blocker is part of
            // the box, so a route that skips it publishes a different origin
            // for the same paragraph.
            let paragraph_box =
                crate::renderer::composer::ParagraphBox::body_for_style(cw, para_style, ctx.dpi());
            // NO_LS 와 저장분할 both go to the frame.
            if inner > 0.0 {
                crate::renderer::composer::recompose_stored_lines_in_frame_with_known_square_band(
                    c,
                    para,
                    paragraph_box,
                    inner,
                    styles,
                    ctx.dpi(),
                    ctx.profile().legacy_hwp3_stored_geometry(),
                    crate::renderer::composer::StoredRowMissPolicy::Reflow,
                    &ctx.float_carve_evidence(),
                    known_square_band,
                    ctx.profile().hwpx_stored_layout(),
                )
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 구성된 줄/저장 줄/빈 문단 경로의 기존 메트릭과 보정을 계산한다.
fn resolve_line_metrics(
    ctx: &ParagraphFormatContext<'_>,
    para: &Paragraph,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
    para_style: Option<&ResolvedParaStyle>,
    column_width_px: Option<f64>,
) -> (Vec<f64>, Vec<f64>) {
    let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
    let ls_type = para_style
        .map(|s| s.line_spacing_type)
        .unwrap_or(crate::model::style::LineSpacingType::Percent);

    // [Task #901 Stage 7] wrap zone host paragraph 의 whitespace-only line 은 height 제외.
    // paragraph_layout 의 skip_advance_empty_wrap 와 정합 — pagination 의 height 계산
    // 이 시각 렌더링과 어긋나 paragraph 11 등이 잘못 다음 페이지로 분할되는 문제 해소.
    let has_picture_shape_square_wrap = para.controls.iter().any(|c| {
        use crate::model::shape::TextWrap;
        let common_opt = match c {
            Control::Picture(pic) if !pic.common.treat_as_char => Some(&pic.common),
            Control::Shape(s) if !s.common().treat_as_char => Some(s.common()),
            _ => None,
        };
        common_opt
            .map(|cm| matches!(cm.text_wrap, TextWrap::Square))
            .unwrap_or(false)
    });
    let has_treat_as_char_picture_shape = para.controls.iter().any(|c| {
        matches!(
            c,
            Control::Picture(pic) if pic.common.treat_as_char
        ) || matches!(
            c,
            Control::Shape(shape) if shape.common().treat_as_char
        )
    });
    let (mut line_heights, mut line_spacings): (Vec<f64>, Vec<f64>) = if let Some(comp) = composed {
        let tac_offsets_px: Vec<(usize, f64, usize)> = comp
            .tac_controls
            .iter()
            .map(|(pos, width_hu, control_index)| {
                (*pos, hwpunit_to_px(*width_hu, ctx.dpi()), *control_index)
            })
            .collect();
        let line_available_width_px = |line_idx: usize| {
            column_width_px.map(|cw| {
                let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let indent = para_style.map(|s| s.indent).unwrap_or(0.0);
                let effective_margin_l =
                    crate::renderer::equation_tac_flow::paragraph_effective_margin_left(
                        margin_l, indent, line_idx,
                    );
                (cw - effective_margin_l - margin_r).max(0.0)
            })
        };
        // [Task #1472] 변환본은 미주 수식 effective indent 불변 위해 scale 절반(2.0→1.0).
        let eq_indent_scale = 2.0
            * if ctx.profile().hwp3_layout() {
                0.5
            } else {
                1.0
            };
        let equation_line_available_width_px = |visual_line_idx: usize| {
            column_width_px.map(|cw| {
                let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let indent = para_style.map(|s| s.indent).unwrap_or(0.0);
                let effective_margin_l = crate::renderer::equation_tac_flow::
                    paragraph_effective_margin_left_with_indent_scale(
                        margin_l,
                        indent,
                        visual_line_idx,
                        eq_indent_scale,
                    );
                (cw - effective_margin_l - margin_r).max(0.0)
            })
        };
        // [#2004] 동일 char_start 에 tac(글자처럼) 그림이 다수 앵커되어 각자 한 줄씩
        // 차지하는 "이미지 스택" 문단: tac_control_indices_for_line 의 char-range 매핑이
        // [start, next_start)=[0,0) 로 비어 중간 줄이 empty_tac_guide_line(0 높이)로 붕괴한다
        // ([860, 0.., 860] 패턴). 한글은 각 그림을 쪽당 1장씩 배치하므로 각 줄에 순서대로
        // 그림 높이를 직접 부여해 pagination 이 줄별로 쪽을 나누게 한다. 게이트를
        // (tac-그림-only + 그림수==줄수 + 전부 동일 char_start + 모든 높이>8px) 로 좁혀
        // 일반 인라인 그림/텍스트 문단 회귀를 차단한다.
        let stacked_tac_picture_heights: Option<Vec<f64>> = {
            let tacs = &comp.tac_controls;
            if para_is_treat_as_char_picture_only(para)
                && tacs.len() >= 2
                && comp.lines.len() == tacs.len()
                && comp
                    .lines
                    .iter()
                    .all(|l| l.char_start == comp.lines[0].char_start)
            {
                let hs: Vec<f64> = tacs
                    .iter()
                    .map(|(_, _, ci)| {
                        para.controls
                            .get(*ci)
                            .and_then(|c| crate::renderer::tac_object_flow_height_px(c, ctx.dpi()))
                            .unwrap_or(0.0)
                    })
                    .collect();
                (hs.iter().all(|h| *h > 8.0)).then_some(hs)
            } else {
                None
            }
        };
        let mut pairs = Vec::with_capacity(comp.lines.len());
        let mut prev_line_reserved_tac_picture_height: Option<f64> = None;
        for (line_idx, line) in comp.lines.iter().enumerate() {
            if let Some(ref hs) = stacked_tac_picture_heights {
                let ls_px = hwpunit_to_px(line.line_spacing, ctx.dpi());
                pairs.push((hs[line_idx], ls_px));
                prev_line_reserved_tac_picture_height = Some(hs[line_idx]);
                continue;
            }
            let runs_all_whitespace = line.runs.iter().all(|r| r.text.trim().is_empty());
            let line_has_tac_control = line_has_tac_control(para, comp, line_idx);
            // [#6972] 저장 줄 높이가 TAC 개체 하나의 흐름 높이와 같으면 그 줄은
            // 개체가 **소유한 줄**이지 빈 guide 줄이 아니다. composer 가 두 줄에
            // 같은 char_start 를 실으면 `tac_control_indices_for_line` 의 char-range
            // 매핑이 [start, start) 로 비어 `line_has_tac_control` 이 거짓이 되고,
            // 전면 크기 TAC 그림 줄(56288 1쪽: 72347HU = 964.6px)이 통째로 0 이 된다.
            // 렌더는 같은 줄을 964.6px 로 그리므로 조판만 어긋나 뒤 개체가 그림 위로
            // 올라온다. 소유 판정은 `line_owning_tac_object_height_px` 하나로 한다(#4333).
            let line_owns_tac_object = crate::renderer::line_owning_tac_object_height_px(
                para,
                hwpunit_to_px(line.line_height, ctx.dpi()),
                ctx.dpi(),
            )
            .is_some();
            let empty_tac_guide_line = runs_all_whitespace
                && !line_has_tac_control
                && !line_owns_tac_object
                && comp
                    .lines
                    .get(line_idx + 1)
                    .is_some_and(|next| next.char_start == line.char_start)
                && comp
                    .tac_controls
                    .iter()
                    .any(|(pos, _, _)| *pos == line.char_start);
            if empty_tac_guide_line {
                pairs.push((0.0, 0.0));
                prev_line_reserved_tac_picture_height = None;
                continue;
            }
            // [#6086] 같은 vpos·다른 column_start 의 연속 저장 세그는 어울림
            // 개체가 한 줄을 좌/우로 가른 **수평 분할**이다 — 같은 시각적
            // 줄이므로 뒤 세그는 높이를 계상하지 않는다. 30098: 순서도 상자
            // 옆 빈 문단 12개가 2세그 세로 적층으로 ×2 계상되어 +288px,
            // 16쪽 vs 한글 15쪽. (#6035 의 쪽-리셋 동일-vpos 쌍은 column_start
            // /폭이 같아 이 게이트에 걸리지 않는다.)
            let horizontal_split_continuation = line_idx > 0
                && comp.lines.len() == para.line_segs.len()
                && para
                    .line_segs
                    .get(line_idx)
                    .zip(para.line_segs.get(line_idx - 1))
                    .is_some_and(|(cur, prev)| {
                        let real = |seg: &crate::model::paragraph::LineSeg| {
                            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                == 0
                        };
                        real(cur)
                            && real(prev)
                            && cur.vertical_pos >= 0
                            && cur.vertical_pos == prev.vertical_pos
                            && cur.column_start != prev.column_start
                    });
            if horizontal_split_continuation {
                pairs.push((0.0, 0.0));
                prev_line_reserved_tac_picture_height = None;
                continue;
            }
            // Square wrap host 의 빈 wrap guide 줄은 높이를 제외하되, 같은 줄에
            // TAC 수식/개체가 있으면 실제 콘텐츠 줄이므로 정상 advance 를 보존한다.
            if has_picture_shape_square_wrap && runs_all_whitespace && !line_has_tac_control {
                pairs.push((0.0, 0.0));
                prev_line_reserved_tac_picture_height = None;
                continue;
            }
            let raw_lh = hwpunit_to_px(line.line_height, ctx.dpi());
            let raw_text_height = para
                .line_segs
                .get(line_idx)
                .map(|seg| hwpunit_to_px(seg.text_height, ctx.dpi()))
                .unwrap_or(0.0);
            let max_fs = crate::renderer::composed_line_max_font_size(line, para, styles);
            let text_before_picture_line =
                text_line_is_picture_lead_in(para, comp, line_idx, raw_lh, max_fs, ctx.dpi());
            let tac_picture_height =
                crate::renderer::line_owning_tac_object_height_px(para, raw_lh, ctx.dpi());
            let tac_picture_height = if text_before_picture_line {
                None
            } else {
                tac_picture_height.or_else(|| {
                    (has_treat_as_char_picture_shape
                        && !runs_all_whitespace
                        && max_fs > 0.0
                        && raw_lh > max_fs * 2.0)
                        .then_some(raw_lh)
                })
            };
            if runs_all_whitespace
                && tac_picture_height.is_none()
                && prev_line_reserved_tac_picture_height
                    .map(|prev| (raw_lh - prev).abs() <= 8.0)
                    .unwrap_or(false)
            {
                pairs.push((0.0, 0.0));
                prev_line_reserved_tac_picture_height = None;
                continue;
            }
            // [#5854] 통짜 합성 사다리 문서는 저장 `line_height` 가 글자 크기보다
            // 크든 작든 실측이 아니다 — 항상 글꼴·문단 스타일로 다시 뽑는다.
            let recompute_lh = text_before_picture_line
                || (max_fs > 0.0 && (raw_lh < max_fs || ctx.uniform_filler_ladder()));
            let (lh, line_spacing_px) = if recompute_lh {
                // [Task #1042 Stage 6c] HWP3/HWP5 line_segs 의 (line_height=base,
                // line_spacing=extra) 의미와 정합되게 분해 — 종전 처럼 ls_val/100 전체를
                // line_height 에 baking 하고 line_spacing_px=0 으로 두면 trailing_ls 제거
                // 효과 (height_for_fit) 가 line_segs 있는 path 와 어긋남.
                use crate::model::style::LineSpacingType;
                if text_before_picture_line {
                    (max_fs.max(1.0), hwpunit_to_px(line.line_spacing, ctx.dpi()))
                } else {
                    match ls_type {
                        LineSpacingType::Percent => {
                            // [#2279] sub-100% 퍼센트 음수 gap 존중 (line_breaking 정합)
                            // 0% 는 실값이다 — line_breaking 과 같은 계약(>=)으로 맞춘다.
                            let extra = if ls_val >= 0.0 {
                                max_fs * (ls_val - 100.0) / 100.0
                            } else {
                                0.0
                            };
                            (max_fs, extra)
                        }
                        LineSpacingType::Fixed => (ls_val.max(max_fs), 0.0),
                        LineSpacingType::SpaceOnly => (max_fs, ls_val.max(0.0)),
                        LineSpacingType::Minimum => (ls_val.max(max_fs), 0.0),
                    }
                }
            } else {
                crate::renderer::corrected_line_metrics_for_source(
                    raw_lh,
                    raw_text_height,
                    hwpunit_to_px(line.line_spacing, ctx.dpi()),
                    max_fs,
                    ls_type,
                    ls_val,
                    para.controls.is_empty(),
                    crate::renderer::controls_mark_section_start(&para.controls),
                )
            };
            let extra_rows =
                crate::renderer::equation_tac_flow::compute_equation_only_tac_line_flow(
                    Some(para),
                    comp,
                    &tac_offsets_px,
                    line_idx,
                    equation_line_available_width_px(0).unwrap_or(f64::INFINITY),
                    equation_line_available_width_px(1).unwrap_or(f64::INFINITY),
                )
                .map(|flow| flow.extra_rows)
                .unwrap_or(0);
            // Pagination은 배치 cursor와 다른 예약 계약을 사용한다. 저장 사다리의
            // 짧은 text advance를 여기에도 적용하면 미주·글자처럼 취급되는 개체의
            // page budget이 줄어들어 이전 줄에 과적재된다. #6656은
            // HeightMeasurer의 fallback 측정 정합 범위이므로 typeset 예약 높이는
            // 종전 줄 상자를 유지한다.
            let flow_lh = lh + extra_rows as f64 * (lh + line_spacing_px);
            pairs.push((flow_lh, line_spacing_px));
            prev_line_reserved_tac_picture_height = tac_picture_height;
        }
        if pairs.is_empty() {
            if let Some(metric) = empty_paragraph_fallback_line_metrics(
                para,
                styles,
                para_style,
                ctx.profile().hwp3_layout(),
            ) {
                pairs.push(metric);
            }
        }
        // [#2287] 저장 LINE_SEG 없는 빈 anchor 문단의 TAC 그림/도형 —
        // composed lines 가 비어 문단 플로우가 0 으로 붕괴하는 것을
        // 개체 폭 greedy wrap 줄 메트릭 합성으로 방지.
        if pairs.is_empty() {
            if let Some(metrics) = crate::renderer::tac_object_stack_line_metrics(
                para,
                ctx.dpi(),
                line_available_width_px(0),
                styles,
                para_style,
            ) {
                pairs.extend(metrics);
            }
        }
        // 저장 LINE_SEG가 전혀 없는 빈 문단도 composer는 placeholder line 하나를
        // 남길 수 있다. 그 경우 `pairs.is_empty()`만으로는 fallback에 들어가지 않아
        // 400HU(약 5.3px)로 축소된다. 한글은 이 문단을 저장 글자모양과 줄간격의
        // 완전한 줄박스로 취급하므로, placeholder 유무와 관계없이 같은 메트릭을
        // 적용해야 pagination과 SVG 렌더가 일치한다 (#3820 p81–82).
        if let Some(metric) = empty_paragraph_fallback_line_metrics(
            para,
            styles,
            para_style,
            ctx.profile().hwp3_layout(),
        ) {
            pairs.clear();
            pairs.push(metric);
        }
        pairs.into_iter().unzip()
    } else if !para.line_segs.is_empty() {
        para.line_segs
            .iter()
            .map(|seg| {
                (
                    hwpunit_to_px(seg.line_height, ctx.dpi()),
                    hwpunit_to_px(seg.line_spacing, ctx.dpi()),
                )
            })
            .unzip()
    } else if let Some((lh, ls)) =
        empty_paragraph_fallback_line_metrics(para, styles, para_style, ctx.profile().hwp3_layout())
    {
        (vec![lh], vec![ls])
    } else {
        (vec![hwpunit_to_px(400, ctx.dpi())], vec![0.0])
    };
    if has_treat_as_char_picture_shape
        && line_heights.len() == 2
        && line_heights[0] > 80.0
        && (line_heights[0] - line_heights[1]).abs() <= 8.0
    {
        line_heights[1] = 0.0;
        line_spacings[1] = 0.0;
    }

    // [#2019 부분 완화] 부동 개체 전용 빈 앵커 문단(별지 서식): 이 경로는
    // 81쪽 산란 재발을 막기 위해 stored line_height 를 빈 문단 fallback 으로 낮춘다.
    // 다만 이것은 한글 2020/2022 정합 모델이 아니다. 일부 Paper 앵커 개체는 실제 렌더
    // extent 를 page-local pagination 에 반영해야 하며, #2019 v3 에서 이 부분을 다시
    // 풀어야 한다. 자리차지·tac=true 는 helper 에서 제외되어 예약 유지.
    if crate::renderer::layout::para_is_floating_overlay_anchor(para) {
        let (lh, ls) = empty_paragraph_fallback_line_metrics(
            para,
            styles,
            para_style,
            ctx.profile().hwp3_layout(),
        )
        .unwrap_or((hwpunit_to_px(400, ctx.dpi()), 0.0));
        line_heights = vec![lh];
        line_spacings = vec![ls];
    }

    (line_heights, line_spacings)
}

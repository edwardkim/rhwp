//! 커서 좌표/히트테스트/셀 커서/경로 기반 조작 관련 native 메서드

use super::super::helpers::{
    color_ref_to_css, find_char_at_x, find_logical_control_positions, has_table_control,
    is_treat_as_char_object_control, navigable_text_len, utf16_pos_to_char_idx, LineInfoResult,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::path::PathSegment;
use crate::renderer::layout::{CellContext, CellPathEntry};
use crate::renderer::render_tree::TextRunNode;

/// 글자겹침 TextRun의 논리적 char_count (1) 반환, 아니면 실제 글자 수 반환.
///
/// `table-vpos-01.hwp`의 boxed 10/11/12처럼 CharOverlap payload가 여러
/// PUA 구성 글자를 갖더라도 편집 커서는 한 글자 단위로 이동해야 한다.
fn effective_char_count(text_run: &TextRunNode) -> usize {
    if text_run.char_overlap.is_some() {
        let chars: Vec<char> = text_run.text.chars().collect();
        return crate::renderer::composer::char_overlap_advance_units(&chars);
    }
    text_run.text.chars().count()
}

fn note_number_format_from_hwp_code(code: u8) -> crate::renderer::NumberFormat {
    match code {
        0 => crate::renderer::NumberFormat::Digit,
        1 => crate::renderer::NumberFormat::CircledDigit,
        2 => crate::renderer::NumberFormat::RomanUpper,
        3 => crate::renderer::NumberFormat::RomanLower,
        4 => crate::renderer::NumberFormat::LatinUpper,
        5 => crate::renderer::NumberFormat::LatinLower,
        8 => crate::renderer::NumberFormat::HangulGaNaDa,
        12 => crate::renderer::NumberFormat::HangulNumber,
        13 => crate::renderer::NumberFormat::HanjaNumber,
        _ => crate::renderer::NumberFormat::Digit,
    }
}

fn note_decoration_char(value: u16) -> Option<char> {
    if value == 0 {
        None
    } else {
        char::from_u32(value as u32).filter(|ch| *ch != '\0')
    }
}

fn note_marker_text(
    number: u16,
    number_shape: u32,
    before_decoration_letter: u16,
    after_decoration_letter: u16,
) -> String {
    let number = crate::renderer::format_number(
        number,
        note_number_format_from_hwp_code(number_shape as u8),
    );
    let prefix = note_decoration_char(before_decoration_letter)
        .map(|ch| ch.to_string())
        .unwrap_or_default();
    let suffix = note_decoration_char(after_decoration_letter)
        .unwrap_or(')')
        .to_string();
    format!("{}{}{}", prefix, number, suffix)
}

/// 한 시각 줄(line)을 구성하는 한 run 의, 클릭 x → 문자 위치 해석에 필요한 최소 정보.
///
/// `hit_test_native` 내부의 `RunInfo` 에서 줄별로 추려 만든 가벼운 view.
/// 이 view 만 분리해 두면 줄 단위 x 해석 로직(`resolve_x_on_line`)을 문서/페이지
/// 트리 없이 단위 테스트할 수 있다.
pub(crate) struct LineRunView<'a> {
    pub bbox_x: f64,
    pub bbox_w: f64,
    pub char_start: usize,
    pub char_count: usize,
    /// 각 글자의 run-local x 좌표(누적 advance). 비어 있으면 글자 위치 미상.
    pub char_positions: &'a [f64],
}

/// run-local x 에 해당하는 글자 인덱스. `hit_test_native` 내부 `find_char_at_x`
/// 와 동일 의미(positions[i] = i번째 글자의 left x, 0.0 포함 안 함)를 유지한다.
fn line_local_char_at_x(positions: &[f64], local_x: f64) -> usize {
    for (i, &px) in positions.iter().enumerate() {
        if i == 0 {
            if local_x < px / 2.0 {
                return 0;
            }
        } else {
            let mid = (positions[i - 1] + px) / 2.0;
            if local_x < mid {
                return i;
            }
        }
    }
    positions.len()
}

/// 한 시각 줄(line)에 속한 run 들(이미 bbox_x 오름차순 정렬) 안에서 클릭 x 를
/// 가장 가까운 문자 위치로 해석한다. 반환: (line_runs 내 인덱스, 절대 char offset).
///
/// 줄을 구성하는 run 이 1개든 여러 개든 동일하게:
///   - x 가 run bbox 안 → 정확한 글자 위치
///   - x 가 두 run 사이의 빈틈 → 더 가까운 쪽 경계(왼쪽 run 끝 / 오른쪽 run 시작)
///   - x 가 첫 run 왼쪽 → 첫 run 시작
///   - x 가 마지막 run 오른쪽 → 마지막 run 끝
///
/// 기존 fallback 들이 줄 시작/끝으로만 스냅하던 결함(클릭 y 가 글리프 bbox 의
/// 행간 여백에 떨어졌을 때, 그리고 다중 run 줄의 run 경계 빈틈)을 정정한다.
pub(crate) fn resolve_x_on_line(line_runs: &[LineRunView], x: f64) -> (usize, usize) {
    debug_assert!(!line_runs.is_empty());
    // 첫 run 왼쪽
    if x < line_runs[0].bbox_x {
        let r = &line_runs[0];
        return (0, r.char_start);
    }
    for (i, r) in line_runs.iter().enumerate() {
        if x <= r.bbox_x + r.bbox_w {
            // 이 run 의 bbox 안 (또는 이전 run 과의 빈틈 안)
            if x >= r.bbox_x {
                let local_x = x - r.bbox_x;
                // char_count 로 클램프: 빈 run(char_count=0)이지만 bbox_w 가 셀 폭만큼
                // 넓은 입력칸의 경우 char_positions(=[0.0])가 len()=1 을 돌려주어
                // char_start+1 (= 줄 끝 너머) 로 새던 결함을 막는다. 글자 인덱스는
                // 결코 그 run 의 char_count 를 넘을 수 없다.
                let local_idx = line_local_char_at_x(r.char_positions, local_x).min(r.char_count);
                return (i, r.char_start + local_idx);
            }
            // 이전 run 과 이 run 사이의 빈틈: 더 가까운 경계로 스냅
            let prev = &line_runs[i - 1];
            let prev_right = prev.bbox_x + prev.bbox_w;
            if (x - prev_right) <= (r.bbox_x - x) {
                return (i - 1, prev.char_start + prev.char_count);
            }
            return (i, r.char_start);
        }
    }
    // 마지막 run 오른쪽
    let last = line_runs.len() - 1;
    let r = &line_runs[last];
    (last, r.char_start + r.char_count)
}

impl DocumentCore {
    /// 1x1 TAC wrapper 표를 시각적으로 unwrap 하여 내부 표를 직접 렌더링한 경우,
    /// RenderNode 는 내부 표 cell_index 를 갖지만 cell_context 는 outer wrapper 표에
    /// 머무를 수 있다. 이 상태로는 `[(outer_ci, inner_cell_idx, ...)]` 처럼 존재하지
    /// 않는 outer cell 을 가리켜 Studio 커서 이동에서 by-path 조회가 실패한다.
    ///
    /// 모델을 확인해 outer 1x1 셀 안의 nested table control 로 path 를 복원한다.
    fn repair_unwrapped_wrapper_cell_context(&self, section_index: usize, ctx: &mut CellContext) {
        if ctx.path.len() != 1 {
            return;
        }
        let outer = ctx.path[0];
        let Some(table) = self
            .document
            .sections
            .get(section_index)
            .and_then(|s| s.paragraphs.get(ctx.parent_para_index))
            .and_then(|p| p.controls.get(outer.control_index))
            .and_then(|c| match c {
                Control::Table(t) => Some(t.as_ref()),
                _ => None,
            })
        else {
            return;
        };
        if table.cells.len() != 1 || outer.cell_index < table.cells.len() {
            return;
        }
        let Some(wrapper_cell) = table.cells.first() else {
            return;
        };
        let Some(wrapper_para) = wrapper_cell.paragraphs.first() else {
            return;
        };
        let Some(nested_ctrl_idx) =
            wrapper_para
                .controls
                .iter()
                .enumerate()
                .find_map(|(ci, ctrl)| match ctrl {
                    Control::Table(t) if outer.cell_index < t.cells.len() => Some(ci),
                    _ => None,
                })
        else {
            return;
        };

        if let Some(first) = ctx.path.first_mut() {
            first.cell_index = 0;
            first.cell_para_index = 0;
        }
        ctx.path.push(CellPathEntry {
            control_index: nested_ctrl_idx,
            cell_index: outer.cell_index,
            cell_para_index: outer.cell_para_index,
            text_direction: outer.text_direction,
        });
    }

    pub fn get_cursor_rect_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::{
            compute_char_positions, estimate_text_width, resolved_to_text_style,
        };
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 문단이 포함된 페이지 찾기
        let pages = self.find_pages_for_paragraph(section_idx, para_idx)?;

        let footnote_marker_positions: Vec<(usize, usize)> = self
            .get_render_paragraph_ref(section_idx, para_idx)
            .ok()
            .map(|para| {
                let ctrl_positions = find_logical_control_positions(para);
                para.controls
                    .iter()
                    .enumerate()
                    .filter(|(_, ctrl)| matches!(ctrl, Control::Footnote(_) | Control::Endnote(_)))
                    .filter_map(|(ci, _)| ctrl_positions.get(ci).copied().map(|pos| (ci, pos)))
                    .collect()
            })
            .unwrap_or_default();

        // 커서 결과를 담을 구조체
        #[derive(Clone, Copy)]
        struct CursorHit {
            page_index: u32,
            x: f64,
            y: f64,
            height: f64,
        }

        fn is_inline_cursor_control(ctrl: &Control) -> bool {
            is_treat_as_char_object_control(ctrl)
        }

        fn text_offset_after_same_pos_inline_controls(
            para: &Paragraph,
            text_offset: usize,
        ) -> usize {
            let ctrl_positions = para.control_text_positions();
            let same_or_before_count = para
                .controls
                .iter()
                .enumerate()
                .filter(|(_, ctrl)| is_inline_cursor_control(ctrl))
                .filter_map(|(ci, _)| ctrl_positions.get(ci))
                .filter(|&&pos| pos <= text_offset)
                .count();
            text_offset + same_or_before_count
        }

        fn collect_inline_control_bboxes(
            node: &RenderNode,
            sec: usize,
            para: usize,
            bboxes: &mut std::collections::HashMap<usize, (f64, f64, f64, f64)>,
        ) {
            match &node.node_type {
                RenderNodeType::Image(image_node) => {
                    if image_node.section_index == Some(sec) && image_node.para_index == Some(para)
                    {
                        if let Some(ci) = image_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Equation(eq_node) => {
                    if eq_node.section_index == Some(sec) && eq_node.para_index == Some(para) {
                        if let Some(ci) = eq_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Group(group_node) => {
                    if group_node.section_index == Some(sec) && group_node.para_index == Some(para)
                    {
                        if let Some(ci) = group_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Rectangle(rect_node) => {
                    if rect_node.section_index == Some(sec) && rect_node.para_index == Some(para) {
                        if let Some(ci) = rect_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Line(line_node) => {
                    if line_node.section_index == Some(sec) && line_node.para_index == Some(para) {
                        if let Some(ci) = line_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Ellipse(ell_node) => {
                    if ell_node.section_index == Some(sec) && ell_node.para_index == Some(para) {
                        if let Some(ci) = ell_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Path(path_node) => {
                    if path_node.section_index == Some(sec) && path_node.para_index == Some(para) {
                        if let Some(ci) = path_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                RenderNodeType::Table(table_node) => {
                    if table_node.section_index == Some(sec) && table_node.para_index == Some(para)
                    {
                        if let Some(ci) = table_node.control_index {
                            bboxes.insert(
                                ci,
                                (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                            );
                        }
                    }
                }
                _ => {}
            }

            for child in &node.children {
                collect_inline_control_bboxes(child, sec, para, bboxes);
            }
        }

        fn collect_text_caret_stops(
            node: &RenderNode,
            sec: usize,
            para_idx: usize,
            para: &Paragraph,
            raw_text_index: &mut usize,
            stops: &mut std::collections::BTreeMap<usize, CursorHit>,
            page_index: u32,
        ) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if text_run.section_index == Some(sec)
                    && text_run.para_index == Some(para_idx)
                    && text_run.cell_context.is_none()
                    && text_run.char_start.is_some()
                {
                    let para_chars: Vec<char> = para.text.chars().collect();
                    let run_chars: Vec<char> = text_run.text.chars().collect();
                    let positions = compute_char_positions(&text_run.text, &text_run.style);
                    let font_size = text_run.style.font_size;
                    let ascent = font_size * 0.8;
                    let caret_y = node.bbox.y + text_run.baseline - ascent;

                    if text_run.text.is_empty() && effective_char_count(text_run) == 0 {
                        if let Some(char_start) = text_run.char_start {
                            stops.entry(char_start).or_insert(CursorHit {
                                page_index,
                                x: node.bbox.x,
                                y: caret_y,
                                height: font_size.max(10.0),
                            });
                        }
                    }

                    for (idx, ch) in run_chars.iter().enumerate() {
                        if *ch == '\u{fffc}' {
                            continue;
                        }
                        if *raw_text_index >= para_chars.len() {
                            break;
                        }
                        if *ch != para_chars[*raw_text_index] {
                            continue;
                        }

                        let logical_start =
                            text_offset_after_same_pos_inline_controls(para, *raw_text_index);
                        let x0 = node.bbox.x + positions.get(idx).copied().unwrap_or(0.0);
                        let x1 = node.bbox.x
                            + positions
                                .get(idx + 1)
                                .copied()
                                .or_else(|| positions.last().copied())
                                .unwrap_or(0.0);

                        stops.entry(logical_start).or_insert(CursorHit {
                            page_index,
                            x: x0,
                            y: caret_y,
                            height: font_size,
                        });
                        stops.entry(logical_start + 1).or_insert(CursorHit {
                            page_index,
                            x: x1,
                            y: caret_y,
                            height: font_size,
                        });
                        *raw_text_index += 1;
                    }
                }
            }

            for child in &node.children {
                collect_text_caret_stops(
                    child,
                    sec,
                    para_idx,
                    para,
                    raw_text_index,
                    stops,
                    page_index,
                );
            }
        }

        fn find_inline_flow_cursor_hit(
            tree: &crate::renderer::render_tree::PageRenderTree,
            sec: usize,
            para_idx: usize,
            para: &Paragraph,
            offset: usize,
            page_index: u32,
        ) -> Option<CursorHit> {
            if !para.controls.iter().any(is_inline_cursor_control) {
                return None;
            }

            let mut stops = std::collections::BTreeMap::new();
            let mut raw_text_index = 0usize;
            collect_text_caret_stops(
                &tree.root,
                sec,
                para_idx,
                para,
                &mut raw_text_index,
                &mut stops,
                page_index,
            );

            let mut control_bboxes = std::collections::HashMap::new();
            collect_inline_control_bboxes(&tree.root, sec, para_idx, &mut control_bboxes);
            let ctrl_positions = find_logical_control_positions(para);
            let raw_ctrl_positions = para.control_text_positions();
            let mut inline_controls = Vec::new();

            for (ci, ctrl) in para.controls.iter().enumerate() {
                if !is_inline_cursor_control(ctrl) {
                    continue;
                }
                let Some(pos) = ctrl_positions.get(ci).copied() else {
                    continue;
                };
                let Some(raw_pos) = raw_ctrl_positions.get(ci).copied() else {
                    continue;
                };
                let Some((x, y, w, h)) = control_bboxes.get(&ci).copied() else {
                    continue;
                };
                inline_controls.push((ci, raw_pos, pos, x, x + w));

                let nearby_text_metrics = stops
                    .range(..=pos + 1)
                    .next_back()
                    .map(|(_, hit)| (hit.y, hit.height))
                    .or_else(|| stops.values().next().map(|hit| (hit.y, hit.height)));
                let line_metrics = nearby_text_metrics
                    .filter(|(text_y, text_h)| {
                        let text_mid = *text_y + *text_h / 2.0;
                        text_mid >= y && text_mid <= y + h
                    })
                    .unwrap_or_else(|| {
                        let fallback_h = 12.0;
                        let baseline = h * 0.85;
                        let ascent = fallback_h * 0.8;
                        (y + (baseline - ascent).max(0.0), fallback_h)
                    });

                stops.insert(
                    pos,
                    CursorHit {
                        page_index,
                        x,
                        y: line_metrics.0,
                        height: line_metrics.1,
                    },
                );
                stops.entry(pos + 1).or_insert(CursorHit {
                    page_index,
                    x: x + w,
                    y: line_metrics.0,
                    height: line_metrics.1,
                });
            }
            inline_controls.sort_by_key(|&(ci, raw_pos, pos, _, _)| (raw_pos, pos, ci));

            if para.text.is_empty()
                && para.controls.iter().all(|ctrl| {
                    matches!(ctrl, Control::Equation(_)) && is_treat_as_char_object_control(ctrl)
                })
            {
                let mut equation_controls = para
                    .controls
                    .iter()
                    .enumerate()
                    .filter_map(|(ci, ctrl)| {
                        if !matches!(ctrl, Control::Equation(_))
                            || !is_treat_as_char_object_control(ctrl)
                        {
                            return None;
                        }
                        let (x, y, w, h) = control_bboxes.get(&ci).copied()?;
                        let raw_pos = raw_ctrl_positions.get(ci).copied().unwrap_or(ci);
                        let pos = ctrl_positions.get(ci).copied().unwrap_or(ci);
                        Some((ci, raw_pos, pos, x, y, w, h))
                    })
                    .collect::<Vec<_>>();
                equation_controls.sort_by_key(|&(ci, raw_pos, pos, _, _, _, _)| (raw_pos, pos, ci));

                for (slot, (_, _, _, x, y, w, h)) in equation_controls.iter().enumerate() {
                    let fallback_h = 12.0;
                    let baseline = *h * 0.85;
                    let ascent = fallback_h * 0.8;
                    let caret_y = *y + (baseline - ascent).max(0.0);
                    if offset == slot * 2 {
                        return Some(CursorHit {
                            page_index,
                            x: *x,
                            y: caret_y,
                            height: fallback_h,
                        });
                    }
                    if offset == slot * 2 + 1 {
                        return Some(CursorHit {
                            page_index,
                            x: *x + *w,
                            y: caret_y,
                            height: fallback_h,
                        });
                    }
                }
            }

            let para_chars: Vec<char> = para.text.chars().collect();
            for pair in inline_controls.windows(2) {
                let (_, prev_raw, prev_pos, _, prev_right) = pair[0];
                let (_, next_raw, next_pos, next_left, _) = pair[1];
                if next_raw <= prev_raw || next_left <= prev_right {
                    continue;
                }

                let Some(between) = para_chars.get(prev_raw..next_raw) else {
                    continue;
                };
                if between.is_empty() || !between.iter().all(|ch| *ch == ' ') {
                    continue;
                }

                let space_count = between.len();
                if next_pos != prev_pos + 1 + space_count {
                    continue;
                }

                let Some(metrics) = stops
                    .get(&(prev_pos + 1))
                    .or_else(|| stops.get(&next_pos))
                    .or_else(|| stops.values().next())
                    .copied()
                else {
                    continue;
                };

                for step in 0..=space_count {
                    let ratio = step as f64 / space_count as f64;
                    let x = prev_right + (next_left - prev_right) * ratio;
                    stops.insert(
                        prev_pos + 1 + step,
                        CursorHit {
                            page_index,
                            x,
                            y: metrics.y,
                            height: metrics.height,
                        },
                    );
                }
            }

            stops.get(&offset).copied()
        }

        let (is_list_para, list_marker_char_shape_id) = self
            .get_render_paragraph_ref(section_idx, para_idx)
            .ok()
            .map(|para| {
                let is_list = self
                    .styles
                    .para_styles
                    .get(para.para_shape_id as usize)
                    .map(|ps| {
                        matches!(
                            ps.head_type,
                            crate::model::style::HeadType::Outline
                                | crate::model::style::HeadType::Number
                                | crate::model::style::HeadType::Bullet
                        )
                    })
                    .unwrap_or(false);
                let marker_style_id = para.char_shapes.first().map(|cs| cs.char_shape_id);
                (is_list, marker_style_id)
            })
            .unwrap_or((false, None));
        let list_marker_char_shape_id = if is_list_para {
            list_marker_char_shape_id
        } else {
            None
        };

        // 렌더 트리에서 커서 위치를 찾는 재귀 함수
        // exact_only: true이면 정확한 매칭(zero-width 앵커)만 반환
        fn find_cursor_in_node(
            node: &RenderNode,
            sec: usize,
            para: usize,
            render_para: Option<&Paragraph>,
            offset: usize,
            page_index: u32,
            exact_only: bool,
            is_list_para: bool,
            footnote_marker_positions: &[(usize, usize)],
        ) -> Option<CursorHit> {
            if let RenderNodeType::FootnoteMarker(ref marker) = node.node_type {
                if marker.section_index == sec && marker.para_index == para {
                    if let Some((_, marker_pos)) = footnote_marker_positions
                        .iter()
                        .find(|(ci, _)| *ci == marker.control_index)
                    {
                        if offset == *marker_pos || offset == *marker_pos + 1 {
                            return Some(CursorHit {
                                page_index,
                                x: if offset == *marker_pos {
                                    node.bbox.x
                                } else {
                                    node.bbox.x + node.bbox.width
                                },
                                y: node.bbox.y,
                                height: node.bbox.height.max(10.0),
                            });
                        }
                    }
                }
            }

            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                // 번호/글머리표 TextRun (char_start: None)은 건너뛴다
                if let Some(char_start) = text_run.char_start {
                    if text_run.section_index == Some(sec)
                        && text_run.para_index == Some(para)
                        && text_run.cell_context.is_none()
                    {
                        let char_count = effective_char_count(text_run);

                        // 커서가 이 TextRun 범위 안에 있는지 확인
                        // char_start <= offset <= char_start + char_count
                        if offset >= char_start && offset <= char_start + char_count {
                            if node.bbox.width <= f64::EPSILON
                                && !text_run.text.is_empty()
                                && text_run.text.chars().all(|ch| ch == '\u{fffc}')
                            {
                                if let Some(render_para) = render_para {
                                    let visible: String = render_para
                                        .text
                                        .chars()
                                        .skip(char_start)
                                        .take(char_count)
                                        .collect();
                                    if !visible.is_empty()
                                        && visible.chars().any(|ch| ch != '\u{fffc}')
                                    {
                                        let positions =
                                            compute_char_positions(&visible, &text_run.style);
                                        let local_offset = offset - char_start;
                                        let x_in_run = if local_offset < positions.len() {
                                            positions[local_offset]
                                        } else if !positions.is_empty() {
                                            *positions.last().unwrap()
                                        } else {
                                            0.0
                                        };
                                        let font_size = text_run.style.font_size;
                                        let ascent = font_size * 0.8;
                                        let caret_y = node.bbox.y + text_run.baseline - ascent;
                                        return Some(CursorHit {
                                            page_index,
                                            x: node.bbox.x + x_in_run,
                                            y: caret_y,
                                            height: font_size,
                                        });
                                    }
                                }
                                // 실제 문단 텍스트로 복원할 수 없는 placeholder는 cursor hit에서 제외한다.
                            } else {
                                let empty_list_anchor = is_list_para
                                    && char_count == 0
                                    && offset == char_start
                                    && text_run.text.is_empty();
                                // exact_only 모드: zero-width 앵커(bbox.width==0)만 허용
                                if empty_list_anchor {
                                    // 빈 번호/글머리표 문단의 body anchor 는 marker 앞쪽 x 에
                                    // 놓일 수 있으므로 fallback 에서 marker 오른쪽 끝으로 보정한다.
                                } else if exact_only
                                    && !(char_count == 0
                                        && offset == char_start
                                        && node.bbox.width == 0.0)
                                {
                                    // skip: 이 TextRun은 경계 매칭일 뿐 정확한 앵커가 아님
                                } else {
                                    let local_offset = offset - char_start;
                                    // PUA 다자리 글자겹침: 커서 위치는 [0.0, bbox.width]
                                    let positions =
                                        if text_run.char_overlap.is_some() && char_count == 1 {
                                            vec![0.0, node.bbox.width]
                                        } else {
                                            compute_char_positions(&text_run.text, &text_run.style)
                                        };
                                    let x_in_run = if local_offset < positions.len() {
                                        positions[local_offset]
                                    } else if !positions.is_empty() {
                                        *positions.last().unwrap()
                                    } else {
                                        0.0
                                    };
                                    // 베이스라인 기반 캐럿 y 계산:
                                    // 같은 줄에 서로 다른 글꼴 크기가 혼재할 때
                                    // 각 글자의 ascent 위치에서 캐럿이 시작되어야 함
                                    let font_size = text_run.style.font_size;
                                    let ascent = font_size * 0.8;
                                    let caret_y = node.bbox.y + text_run.baseline - ascent;
                                    return Some(CursorHit {
                                        page_index,
                                        x: node.bbox.x + x_in_run,
                                        y: caret_y,
                                        height: font_size,
                                    });
                                }
                            }
                        }
                    }
                } // if let Some(char_start)

                // 도형 조판부호 마커 (char_start=None, ShapeMarker(pos))
                if let crate::renderer::render_tree::FieldMarkerType::ShapeMarker(marker_pos) =
                    text_run.field_marker
                {
                    if text_run.section_index == Some(sec)
                        && text_run.para_index == Some(para)
                        && text_run.cell_context.is_none()
                    {
                        let font_size = text_run.style.font_size;
                        let ascent = font_size * 0.8;
                        let caret_y = node.bbox.y + text_run.baseline - ascent;
                        if marker_pos == offset {
                            // 마커 왼쪽 (마커 앞)
                            return Some(CursorHit {
                                page_index,
                                x: node.bbox.x,
                                y: caret_y,
                                height: font_size.max(10.0),
                            });
                        }
                        if marker_pos + 1 == offset {
                            // 마커 오른쪽 (마커 뒤)
                            return Some(CursorHit {
                                page_index,
                                x: node.bbox.x + node.bbox.width,
                                y: caret_y,
                                height: font_size.max(10.0),
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_in_node(
                    child,
                    sec,
                    para,
                    render_para,
                    offset,
                    page_index,
                    exact_only,
                    is_list_para,
                    footnote_marker_positions,
                ) {
                    return Some(hit);
                }
            }
            None
        }

        // 후보 페이지를 순회하며 커서 위치 탐색
        // 1차: 정확한 앵커(zero-width 노드) 우선 검색, 2차: 일반 검색
        for &page_num in &pages {
            let tree = self.build_page_tree_cached(page_num)?;
            if !self.show_control_codes {
                if let Ok(para) = self.get_render_paragraph_ref(section_idx, para_idx) {
                    if let Some(hit) = find_inline_flow_cursor_hit(
                        &tree,
                        section_idx,
                        para_idx,
                        para,
                        char_offset,
                        page_num,
                    ) {
                        return Ok(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                            hit.page_index, hit.x, hit.y, hit.height
                        ));
                    }
                }
            }
            let render_para_for_cursor = self.get_render_paragraph_ref(section_idx, para_idx).ok();
            let exact_hit = find_cursor_in_node(
                &tree.root,
                section_idx,
                para_idx,
                render_para_for_cursor,
                char_offset,
                page_num,
                true,
                is_list_para,
                &footnote_marker_positions,
            );
            let hit_result = exact_hit.or_else(|| {
                find_cursor_in_node(
                    &tree.root,
                    section_idx,
                    para_idx,
                    render_para_for_cursor,
                    char_offset,
                    page_num,
                    false,
                    is_list_para,
                    &footnote_marker_positions,
                )
            });
            if let Some(hit) = hit_result {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    hit.page_index, hit.x, hit.y, hit.height
                ));
            }
        }

        // 조판부호 감추기 모드: 인라인 도형 컨트롤 위치에서 커서 좌표 반환
        // treat_as_char Shape는 inline_shape_positions에서 좌표를 가져와 커서 표시
        if !self.show_control_codes {
            if let Ok(para) = self.get_render_paragraph_ref(section_idx, para_idx) {
                let text_len = para.text.chars().count();
                let ctrl_positions =
                    crate::document_core::helpers::find_logical_control_positions(para);

                // char_offset 위치에 인라인 컨트롤이 있는지 확인
                let inline_ctrl = para.controls.iter().enumerate().find(|(ci, ctrl)| {
                    is_inline_cursor_control(ctrl)
                        && ctrl_positions.get(*ci).copied() == Some(char_offset)
                        && char_offset != text_len
                });
                // 텍스트 범위 밖이지만 navigable 범위 내 (도형이 텍스트 뒤에 있을 때)
                let beyond_ctrl =
                    if char_offset > text_len && char_offset <= navigable_text_len(para) {
                        para.controls.iter().enumerate().find(|(ci, ctrl)| {
                            is_inline_cursor_control(ctrl)
                                && ctrl_positions.get(*ci).copied() == Some(char_offset)
                        })
                    } else {
                        None
                    };

                if let Some((ci, _ctrl)) = inline_ctrl.or(beyond_ctrl) {
                    // inline_shape_positions에서 Shape 좌표 조회
                    let first_page = pages[0];
                    let tree = self.build_page_tree(first_page)?;
                    if let Some((sx, sy)) =
                        tree.get_inline_shape_position(section_idx, para_idx, ci, None)
                    {
                        let shape_h = if let Some(Control::Shape(s)) = para.controls.get(ci) {
                            crate::renderer::hwpunit_to_px(
                                s.common().height as i32,
                                crate::renderer::DEFAULT_DPI,
                            )
                        } else if let Some(Control::Picture(p)) = para.controls.get(ci) {
                            crate::renderer::hwpunit_to_px(
                                p.common.height as i32,
                                crate::renderer::DEFAULT_DPI,
                            )
                        } else {
                            16.0
                        };
                        return Ok(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                            first_page, sx, sy, shape_h
                        ));
                    }
                }
            }
        }

        // TextRun에서 찾지 못한 경우 (빈 문단 등): 첫 페이지에서 문단 위치 추정
        let first_page = pages[0];
        let tree = self.build_page_tree(first_page)?;

        #[derive(Clone, Copy)]
        struct ParaLineHit {
            line_x: f64,
            y: f64,
            height: f64,
            first_body_x: Option<f64>,
            marker_end_x: Option<f64>,
        }

        impl ParaLineHit {
            fn cursor_x(self, is_list_para: bool, char_offset: usize) -> f64 {
                if is_list_para && char_offset == 0 {
                    self.marker_end_x
                        .or(self.first_body_x)
                        .unwrap_or(self.line_x)
                } else if is_list_para {
                    self.first_body_x
                        .or(self.marker_end_x)
                        .unwrap_or(self.line_x)
                } else {
                    self.line_x
                }
            }
        }

        fn collect_para_line_text_runs(
            node: &RenderNode,
            sec: usize,
            para: usize,
            is_list_para: bool,
            list_marker_char_shape_id: Option<u32>,
            styles: &crate::renderer::style_resolver::ResolvedStyleSet,
            hit: &mut ParaLineHit,
        ) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if text_run.section_index == Some(sec)
                    && text_run.para_index == Some(para)
                    && text_run.cell_context.is_none()
                {
                    if text_run.char_start.is_some() {
                        hit.first_body_x.get_or_insert(node.bbox.x);
                        hit.height = hit.height.max(text_run.style.font_size);
                    } else if is_list_para
                        && text_run.field_marker
                            == crate::renderer::render_tree::FieldMarkerType::None
                    {
                        let marker_width = list_marker_char_shape_id
                            .map(|cs_id| {
                                let marker_style = resolved_to_text_style(styles, cs_id, 0);
                                estimate_text_width(&text_run.text, &marker_style)
                            })
                            .unwrap_or(node.bbox.width);
                        hit.marker_end_x.get_or_insert(node.bbox.x + marker_width);
                        hit.height = hit.height.max(text_run.style.font_size);
                    }
                }
            }

            for child in &node.children {
                collect_para_line_text_runs(
                    child,
                    sec,
                    para,
                    is_list_para,
                    list_marker_char_shape_id,
                    styles,
                    hit,
                );
            }
        }

        // 해당 문단의 첫 TextLine을 찾아 빈 list 문단이면 marker 뒤 본문 시작 x까지 수집한다.
        fn find_para_line(
            node: &RenderNode,
            sec: usize,
            para: usize,
            is_list_para: bool,
            list_marker_char_shape_id: Option<u32>,
            styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        ) -> Option<ParaLineHit> {
            if let RenderNodeType::TextLine(ref line) = node.node_type {
                if line.section_index == Some(sec) && line.para_index == Some(para) {
                    let mut hit = ParaLineHit {
                        line_x: node.bbox.x,
                        y: node.bbox.y,
                        height: node.bbox.height,
                        first_body_x: None,
                        marker_end_x: None,
                    };
                    collect_para_line_text_runs(
                        node,
                        sec,
                        para,
                        is_list_para,
                        list_marker_char_shape_id,
                        styles,
                        &mut hit,
                    );
                    return Some(hit);
                }
            }

            // TextLine을 찾지 못하는 예외적인 렌더 트리에서는 기존 TextRun fallback을 보존한다.
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if text_run.section_index == Some(sec)
                    && text_run.para_index == Some(para)
                    && text_run.cell_context.is_none()
                    && text_run.char_start.is_some()
                {
                    return Some(ParaLineHit {
                        line_x: node.bbox.x,
                        y: node.bbox.y,
                        height: node.bbox.height,
                        first_body_x: Some(node.bbox.x),
                        marker_end_x: None,
                    });
                }
            }

            for child in &node.children {
                if let Some(r) = find_para_line(
                    child,
                    sec,
                    para,
                    is_list_para,
                    list_marker_char_shape_id,
                    styles,
                ) {
                    return Some(r);
                }
            }
            None
        }

        fn is_non_inline_ole_control(para: &Paragraph, control_index: usize) -> bool {
            if !para.text.is_empty() {
                return false;
            }
            para.controls.get(control_index).is_some_and(|control| {
                matches!(
                    control,
                    Control::Shape(shape)
                        if matches!(shape.as_ref(), crate::model::shape::ShapeObject::Ole(_))
                            && !shape.common().treat_as_char
                )
            })
        }

        fn ole_object_caret_metrics(
            para: &Paragraph,
            styles: &crate::renderer::style_resolver::ResolvedStyleSet,
            ole_top: f64,
        ) -> (f64, f64) {
            let line_height = para
                .line_segs
                .first()
                .map(|line| {
                    crate::renderer::hwpunit_to_px(line.line_height, crate::renderer::DEFAULT_DPI)
                })
                .filter(|height| *height > 0.0)
                .unwrap_or(13.0);
            let text_height = para
                .line_segs
                .first()
                .map(|line| {
                    crate::renderer::hwpunit_to_px(line.text_height, crate::renderer::DEFAULT_DPI)
                })
                .filter(|height| *height > 0.0)
                .unwrap_or(line_height);
            let baseline = para
                .line_segs
                .first()
                .map(|line| {
                    crate::renderer::hwpunit_to_px(
                        line.baseline_distance,
                        crate::renderer::DEFAULT_DPI,
                    )
                })
                .filter(|baseline| *baseline > 0.0)
                .unwrap_or(text_height * 0.85);
            let font_size = para
                .char_shape_id_at(0)
                .and_then(|id| styles.char_styles.get(id as usize))
                .map(|style| style.font_size)
                .filter(|size| *size > 0.0)
                .unwrap_or(text_height)
                .max(10.0);
            let caret_height = font_size.min(line_height.max(font_size));
            let caret_y = ole_top + (baseline - caret_height * 0.85).max(0.0);
            (caret_y, caret_height)
        }

        fn find_ole_object_cursor_hit(
            node: &RenderNode,
            sec: usize,
            para_idx: usize,
            render_para: Option<&Paragraph>,
            styles: &crate::renderer::style_resolver::ResolvedStyleSet,
            offset: usize,
            page_index: u32,
        ) -> Option<CursorHit> {
            if offset != 0 {
                return None;
            }

            let control_ref = match &node.node_type {
                RenderNodeType::RawSvg(raw_node) => raw_node.control_ref.as_ref(),
                RenderNodeType::Placeholder(placeholder_node) => {
                    placeholder_node.control_ref.as_ref()
                }
                _ => None,
            };
            if let Some(control_ref) = control_ref.filter(|control_ref| control_ref.kind == "ole") {
                if control_ref.section_index == sec
                    && control_ref.para_index == para_idx
                    && render_para.is_some_and(|para| {
                        is_non_inline_ole_control(para, control_ref.control_index)
                    })
                {
                    let (y, height) = render_para
                        .map(|para| ole_object_caret_metrics(para, styles, node.bbox.y))
                        .unwrap_or((node.bbox.y, 10.0));
                    return Some(CursorHit {
                        page_index,
                        x: node.bbox.x + node.bbox.width,
                        y,
                        height,
                    });
                }
            }

            for child in &node.children {
                if let Some(hit) = find_ole_object_cursor_hit(
                    child,
                    sec,
                    para_idx,
                    render_para,
                    styles,
                    offset,
                    page_index,
                ) {
                    return Some(hit);
                }
            }
            None
        }

        if let Some(hit) = find_ole_object_cursor_hit(
            &tree.root,
            section_idx,
            para_idx,
            self.get_render_paragraph_ref(section_idx, para_idx).ok(),
            &self.styles,
            char_offset,
            first_page,
        ) {
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                hit.page_index, hit.x, hit.y, hit.height
            ));
        }

        if let Some(line_hit) = find_para_line(
            &tree.root,
            section_idx,
            para_idx,
            is_list_para,
            list_marker_char_shape_id,
            &self.styles,
        ) {
            let x = line_hit.cursor_x(is_list_para, char_offset);
            let y = line_hit.y;
            let h = line_hit.height;
            // 인라인 도형 컨트롤이 있는 경우: char_offset에 따라 x 위치 조정
            let adjusted_x = if char_offset > 0 {
                // 해당 문단의 인라인 Shape/Picture/Table 노드 bbox를 수집
                fn collect_inline_bboxes(
                    node: &RenderNode,
                    sec: usize,
                    para: usize,
                    render_para: Option<&Paragraph>,
                    bboxes: &mut Vec<(f64, f64)>,
                ) {
                    fn is_caret_control(
                        render_para: Option<&Paragraph>,
                        control_index: Option<usize>,
                    ) -> bool {
                        let Some(ci) = control_index else {
                            return false;
                        };
                        render_para
                            .and_then(|para| para.controls.get(ci))
                            .is_some_and(is_treat_as_char_object_control)
                    }

                    match &node.node_type {
                        RenderNodeType::Line(ln)
                            if ln.section_index == Some(sec) && ln.para_index == Some(para) =>
                        {
                            if is_caret_control(render_para, ln.control_index) {
                                bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                            }
                        }
                        RenderNodeType::Rectangle(rn)
                            if rn.section_index == Some(sec) && rn.para_index == Some(para) =>
                        {
                            if is_caret_control(render_para, rn.control_index) {
                                bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                            }
                        }
                        RenderNodeType::Ellipse(en)
                            if en.section_index == Some(sec) && en.para_index == Some(para) =>
                        {
                            if is_caret_control(render_para, en.control_index) {
                                bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                            }
                        }
                        RenderNodeType::Table(tn)
                            if tn.section_index == Some(sec) && tn.para_index == Some(para) =>
                        {
                            if is_caret_control(render_para, tn.control_index) {
                                bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                            }
                        }
                        RenderNodeType::Image(im)
                            if im.section_index == Some(sec) && im.para_index == Some(para) =>
                        {
                            if is_caret_control(render_para, im.control_index) {
                                bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                            }
                        }
                        _ => {}
                    }
                    for child in &node.children {
                        collect_inline_bboxes(child, sec, para, render_para, bboxes);
                    }
                }
                let mut bboxes = Vec::new();
                let render_para = self.get_render_paragraph_ref(section_idx, para_idx).ok();
                collect_inline_bboxes(&tree.root, section_idx, para_idx, render_para, &mut bboxes);
                bboxes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                if char_offset <= bboxes.len() && !bboxes.is_empty() {
                    if char_offset >= bboxes.len() {
                        // 마지막 도형 뒤
                        bboxes.last().map_or(x, |b| b.1)
                    } else {
                        // char_offset번째 도형의 왼쪽
                        bboxes[char_offset].0
                    }
                } else {
                    x
                }
            } else {
                x
            };

            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                first_page, adjusted_x, y, h
            ));
        }

        Err(HwpError::RenderError(format!(
            "커서 위치를 찾을 수 없습니다: sec={}, para={}, offset={}",
            section_idx, para_idx, char_offset
        )))
    }

    /// 페이지 좌표에서 문서 위치 찾기 (네이티브)
    pub fn hit_test_native(&self, page_num: u32, x: f64, y: f64) -> Result<String, HwpError> {
        use crate::renderer::layout::{compute_char_positions, CellContext, CellPathEntry};
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree_cached(page_num)?;

        // 문자 위치를 미리 계산한 TextRun 정보
        struct RunInfo {
            section_index: usize,
            paragraph_index: usize,
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            // 셀/글상자 컨텍스트 (본문 텍스트는 None)
            cell_context: Option<CellContext>,
            is_textbox: bool,
            // 소속 칼럼 인덱스 (다단 지원)
            column_index: Option<u16>,
            // 소속 표 RenderNode id. 중첩 표에서 같은 cell_index가 반복되므로
            // TextRun과 TableCell bbox를 같은 표 단위로 묶는 데 사용한다.
            table_id: Option<u32>,
        }

        /// 안내문(guide text) TextRun 정보 (char_start: None)
        struct GuideRunInfo {
            section_index: usize,
            paragraph_index: usize,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            cell_context: Option<CellContext>,
        }

        /// 셀 bbox 정보
        struct CellBboxInfo {
            table_id: Option<u32>,
            section_index: usize,
            parent_para_index: usize,
            control_index: usize,
            cell_index: usize,
            text_direction: u8,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
            // Table 노드에서 meta가 채워졌는지 여부 (false이면 TextRun에서만 보완됨)
            has_meta: bool,
            cell_context: Option<CellContext>,
        }

        /// [Task #919] 글상자(TextBox) bbox 정보 — Shape 컨트롤의 외곽 도형 노드
        /// (Rectangle/Ellipse/Path) 가 layout_textbox_content 로 자식에 텍스트를 가진 경우.
        /// 글상자 안 빈 영역 클릭 시 첫 paragraph 진입에 사용.
        struct TextBoxBboxInfo {
            section_index: usize,
            parent_para_index: usize,
            control_index: usize,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
        }

        fn table_ctx_from_node(
            node: &RenderNode,
            current_table_ctx: Option<&CellContext>,
            current_cell_ctx: Option<&CellContext>,
        ) -> Option<CellContext> {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                match (tn.para_index, tn.control_index) {
                    (Some(pi), Some(ci)) => {
                        if let Some(parent_ctx) = current_cell_ctx {
                            let mut ctx = parent_ctx.clone();
                            if let Some(last) = ctx.path.last_mut() {
                                last.cell_para_index = pi;
                            }
                            ctx.path.push(CellPathEntry {
                                control_index: ci,
                                cell_index: 0,
                                cell_para_index: 0,
                                text_direction: 0,
                            });
                            Some(ctx)
                        } else {
                            Some(CellContext {
                                parent_para_index: pi,
                                path: vec![CellPathEntry {
                                    control_index: ci,
                                    cell_index: 0,
                                    cell_para_index: 0,
                                    text_direction: 0,
                                }],
                            })
                        }
                    }
                    _ => current_table_ctx.cloned(),
                }
            } else {
                current_table_ctx.cloned()
            }
        }

        fn cell_ctx_for_table_cell(
            table_ctx: Option<&CellContext>,
            cell_index: usize,
            cell_para_index: usize,
            text_direction: u8,
        ) -> Option<CellContext> {
            table_ctx.map(|ctx| {
                let mut cell_ctx = ctx.clone();
                if let Some(last) = cell_ctx.path.last_mut() {
                    last.cell_index = cell_index;
                    last.cell_para_index = cell_para_index;
                    last.text_direction = text_direction;
                }
                cell_ctx
            })
        }

        fn effective_cell_context(
            text_ctx: &Option<CellContext>,
            traversal_ctx: &Option<CellContext>,
        ) -> Option<CellContext> {
            match (text_ctx, traversal_ctx) {
                (Some(text_ctx), Some(traversal_ctx))
                    if traversal_ctx.path.len() >= text_ctx.path.len() =>
                {
                    let mut ctx = traversal_ctx.clone();
                    if let (Some(dst), Some(src)) = (ctx.path.last_mut(), text_ctx.path.last()) {
                        dst.cell_para_index = src.cell_para_index;
                        dst.text_direction = src.text_direction;
                    }
                    Some(ctx)
                }
                (Some(text_ctx), _) => Some(text_ctx.clone()),
                (None, _) => None,
            }
        }

        fn collect_runs(
            node: &RenderNode,
            runs: &mut Vec<RunInfo>,
            guide_runs: &mut Vec<GuideRunInfo>,
            cell_bboxes: &mut Vec<CellBboxInfo>,
            textbox_bboxes: &mut Vec<TextBoxBboxInfo>,
            current_column: Option<u16>,
            current_table_id: Option<u32>,
            // Table 노드에서 전파되는 (section_index, parent_para_index, control_index)
            current_table_meta: Option<(usize, usize, usize)>,
            current_table_ctx: Option<CellContext>,
            current_cell_ctx: Option<CellContext>,
        ) {
            // Column 노드 진입 시 칼럼 인덱스 전파
            let col = if let RenderNodeType::Column(col_idx) = node.node_type {
                Some(col_idx)
            } else {
                current_column
            };
            // Table 노드 진입 시 section_index / parent_para_index / control_index 전파
            let current_table_id = if matches!(node.node_type, RenderNodeType::Table(_)) {
                Some(node.id)
            } else {
                current_table_id
            };
            let table_ctx =
                table_ctx_from_node(node, current_table_ctx.as_ref(), current_cell_ctx.as_ref());
            let table_section_index = if let RenderNodeType::Table(ref tn) = node.node_type {
                tn.section_index
                    .or_else(|| current_table_meta.map(|(si, _, _)| si))
            } else {
                current_table_meta.map(|(si, _, _)| si)
            };
            let table_meta = if let Some(ref ctx) = table_ctx {
                table_section_index.map(|si| (si, ctx.parent_para_index, ctx.path[0].control_index))
            } else if let RenderNodeType::Table(ref tn) = node.node_type {
                match (tn.section_index, tn.para_index, tn.control_index) {
                    (Some(si), Some(pi), Some(ci)) => Some((si, pi, ci)),
                    _ => current_table_meta,
                }
            } else {
                current_table_meta
            };
            let mut child_cell_ctx = current_cell_ctx.clone();
            // [Task #919] 글상자(TextBox) bbox 수집 — Shape 컨트롤 (Rectangle/Ellipse/Path)
            // 노드가 layout_textbox_content 로 자식에 텍스트를 가진 경우. 외곽 도형 노드의
            // section/para/control 메타 + bbox 가 모두 채워져 있으므로 그대로 사용.
            // (글상자 안 빈 영역 클릭 시 첫 paragraph 진입에 사용)
            let textbox_meta: Option<(usize, usize, usize)> = match &node.node_type {
                RenderNodeType::Rectangle(r) => {
                    match (r.section_index, r.para_index, r.control_index) {
                        (Some(si), Some(pi), Some(ci)) => Some((si, pi, ci)),
                        _ => None,
                    }
                }
                RenderNodeType::Ellipse(e) => {
                    match (e.section_index, e.para_index, e.control_index) {
                        (Some(si), Some(pi), Some(ci)) => Some((si, pi, ci)),
                        _ => None,
                    }
                }
                RenderNodeType::Path(p) => match (p.section_index, p.para_index, p.control_index) {
                    (Some(si), Some(pi), Some(ci)) => Some((si, pi, ci)),
                    _ => None,
                },
                _ => None,
            };
            if let Some((si, pi, ci)) = textbox_meta {
                textbox_bboxes.push(TextBoxBboxInfo {
                    section_index: si,
                    parent_para_index: pi,
                    control_index: ci,
                    x: node.bbox.x,
                    y: node.bbox.y,
                    w: node.bbox.width,
                    h: node.bbox.height,
                });
            }
            // TableCell 노드의 bbox 수집
            if let RenderNodeType::TableCell(ref tc) = node.node_type {
                if let Some(cell_idx) = tc.model_cell_index {
                    let cell_ctx = cell_ctx_for_table_cell(
                        table_ctx.as_ref(),
                        cell_idx as usize,
                        0,
                        tc.text_direction,
                    );
                    child_cell_ctx = cell_ctx.clone();
                    // table_meta가 있으면 즉시 보완, 없으면 자식 TextRun에서 보완
                    let (si, ppi, ci, has_meta) = table_meta
                        .map(|(si, ppi, ci)| (si, ppi, ci, true))
                        .unwrap_or((0, 0, 0, false));
                    cell_bboxes.push(CellBboxInfo {
                        table_id: current_table_id,
                        section_index: si,
                        parent_para_index: ppi,
                        control_index: ci,
                        cell_index: cell_idx as usize,
                        text_direction: tc.text_direction,
                        x: node.bbox.x,
                        y: node.bbox.y,
                        w: node.bbox.width,
                        h: node.bbox.height,
                        has_meta,
                        cell_context: cell_ctx,
                    });
                }
            }
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let (Some(si), Some(pi)) = (text_run.section_index, text_run.para_index) {
                    let cell_context =
                        effective_cell_context(&text_run.cell_context, &current_cell_ctx);
                    // 머리말/꼬리말·각주 마커 TextRun 건너뛰기
                    if pi >= (usize::MAX - 3000) { /* skip marker runs */
                    } else if let Some(cs) = text_run.char_start {
                        let ecc = effective_char_count(text_run);
                        let positions = if text_run.char_overlap.is_some() && ecc == 1 {
                            vec![0.0, node.bbox.width]
                        } else {
                            compute_char_positions(&text_run.text, &text_run.style)
                        };
                        runs.push(RunInfo {
                            section_index: si,
                            paragraph_index: pi,
                            char_start: cs,
                            char_count: ecc,
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_w: node.bbox.width,
                            bbox_h: node.bbox.height,
                            cell_context,
                            is_textbox: false,
                            column_index: col,
                            table_id: current_table_id,
                        });
                    } else {
                        // char_start: None → 안내문 TextRun
                        guide_runs.push(GuideRunInfo {
                            section_index: si,
                            paragraph_index: pi,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_w: node.bbox.width,
                            bbox_h: node.bbox.height,
                            cell_context,
                        });
                    }
                }
            }
            for child in &node.children {
                collect_runs(
                    child,
                    runs,
                    guide_runs,
                    cell_bboxes,
                    textbox_bboxes,
                    col,
                    current_table_id,
                    table_meta,
                    table_ctx.clone(),
                    child_cell_ctx.clone(),
                );
            }
        }

        /// 줄 단위 x 해석을 모듈 스코프 `resolve_x_on_line` 에 위임한다.
        /// (모듈 스코프 함수는 문서/페이지 트리 없이 단위 테스트가 가능하다.)
        fn resolve_x_on_line(line_runs: &[&RunInfo], x: f64) -> (usize, usize) {
            let views: Vec<super::cursor_rect::LineRunView> = line_runs
                .iter()
                .map(|r| super::cursor_rect::LineRunView {
                    bbox_x: r.bbox_x,
                    bbox_w: r.bbox_w,
                    char_start: r.char_start,
                    char_count: r.char_count,
                    char_positions: &r.char_positions,
                })
                .collect();
            super::cursor_rect::resolve_x_on_line(&views, x)
        }

        fn format_hit(run: &RunInfo, offset: usize, page_num: u32) -> String {
            let base = format!(
                "\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{}",
                run.section_index, run.paragraph_index, offset
            );
            // 커서 x 좌표: char_positions로 정확한 위치 계산
            let cursor_x = if offset <= run.char_start {
                run.bbox_x
            } else {
                let local_idx = offset - run.char_start;
                if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                }
            };
            let cursor_rect = format!(
                ",\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                page_num, cursor_x, run.bbox_y, run.bbox_h
            );
            if let Some(ref ctx) = run.cell_context {
                let outer = &ctx.path[0];
                let tb = if run.is_textbox {
                    ",\"isTextBox\":true"
                } else {
                    ""
                };
                // cellPath: 전체 중첩 경로 배열
                let path_entries: Vec<String> = ctx
                    .path
                    .iter()
                    .map(|e| {
                        format!(
                            "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                            e.control_index, e.cell_index, e.cell_para_index
                        )
                    })
                    .collect();
                let cell_path = format!(",\"cellPath\":[{}]", path_entries.join(","));
                format!("{{{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}{}{}{}}}",
                    base, ctx.parent_para_index, outer.control_index, outer.cell_index, outer.cell_para_index,
                    cell_path, tb, cursor_rect)
            } else {
                format!("{{{}{}}}", base, cursor_rect)
            }
        }

        /// [Task #919] 글상자 빈 영역 클릭 시 글상자 첫 paragraph (cellParaIndex=0)
        /// 시작 위치로 진입 응답을 만든다. table-in-tbox.hwp 의 글상자 안 표 셀
        /// 빈 영역 등을 한컴 UX 정합으로 처리.
        fn format_textbox_entry(tb: &TextBoxBboxInfo, page_num: u32) -> String {
            // cell_index=0 (글상자 외곽 자체), cellParaIndex=0 (글상자 첫 paragraph)
            // 캐럿: 글상자 좌상단 + 약간 안쪽 패딩.
            let caret_h = (tb.h - 4.0).max(12.0).min(20.0);
            format!(
                "{{\"sectionIndex\":{},\"paragraphIndex\":0,\"charOffset\":0,\
                 \"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":0,\"cellParaIndex\":0,\
                 \"cellPath\":[{{\"controlIndex\":{},\"cellIndex\":0,\"cellParaIndex\":0}}],\
                 \"isTextBox\":true,\
                 \"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                tb.section_index,
                tb.parent_para_index,
                tb.control_index,
                tb.control_index,
                page_num,
                tb.x + 2.0,
                tb.y + 2.0,
                caret_h
            )
        }

        fn text_run_hit_allowed_by_textbox_bbox(
            run: &RunInfo,
            textbox_bboxes: &[TextBoxBboxInfo],
            x: f64,
            y: f64,
        ) -> bool {
            if !run.is_textbox {
                return true;
            }
            let Some(ctx) = run.cell_context.as_ref() else {
                return false;
            };
            let outer = &ctx.path[0];
            textbox_bboxes.iter().any(|tb| {
                tb.section_index == run.section_index
                    && tb.parent_para_index == ctx.parent_para_index
                    && tb.control_index == outer.control_index
                    && x >= tb.x
                    && x <= tb.x + tb.w
                    && y >= tb.y
                    && y <= tb.y + tb.h
            })
        }

        fn same_cell_context(a: &Option<CellContext>, b: &Option<CellContext>) -> bool {
            match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    a.parent_para_index == b.parent_para_index
                        && a.path.len() == b.path.len()
                        && a.path.iter().zip(&b.path).all(|(a, b)| {
                            a.control_index == b.control_index
                                && a.cell_index == b.cell_index
                                && a.cell_para_index == b.cell_para_index
                                && a.text_direction == b.text_direction
                        })
                }
                _ => false,
            }
        }

        fn inline_image_caret_metrics(y: f64, h: f64) -> (f64, f64) {
            let fallback_h = 12.0;
            let baseline = h * 0.85;
            let ascent = fallback_h * 0.8;
            (y + (baseline - ascent).max(0.0), fallback_h)
        }

        fn collect_body_inline_image_hits(
            core: &DocumentCore,
            node: &RenderNode,
            hits: &mut Vec<(usize, usize, usize, f64, f64, f64, f64)>,
        ) {
            if let RenderNodeType::Image(ref img) = node.node_type {
                if img.cell_context.is_none() {
                    if let (Some(si), Some(pi), Some(ci)) =
                        (img.section_index, img.para_index, img.control_index)
                    {
                        let char_offset = core
                            .document
                            .sections
                            .get(si)
                            .and_then(|section| section.paragraphs.get(pi))
                            .and_then(|para| {
                                let ctrl = para.controls.get(ci)?;
                                if !is_treat_as_char_object_control(ctrl) {
                                    return None;
                                }
                                find_logical_control_positions(para).get(ci).copied()
                            });
                        if let Some(char_offset) = char_offset {
                            hits.push((
                                si,
                                pi,
                                char_offset,
                                node.bbox.x,
                                node.bbox.y,
                                node.bbox.width,
                                node.bbox.height,
                            ));
                        }
                    }
                }
            }

            for child in &node.children {
                collect_body_inline_image_hits(core, child, hits);
            }
        }

        fn format_body_inline_image_hit(
            page_num: u32,
            si: usize,
            pi: usize,
            offset: usize,
            x: f64,
            y: f64,
            h: f64,
        ) -> String {
            format!(
                "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                si, pi, offset, page_num, x, y, h
            )
        }

        let mut runs: Vec<RunInfo> = Vec::new();
        let mut guide_runs: Vec<GuideRunInfo> = Vec::new();
        let mut cell_bboxes: Vec<CellBboxInfo> = Vec::new();
        let mut textbox_bboxes: Vec<TextBoxBboxInfo> = Vec::new();
        collect_runs(
            &tree.root,
            &mut runs,
            &mut guide_runs,
            &mut cell_bboxes,
            &mut textbox_bboxes,
            None,
            None,
            None,
            None,
            None,
        );

        // cell_bboxes의 section_index/parent_para_index/control_index/cellPath를 runs로 보완.
        // Table 노드에서 이미 채워진 최외곽 메타는 유지하되, 중첩 표의 Table 노드에는
        // 메타가 없을 수 있으므로 같은 RenderNode 표 안의 TextRun cell_context를 템플릿으로 쓴다.
        // cell_index는 표마다 반복되므로 같은 table_id 범위 안에서만 매칭해야 한다.
        for cb in &mut cell_bboxes {
            let same_cell_run = runs.iter().find(|r| {
                r.table_id == cb.table_id
                    && r.cell_context
                        .as_ref()
                        .map(|ctx| ctx.innermost().cell_index == cb.cell_index)
                        .unwrap_or(false)
            });
            let template_run = same_cell_run.or_else(|| {
                runs.iter()
                    .find(|r| r.table_id == cb.table_id && r.cell_context.is_some())
            });

            if let Some(run) = template_run {
                if let Some(ref ctx) = run.cell_context {
                    let mut cell_ctx = ctx.clone();
                    if let Some(last) = cell_ctx.path.last_mut() {
                        last.cell_index = cb.cell_index;
                        last.cell_para_index = 0;
                        last.text_direction = cb.text_direction;
                    }
                    cb.section_index = run.section_index;
                    cb.parent_para_index = cell_ctx.parent_para_index;
                    cb.control_index = cell_ctx.path[0].control_index;
                    cb.cell_context = Some(cell_ctx);
                    cb.has_meta = true;
                }
            }
        }

        // is_textbox 정확 판별: document의 실제 컨트롤 타입으로 재확인
        for run in &mut runs {
            if let Some(ref ctx) = run.cell_context {
                let outer = &ctx.path[0];
                if outer.cell_index == 0 {
                    let is_shape = self
                        .document
                        .sections
                        .get(run.section_index)
                        .and_then(|s| s.paragraphs.get(ctx.parent_para_index))
                        .and_then(|p| p.controls.get(outer.control_index))
                        .map(|c| matches!(c, Control::Shape(_)))
                        .unwrap_or(false);
                    run.is_textbox = is_shape;
                } else {
                    run.is_textbox = false;
                }
            } else {
                run.is_textbox = false;
            }
        }

        for run in &mut runs {
            if let Some(ref mut ctx) = run.cell_context {
                self.repair_unwrapped_wrapper_cell_context(run.section_index, ctx);
            }
        }
        for cb in &mut cell_bboxes {
            if let Some(ref mut ctx) = cb.cell_context {
                self.repair_unwrapped_wrapper_cell_context(cb.section_index, ctx);
                let outer = &ctx.path[0];
                cb.parent_para_index = ctx.parent_para_index;
                cb.control_index = outer.control_index;
                cb.cell_index = ctx.innermost().cell_index;
            }
        }

        let mut inline_image_hits = Vec::new();
        collect_body_inline_image_hits(self, &tree.root, &mut inline_image_hits);
        for (si, pi, char_offset, ix, iy, iw, ih) in inline_image_hits {
            let (caret_y, caret_h) = inline_image_caret_metrics(iy, ih);
            let right = ix + iw;
            if x >= ix && x <= right && y >= iy && y <= iy + ih {
                let offset = if x > ix + iw / 2.0 {
                    char_offset + 1
                } else {
                    char_offset
                };
                let caret_x = if offset == char_offset { ix } else { right };
                return Ok(format_body_inline_image_hit(
                    page_num, si, pi, offset, caret_x, caret_y, caret_h,
                ));
            }
            if x >= right && x <= right + caret_h && y >= caret_y && y <= caret_y + caret_h {
                return Ok(format_body_inline_image_hit(
                    page_num,
                    si,
                    pi,
                    char_offset + 1,
                    right,
                    caret_y,
                    caret_h,
                ));
            }
            if x <= ix && x >= ix - caret_h && y >= caret_y && y <= caret_y + caret_h {
                return Ok(format_body_inline_image_hit(
                    page_num,
                    si,
                    pi,
                    char_offset,
                    ix,
                    caret_y,
                    caret_h,
                ));
            }
        }

        // 0. 안내문(guide text) 히트 검사 — 필드 클릭 진입
        // 안내문 위 클릭 시 해당 필드의 시작 위치로 커서를 보낸다.
        for gr in &guide_runs {
            if x >= gr.bbox_x
                && x <= gr.bbox_x + gr.bbox_w
                && y >= gr.bbox_y
                && y <= gr.bbox_y + gr.bbox_h
            {
                let guide_char_offset = runs
                    .iter()
                    .find(|run| {
                        run.section_index == gr.section_index
                            && run.paragraph_index == gr.paragraph_index
                            && run.char_count == 0
                            && same_cell_context(&run.cell_context, &gr.cell_context)
                            && (run.bbox_x - gr.bbox_x).abs() < 0.5
                            && (run.bbox_y - gr.bbox_y).abs() < 1.0
                    })
                    .map(|run| run.char_start);
                // 필드 시작 위치 찾기: 해당 문단의 field_ranges에서 검색
                if let Some(field_hit) = self.find_field_hit_for_guide(
                    gr.section_index,
                    gr.paragraph_index,
                    &gr.cell_context,
                    page_num,
                    guide_char_offset,
                    gr.bbox_x,
                    gr.bbox_y,
                    gr.bbox_h,
                ) {
                    return Ok(field_hit);
                }
            }
        }

        if runs.is_empty() {
            // 텍스트가 없는 페이지: 첫 구역의 첫 문단 시작 반환
            let (page_content, _, _) = self.find_page(page_num)?;
            return Ok(format!(
                "{{\"sectionIndex\":{},\"paragraphIndex\":0,\"charOffset\":0}}",
                page_content.section_index
            ));
        }

        // 0.5. 인라인 Shape 히트 검사 (treat_as_char 도형 클릭)
        // inline_shape_positions에 등록된 Shape의 bbox를 검사하여
        // 클릭 시 해당 Shape의 텍스트 위치(char_offset)를 반환
        for (key, &(sx, sy)) in tree.inline_shape_positions() {
            let (si, pi, ci, ref cell_path) = *key;
            // [Task #1151 v4] 셀 안 inline picture/shape 도 hit-test 진입.
            // cell_path 가 있으면 셀 안 paragraph 에서 control 을 조회, 없으면 outer
            // paragraph 에서 조회 (기존 본문 path).
            if let Some(section) = self.document.sections.get(si) {
                let target_para = if cell_path.is_empty() {
                    section.paragraphs.get(pi)
                } else {
                    // cell_path 의 마지막 entry = picture/shape 가 있는 셀의 (table_ci,
                    // cell_idx, cell_para_idx). 중첩 표는 본 분기에서 첫 외곽 표 하나만
                    // resolve (셀 안 표 안 picture 는 후속 task).
                    let last = cell_path.last().copied().unwrap_or((0, 0, 0));
                    section
                        .paragraphs
                        .get(pi)
                        .and_then(|p| p.controls.get(last.0))
                        .and_then(|c| match c {
                            Control::Table(t) => t.cells.get(last.1),
                            _ => None,
                        })
                        .and_then(|cell| cell.paragraphs.get(last.2))
                };
                if let Some(para) = target_para {
                    if let Some(ctrl) = para.controls.get(ci) {
                        let (sw, sh) = match ctrl {
                            Control::Shape(s) => (
                                crate::renderer::hwpunit_to_px(
                                    s.common().width as i32,
                                    crate::renderer::DEFAULT_DPI,
                                ),
                                crate::renderer::hwpunit_to_px(
                                    s.common().height as i32,
                                    crate::renderer::DEFAULT_DPI,
                                ),
                            ),
                            Control::Picture(p) => (
                                crate::renderer::hwpunit_to_px(
                                    p.common.width as i32,
                                    crate::renderer::DEFAULT_DPI,
                                ),
                                crate::renderer::hwpunit_to_px(
                                    p.common.height as i32,
                                    crate::renderer::DEFAULT_DPI,
                                ),
                            ),
                            _ => continue,
                        };
                        if x >= sx && x <= sx + sw && y >= sy && y <= sy + sh {
                            // [Task #919] 글상자(Shape with text_box) 영역 hit — 본문
                            // TextRun 위치가 아닌 메인 매칭으로 fall-through 한다.
                            // 메인 매칭 (cell hit / textbox hit / body hit) 이 글상자
                            // 안 표 셀 / 글상자 빈 영역 / 본문 우선순위로 처리.
                            // (이 분기에서 글상자 안 표 셀을 가로채면 안 됨)
                            let shape_has_textbox = match ctrl {
                                Control::Shape(s) => match s.as_ref() {
                                    crate::model::shape::ShapeObject::Rectangle(r) => {
                                        r.drawing.text_box.is_some()
                                    }
                                    crate::model::shape::ShapeObject::Ellipse(e) => {
                                        e.drawing.text_box.is_some()
                                    }
                                    crate::model::shape::ShapeObject::Polygon(p) => {
                                        p.drawing.text_box.is_some()
                                    }
                                    crate::model::shape::ShapeObject::Arc(a) => {
                                        a.drawing.text_box.is_some()
                                    }
                                    crate::model::shape::ShapeObject::Curve(c) => {
                                        c.drawing.text_box.is_some()
                                    }
                                    _ => false,
                                },
                                _ => false,
                            };
                            if shape_has_textbox {
                                // 글상자는 메인 매칭에서 처리 — 0.5 분기 break.
                                break;
                            }
                            if !is_treat_as_char_object_control(ctrl) {
                                // 자리차지/글 앞으로/글 뒤로 개체는 본문 문자 슬롯이 아니므로
                                // 그림 bbox 클릭을 커서 앞/뒤 offset으로 변환하지 않는다.
                                continue;
                            }
                            let ctrl_positions =
                                crate::document_core::helpers::find_logical_control_positions(para);
                            let char_offset = ctrl_positions.get(ci).copied().unwrap_or(0);
                            // 클릭이 Shape 오른쪽 절반이면 Shape 뒤(offset+1)
                            let offset = if x > sx + sw / 2.0 {
                                char_offset + 1
                            } else {
                                char_offset
                            };
                            // 가장 가까운 TextRun을 찾아 format_hit 호출
                            let nearest = runs
                                .iter()
                                .enumerate()
                                .filter(|(_, r)| {
                                    r.section_index == si
                                        && r.paragraph_index == pi
                                        && r.cell_context.is_none()
                                })
                                .min_by_key(|(_, r)| {
                                    if offset >= r.char_start
                                        && offset <= r.char_start + r.char_count
                                    {
                                        0i64
                                    } else {
                                        (offset as i64 - r.char_start as i64).abs()
                                    }
                                });
                            if let Some((idx, _)) = nearest {
                                return Ok(format_hit(&runs[idx], offset, page_num));
                            }
                            // TextRun이 없으면 기본 반환
                            // [Task #1151 v4] cell_path 가 있으면 cellPath / innerControlIdx
                            // 정보 추가 — studio 측 picture select / 그림 속성 대화상자가
                            // cellPath 인식하여 셀 안 picture 를 정상 처리.
                            let cell_path_str = if cell_path.is_empty() {
                                String::new()
                            } else {
                                let entries: Vec<String> = cell_path
                                    .iter()
                                    .map(|(t, c, p)| format!("[{},{},{}]", t, c, p))
                                    .collect();
                                format!(
                                    ",\"cellPath\":[{}],\"innerControlIdx\":{}",
                                    entries.join(","),
                                    ci
                                )
                            };
                            return Ok(format!(
                                "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}{}}}",
                                si, pi, offset, page_num, sx, sy, sh, cell_path_str
                            ));
                        }
                    }
                }
            }
        }

        // 1. 정확한 bbox 히트 검사
        // 셀/글상자 TextRun을 본문 TextRun보다 우선한다.
        // (본문 TextRun이 컨트롤 높이만큼 큰 bbox를 가져서 글상자 영역을 덮을 수 있음)
        // 셀 후보가 여럿이면 bbox 면적이 가장 작은 것 = 가장 specific 한 셀 선택.
        // (Task #717 의 cell_bboxes selection L671-675 와 동일 best-match 패턴 — closes #857.
        //  중첩 표에서 외곽 셀의 빈 placeholder TextRun (bbox 가 paragraph 영역 전체) 이
        //  inner cell 의 실제 TextRun (작은 bbox) 보다 트리 순서상 먼저 매칭되어
        //  외곽이 선점되던 결함 정정.)
        let mut hit_body: Option<(usize, usize)> = None; // (run_idx, char_offset)
        let mut hit_cell: Option<(usize, usize)> = None;
        let mut hit_cell_area: Option<i64> = None;
        for (i, run) in runs.iter().enumerate() {
            if !text_run_hit_allowed_by_textbox_bbox(run, &textbox_bboxes, x, y) {
                continue;
            }
            if x >= run.bbox_x
                && x <= run.bbox_x + run.bbox_w
                && y >= run.bbox_y
                && y <= run.bbox_y + run.bbox_h
            {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x(&run.char_positions, local_x);
                if run.cell_context.is_some() {
                    let area = (run.bbox_w.max(0.0) * run.bbox_h.max(0.0) * 1000.0) as i64;
                    if hit_cell_area.map_or(true, |best_area| area < best_area) {
                        hit_cell = Some((i, run.char_start + char_offset));
                        hit_cell_area = Some(area);
                    }
                } else if hit_body.is_none() {
                    hit_body = Some((i, run.char_start + char_offset));
                }
            }
        }

        // [Task #919] 우선순위 — 가장 specific 한 매칭부터:
        //   1. hit_cell (셀 안 텍스트 위 hit — best-match area)
        //   2. clicked_cell (셀 bbox 매칭 — 텍스트가 없는 빈 셀 포함)
        //      ⇒ 글상자 안 표의 빈 셀도 여기서 매칭
        //   3. textbox_hit (글상자 안 빈 영역 — 셀 없는 영역)
        //   4. hit_body (본문 fall-through)
        //
        // 글상자 안 표 셀 (textbox 안 cell) 매칭이 textbox 영역보다 specific 이므로
        // clicked_cell 을 textbox_hit 보다 먼저 처리한다.
        if let Some((idx, offset)) = hit_cell {
            return Ok(format_hit(&runs[idx], offset, page_num));
        }

        // 클릭 좌표가 속한 칼럼 결정 (다단 지원)
        let click_column = self.find_column_at_x(page_num, x);

        // 2. 셀 bbox 기반으로 클릭한 셀 판별 (글상자 안 표 셀 포함)
        let clicked_cell: Option<&CellBboxInfo> = cell_bboxes
            .iter()
            .filter(|cb| cb.has_meta)
            .filter(|cb| x >= cb.x && x <= cb.x + cb.w && y >= cb.y && y <= cb.y + cb.h)
            .min_by_key(|cb| ((cb.w.max(0.0) * cb.h.max(0.0)) * 1000.0) as i64);

        // 셀 내부 클릭이면: 해당 셀의 run만 검색하여 가장 가까운 위치 반환
        if let Some(cb) = clicked_cell {
            let cell_runs: Vec<&RunInfo> = runs
                .iter()
                .filter(|r| {
                    r.cell_context
                        .as_ref()
                        .map(|ctx| {
                            r.table_id == cb.table_id
                                && ctx.parent_para_index == cb.parent_para_index
                                && ctx.innermost().cell_index == cb.cell_index
                        })
                        .unwrap_or(false)
                })
                .collect();

            if !cell_runs.is_empty() {
                // 클릭이 속한 시각 줄(line)을 고른다.
                //   1순위: 클릭 y 가 글리프 bbox 안에 드는 run 의 줄
                //   2순위: (행간 여백 클릭) 클릭 y 에 세로로 가장 가까운 run 의 줄
                // 줄 식별은 run 의 bbox_y(줄 윗변)로 한다. 같은 줄의 run 은 bbox_y 가 같다.
                let line_y = cell_runs
                    .iter()
                    .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
                    .map(|r| r.bbox_y)
                    .next()
                    .or_else(|| {
                        cell_runs
                            .iter()
                            .min_by(|a, b| {
                                let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
                                let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
                                da.partial_cmp(&db).unwrap()
                            })
                            .map(|r| r.bbox_y)
                    });

                if let Some(ly) = line_y {
                    let mut line_runs: Vec<&RunInfo> = cell_runs
                        .iter()
                        .copied()
                        .filter(|r| (r.bbox_y - ly).abs() < 1.0)
                        .collect();
                    line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());
                    let (idx, offset) = resolve_x_on_line(&line_runs, x);
                    return Ok(format_hit(line_runs[idx], offset, page_num));
                }
                return Ok(format_hit(cell_runs[0], cell_runs[0].char_start, page_num));
            }

            // 양식 컨트롤(FormObject)만 있는 셀: TextRun이 없어 cell_runs가 비어있음.
            // table_meta(또는 runs)에서 채워진 meta로 커서 진입.
            if cb.has_meta {
                let caret_h = (cb.h - 4.0).max(12.0);
                if let Some(ref ctx) = cb.cell_context {
                    let outer = &ctx.path[0];
                    let inner = ctx.innermost();
                    let path_entries: Vec<String> = ctx
                        .path
                        .iter()
                        .map(|e| {
                            format!(
                                "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                                e.control_index, e.cell_index, e.cell_para_index
                            )
                        })
                        .collect();
                    return Ok(format!(
                        "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":0,\
                         \"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{},\
                         \"cellPath\":[{}],\
                         \"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                        cb.section_index, inner.cell_para_index,
                        ctx.parent_para_index, outer.control_index, outer.cell_index, outer.cell_para_index,
                        path_entries.join(","),
                        page_num,
                        cb.x + 2.0, cb.y + 2.0, caret_h
                    ));
                }
                return Ok(format!(
                    "{{\"sectionIndex\":{},\"paragraphIndex\":0,\"charOffset\":0,\
                     \"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":0,\
                     \"cellPath\":[{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":0}}],\
                     \"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                    cb.section_index,
                    cb.parent_para_index, cb.control_index, cb.cell_index,
                    cb.control_index, cb.cell_index,
                    page_num,
                    cb.x + 2.0, cb.y + 2.0, caret_h
                ));
            }
        }

        // [Task #919] 글상자 빈 영역 매칭 — clicked_cell 매칭 안 됐으면 글상자 시도.
        // 글상자 안 표 셀은 위쪽 clicked_cell 분기에서 이미 처리됨.
        // 후보 여럿이면 가장 작은 (가장 specific) 것 선택.
        let textbox_hit: Option<&TextBoxBboxInfo> = textbox_bboxes
            .iter()
            .filter(|tb| x >= tb.x && x <= tb.x + tb.w && y >= tb.y && y <= tb.y + tb.h)
            .min_by_key(|tb| ((tb.w.max(0.0) * tb.h.max(0.0)) * 1000.0) as i64);
        if let Some(tb) = textbox_hit {
            return Ok(format_textbox_entry(tb, page_num));
        }

        // 본문 hit (1차 hit 검사에서 발견된 본문 paragraph)
        if let Some((idx, offset)) = hit_body {
            return Ok(format_hit(&runs[idx], offset, page_num));
        }

        // 같은 줄(y 범위)에서 가장 가까운 본문 TextRun 찾기
        // 다단: 클릭 칼럼의 run만 필터
        let mut same_line_runs: Vec<&RunInfo> = runs
            .iter()
            .filter(|r| r.cell_context.is_none()) // 본문 run만
            .filter(|r| text_run_hit_allowed_by_textbox_bbox(r, &textbox_bboxes, x, y))
            .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
            .filter(|r| {
                click_column.is_none() || r.column_index.is_none() || r.column_index == click_column
            })
            .collect();

        if !same_line_runs.is_empty() {
            same_line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());
            // 줄 안에서 클릭 x 를 가장 가까운 문자 위치로 해석
            // (다중 run 줄의 run 경계 빈틈 포함 — 줄 끝으로 스냅하지 않음)
            let (idx, offset) = resolve_x_on_line(&same_line_runs, x);
            return Ok(format_hit(same_line_runs[idx], offset, page_num));
        }

        // 3. 가장 가까운 줄 찾기 (y 거리 기준)
        // 다단: 클릭 칼럼의 run을 우선 후보로 사용
        let column_runs: Vec<&RunInfo> = runs
            .iter()
            .filter(|r| text_run_hit_allowed_by_textbox_bbox(r, &textbox_bboxes, x, y))
            .filter(|r| {
                click_column.is_none() || r.column_index.is_none() || r.column_index == click_column
            })
            .collect();
        let all_allowed_runs: Vec<&RunInfo> = runs
            .iter()
            .filter(|r| text_run_hit_allowed_by_textbox_bbox(r, &textbox_bboxes, x, y))
            .collect();
        let candidate_runs = if column_runs.is_empty() {
            &all_allowed_runs
        } else {
            &column_runs
        };
        if candidate_runs.is_empty() {
            let (page_content, _, _) = self.find_page(page_num)?;
            return Ok(format!(
                "{{\"sectionIndex\":{},\"paragraphIndex\":0,\"charOffset\":0}}",
                page_content.section_index
            ));
        }

        let closest = candidate_runs
            .iter()
            .min_by(|a, b| {
                let dist_a = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
                let dist_b = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        let target_y = closest.bbox_y;
        let target_h = closest.bbox_h;
        let mut line_runs: Vec<&RunInfo> = candidate_runs
            .iter()
            .filter(|r| (r.bbox_y - target_y).abs() < 1.0 && (r.bbox_h - target_h).abs() < 1.0)
            .copied()
            .collect();
        line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());

        // 줄 안에서 클릭 x 를 가장 가까운 문자 위치로 해석
        // (run 경계 빈틈 포함 — 줄 끝으로만 스냅하지 않음)
        let (idx, offset) = resolve_x_on_line(&line_runs, x);
        Ok(format_hit(line_runs[idx], offset, page_num))
    }

    /// 안내문 클릭 시 필드 시작 위치를 찾아 hitTest 결과를 반환한다.
    fn find_field_hit_for_guide(
        &self,
        section_index: usize,
        paragraph_index: usize,
        cell_context: &Option<crate::renderer::layout::CellContext>,
        page_num: u32,
        guide_char_offset: Option<usize>,
        guide_x: f64,
        guide_y: f64,
        guide_h: f64,
    ) -> Option<String> {
        use crate::model::control::{Control, FieldType};

        // 문단 접근: cell_context가 있으면 전체 경로를 따라가기 (중첩 표 지원)
        let para = if let Some(ctx) = cell_context {
            let path: Vec<(usize, usize, usize)> = ctx
                .path
                .iter()
                .map(|e| (e.control_index, e.cell_index, e.cell_para_index))
                .collect();
            self.resolve_paragraph_by_path(section_index, ctx.parent_para_index, &path)
                .ok()?
        } else {
            self.document
                .sections
                .get(section_index)?
                .paragraphs
                .get(paragraph_index)?
        };

        let build_hit = |char_offset: usize, field: &crate::model::control::Field| {
            let base = format!(
                "\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{}",
                section_index, paragraph_index, char_offset,
            );
            let cursor_rect = format!(
                ",\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                page_num, guide_x, guide_y, guide_h,
            );
            let field_info = format!(
                ",\"isField\":true,\"fieldId\":{},\"fieldType\":\"{}\"",
                field.field_id,
                field.field_type_str(),
            );
            if let Some(ctx) = cell_context {
                let outer = &ctx.path[0];
                let tb = if matches!(
                    self.document
                        .sections
                        .get(section_index)
                        .and_then(|s| s.paragraphs.get(ctx.parent_para_index))
                        .and_then(|p| p.controls.get(outer.control_index)),
                    Some(Control::Shape(_))
                ) {
                    ",\"isTextBox\":true"
                } else {
                    ""
                };
                let path_entries: Vec<String> = ctx
                    .path
                    .iter()
                    .map(|e| {
                        format!(
                            "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                            e.control_index, e.cell_index, e.cell_para_index
                        )
                    })
                    .collect();
                let cell_path = format!(",\"cellPath\":[{}]", path_entries.join(","));
                format!(
                    "{{{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}{}{}{}{}}}",
                    base, ctx.parent_para_index, outer.control_index,
                    outer.cell_index, outer.cell_para_index,
                    cell_path, tb, field_info, cursor_rect,
                )
            } else {
                format!("{{{}{}{}}}", base, field_info, cursor_rect)
            }
        };

        let mut first_empty_clickhere = None;
        let mut first_clickhere = None;

        // 안내문은 빈 ClickHere에만 표시되므로 빈 field range를 우선 매칭한다.
        for fr in &para.field_ranges {
            if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
                if field.field_type == FieldType::ClickHere {
                    let is_empty = fr.start_char_idx == fr.end_char_idx;
                    if is_empty {
                        if guide_char_offset == Some(fr.start_char_idx) {
                            return Some(build_hit(fr.start_char_idx, field));
                        }
                        if first_empty_clickhere.is_none() {
                            first_empty_clickhere = Some(build_hit(fr.start_char_idx, field));
                        }
                    }
                    if first_clickhere.is_none() {
                        first_clickhere = Some(build_hit(fr.start_char_idx, field));
                    }
                }
            }
        }

        first_empty_clickhere.or(first_clickhere)
    }

    /// 셀의 (col, row, pad_left_px, pad_top_px, pad_bottom_px)를 모델에서 조회한다.
    pub(crate) fn resolve_cell_position(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<(u16, u16, f64, f64, f64), HwpError> {
        use crate::model::control::Control;
        let para = self
            .document
            .sections
            .get(section_idx)
            .and_then(|s| s.paragraphs.get(parent_para_idx))
            .ok_or_else(|| HwpError::RenderError("문단 없음".to_string()))?;
        let ctrl = para
            .controls
            .get(control_idx)
            .ok_or_else(|| HwpError::RenderError("컨트롤 없음".to_string()))?;
        match ctrl {
            Control::Table(ref tbl) => {
                let cell = tbl
                    .cells
                    .get(cell_idx)
                    .ok_or_else(|| HwpError::RenderError("셀 없음".to_string()))?;
                let dpi_scale = 96.0 / 7200.0;
                Ok((
                    cell.col,
                    cell.row,
                    cell.padding.left as f64 * dpi_scale,
                    cell.padding.top as f64 * dpi_scale,
                    cell.padding.bottom as f64 * dpi_scale,
                ))
            }
            Control::Shape(_) | Control::Picture(_) => {
                // 글상자/그림 캡션은 패딩 없음
                Ok((0, 0, 0.0, 0.0, 0.0))
            }
            _ => Err(HwpError::RenderError("표 컨트롤이 아닙니다".to_string())),
        }
    }

    /// 표 셀 내부 커서의 픽셀 좌표를 반환한다 (네이티브)
    pub fn get_cursor_rect_in_cell_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 표 캡션은 cell_index=65534 센티널로 렌더 트리에도 동일하게 저장됨

        // 테이블이 포함된 본문 문단의 페이지 찾기
        let pages = self.find_pages_for_paragraph(section_idx, parent_para_idx)?;

        struct CursorHit {
            page_index: u32,
            x: f64,
            y: f64,
            height: f64,
        }

        fn find_cursor_in_cell(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            c_idx: usize,
            cp_idx: usize,
            offset: usize,
            page_index: u32,
        ) -> Option<CursorHit> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                let matches_cell = text_run.cell_context.as_ref().map_or(false, |ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path[0].control_index == ctrl_idx
                        && ctx.path[0].cell_index == c_idx
                        && ctx.path[0].cell_para_index == cp_idx
                });
                if matches_cell {
                    let char_start = text_run.char_start.unwrap_or(0);
                    let char_count = effective_char_count(text_run);

                    if offset >= char_start && offset <= char_start + char_count {
                        let local_offset = offset - char_start;
                        let positions = if text_run.char_overlap.is_some() && char_count == 1 {
                            vec![0.0, node.bbox.width]
                        } else {
                            compute_char_positions(&text_run.text, &text_run.style)
                        };
                        let x_in_run = if local_offset < positions.len() {
                            positions[local_offset]
                        } else if !positions.is_empty() {
                            *positions.last().unwrap()
                        } else {
                            0.0
                        };
                        // 베이스라인 기반 캐럿 y 계산 (본문과 동일)
                        let font_size = text_run.style.font_size;
                        let ascent = font_size * 0.8;
                        let caret_y = node.bbox.y + text_run.baseline - ascent;
                        return Some(CursorHit {
                            page_index,
                            x: node.bbox.x + x_in_run,
                            y: caret_y,
                            height: font_size,
                        });
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_in_cell(
                    child,
                    parent_para,
                    ctrl_idx,
                    c_idx,
                    cp_idx,
                    offset,
                    page_index,
                ) {
                    return Some(hit);
                }
            }
            None
        }

        for &page_num in &pages {
            let tree = self.build_page_tree(page_num)?;
            if let Some(hit) = find_cursor_in_cell(
                &tree.root,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                page_num,
            ) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    hit.page_index, hit.x, hit.y, hit.height
                ));
            }
        }

        // 빈 셀 fallback: 해당 셀의 아무 TextRun을 찾아 위치 반환
        let first_page = pages[0];
        let tree = self.build_page_tree(first_page)?;

        fn find_cell_run(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            c_idx: usize,
            cp_idx: usize,
        ) -> Option<(f64, f64, f64)> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                let matches_cell = text_run.cell_context.as_ref().map_or(false, |ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path[0].control_index == ctrl_idx
                        && ctx.path[0].cell_index == c_idx
                        && ctx.path[0].cell_para_index == cp_idx
                });
                if matches_cell {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            for child in &node.children {
                if let Some(r) = find_cell_run(child, parent_para, ctrl_idx, c_idx, cp_idx) {
                    return Some(r);
                }
            }
            None
        }

        if let Some((x, y, h)) = find_cell_run(
            &tree.root,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        ) {
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                first_page, x, y, h
            ));
        }

        // 빈 셀 최종 fallback: 모델에서 셀의 col/row를 조회한 뒤
        // 렌더 트리의 TableCell 노드 bbox + 패딩으로 커서 위치 산출
        let cell_pos =
            self.resolve_cell_position(section_idx, parent_para_idx, control_idx, cell_idx)?;

        fn find_table_cell_bbox(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            target_col: u16,
            target_row: u16,
        ) -> Option<(f64, f64, f64, f64)> {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                let matches_table =
                    tn.para_index == Some(parent_para) && tn.control_index == Some(ctrl_idx);
                if matches_table {
                    for child in &node.children {
                        if let RenderNodeType::TableCell(ref tc) = child.node_type {
                            if tc.col == target_col && tc.row == target_row {
                                return Some((
                                    child.bbox.x,
                                    child.bbox.y,
                                    child.bbox.width,
                                    child.bbox.height,
                                ));
                            }
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(r) =
                    find_table_cell_bbox(child, parent_para, ctrl_idx, target_col, target_row)
                {
                    return Some(r);
                }
            }
            None
        }

        if let Some((cx, cy, _cw, ch)) = find_table_cell_bbox(
            &tree.root,
            parent_para_idx,
            control_idx,
            cell_pos.0,
            cell_pos.1,
        ) {
            // 셀 bbox 좌상단 + 패딩 위치에 커서 배치
            let pad_left = cell_pos.2;
            let pad_top = cell_pos.3;
            let caret_h = (ch - pad_top - cell_pos.4).max(10.0); // 패딩 제외한 높이
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                first_page,
                cx + pad_left,
                cy + pad_top,
                caret_h
            ));
        }

        Err(HwpError::RenderError(format!(
            "셀 커서 위치를 찾을 수 없습니다: sec={}, parentPara={}, ctrl={}, cell={}, cellPara={}, offset={}",
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset
        )))
    }

    // ─── 컨테이너 렌더 범위 조회 ──────────────────────────────

    /// 지정된 컨테이너(글상자/표 셀) 내에서 실제로 렌더링된 마지막 문단 인덱스를 반환한다.
    /// 렌더 트리의 TextRun 노드 중 해당 컨테이너에 속한 것의 cell_para_index 최대값을 구한다.
    /// 렌더된 TextRun이 없으면 None을 반환한다.
    pub(crate) fn last_rendered_para_in_container(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Option<usize> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        fn max_para_in_node(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            c_idx: usize,
        ) -> Option<usize> {
            let mut result: Option<usize> = None;
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if let Some(ref ctx) = tr.cell_context {
                    if ctx.parent_para_index == parent_para
                        && ctx.path[0].control_index == ctrl_idx
                        && ctx.path[0].cell_index == c_idx
                    {
                        let cp = ctx.path[0].cell_para_index;
                        result = Some(result.map_or(cp, |prev: usize| prev.max(cp)));
                    }
                }
            }
            for child in &node.children {
                if let Some(cp) = max_para_in_node(child, parent_para, ctrl_idx, c_idx) {
                    result = Some(result.map_or(cp, |prev: usize| prev.max(cp)));
                }
            }
            result
        }

        // 해당 문단이 포함된 페이지들에서 검색
        let pages = self
            .find_pages_for_paragraph(section_idx, parent_para_idx)
            .ok()?;
        let mut max_para: Option<usize> = None;
        for &page_num in &pages {
            let tree = self.build_page_tree(page_num).ok()?;
            if let Some(cp) = max_para_in_node(&tree.root, parent_para_idx, control_idx, cell_idx) {
                max_para = Some(max_para.map_or(cp, |prev: usize| prev.max(cp)));
            }
        }
        max_para
    }

    // ─── 경로 기반 중첩 표 Native API ──────────────────────────

    /// 경로 기반 커서 좌표 조회 (네이티브).
    pub(crate) fn get_cursor_rect_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        self.get_cursor_rect_by_path_with_hint(
            section_idx,
            parent_para_idx,
            path_json,
            char_offset,
            None,
        )
    }

    /// [#2021] 후보 페이지를 힌트 페이지(±1) 우선으로 재배열한다 — 거대 표처럼 호스트
    /// 문단이 수십 페이지에 걸칠 때, 캐시 무효화 직후의 선형 페이지 트리 재빌드 탐색이
    /// 캐럿 페이지 인덱스에 비례해 커지는 것을 차단한다 (탐색 순서만 변경, 좌표 불변).
    fn order_pages_by_hint(pages: Vec<u32>, hint_page: Option<u32>) -> Vec<u32> {
        let Some(hint) = hint_page else { return pages };
        let mut ordered = Vec::with_capacity(pages.len());
        for cand in [Some(hint), hint.checked_sub(1), hint.checked_add(1)]
            .into_iter()
            .flatten()
        {
            if pages.contains(&cand) && !ordered.contains(&cand) {
                ordered.push(cand);
            }
        }
        for p in pages {
            if !ordered.contains(&p) {
                ordered.push(p);
            }
        }
        ordered
    }

    pub(crate) fn get_cursor_rect_by_path_with_hint(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
        char_offset: usize,
        hint_page: Option<u32>,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::{compute_char_positions, CellContext, CellPathEntry};
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = Self::parse_cell_path(path_json)?;
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let _para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, &path)?;

        // 커서 좌표를 렌더 트리에서 찾기 ([#2021] 힌트 페이지 우선 탐색)
        let pages = Self::order_pages_by_hint(
            self.find_pages_for_paragraph(section_idx, parent_para_idx)?,
            hint_page,
        );

        fn table_ctx_from_node(
            node: &RenderNode,
            current_table_ctx: Option<&CellContext>,
            current_cell_ctx: Option<&CellContext>,
        ) -> Option<CellContext> {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                match (tn.para_index, tn.control_index) {
                    (Some(pi), Some(ci)) => {
                        if let Some(parent_ctx) = current_cell_ctx {
                            let mut ctx = parent_ctx.clone();
                            if let Some(last) = ctx.path.last_mut() {
                                last.cell_para_index = pi;
                            }
                            ctx.path.push(CellPathEntry {
                                control_index: ci,
                                cell_index: 0,
                                cell_para_index: 0,
                                text_direction: 0,
                            });
                            Some(ctx)
                        } else {
                            Some(CellContext {
                                parent_para_index: pi,
                                path: vec![CellPathEntry {
                                    control_index: ci,
                                    cell_index: 0,
                                    cell_para_index: 0,
                                    text_direction: 0,
                                }],
                            })
                        }
                    }
                    _ => current_table_ctx.cloned(),
                }
            } else {
                current_table_ctx.cloned()
            }
        }

        fn cell_ctx_for_table_cell(
            table_ctx: Option<&CellContext>,
            cell_index: usize,
            cell_para_index: usize,
            text_direction: u8,
        ) -> Option<CellContext> {
            table_ctx.map(|ctx| {
                let mut cell_ctx = ctx.clone();
                if let Some(last) = cell_ctx.path.last_mut() {
                    last.cell_index = cell_index;
                    last.cell_para_index = cell_para_index;
                    last.text_direction = text_direction;
                }
                cell_ctx
            })
        }

        fn effective_cell_context(
            text_ctx: &Option<CellContext>,
            traversal_ctx: &Option<CellContext>,
        ) -> Option<CellContext> {
            match (text_ctx, traversal_ctx) {
                (Some(text_ctx), Some(traversal_ctx))
                    if traversal_ctx.path.len() >= text_ctx.path.len() =>
                {
                    let mut ctx = traversal_ctx.clone();
                    if let (Some(dst), Some(src)) = (ctx.path.last_mut(), text_ctx.path.last()) {
                        dst.cell_para_index = src.cell_para_index;
                        dst.text_direction = src.text_direction;
                    }
                    Some(ctx)
                }
                (Some(text_ctx), _) => Some(text_ctx.clone()),
                (None, _) => None,
            }
        }

        fn cell_context_matches(
            ctx: &Option<CellContext>,
            parent_para: usize,
            path: &[(usize, usize, usize)],
        ) -> bool {
            ctx.as_ref().map_or(false, |ctx| {
                ctx.parent_para_index == parent_para
                    && ctx.path.len() == path.len()
                    && ctx.path.iter().zip(path.iter()).all(|(a, b)| {
                        a.control_index == b.0 && a.cell_index == b.1 && a.cell_para_index == b.2
                    })
            })
        }

        // 렌더 트리에서 경로가 일치하는 TextRun 찾기
        fn find_cursor_by_path(
            core: &DocumentCore,
            node: &RenderNode,
            section_idx: usize,
            parent_para: usize,
            path: &[(usize, usize, usize)],
            offset: usize,
            page: u32,
            current_table_ctx: Option<CellContext>,
            current_cell_ctx: Option<CellContext>,
        ) -> Option<(u32, f64, f64, f64)> {
            let table_ctx =
                table_ctx_from_node(node, current_table_ctx.as_ref(), current_cell_ctx.as_ref());
            let mut child_cell_ctx = current_cell_ctx.clone();
            if let RenderNodeType::TableCell(ref tc) = node.node_type {
                if let Some(cell_idx) = tc.model_cell_index {
                    child_cell_ctx = cell_ctx_for_table_cell(
                        table_ctx.as_ref(),
                        cell_idx as usize,
                        0,
                        tc.text_direction,
                    );
                }
            }
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                let mut cell_context = effective_cell_context(&tr.cell_context, &current_cell_ctx);
                if let Some(ref mut ctx) = cell_context {
                    core.repair_unwrapped_wrapper_cell_context(section_idx, ctx);
                }
                if cell_context_matches(&cell_context, parent_para, path) {
                    let cs = tr.char_start.unwrap_or(0);
                    let cc = effective_char_count(tr);
                    if offset >= cs && offset <= cs + cc {
                        let positions = if tr.char_overlap.is_some() && cc == 1 {
                            vec![0.0, node.bbox.width]
                        } else {
                            compute_char_positions(&tr.text, &tr.style)
                        };
                        let lo = offset - cs;
                        let xr = if lo < positions.len() {
                            positions[lo]
                        } else if !positions.is_empty() {
                            *positions.last().unwrap()
                        } else {
                            0.0
                        };
                        return Some((page, node.bbox.x + xr, node.bbox.y, node.bbox.height));
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_by_path(
                    core,
                    child,
                    section_idx,
                    parent_para,
                    path,
                    offset,
                    page,
                    table_ctx.clone(),
                    child_cell_ctx.clone(),
                ) {
                    return Some(hit);
                }
            }
            None
        }

        for &page_num in &pages {
            let tree = self.build_page_tree_cached(page_num)?;
            if let Some((pi, x, y, h)) = find_cursor_by_path(
                self,
                &tree.root,
                section_idx,
                parent_para_idx,
                &path,
                char_offset,
                page_num,
                None,
                None,
            ) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    pi, x, y, h
                ));
            }
        }

        // fallback: 아무 TextRun이라도 찾기
        for &page_num in &pages {
            let tree = self.build_page_tree_cached(page_num)?;
            fn find_any_run(
                core: &DocumentCore,
                node: &RenderNode,
                section_idx: usize,
                parent_para: usize,
                path: &[(usize, usize, usize)],
                page: u32,
                current_table_ctx: Option<CellContext>,
                current_cell_ctx: Option<CellContext>,
            ) -> Option<(u32, f64, f64, f64)> {
                let table_ctx = table_ctx_from_node(
                    node,
                    current_table_ctx.as_ref(),
                    current_cell_ctx.as_ref(),
                );
                let mut child_cell_ctx = current_cell_ctx.clone();
                if let RenderNodeType::TableCell(ref tc) = node.node_type {
                    if let Some(cell_idx) = tc.model_cell_index {
                        child_cell_ctx = cell_ctx_for_table_cell(
                            table_ctx.as_ref(),
                            cell_idx as usize,
                            0,
                            tc.text_direction,
                        );
                    }
                }
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    let mut cell_context =
                        effective_cell_context(&tr.cell_context, &current_cell_ctx);
                    if let Some(ref mut ctx) = cell_context {
                        core.repair_unwrapped_wrapper_cell_context(section_idx, ctx);
                    }
                    if cell_context_matches(&cell_context, parent_para, path) {
                        return Some((page, node.bbox.x, node.bbox.y, node.bbox.height));
                    }
                }
                for child in &node.children {
                    if let Some(hit) = find_any_run(
                        core,
                        child,
                        section_idx,
                        parent_para,
                        path,
                        page,
                        table_ctx.clone(),
                        child_cell_ctx.clone(),
                    ) {
                        return Some(hit);
                    }
                }
                None
            }
            if let Some((pi, x, y, h)) = find_any_run(
                self,
                &tree.root,
                section_idx,
                parent_para_idx,
                &path,
                page_num,
                None,
                None,
            ) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    pi, x, y, h
                ));
            }
        }

        Err(HwpError::RenderError(format!(
            "경로 기반 커서 위치를 찾을 수 없습니다: sec={}, ppi={}, path={}, offset={}",
            section_idx, parent_para_idx, path_json, char_offset
        )))
    }

    /// 경로 기반 셀 정보 조회 (네이티브).
    pub(crate) fn get_cell_info_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path(path_json)?;
        let cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &path)?;

        Ok(format!(
            "{{\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{}}}",
            cell.row, cell.col, cell.row_span, cell.col_span
        ))
    }

    /// 경로 기반 표 차원 조회 (네이티브).
    pub(crate) fn get_table_dimensions_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path(path_json)?;
        let table = self.resolve_table_by_path(section_idx, parent_para_idx, &path)?;

        Ok(format!(
            "{{\"rowCount\":{},\"colCount\":{},\"cellCount\":{}}}",
            table.row_count,
            table.col_count,
            table.cells.len()
        ))
    }

    /// 경로 기반 표 셀 바운딩박스 조회 (네이티브).
    pub(crate) fn get_table_cell_bboxes_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = Self::parse_cell_path(path_json)?;
        // 표가 존재하는지 검증
        let _table = self.resolve_table_by_path(section_idx, parent_para_idx, &path)?;

        // 렌더 트리에서 TextRun의 cell_context.path가 일치하는 TableCell의 부모 Table 노드를 찾는다
        fn find_nested_table_cells(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
            page_idx: usize,
            result: &mut Vec<String>,
        ) -> bool {
            // Table 노드를 발견하면 자식 TableCell 중 TextRun의 cell_context가 경로와 일치하는지 확인
            if let RenderNodeType::Table(_) = node.node_type {
                // 이 테이블의 셀에서 TextRun을 찾아 경로 매칭 여부 확인
                fn check_table_match(
                    node: &RenderNode,
                    parent_para: usize,
                    path: &[(usize, usize, usize)],
                ) -> bool {
                    if let RenderNodeType::TextRun(ref tr) = node.node_type {
                        return tr.cell_context.as_ref().map_or(false, |ctx| {
                            ctx.parent_para_index == parent_para
                                && ctx.path.len() == path.len()
                                && ctx.path.iter().zip(path.iter()).enumerate().all(
                                    |(i, (a, b))| {
                                        if i < path.len() - 1 {
                                            // 중간 경로: 전체 매칭 (어떤 셀/문단을 경유하는지)
                                            a.control_index == b.0
                                                && a.cell_index == b.1
                                                && a.cell_para_index == b.2
                                        } else {
                                            // 마지막 경로: control_index만 매칭 (이 표의 모든 셀 포함)
                                            a.control_index == b.0
                                        }
                                    },
                                )
                        });
                    }
                    for child in &node.children {
                        // 중첩 Table 노드는 건너뛴다 — find_nested_table_cells가 별도로 처리
                        if matches!(child.node_type, RenderNodeType::Table(_)) {
                            continue;
                        }
                        if check_table_match(child, parent_para, path) {
                            return true;
                        }
                    }
                    false
                }

                if check_table_match(node, parent_para, path) {
                    // 이 테이블의 직속 셀 bbox 수집
                    for (cell_idx, child) in node.children.iter().enumerate() {
                        if let RenderNodeType::TableCell(ref cn) = child.node_type {
                            result.push(format!(
                                "{{\"cellIdx\":{},\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}}",
                                cell_idx, cn.row, cn.col, cn.row_span, cn.col_span,
                                page_idx,
                                child.bbox.x, child.bbox.y, child.bbox.width, child.bbox.height
                            ));
                        }
                    }
                    return true;
                }
            }

            for child in &node.children {
                if find_nested_table_cells(child, parent_para, path, page_idx, result) {
                    return true;
                }
            }
            false
        }

        let mut cells = Vec::new();
        let total_pages = self.page_count() as usize;
        let mut found = false;
        for page_num in 0..total_pages {
            let tree = self.build_page_tree(page_num as u32)?;
            if find_nested_table_cells(&tree.root, parent_para_idx, &path, page_num, &mut cells) {
                found = true;
            } else if found {
                // 이전 페이지에서 표를 찾았으나 이 페이지에는 없음 → 표가 끝남
                break;
            }
        }

        if cells.is_empty() {
            return Err(HwpError::RenderError(format!(
                "경로 기반 표 셀 bbox를 찾을 수 없습니다: sec={}, ppi={}, path={}",
                section_idx, parent_para_idx, path_json
            )));
        }

        Ok(format!("[{}]", cells.join(",")))
    }

    /// 경로 기반 수직 커서 이동 (네이티브).
    pub(crate) fn move_vertical_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
        char_offset: usize,
        delta: i32,
        preferred_x: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;

        let path = Self::parse_cell_path(path_json)?;
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &path)?;
        let cell_para_count = cell.paragraphs.len();
        let current_para_idx = path.last().unwrap().2; // cellParaIndex

        let para = cell.paragraphs.get(current_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀문단 {} 범위 초과", current_para_idx))
        })?;

        // preferredX 결정
        let actual_px = if preferred_x < 0.0 {
            match self.get_cursor_rect_by_path_native(
                section_idx,
                parent_para_idx,
                path_json,
                char_offset,
            ) {
                Ok(json) => super::super::helpers::json_f64(&json, "x").unwrap_or(0.0),
                Err(_) => 0.0,
            }
        } else {
            preferred_x
        };

        // 줄 정보 계산
        let line_info =
            Self::compute_line_info_struct(para, char_offset).unwrap_or(LineInfoResult {
                line_index: 0,
                line_count: 1,
                char_start: 0,
                char_end: navigable_text_len(para),
            });
        let target_line = line_info.line_index as i32 + delta;

        // 결과: (new_path, new_char_offset)
        let (new_path, new_offset) = if target_line >= 0
            && (target_line as usize) < line_info.line_count
        {
            // CASE A: 같은 문단 내 다른 줄 — preferredX 기반 오프셋 찾기
            let target_range = Self::get_line_char_range(para, target_line as usize);
            let best = self.find_best_offset_by_x_in_path(
                section_idx,
                parent_para_idx,
                &path,
                current_para_idx,
                target_range.0,
                target_range.1,
                actual_px,
            );
            let mut p = path.clone();
            p.last_mut().unwrap().2 = current_para_idx;
            (p, best)
        } else if delta < 0 && current_para_idx > 0 {
            // CASE B-1: 이전 문단 마지막 줄
            let prev_para = current_para_idx - 1;
            let prev = &cell.paragraphs[prev_para];
            let prev_line_count = Self::compute_line_info_struct(prev, 0)
                .map(|li| li.line_count)
                .unwrap_or(1);
            let last_line = prev_line_count.saturating_sub(1);
            let target_range = Self::get_line_char_range(prev, last_line);
            let best = self.find_best_offset_by_x_in_path(
                section_idx,
                parent_para_idx,
                &path,
                prev_para,
                target_range.0,
                target_range.1,
                actual_px,
            );
            let mut p = path.clone();
            p.last_mut().unwrap().2 = prev_para;
            (p, best)
        } else if delta > 0 && current_para_idx + 1 < cell_para_count {
            // CASE B-2: 다음 문단 첫 줄
            let next_para = current_para_idx + 1;
            let next = &cell.paragraphs[next_para];
            let target_range = Self::get_line_char_range(next, 0);
            let best = self.find_best_offset_by_x_in_path(
                section_idx,
                parent_para_idx,
                &path,
                next_para,
                target_range.0,
                target_range.1,
                actual_px,
            );
            let mut p = path.clone();
            p.last_mut().unwrap().2 = next_para;
            (p, best)
        } else {
            // CASE C: 셀 경계 — 인접 셀 이동 시도
            let table = self.resolve_table_by_path(section_idx, parent_para_idx, &path)?;
            let last_entry = path.last().unwrap();
            let cell_idx = last_entry.1;
            let current_cell = &table.cells[cell_idx];

            let target_row = if delta > 0 {
                (current_cell.row + current_cell.row_span) as i32
            } else {
                current_cell.row as i32 - 1
            };

            if target_row >= 0 && (target_row as u16) < table.row_count {
                if let Some(target_cell_idx) =
                    table.cell_index_at(target_row as u16, current_cell.col)
                {
                    // 인접 셀로 이동
                    let target_cell = &table.cells[target_cell_idx];
                    let (target_cpi, target_line_idx) = if delta > 0 {
                        (0usize, 0usize)
                    } else {
                        let last_cpi = target_cell.paragraphs.len().saturating_sub(1);
                        let last_line = target_cell
                            .paragraphs
                            .get(last_cpi)
                            .map(|p| {
                                if p.line_segs.is_empty() {
                                    0
                                } else {
                                    p.line_segs.len() - 1
                                }
                            })
                            .unwrap_or(0);
                        (last_cpi, last_line)
                    };
                    let mut new_p = path.clone();
                    let last = new_p.last_mut().unwrap();
                    last.1 = target_cell_idx; // cellIndex 갱신
                    last.2 = target_cpi; // cellParaIndex 갱신

                    if let Some(target_para) = target_cell.paragraphs.get(target_cpi) {
                        let target_range = Self::get_line_char_range(target_para, target_line_idx);
                        let best = self.find_best_offset_by_x_in_path(
                            section_idx,
                            parent_para_idx,
                            &new_p,
                            target_cpi,
                            target_range.0,
                            target_range.1,
                            actual_px,
                        );
                        (new_p, best)
                    } else {
                        (new_p, 0)
                    }
                } else {
                    // 해당 행/열에 셀 없음 — 현재 위치 유지
                    (path.clone(), char_offset)
                }
            } else {
                // CASE D: 중첩 표 경계 탈출 — 부모 셀의 다음/이전 문단으로
                if path.len() >= 2 {
                    // 부모 레벨 경로로 올라감
                    let mut parent_path = path[..path.len() - 1].to_vec();
                    let parent_last = parent_path.last().unwrap();
                    let parent_cell =
                        self.resolve_cell_by_path(section_idx, parent_para_idx, &parent_path)?;
                    let parent_cpi = parent_last.2;

                    if delta > 0 && parent_cpi + 1 < parent_cell.paragraphs.len() {
                        // 부모 셀의 다음 문단 첫 줄
                        let next_cpi = parent_cpi + 1;
                        let next_para = &parent_cell.paragraphs[next_cpi];
                        let target_range = Self::get_line_char_range(next_para, 0);
                        parent_path.last_mut().unwrap().2 = next_cpi;
                        let best = self.find_best_offset_by_x_in_path(
                            section_idx,
                            parent_para_idx,
                            &parent_path,
                            next_cpi,
                            target_range.0,
                            target_range.1,
                            actual_px,
                        );
                        (parent_path, best)
                    } else if delta < 0 && parent_cpi > 0 {
                        // 부모 셀의 이전 문단 마지막 줄
                        let prev_cpi = parent_cpi - 1;
                        let prev_para = &parent_cell.paragraphs[prev_cpi];
                        let prev_line_count = Self::compute_line_info_struct(prev_para, 0)
                            .map(|li| li.line_count)
                            .unwrap_or(1);
                        let last_line = prev_line_count.saturating_sub(1);
                        let target_range = Self::get_line_char_range(prev_para, last_line);
                        parent_path.last_mut().unwrap().2 = prev_cpi;
                        let best = self.find_best_offset_by_x_in_path(
                            section_idx,
                            parent_para_idx,
                            &parent_path,
                            prev_cpi,
                            target_range.0,
                            target_range.1,
                            actual_px,
                        );
                        (parent_path, best)
                    } else {
                        // 부모 셀 경계에서도 더 이상 이동 불가 — 현재 위치 유지
                        (path.clone(), char_offset)
                    }
                } else {
                    // depth=1 표 경계 — 현재 위치 유지 (본문 탈출은 flat API에서 처리)
                    (path.clone(), char_offset)
                }
            }
        };

        let new_para = new_path.last().unwrap().2;
        let path_json_out = Self::format_path_json(&new_path);

        // 커서 좌표 획득
        let (rect_valid, page_idx, fx, fy, fh) = match self.get_cursor_rect_by_path_native(
            section_idx,
            parent_para_idx,
            &path_json_out,
            new_offset,
        ) {
            Ok(json) => (
                true,
                super::super::helpers::json_f64(&json, "pageIndex").unwrap_or(0.0) as usize,
                super::super::helpers::json_f64(&json, "x").unwrap_or(0.0),
                super::super::helpers::json_f64(&json, "y").unwrap_or(0.0),
                super::super::helpers::json_f64(&json, "height").unwrap_or(18.0),
            ),
            Err(_) => (false, 0, 0.0, 0.0, 18.0),
        };

        // MoveVerticalResult 형식 (톱레벨 pageIndex/x/y/height)
        let rect_valid_str = if rect_valid {
            ""
        } else {
            ",\"rectValid\":false"
        };
        Ok(format!(
            "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{},\"cellPath\":{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1},\"preferredX\":{:.1}{}}}",
            section_idx, new_para, new_offset,
            parent_para_idx, new_path[0].0, new_path[0].1, new_path[0].2,
            path_json_out,
            page_idx, fx, fy, fh, actual_px, rect_valid_str
        ))
    }

    /// 경로 기반 문단 내 지정 범위에서 preferredX에 가장 가까운 char offset을 찾는다.
    pub(crate) fn find_best_offset_by_x_in_path(
        &self,
        sec: usize,
        ppi: usize,
        path: &[(usize, usize, usize)],
        para_idx: usize,
        range_start: usize,
        range_end: usize,
        target_x: f64,
    ) -> usize {
        let mut best_offset = range_start;
        let mut best_dist = f64::MAX;

        // 경로에서 para_idx를 사용하는 새 경로 생성
        let mut test_path = path.to_vec();
        if let Some(last) = test_path.last_mut() {
            last.2 = para_idx;
        }
        let path_json = Self::format_path_json(&test_path);

        for offset in range_start..=range_end {
            if let Ok(json) = self.get_cursor_rect_by_path_native(sec, ppi, &path_json, offset) {
                if let Some(x) = super::super::helpers::json_f64(&json, "x") {
                    let dist = (x - target_x).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_offset = offset;
                    }
                }
            }
        }
        best_offset
    }

    /// CellPath를 JSON 문자열로 포맷한다.
    pub(crate) fn format_path_json(path: &[(usize, usize, usize)]) -> String {
        let entries: Vec<String> = path
            .iter()
            .map(|(ci, cei, cpi)| {
                format!(
                    "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                    ci, cei, cpi
                )
            })
            .collect();
        format!("[{}]", entries.join(","))
    }

    // ─── Phase 2 Native 끝 ──────────────────────────────────

    /// 클릭 x 좌표가 속한 칼럼 인덱스를 반환한다 (다단 히트 테스트용).
    pub(crate) fn find_column_at_x(&self, page_num: u32, x: f64) -> Option<u16> {
        let (page_content, _, _) = self.find_page(page_num).ok()?;
        let areas = &page_content.layout.column_areas;
        if areas.len() <= 1 {
            return None; // 단일 단 — 칼럼 필터링 불필요
        }
        for (i, area) in areas.iter().enumerate() {
            if x >= area.x && x <= area.x + area.width {
                return Some(i as u16);
            }
        }
        // 칼럼 영역 사이(간격)에 클릭한 경우 가장 가까운 칼럼 반환
        areas
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = (x - (a.x + a.width / 2.0)).abs();
                let db = (x - (b.x + b.width / 2.0)).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i as u16)
    }

    /// 머리말/꼬리말 내 커서 좌표를 반환한다.
    ///
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    pub fn get_cursor_rect_in_header_footer_native(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
        preferred_page: i32,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 머리말/꼬리말 문단의 para_index 마커 값
        // layout_header_footer_paragraphs에서 para_index = usize::MAX - i 로 설정됨
        let marker_para_idx = usize::MAX - hf_para_idx;

        // Header/Footer 노드 타입 판별
        let is_target_node = |nt: &RenderNodeType| -> bool {
            if is_header {
                matches!(nt, RenderNodeType::Header)
            } else {
                matches!(nt, RenderNodeType::Footer)
            }
        };

        struct CursorHit {
            page_index: u32,
            x: f64,
            y: f64,
            height: f64,
        }

        // Header/Footer 서브트리에서 TextRun 찾기
        fn find_cursor_in_hf_subtree(
            node: &RenderNode,
            marker_para: usize,
            offset: usize,
            page_index: u32,
        ) -> Option<CursorHit> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let Some(char_start) = text_run.char_start {
                    if text_run.para_index == Some(marker_para) && text_run.cell_context.is_none() {
                        let char_count = effective_char_count(text_run);
                        if offset >= char_start && offset <= char_start + char_count {
                            let local_offset = offset - char_start;
                            let positions = if text_run.char_overlap.is_some() && char_count == 1 {
                                vec![0.0, node.bbox.width]
                            } else {
                                compute_char_positions(&text_run.text, &text_run.style)
                            };
                            let x_in_run = if local_offset < positions.len() {
                                positions[local_offset]
                            } else if !positions.is_empty() {
                                *positions.last().unwrap()
                            } else {
                                0.0
                            };
                            let font_size = text_run.style.font_size;
                            let ascent = font_size * 0.8;
                            let caret_y = node.bbox.y + text_run.baseline - ascent;
                            return Some(CursorHit {
                                page_index,
                                x: node.bbox.x + x_in_run,
                                y: caret_y,
                                height: font_size,
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_in_hf_subtree(child, marker_para, offset, page_index)
                {
                    return Some(hit);
                }
            }
            None
        }

        // Header/Footer 노드에서 빈 문단 폴백
        fn find_hf_para_line(node: &RenderNode, marker_para: usize) -> Option<(f64, f64, f64)> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if text_run.para_index == Some(marker_para)
                    && text_run.cell_context.is_none()
                    && text_run.char_start.is_some()
                {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            if let RenderNodeType::TextLine(ref line) = node.node_type {
                if line.para_index == Some(marker_para) {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            for child in &node.children {
                if let Some(r) = find_hf_para_line(child, marker_para) {
                    return Some(r);
                }
            }
            None
        }

        // preferred_page가 지정되면 해당 페이지를 먼저 탐색
        let total_pages = self.page_count();
        let page_order: Vec<u32> = if preferred_page >= 0 && (preferred_page as u32) < total_pages {
            let pref = preferred_page as u32;
            std::iter::once(pref)
                .chain((0..total_pages).filter(move |&p| p != pref))
                .collect()
        } else {
            (0..total_pages).collect()
        };
        for page_num in page_order {
            let tree = self.build_page_tree(page_num)?;
            // 루트의 자식에서 Header/Footer 노드 찾기
            for child in &tree.root.children {
                if is_target_node(&child.node_type) {
                    if let Some(hit) =
                        find_cursor_in_hf_subtree(child, marker_para_idx, char_offset, page_num)
                    {
                        return Ok(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                            hit.page_index, hit.x, hit.y, hit.height
                        ));
                    }
                    // 빈 문단 폴백
                    if let Some((x, y, h)) = find_hf_para_line(child, marker_para_idx) {
                        return Ok(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                            page_num, x, y, h
                        ));
                    }
                    // Header/Footer 노드는 있지만 TextRun이 없는 경우 — 영역 좌표 반환
                    return Ok(format!(
                        "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                        page_num,
                        child.bbox.x,
                        child.bbox.y,
                        if child.bbox.height > 0.0 { 12.0 } else { 12.0 }
                    ));
                }
            }
        }

        Err(HwpError::RenderError(format!(
            "머리말/꼬리말 커서 위치를 찾을 수 없습니다: sec={}, is_header={}, hf_para={}",
            section_idx, is_header, hf_para_idx
        )))
    }

    /// 머리말/꼬리말 영역의 히트테스트
    ///
    /// 페이지 좌표가 머리말 또는 꼬리말 영역에 해당하는지 판별.
    /// 반환: JSON `{"hit":true,"isHeader":bool,"sectionIndex":N,"applyTo":N}`
    /// 또는 `{"hit":false}`
    ///
    /// Issue #595: Header/Footer 노드의 bbox 는 `expand_bbox_to_children` 으로
    /// 자식 (예: 단 구분선 line) 까지 확장되어 본문 영역을 침범할 수 있음.
    /// hit 판정은 layout 의 정확한 `header_area` / `footer_area` 로 수행하여
    /// bbox 확장과 무관하게 본질 영역만 hit.
    pub fn hit_test_header_footer_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        let (page_content, _, _) = self.find_page(page_num)?;
        let layout = &page_content.layout;

        // 머리말 영역 hit 판정 (layout.header_area — 정확한 머리말 범위)
        let h = &layout.header_area;
        if x >= h.x && x <= h.x + h.width && y >= h.y && y <= h.y + h.height {
            // active header에서 source_section_index와 apply_to 추출
            // 머리말은 이전 구역에서 상속될 수 있으므로 source_section_index 우선
            if let Some((source_sec, apply_to)) = self.get_active_hf_info(page_num, true) {
                return Ok(format!(
                    "{{\"hit\":true,\"isHeader\":true,\"sectionIndex\":{},\"applyTo\":{}}}",
                    source_sec, apply_to
                ));
            }
            // active 정보가 없는 경우 fallback (빈 머리말 영역 — 신규 생성 대상)
            let (section_idx, _) = self.find_section_for_page(page_num);
            return Ok(format!(
                "{{\"hit\":true,\"isHeader\":true,\"sectionIndex\":{},\"applyTo\":0}}",
                section_idx
            ));
        }

        // 꼬리말 영역 hit 판정 (layout.footer_area)
        let f = &layout.footer_area;
        if x >= f.x && x <= f.x + f.width && y >= f.y && y <= f.y + f.height {
            if let Some((source_sec, apply_to)) = self.get_active_hf_info(page_num, false) {
                return Ok(format!(
                    "{{\"hit\":true,\"isHeader\":false,\"sectionIndex\":{},\"applyTo\":{}}}",
                    source_sec, apply_to
                ));
            }
            let (section_idx, _) = self.find_section_for_page(page_num);
            return Ok(format!(
                "{{\"hit\":true,\"isHeader\":false,\"sectionIndex\":{},\"applyTo\":0}}",
                section_idx
            ));
        }

        Ok("{\"hit\":false}".to_string())
    }

    /// 페이지 번호로 구역 인덱스를 찾는다.
    fn find_section_for_page(&self, page_num: u32) -> (usize, usize) {
        let mut offset = 0u32;
        for (si, pr) in self.pagination.iter().enumerate() {
            let count = pr.pages.len() as u32;
            if page_num < offset + count {
                return (si, (page_num - offset) as usize);
            }
            offset += count;
        }
        (0, 0)
    }

    /// 해당 페이지에서 활성화된 머리말/꼬리말의 apply_to 값을 반환한다.
    fn get_active_hf_apply_to(&self, _section_idx: usize, page_num: u32, is_header: bool) -> u8 {
        self.get_active_hf_info(page_num, is_header)
            .map(|(_, apply_to)| apply_to)
            .unwrap_or(0)
    }

    /// 해당 페이지에서 활성화된 머리말/꼬리말의 (source_section_index, apply_to)를 반환한다.
    fn get_active_hf_info(&self, page_num: u32, is_header: bool) -> Option<(usize, u8)> {
        use crate::model::header_footer::HeaderFooterApply;

        let mut offset = 0u32;
        for (_si, pr) in self.pagination.iter().enumerate() {
            let count = pr.pages.len() as u32;
            if page_num < offset + count {
                let local_page = (page_num - offset) as usize;
                let page = &pr.pages[local_page];
                let hf_ref = if is_header {
                    &page.active_header
                } else {
                    &page.active_footer
                };
                if let Some(ref r) = hf_ref {
                    let source_sec = r.source_section_index;
                    if let Some(section) = self.document.sections.get(source_sec) {
                        if let Some(para) = section.paragraphs.get(r.para_index) {
                            if let Some(ctrl) = para.controls.get(r.control_index) {
                                let apply_to = match ctrl {
                                    Control::Header(h) => match h.apply_to {
                                        HeaderFooterApply::Both => 0,
                                        HeaderFooterApply::Even => 1,
                                        HeaderFooterApply::Odd => 2,
                                    },
                                    Control::Footer(f) => match f.apply_to {
                                        HeaderFooterApply::Both => 0,
                                        HeaderFooterApply::Even => 1,
                                        HeaderFooterApply::Odd => 2,
                                    },
                                    _ => 0,
                                };
                                return Some((source_sec, apply_to));
                            }
                        }
                    }
                }
                return None;
            }
            offset += count;
        }
        None
    }

    /// 머리말/꼬리말 내부 텍스트 히트테스트
    ///
    /// 편집 모드에서 클릭한 좌표가 어느 문단·문자 위치에 해당하는지 반환.
    /// 반환: JSON `{"hit":true,"paraIndex":N,"charOffset":N,"cursorRect":{...}}`
    /// 또는 `{"hit":false}`
    pub fn hit_test_in_header_footer_native(
        &self,
        page_num: u32,
        is_header: bool,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree(page_num)?;

        // Header/Footer 서브트리 찾기
        let hf_node = tree.root.children.iter().find(|child| {
            if is_header {
                matches!(child.node_type, RenderNodeType::Header)
            } else {
                matches!(child.node_type, RenderNodeType::Footer)
            }
        });
        let hf_node = match hf_node {
            Some(n) => n,
            None => return Ok("{\"hit\":false}".to_string()),
        };

        // TextRun 정보 수집
        struct HfRunInfo {
            hf_para_idx: usize, // 머리말/꼬리말 내 문단 인덱스 (0, 1, 2, ...)
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            baseline: f64,
            font_size: f64,
        }

        fn collect_hf_runs(node: &RenderNode, runs: &mut Vec<HfRunInfo>) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let (Some(marker_para), Some(cs)) = (text_run.para_index, text_run.char_start) {
                    // marker_para = usize::MAX - hf_para_idx → 복원
                    if marker_para >= (usize::MAX - 1000) {
                        let hf_para_idx = usize::MAX - marker_para;
                        let positions = compute_char_positions(&text_run.text, &text_run.style);
                        runs.push(HfRunInfo {
                            hf_para_idx,
                            char_start: cs,
                            char_count: text_run.text.chars().count(),
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_w: node.bbox.width,
                            bbox_h: node.bbox.height,
                            baseline: text_run.baseline,
                            font_size: text_run.style.font_size,
                        });
                    }
                }
            }
            for child in &node.children {
                collect_hf_runs(child, runs);
            }
        }

        let mut runs: Vec<HfRunInfo> = Vec::new();
        collect_hf_runs(hf_node, &mut runs);

        if runs.is_empty() {
            // TextRun이 없는 경우 — 빈 머리말/꼬리말
            return Ok(format!(
                "{{\"hit\":true,\"paraIndex\":0,\"charOffset\":0,\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                page_num, hf_node.bbox.x, hf_node.bbox.y, 12.0
            ));
        }

        // hitTest용 헬퍼: char_positions에서 x 좌표로 문자 오프셋 찾기
        fn find_char_at_x_hf(positions: &[f64], local_x: f64) -> usize {
            for (i, &px) in positions.iter().enumerate() {
                if i == 0 {
                    if local_x < px / 2.0 {
                        return 0;
                    }
                } else {
                    let mid = (positions[i - 1] + px) / 2.0;
                    if local_x < mid {
                        return i;
                    }
                }
            }
            positions.len()
        }

        fn format_hf_hit(run: &HfRunInfo, char_offset: usize, page_num: u32) -> String {
            let cursor_x = if char_offset <= run.char_start {
                run.bbox_x
            } else {
                let local_idx = char_offset - run.char_start;
                if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                }
            };
            let ascent = run.font_size * 0.8;
            let cursor_y = run.bbox_y + run.baseline - ascent;
            format!(
                "{{\"hit\":true,\"paraIndex\":{},\"charOffset\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                run.hf_para_idx, char_offset, page_num, cursor_x, cursor_y, run.font_size
            )
        }

        // 1단계: 정확한 bbox 히트
        for run in &runs {
            if x >= run.bbox_x
                && x <= run.bbox_x + run.bbox_w
                && y >= run.bbox_y
                && y <= run.bbox_y + run.bbox_h
            {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x_hf(&run.char_positions, local_x);
                return Ok(format_hf_hit(run, run.char_start + char_offset, page_num));
            }
        }

        // 2단계: 같은 줄(y 범위)에서 가장 가까운 run
        let same_line: Vec<&HfRunInfo> = runs
            .iter()
            .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
            .collect();
        if !same_line.is_empty() {
            if x < same_line[0].bbox_x {
                let run = same_line[0];
                return Ok(format_hf_hit(run, run.char_start, page_num));
            }
            let last = same_line.last().unwrap();
            return Ok(format_hf_hit(
                last,
                last.char_start + last.char_count,
                page_num,
            ));
        }

        // 3단계: 가장 가까운 줄
        let closest = runs
            .iter()
            .min_by(|a, b| {
                let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
                let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let target_y = closest.bbox_y;
        let target_h = closest.bbox_h;
        let mut line_runs: Vec<&HfRunInfo> = runs
            .iter()
            .filter(|r| (r.bbox_y - target_y).abs() < 1.0 && (r.bbox_h - target_h).abs() < 1.0)
            .collect();
        line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());

        if x < line_runs[0].bbox_x {
            let run = line_runs[0];
            return Ok(format_hf_hit(run, run.char_start, page_num));
        }

        for run in &line_runs {
            if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x_hf(&run.char_positions, local_x);
                return Ok(format_hf_hit(run, run.char_start + char_offset, page_num));
            }
        }

        let last = line_runs.last().unwrap();
        Ok(format_hf_hit(
            last,
            last.char_start + last.char_count,
            page_num,
        ))
    }

    /// 각주 영역 히트테스트
    ///
    /// 페이지 좌표가 각주 영역에 해당하는지 판별.
    /// 반환: JSON `{"hit":true,"footnoteIndex":N}` 또는 `{"hit":false}`
    pub fn hit_test_footnote_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::RenderNodeType;

        let tree = self.build_page_tree(page_num)?;

        for child in &tree.root.children {
            if !matches!(child.node_type, RenderNodeType::FootnoteArea) {
                continue;
            }

            if x >= child.bbox.x
                && x <= child.bbox.x + child.bbox.width
                && y >= child.bbox.y
                && y <= child.bbox.y + child.bbox.height
            {
                // FootnoteArea 내에서 가장 가까운 TextRun의 footnote_index 반환
                let mut fn_idx = 0usize;
                fn find_fn_idx(node: &crate::renderer::render_tree::RenderNode, best: &mut usize) {
                    if let RenderNodeType::TextRun(ref tr) = node.node_type {
                        if let Some(pi) = tr.para_index {
                            if pi >= (usize::MAX - 3000) {
                                if let Some(si) = tr.section_index {
                                    *best = si;
                                }
                            }
                        }
                    }
                    for c in &node.children {
                        find_fn_idx(c, best);
                    }
                }
                find_fn_idx(child, &mut fn_idx);
                return Ok(format!("{{\"hit\":true,\"footnoteIndex\":{}}}", fn_idx));
            }
        }

        Ok("{\"hit\":false}".to_string())
    }

    /// 각주 내부 텍스트 히트테스트
    ///
    /// 편집 모드에서 클릭한 좌표의 각주 내 문단·문자 위치를 반환.
    /// 반환: JSON `{"hit":true,"fnParaIndex":N,"charOffset":N,"footnoteIndex":N,"cursorRect":{...}}`
    pub fn hit_test_in_footnote_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree(page_num)?;

        let fn_node = tree
            .root
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::FootnoteArea));
        let fn_node = match fn_node {
            Some(n) => n,
            None => return Ok("{\"hit\":false}".to_string()),
        };

        // 각주 TextRun 정보 수집
        struct FnRunInfo {
            footnote_index: usize,
            fn_para_idx: usize,
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            baseline: f64,
            font_size: f64,
        }

        // 번호 TextRun(char_start: None) 정보 — 빈 각주의 footnoteIndex/위치 결정용
        struct FnNumberInfo {
            footnote_index: usize,
            fn_para_idx: usize,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            font_size: f64,
            baseline: f64,
        }

        fn collect_fn_runs(
            node: &RenderNode,
            runs: &mut Vec<FnRunInfo>,
            number_runs: &mut Vec<FnNumberInfo>,
        ) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let (Some(marker_para), Some(marker_section)) =
                    (text_run.para_index, text_run.section_index)
                {
                    if marker_para >= (usize::MAX - 3000) && marker_para < (usize::MAX - 1000) {
                        let fn_para_idx = usize::MAX - 2000 - marker_para;
                        if let Some(cs) = text_run.char_start {
                            // 본문 텍스트 TextRun
                            let positions = compute_char_positions(&text_run.text, &text_run.style);
                            runs.push(FnRunInfo {
                                footnote_index: marker_section,
                                fn_para_idx,
                                char_start: cs,
                                char_count: text_run.text.chars().count(),
                                char_positions: positions,
                                bbox_x: node.bbox.x,
                                bbox_y: node.bbox.y,
                                bbox_w: node.bbox.width,
                                bbox_h: node.bbox.height,
                                baseline: text_run.baseline,
                                font_size: text_run.style.font_size,
                            });
                        } else {
                            // 번호 TextRun (char_start: None)
                            number_runs.push(FnNumberInfo {
                                footnote_index: marker_section,
                                fn_para_idx,
                                bbox_x: node.bbox.x,
                                bbox_y: node.bbox.y,
                                bbox_w: node.bbox.width,
                                bbox_h: node.bbox.height,
                                font_size: text_run.style.font_size,
                                baseline: text_run.baseline,
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                collect_fn_runs(child, runs, number_runs);
            }
        }

        let mut runs: Vec<FnRunInfo> = Vec::new();
        let mut number_runs: Vec<FnNumberInfo> = Vec::new();
        collect_fn_runs(fn_node, &mut runs, &mut number_runs);

        // Y 좌표로 가장 가까운 각주의 footnoteIndex 결정 (텍스트 run이 없는 빈 각주 지원)
        if runs.is_empty()
            || !runs
                .iter()
                .any(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
        {
            // 번호 run에서 Y 좌표로 가장 가까운 각주 찾기
            let closest_num = number_runs.iter().min_by(|a, b| {
                let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
                let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(nr) = closest_num {
                let ascent = nr.font_size * 0.8;
                let cursor_y = nr.bbox_y + nr.baseline - ascent;
                return Ok(format!(
                    "{{\"hit\":true,\"fnParaIndex\":{},\"charOffset\":0,\"footnoteIndex\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                    nr.fn_para_idx, nr.footnote_index, page_num,
                    nr.bbox_x + nr.bbox_w, cursor_y, nr.font_size
                ));
            }
            return Ok(format!(
                "{{\"hit\":true,\"fnParaIndex\":0,\"charOffset\":0,\"footnoteIndex\":0,\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                page_num, fn_node.bbox.x, fn_node.bbox.y, 12.0
            ));
        }

        fn find_char_at_x(positions: &[f64], local_x: f64) -> usize {
            for (i, &px) in positions.iter().enumerate() {
                if i == 0 {
                    if local_x < px / 2.0 {
                        return 0;
                    }
                } else {
                    let mid = (positions[i - 1] + px) / 2.0;
                    if local_x < mid {
                        return i;
                    }
                }
            }
            positions.len()
        }

        fn format_fn_hit(run: &FnRunInfo, char_offset: usize, page_num: u32) -> String {
            let cursor_x = if char_offset <= run.char_start {
                run.bbox_x
            } else {
                let local_idx = char_offset - run.char_start;
                if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                }
            };
            let ascent = run.font_size * 0.8;
            let cursor_y = run.bbox_y + run.baseline - ascent;
            format!(
                "{{\"hit\":true,\"fnParaIndex\":{},\"charOffset\":{},\"footnoteIndex\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                run.fn_para_idx, char_offset, run.footnote_index, page_num, cursor_x, cursor_y, run.font_size
            )
        }

        // 1단계: 정확한 bbox 히트
        for run in &runs {
            if x >= run.bbox_x
                && x <= run.bbox_x + run.bbox_w
                && y >= run.bbox_y
                && y <= run.bbox_y + run.bbox_h
            {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x(&run.char_positions, local_x);
                return Ok(format_fn_hit(run, run.char_start + char_offset, page_num));
            }
        }

        // 2단계: 같은 줄(y 범위)에서 가장 가까운 run
        let same_line: Vec<&FnRunInfo> = runs
            .iter()
            .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
            .collect();
        if !same_line.is_empty() {
            if x < same_line[0].bbox_x {
                let run = same_line[0];
                return Ok(format_fn_hit(run, run.char_start, page_num));
            }
            let last = same_line.last().unwrap();
            return Ok(format_fn_hit(
                last,
                last.char_start + last.char_count,
                page_num,
            ));
        }

        // 3단계: 가장 가까운 줄
        let closest = runs
            .iter()
            .min_by(|a, b| {
                let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
                let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let target_y = closest.bbox_y;
        let target_h = closest.bbox_h;
        let mut line_runs: Vec<&FnRunInfo> = runs
            .iter()
            .filter(|r| (r.bbox_y - target_y).abs() < 1.0 && (r.bbox_h - target_h).abs() < 1.0)
            .collect();
        line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());

        if x < line_runs[0].bbox_x {
            let run = line_runs[0];
            return Ok(format_fn_hit(run, run.char_start, page_num));
        }

        let last = line_runs.last().unwrap();
        Ok(format_fn_hit(
            last,
            last.char_start + last.char_count,
            page_num,
        ))
    }

    /// 각주/미주 내부 선택 영역의 줄별 사각형을 계산한다.
    pub fn get_selection_rects_in_footnote_native(
        &self,
        page_num: u32,
        footnote_index: usize,
        start_fn_para_idx: usize,
        start_char_offset: usize,
        end_fn_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree(page_num)?;
        let Some(fn_node) = tree
            .root
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::FootnoteArea))
        else {
            return Ok("[]".to_string());
        };

        #[derive(Clone)]
        struct FnRunInfo {
            fn_para_idx: usize,
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
        }

        fn collect_runs(node: &RenderNode, footnote_index: usize, runs: &mut Vec<FnRunInfo>) {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if tr.section_index == Some(footnote_index) {
                    if let (Some(marker_para), Some(cs)) = (tr.para_index, tr.char_start) {
                        if marker_para >= (usize::MAX - 3000) && marker_para < (usize::MAX - 1000) {
                            let fn_para_idx = usize::MAX - 2000 - marker_para;
                            runs.push(FnRunInfo {
                                fn_para_idx,
                                char_start: cs,
                                char_count: tr.text.chars().count(),
                                char_positions: compute_char_positions(&tr.text, &tr.style),
                                bbox_x: node.bbox.x,
                                bbox_y: node.bbox.y,
                                bbox_w: node.bbox.width,
                                bbox_h: node.bbox.height,
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                collect_runs(child, footnote_index, runs);
            }
        }

        fn cmp_pos(a_para: usize, a_off: usize, b_para: usize, b_off: usize) -> std::cmp::Ordering {
            a_para.cmp(&b_para).then_with(|| a_off.cmp(&b_off))
        }

        fn x_at(run: &FnRunInfo, char_offset: usize) -> f64 {
            if char_offset <= run.char_start {
                return run.bbox_x;
            }
            let local_idx = char_offset - run.char_start;
            if local_idx < run.char_positions.len() {
                run.bbox_x + run.char_positions[local_idx]
            } else {
                run.bbox_x + run.bbox_w
            }
        }

        let mut runs = Vec::new();
        collect_runs(fn_node, footnote_index, &mut runs);
        runs.sort_by(|a, b| {
            a.fn_para_idx
                .cmp(&b.fn_para_idx)
                .then_with(|| {
                    a.bbox_y
                        .partial_cmp(&b.bbox_y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    a.bbox_x
                        .partial_cmp(&b.bbox_x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut rects = Vec::new();
        for run in runs {
            let run_start = (run.fn_para_idx, run.char_start);
            let run_end = (run.fn_para_idx, run.char_start + run.char_count);

            if cmp_pos(run_end.0, run_end.1, start_fn_para_idx, start_char_offset)
                != std::cmp::Ordering::Greater
                || cmp_pos(run_start.0, run_start.1, end_fn_para_idx, end_char_offset)
                    != std::cmp::Ordering::Less
            {
                continue;
            }

            let sel_start = if run.fn_para_idx == start_fn_para_idx {
                start_char_offset.max(run.char_start)
            } else {
                run.char_start
            };
            let sel_end = if run.fn_para_idx == end_fn_para_idx {
                end_char_offset.min(run.char_start + run.char_count)
            } else {
                run.char_start + run.char_count
            };

            if sel_end <= sel_start {
                continue;
            }

            let x1 = x_at(&run, sel_start);
            let x2 = x_at(&run, sel_end);
            let width = (x2 - x1).max(1.0);
            rects.push(format!(
                "{{\"pageIndex\":{},\"x\":{:.2},\"y\":{:.2},\"width\":{:.2},\"height\":{:.2}}}",
                page_num, x1, run.bbox_y, width, run.bbox_h
            ));
        }

        Ok(format!("[{}]", rects.join(",")))
    }

    fn global_page_base_for_section(&self, section_idx: usize) -> u32 {
        self.pagination
            .iter()
            .take(section_idx)
            .map(|pr| pr.pages.len() as u32)
            .sum()
    }

    fn cursor_rect_for_render_paragraph(
        &self,
        page_num: u32,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<Option<String>, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        struct CursorRun {
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            baseline: f64,
            font_size: f64,
        }

        fn collect_runs(
            node: &RenderNode,
            section_idx: usize,
            para_idx: usize,
            runs: &mut Vec<CursorRun>,
        ) {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if tr.section_index == Some(section_idx) && tr.para_index == Some(para_idx) {
                    if let Some(cs) = tr.char_start {
                        let positions = compute_char_positions(&tr.text, &tr.style);
                        runs.push(CursorRun {
                            char_start: cs,
                            char_count: effective_char_count(tr),
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            baseline: tr.baseline,
                            font_size: tr.style.font_size,
                        });
                    }
                }
            }
            for child in &node.children {
                collect_runs(child, section_idx, para_idx, runs);
            }
        }

        let tree = self.build_page_tree(page_num)?;
        let mut runs = Vec::new();
        collect_runs(&tree.root, section_idx, para_idx, &mut runs);
        runs.sort_by(|a, b| {
            a.char_start
                .cmp(&b.char_start)
                .then_with(|| {
                    a.bbox_y
                        .partial_cmp(&b.bbox_y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    a.bbox_x
                        .partial_cmp(&b.bbox_x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        if runs.is_empty() {
            return Ok(None);
        }

        for run in &runs {
            if char_offset >= run.char_start && char_offset <= run.char_start + run.char_count {
                let local_idx = char_offset - run.char_start;
                let cursor_x = if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                };
                let ascent = run.font_size * 0.8;
                let cursor_y = run.bbox_y + run.baseline - ascent;
                return Ok(Some(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    page_num, cursor_x, cursor_y, run.font_size
                )));
            }
        }

        let last = runs.last().unwrap();
        let cursor_x = if !last.char_positions.is_empty() {
            last.bbox_x + last.char_positions.last().copied().unwrap_or(0.0)
        } else {
            last.bbox_x
        };
        let ascent = last.font_size * 0.8;
        let cursor_y = last.bbox_y + last.baseline - ascent;
        Ok(Some(format!(
            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
            page_num, cursor_x, cursor_y, last.font_size
        )))
    }

    fn find_body_footnote_edit_target(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Option<(u32, usize)> {
        use crate::renderer::pagination::FootnoteSource;

        let pr = self.pagination.get(section_idx)?;
        let page_base = self.global_page_base_for_section(section_idx);
        for (local_page_idx, page) in pr.pages.iter().enumerate() {
            if let Some(footnote_idx) = page.footnotes.iter().position(|fn_ref| {
                matches!(
                    &fn_ref.source,
                    FootnoteSource::Body { para_index, control_index }
                        if *para_index == para_idx && *control_index == control_idx
                )
            }) {
                return Some((page_base + local_page_idx as u32, footnote_idx));
            }
        }
        None
    }

    fn find_endnote_edit_target(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        note_para_idx: usize,
    ) -> Option<(u32, usize, usize)> {
        let pr = self.pagination.get(section_idx)?;
        let local_idx = pr.endnote_para_sources.iter().position(|src| {
            src.section_index == section_idx
                && src.para_index == para_idx
                && src.control_index == control_idx
                && src.note_para_index == note_para_idx
        })?;
        let virtual_para_idx =
            self.document.sections.get(section_idx)?.paragraphs.len() + local_idx;
        let page_base = self.global_page_base_for_section(section_idx);
        for (local_page_idx, page) in pr.pages.iter().enumerate() {
            if page
                .column_contents
                .iter()
                .flat_map(|col| col.items.iter())
                .any(|item| item.para_index() == virtual_para_idx)
            {
                return Some((
                    page_base + local_page_idx as u32,
                    local_idx,
                    virtual_para_idx,
                ));
            }
        }
        None
    }

    pub fn get_note_edit_info_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)))?;
        let ctrl = para.controls.get(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;

        match ctrl {
            Control::Footnote(_) => {
                let (page_num, footnote_index) = self
                    .find_body_footnote_edit_target(section_idx, para_idx, control_idx)
                    .ok_or_else(|| {
                        HwpError::RenderError("각주 렌더 위치를 찾을 수 없습니다".to_string())
                    })?;
                Ok(format!(
                    "{{\"ok\":true,\"kind\":\"footnote\",\"pageNum\":{},\"footnoteIndex\":{},\"fnParaIndex\":0,\"charOffset\":2}}",
                    page_num, footnote_index
                ))
            }
            Control::Endnote(_) => {
                let (page_num, endnote_index, virtual_para_idx) = self
                    .find_endnote_edit_target(section_idx, para_idx, control_idx, 0)
                    .ok_or_else(|| {
                        HwpError::RenderError("미주 렌더 위치를 찾을 수 없습니다".to_string())
                    })?;
                Ok(format!(
                    "{{\"ok\":true,\"kind\":\"endnote\",\"pageNum\":{},\"footnoteIndex\":{},\"fnParaIndex\":0,\"charOffset\":2,\"virtualParaIndex\":{}}}",
                    page_num, endnote_index, virtual_para_idx
                ))
            }
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 각주/미주가 아닙니다".to_string(),
            )),
        }
    }

    pub fn get_cursor_rect_in_note_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        note_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)))?;
        let ctrl = para.controls.get(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;

        match ctrl {
            Control::Footnote(_) => {
                let (page_num, footnote_index) = self
                    .find_body_footnote_edit_target(section_idx, para_idx, control_idx)
                    .ok_or_else(|| {
                        HwpError::RenderError("각주 렌더 위치를 찾을 수 없습니다".to_string())
                    })?;
                self.get_cursor_rect_in_footnote_native(
                    page_num,
                    footnote_index,
                    note_para_idx,
                    char_offset,
                )
            }
            Control::Endnote(endnote) => {
                let (page_num, _, virtual_para_idx) = self
                    .find_endnote_edit_target(section_idx, para_idx, control_idx, note_para_idx)
                    .ok_or_else(|| {
                        HwpError::RenderError("미주 렌더 위치를 찾을 수 없습니다".to_string())
                    })?;
                let render_char_offset = if note_para_idx == 0 {
                    char_offset
                        + note_marker_text(
                            endnote.number,
                            endnote.number_shape,
                            endnote.before_decoration_letter,
                            endnote.after_decoration_letter,
                        )
                        .chars()
                        .count()
                        + 1
                } else {
                    char_offset
                };
                self.cursor_rect_for_render_paragraph(
                    page_num,
                    section_idx,
                    virtual_para_idx,
                    render_char_offset,
                )?
                .ok_or_else(|| {
                    HwpError::RenderError("미주 커서 위치를 찾을 수 없습니다".to_string())
                })
            }
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 각주/미주가 아닙니다".to_string(),
            )),
        }
    }

    /// 각주 내 커서 위치 (커서 렉트) 계산
    ///
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    pub fn get_cursor_rect_in_footnote_native(
        &self,
        page_num: u32,
        footnote_index: usize,
        fn_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree(page_num)?;

        let fn_node = tree
            .root
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::FootnoteArea));
        let fn_node = match fn_node {
            Some(n) => n,
            None => {
                return Err(HwpError::RenderError(
                    "각주 영역을 찾을 수 없습니다".to_string(),
                ))
            }
        };

        let marker_para = usize::MAX - 2000 - fn_para_idx;

        // 해당 각주/문단의 TextRun 찾기
        struct FnCursorRun {
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_h: f64,
            baseline: f64,
            font_size: f64,
        }

        fn collect_cursor_runs(
            node: &RenderNode,
            target_section: usize,
            target_para: usize,
            runs: &mut Vec<FnCursorRun>,
        ) {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if tr.section_index == Some(target_section) && tr.para_index == Some(target_para) {
                    if let Some(cs) = tr.char_start {
                        let positions = compute_char_positions(&tr.text, &tr.style);
                        runs.push(FnCursorRun {
                            char_start: cs,
                            char_count: tr.text.chars().count(),
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_h: node.bbox.height,
                            baseline: tr.baseline,
                            font_size: tr.style.font_size,
                        });
                    }
                }
            }
            for c in &node.children {
                collect_cursor_runs(c, target_section, target_para, runs);
            }
        }

        let mut runs: Vec<FnCursorRun> = Vec::new();
        collect_cursor_runs(fn_node, footnote_index, marker_para, &mut runs);

        if runs.is_empty() {
            // 폴백: 번호 TextRun(char_start=None) 뒤의 위치를 찾기
            // 번호 run은 section_index=footnote_index, para_index=marker_para, char_start=None
            fn find_number_run_end(
                node: &RenderNode,
                target_sec: usize,
                target_para: usize,
            ) -> Option<(f64, f64, f64)> {
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    if tr.section_index == Some(target_sec)
                        && tr.para_index == Some(target_para)
                        && tr.char_start.is_none()
                    {
                        // 번호 run의 오른쪽 끝
                        return Some((
                            node.bbox.x + node.bbox.width,
                            node.bbox.y + tr.baseline - tr.style.font_size * 0.8,
                            tr.style.font_size,
                        ));
                    }
                }
                for c in &node.children {
                    if let Some(r) = find_number_run_end(c, target_sec, target_para) {
                        return Some(r);
                    }
                }
                None
            }
            if let Some((x, y, h)) = find_number_run_end(fn_node, footnote_index, marker_para) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    page_num, x, y, h
                ));
            }
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                page_num, fn_node.bbox.x, fn_node.bbox.y, 12.0
            ));
        }

        // char_offset에 해당하는 run 찾기
        for run in &runs {
            if char_offset >= run.char_start && char_offset <= run.char_start + run.char_count {
                let local_idx = char_offset - run.char_start;
                let cursor_x = if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                };
                let ascent = run.font_size * 0.8;
                let cursor_y = run.bbox_y + run.baseline - ascent;
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    page_num, cursor_x, cursor_y, run.font_size
                ));
            }
        }

        // 마지막 run의 끝
        let last = runs.last().unwrap();
        let cursor_x = if !last.char_positions.is_empty() {
            last.bbox_x + last.char_positions.last().copied().unwrap_or(0.0)
        } else {
            last.bbox_x
        };
        let ascent = last.font_size * 0.8;
        let cursor_y = last.bbox_y + last.baseline - ascent;
        Ok(format!(
            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
            page_num, cursor_x, cursor_y, last.font_size
        ))
    }

    fn find_page_body_footnote_index(
        &self,
        page_num: u32,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Option<usize> {
        use crate::renderer::pagination::FootnoteSource;

        let (page_section_idx, local_page) = self.find_section_for_page(page_num);
        if page_section_idx != section_idx {
            return None;
        }

        let pr = self.pagination.get(section_idx)?;
        let page = pr.pages.get(local_page)?;
        page.footnotes.iter().position(|fn_ref| {
            matches!(
                &fn_ref.source,
                FootnoteSource::Body { para_index, control_index }
                    if *para_index == para_idx && *control_index == control_idx
            )
        })
    }

    /// 본문 인라인 각주 마커 히트테스트
    ///
    /// 각주 영역(zone)이 아니라 본문 TextLine 안의 FootnoteMarker bbox를 대상으로 한다.
    /// 반환: JSON `{"hit":true,...}` 또는 `{"hit":false}`
    pub fn hit_test_body_footnote_marker_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        struct MarkerHit {
            section_index: usize,
            paragraph_index: usize,
            control_index: usize,
            footnote_number: u16,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
        }

        fn find_marker(node: &RenderNode, x: f64, y: f64) -> Option<MarkerHit> {
            if let RenderNodeType::FootnoteMarker(ref marker) = node.node_type {
                if x >= node.bbox.x
                    && x <= node.bbox.x + node.bbox.width
                    && y >= node.bbox.y
                    && y <= node.bbox.y + node.bbox.height
                {
                    return Some(MarkerHit {
                        section_index: marker.section_index,
                        paragraph_index: marker.para_index,
                        control_index: marker.control_index,
                        footnote_number: marker.number,
                        bbox_x: node.bbox.x,
                        bbox_y: node.bbox.y,
                        bbox_w: node.bbox.width,
                        bbox_h: node.bbox.height,
                    });
                }
            }

            for child in &node.children {
                if let Some(hit) = find_marker(child, x, y) {
                    return Some(hit);
                }
            }

            None
        }

        let tree = self.build_page_tree_cached(page_num)?;
        let hit = match find_marker(&tree.root, x, y) {
            Some(hit) => hit,
            None => return Ok("{\"hit\":false}".to_string()),
        };

        let footnote_index = match self.find_page_body_footnote_index(
            page_num,
            hit.section_index,
            hit.paragraph_index,
            hit.control_index,
        ) {
            Some(idx) => idx,
            None => return Ok("{\"hit\":false}".to_string()),
        };

        Ok(format!(
            "{{\"hit\":true,\"sectionIndex\":{},\"paragraphIndex\":{},\"controlIndex\":{},\"footnoteNumber\":{},\"footnoteIndex\":{},\"bbox\":{{\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
            hit.section_index,
            hit.paragraph_index,
            hit.control_index,
            hit.footnote_number,
            footnote_index,
            hit.bbox_x,
            hit.bbox_y,
            hit.bbox_w,
            hit.bbox_h,
            page_num,
            hit.bbox_x + hit.bbox_w,
            hit.bbox_y,
            hit.bbox_h
        ))
    }

    /// 페이지의 각주 참조 정보를 반환한다.
    ///
    /// footnoteIndex에 해당하는 FootnoteRef의 source(para_index, control_index)를 반환.
    /// 반환: JSON `{"ok":true,"sectionIdx":N,"paraIdx":N,"controlIdx":N}`
    pub fn get_page_footnote_info_native(
        &self,
        page_num: u32,
        footnote_index: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::pagination::FootnoteSource;

        let (section_idx, local_page) = self.find_section_for_page(page_num);
        let pr = self
            .pagination
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역을 찾을 수 없습니다".to_string()))?;
        let page = pr
            .pages
            .get(local_page)
            .ok_or_else(|| HwpError::RenderError("페이지를 찾을 수 없습니다".to_string()))?;

        let fn_ref = page.footnotes.get(footnote_index).ok_or_else(|| {
            HwpError::RenderError(format!(
                "각주 인덱스 {} 범위 초과 (총 {}개)",
                footnote_index,
                page.footnotes.len()
            ))
        })?;

        let (para_idx, control_idx, source_type) = match &fn_ref.source {
            FootnoteSource::Body {
                para_index,
                control_index,
            } => (*para_index, *control_index, "body"),
            FootnoteSource::TableCell {
                para_index,
                table_control_index,
                ..
            } => (*para_index, *table_control_index, "table"),
            FootnoteSource::ShapeTextBox {
                para_index,
                shape_control_index,
                ..
            } => (*para_index, *shape_control_index, "shape"),
        };

        Ok(format!(
            "{{\"ok\":true,\"sectionIdx\":{},\"paraIdx\":{},\"controlIdx\":{},\"sourceType\":\"{}\"}}",
            section_idx, para_idx, control_idx, source_type
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_x_on_line, LineRunView};

    // 줄 단위 x 해석(`resolve_x_on_line`)의 회귀 테스트.
    //
    // 핵심은 "줄 시작/끝으로 스냅하지 않는다" 이다. char_positions 는 production
    // 의 `compute_char_positions` 와 동일하게 `[0.0, w1, w1+w2, ...]` (char_count+1
    // 개, 0.0 포함) 형태로 구성한다. 줄 안 글자 경계로의 정확한 라운딩은
    // production `find_char_at_x` 의 미드포인트 규칙을 그대로 따르므로, 여기서는
    // "줄 시작/끝 상수로 붕괴하지 않고 x 에 따라 단조 증가" 라는 회귀 핵심 속성을
    // 검증한다.

    /// char_count 개 글자가 균등 폭 `w` 로 놓인 한 run.
    fn run(
        bbox_x: f64,
        char_start: usize,
        char_count: usize,
        w: f64,
        positions: &[f64],
    ) -> LineRunView<'_> {
        LineRunView {
            bbox_x,
            bbox_w: w * char_count as f64,
            char_start,
            char_count,
            char_positions: positions,
        }
    }

    /// 10글자 run 하나로 된 줄. bbox_x=100, 글자 폭 10px. char_start=7 (줄 시작이
    /// 문단 offset 0 이 아님을 확인).
    const POS10: [f64; 11] = [
        0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0,
    ];

    /// 회귀: 줄 한가운데 x 의 클릭은 줄 시작도 끝도 아닌 중간 글자 offset 으로
    /// 해석되어야 한다. (leading-gap 클릭이 줄 시작/끝으로 스냅하던 버그)
    ///
    /// `resolve_x_on_line` 은 클릭 y 와 무관하게 줄 안에서 x 만으로 해석하므로,
    /// 글리프 bbox 의 행간 여백(leading gap)에 떨어진 클릭도 동일하게 이 경로를
    /// 타게 되어 더 이상 줄 시작/끝으로 스냅하지 않는다.
    #[test]
    fn mid_line_x_resolves_to_mid_line_offset_not_start_or_end() {
        let line = vec![run(100.0, 7, 10, 10.0, &POS10)];
        let line_start = 7; // char_start
        let line_end = 17; // char_start + char_count

        // 줄 한가운데(x=150)
        let (idx, offset) = resolve_x_on_line(&line, 150.0);
        assert_eq!(idx, 0, "단일 run 줄이므로 run 인덱스는 0");
        assert!(
            offset > line_start && offset < line_end,
            "회귀: 줄 한가운데(x=150) 클릭이 줄 시작({line_start})/끝({line_end}) 사이의 \
             중간 글자로 해석되어야 함, got {offset}"
        );
    }

    /// 줄 전체에 x 를 쓸어가며 해석한 offset 이 (약)단조 증가하고, 시작/끝 상수로
    /// 붕괴하지 않아야 한다. (어떤 내부 x 도 줄 시작/끝으로 스냅하지 않음을 확인)
    #[test]
    fn interior_x_sweep_is_monotonic_and_spans_the_line() {
        let line = vec![run(100.0, 7, 10, 10.0, &POS10)];
        let mut prev = 0usize;
        let mut distinct = std::collections::BTreeSet::new();
        let mut x = 100.0; // bbox_x
        while x <= 200.0 {
            let (_, offset) = resolve_x_on_line(&line, x);
            assert!(
                offset >= prev,
                "x={x} 에서 offset={offset} 이 직전 {prev} 보다 작음 (단조 증가 위반)"
            );
            distinct.insert(offset);
            prev = offset;
            x += 2.0;
        }
        assert!(
            distinct.len() >= 8,
            "회귀: 줄 내부 x 스윕이 {} 개의 distinct offset 만 냄 {distinct:?}; \
             줄 시작/끝 상수로 스냅하면 1~2 개로 붕괴함",
            distinct.len()
        );
    }

    /// 줄 경계 밖: 왼쪽은 줄 시작, 오른쪽은 줄 끝으로 클램프.
    #[test]
    fn outside_line_clamps_to_start_and_end() {
        let line = vec![run(100.0, 7, 10, 10.0, &POS10)];
        assert_eq!(resolve_x_on_line(&line, 50.0), (0, 7), "줄 왼쪽 → 줄 시작");
        assert_eq!(
            resolve_x_on_line(&line, 999.0),
            (0, 17),
            "줄 오른쪽 → 줄 끝"
        );
    }

    /// 다중 run 줄: 두 run 사이의 빈틈 클릭은 줄 끝으로 스냅하지 않고
    /// 더 가까운 run 경계로 해석되어야 한다. (다중 run 줄 inter-run-gap 버그)
    #[test]
    fn inter_run_gap_snaps_to_nearer_boundary_not_line_end() {
        const P0: [f64; 4] = [0.0, 10.0, 20.0, 30.0]; // 3 chars, 폭 10
        const P1: [f64; 4] = [0.0, 10.0, 20.0, 30.0]; // 3 chars, 폭 10
                                                      // run0: x[100,130], chars 0..3 ; 빈틈 ; run1: x[200,230], chars 5..8
        let line = vec![run(100.0, 0, 3, 10.0, &P0), run(200.0, 5, 3, 10.0, &P1)];
        let line_end = 5 + 3; // 8

        // 빈틈 안에서 왼쪽 run 에 가까운 x(140) → 왼쪽 run 끝 (offset 3)
        let (idx, offset) = resolve_x_on_line(&line, 140.0);
        assert_eq!((idx, offset), (0, 3), "빈틈 왼쪽 → 왼쪽 run 끝");
        assert_ne!(
            offset, line_end,
            "회귀: 빈틈 클릭이 줄 끝으로 스냅하면 안 됨"
        );

        // 빈틈 안에서 오른쪽 run 에 가까운 x(190) → 오른쪽 run 시작 (offset 5)
        let (idx, offset) = resolve_x_on_line(&line, 190.0);
        assert_eq!((idx, offset), (1, 5), "빈틈 오른쪽 → 오른쪽 run 시작");
    }

    /// 회귀: 빈 입력칸(char_count=0)이지만 bbox_w 가 셀 폭만큼 넓은 run.
    /// char_positions = [0.0] 한 개뿐이라 라운딩 함수가 len()=1 을 돌려주지만,
    /// 글자 인덱스는 char_count(=0) 로 클램프되어 offset 은 항상 char_start 여야 한다.
    /// (exam_social/exam_science 답안지 `성명` 빈 입력칸 클릭이 offset 1 로 새던 버그)
    #[test]
    fn empty_run_with_wide_bbox_clamps_to_char_start() {
        const EMPTY: [f64; 1] = [0.0];
        let line = vec![LineRunView {
            bbox_x: 212.7,
            bbox_w: 97.0, // 빈 입력칸 폭(글자 없음)
            char_start: 0,
            char_count: 0,
            char_positions: &EMPTY,
        }];
        // bbox 한참 안쪽 클릭(x=250)도 빈 run 이므로 offset 0 (줄/run 시작).
        assert_eq!(resolve_x_on_line(&line, 250.0), (0, 0));
        // 오른쪽 끝 너머도 마찬가지.
        assert_eq!(resolve_x_on_line(&line, 999.0), (0, 0));
    }

    /// [#2021] 계측 프로브 — 대형 표 문서의 콜드 rect 비용 분해.
    /// 실행: cargo test --profile release-test --lib document_core::queries::cursor_rect -- --ignored --nocapture
    #[test]
    #[ignore = "계측 프로브 — 수동 실행"]
    fn probe_2021_cursor_rect_by_path_cold_cost() {
        use std::time::Instant;
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
        let bytes = std::fs::read(&p).expect("read sample");
        let core = crate::document_core::DocumentCore::from_bytes(&bytes).expect("parse");

        let doc = core.document();
        let mut host = None;
        'outer: for (pi, para) in doc.sections[0].paragraphs.iter().enumerate() {
            for (ci, c) in para.controls.iter().enumerate() {
                if matches!(c, crate::model::control::Control::Table(_)) {
                    host = Some((pi, ci));
                    break 'outer;
                }
            }
        }
        let (host_pi, host_ci) = host.expect("표 없음");
        let path_json = format!(
            r#"[{{"controlIndex":{},"cellIndex":0,"cellParaIndex":0}}]"#,
            host_ci
        );

        let total_pages = core.page_count();
        let pages = core.find_pages_for_paragraph(0, host_pi).expect("pages");
        eprintln!(
            "#2021 probe: 총 {}쪽 / 호스트 pi={} 걸친 페이지 = {}개 (first={:?} last={:?})",
            total_pages,
            host_pi,
            pages.len(),
            pages.first(),
            pages.last()
        );

        let t = Instant::now();
        let warm = core.get_cursor_rect_by_path_native(0, host_pi, &path_json, 0);
        eprintln!(
            "#2021 probe: 웜 ok={} ({:.1}ms)",
            warm.is_ok(),
            t.elapsed().as_secs_f64() * 1000.0
        );

        core.invalidate_page_tree_cache();
        let t = Instant::now();
        let cold = core.get_cursor_rect_by_path_native(0, host_pi, &path_json, 0);
        let cold_ms = t.elapsed().as_secs_f64() * 1000.0;
        eprintln!(
            "#2021 probe: 콜드 ok={} ({:.1}ms) rect={:?}",
            cold.is_ok(),
            cold_ms,
            cold.as_deref().unwrap_or("-")
        );

        // 셀 내부 깊은 문단 — 실제 시나리오(후반 페이지에 렌더되는 셀 문단) 재현
        if let Some(crate::model::control::Control::Table(t)) =
            doc.sections[0].paragraphs[host_pi].controls.get(host_ci)
        {
            let n_paras = t.cells.first().map(|c| c.paragraphs.len()).unwrap_or(0);
            for frac in [2usize, 4, 10] {
                let cpi = n_paras.saturating_sub(1) / frac * (frac - 1);
                let path_deep = format!(
                    r#"[{{"controlIndex":{},"cellIndex":0,"cellParaIndex":{}}}]"#,
                    host_ci, cpi
                );
                core.invalidate_page_tree_cache();
                let t2 = Instant::now();
                let r = core.get_cursor_rect_by_path_native(0, host_pi, &path_deep, 0);
                eprintln!(
                    "#2021 probe: 셀0 문단 {}/{} 콜드 ok={} ({:.1}ms) rect={:?}",
                    cpi,
                    n_paras,
                    r.is_ok(),
                    t2.elapsed().as_secs_f64() * 1000.0,
                    r.as_deref().unwrap_or("-")
                );
            }
        }

        // 뒤쪽 셀(마지막 셀) — 캐럿 페이지가 문서 후반일 때의 선형 탐색 비용
        if let Some(crate::model::control::Control::Table(t)) =
            doc.sections[0].paragraphs[host_pi].controls.get(host_ci)
        {
            let last_cell = t.cells.len().saturating_sub(1);
            let path_last = format!(
                r#"[{{"controlIndex":{},"cellIndex":{},"cellParaIndex":0}}]"#,
                host_ci, last_cell
            );
            core.invalidate_page_tree_cache();
            let t2 = Instant::now();
            let cold_last = core.get_cursor_rect_by_path_native(0, host_pi, &path_last, 0);
            eprintln!(
                "#2021 probe: 마지막 셀(idx {}) 콜드 ok={} ({:.1}ms) rect={:?}",
                last_cell,
                cold_last.is_ok(),
                t2.elapsed().as_secs_f64() * 1000.0,
                cold_last.as_deref().unwrap_or("-")
            );
        }

        if let Ok(ref json) = cold {
            let page: u32 = json
                .split("\"pageIndex\":")
                .nth(1)
                .and_then(|s| s.split(&[',', '}'][..]).next())
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            core.invalidate_page_tree_cache();
            let t = Instant::now();
            let _ = core.build_page_tree_cached(page);
            eprintln!(
                "#2021 probe: 캐럿 페이지({}) 1장 빌드 = {:.1}ms",
                page,
                t.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    /// [#2021] 힌트 탐색 동등성 핀 — 힌트 유/무·오힌트 fallback 모두 좌표 완전 일치.
    #[test]
    fn issue_2021_hint_search_equivalence() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
        let bytes = std::fs::read(&p).expect("read sample");
        let core = crate::document_core::DocumentCore::from_bytes(&bytes).expect("parse");
        let doc = core.document();
        let mut host = None;
        'outer: for (pi, para) in doc.sections[0].paragraphs.iter().enumerate() {
            for (ci, c) in para.controls.iter().enumerate() {
                if matches!(c, crate::model::control::Control::Table(_)) {
                    host = Some((pi, ci));
                    break 'outer;
                }
            }
        }
        let (host_pi, host_ci) = host.expect("표 없음");
        let total = core.page_count() as u32;
        for cell in 0..3usize {
            let path = format!(
                r#"[{{"controlIndex":{},"cellIndex":{},"cellParaIndex":0}}]"#,
                host_ci, cell
            );
            core.invalidate_page_tree_cache();
            let base = core
                .get_cursor_rect_by_path_with_hint(0, host_pi, &path, 0, None)
                .expect("base rect");
            // 정힌트(정답 페이지) / 오힌트(마지막 페이지·범위 밖) 전부 동일해야 한다.
            let base_page: u32 = base
                .split("\"pageIndex\":")
                .nth(1)
                .and_then(|s| s.split(&[',', '}'][..]).next())
                .and_then(|s| s.trim().parse().ok())
                .expect("pageIndex");
            for hint in [
                Some(base_page),
                Some(total.saturating_sub(1)),
                Some(total + 7),
            ] {
                core.invalidate_page_tree_cache();
                let hinted = core
                    .get_cursor_rect_by_path_with_hint(0, host_pi, &path, 0, hint)
                    .expect("hinted rect");
                assert_eq!(
                    base, hinted,
                    "#2021 힌트 {hint:?} 좌표 불일치 (cell {cell})"
                );
            }
        }
    }
}

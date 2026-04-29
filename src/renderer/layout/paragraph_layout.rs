//! 문단 레이아웃 (인라인 표, 문단 전체/부분, composed/raw) + 번호 매기기

use crate::model::paragraph::Paragraph;
use crate::model::style::{Alignment, HeadType, LineSpacingType, Numbering, UnderlineType};
use crate::model::control::Control;
use crate::model::bin_data::BinDataContent;
use super::super::render_tree::*;
use super::super::page_layout::LayoutRect;
use super::super::height_measurer::MeasuredTable;
use super::super::composer::{ComposedParagraph, compose_paragraph};
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{TextStyle, ShapeStyle, TabStop, hwpunit_to_px, format_number, NumberFormat as NumFmt, AutoNumberCounter};
use super::{LayoutEngine, CellContext};
use super::text_measurement::{resolved_to_text_style, estimate_text_width, compute_char_positions, extract_tab_leaders_with_extended, find_next_tab_stop};
use super::border_rendering::create_border_line_nodes;
use super::utils::{resolve_numbering_id, expand_numbering_format, numbering_format_to_number_format, find_bin_data};

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
    let (tp, tt, ft) = find_next_tab_stop(
        abs_before, tab_stops, tw, auto_tab_right, available_width,
    );
    if tt == 1 || tt == 2 {
        Some((tp, tt, ft))
    } else {
        None
    }
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
        let para_style_id = composed.map(|c| c.para_style_id as usize)
            .unwrap_or(para.para_shape_id as usize);
        let para_style = styles.para_styles.get(para_style_id);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);
        let alignment = para_style.map(|s| s.alignment).unwrap_or(Alignment::Left);

        let y = y_start + spacing_before;

        // 2. treat_as_char 표 목록과 폭 수집
        let inline_tables: Vec<(usize, &crate::model::table::Table)> = para.controls.iter().enumerate()
            .filter_map(|(i, c)| {
                if let Control::Table(t) = c {
                    if t.common.treat_as_char { return Some((i, t.as_ref())); }
                }
                None
            })
            .collect();

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
            let prev_char_utf16_len = if text_chars[i - 1] >= '\u{10000}' { 2u32 } else { 1 };
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
        let table_widths: Vec<f64> = inline_tables.iter().map(|(_, t)| {
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
                        if w > col_widths[c] { col_widths[c] = w; }
                    }
                }
            }
            let total: f64 = col_widths.iter().sum::<f64>()
                + cell_spacing * (col_count.saturating_sub(1) as f64);
            total
        }).collect();

        // 4b. 텍스트 세그먼트 폭 계산
        let char_style_id = para.char_shapes.first()
            .map(|cs| cs.char_shape_id as u32)
            .unwrap_or(0);

        let seg_widths: Vec<f64> = segments.iter().map(|(s, e)| {
            let seg_text: String = text_chars[*s..*e].iter().collect();
            if seg_text.is_empty() { return 0.0; }
            // 세그먼트 내 char_shape 변경을 고려한 폭 계산
            let mut total = 0.0;
            for ch_idx in *s..*e {
                // 해당 문자의 char_shape 찾기
                let utf16_pos = offsets[ch_idx];
                let cs_id = para.char_shapes.iter().rev()
                    .find(|cs| cs.start_pos <= utf16_pos)
                    .map(|cs| cs.char_shape_id as u32)
                    .unwrap_or(char_style_id);
                let ch = text_chars[ch_idx];
                let lang = super::super::style_resolver::detect_lang_category(ch);
                let ts = resolved_to_text_style(styles, cs_id, lang);
                total += estimate_text_width(&ch.to_string(), &ts);
            }
            total
        }).collect();

        // 5. 총 폭과 정렬 계산
        let total_width: f64 = seg_widths.iter().sum::<f64>() + table_widths.iter().sum::<f64>();
        let available_width = col_area.width - margin_left - margin_right;
        let start_x = match alignment {
            Alignment::Center | Alignment::Distribute => col_area.x + margin_left + (available_width - total_width).max(0.0) / 2.0,
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
            let default_cs = para.char_shapes.first().map(|cs| cs.char_shape_id as u32).unwrap_or(0);
            let ts = resolved_to_text_style(styles, default_cs, 0);
            if ts.font_size > 0.0 { ts.font_size } else { 12.0 }
        };
        let baseline_dist = if let Some(ls) = para.line_segs.first() {
            ensure_min_baseline(hwpunit_to_px(ls.baseline_distance, self.dpi), para_max_font_size)
        } else {
            line_height * 0.8
        };
        // 텍스트 줄(표 아래) 전용 메트릭: line_seg[1]이 있으면 사용
        let text_line_baseline = if let Some(ls) = para.line_segs.get(1) {
            ensure_min_baseline(hwpunit_to_px(ls.baseline_distance, self.dpi), para_max_font_size)
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

        // LINE_SEG 기반 줄 나눔 위치 결정:
        // ls[1].text_start가 있으면 해당 UTF-16 위치에서 줄 나눔 (한컴 저장값 존재)
        // ls[1]이 없으면 자체 right_margin 기반 줄 나눔 (동적 reflow)
        let line_break_char_idx: Option<usize> = if para.line_segs.len() > 1 {
            let ts = para.line_segs[1].text_start as u32;
            // UTF-16 text_start를 char index로 변환 (제어문자 갭 보정)
            // text_start는 제어문자 8 code unit 포함한 절대 UTF-16 위치
            let mut utf16_pos = 0u32;
            let mut ctrl_gap = 0u32;
            // char_offsets에서 제어문자 갭 계산
            if !para.char_offsets.is_empty() {
                let first_offset = para.char_offsets[0];
                ctrl_gap += first_offset; // 선행 컨트롤
                for i in 1..para.char_offsets.len() {
                    let prev_len = if text_chars[i-1] >= '\u{10000}' { 2u32 } else { 1 };
                    let gap = para.char_offsets[i] - para.char_offsets[i-1];
                    if gap > prev_len + 4 {
                        ctrl_gap += gap - prev_len; // 중간 컨트롤 갭
                    }
                }
            }
            // text_start에서 ctrl_gap을 빼서 순수 텍스트 char index 추정
            let text_only_ts = ts.saturating_sub(ctrl_gap);
            // UTF-16 → char index 변환
            let mut char_idx = 0usize;
            let mut u16_accum = 0u32;
            for (i, ch) in text_chars.iter().enumerate() {
                if u16_accum >= text_only_ts {
                    char_idx = i;
                    break;
                }
                u16_accum += if *ch >= '\u{10000}' { 2 } else { 1 };
                char_idx = i + 1;
            }
            if char_idx > 0 && char_idx <= text_chars.len() {
                Some(char_idx)
            } else {
                None
            }
        } else {
            None
        };

        let mut inline_x = start_x;
        let mut current_y = y;
        let mut table_idx = 0;
        let mut max_table_bottom = y; // 표의 최대 하단 y (표 높이를 줄 높이로 사용하기 위함)
        let mut wrapped_below_table = false; // 텍스트가 표 아래로 줄바꿈되었는지

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
                        para.char_shapes.iter().rev()
                            .find(|cs| cs.start_pos <= utf16_pos)
                            .map(|cs| cs.char_shape_id as u32)
                            .unwrap_or(char_style_id)
                    };

                    for ch_idx in *s..*e {
                        // 각주 마커 삽입: 현재 문자 위치에 각주가 있으면 먼저 run flush + FootnoteMarker 노드 삽입
                        if let Some(&(_, fn_num)) = composed.and_then(|c| c.footnote_positions.iter().find(|&&(pos, _)| pos == ch_idx)) {
                            // 현재까지 누적된 run 출력
                            if ch_idx > line_run_start {
                                let run_text: String = text_chars[line_run_start..ch_idx].iter().collect();
                                let first_lang = super::super::style_resolver::detect_lang_category(text_chars[line_run_start]);
                                let run_ts = resolved_to_text_style(styles, current_cs_id, first_lang);
                                let run_width = estimate_text_width(&run_text, &run_ts);
                                let run_bbox_h = if wrapped_below_table { text_line_baseline } else { baseline_dist };
                                let run_id = tree.next_id();
                                let run_node = RenderNode::new(run_id,
                                    RenderNodeType::TextRun(TextRunNode {
                                        text: run_text, style: run_ts,
                                        char_shape_id: Some(current_cs_id),
                                        para_shape_id: Some(para_style_id as u16),
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        char_start: Some(line_run_start),
                                        cell_context: None, is_para_end: false, is_line_break_end: false,
                                        rotation: 0.0, is_vertical: false, char_overlap: None,
                                        border_fill_id: styles.char_styles.get(current_cs_id as usize).map(|cs| cs.border_fill_id).unwrap_or(0),
                                        baseline: run_bbox_h, field_marker: FieldMarkerType::None,
                                    }),
                                    BoundingBox::new(line_run_x, current_y, run_width, run_bbox_h),
                                );
                                col_node.children.push(run_node);
                                inline_x += run_width;
                                line_run_x = inline_x;
                                line_run_start = ch_idx;
                            }
                            // FootnoteMarker 노드 삽입 (위첨자로 렌더링됨)
                            let fn_text = format!("{})", fn_num);
                            let base_ts = resolved_to_text_style(styles, current_cs_id, 0);
                            let sup_font_size = (base_ts.font_size * 0.55).max(7.0);
                            let sup_ts = TextStyle { font_size: sup_font_size, font_family: base_ts.font_family.clone(), ..Default::default() };
                            let sup_w = estimate_text_width(&fn_text, &sup_ts);
                            let run_bbox_h = if wrapped_below_table { text_line_baseline } else { baseline_dist };
                            // 각주 컨트롤 인덱스 찾기
                            let fn_ctrl_idx = composed.map(|c| {
                                c.footnote_positions.iter().position(|&(p, _)| p == ch_idx).unwrap_or(0)
                            }).unwrap_or(0);
                            let marker_id = tree.next_id();
                            let marker_node = RenderNode::new(marker_id,
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
                        let cs_id = para.char_shapes.iter().rev()
                            .find(|cs| cs.start_pos <= utf16_pos)
                            .map(|cs| cs.char_shape_id as u32)
                            .unwrap_or(char_style_id);

                        let ch = text_chars[ch_idx];
                        let lang = super::super::style_resolver::detect_lang_category(ch);
                        let ts = resolved_to_text_style(styles, cs_id, lang);
                        let ch_w = estimate_text_width(&ch.to_string(), &ts);

                        // char_shape 변경 또는 줄바꿈 시 누적된 run을 출력
                        // LINE_SEG 기반 줄 나눔: text_start 위치에서 강제 개행
                        let need_wrap = if let Some(break_idx) = line_break_char_idx {
                            ch_idx >= break_idx && !wrapped_below_table
                        } else {
                            inline_x + ch_w > right_margin + 0.5 && inline_x > line_start_x + 1.0
                        };
                        let cs_changed = cs_id != current_cs_id;

                        // 줄바꿈된 텍스트의 BoundingBox 높이: 표 줄 vs 텍스트 줄
                        let run_bbox_h = if wrapped_below_table { text_line_baseline } else { baseline_dist };

                        if (cs_changed || need_wrap) && ch_idx > line_run_start {
                            // 누적된 run 출력
                            let run_text: String = text_chars[line_run_start..ch_idx].iter().collect();
                            let first_lang = super::super::style_resolver::detect_lang_category(text_chars[line_run_start]);
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
                                    border_fill_id: styles.char_styles.get(current_cs_id as usize)
                                        .map(|cs| cs.border_fill_id).unwrap_or(0),
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
                    let remaining_bbox_h = if wrapped_below_table { text_line_baseline } else { baseline_dist };

                    // 남은 run 출력
                    if line_run_start < *e {
                        let run_text: String = text_chars[line_run_start..*e].iter().collect();
                        let first_lang = super::super::style_resolver::detect_lang_category(text_chars[line_run_start]);
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
                                border_fill_id: styles.char_styles.get(current_cs_id as usize)
                                    .map(|cs| cs.border_fill_id).unwrap_or(0),
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
                let mt = measured_tables.iter().find(|mt|
                    mt.para_index == para_index && mt.control_index == *ctrl_idx
                );
                let tw = table_widths[table_idx];
                let tbl_h = mt.map(|m| m.total_height)
                    .unwrap_or_else(|| hwpunit_to_px(tbl.common.height as i32, self.dpi));
                let om_bottom = hwpunit_to_px(tbl.outer_margin_bottom as i32, self.dpi);
                let tbl_y = (current_y + baseline_dist + om_bottom - tbl_h).max(current_y);

                let table_bottom = self.layout_table(
                    tree, col_node, tbl,
                    section_index, styles, col_area, tbl_y,
                    bin_data_content, mt, 0,
                    Some((para_index, *ctrl_idx)),
                    Alignment::Left, None, 0.0, 0.0,
                    Some(inline_x), None, None,
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
            let mt = measured_tables.iter().find(|mt|
                mt.para_index == para_index && mt.control_index == *ctrl_idx
            );
            let tw = table_widths[table_idx];
            let tbl_h = mt.map(|m| m.total_height)
                .unwrap_or_else(|| hwpunit_to_px(tbl.common.height as i32, self.dpi));
            let om_bottom = hwpunit_to_px(tbl.outer_margin_bottom as i32, self.dpi);
            let tbl_y = (current_y + baseline_dist + om_bottom - tbl_h).max(current_y);

            let table_bottom = self.layout_table(
                tree, col_node, tbl,
                section_index, styles, col_area, tbl_y,
                bin_data_content, mt, 0,
                Some((para_index, *ctrl_idx)),
                Alignment::Left, None, 0.0, 0.0,
                Some(inline_x), None, None,
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
        let effective_line_bottom = max_table_bottom.max(text_bottom).max(y + line_height + line_spacing);
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
    ) -> f64 {
        let end_line = composed
            .map(|c| c.lines.len())
            .unwrap_or(para.line_segs.len());
        self.layout_partial_paragraph(
            tree, col_node, para, composed, styles, col_area, y_start, 0, end_line,
            section_index, para_index, multi_col_width_hu, bin_data_content,
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
    ) -> f64 {
        if let Some(comp) = composed {
            return self.layout_composed_paragraph(
                tree, col_node, comp, styles, col_area, y_start, start_line, end_line,
                section_index, para_index, None, false, 0.0, multi_col_width_hu,
                Some(para), bin_data_content,
            );
        }

        // ComposedParagraph 없는 경우 기존 방식 fallback
        self.layout_raw_paragraph(
            tree, col_node, para, col_area, y_start, start_line, end_line,
        )
    }

    /// ComposedParagraph를 사용한 레이아웃
    /// `is_last_cell_para`: 셀 내 마지막 문단이면 true (마지막 줄의 trailing line_spacing 제외)
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
        is_last_cell_para: bool,
        first_line_x_offset: f64,
        multi_col_width_hu: Option<i32>,
        para: Option<&Paragraph>,
        bin_data_content: Option<&[BinDataContent]>,
    ) -> f64 {
        let mut y = y_start;
        let end = end_line.min(composed.lines.len());

        // 문단 스타일에서 여백 및 정렬 정보
        let para_style = styles.para_styles.get(composed.para_style_id as usize);
        let box_margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let box_margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let indent = para_style.map(|s| s.indent).unwrap_or(0.0);

        // 문단에 시각적 테두리(stroke 있는 border)가 있고 border_spacing 좌/우가 0인
        // 파일의 경우, 한컴은 paragraph margin 값을 inner padding으로도 사용하여
        // 텍스트가 테두리에 붙지 않도록 한다. rhwp는 동일 효과를 내기 위해 텍스트
        // 위치 계산에 사용하는 margin 값에만 inner padding을 더하고, 박스(테두리)
        // 위치는 원래 margin 값을 그대로 사용한다.
        // 배경(fill)만 있는 문단은 영향 받지 않도록 stroke 유무를 확인한다.
        let para_border_fill_id_pre = para_style.map(|s| s.border_fill_id).unwrap_or(0);
        let has_visible_stroke = if para_border_fill_id_pre > 0 {
            let idx = (para_border_fill_id_pre as usize).saturating_sub(1);
            styles.border_styles.get(idx)
                .map(|bs| bs.borders.iter().any(|b|
                    !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0))
                .unwrap_or(false)
        } else {
            false
        };
        let bs_left_px = para_style.map(|s| s.border_spacing[0]).unwrap_or(0.0);
        let bs_right_px = para_style.map(|s| s.border_spacing[1]).unwrap_or(0.0);
        let (inner_pad_left, inner_pad_right) = if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
            (box_margin_left, box_margin_right)
        } else {
            (0.0, 0.0)
        };
        let margin_left = box_margin_left + inner_pad_left;
        let margin_right = box_margin_right + inner_pad_right;
        let alignment = para_style.map(|s| s.alignment).unwrap_or(Alignment::Justify);
        let spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);
        let tab_width = para_style.map(|s| s.default_tab_width).unwrap_or(0.0);
        let tab_stops = para_style.map(|s| s.tab_stops.clone()).unwrap_or_default();
        let auto_tab_right = para_style.map(|s| s.auto_tab_right).unwrap_or(false);

        // treat_as_char 컨트롤의 px 폭 목록 (절대 char 위치, px 폭, control_index) — 정렬 보장
        let tac_offsets_px: Vec<(usize, f64, usize)> = {
            let mut v: Vec<(usize, f64, usize)> = composed.tac_controls.iter()
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
        // 단/페이지의 맨 처음 문단은 spacing_before 적용하지 않음
        let is_column_top = (y - col_area.y).abs() < 1.0;
        if start_line == 0 && spacing_before > 0.0 && !is_column_top {
            y += spacing_before;
        }

        // 문단 전체에서 모든 라인의 runs가 비어있는지 확인
        // (텍스트 없이 TAC 이미지만 있는 문단)
        let all_runs_empty = composed.lines[start_line..end].iter().all(|l| l.runs.is_empty());

        // 개요 번호/글머리표 마커 폭 사전 계산 (첫 줄 가용폭 차감용)
        let numbering_width = if start_line == 0 {
            if let Some(ref num_text) = composed.numbering_text {
                let num_style = composed.lines.first()
                    .and_then(|l| l.runs.first())
                    .map(|r| resolved_to_text_style(styles, r.char_style_id, r.lang_index))
                    .unwrap_or_else(|| resolved_to_text_style(styles, 0, 0));
                estimate_text_width(num_text, &num_style)
            } else { 0.0 }
        } else { 0.0 };

        // 배경/테두리 렌더링을 위한 시작 위치 기록
        // 문단 경계 = 이전 문단 끝 = y_start (spacing_before 적용 전)
        let bg_y_start = if para_border_fill_id > 0 {
            y_start
        } else {
            y
        };
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

        for line_idx in start_line..end {
            let comp_line = &composed.lines[line_idx];

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
            let max_fs = comp_line.runs.iter()
                .map(|r| {
                    let ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                    if ts.font_size > 0.0 { ts.font_size } else { 12.0 }
                })
                .fold(0.0f64, f64::max);
            // LineSeg.line_height는 HWP에서 줄간격이 이미 반영된 값.
            // PARA_LINE_SEG가 없는 폴백(400 HWPUNIT=5.333px) 등 line_height가 폰트 크기보다 작으면,
            // ParaShape의 줄간격 설정(line_spacing_type + line_spacing)으로 올바른 줄 높이를 계산한다.
            let raw_lh = hwpunit_to_px(comp_line.line_height, self.dpi);
            let line_height = {
                let ls_val  = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                let ls_type = para_style.map(|s| s.line_spacing_type).unwrap_or(LineSpacingType::Percent);
                crate::renderer::corrected_line_height(raw_lh, max_fs, ls_type, ls_val)
            };
            // 인라인 Shape(글상자)가 있는 줄: line_height에 Shape 높이가 포함됨
            // Shape는 별도 패스에서 para_y 기준으로 렌더링되므로,
            // 텍스트의 y와 line_height를 폰트 기반으로 보정하여 baseline 정렬
            let has_tac_shape = !tac_offsets_px.is_empty() && para.map(|p| {
                tac_offsets_px.iter().any(|(_, _, ci)| {
                    p.controls.get(*ci).map(|c| matches!(c, Control::Shape(_))).unwrap_or(false)
                })
            }).unwrap_or(false);
            let (line_height, baseline) = if has_tac_shape && raw_lh > max_fs * 1.5 {
                // Shape 높이가 line_height에 포함 → 폰트 기반 line_height 사용
                let font_lh = max_fs * 1.2; // 폰트 크기의 120%
                let font_bl = max_fs * 0.85;
                (font_lh, ensure_min_baseline(font_bl, max_fs))
            } else {
                (line_height, ensure_min_baseline(
                    hwpunit_to_px(comp_line.baseline_distance, self.dpi), max_fs))
            };

            // 들여쓰기/내어쓰기: 문단 여백은 무조건 적용
            // - 보통(ind=0): 모든 줄 margin_left
            // - 들여쓰기(ind>0): 첫줄 margin_left+indent, 다음줄 margin_left
            // - 내어쓰기(ind<0): 첫줄 margin_left, 다음줄 margin_left+|indent|
            let line_indent = if indent > 0.0 {
                if line_idx == 0 { indent } else { 0.0 }
            } else if indent < 0.0 {
                if line_idx == 0 { 0.0 } else { indent.abs() }
            } else {
                0.0
            };
            let effective_margin_left = margin_left + line_indent;

            // 인라인 Shape가 있는 줄: 텍스트 y를 Shape 하단 baseline에 맞춤
            let text_y = if has_tac_shape && raw_lh > max_fs * 1.5 {
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
            if cell_ctx.is_none() && text_y + line_height > col_bottom + 0.5 {
                eprintln!(
                    "LAYOUT_OVERFLOW_DRAW: section={} pi={} line={} y={:.1} col_bottom={:.1} overflow={:.1}px",
                    section_index, para_index, line_idx,
                    text_y + line_height, col_bottom, text_y + line_height - col_bottom,
                );
            }
            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine({
                    let vpos = para.and_then(|p| p.line_segs.get(line_idx)).map(|ls| ls.vertical_pos).unwrap_or(0);
                    TextLineNode::with_para_vpos(line_height, baseline, section_index, para_index, line_idx as u32, vpos)
                }),
                BoundingBox::new(
                    col_area.x + effective_margin_left,
                    text_y,
                    col_area.width - effective_margin_left - margin_right,
                    line_height,
                ),
            );

            let inline_offset = if line_idx == start_line { first_line_x_offset } else { 0.0 };
            // 번호/글머리표 마커: 모든 줄에서 마커 폭만큼 가용폭 차감 (행잉 인덴트)
            let num_offset = if numbering_width > 0.0 { numbering_width } else { 0.0 };
            let available_width = col_area.width - effective_margin_left - margin_right - inline_offset - num_offset;


            // 텍스트 정렬을 위한 전체 줄 폭 계산 (자연 폭, 추가 간격 미포함)
            // treat_as_char 이미지 폭도 포함하여 정확한 폭 산출
            let mut est_x = effective_margin_left + inline_offset;
            let est_x_start = est_x;
            let mut pending_right_tab_est: Option<(f64, u8, u8)> = None;
            let mut run_char_pos_est = comp_line.char_start;
            // cross-run 탭 감지용 inline_tabs(composed.tab_extended) 커서 — Task #290
            let mut inline_tab_cursor_est: usize = 0;
            for run in &comp_line.runs {
                let run_char_count_est = if run.char_overlap.is_some() {
                    let chars: Vec<char> = run.text.chars().collect();
                    if crate::renderer::composer::decode_pua_overlap_number(&chars).is_some() {
                        1
                    } else {
                        chars.len()
                    }
                } else {
                    run.text.chars().count()
                };
                let run_char_end_est = run_char_pos_est + run_char_count_est;
                let mut ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                ts.default_tab_width = tab_width;
                ts.tab_stops = tab_stops.clone();
                ts.auto_tab_right = auto_tab_right;
                ts.available_width = available_width;
                ts.inline_tabs = composed.tab_extended.clone();
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
                        let effective_pos = if tab_type == 1 && fill_type != 0 {
                            effective_margin_left + available_width
                        } else {
                            tab_pos
                        };
                        match tab_type {
                            1 => {
                                // [Task #279] 전체 run 폭 기준 — 선행 공백이 있는 경우 (예: " 16")
                                // run 시작 x 가 좌측으로 가서 시각적으로 페이지번호 right edge 가
                                // effective_pos 에 정렬되도록.
                                let run_w = estimate_text_width(&run.text, &ts);
                                est_x = effective_pos - run_w;
                            }
                            2 => {
                                let run_w = estimate_text_width(&run.text, &ts);
                                est_x = effective_pos - run_w / 2.0;
                            }
                            _ => {}
                        }
                    }
                }
                // 글자겹침 run: PUA 다자리 숫자는 1글자 폭, 그 외는 font_size * char_count
                if run.char_overlap.is_some() {
                    let fs = if ts.font_size > 0.0 { ts.font_size } else { 12.0 };
                    let chars: Vec<char> = run.text.chars().collect();
                    let w = if crate::renderer::composer::decode_pua_overlap_number(&chars).is_some() {
                        fs // 다자리 PUA 숫자는 하나의 원/사각형 = 1글자 폭
                    } else {
                        fs * run_char_count_est as f64
                    };
                    est_x += w;
                    run_char_pos_est = run_char_end_est;
                    inline_tab_cursor_est += run.text.chars().filter(|c| *c == '\t').count();
                    continue;
                }
                // treat_as_char 분기점 처리: run 내 tac 위치에서 이미지 폭 삽입
                // 마지막 run에서는 run_char_end 위치의 TAC도 포함
                let run_chars_est: Vec<char> = run.text.chars().collect();
                let mut seg_start_est = 0usize;
                let is_last_run_est_tac = run_char_end_est >= comp_line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>() + comp_line.char_start;
                for &(tac_abs_pos, tac_w, _) in tac_offsets_px.iter()
                    .filter(|(pos, _, _)| *pos >= run_char_pos_est && (*pos < run_char_end_est || (is_last_run_est_tac && *pos == run_char_end_est)))
                {
                    let tac_rel = tac_abs_pos - run_char_pos_est;
                    if seg_start_est < tac_rel {
                        let seg: String = run_chars_est[seg_start_est..tac_rel].iter().collect();
                        ts.line_x_offset = est_x;
                        est_x += estimate_text_width(&seg, &ts);
                    }
                    est_x += tac_w;
                    seg_start_est = tac_rel;
                }
                // 마지막 세그먼트 처리
                let remaining_est: String = run_chars_est[seg_start_est..].iter().collect();
                ts.line_x_offset = est_x;
                if !remaining_est.is_empty() {
                    est_x += estimate_text_width(&remaining_est, &ts);
                }
                // run이 \t로 끝나면 다음 run에 오른쪽/가운데 탭 조정 필요 — Task #290:
                // inline_tabs(composed.tab_extended) 가 LEFT 를 명시하면 cross-run pending 을 설정하지 않는다.
                // [Task #279] trailing 공백 (\t 뒤에 따라오는 ' ') 도 허용 — 목차 소제목의
                // 들여쓰기 문단에서 한컴이 "\t " 형태로 저장하는 케이스가 있음.
                let trimmed_end = run.text.trim_end_matches(|c: char| c == ' ' || c == '\u{2007}');
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
                // 각주 마커 폭: run 내에 각주가 있으면 마커 위첨자 폭 추가
                let is_last_run_est = run_char_end_est >= comp_line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>() + comp_line.char_start;
                for &(fpos, fnum) in composed.footnote_positions.iter() {
                    if fpos >= run_char_pos_est && (fpos < run_char_end_est || (is_last_run_est && fpos == run_char_end_est)) {
                        let fn_text = format!("{})", fnum);
                        let sup_size = (ts.font_size * 0.55).max(7.0);
                        let sup_ts = TextStyle { font_size: sup_size, font_family: ts.font_family.clone(), ..Default::default() };
                        est_x += estimate_text_width(&fn_text, &sup_ts);
                    }
                }
                run_char_pos_est = run_char_end_est;
                inline_tab_cursor_est += run.text.chars().filter(|c| *c == '\t').count();
            }
            // 교차 run 탭으로 인한 역방향 이동이 있을 수 있으므로
            // est_x 차이로 정확한 점유 폭을 계산
            let mut total_text_width = (est_x - est_x_start).max(0.0);
            // TAC 이미지/Shape 폭이 est_x에 미포함된 경우 별도 추가
            // (이미지가 텍스트 끝 위치에 있으면 run 범위 필터에서 제외됨)
            let total_tac_width_in_line: f64 = tac_offsets_px.iter()
                .filter(|(pos, _, _)| {
                    let line_start = comp_line.char_start;
                    let line_end = line_start + comp_line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>();
                    *pos >= line_start && *pos <= line_end
                })
                .map(|(_, w, _)| w)
                .sum();
            if total_tac_width_in_line > 0.0 && total_text_width < total_tac_width_in_line {
                total_text_width += total_tac_width_in_line;
            }
            let is_last_line_of_para = line_idx == end - 1 && end == composed.lines.len();


            // 정렬별 간격 분배 계산
            let has_forced_break = comp_line.has_line_break;
            let needs_justify = alignment == Alignment::Justify
                && !is_last_line_of_para && !has_forced_break;
            let needs_distribute = alignment == Alignment::Distribute
                || (alignment == Alignment::Split && !is_last_line_of_para && !has_forced_break);

            let has_tabs = comp_line.runs.iter().any(|r| r.text.contains('\t'));
            let total_char_count: usize = comp_line.runs.iter()
                .map(|r| r.text.chars().filter(|c| *c != '\t').count()).sum();

            // Task #352: 라인 내 dash leader (3+ 연속 '-') 글자 수 카운트.
            // visible_count 까지의 chars 에서만 카운트 (후행 공백 제외).
            let count_dash_leaders = |chars: &[char]| -> usize {
                let mut count = 0;
                let n = chars.len();
                let mut i = 0;
                while i < n {
                    if chars[i] == '-' {
                        let mut j = i;
                        while j < n && chars[j] == '-' { j += 1; }
                        let run_len = j - i;
                        if run_len >= 3 { count += run_len; }
                        i = j;
                    } else { i += 1; }
                }
                count
            };

            let (extra_word_sp, extra_char_sp, extra_dash_sp) = if needs_justify {
                // 양쪽 정렬: 후행 공백 제외한 내부 공백에 분배
                let all_chars: Vec<char> = comp_line.runs.iter()
                    .flat_map(|r| r.text.chars()).collect();
                let trailing_spaces = all_chars.iter().rev()
                    .take_while(|c| **c == ' ').count();
                let visible_count = all_chars.len() - trailing_spaces;
                let interior_spaces = all_chars[..visible_count].iter()
                    .filter(|c| **c == ' ').count();
                let leader_dashes = count_dash_leaders(&all_chars[..visible_count]);
                if interior_spaces > 0 {
                    // 후행 공백 폭 계산
                    let trailing_width = if trailing_spaces > 0 {
                        if let Some(last_run) = comp_line.runs.last() {
                            let mut ts = resolved_to_text_style(styles, last_run.char_style_id, last_run.lang_index);
                            ts.default_tab_width = tab_width;
                            let trailing_str: String = " ".repeat(trailing_spaces);
                            estimate_text_width(&trailing_str, &ts)
                        } else { 0.0 }
                    } else { 0.0 };
                    let effective_used = total_text_width - trailing_width;
                    let slack = available_width - effective_used;
                    if leader_dashes > 0 && slack > 0.0 {
                        // Task #352: 라인에 dash leader 가 있고 슬랙이 양수면
                        // dash 가 흡수 (PDF elastic leader 동작 모방). 공백·일반
                        // 글자 자연 폭 유지.
                        (0.0, 0.0, slack / leader_dashes as f64)
                    } else {
                        // 양쪽 정렬: 단어 간격 분배 (또는 음수 슬랙 시 압축)
                        let raw_ews = slack / interior_spaces as f64;
                        let space_base_w = estimate_text_width(" ", &resolved_to_text_style(
                            styles, comp_line.runs[0].char_style_id, comp_line.runs[0].lang_index));
                        let min_ews = -(space_base_w * 0.5);
                        (raw_ews.max(min_ews), 0.0, 0.0)
                    }
                } else if total_char_count > 1 {
                    // 양쪽 정렬이지만 공백 없음 (일본어/숫자 등):
                    let slack = available_width - total_text_width;
                    if leader_dashes > 0 && slack > 0.0 {
                        (0.0, 0.0, slack / leader_dashes as f64)
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
                let avg_char_w = total_text_width / total_char_count as f64;
                let min_sp = -avg_char_w * 0.5;
                (0.0, raw.max(min_sp), 0.0)
            } else if total_text_width > available_width && total_char_count > 1 && !has_tabs {
                // 비정렬(왼쪽/오른쪽/가운데) 텍스트가 오버플로우할 때 글자 간격 압축
                let raw = (available_width - total_text_width) / total_char_count as f64;
                let avg_char_w = total_text_width / total_char_count as f64;
                let min_sp = -avg_char_w * 0.5;
                (0.0, raw.max(min_sp), 0.0)
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
                    let natural_w: f64 = comp_line.runs.iter().map(|r| {
                        let mut ts = resolved_to_text_style(styles, r.char_style_id, r.lang_index);
                        ts.default_tab_width = tab_width;
                        ts.letter_spacing = 0.0;
                        estimate_text_width(&r.text, &ts)
                    }).sum();
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
                    if delta.abs() < 0.5 { break; }
                    extra += delta / total_char_count as f64;
                }
                (0.0, extra, 0.0)
            } else {
                (0.0, 0.0, 0.0)
            };

            // 셀 underflow 분기로 자간 확장된 경우 정렬 기준 폭은 확장 후 폭이어야 함
            let effective_text_width = if extra_char_sp > 0.0
                && cell_ctx.is_some()
                && !needs_justify
                && !needs_distribute
                && total_char_count > 1
            {
                total_text_width + extra_char_sp * total_char_count as f64
            } else {
                total_text_width
            };

            // 비첫줄에서 번호 마커 오프셋 (첫 줄은 마커 렌더링이 x를 전진시킴)
            let num_x_offset = if num_offset > 0.0 && !(line_idx == start_line && start_line == 0) {
                num_offset
            } else { 0.0 };
            let x_start = match alignment {
                Alignment::Center => {
                    col_area.x + effective_margin_left + inline_offset + num_x_offset + (available_width - effective_text_width).max(0.0) / 2.0
                }
                Alignment::Distribute if !needs_distribute || total_char_count <= 1 => {
                    col_area.x + effective_margin_left + inline_offset + num_x_offset + (available_width - effective_text_width).max(0.0) / 2.0
                }
                Alignment::Right => {
                    col_area.x + effective_margin_left + inline_offset + num_x_offset + (available_width - effective_text_width).max(0.0)
                }
                _ => col_area.x + effective_margin_left + inline_offset + num_x_offset, // Left, Justify, Split, Distribute(분배중)
            };

            // TextRun 노드 생성
            // 선행 공백은 x좌표 오프셋으로 처리하여 SVG 뷰어의 폰트 메트릭과 무관하게 정렬
            let mut x = x_start;

            // 개요 번호/글머리표: 첫 줄에서 별도 TextRunNode로 렌더링 (char_start: None)
            if line_idx == start_line && start_line == 0 {
                if let Some(ref num_text) = composed.numbering_text {
                    let num_style = if let Some(first_run) = comp_line.runs.first() {
                        resolved_to_text_style(styles, first_run.char_style_id, first_run.lang_index)
                    } else {
                        resolved_to_text_style(styles, 0, 0)
                    };
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
                    let ctrl_positions = crate::document_core::helpers::find_control_text_positions(pa);
                    pa.controls.iter().enumerate().filter_map(|(ci, ctrl)| {
                        let pos = ctrl_positions.get(ci).copied().unwrap_or(0);
                        match ctrl {
                            Control::Shape(s) => Some((pos, format!("[{}]", s.shape_name()))),
                            Control::Picture(_) => Some((pos, "[그림]".to_string())),
                            Control::Table(t) if t.common.treat_as_char => Some((pos, "[표]".to_string())),
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
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // 각주 마커 위치 수집
            let fn_positions: &[(usize, u16)] = &composed.footnote_positions;
            let mut fn_marker_inserted = vec![false; fn_positions.len()];

            let mut pending_right_tab_render: Option<(f64, u8, u8)> = None;
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
                        let base_style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                        let mut ms = base_style;
                        ms.color = 0x0000FF; // BGR: 빨간색
                        ms.font_size *= 0.55;
                        let mw = estimate_text_width(stext, &ms);
                        let mid = tree.next_id();
                        let mn = RenderNode::new(mid, RenderNodeType::TextRun(TextRunNode {
                            text: stext.clone(), style: ms,
                            char_shape_id: None,
                            para_shape_id: Some(composed.para_style_id),
                            section_index: Some(section_index),
                            para_index: Some(para_index),
                            char_start: None,
                            cell_context: cell_ctx.clone(),
                            is_para_end: false, is_line_break_end: false,
                            rotation: 0.0, is_vertical: false,
                            char_overlap: None, border_fill_id: 0, baseline,
                            field_marker: FieldMarkerType::ShapeMarker(*spos),
                        }), BoundingBox::new(x, y, mw, line_height));
                        line_node.children.push(mn);
                        x += mw;
                    }
                }
                let mut text_style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                text_style.default_tab_width = tab_width;
                text_style.tab_stops = tab_stops.clone();
                text_style.auto_tab_right = auto_tab_right;
                text_style.available_width = available_width;
                text_style.inline_tabs = composed.tab_extended.clone();
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
                    let effective_pos = if tab_type == 1 && fill_type != 0 {
                        effective_margin_left + available_width
                    } else {
                        tab_pos
                    };
                    match tab_type {
                        1 => {
                            // [Task #279] 전체 run 폭 기준 — 선행 공백이 있는 경우 (예: " 16")
                            // run 시작 x 가 좌측으로 가서 시각적으로 페이지번호 right edge 가
                            // effective_pos 에 정렬되도록.
                            let next_w = estimate_text_width(&run.text, &text_style);
                            x = col_area.x + effective_pos - next_w;
                        }
                        2 => {
                            let next_w = estimate_text_width(&run.text, &text_style);
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
                    if let Some(prev_run_node) = line_node.children.iter_mut().rev()
                        .find(|n| {
                            if let RenderNodeType::TextRun(tr) = &n.node_type {
                                !tr.style.tab_leaders.is_empty()
                            } else {
                                false
                            }
                        })
                    {
                        let prev_bbox_x = prev_run_node.bbox.x;
                        if let RenderNodeType::TextRun(prev_text_run) = &mut prev_run_node.node_type {
                            if let Some(last_leader) = prev_text_run.style.tab_leaders.last_mut() {
                                let space_gap = if text_style.font_size > 0.0 {
                                    text_style.font_size * 0.25
                                } else {
                                    3.0
                                };
                                let new_end_x = (x - prev_bbox_x - space_gap).max(last_leader.start_x);
                                if new_end_x < last_leader.end_x {
                                    last_leader.end_x = new_end_x;
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
                let run_border_fill_id = styles.char_styles.get(run.char_style_id as usize)
                    .map(|cs| cs.border_fill_id).unwrap_or(0);
                let full_width = if run.char_overlap.is_some() {
                    // 글자겹침: PUA 다자리 숫자는 1글자 폭, 그 외는 font_size * char_count
                    let fs = if text_style.font_size > 0.0 { text_style.font_size } else { 12.0 };
                    let chars: Vec<char> = run.text.chars().collect();
                    if crate::renderer::composer::decode_pua_overlap_number(&chars).is_some() {
                        fs
                    } else {
                        fs * run.text.chars().count() as f64
                    }
                } else {
                    estimate_text_width(&run.text, &text_style)
                };
                // 탭 리더 계산: 탭이 포함된 run에서 채움 기호 정보 추출
                // inline_tabs를 일시 제거하여 tab_stops 기반 위치 계산과 일관되게 함
                if has_tabs && run.text.contains('\t') {
                    let saved_inline_tabs = std::mem::take(&mut text_style.inline_tabs);
                    let positions = compute_char_positions(&run.text, &text_style);
                    text_style.inline_tabs = saved_inline_tabs;
                    text_style.tab_leaders = extract_tab_leaders_with_extended(&run.text, &positions, &text_style, &composed.tab_extended);
                }
                // 교차 run 오른쪽/가운데 탭 감지 — Task #290:
                // inline_tabs(composed.tab_extended) 가 LEFT 를 명시하면 cross-run pending 을 설정하지 않는다.
                // [Task #279] trailing 공백 (\t 뒤에 따라오는 ' ') 도 허용 — 목차 소제목의
                // 들여쓰기 문단에서 한컴이 "\t " 형태로 저장하는 케이스가 있음.
                let trimmed_end_r = run.text.trim_end_matches(|c: char| c == ' ' || c == '\u{2007}');
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
                let run_char_count = if run.char_overlap.is_some() {
                    // 글자겹침(CharOverlap)은 HWP char_offset 공간에서 1개 위치만 차지
                    let chars: Vec<char> = run.text.chars().collect();
                    if crate::renderer::composer::decode_pua_overlap_number(&chars).is_some() {
                        1
                    } else {
                        chars.len()
                    }
                } else {
                    run.text.chars().count()
                };
                let run_char_end = run_char_pos + run_char_count;
                let is_last_run = is_last_line_of_para && is_last_run_of_line(run_idx);
                let is_line_break = comp_line.has_line_break && is_last_run_of_line(run_idx);

                // treat_as_char 분기점: run 내 이미지 위치 목록 (rel_pos, width_px, control_index)
                // 마지막 run에서는 run_char_end 위치의 TAC도 포함 (문단 끝 수식/그림)
                let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
                    .filter(|(pos, _, _)| *pos >= run_char_pos && (*pos < run_char_end || (is_last_run && *pos == run_char_end)))
                    .map(|(pos, w, ci)| (pos - run_char_pos, *w, *ci))
                    .collect();

                if run_tacs.is_empty() {
                    // tac 없음: 기존 렌더링 경로
                    // 선행 공백 분리
                    let leading_spaces: String = run.text.chars().take_while(|c| *c == ' ').collect();
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
                            let char_w = if run_char_count > 0 { run_w / run_char_count as f64 } else { 0.0 };
                            for rt in &p.range_tags {
                                let rt_type = (rt.tag >> 24) & 0xFF;
                                if rt_type != 2 { continue; }
                                let rt_start = rt.start as usize;
                                let rt_end = rt.end as usize;
                                // run과 RangeTag가 겹치는 문자 범위
                                let overlap_start = rt_start.max(run_char_pos);
                                let overlap_end = rt_end.min(run_char_end);
                                if overlap_start >= overlap_end { continue; }
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
                        // run 내 각주 위치 수집 (run 내 상대 위치, 각주 번호, fn_positions 인덱스)
                        // 마지막 run에서는 run_char_end 위치의 각주도 포함 (문단 끝 각주)
                        let is_last = is_last_run_of_line(run_idx);
                        let run_fn_markers: Vec<(usize, u16, usize)> = fn_positions.iter().enumerate()
                            .filter_map(|(fni, &(fpos, fnum))| {
                                let in_range = fpos >= run_char_pos && (fpos < run_char_end || (is_last && fpos == run_char_end));
                                if !fn_marker_inserted[fni] && in_range {
                                    Some((fpos - run_char_pos, fnum, fni))
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

                            for &(rel_pos, fnum, fni) in &run_fn_markers {
                                fn_marker_inserted[fni] = true;
                                // 각주 앞 텍스트 세그먼트
                                if rel_pos > seg_start {
                                    let seg_text: String = run_chars[seg_start..rel_pos].iter().collect();
                                    let seg_w = estimate_text_width(&seg_text, &text_style);
                                    let seg_id = tree.next_id();
                                    let seg_node = RenderNode::new(seg_id,
                                        RenderNodeType::TextRun(TextRunNode {
                                            text: seg_text, style: text_style.clone(),
                                            char_shape_id: Some(run.char_style_id),
                                            para_shape_id: Some(composed.para_style_id),
                                            section_index: Some(section_index),
                                            para_index: Some(para_index),
                                            char_start: Some(sub_char_offset),
                                            cell_context: cell_ctx.clone(),
                                            is_para_end: false, is_line_break_end: false,
                                            rotation: 0.0, is_vertical: false,
                                            char_overlap: None, border_fill_id: run_border_fill_id,
                                            baseline, field_marker: FieldMarkerType::None,
                                        }),
                                        BoundingBox::new(sub_x, y, seg_w, line_height),
                                    );
                                    line_node.children.push(seg_node);
                                    sub_x += seg_w;
                                    sub_char_offset += rel_pos - seg_start;
                                }
                                // FootnoteMarker 노드
                                let fn_text = format!("{})", fnum);
                                let base_ts = &text_style;
                                let sup_size = (base_ts.font_size * 0.55).max(7.0);
                                let sup_ts = TextStyle { font_size: sup_size, font_family: base_ts.font_family.clone(), color: base_ts.color, ..Default::default() };
                                let sup_w = estimate_text_width(&fn_text, &sup_ts);
                                let fid = tree.next_id();
                                let fn_node = RenderNode::new(fid, RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
                                    number: fnum,
                                    text: fn_text,
                                    base_font_size: base_ts.font_size,
                                    font_family: base_ts.font_family.clone(),
                                    color: base_ts.color,
                                    section_index,
                                    para_index,
                                    control_index: fni,
                                }), BoundingBox::new(sub_x, y, sup_w, line_height));
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
                                let seg_node = RenderNode::new(seg_id,
                                    RenderNodeType::TextRun(TextRunNode {
                                        text: seg_text, style: text_style,
                                        char_shape_id: Some(run.char_style_id),
                                        para_shape_id: Some(composed.para_style_id),
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        char_start: Some(sub_char_offset),
                                        cell_context: cell_ctx.clone(),
                                        is_para_end: is_last_run,
                                        is_line_break_end: is_line_break,
                                        rotation: 0.0, is_vertical: false,
                                        char_overlap: run.char_overlap.clone(),
                                        border_fill_id: run_border_fill_id,
                                        baseline, field_marker: FieldMarkerType::None,
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
                                let nodes = create_border_line_nodes(tree, &bs.borders[bi], lx1, ly1, lx2, ly2);
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
                                seg_style.tab_leaders = extract_tab_leaders_with_extended(&seg_text, &positions, &seg_style, &composed.tab_extended);
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
                                    let img_y = (y + baseline - pic_h).max(y);
                                    let bin_data_id = pic.image_attr.bin_data_id;
                                    let image_data = find_bin_data(bdc, bin_data_id)
                                        .map(|c| c.data.clone());
                                    let crop = {
                                        let c = &pic.crop;
                                        if c.right > c.left && c.bottom > c.top
                                            && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0) {
                                            Some((c.left, c.top, c.right, c.bottom))
                                        } else { None }
                                    };
                                    let original_size_hu = if pic.shape_attr.original_width > 0
                                        && pic.shape_attr.original_height > 0 {
                                        Some((pic.shape_attr.original_width, pic.shape_attr.original_height))
                                    } else { None };
                                    let img_id = tree.next_id();
                                    let img_node = RenderNode::new(
                                        img_id,
                                        RenderNodeType::Image(ImageNode {
                                            section_index: Some(section_index),
                                            para_index: Some(para_index),
                                            control_index: Some(tac_ci),
                                            crop,
                                            original_size_hu,
                                            effect: pic.image_attr.effect,
                                            brightness: pic.image_attr.brightness,
                                            contrast: pic.image_attr.contrast,
                                            ..ImageNode::new(bin_data_id, image_data)
                                        }),
                                        BoundingBox::new(x, img_y, tac_w, pic_h),
                                    );
                                    line_node.children.push(img_node);
                                }
                            }
                        }
                        // 인라인 Shape(글상자) 렌더링: 텍스트 흐름 순서에 맞게 배치
                        // Shape 내부의 텍스트/테두리를 직접 렌더링하고, 별도 Shape 패스에서는 스킵
                        if let Some(p) = para {
                            if let Some(Control::Shape(shape)) = p.controls.get(tac_ci) {
                                let common = shape.common();
                                let shape_h = hwpunit_to_px(common.height as i32, self.dpi);
                                let shape_y = (y + baseline - shape_h).max(y);
                                // 인라인 좌표 등록 → shape_layout.rs에서 이 Shape를 스킵
                                tree.set_inline_shape_position(section_index, para_index, tac_ci, x, shape_y);
                            }
                        }
                        // 인라인 수식: 직접 EquationNode로 렌더링
                        if let Some(p) = para {
                            if let Some(Control::Equation(eq)) = p.controls.get(tac_ci) {
                                // 수식 스크립트 → AST → 레이아웃 → SVG 조각
                                let tokens = crate::renderer::equation::tokenizer::tokenize(&eq.script);
                                let ast = crate::renderer::equation::parser::EqParser::new(tokens).parse();
                                let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                                let layout_box = crate::renderer::equation::layout::EqLayout::new(font_size_px).layout(&ast);
                                let color_str = crate::renderer::equation::svg_render::eq_color_to_svg(eq.color);
                                let svg_content = crate::renderer::equation::svg_render::render_equation_svg(
                                    &layout_box, &color_str, font_size_px,
                                );
                                // HWP 저장 높이를 우선 사용 (한컴 조판 결과 기준)
                                let hwp_eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                                let eq_h = if hwp_eq_h > 0.0 { hwp_eq_h } else { layout_box.height };
                                // 수식 baseline을 텍스트 baseline에 맞춤
                                // HWP 높이와 레이아웃 높이가 다르면 baseline도 비례 조정
                                let eq_y = if hwp_eq_h > 0.0 && layout_box.height > 0.0 {
                                    let scale = hwp_eq_h / layout_box.height;
                                    (y + baseline - layout_box.baseline * scale).max(y)
                                } else {
                                    (y + baseline - layout_box.baseline).max(y)
                                };
                                let (eq_cell_idx, eq_cell_para_idx) = if let Some(ref ctx) = cell_ctx {
                                    (Some(ctx.path[0].cell_index), Some(ctx.path[0].cell_para_index))
                                } else {
                                    (None, None)
                                };
                                let eq_node = RenderNode::new(
                                    tree.next_id(),
                                    RenderNodeType::Equation(crate::renderer::render_tree::EquationNode {
                                        svg_content,
                                        layout_box,
                                        color_str,
                                        color: eq.color,
                                        font_size: font_size_px,
                                        section_index: Some(section_index),
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
                                    }),
                                    BoundingBox::new(x, eq_y, tac_w, eq_h),
                                );
                                line_node.children.push(eq_node);
                                // 인라인 좌표 등록 → shape_layout에서 이 수식을 스킵
                                tree.set_inline_shape_position(section_index, para_index, tac_ci, x, eq_y);
                            }
                        }
                        // 인라인 TAC 표: 텍스트 흐름 위치에 직접 렌더링
                        // 표 하단 = 베이스라인 + outer_margin_bottom
                        if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                            if let Some(Control::Table(t)) = p.controls.get(tac_ci) {
                                if t.common.treat_as_char {
                                    let table_h = hwpunit_to_px(t.common.height as i32, self.dpi);
                                    let om_bottom = hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi);
                                    let table_y = (y + baseline + om_bottom - table_h).max(y);
                                    self.layout_table(
                                        tree, col_node, t,
                                        section_index, styles, col_area,
                                        table_y, bdc, None, 0,
                                        Some((para_index, tac_ci)),
                                        alignment, None, 0.0, 0.0,
                                        Some(x), None, None,
                                    );
                                    // 스킵 마커 등록 (별도 Table PageItem에서 중복 렌더 방지)
                                    tree.set_inline_shape_position(section_index, para_index, tac_ci, x, table_y);
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
                                    (ctx.parent_para_index, e.control_index, e.cell_index, e.cell_para_index)
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
                        seg_start = tac_rel;
                    }

                    // 마지막 tac 이후 텍스트 세그먼트 렌더링
                    let remaining: String = run_chars[seg_start..].iter().collect();
                    if !remaining.is_empty() {
                        let mut seg_style = text_style.clone();
                        seg_style.line_x_offset = x - col_area.x;
                        if has_tabs && remaining.contains('\t') {
                            let positions = compute_char_positions(&remaining, &seg_style);
                            seg_style.tab_leaders = extract_tab_leaders_with_extended(&remaining, &positions, &seg_style, &composed.tab_extended);
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
                    let mn = RenderNode::new(mid, RenderNodeType::TextRun(TextRunNode {
                        text: stext.clone(), style: ms,
                        char_shape_id: None,
                        para_shape_id: Some(composed.para_style_id),
                        section_index: Some(section_index),
                        para_index: Some(para_index),
                        char_start: None,
                        cell_context: cell_ctx.clone(),
                        is_para_end: false, is_line_break_end: false,
                        rotation: 0.0, is_vertical: false,
                        char_overlap: None, border_fill_id: 0, baseline,
                        field_marker: FieldMarkerType::ShapeMarker(*spos),
                    }), BoundingBox::new(x, y, mw, line_height));
                    line_node.children.push(mn);
                    x += mw;
                }
            }

            // run 루프 종료 후, run 범위 밖(pos >= run_char_pos)의 미매칭 TAC 이미지 렌더링
            if !comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
                if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                    for &(tac_pos, tac_w, tac_ci) in &tac_offsets_px {
                        if tac_pos <= run_char_pos {
                            continue; // run 범위 내(또는 끝): 이미 run 내에서 처리됨
                        }
                        if let Some(ctrl) = p.controls.get(tac_ci) {
                            if let Control::Picture(pic) = ctrl {
                                let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                let img_y = (y + baseline - pic_h).max(y);
                                let bin_data_id = pic.image_attr.bin_data_id;
                                let image_data = find_bin_data(bdc, bin_data_id)
                                    .map(|c| c.data.clone());
                                let crop = {
                                    let c = &pic.crop;
                                    if c.right > c.left && c.bottom > c.top
                                        && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0) {
                                        Some((c.left, c.top, c.right, c.bottom))
                                    } else { None }
                                };
                                let original_size_hu = if pic.shape_attr.original_width > 0
                                    && pic.shape_attr.original_height > 0 {
                                    Some((pic.shape_attr.original_width, pic.shape_attr.original_height))
                                } else { None };
                                let img_id = tree.next_id();
                                let img_node = RenderNode::new(
                                    img_id,
                                    RenderNodeType::Image(ImageNode {
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        control_index: Some(tac_ci),
                                        crop,
                                        original_size_hu,
                                        effect: pic.image_attr.effect,
                                        brightness: pic.image_attr.brightness,
                                        contrast: pic.image_attr.contrast,
                                        ..ImageNode::new(bin_data_id, image_data)
                                    }),
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
                                (ctx.parent_para_index, e.control_index, e.cell_index, e.cell_para_index)
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

            // runs가 비어있으면 빈 TextRun 생성 (빈 셀 편집용)
            if comp_line.runs.is_empty() {
                // runs가 없는 빈 줄에서 treat_as_char 이미지 렌더링
                // 테이블 셀 내부에서는 table_layout.rs가 layout_picture로 이미 처리하므로 스킵.
                // 셀 외부에서 텍스트 없이 TAC만 있는 문단인 경우에만 여기서 렌더링.
                if cell_ctx.is_none() && all_runs_empty && !tac_offsets_px.is_empty() && line_idx == start_line {
                    if let (Some(p), Some(bdc)) = (para, bin_data_content) {
                        // TAC 이미지 전체 폭 계산 후 문단 정렬 적용
                        let total_tac_width: f64 = tac_offsets_px.iter().map(|(_, w, _)| w).sum();
                        let align_offset = match alignment {
                            Alignment::Center | Alignment::Distribute => {
                                (available_width - total_tac_width).max(0.0) / 2.0
                            }
                            Alignment::Right => {
                                (available_width - total_tac_width).max(0.0)
                            }
                            _ => 0.0, // Left, Justify
                        };
                        let mut img_x = col_area.x + effective_margin_left + align_offset;
                        for &(_, tac_w, tac_ci) in &tac_offsets_px {
                            if let Some(ctrl) = p.controls.get(tac_ci) {
                                if let Control::Picture(pic) = ctrl {
                                    let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    let img_y = (y + baseline - pic_h).max(y);
                                    let bin_data_id = pic.image_attr.bin_data_id;
                                    let image_data = find_bin_data(bdc, bin_data_id)
                                        .map(|c| c.data.clone());
                                    let crop = {
                                        let c = &pic.crop;
                                        if c.right > c.left && c.bottom > c.top
                                            && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0) {
                                            Some((c.left, c.top, c.right, c.bottom))
                                        } else { None }
                                    };
                                    let original_size_hu = if pic.shape_attr.original_width > 0
                                        && pic.shape_attr.original_height > 0 {
                                        Some((pic.shape_attr.original_width, pic.shape_attr.original_height))
                                    } else { None };
                                    let img_id = tree.next_id();
                                    let img_node = RenderNode::new(
                                        img_id,
                                        RenderNodeType::Image(ImageNode {
                                            section_index: Some(section_index),
                                            para_index: Some(para_index),
                                            control_index: Some(tac_ci),
                                            crop,
                                            original_size_hu,
                                            effect: pic.image_attr.effect,
                                            brightness: pic.image_attr.brightness,
                                            contrast: pic.image_attr.contrast,
                                            ..ImageNode::new(bin_data_id, image_data)
                                        }),
                                        BoundingBox::new(img_x, img_y, tac_w, pic_h),
                                    );
                                    line_node.children.push(img_node);
                                    // [Task #418/#376] layout_shape_item 의 Task #347 분기 (빈 문단 +
                                    // TAC Picture 직접 emit) 와 이중 렌더링되지 않도록 인라인 위치를
                                    // 등록한다. layout_shape_item 은 등록된 경우 push 를 스킵한다.
                                    tree.set_inline_shape_position(
                                        section_index, para_index, tac_ci, img_x, img_y,
                                    );
                                    img_x += tac_w;
                                }
                            }
                        }
                    }
                }

                let run_id = tree.next_id();
                let text_style = resolved_to_text_style(styles, 0, 0);
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: String::new(),
                        style: text_style,
                        char_shape_id: None,
                        para_shape_id: Some(composed.para_style_id),
                        section_index: Some(section_index),
                        para_index: Some(para_index),
                        char_start: Some(char_offset),
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
                    BoundingBox::new(x_start, y, available_width, line_height),
                );
                line_node.children.push(run_node);
            }

            // [Task #287] 빈 runs 줄의 TAC 수식 인라인 처리
            // 큰 디스플레이 수식이 자체 LINE_SEG 를 가질 때 comp_line.runs 가 비어있는데,
            // run 루프가 돌지 않아 수식이 인라인 경로로 렌더되지 않고 shape_layout display
            // 경로로 떨어져 col_area.y 에 고정되던 문제를 해결한다.
            if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
                let line_start_char = comp_line.char_start;
                let line_end_char = composed.lines.get(line_idx + 1)
                    .map(|l| l.char_start)
                    .unwrap_or(usize::MAX);
                let mut inline_x = col_area.x + effective_margin_left;
                for &(tac_pos, tac_w, tac_ci) in &tac_offsets_px {
                    if tac_pos < line_start_char || tac_pos >= line_end_char {
                        continue;
                    }
                    if let Some(p) = para {
                        if let Some(Control::Equation(eq)) = p.controls.get(tac_ci) {
                            let tokens = crate::renderer::equation::tokenizer::tokenize(&eq.script);
                            let ast = crate::renderer::equation::parser::EqParser::new(tokens).parse();
                            let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                            let layout_box = crate::renderer::equation::layout::EqLayout::new(font_size_px).layout(&ast);
                            let color_str = crate::renderer::equation::svg_render::eq_color_to_svg(eq.color);
                            let svg_content = crate::renderer::equation::svg_render::render_equation_svg(
                                &layout_box, &color_str, font_size_px,
                            );
                            let hwp_eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                            let eq_h = if hwp_eq_h > 0.0 { hwp_eq_h } else { layout_box.height };
                            let eq_y = if hwp_eq_h > 0.0 && layout_box.height > 0.0 {
                                let scale = hwp_eq_h / layout_box.height;
                                (y + baseline - layout_box.baseline * scale).max(y)
                            } else {
                                (y + baseline - layout_box.baseline).max(y)
                            };
                            let (eq_cell_idx, eq_cell_para_idx) = if let Some(ref ctx) = cell_ctx {
                                (Some(ctx.path[0].cell_index), Some(ctx.path[0].cell_para_index))
                            } else {
                                (None, None)
                            };
                            let eq_node = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Equation(crate::renderer::render_tree::EquationNode {
                                    svg_content,
                                    layout_box,
                                    color_str,
                                    color: eq.color,
                                    font_size: font_size_px,
                                    section_index: Some(section_index),
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
                                }),
                                BoundingBox::new(inline_x, eq_y, tac_w, eq_h),
                            );
                            line_node.children.push(eq_node);
                            tree.set_inline_shape_position(section_index, para_index, tac_ci, inline_x, eq_y);
                            inline_x += tac_w;
                        }
                    }
                }
            }

            // ClickHere 필드 처리: 안내문 + 조판부호 마커 ([누름틀 시작]/[누름틀 끝])
            // char_x_map을 이용하여 필드 위치에 맞는 x 좌표 계산
            if let Some(p) = para {
                let line_char_end = char_offset;
                let line_char_start = comp_line.char_start;
                let active = self.active_field.borrow();
                let ctrl_codes = self.show_control_codes.get();

                // char_x_map에서 특정 char_idx에 해당하는 x 좌표를 보간 계산
                let find_x_for_char = |target: usize| -> f64 {
                    for i in 0..char_x_map.len().saturating_sub(1) {
                        let (c0, x0) = char_x_map[i];
                        let (c1, x1) = char_x_map[i + 1];
                        if target >= c0 && target <= c1 {
                            if c1 == c0 { return x0; }
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
                        let start_in_line = fr.start_char_idx >= line_char_start && fr.start_char_idx <= line_char_end;
                        let end_in_line = fr.end_char_idx >= line_char_start && fr.end_char_idx <= line_char_end;

                        if !start_in_line && !end_in_line { continue; }

                        let is_active = if let Some((af_sec, af_para, af_ctrl, ref af_cell)) = *active {
                            if af_sec != section_index || af_para != para_index || af_ctrl != fr.control_idx {
                                false
                            } else {
                                // cell_path 전체 일치 확인
                                match (af_cell, &cell_ctx) {
                                    (None, None) => true,
                                    (Some(af_path), Some(ctx)) => {
                                        // af_path와 ctx.path의 (control_index, cell_index) 쌍이 모두 일치해야 함
                                        af_path.len() == ctx.path.len()
                                        && af_path.iter().zip(ctx.path.iter()).all(|(&(ac, ax, _ap), entry)| {
                                            ac == entry.control_index && ax == entry.cell_index
                                        })
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
                                    field_marker: FieldMarkerType::FieldBegin,
                                }),
                                BoundingBox::new(marker_x, y, marker_w, line_height),
                            );
                            markers.push(MarkerInsert { marker_x, marker_w, node: m_node });
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
                                    para_shape_id: Some(composed.para_style_id),
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
                            markers.push(MarkerInsert { marker_x: anchor_x, marker_w: 0.0, node: anchor_node });
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
                                        field_marker: FieldMarkerType::None,
                                    }),
                                    BoundingBox::new(guide_x, y, guide_width, line_height),
                                );
                                markers.push(MarkerInsert { marker_x: guide_x, marker_w: guide_width, node: guide_node });
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
                                    field_marker: FieldMarkerType::FieldEnd,
                                }),
                                BoundingBox::new(marker_x, y, marker_w, line_height),
                            );
                            markers.push(MarkerInsert { marker_x, marker_w, node: m_node });
                        }
                    }
                }

                // 책갈피 조판부호 마커
                if ctrl_codes {
                    let ctrl_positions = crate::document_core::helpers::find_control_text_positions(p);
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
                                        field_marker: FieldMarkerType::None,
                                    }),
                                    BoundingBox::new(marker_x, y, marker_w, line_height),
                                );
                                markers.push(MarkerInsert { marker_x, marker_w, node: m_node });
                            }
                        }
                    }
                }

                // 도형 조판부호 마커는 텍스트 런 루프 내에서 직접 처리됨 (MarkerInsert 불사용)

                // 마커를 왼쪽부터 삽입하면서, 각 마커 뒤의 기존 노드와 이후 마커를 오른쪽으로 shift
                // zero-width 앵커(커서 위치용)는 shift하지 않고 원래 위치 유지
                markers.sort_by(|a, b| a.marker_x.partial_cmp(&b.marker_x).unwrap_or(std::cmp::Ordering::Equal));
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
                x += accumulated_shift;
            }

            // 강제 줄바꿈(\n)이 이 줄에서 제거되었으므로 char_offset에 1을 더하여
            // 다음 줄의 TextRun.char_start가 올바른 문서 좌표를 가리키도록 한다.
            if comp_line.has_line_break {
                char_offset += 1;
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
            if is_cell_last_line && cell_ctx.is_some() {
                y += line_height;
            } else {
                let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
                y += line_height + line_spacing_px;
            }
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
                let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
                    (ox + box_margin_left, ow - box_margin_left)
                } else {
                    (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
                };
                self.para_border_ranges.borrow_mut().push(
                    (para_border_fill_id, box_x, bg_y_start, box_w, y, top_inset, bottom_inset, is_partial_start, is_partial_end)
                );
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
                RenderNodeType::TextLine(TextLineNode::with_para(default_height, default_height * 0.8, section_index, para_index)),
                BoundingBox::new(col_area.x, y, col_area.width, default_height),
            );

            // 빈 문단에도 TextRun 노드를 생성하여 캐럿 위치 제공
            let run_id = tree.next_id();
            let text_style = resolved_to_text_style(styles, 0, 0);
            let run_node = RenderNode::new(
                run_id,
                RenderNodeType::TextRun(TextRunNode {
                    text: String::new(),
                    style: text_style,
                    char_shape_id: None,
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
            if y + line_height > col_bottom + 0.5 {
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
                let numbering_id = resolve_numbering_id(para_style.head_type, para_style.numbering_id, outline_numbering_id);
                let level = para_style.para_level;
                if numbering_id == 0 { return None; }
                let numbering = styles.numberings.get((numbering_id - 1) as usize)?;

                let counters = self.numbering_state.borrow_mut().advance(numbering_id, level, para.numbering_restart);
                let start_numbers = numbering.level_start_numbers;

                let level_idx = (level as usize).min(6);
                let format_str = &numbering.level_formats[level_idx];
                if format_str.is_empty() { return None; }

                let text = expand_numbering_format(format_str, &counters, numbering, &start_numbers);
                if text.is_empty() { return None; }
                text
            }
            HeadType::Bullet => {
                // Bullet: numbering_id(1-based)로 Bullet 참조
                let bullet_id = para_style.numbering_id;
                if bullet_id == 0 { return None; }
                let bullet = styles.bullets.get((bullet_id - 1) as usize)?;
                // U+FFFF는 이미지 글머리표 표시자 — 문자 렌더링 불가, 건너뜀
                if bullet.bullet_char == '\u{FFFF}' { return None; }
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
                    format!("{}{}{}",
                        if an.prefix_char != '\0' { an.prefix_char.to_string() } else { String::new() },
                        num_str,
                        if an.suffix_char != '\0' { an.suffix_char.to_string() } else { String::new() },
                    )
                } else {
                    num_str
                };

                // 각 줄의 텍스트에서 연속된 두 공백("  ")을 찾아 번호로 대체
                // HWP/HWPX 모두 AutoNumber 위치에 공백 placeholder 삽입
                for line in &mut composed.lines {
                    for run in &mut line.runs {
                        if let Some(pos) = run.text.find("  ") {
                            run.text = format!("{}{}{}", &run.text[..pos+1], num_str, &run.text[pos+1..]);
                            return; // 첫 번째 발견 시 처리 완료
                        }
                    }
                }
            }
        }
    }
}

/// HWP PUA 문자(0xF000~0xF0FF)를 표준 Unicode로 매핑
/// 기준: Wingdings 폰트 → Unicode 매핑 (alanwood.net/demos/wingdings.html)
/// HWP 글머리표는 Wingdings 폰트 문자를 PUA(0xF000+code)로 저장
pub(crate) fn map_pua_bullet_char(ch: char) -> char {
    let code = ch as u32;
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
        0xA0 => '\u{25AA}', // ▪ Black small square
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

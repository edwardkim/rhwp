//! 표 셀 내용 레이아웃 (세로쓰기, 셀 도형, 내장 표)

use crate::model::paragraph::Paragraph;
use crate::model::control::Control;
use crate::model::style::Alignment;
use crate::model::bin_data::BinDataContent;
use crate::model::table::VerticalAlign;
use super::super::render_tree::*;
use super::super::page_layout::LayoutRect;
use super::super::composer::{compose_paragraph, ComposedParagraph};
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, TextStyle, ShapeStyle};
use super::{LayoutEngine, CellContext, CellPathEntry};
use super::border_rendering::{build_row_col_x, collect_cell_borders, render_edge_borders, render_transparent_borders};
use super::text_measurement::{resolved_to_text_style, is_cjk_char, is_vertical_rotate_char, vertical_substitute_char};
use super::utils::{extract_shape_transform, find_bin_data};

impl LayoutEngine {
    /// 세로쓰기 셀의 텍스트를 수직 방향으로 배치한다.
    ///
    /// HWP 세로쓰기 규칙:
    /// - 텍스트 방향: 위→아래, 열(column)은 오른쪽→왼쪽
    /// - text_direction: 1=영문 눕힘(회전), 2=영문 세움(직립)
    /// - 정렬 매핑: Top→오른쪽(첫 열), Center→중앙, Bottom→왼쪽
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_vertical_cell_text(
        &self,
        tree: &mut PageRenderTree,
        cell_node: &mut RenderNode,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        inner_area: &LayoutRect,
        vertical_align: VerticalAlign,
        text_direction: u8,
        section_index: usize,
        table_meta: Option<(usize, usize)>,
        cell_idx: usize,
        enclosing_cell_ctx: Option<CellContext>,
    ) {
        // 1. line_seg 기반으로 composed lines를 열(column)로 변환
        //    세로쓰기에서 각 composed line = 하나의 열
        //    line_seg.line_height = 열 폭, line_seg.line_spacing = 열 간격
        struct CharInfo {
            ch: char,
            style: TextStyle,
            char_style_id: u32,
            para_style_id: u16,
            cell_para_index: usize,
            char_offset: usize,
            is_para_end: bool,
        }

        struct ColumnInfo {
            start_idx: usize,
            end_idx: usize,   // exclusive
            col_width: f64,   // line_height + line_spacing (px), 마지막 칼럼은 line_height만
            col_spacing: f64, // 항상 0 (line_spacing이 col_width에 흡수됨)
            total_height: f64,
            alignment: Alignment,
            absorbed_spacing: f64, // 흡수된 line_spacing (px) — 마지막 칼럼 후처리용
        }

        let get_alignment = |para_style_id: u16| -> Alignment {
            styles.para_styles.get(para_style_id as usize)
                .map(|s| s.alignment)
                .unwrap_or(Alignment::Left)
        };

        let mut chars: Vec<CharInfo> = Vec::new();
        let mut columns: Vec<ColumnInfo> = Vec::new();

        for (cp_idx, composed) in composed_paras.iter().enumerate() {
            let para = paragraphs.get(cp_idx);
            let alignment = get_alignment(composed.para_style_id);

            if composed.lines.is_empty() {
                // 빈 문단: 빈 열 추가 (개행)
                // 칼럼 너비 = line_height + line_spacing (전체 피치를 칼럼에 흡수)
                let ls = para.and_then(|p| p.line_segs.first());
                let spacing = ls.map(|l| hwpunit_to_px(l.line_spacing, self.dpi)).unwrap_or(0.0);
                columns.push(ColumnInfo {
                    start_idx: chars.len(),
                    end_idx: chars.len(),
                    col_width: ls.map(|l| hwpunit_to_px(l.line_height + l.line_spacing, self.dpi))
                        .unwrap_or(13.0),
                    col_spacing: 0.0,
                    total_height: 0.0,
                    alignment,
                    absorbed_spacing: spacing,
                });
                continue;
            }

            let mut char_offset = 0usize;
            for (line_idx, line) in composed.lines.iter().enumerate() {
                let ls = para.and_then(|p| p.line_segs.get(line_idx));
                // 칼럼 너비 = line_height + line_spacing (전체 피치 흡수)
                // 마지막 칼럼은 후처리로 line_spacing분 제거
                let col_width = ls.map(|l| hwpunit_to_px(l.line_height + l.line_spacing, self.dpi))
                    .unwrap_or(13.0);
                let col_spacing = 0.0;
                let absorbed_spacing = ls.map(|l| hwpunit_to_px(l.line_spacing, self.dpi)).unwrap_or(0.0);

                let col_start = chars.len();
                let mut col_height = 0.0;

                for run in &line.runs {
                    let text_style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                    for ch in run.text.chars() {
                        if ch == '\n' || ch == '\r' {
                            char_offset += 1;
                            continue;
                        }
                        let is_rotate = is_vertical_rotate_char(ch);
                        let needs_rotation = is_rotate
                            || (text_direction == 1 && !is_cjk_char(ch));
                        // 세로쓰기에서 구두점/기호만 반칸 advance (영문/숫자는 캐릭터 높이)
                        let half_advance = needs_rotation
                            || (!is_cjk_char(ch) && !ch.is_ascii_alphanumeric());
                        let advance = if half_advance {
                            text_style.font_size * 0.5
                        } else {
                            text_style.font_size
                        };
                        chars.push(CharInfo {
                            ch,
                            style: text_style.clone(),
                            char_style_id: run.char_style_id,
                            para_style_id: composed.para_style_id,
                            cell_para_index: cp_idx,
                            char_offset,
                            is_para_end: false,
                        });
                        col_height += advance;
                        char_offset += 1;
                    }
                }

                // 문단의 마지막 줄이면 마지막 글자에 is_para_end 표시
                if line_idx == composed.lines.len() - 1 {
                    if let Some(last) = chars.last_mut() {
                        if last.cell_para_index == cp_idx {
                            last.is_para_end = true;
                        }
                    }
                }

                columns.push(ColumnInfo {
                    start_idx: col_start,
                    end_idx: chars.len(),
                    col_width,
                    col_spacing,
                    total_height: col_height,
                    alignment,
                    absorbed_spacing,
                });
            }
        }

        if chars.is_empty() && columns.iter().all(|c| c.start_idx == c.end_idx) {
            return;
        }

        // 마지막 칼럼은 뒤에 간격이 불필요하므로 흡수된 line_spacing분 제거
        if let Some(last_col) = columns.last_mut() {
            last_col.col_width -= last_col.absorbed_spacing;
        }

        // 2. 열 배치 x좌표 계산 (오른쪽→왼쪽)
        //    total = col[0].w + col[0].s + col[1].w + col[1].s + ... + col[n-1].w
        let total_cols_width: f64 = if columns.is_empty() {
            0.0
        } else {
            columns.iter().map(|c| c.col_width).sum::<f64>()
                + columns[..columns.len() - 1].iter().map(|c| c.col_spacing).sum::<f64>()
        };

        // 열이 셀보다 넓으면 첫 열이 오른쪽 가장자리에서 시작하도록 클램핑
        let right_aligned = inner_area.x + inner_area.width - total_cols_width;
        let cols_x_start = match vertical_align {
            VerticalAlign::Top => right_aligned,
            VerticalAlign::Center => {
                let centered = inner_area.x + (inner_area.width - total_cols_width) / 2.0;
                centered.min(right_aligned)
            }
            VerticalAlign::Bottom => inner_area.x.min(right_aligned),
        };

        // 3. 각 글자를 TextLine + TextRun 노드로 생성
        let mut col_x = cols_x_start + total_cols_width;

        for col in &columns {
            col_x -= col.col_width;

            let free_space = (inner_area.height - col.total_height).max(0.0);
            let y_start = inner_area.y + match col.alignment {
                Alignment::Center | Alignment::Distribute => free_space / 2.0,
                Alignment::Right => free_space,
                _ => 0.0,
            };
            let mut char_y = y_start;
            let col_bottom = inner_area.y + inner_area.height;

            for i in col.start_idx..col.end_idx {
                let ci = &chars[i];
                let is_rotate = is_vertical_rotate_char(ci.ch);
                let needs_rotation = is_rotate
                    || (text_direction == 1 && !is_cjk_char(ci.ch));
                // 세로쓰기에서 구두점/기호만 반칸 advance (영문/숫자는 캐릭터 높이)
                let half_advance = needs_rotation
                    || (!is_cjk_char(ci.ch) && !ci.ch.is_ascii_alphanumeric());
                let advance = if half_advance {
                    ci.style.font_size * 0.5
                } else {
                    ci.style.font_size
                };

                // 열 높이 초과 시 렌더링 중단
                if char_y + advance > col_bottom + 0.5 {
                    break;
                }

                // 세로쓰기: 모든 문자를 칼럼 중앙에 전각 배치 (영문눕힘과 동일)
                let char_width = ci.style.font_size;

                let char_x = col_x + (col.col_width - char_width) / 2.0;
                // 기호 대체: 세로 형태 Unicode가 있으면 대체 문자를 사용 (회전 불필요)
                let (render_ch, rotation) = if needs_rotation {
                    if let Some(sub) = vertical_substitute_char(ci.ch) {
                        (sub, 0.0)
                    } else {
                        (ci.ch, 90.0)
                    }
                } else {
                    (ci.ch, 0.0)
                };

                let line_id = tree.next_id();
                let mut line_node = RenderNode::new(
                    line_id,
                    RenderNodeType::TextLine(TextLineNode::new(advance, advance * 0.85)),
                    BoundingBox::new(char_x, char_y, char_width, advance),
                );

                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: render_ch.to_string(),
                        style: ci.style.clone(),
                        char_shape_id: Some(ci.char_style_id),
                        para_shape_id: Some(ci.para_style_id),
                        section_index: Some(section_index),
                        para_index: Some(ci.cell_para_index),
                        char_start: Some(ci.char_offset),
                        cell_context: if let Some(ref ctx) = enclosing_cell_ctx {
                            let mut new_ctx = ctx.clone();
                            if let Some(last) = new_ctx.path.last_mut() {
                                last.cell_index = cell_idx;
                                last.cell_para_index = ci.cell_para_index;
                            }
                            Some(new_ctx)
                        } else {
                            table_meta.map(|(pi, ctrl_ci)| CellContext {
                                parent_para_index: pi,
                                path: vec![CellPathEntry {
                                    control_index: ctrl_ci,
                                    cell_index: cell_idx,
                                    cell_para_index: ci.cell_para_index,
                                    text_direction: 0,
                                }],
                            })
                        },
                        is_para_end: ci.is_para_end,
                        is_line_break_end: false,
                        rotation,
                        is_vertical: true,
                        char_overlap: None,
                        border_fill_id: styles.char_styles.get(ci.char_style_id as usize)
                            .map(|cs| cs.border_fill_id).unwrap_or(0),
                        baseline: advance * 0.85,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(char_x, char_y, char_width, advance),
                );

                line_node.children.push(run_node);
                cell_node.children.push(line_node);

                char_y += advance;
            }

            col_x -= col.col_spacing;
        }
    }


    /// 테이블 셀 내 도형(Shape) 컨트롤을 레이아웃한다.
    pub(crate) fn layout_cell_shape(
        &self,
        tree: &mut PageRenderTree,
        cell_node: &mut RenderNode,
        shape: &crate::model::shape::ShapeObject,
        inner_area: &LayoutRect,
        para_y: f64,
        para_alignment: Alignment,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
    ) {
        let child_common = shape.common();

        let child_w = hwpunit_to_px(child_common.width as i32, self.dpi);
        let child_h = hwpunit_to_px(child_common.height as i32, self.dpi);

        let (child_x, child_y) = if child_common.treat_as_char {
            // 인라인: 문단 정렬에 따라 배치
            let x = match para_alignment {
                Alignment::Center | Alignment::Distribute => {
                    inner_area.x + (inner_area.width - child_w).max(0.0) / 2.0
                }
                Alignment::Right => {
                    inner_area.x + (inner_area.width - child_w).max(0.0)
                }
                _ => inner_area.x,
            };
            (x, para_y)
        } else {
            // 셀 내 비-TAC 도형: horz_align/vert_align 속성 기반 배치
            use crate::model::shape::{HorzAlign, VertAlign};
            let h_offset = hwpunit_to_px(child_common.horizontal_offset as i32, self.dpi);
            let v_offset = hwpunit_to_px(child_common.vertical_offset as i32, self.dpi);
            let x = match child_common.horz_align {
                HorzAlign::Right | HorzAlign::Outside => {
                    inner_area.x + inner_area.width - child_w - h_offset
                }
                HorzAlign::Center => {
                    inner_area.x + (inner_area.width - child_w) / 2.0 + h_offset
                }
                _ => inner_area.x + h_offset,
            };
            let y = match child_common.vert_align {
                VertAlign::Bottom | VertAlign::Outside => {
                    inner_area.y + inner_area.height - child_h - v_offset
                }
                VertAlign::Center => {
                    inner_area.y + (inner_area.height - child_h) / 2.0 + v_offset
                }
                _ => inner_area.y + v_offset,
            };
            (x, y)
        };

        let empty_map = std::collections::HashMap::new();
        self.layout_shape_object(
            tree, cell_node, shape,
            child_x, child_y, child_w, child_h,
            0, 0, 0,
            styles, bin_data_content,
            &empty_map,
            &[],
        );
    }

    /// TextBox 내부에 포함된 표를 레이아웃한다.
    /// enclosing_ctx: (section_index, body_para_index, 상위 경로, 표의 컨트롤 인덱스)
    pub(crate) fn layout_embedded_table(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        container: &LayoutRect,
        y_start: f64,
        enclosing_ctx: Option<(usize, usize, &[CellPathEntry], usize)>,
        bin_data_content: &[BinDataContent],
        host_alignment: Alignment,
    ) -> f64 {
        if table.cells.is_empty() {
            return y_start;
        }

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // 열 폭 계산
        let mut col_widths = vec![0.0f64; col_count];
        for cell in &table.cells {
            if cell.col_span == 1 && (cell.col as usize) < col_count {
                let w = hwpunit_to_px(cell.width as i32, self.dpi);
                if w > col_widths[cell.col as usize] {
                    col_widths[cell.col as usize] = w;
                }
            }
        }
        for c in 0..col_count {
            if col_widths[c] <= 0.0 {
                col_widths[c] = container.width / col_count as f64;
            }
        }

        // 글상자 내부 표: 셀 너비 합이 컨테이너 폭을 초과하면 비례 축소
        let col_sum: f64 = col_widths.iter().sum();
        let max_w = {
            let common_w = hwpunit_to_px(table.common.width as i32, self.dpi);
            if common_w > 0.0 && common_w < container.width {
                common_w
            } else {
                container.width
            }
        };
        if col_sum > max_w + 1.0 {
            let scale = max_w / col_sum;
            for w in &mut col_widths {
                *w *= scale;
            }
        }

        // 행 높이 계산 (layout_table과 동일한 resolve_row_heights 사용)
        let row_heights = self.resolve_row_heights(table, col_count, row_count, None, styles);

        // 누적 위치 계산
        let mut col_x = vec![0.0f64; col_count + 1];
        for i in 0..col_count {
            col_x[i + 1] = col_x[i] + col_widths[i] + if i + 1 < col_count { cell_spacing } else { 0.0 };
        }
        let mut row_y = vec![0.0f64; row_count + 1];
        for i in 0..row_count {
            row_y[i + 1] = row_y[i] + row_heights[i] + if i + 1 < row_count { cell_spacing } else { 0.0 };
        }

        // 행별 열 위치 계산 (셀별 독립 너비 지원)
        let row_col_x = build_row_col_x(table, &col_widths, col_count, row_count, cell_spacing, self.dpi);

        let table_width = row_col_x.iter()
            .map(|rx| rx.last().copied().unwrap_or(0.0))
            .fold(col_x.last().copied().unwrap_or(0.0), f64::max);
        let table_height = row_y.last().copied().unwrap_or(0.0);
        // TAC 표: 호스트 문단 정렬에 따라 배치
        let table_x = match host_alignment {
            Alignment::Center | Alignment::Distribute => container.x + (container.width - table_width).max(0.0) / 2.0,
            Alignment::Right => container.x + (container.width - table_width).max(0.0),
            _ => container.x, // 왼쪽 정렬 (기본)
        };
        let table_y = y_start;

        // 엣지 기반 테두리 수집을 위한 그리드 생성
        use crate::model::style::BorderLine;
        let mut h_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; col_count]; row_count + 1];
        let mut v_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; row_count]; col_count + 1];

        // 표 노드 생성
        let table_id = tree.next_id();
        let mut table_node = RenderNode::new(
            table_id,
            RenderNodeType::Table(TableNode {
                row_count: table.row_count,
                col_count: table.col_count,
                border_fill_id: table.border_fill_id,
                section_index: None,
                para_index: None,
                control_index: None,
            }),
            BoundingBox::new(table_x, table_y, table_width, table_height),
        );

        // 표 배경 렌더링 (표 > 배경 > 색 > 면색)
        if table.border_fill_id > 0 {
            let tbl_idx = (table.border_fill_id as usize).saturating_sub(1);
            if let Some(tbl_bs) = styles.border_styles.get(tbl_idx) {
                self.render_cell_background(
                    tree, &mut table_node, Some(tbl_bs),
                    table_x, table_y, table_width, table_height,
                    bin_data_content,
                );
            }
        }

        // 각 셀 레이아웃
        for (cell_enum_idx, cell) in table.cells.iter().enumerate() {
            let c = cell.col as usize;
            let r = cell.row as usize;
            if c >= col_count || r >= row_count {
                continue;
            }

            let rcx = &row_col_x[r];
            let cell_x = table_x + rcx[c];
            let cell_y = table_y + row_y[r];
            let end_col = (c + cell.col_span as usize).min(col_count);
            let end_row = (r + cell.row_span as usize).min(row_count);
            let cell_w = rcx[end_col] - rcx[c];
            let cell_h = row_y[end_row] - row_y[r];

            let cell_id = tree.next_id();
            let mut cell_node = RenderNode::new(
                cell_id,
                RenderNodeType::TableCell(TableCellNode {
                    col: cell.col,
                    row: cell.row,
                    col_span: cell.col_span,
                    row_span: cell.row_span,
                    border_fill_id: cell.border_fill_id,
                    text_direction: cell.text_direction,
                    clip: false,
                    model_cell_index: Some(cell_enum_idx as u32),
                }),
                BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
            );

            // 셀 BorderFill
            let border_style = if cell.border_fill_id > 0 {
                let idx = (cell.border_fill_id as usize).saturating_sub(1);
                styles.border_styles.get(idx)
            } else {
                None
            };

            // 셀 배경
            let fill_color = border_style.and_then(|bs| bs.fill_color);
            let gradient = border_style.and_then(|bs| bs.gradient.clone());
            if fill_color.is_some() || gradient.is_some() {
                let rect_id = tree.next_id();
                let rect_node = RenderNode::new(
                    rect_id,
                    RenderNodeType::Rectangle(RectangleNode::new(
                        0.0,
                        ShapeStyle {
                            fill_color,
                            stroke_color: None,
                            stroke_width: 0.0,
                            ..Default::default()
                        },
                        gradient,
                    )),
                    BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
                );
                cell_node.children.push(rect_node);
            }

            // 셀 테두리를 엣지 그리드에 수집
            if let Some(bs) = border_style {
                collect_cell_borders(
                    &mut h_edges, &mut v_edges,
                    c, r, cell.col_span as usize, cell.row_span as usize,
                    &bs.borders,
                );
            }

            // 셀 패딩 (apply_inner_margin 고려)
            let (mut pad_left, mut pad_right, pad_top, _pad_bottom) = self.resolve_cell_padding(cell, table);

            // 셀 내 문단 레이아웃
            let composed_paras: Vec<_> = cell.paragraphs.iter()
                .map(|p| compose_paragraph(p))
                .collect();

            // 텍스트 오버플로우 시 좌우 패딩 축소
            let (new_pl, new_pr) = self.shrink_cell_padding_for_overflow(
                pad_left, pad_right, cell_w, &composed_paras, &cell.paragraphs, styles,
            );
            pad_left = new_pl;
            pad_right = new_pr;

            let inner_x = cell_x + pad_left;
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let inner_area = LayoutRect {
                x: inner_x,
                y: cell_y + pad_top,
                width: inner_width,
                height: cell_h,
            };

            let mut para_y = cell_y + pad_top;
            let para_count = composed_paras.len();
            let cell_idx = cell_enum_idx;
            for (pidx, (composed, para)) in composed_paras.iter().zip(cell.paragraphs.iter()).enumerate() {
                // enclosing context가 있으면 글상자 경로 + 표 셀 경로를 합성
                let cell_ctx = enclosing_ctx.map(|(sec_idx, para_idx, parent_path, table_ci)| {
                    let mut path = parent_path.to_vec();
                    path.push(CellPathEntry {
                        control_index: table_ci,
                        cell_index: cell_idx,
                        cell_para_index: pidx,
                        text_direction: cell.text_direction,
                    });
                    (sec_idx, para_idx, CellContext {
                        parent_para_index: para_idx,
                        path,
                    })
                });
                let (sec_for_layout, para_for_layout, ctx) = match cell_ctx {
                    Some((s, p, c)) => (s, pidx, Some(c)),
                    None => (0, 0, None),
                };
                para_y = self.layout_composed_paragraph(
                    tree,
                    &mut cell_node,
                    composed,
                    styles,
                    &inner_area,
                    para_y,
                    0,
                    composed.lines.len(),
                    sec_for_layout, para_for_layout, ctx,
                    pidx + 1 == para_count,
                    0.0,
                    None, Some(para), None,
                    None,  // 셀 컨텍스트 — wrap zone 무관
                );

                // 셀 내 그림/도형 컨트롤 렌더링
                for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Picture(pic) => {
                            let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            // 셀 내부에 맞추어 크기 제한
                            let fit_w = pic_w.min(inner_width);
                            let fit_h = if pic_w > 0.0 { pic_h * (fit_w / pic_w) } else { pic_h };
                            // TAC: 문단 시작 위치 (표의 왼쪽 상단)
                            let pic_x = inner_x;
                            // vpos 기반 y 위치: LINE_SEG의 vertical_pos 사용
                            let pic_y = if let Some(first_ls) = para.line_segs.first() {
                                cell_y + pad_top + hwpunit_to_px(first_ls.vertical_pos, self.dpi)
                            } else {
                                para_y - fit_h
                            };

                            let bin_id = pic.image_attr.bin_data_id;
                            let img_data = find_bin_data(bin_data_content, bin_id)
                                .map(|bd| bd.data.clone());
                            let img_node_id = tree.next_id();
                            let img_node = RenderNode::new(
                                img_node_id,
                                RenderNodeType::Image(ImageNode {
                                    bin_data_id: bin_id,
                                    data: img_data,
                                    section_index: None,
                                    para_index: None,
                                    control_index: Some(ctrl_idx),
                                    fill_mode: None,
                                    original_size: None,
                                    transform: extract_shape_transform(&pic.shape_attr),
                                    crop: None,
                                    original_size_hu: None,
                                    effect: pic.image_attr.effect,
                                    brightness: pic.image_attr.brightness,
                                    contrast: pic.image_attr.contrast,
                                    text_wrap: None,
                                }),
                                BoundingBox::new(pic_x, pic_y, fit_w, fit_h),
                            );
                            cell_node.children.push(img_node);
                        }
                        _ => {}
                    }
                }
            }

            table_node.children.push(cell_node);
        }

        // 엣지 기반 테두리 렌더링
        table_node.children.extend(render_edge_borders(
            tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
        ));
        if self.show_transparent_borders.get() {
            table_node.children.extend(render_transparent_borders(
                tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
            ));
        }

        parent.children.push(table_node);
        table_y + table_height
    }
}

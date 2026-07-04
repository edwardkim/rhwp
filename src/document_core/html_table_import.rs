//! HTML 표 파싱 + BorderFill 생성 + 이미지 파싱 관련 native 메서드

use super::helpers::*;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::common_obj_offsets;
use crate::renderer::style_resolver::resolve_styles;

impl DocumentCore {
    pub(crate) fn parse_table_html(&mut self, paragraphs: &mut Vec<Paragraph>, table_html: &str) {
        use crate::model::control::Control;
        use crate::model::table::{Cell, Table, TablePageBreak};

        // --- 1. HTML 파싱: 행/셀 구조 추출 ---
        let table_lower = table_html.to_lowercase();

        struct ParsedCell {
            col_span: u16,
            row_span: u16,
            width_pt: f64,
            height_pt: f64,
            padding_pt: [f64; 4],          // left, right, top, bottom
            border_widths_pt: [f64; 4],    // left, right, top, bottom
            border_colors: [u32; 4],       // BGR
            border_styles: [u8; 4],        // 0=none, 1=solid, 2=dashed, 3=dotted, 4=double
            background_color: Option<u32>, // BGR
            content_html: String,
            is_header: bool,
            vertical_align: u8, // 0=top, 1=center, 2=bottom
        }

        let mut parsed_rows: Vec<Vec<ParsedCell>> = Vec::new();

        let mut pos = 0;
        while let Some(tr_start) = table_lower[pos..].find("<tr") {
            let tr_abs = pos + tr_start;
            let tr_end = find_closing_tag(table_html, tr_abs, "tr");
            let tr_inner = &table_html[tr_abs..tr_end.min(table_html.len())];
            let tr_inner_lower = tr_inner.to_lowercase();

            let mut row_cells: Vec<ParsedCell> = Vec::new();

            // <td>와 <th>를 출현 순서대로 처리
            let mut td_pos = 0;
            loop {
                let td_match = tr_inner_lower[td_pos..].find("<td");
                let th_match = tr_inner_lower[td_pos..].find("<th");

                let (tag_offset, is_th) = match (td_match, th_match) {
                    (Some(a), Some(b)) => {
                        if a <= b {
                            (a, false)
                        } else {
                            (b, true)
                        }
                    }
                    (Some(a), None) => (a, false),
                    (None, Some(b)) => (b, true),
                    (None, None) => break,
                };

                let cell_abs = td_pos + tag_offset;
                let tag_name = if is_th { "th" } else { "td" };

                // <td ...> 태그에서 속성 추출
                if let Some(gt) = tr_inner[cell_abs..].find('>') {
                    let tag_str = &tr_inner[cell_abs..cell_abs + gt + 1];

                    // colspan / rowspan 파싱
                    let col_span = parse_html_attr_u16(tag_str, "colspan").unwrap_or(1).max(1);
                    let row_span = parse_html_attr_u16(tag_str, "rowspan").unwrap_or(1).max(1);

                    // 인라인 style 파싱
                    let css = parse_inline_style(tag_str);
                    let css_lower = css.to_lowercase();

                    // 크기 파싱
                    let width_pt = parse_css_dimension_pt(&css_lower, "width");
                    let height_pt = parse_css_dimension_pt(&css_lower, "height");

                    // 패딩 파싱
                    let padding_pt = parse_css_padding_pt(&css_lower);

                    // 테두리 파싱 (left, right, top, bottom)
                    let mut border_widths_pt = [0.0f64; 4];
                    let mut border_colors = [0u32; 4]; // black
                    let mut border_styles = [1u8; 4]; // solid default

                    // 축약형 border 먼저
                    if let Some(bval) = parse_css_value(&css_lower, "border") {
                        let (w, c, s) = parse_css_border_shorthand(&bval);
                        for i in 0..4 {
                            border_widths_pt[i] = w;
                            border_colors[i] = c;
                            border_styles[i] = s;
                        }
                    }
                    // 개별 방향 오버라이드
                    let sides = ["border-left", "border-right", "border-top", "border-bottom"];
                    for (i, side) in sides.iter().enumerate() {
                        if let Some(bval) = parse_css_value(&css_lower, side) {
                            let (w, c, s) = parse_css_border_shorthand(&bval);
                            border_widths_pt[i] = w;
                            border_colors[i] = c;
                            border_styles[i] = s;
                        }
                    }

                    // 배경색
                    let background_color = parse_css_value(&css_lower, "background-color")
                        .or_else(|| parse_css_value(&css_lower, "background"))
                        .and_then(|v| css_color_to_hwp_bgr(&v));

                    // 수직 정렬 (0=미지정, 1=center, 2=bottom, 3=명시적 top)
                    let vertical_align =
                        match parse_css_value(&css_lower, "vertical-align").as_deref() {
                            Some("middle") | Some("center") => 1u8,
                            Some("bottom") => 2u8,
                            Some("top") => 3u8, // 명시적 top
                            _ => 0u8,           // 미지정 → Center (HWP 기본)
                        };

                    // 셀 내용 HTML 추출
                    let content_start = cell_abs + gt + 1;
                    let close_tag = format!("</{}>", tag_name);
                    let content_end =
                        if let Some(close) = tr_inner_lower[content_start..].find(&close_tag) {
                            content_start + close
                        } else {
                            tr_inner.len()
                        };
                    let content_html = tr_inner[content_start..content_end].to_string();

                    row_cells.push(ParsedCell {
                        col_span,
                        row_span,
                        width_pt,
                        height_pt,
                        padding_pt,
                        border_widths_pt,
                        border_colors,
                        border_styles,
                        background_color,
                        content_html,
                        is_header: is_th,
                        vertical_align,
                    });

                    td_pos = content_end + close_tag.len();
                } else {
                    break;
                }
            }

            if !row_cells.is_empty() {
                parsed_rows.push(row_cells);
            }
            pos = tr_end;
        }

        if parsed_rows.is_empty() {
            return;
        }

        // --- 2. 그리드 정규화: 실제 col 인덱스 계산 ---
        let row_count = parsed_rows.len() as u16;
        // colspan 합산으로 최대 열 수 추정
        let mut max_cols: usize = 0;
        for row in &parsed_rows {
            let sum: usize = row.iter().map(|c| c.col_span as usize).sum();
            if sum > max_cols {
                max_cols = sum;
            }
        }
        max_cols = max_cols.max(1);
        // rowspan 처리를 위한 점유 그리드
        let grid_rows = row_count as usize + 16;
        let grid_cols = max_cols + 16;
        let mut occupied = vec![vec![false; grid_cols]; grid_rows];

        struct CellPos {
            row: u16,
            col: u16,
            col_span: u16,
            row_span: u16,
            parsed_row: usize,
            parsed_col: usize,
        }

        let mut cell_positions: Vec<CellPos> = Vec::new();
        let mut actual_col_count: u16 = 0;

        for (ri, row) in parsed_rows.iter().enumerate() {
            let mut col_cursor: usize = 0;
            for (ci, cell) in row.iter().enumerate() {
                // 이미 점유된 위치 건너뛰기
                while col_cursor < grid_cols && occupied[ri][col_cursor] {
                    col_cursor += 1;
                }
                let col = col_cursor as u16;

                // 점유 표시
                for dr in 0..cell.row_span as usize {
                    for dc in 0..cell.col_span as usize {
                        let r = ri + dr;
                        let c = col_cursor + dc;
                        if r < grid_rows && c < grid_cols {
                            occupied[r][c] = true;
                        }
                    }
                }

                cell_positions.push(CellPos {
                    row: ri as u16,
                    col,
                    col_span: cell.col_span,
                    row_span: cell.row_span,
                    parsed_row: ri,
                    parsed_col: ci,
                });

                let end_col = col + cell.col_span;
                if end_col > actual_col_count {
                    actual_col_count = end_col;
                }
                col_cursor += cell.col_span as usize;
            }
        }

        let col_count = actual_col_count.max(1);

        // --- 3. 셀 크기 계산 ---
        let default_page_width: u32 = 42520; // A4 좌우 여백 제외
        let default_col_width = default_page_width / col_count as u32;
        let default_row_height: u32 = 1000;

        // 열별 폭 (CSS 지정 우선, 없으면 균등 분할)
        let mut col_widths = vec![0u32; col_count as usize];
        for cp in &cell_positions {
            if cp.col_span == 1 {
                let pc = &parsed_rows[cp.parsed_row][cp.parsed_col];
                if pc.width_pt > 0.0 {
                    let w = (pc.width_pt * 100.0).round() as u32;
                    if w > col_widths[cp.col as usize] {
                        col_widths[cp.col as usize] = w;
                    }
                }
            }
        }
        for w in col_widths.iter_mut() {
            if *w == 0 {
                *w = default_col_width;
            }
        }

        // 행별 높이
        let mut row_heights = vec![0u32; row_count as usize];
        for cp in &cell_positions {
            if cp.row_span == 1 {
                let pc = &parsed_rows[cp.parsed_row][cp.parsed_col];
                if pc.height_pt > 0.0 {
                    let h = (pc.height_pt * 100.0).round() as u32;
                    if h > row_heights[cp.row as usize] {
                        row_heights[cp.row as usize] = h;
                    }
                }
            }
        }
        for h in row_heights.iter_mut() {
            if *h == 0 {
                *h = default_row_height;
            }
        }

        // --- 4. BorderFill 생성 및 Cell 구조체 조립 ---
        let mut cells: Vec<Cell> = Vec::new();
        let mut has_header_row = false;

        for cp in &cell_positions {
            let pc = &parsed_rows[cp.parsed_row][cp.parsed_col];

            // 셀 폭/높이 (병합 고려)
            let cell_width: u32 = (cp.col..cp.col + cp.col_span)
                .map(|c| {
                    col_widths
                        .get(c as usize)
                        .copied()
                        .unwrap_or(default_col_width)
                })
                .sum();
            let cell_height: u32 = (cp.row..cp.row + cp.row_span)
                .map(|r| {
                    row_heights
                        .get(r as usize)
                        .copied()
                        .unwrap_or(default_row_height)
                })
                .sum();

            // BorderFill 생성/재사용
            let border_fill_id = self.create_border_fill_from_css(
                &pc.border_widths_pt,
                &pc.border_colors,
                &pc.border_styles,
                pc.background_color,
            );

            // 패딩 (pt → HWPUNIT16, CSS 미지정 시 기본 1.4mm ≈ 397 HWPUNIT)
            let default_pad: f64 = 141.0; // ~0.5mm HWPUNIT
            let padding = crate::model::Padding {
                left: if pc.padding_pt[0] > 0.01 {
                    (pc.padding_pt[0] * 100.0).round() as i16
                } else {
                    default_pad as i16
                },
                right: if pc.padding_pt[1] > 0.01 {
                    (pc.padding_pt[1] * 100.0).round() as i16
                } else {
                    default_pad as i16
                },
                top: if pc.padding_pt[2] > 0.01 {
                    (pc.padding_pt[2] * 100.0).round() as i16
                } else {
                    default_pad as i16
                },
                bottom: if pc.padding_pt[3] > 0.01 {
                    (pc.padding_pt[3] * 100.0).round() as i16
                } else {
                    default_pad as i16
                },
            };

            // 셀 내용 파싱
            // &nbsp; 등 HTML 엔티티를 디코딩한 후 공백만 남으면 빈 셀로 처리
            let cell_paragraphs = if pc.content_html.trim().is_empty()
                || html_to_plain_text(&pc.content_html).is_empty()
            {
                vec![Paragraph::new_empty()]
            } else {
                let parsed = self.parse_html_to_paragraphs(&pc.content_html);
                if parsed.is_empty()
                    || parsed
                        .iter()
                        .all(|p| p.text.trim().is_empty() && p.controls.is_empty())
                {
                    vec![Paragraph::new_empty()]
                } else {
                    parsed
                }
            };

            // 셀 문단의 para_shape_id (DIFF-3 수정)
            // 기본 "본문" ParaShape (id=0) 사용 — 유효한 참조를 보장
            let cell_para_shape_id: u16 = 0;

            // 셀 문단 보정: char_count_msb, char_count, para_shape_id, raw_header_extra, line_segs
            let mut cell_paragraphs = cell_paragraphs;
            for cp_para in &mut cell_paragraphs {
                cp_para.char_count_msb = true; // 셀 문단은 항상 MSB 설정
                                               // char_count에 문단끝 마커(+1) 포함
                let text_chars = cp_para.text.chars().count() as u32;
                cp_para.char_count = text_chars + 1;

                // para_shape_id: 기본 "본문" ParaShape 사용 (DIFF-3)
                cp_para.para_shape_id = cell_para_shape_id;

                // DIFF-2: char_shapes가 비어있으면 기본 CharShapeRef 추가
                // 모든 셀 문단은 최소 1개의 명시적 CharShapeRef를 가져야 함
                if cp_para.char_shapes.is_empty() {
                    let base_cs_id = if !self.document.doc_info.char_shapes.is_empty() {
                        0u32
                    } else {
                        0u32
                    };
                    cp_para
                        .char_shapes
                        .push(crate::model::paragraph::CharShapeRef {
                            start_pos: 0,
                            char_shape_id: base_cs_id,
                        });
                }

                // raw_header_extra에 instance_id = 0x80000000 설정
                if cp_para.raw_header_extra.len() >= 10 {
                    cp_para.raw_header_extra[6..10].copy_from_slice(&0x80000000u32.to_le_bytes());
                } else {
                    let mut rhe = vec![0u8; 10];
                    let n_cs = cp_para.char_shapes.len() as u16;
                    rhe[0..2].copy_from_slice(&n_cs.to_le_bytes());
                    // [2..4] n_range_tags = 0
                    let n_ls = cp_para.line_segs.len().max(1) as u16;
                    rhe[4..6].copy_from_slice(&n_ls.to_le_bytes());
                    rhe[6..10].copy_from_slice(&0x80000000u32.to_le_bytes());
                    cp_para.raw_header_extra = rhe;
                }

                // line_segs: 폰트 크기 기반 높이 계산
                let font_size = cp_para
                    .char_shapes
                    .first()
                    .and_then(|cs| {
                        self.document
                            .doc_info
                            .char_shapes
                            .get(cs.char_shape_id as usize)
                    })
                    .map(|cs| cs.base_size.max(400))
                    .unwrap_or(1000);
                let line_h = font_size;
                let text_h = font_size;
                let baseline = (font_size as f64 * 0.85) as i32;
                let spacing = (font_size as f64 * 0.6) as i32;
                // seg_width: 셀 폭에서 좌우 패딩을 뺀 텍스트 영역 폭
                let seg_w = (cell_width as i32) - (padding.left as i32) - (padding.right as i32);
                // tag(flags): bit 17(first segment) + bit 18(last segment).
                let line_tag: u32 = crate::model::paragraph::LineSeg::TAG_SINGLE_SEGMENT_LINE;

                if cp_para.line_segs.is_empty() {
                    cp_para.line_segs.push(crate::model::paragraph::LineSeg {
                        text_start: 0,
                        line_height: line_h,
                        text_height: text_h,
                        baseline_distance: baseline,
                        line_spacing: spacing,
                        segment_width: seg_w,
                        tag: line_tag,
                        ..Default::default()
                    });
                } else {
                    for ls in &mut cp_para.line_segs {
                        if ls.line_height < font_size {
                            ls.line_height = line_h;
                            ls.text_height = text_h;
                            ls.baseline_distance = baseline;
                            ls.line_spacing = spacing;
                        }
                        if ls.segment_width == 0 {
                            ls.segment_width = seg_w;
                        }
                        if ls.tag == 0 {
                            ls.tag = line_tag;
                        }
                    }
                }
            }

            if pc.is_header {
                has_header_row = true;
            }

            // list_header_width_ref: is_header면 bit 2 설정
            let lh_width_ref: u16 = if pc.is_header { 0x04 } else { 0 };

            // vertical_align → VerticalAlign enum
            // CSS에서 지정하지 않으면 기본값 Center (정상 HWP 파일 패턴)
            let v_align = match pc.vertical_align {
                0 => crate::model::table::VerticalAlign::Center, // CSS 미지정 → Center (HWP 기본)
                1 => crate::model::table::VerticalAlign::Center,
                2 => crate::model::table::VerticalAlign::Bottom,
                3 => crate::model::table::VerticalAlign::Top, // 명시적 top
                _ => crate::model::table::VerticalAlign::Center,
            };

            // raw_list_extra: 13바이트 (첫 4바이트 = 셀 폭 u32, 나머지 0)
            // 정상 파일에서 raw_list_extra[0..4] = cell_width (u32)
            let mut raw_list_extra = vec![0u8; 13];
            raw_list_extra[0..4].copy_from_slice(&cell_width.to_le_bytes());

            cells.push(Cell {
                col: cp.col,
                row: cp.row,
                col_span: cp.col_span,
                row_span: cp.row_span,
                width: cell_width,
                height: cell_height,
                padding,
                border_fill_id,
                paragraphs: cell_paragraphs,
                is_header: pc.is_header,
                list_header_width_ref: lh_width_ref,
                vertical_align: v_align,
                raw_list_extra,
                ..Default::default()
            });
        }

        // 행 우선 순서로 정렬
        cells.sort_by(|a, b| a.row.cmp(&b.row).then(a.col.cmp(&b.col)));

        // --- 5. Table 구조체 조립 ---
        let total_width: u32 = col_widths.iter().sum();
        let total_height: u32 = row_heights.iter().sum();

        // table.attr: 기존 문서의 표와 동일한 패턴 사용
        // 0x082A2311 = treat_as_char | vert_rel_to=Para | horz_rel_to=Column |
        //              allow_overlap | width_criterion | various layout flags
        // 정상 HWP 파일의 모든 표에서 사용되는 표준값
        let table_attr: u32 = 0x082A2311;

        // raw_ctrl_data: CommonObjAttr 전체 (attr 포함, parse_common_obj_attr 정합)
        // [0..4] attr, [4..8] vertical_offset, [8..12] horizontal_offset,
        // [12..16] width, [16..20] height, [20..24] z_order,
        // [24..26] margin.left, [26..28] margin.right,
        // [28..30] margin.top, [30..32] margin.bottom,
        // [32..36] instance_id, [36..38] desc_len(=0)
        let outer_margin: i16 = 283; // 바깥 여백 ~1mm
        let mut raw_ctrl_data = vec![0u8; 38]; // 32(base) + 2(desc_len) + 4(extra)
        raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&table_attr.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
        // 바깥 여백 (left, right, top, bottom)
        raw_ctrl_data[common_obj_offsets::MARGIN_LEFT].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
            .copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_TOP].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
            .copy_from_slice(&outer_margin.to_le_bytes());
        // [32..36] instance_id (DIFF-7 수정: 해시 기반 유니크 값 생성)
        // 정상 HWP 파일에서는 instance_id가 고유한 비-0 값을 가짐
        let instance_id: u32 = {
            // 행/열 수, 셀 수, 총 폭/높이를 조합한 간단한 해시
            let mut h: u32 = 0x7c150000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            h = h.wrapping_add(total_height.wrapping_mul(0x1b));
            h ^= cells.len() as u32 * 0x4b69;
            if h == 0 {
                h = 0x7c154b69;
            } // 절대 0이 되지 않도록
            h
        };
        raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());
        // [36..38] desc_len = 0

        // row_sizes: 각 행의 셀 수
        let row_sizes: Vec<i16> = (0..row_count)
            .map(|r| cells.iter().filter(|c| c.row == r).count() as i16)
            .collect();

        // 표 전체 기본 BorderFill: 정상 파일에서 모든 표가 border_fill_id=3 사용
        // border_fill_id는 1-based (DocInfo.border_fills 인덱스 + 1)
        let table_border_fill_id = if self.document.doc_info.border_fills.len() >= 3 {
            3u16
        } else if !self.document.doc_info.border_fills.is_empty() {
            1u16
        } else {
            0u16
        };

        // HTML <table> CSS에서 표 패딩 파싱
        let table_style =
            parse_inline_style(&table_html[..table_html.find('>').unwrap_or(table_html.len()) + 1])
                .to_lowercase();
        let table_padding_pt = parse_css_padding_pt(&table_style);
        // 기본값: L:510 R:510 T:141 B:141 (정상 HWP 파일 패턴)
        let table_padding = crate::model::Padding {
            left: if table_padding_pt[0] > 0.01 {
                (table_padding_pt[0] * 100.0).round() as i16
            } else {
                510
            },
            right: if table_padding_pt[1] > 0.01 {
                (table_padding_pt[1] * 100.0).round() as i16
            } else {
                510
            },
            top: if table_padding_pt[2] > 0.01 {
                (table_padding_pt[2] * 100.0).round() as i16
            } else {
                141
            },
            bottom: if table_padding_pt[3] > 0.01 {
                (table_padding_pt[3] * 100.0).round() as i16
            } else {
                141
            },
        };

        // raw_table_record_attr: 정상 파일 패턴 기반 (DIFF-5 수정)
        // bit 1: 셀 분리 금지 (항상 설정), bit 2: repeat_header
        // bit 26: 추가 레이아웃 속성
        // 정상 HWP 파일에서 모든 표는 bit 1 (셀 분리 금지) 이 항상 설정됨
        let tbl_rec_attr: u32 = 0x04000006; // bit 1(셀분리금지) + bit 2 + bit 26

        let outer_margin: i16 = 283; // 바깥 여백 기본값 ~1mm
        let mut table = Table {
            attr: table_attr,
            row_count,
            col_count,
            cell_spacing: 0,
            padding: table_padding,
            row_sizes,
            border_fill_id: table_border_fill_id,
            zones: Vec::new(),
            cells,
            cell_grid: Vec::new(),
            page_break: TablePageBreak::None,
            repeat_header: has_header_row,
            caption: None,
            common: Default::default(),
            outer_margin_left: outer_margin,
            outer_margin_right: outer_margin,
            outer_margin_top: outer_margin,
            outer_margin_bottom: outer_margin,
            raw_ctrl_data,
            raw_table_record_attr: tbl_rec_attr,
            raw_table_record_extra: vec![0u8; 2], // 표준 추가 2바이트
            dirty: true,
            local_resize_rows: Vec::new(),
            local_resize_cols: Vec::new(),
            local_resize_cell_widths: Vec::new(),
            local_resize_cell_heights: Vec::new(),
        };
        table.rebuild_grid();

        // --- 6. Table Control을 포함하는 Paragraph 생성 ---
        // 제어문자는 text에 포함하지 않음 (serialize_para_text가 controls에서 생성)
        let default_char_shape_id = if !self.document.doc_info.char_shapes.is_empty() {
            0u32
        } else {
            self.document
                .doc_info
                .char_shapes
                .push(crate::model::style::CharShape::default());
            0
        };

        // 표 문단의 para_shape_id: 기존 문서의 표 문단에서 사용하는 값 탐색
        // 정상 파일에서 표 문단은 ps_id=1 사용 (기본 "본문" 스타일)
        let table_para_shape_id = {
            let mut found_ps = 0u16;
            'outer: for section in &self.document.sections {
                for para in &section.paragraphs {
                    for ctrl in &para.controls {
                        if let Control::Table(_) = ctrl {
                            found_ps = para.para_shape_id;
                            break 'outer;
                        }
                    }
                }
            }
            if found_ps == 0 && self.document.doc_info.para_shapes.len() > 1 {
                1u16 // 기본 "본문" ParaShape
            } else {
                found_ps
            }
        };

        // raw_header_extra: [0..2] n_char_shapes, [2..4] n_range_tags, [4..6] n_line_segs, [6..10] instance_id
        // 정상 파일에서 표 문단의 instance_id = 0x80000000
        let mut table_raw_header_extra = vec![0u8; 10];
        table_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes()); // n_char_shapes=1
                                                                           // [2..4] n_range_tags=0, [4..6] n_line_segs=1
        table_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());
        // [6..10] instance_id=0x80000000
        table_raw_header_extra[6..10].copy_from_slice(&0x80000000u32.to_le_bytes());

        let table_para = Paragraph {
            text: String::new(),
            char_count: 9, // 확장 제어문자(8 code units) + 문단끝(1 code unit)
            control_mask: 0x00000800, // DrawTableObject (bit 11)
            char_offsets: vec![],
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            line_segs: vec![crate::model::paragraph::LineSeg {
                text_start: 0,
                line_height: total_height.min(i32::MAX as u32) as i32,
                text_height: total_height.min(i32::MAX as u32) as i32,
                baseline_distance: (total_height as f64 * 0.85).min(i32::MAX as f64) as i32,
                line_spacing: 600,
                segment_width: total_width.min(i32::MAX as u32) as i32,
                tag: crate::model::paragraph::LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            para_shape_id: table_para_shape_id,
            style_id: 0,
            controls: vec![Control::Table(Box::new(table))],
            ctrl_data_records: vec![None],
            has_para_text: true,
            raw_header_extra: table_raw_header_extra,
            // 표 문단 자체의 MSB는 false (기존 HWP 문서 패턴)
            // FIX-1은 빈 문서 케이스에만 해당, 내용이 있는 문서에서는 false
            // 셀 내부 문단의 MSB는 true (셀 보정 코드에서 설정)
            char_count_msb: false,
            ..Default::default()
        };

        paragraphs.push(table_para);
    }

    /// CSS 테두리/배경 정보로 BorderFill을 생성하고 DocInfo에 등록한다.
    /// 동일한 BorderFill이 이미 있으면 기존 ID를 반환한다.
    pub(crate) fn create_border_fill_from_css(
        &mut self,
        border_widths_pt: &[f64; 4],
        border_colors: &[u32; 4],
        border_styles: &[u8; 4],
        background_color: Option<u32>,
    ) -> u16 {
        use crate::model::style::{
            BorderFill, BorderLine, BorderLineType, CenterLine, DiagonalLine, Fill, FillType,
            SolidFill,
        };

        let mut borders = [BorderLine::default(); 4];
        for i in 0..4 {
            if border_widths_pt[i] > 0.01 {
                borders[i] = BorderLine {
                    line_type: match border_styles[i] {
                        0 => BorderLineType::None,
                        1 => BorderLineType::Solid,
                        2 => BorderLineType::Dash,
                        3 => BorderLineType::Dot,
                        4 => BorderLineType::Double,
                        _ => BorderLineType::Solid,
                    },
                    width: css_border_width_to_hwp(border_widths_pt[i]),
                    color: border_colors[i],
                };
            } else {
                borders[i] = BorderLine {
                    line_type: BorderLineType::None,
                    width: 0,
                    color: 0,
                };
            }
        }

        let fill = if let Some(bg) = background_color {
            if bg != 0xFFFFFF {
                Fill {
                    fill_type: FillType::Solid,
                    solid: Some(SolidFill {
                        background_color: bg,
                        pattern_color: 0,
                        pattern_type: -1_i32, // 무늬 없음
                    }),
                    ..Default::default()
                }
            } else {
                Fill::default()
            }
        } else {
            Fill::default()
        };

        let bf = BorderFill {
            raw_data: None,
            attr: 0,
            borders,
            diagonal: DiagonalLine::default(),
            center_line: CenterLine::None,
            fill,
        };

        // 기존 BorderFill에서 동일한 항목 검색
        for (i, existing) in self.document.doc_info.border_fills.iter().enumerate() {
            if border_fills_equal(existing, &bf) {
                return (i + 1) as u16; // border_fill_id는 1-based
            }
        }

        // 새로 추가
        self.document.doc_info.border_fills.push(bf);
        self.document.doc_info.raw_stream_dirty = true;
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        self.document.doc_info.border_fills.len() as u16
    }

    /// JSON에서 border/fill 속성을 파싱하여 BorderFill을 생성/재사용한다.
    /// 프론트엔드 글자 테두리/배경 대화상자에서 호출된다.
    pub(crate) fn create_border_fill_from_json(&mut self, json: &str) -> u16 {
        use crate::model::style::{
            BorderFill, BorderLine, CenterLine, DiagonalLine, Fill, FillType, SolidFill,
        };

        fn json_diag_bits(json: &str, key: &str) -> Option<u16> {
            json_i32(json, key)
                .map(|v| (v as u16) & 0x07)
                .or_else(|| json_bool(json, key).map(|v| if v { 0b010 } else { 0 }))
        }

        fn json_center_line(json: &str) -> Option<CenterLine> {
            json_str(json, "centerLine").map(|value| {
                let normalized = value.trim().to_ascii_uppercase();
                match normalized.as_str() {
                    "VERTICAL" | "HORIZONTAL_BAR" => CenterLine::Vertical,
                    "HORIZONTAL" | "VERTICAL_BAR" => CenterLine::Horizontal,
                    "CROSS" => CenterLine::Cross,
                    _ => CenterLine::None,
                }
            })
        }

        let mut bf = json_u32(json, "borderFillId")
            .and_then(|id| {
                if id == 0 {
                    None
                } else {
                    self.document
                        .doc_info
                        .border_fills
                        .get((id - 1) as usize)
                        .cloned()
                }
            })
            .unwrap_or(BorderFill {
                raw_data: None,
                attr: 0,
                borders: [BorderLine::default(); 4],
                diagonal: DiagonalLine::default(),
                center_line: CenterLine::None,
                fill: Fill::default(),
            });
        bf.raw_data = None;

        // 4방향 테두리 파싱
        let dir_keys = ["borderLeft", "borderRight", "borderTop", "borderBottom"];
        for (i, key) in dir_keys.iter().enumerate() {
            if let Some(obj_str) = json_object(json, key) {
                let type_val = json_i32(&obj_str, "type").unwrap_or(0);
                bf.borders[i].line_type = u8_to_border_line_type(type_val as u8);
                bf.borders[i].width = json_i32(&obj_str, "width").unwrap_or(0) as u8;
                bf.borders[i].color = json_color(&obj_str, "color").unwrap_or(0);
            }
        }

        // 채우기 파싱
        if let Some(fill_type_str) = json_str(json, "fillType") {
            bf.fill = if fill_type_str == "solid" {
                let bg = json_color(json, "fillColor").unwrap_or(0xFFFFFF);
                let pat_c = json_color(json, "patternColor").unwrap_or(0);
                let pat_t = json_i32(json, "patternType").unwrap_or(0);
                Fill {
                    fill_type: FillType::Solid,
                    solid: Some(SolidFill {
                        background_color: bg,
                        pattern_color: pat_c,
                        pattern_type: pat_t,
                    }),
                    ..Default::default()
                }
            } else {
                Fill::default()
            };
        }

        let has_diagonal_json = json.contains("\"diagonalLine\"")
            || json.contains("\"diagonalSlash\"")
            || json.contains("\"diagonalBackSlash\"")
            || json.contains("\"diagonalWidth\"")
            || json.contains("\"diagonalColor\"")
            || json.contains("\"centerLine\"");
        if has_diagonal_json {
            let slash_bits_json = json_diag_bits(json, "diagonalSlash");
            let backslash_bits_json = json_diag_bits(json, "diagonalBackSlash");
            let center_line_json = json_center_line(json);
            let mut slash_bits = slash_bits_json.unwrap_or(((bf.attr >> 2) & 0x07) as u16);
            let mut backslash_bits = backslash_bits_json.unwrap_or(((bf.attr >> 5) & 0x07) as u16);
            let explicit_diagonal_bits = slash_bits_json.is_some() || backslash_bits_json.is_some();
            if let Some(center_line) = center_line_json {
                bf.center_line = center_line;
            }
            bf.diagonal.diagonal_type =
                json_i32(json, "diagonalLine").unwrap_or(bf.diagonal.diagonal_type as i32) as u8;
            bf.diagonal.width =
                json_i32(json, "diagonalWidth").unwrap_or(bf.diagonal.width as i32) as u8;
            bf.diagonal.color = json_color(json, "diagonalColor").unwrap_or(bf.diagonal.color);

            if bf.center_line != CenterLine::None
                && (center_line_json.is_some() || !explicit_diagonal_bits)
            {
                slash_bits = 0;
                backslash_bits = 0;
            } else if slash_bits != 0 || backslash_bits != 0 {
                bf.center_line = CenterLine::None;
            }

            bf.attr &= !((0x07 << 2)
                | (0x07 << 5)
                | (0x03 << 8)
                | (1 << 10)
                | (1 << 11)
                | (1 << 12)
                | (1 << 13));
            bf.attr |= (slash_bits & 0x07) << 2;
            bf.attr |= (backslash_bits & 0x07) << 5;
            bf.attr |= bf.center_line.hwp_attr_bits();
        }

        // 기존 BorderFill에서 동일한 항목 검색
        for (i, existing) in self.document.doc_info.border_fills.iter().enumerate() {
            if border_fills_equal(existing, &bf) {
                return (i + 1) as u16;
            }
        }

        // 새로 추가
        self.document.doc_info.border_fills.push(bf);
        self.document.doc_info.raw_stream_dirty = true;
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        self.document.doc_info.border_fills.len() as u16
    }

    /// <img> 태그를 파싱하여 이미지 데이터를 문서에 추가한다.
    /// (base64 data URI만 지원)
    pub(crate) fn parse_img_html(&mut self, paragraphs: &mut Vec<Paragraph>, img_tag: &str) {
        // src="data:image/...;base64,..." 추출
        let src = if let Some(src_start) = img_tag.find("src=\"") {
            let after = &img_tag[src_start + 5..];
            if let Some(end) = after.find('"') {
                &after[..end]
            } else {
                return;
            }
        } else if let Some(src_start) = img_tag.find("src='") {
            let after = &img_tag[src_start + 5..];
            if let Some(end) = after.find('\'') {
                &after[..end]
            } else {
                return;
            }
        } else {
            return;
        };

        if !src.starts_with("data:") {
            // 외부 URL 이미지는 처리하지 않음 — 텍스트로 대체
            let mut para = Paragraph::default();
            para.text = "[이미지]".to_string();
            para.char_count = para.text.encode_utf16().count() as u32;
            para.char_offsets = para
                .text
                .chars()
                .scan(0u32, |acc, c| {
                    let off = *acc;
                    *acc += c.len_utf16() as u32;
                    Some(off)
                })
                .collect();
            paragraphs.push(para);
            return;
        }

        // data:image/png;base64,XXXXX 파싱
        let after_data = &src[5..]; // "image/png;base64,XXXXX"
        let base64_start = if let Some(comma) = after_data.find(',') {
            comma + 1
        } else {
            return;
        };
        let base64_str = &after_data[base64_start..];

        use base64::Engine;
        let decoded = match base64::engine::general_purpose::STANDARD.decode(base64_str) {
            Ok(d) => d,
            Err(_) => return,
        };

        if decoded.is_empty() {
            return;
        }

        // BinData로 등록
        let new_bin_id = (self.document.bin_data_content.len() + 1) as u16;
        self.document
            .bin_data_content
            .push(crate::model::bin_data::BinDataContent {
                id: new_bin_id,
                data: decoded.clone(),
                extension: detect_clipboard_image_mime(&decoded)
                    .split('/')
                    .nth(1)
                    .unwrap_or("png")
                    .to_string(),
            });

        // width/height 추출
        let width = parse_html_attr_f64(img_tag, "width").unwrap_or(200.0);
        let height = parse_html_attr_f64(img_tag, "height").unwrap_or(150.0);

        // px → HWPUNIT
        let w_hu = crate::renderer::px_to_hwpunit(width, self.dpi) as u32;
        let h_hu = crate::renderer::px_to_hwpunit(height, self.dpi) as u32;

        // Picture Control 생성 (placeholder로 텍스트 표현)
        let mut para = Paragraph::default();
        para.text = "[이미지]".to_string();
        para.char_count = para.text.encode_utf16().count() as u32;
        para.char_offsets = para
            .text
            .chars()
            .scan(0u32, |acc, c| {
                let off = *acc;
                *acc += c.len_utf16() as u32;
                Some(off)
            })
            .collect();

        // Picture 컨트롤 생성
        let mut pic = crate::model::image::Picture::default();
        pic.image_attr.bin_data_id = new_bin_id;
        pic.common.width = w_hu;
        pic.common.height = h_hu;
        pic.common.vertical_offset = 0;
        pic.common.horizontal_offset = 0;
        para.controls.push(Control::Picture(Box::new(pic)));

        paragraphs.push(para);
    }
}

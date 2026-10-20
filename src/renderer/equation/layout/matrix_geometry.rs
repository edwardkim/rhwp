//! 행렬·cases·정렬 행과 세로 묶음의 공유 배치.

use super::{
    EqLayout, EqNode, LayoutBox, LayoutKind, MatrixStyle, PileAlign, MATRIX_COL_GAP, MATRIX_ROW_GAP,
};

// 글꼴·색상과 단일 행의 포장은 내용 소속을 바꾸지 않는다. 분수·조건식·
// 첨자 등의 실제 부모가 있으면 그 구조가 바깥 leading을 소유한다.
pub(super) fn is_standalone_eqalign(node: &EqNode) -> bool {
    match node {
        EqNode::EqAlign { .. } => true,
        EqNode::Row(items) if items.iter().any(|item| matches!(item, EqNode::Newline)) => true,
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => is_standalone_eqalign(body),
        EqNode::Row(items) if items.len() == 1 => is_standalone_eqalign(&items[0]),
        _ => false,
    }
}

impl EqLayout {
    pub(super) fn layout_matrix(
        &self,
        rows: &[Vec<EqNode>],
        style: MatrixStyle,
        fs: f64,
    ) -> LayoutBox {
        if matches!(style, MatrixStyle::Ladder | MatrixStyle::ShortLadder) {
            return self.layout_ladder(rows, style, fs);
        }
        if style == MatrixStyle::Plain && !rows.is_empty() {
            return self.layout_plain_matrix(rows, fs);
        }
        if rows.is_empty() {
            return LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Empty,
            };
        }

        if self.hancom_math_face {
            return self.layout_source_framed_matrix(rows, style, fs);
        }

        let col_gap = fs * MATRIX_COL_GAP;
        let row_gap = fs * MATRIX_ROW_GAP;

        // 모든 셀 레이아웃
        let mut cell_boxes: Vec<Vec<LayoutBox>> = rows
            .iter()
            .map(|row| row.iter().map(|c| self.layout_node(c, fs)).collect())
            .collect();

        let num_cols = cell_boxes.iter().map(|r| r.len()).max().unwrap_or(0);

        // 열 폭 계산
        let mut col_widths = vec![0.0f64; num_cols];
        for row in &cell_boxes {
            for (ci, cell) in row.iter().enumerate() {
                if ci < num_cols {
                    col_widths[ci] = col_widths[ci].max(cell.width);
                }
            }
        }

        // 행 높이 계산
        let row_heights: Vec<f64> = cell_boxes
            .iter()
            .map(|row| row.iter().map(|c| c.height).fold(fs, f64::max))
            .collect();

        // 셀 위치 배정
        let mut y = 0.0;
        for (ri, row) in cell_boxes.iter_mut().enumerate() {
            let rh = row_heights[ri];
            let mut x = 0.0;
            for (ci, cell) in row.iter_mut().enumerate() {
                let cw = if ci < num_cols {
                    col_widths[ci]
                } else {
                    cell.width
                };
                cell.x = x + (cw - cell.width) / 2.0;
                cell.y = y + (rh - cell.height) / 2.0;
                x += cw + if ci + 1 < num_cols { col_gap } else { 0.0 };
            }
            y += rh + row_gap;
        }

        let total_w: f64 =
            col_widths.iter().sum::<f64>() + col_gap * (num_cols.saturating_sub(1)) as f64;
        let total_h = y - row_gap;
        let bracket_pad = fs * 0.2;

        // 괄호 포함 폭
        let paren_w = match style {
            MatrixStyle::Plain => 0.0,
            _ => fs * 0.3,
        };
        let full_w = total_w + paren_w * 2.0 + bracket_pad * 2.0;

        // 셀 x 오프셋 (괄호 포함)
        let x_offset = paren_w + bracket_pad;
        for row in &mut cell_boxes {
            for cell in row.iter_mut() {
                cell.x += x_offset;
            }
        }

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: full_w,
            height: total_h,
            baseline: total_h / 2.0,
            kind: LayoutKind::Matrix {
                cells: cell_boxes,
                style,
            },
        }
    }

    fn layout_source_framed_matrix(
        &self,
        rows: &[Vec<EqNode>],
        style: MatrixStyle,
        fs: f64,
    ) -> LayoutBox {
        // 독립 한컴 저장·Print에서 괄호 유무와 무관하게 셀의 줄·열 프레임은 같다.
        // 구분자 슬롯은 측정과 SVG·Canvas가 함께 소비하는 Paren에 한 번 예약한다.
        let mut body = self.layout_plain_matrix(rows, fs);
        let (left, right, em) = match style {
            MatrixStyle::Paren => ("(", ")", 0.4),
            MatrixStyle::Bracket => ("[", "]", 0.5),
            MatrixStyle::Vert => ("|", "|", 0.5),
            _ => return body,
        };
        let slot = super::source_font_text::logical_em_advance(em, fs, self.dpi);
        body.x = slot;
        let height = body.height;
        let baseline = body.baseline;
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: body.width + slot * 2.0,
            height,
            baseline,
            kind: LayoutKind::Paren {
                left: left.to_string(),
                right: right.to_string(),
                left_width: slot,
                right_width: slot,
                delimiter_height: height,
                glyph_delimiters: false,
                left_outline: self.delimiter_outline(left, slot, height, baseline, fs, false),
                right_outline: self.delimiter_outline(right, slot, height, baseline, fs, false),
                body: Box::new(body),
            },
        }
    }

    pub(super) fn layout_cases(&self, rows: &[EqNode], fs: f64) -> LayoutBox {
        // 독립 11/22pt의 2·3·4행 정상 저장/Print: 일반 행 프레임 합 + 경계마다 0.15em.
        let row_gap = fs
            * if self.hancom_math_face {
                0.15
            } else {
                MATRIX_ROW_GAP
            };
        let brace_width = if self.hancom_math_face {
            super::source_font_advance::curly_slot(fs, self.dpi)
        } else {
            // 이전 두 0.3em 예약의 실제 본체 원점·전체 폭을 보존한다.
            fs * 0.6
        };
        let mut row_boxes = self.layout_cases_rows(rows, fs);

        let mut y = 0.0;
        for b in &mut row_boxes {
            // 명시적 빈 행은 표시 글자 없이도 한 글자 줄을 점유한다.
            if b.height == 0.0 {
                b.height = fs;
                b.baseline = fs * 0.8;
            }
            b.x = 0.0; // 중괄호 슬롯은 아래 body 원점에서 한 번 소비한다.
            b.y = y;
            y += b.height + row_gap;
        }
        let total_h = y - row_gap;
        // 실제 행 원점과 점유 끝을 폭으로 공유한다. 중괄호의 예약은
        // 아래 Paren의 body.x에서 한 번 소비하며, 빈 오른쪽 여백을
        // Row 폭에 다시 넣으면 뒤 본문과 delimiter가 그만큼 밀린다.
        let full_w = row_boxes
            .iter()
            .map(|row| row.x + row.width)
            .fold(0.0f64, f64::max);
        let full_w = if self.hancom_math_face && full_w == 0.0 && !row_boxes.is_empty() {
            fs * 0.5
        } else {
            full_w
        };
        // 행 구성과 관계없이 묶음의 수학 축을 가운데에 놓는다. 왼쪽 식의
        // 기준선은 일반 글줄의 축 거리만큼 내려간다. 분수 행을 앞뒤로
        // 옮겨도 두 글자 기준선의 평균으로 축이 바뀌지 않는다.
        let axis_offset = if self.hancom_math_face {
            super::super::font_frame_metrics::line_at_dpi(fs, self.dpi)
                .map_or(fs * 0.3, |(height, baseline)| baseline - height / 2.0)
        } else {
            0.0
        };
        let baseline = total_h / 2.0 + axis_offset;

        // 중괄호 포함 레이아웃 → Paren으로 래핑
        let inner = LayoutBox {
            font_size: fs,
            // 외곽 폭에 예약한 왼쪽 중괄호 폭을 실제 자식 원점에서도 소비한다.
            x: brace_width,
            y: 0.0,
            width: full_w,
            height: total_h,
            baseline,
            kind: LayoutKind::Row(row_boxes),
        };

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: full_w + brace_width,
            height: total_h,
            baseline,
            kind: LayoutKind::Paren {
                left: "{".to_string(),
                right: String::new(),
                left_width: brace_width,
                right_width: 0.0,
                delimiter_height: total_h,
                glyph_delimiters: false,
                left_outline: self.delimiter_outline(
                    "{",
                    brace_width,
                    total_h,
                    baseline,
                    fs,
                    false,
                ),
                right_outline: None,
                body: Box::new(inner),
            },
        }
    }

    pub(super) fn layout_eqalign(
        &self,
        rows: &[(EqNode, EqNode)],
        alignment_marks: &[bool],
        fs: f64,
    ) -> LayoutBox {
        self.layout_aligned_rows(rows, alignment_marks, fs, fs * 0.15)
    }

    pub(super) fn layout_newline_rows(&self, children: &[EqNode], fs: f64) -> LayoutBox {
        // 빈 행도 남기고, 일반 줄은 & 정렬 여백 없이 왼쪽 원점을 공유한다.
        let rows: Vec<_> = children
            .split(|node| matches!(node, EqNode::Newline))
            .map(|line| (EqNode::Empty, EqNode::Row(line.to_vec()).simplify()))
            .collect();
        self.layout_aligned_rows(&rows, &vec![false; rows.len()], fs, 0.0)
    }

    fn layout_aligned_rows(
        &self,
        rows: &[(EqNode, EqNode)],
        alignment_marks: &[bool],
        fs: f64,
        align_gap: f64,
    ) -> LayoutBox {
        // 독립 정렬 묶음의 Print 피치는 1.1em이다. 분수·CASES 등 실제
        // 부모가 있으면 바깥 leading을 그 부모가 소유하므로 중복 예약하지 않는다.
        let row_gap = fs
            * if self.hancom_math_face {
                if self.standalone_eqalign {
                    0.1
                } else {
                    0.0
                }
            } else {
                MATRIX_ROW_GAP
            };

        // 각 행의 왼쪽/오른쪽 레이아웃 계산
        let nested = Self {
            standalone_eqalign: false,
            ..*self
        };
        let mut laid_rows: Vec<(LayoutBox, LayoutBox)> = rows
            .iter()
            .map(|(l, r)| (nested.layout_node(l, fs), nested.layout_node(r, fs)))
            .collect();

        // &가 하나라도 있으면 첫 칸도 전체 행의 공유 열이다.
        // &가 없는 묶음과 오른쪽이 빈 & 칸은 원문 표시로 구분한다.
        let aligned = !self.hancom_math_face || alignment_marks.iter().any(|marked| *marked);
        let gap = rows
            .iter()
            .map(|(_, right)| {
                if self.hancom_math_face {
                    self.alignment_boundary_gap(right, fs)
                } else {
                    align_gap
                }
            })
            .fold(0.0, f64::max);
        let left_column_width = laid_rows
            .iter()
            .map(|(left, _)| left.width)
            .fold(0.0f64, f64::max);

        let mut y = 0.0;
        let mut total_w = 0.0f64;
        let mut first_baseline = None;
        for (left, right) in &mut laid_rows {
            // 각 행의 본체는 공유 첫 칸에 오른쪽 정렬한다.
            left.x = if aligned {
                left_column_width - left.width
            } else {
                0.0
            };
            let has_right = !matches!(right.kind, LayoutKind::Empty);
            right.x = if aligned {
                left_column_width
            } else {
                left.width
            } + if has_right { gap } else { 0.0 };

            let (left_h, left_bl) = self.matrix_cell_frame(left, fs);
            let (right_h, right_bl) = self.matrix_cell_frame(right, fs);
            let empty = left_h == 0.0 && right_h == 0.0;
            let row_bl = if empty {
                fs * if self.hancom_math_face { 0.86 } else { 0.8 }
            } else {
                left_bl.max(right_bl)
            };
            let row_h = if empty {
                fs
            } else {
                row_bl + (left_h - left_bl).max(right_h - right_bl)
            };
            first_baseline.get_or_insert(row_bl);
            // 베이스라인 정렬
            left.y = y + (row_bl - left.baseline);
            right.y = y + (row_bl - right.baseline);

            total_w = total_w.max(right.x + right.width);
            y += row_h + row_gap;
        }
        let total_h = (y - row_gap).max(0.0);

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: if total_w == 0.0 && self.hancom_math_face {
                fs * 0.5
            } else {
                total_w
            },
            height: total_h,
            baseline: if self.hancom_math_face {
                first_baseline.unwrap_or(fs * 0.8)
            } else {
                total_h / 2.0
            },
            kind: LayoutKind::EqAlign { rows: laid_rows },
        }
    }

    pub(super) fn layout_pile(&self, rows: &[EqNode], align: PileAlign, fs: f64) -> LayoutBox {
        // 한컴 PILE은 논리 행 프레임과 경계마다 0.15em을 예약한다.
        // 글리프 높이만 더하면 일반 글자·분수·행렬이 다른 행 피치를 쓴다.
        let row_gap = fs
            * if self.hancom_math_face {
                0.15
            } else {
                MATRIX_ROW_GAP
            };
        let mut row_boxes: Vec<LayoutBox> = rows
            .iter()
            .map(|row| {
                let child = self.layout_node(row, fs);
                if self.hancom_math_face {
                    self.matrix_row_box(child, fs)
                } else {
                    child
                }
            })
            .collect();

        let max_w = row_boxes.iter().map(|b| b.width).fold(0.0f64, f64::max);
        let max_w = if max_w == 0.0 && self.hancom_math_face && !row_boxes.is_empty() {
            fs * 0.5
        } else {
            max_w
        };
        let mut y = 0.0;
        for b in &mut row_boxes {
            b.x = match align {
                PileAlign::Left => 0.0,
                PileAlign::Center => (max_w - b.width) / 2.0,
                PileAlign::Right => max_w - b.width,
            };
            b.y = y;
            y += b.height + row_gap;
        }
        let total_h = y - row_gap;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: max_w,
            height: total_h,
            // 가운데 수학 축에서 본문 기준선까지의 거리를 보존한다.
            // 논리 글자 프레임의 기준선 0.86em에서 가운데 0.5em을 뺀다.
            baseline: total_h / 2.0
                + if self.hancom_math_face {
                    fs * 0.36
                } else {
                    0.0
                },
            kind: LayoutKind::Row(row_boxes),
        }
    }
}

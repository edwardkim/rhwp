//! 나눗셈 사다리의 셀·괘선을 측정 단계에서 함께 배치한다.
//!
//! 한컴 수식 사양 1.3의 LADDER/SLADDER 문법과 독립 2024 Print의
//! 8/12/16/24pt 대조에 근거한다. 마지막 행은 나누는 수가 없는 결과 행이다.
//! 괘선은 사설 글리프 윤곽을 복제하지 않고 일반 직사각형으로 구성한다.

use super::{EqLayout, EqNode, LayoutBox, LayoutKind, MatrixStyle};
use crate::renderer::equation::GlyphOutlineCommand as Command;

impl EqLayout {
    pub(super) fn layout_ladder(
        &self,
        rows: &[Vec<EqNode>],
        style: MatrixStyle,
        fs: f64,
    ) -> LayoutBox {
        if rows.is_empty() {
            return self.layout_node(&EqNode::Empty, fs);
        }
        let short = style == MatrixStyle::ShortLadder;
        let division_rows = if rows.len() == 1 { 1 } else { rows.len() - 1 };
        let columns = rows[..division_rows]
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(2);
        let mut cells: Vec<Vec<LayoutBox>> = rows
            .iter()
            .enumerate()
            .map(|(ri, row)| {
                // 결과 행에는 제수 칸이 없다. 끝의 빈 칸은 템플릿 자리다.
                let capacity = if ri >= division_rows {
                    columns - 1
                } else {
                    columns
                };
                row.iter()
                    .take(capacity)
                    .enumerate()
                    .filter(|(ci, _)| !short || ri != 0 || *ci < 2)
                    .map(|(_, node)| self.layout_node(node, fs))
                    .collect()
            })
            .collect();
        let main_columns = if short { 2 } else { columns };
        let mut main_widths = vec![0.0_f64; main_columns];
        let mut remainder_widths = vec![0.0_f64; columns.saturating_sub(2)];
        for (ri, row) in cells.iter().enumerate() {
            let shift = usize::from(ri >= division_rows);
            for (ci, cell) in row.iter().enumerate() {
                let column = ci + shift;
                if column < main_columns {
                    main_widths[column] = main_widths[column].max(cell.width);
                } else if let Some(width) = remainder_widths.get_mut(column - 2) {
                    *width = width.max(cell.width);
                }
            }
        }
        // 독립 한컴 저장·Print의 24개 글자/숫자 대조군은 각 열의 폭을 따로
        // 정하고 제수 열에만 반 em을 더한다. 모든 열을 최대 셀 폭으로 만들면
        // 한 자리 제수·긴 피제수의 원점이 어긋난다. 공백 유무는 폭을 바꾸지 않는다.
        main_widths[0] += fs * 0.5;
        let gap = fs * 0.2;
        let main_width = main_widths.iter().copied().fold(0.0_f64, f64::max);
        let slot = main_width + gap;
        let mut starts = Vec::with_capacity(columns);
        let mut width = 0.0;
        for &column_width in &main_widths {
            starts.push(width);
            width += column_width + gap;
        }
        let mut widths = main_widths;
        if short {
            for remainder_width in &remainder_widths {
                // 값 칸 다음에 점 칸, 그 뒤에 나머지 칸을 둔다.
                starts.push(width + slot);
                widths.push(*remainder_width);
                width += slot + remainder_width + gap;
            }
        }

        let (line_height, line_baseline) =
            super::super::font_frame_metrics::line_at_dpi(fs, self.dpi).unwrap_or((fs, fs * 0.86));
        let mut y = 0.0;
        let mut outlines = Vec::new();
        let mut dots = Vec::new();
        for (ri, row) in cells.iter_mut().enumerate() {
            // 저장본과 독립 Print가 확인한 논리 글줄 프레임을 사용한다.
            // 표시 글꼴의 자연 EM 상자를 줄 예약 높이로 다시 합산하지 않는다.
            let frames: Vec<(f64, f64)> = row
                .iter()
                .map(|cell| {
                    super::super::logical_font_frame::metrics_at_dpi(cell, fs, self.dpi)
                        .unwrap_or((cell.height, cell.baseline))
                })
                .collect();
            let ascent = frames
                .iter()
                .map(|(_, baseline)| *baseline)
                .fold(line_baseline, f64::max);
            let descent = frames
                .iter()
                .map(|(height, baseline)| height - baseline)
                .fold(line_height - line_baseline, f64::max);
            let height = ascent + descent;
            let shift = usize::from(ri >= division_rows);
            for (ci, cell) in row.iter_mut().enumerate() {
                let column = ci + shift;
                cell.x = starts[column] + (widths[column] - cell.width) / 2.0;
                cell.y = y + ascent - cell.baseline;
                if short && column >= 2 && ri > 0 {
                    let dot_width = fs;
                    dots.push(LayoutBox {
                        font_size: fs,
                        x: starts[column] - slot + (main_width - dot_width) / 2.0,
                        y: y + ascent - fs * 0.8,
                        width: dot_width,
                        height: fs,
                        baseline: fs * 0.8,
                        kind: LayoutKind::FontGlyph {
                            text: "⋯".into(),
                            font_family: "HYhwpEQ, Latin Modern Math, STIX Two Math, serif",
                            paint: None,
                            outline: None,
                        },
                    });
                }
            }
            if ri < division_rows {
                let left = starts[1] - fs * 0.25;
                let right = starts[main_columns - 1] + widths[main_columns - 1] + gap - fs * 0.25;
                let bottom = y + height + fs * 0.05;
                let thickness = fs * 0.06;
                rectangle(
                    &mut outlines,
                    left,
                    y,
                    thickness,
                    bottom - y + thickness / 2.0,
                );
                rectangle(
                    &mut outlines,
                    left,
                    bottom - thickness / 2.0,
                    right - left,
                    thickness,
                );
            }
            y += height + fs * 0.15;
        }
        // 한 줄의 기준선을 가운데 정렬한 전체 줄 묶음으로 옮긴다.
        // 한컴 저장 기준선은 정수 %를 올림하므로 HWPUNIT 정수로 계산한다.
        let baseline = centered_baseline(y, line_height, line_baseline, fs, self.dpi);
        // 모든 출력 경로가 같은 셀 원점·괘선 윤곽을 소비한다.
        dots.push(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: y,
            baseline,
            kind: LayoutKind::FontGlyph {
                text: String::new(),
                font_family: "serif",
                paint: None,
                outline: Some(outlines),
            },
        });
        cells.push(dots);
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: y,
            baseline,
            kind: LayoutKind::Matrix { cells, style },
        }
    }
}

pub(super) fn centered_baseline(
    height: f64,
    line_height: f64,
    line_baseline: f64,
    fs: f64,
    dpi: f64,
) -> f64 {
    let pitch = line_height + fs * 0.15;
    let single_percent = ((line_baseline + fs * 0.075) / pitch * 100.0).round() as u64;
    let height_hu = (height * 7200.0 / dpi).round() as u64;
    let pitch_hu = (pitch * 7200.0 / dpi).round() as u64;
    if height_hu == 0 {
        return 0.0;
    }
    let numerator = 50 * height_hu + single_percent.saturating_sub(50) * pitch_hu;
    let percent = numerator.div_ceil(height_hu);
    height * percent as f64 / 100.0
}

fn rectangle(commands: &mut Vec<Command>, x: f64, y: f64, width: f64, height: f64) {
    commands.extend([
        Command::Move((x, y)),
        Command::Line((x + width, y)),
        Command::Line((x + width, y + height)),
        Command::Line((x, y + height)),
        Command::Close,
    ]);
}

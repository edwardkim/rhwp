//! 원고지 칸 조판.
//!
//! 줄 격자·글자 격자가 있고 원고지 정서법(`wonggojiFormat`)이 켜진 구역에서 한/글 12.30(Mac)은
//! 본문 글자를 칸에 하나씩 놓는다. 아래 규칙은 한/글이 만든 200자 원고지(B5, 칸 2863HU·줄 3968HU,
//! 함초롬바탕 19.2pt) 문서의 인쇄 PDF 글자 원점과 저장 줄(`linesegarray`)에서 읽었다.
//!
//! - 한 칸에 한 글자. 한글·한자·전각 기호·공백·라틴 대문자·`-`·`(`·`"`·`!`·`?` 같은 기호는 한 칸이다.
//! - 라틴 소문자·숫자·`.`·`,`는 이어지면 두 자가 한 칸에 든다(`20|24`, `ap|py`, `t.`, `3,|00|0`).
//! - 글리프는 칸 가운데에 놓는다(둘이면 두 전진폭의 합을 가운데에). 홀로 칸을 차지한 `.`·`,`는
//!   숫자 둘이 들 자리의 왼쪽에 놓인다.
//! - 줄은 칸 수로 나눈다. 한글은 글자마다 끊고, 라틴·숫자 낱말은 본래 폭으로 남은 칸에 드는지 본 뒤
//!   칸에 놓는다(그래서 대문자 낱말은 줄 끝을 넘칠 수 있다). 공백과 줄 머리 금칙 글자는 줄 끝에 매단다.
//! - 줄은 줄 격자 한 칸 높이를 차지하고 글자는 그 가운데에 선다.

use crate::model::control::Control;
use crate::model::paragraph::CellGrid;

/// 이어지면 두 자가 한 칸에 드는 반각 글자.
pub(crate) fn pairs_in_cell(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == ','
}

/// 라틴 낱말을 이루는 글자(공백이 아닌 ASCII 글자·기호).
fn in_latin_word(ch: char) -> bool {
    ch.is_ascii_graphic()
}

/// 한 줄 글자들의 칸 배치. `cell[i]`는 i번째 글자가 든 칸(줄 첫 칸이 0), `offset[i]`는 그 칸
/// 왼쪽에서 글리프 원점까지의 거리, `cells`는 줄이 차지한 칸 수다.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LinePlacement {
    pub cell: Vec<usize>,
    pub offset: Vec<f64>,
    pub cells: usize,
}

/// 한 줄을 칸에 놓는다. `advances[i]`는 i번째 글자의 전진폭, `digit_advances[i]`는 그 글자 글꼴의
/// 숫자 폭(홀로 선 `.`·`,` 자리), `cols`는 줄의 칸 수, `cell`은 칸 폭이다. 단위는 셋이 같으면 된다.
/// 줄 끝 다음 한 칸(`cols`)까지는 매단 글자를 칸에 놓는다.
pub(crate) fn place_line(
    chars: &[char],
    advances: &[f64],
    digit_advances: &[f64],
    first_cell: usize,
    cols: usize,
    cell: f64,
) -> LinePlacement {
    let mut placement = LinePlacement {
        cell: vec![0; chars.len()],
        offset: vec![0.0; chars.len()],
        cells: first_cell,
    };
    let advance = |i: usize| advances.get(i).copied().unwrap_or(0.0);
    let mut i = 0;
    while i < chars.len() {
        let k = placement.cells;
        if k > cols {
            // 줄 끝 매단 칸 뒤로 넘친 글자는 칸 없이 본래 폭으로 잇는다(한/글 `E|FG`).
            let mut x = 0.0;
            for j in i..chars.len() {
                placement.cell[j] = k;
                placement.offset[j] = x;
                x += advance(j);
            }
            placement.cells = k + 1;
            break;
        }
        if pairs_in_cell(chars[i]) && chars.get(i + 1).copied().is_some_and(pairs_in_cell) {
            let start = (cell - advance(i) - advance(i + 1)) / 2.0;
            placement.cell[i] = k;
            placement.cell[i + 1] = k;
            placement.offset[i] = start;
            placement.offset[i + 1] = start + advance(i);
            i += 2;
        } else {
            placement.cell[i] = k;
            placement.offset[i] = if matches!(chars[i], '.' | ',') {
                (cell - 2.0 * digit_advances.get(i).copied().unwrap_or(0.0)) / 2.0
            } else {
                (cell - advance(i)) / 2.0
            };
            i += 1;
        }
        placement.cells = k + 1;
    }
    placement
}

/// 글자 차례 `chars`(라틴 낱말 하나)가 칸에 놓이면 차지하는 칸 수.
fn word_cells(chars: &[char]) -> usize {
    let mut cells = 0;
    let mut i = 0;
    while i < chars.len() {
        i += if pairs_in_cell(chars[i]) && chars.get(i + 1).copied().is_some_and(pairs_in_cell) {
            2
        } else {
            1
        };
        cells += 1;
    }
    cells
}

/// 문단을 줄로 나눈다. 돌려주는 값은 줄마다 첫 글자 차례다(첫 줄은 0).
///
/// `natural[i]`는 i번째 글자의 본래 전진폭(HWPUNIT, 라틴 낱말이 남은 칸에 드는지 볼 때만 쓴다),
/// `cols`는 한 줄 칸 수, `first_cells`는 첫 줄 들여쓰기 칸 수다. `\n`은 강제 줄 바꿈이다.
pub(crate) fn line_starts(
    chars: &[char],
    natural: &[i32],
    cols: usize,
    cell: i32,
    first_cells: usize,
) -> Vec<usize> {
    let cols = cols.max(1);
    let line_width = i64::from(cell) * cols as i64;
    let mut starts = vec![0];
    let mut line_start = 0;
    let mut k = first_cells;
    let mut i = 0;
    let mut break_before = |i: usize, line_start: &mut usize, k: &mut usize| {
        starts.push(i);
        *line_start = i;
        *k = 0;
    };
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\n' {
            // 문단 끝의 강제 줄 바꿈도 빈 줄 하나를 연다(캐럿이 설 줄).
            i += 1;
            break_before(i, &mut line_start, &mut k);
            continue;
        }
        if ch == ' ' || ch == '\t' {
            // 줄 끝을 넘는 공백은 그 줄에 매달고 다음 줄을 공백으로 시작하지 않는다.
            k += 1;
            i += 1;
            continue;
        }
        if in_latin_word(ch) {
            let end = (i..chars.len())
                .find(|&j| !in_latin_word(chars[j]))
                .unwrap_or(chars.len());
            let width: i64 = (i..end)
                .map(|j| i64::from(natural.get(j).copied().unwrap_or(cell)))
                .sum();
            let fits = i64::from(cell) * k as i64 + width <= line_width;
            if !fits
                && i > line_start
                && k > 0
                && !crate::renderer::composer::is_line_start_forbidden(ch)
            {
                break_before(i, &mut line_start, &mut k);
            }
            k += word_cells(&chars[i..end]);
            i = end;
            continue;
        }
        if k >= cols && i > line_start && !crate::renderer::composer::is_line_start_forbidden(ch) {
            break_before(i, &mut line_start, &mut k);
        }
        k += 1;
        i += 1;
    }
    starts
}

/// 들여쓰기(HWPUNIT)가 첫 줄에서 비우는 칸 수.
pub(crate) fn indent_cells(indent_hu: i32, grid: CellGrid) -> usize {
    if indent_hu <= 0 || grid.cell <= 0 {
        0
    } else {
        ((indent_hu + grid.cell - 1) / grid.cell) as usize
    }
}

/// 글자 높이 `height`(HWPUNIT)의 줄이 차지하는 줄 격자 높이.
pub(crate) fn line_pitch(height: i32, grid: CellGrid) -> i32 {
    let steps = if height <= 0 {
        1
    } else {
        height.saturating_add(grid.line - 1) / grid.line
    };
    steps.max(1).saturating_mul(grid.line)
}

/// 문단 배치가 만든 한 줄의 글자 런을 칸에 놓는다. `origin`은 줄 첫 칸 왼쪽(px), `width`는
/// 문단 여백을 뺀 줄 폭(px)이고 첫 줄은
/// 들여쓰기 칸 `first_cells`를 비운다. 양쪽 정렬 등으로 벌린 간격은 버린다. 커서·선택·그리기가
/// 모두 런의 `layout_positions`를 읽으므로 글자 자리는 여기 한 곳에서 정해진다.
pub(crate) fn place_line_runs(
    line: &mut crate::renderer::render_tree::RenderNode,
    grid: CellGrid,
    origin: f64,
    width: f64,
    first_cells: usize,
    dpi: f64,
    next_id: &mut dyn FnMut() -> crate::renderer::render_tree::NodeId,
) {
    use crate::renderer::layout::{compute_char_positions, estimate_text_width};
    use crate::renderer::render_tree::RenderNodeType;

    let cell = crate::renderer::hwpunit_to_px(grid.cell, dpi);
    if cell <= 0.0 {
        return;
    }
    // 표시 글자가 원문과 다른 런(필드 결과·글자 겹침)은 칸 수를 셀 수 없어 줄째로 둔다.
    if line.children.iter().any(|child| match &child.node_type {
        RenderNodeType::TextRun(run) => run.display_text.is_some() || run.char_overlap.is_some(),
        _ => false,
    }) {
        return;
    }
    let mut chars = Vec::new();
    let mut advances = Vec::new();
    let mut digits = Vec::new();
    for child in &mut line.children {
        let RenderNodeType::TextRun(run) = &mut child.node_type else {
            continue;
        };
        run.style.extra_word_spacing = 0.0;
        run.style.extra_char_spacing = 0.0;
        run.style.extra_dash_advance = 0.0;
        let positions = compute_char_positions(&run.text, &run.style);
        let digit = estimate_text_width("0", &run.style);
        for (i, ch) in run.text.chars().enumerate() {
            chars.push(ch);
            advances.push(positions.get(i + 1).copied().unwrap_or(0.0) - positions[i]);
            digits.push(digit);
        }
    }
    let cols = (width / cell + 1e-6).floor().max(1.0) as usize;
    let placement = place_line(&chars, &advances, &digits, first_cells, cols, cell);
    // 런을 칸마다 나눈다. 한 런 안의 글자 경계가 본래 전진폭이어야 그리기의 글리프 폭 맞춤
    // (SVG textLength·Canvas scaleX)이 칸 사이 빈자리만큼 글리프를 늘이지 않는다.
    let mut next = 0;
    for mut child in std::mem::take(&mut line.children) {
        // 칸으로 나눈 조각은 원래 런을 본뜬다.
        let RenderNodeType::TextRun(run) = &mut child.node_type else {
            line.children.push(child);
            continue;
        };
        let text: Vec<char> = run.text.chars().collect();
        if text.is_empty() {
            // 문단 끝 표시 런은 다음 칸 왼쪽에 둔다.
            child.bbox.x = origin + placement.cells as f64 * cell;
            line.children.push(child);
            continue;
        }
        // 누름틀 경계 표지는 나누면 자리가 흐트러져 런째로 둔다.
        let splittable = matches!(
            run.field_marker,
            crate::renderer::render_tree::FieldMarkerType::None
        );
        let mut start = 0;
        while start < text.len() {
            let mut end = start + 1;
            while end < text.len()
                && (!splittable || placement.cell[next + end] == placement.cell[next + start])
            {
                end += 1;
            }
            let x = |i: usize| {
                origin + placement.cell[next + i] as f64 * cell + placement.offset[next + i]
            };
            let left = x(start);
            let mut positions = Vec::with_capacity(end - start + 1);
            let mut previous = 0.0f64;
            for i in start..end {
                previous = previous.max(x(i) - left);
                positions.push(previous);
            }
            positions.push(previous.max(x(end - 1) - left + advances[next + end - 1]));
            let mut piece = child.clone();
            if start > 0 {
                piece.id = next_id();
            }
            if let RenderNodeType::TextRun(part) = &mut piece.node_type {
                if start > 0 || end < text.len() {
                    part.text = text[start..end].iter().collect();
                    part.char_start = part.char_start.map(|first| first + start);
                }
                if end < text.len() {
                    part.is_para_end = false;
                    part.is_line_break_end = false;
                }
            }
            piece.bbox.x = left;
            piece.bbox.width = positions[end - start];
            if let RenderNodeType::TextRun(part) = &mut piece.node_type {
                part.layout_positions = Some(positions);
            }
            line.children.push(piece);
            start = end;
        }
        next += text.len();
    }
}

/// 칸에 맞춰 짜는 본문 문단인가: 문단 모양이 줄 격자를 쓰고(문단 모양 bit 8) 글자 사이에
/// 자리를 차지하는 개체가 없는 글 문단이다. 표·그림이 든 문단은 지금처럼 짠다.
pub(crate) fn paragraph_takes_grid(
    para: &crate::model::paragraph::Paragraph,
    para_shapes: &[crate::model::style::ParaShape],
) -> bool {
    !para.line_segs.is_empty() && shape_takes_grid(para, para_shapes)
}

/// 줄과 상관없이 문단 모양·개체로 본 칸 조판 대상인가. 저장 줄 없이 연 문단에 줄을 지을지 정한다.
pub(crate) fn shape_takes_grid(
    para: &crate::model::paragraph::Paragraph,
    para_shapes: &[crate::model::style::ParaShape],
) -> bool {
    para_shapes
        .get(para.para_shape_id as usize)
        .is_some_and(|shape| shape.attr1 & (1 << 8) != 0)
        && para.controls.iter().all(|control| {
            // 머리말·꼬리말은 첫 문단에 붙지만 글 사이에 자리를 차지하지 않는다.
            matches!(control, Control::Header(_) | Control::Footer(_))
                || crate::renderer::composer::control_is_width_neutral_marker(control)
        })
}

/// 렌더 정규화본의 원고지 문단에 칸 격자를 붙이고 줄을 줄 격자 높이로 바꾼다.
///
/// 저장 줄의 높이·기준선은 글자 크기(한/글도 그렇게 저장한다)라 렌더 사본에서만 줄 상자를 격자
/// 높이로 키우고 기준선을 그 가운데로 내린다. 세로 위치는 앞 문단 끝에서 이어 붙이되, 저장 위치가
/// 앞 문단보다 작아진 곳(쪽 첫 줄)은 저장값을 따른다. 바꾼 문단 차례를 돌려준다.
pub(crate) fn project_section_rows(
    paragraphs: &mut [crate::model::paragraph::Paragraph],
    para_shapes: &[crate::model::style::ParaShape],
    grid: CellGrid,
) -> Vec<usize> {
    let mut changed = Vec::new();
    let mut next_y: Option<i32> = None;
    let mut previous_first: Option<i32> = None;
    for (index, para) in paragraphs.iter_mut().enumerate() {
        let Some(first) = para.line_segs.first().map(|line| line.vertical_pos) else {
            continue;
        };
        if !paragraph_takes_grid(para, para_shapes) {
            next_y = para.line_segs.last().map(|line| {
                line.vertical_pos
                    .saturating_add(line.line_height)
                    .saturating_add(line.line_spacing)
            });
            previous_first = Some(first);
            continue;
        }
        let mut y = match (next_y, previous_first) {
            (_, Some(previous)) if first < previous => first,
            (Some(end), _) => end,
            _ => first,
        };
        for line in &mut para.line_segs {
            let pitch = line_pitch(line.line_height, grid);
            line.baseline_distance = line
                .baseline_distance
                .saturating_add((pitch - line.line_height) / 2);
            line.line_height = pitch;
            line.text_height = pitch;
            line.line_spacing = 0;
            line.vertical_pos = y;
            y = y.saturating_add(pitch);
        }
        para.layout_cell_grid = Some(grid);
        next_y = Some(y);
        previous_first = Some(first);
        changed.push(index);
    }
    changed
}

/// 편집으로 다시 짠 원고지 문단의 줄을 칸 수로 다시 나눈다. 줄 높이·기준선은 방금 짠 첫 줄을
/// 따르고 세로 위치는 줄 격자 한 칸씩 내려간다. `width_hu`는 문단 여백을 뺀 줄 폭이다.
pub(crate) fn rebreak_paragraph(
    para: &mut crate::model::paragraph::Paragraph,
    grid: CellGrid,
    width_hu: i32,
    indent_hu: i32,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) {
    use crate::model::paragraph::LineSeg;
    let Some(template) = para.line_segs.first().cloned() else {
        return;
    };
    let chars: Vec<char> = para.text.chars().collect();
    let natural: Vec<i32> = chars
        .iter()
        .enumerate()
        .map(|(i, &ch)| {
            if !ch.is_ascii_graphic() {
                return grid.cell;
            }
            let style = crate::renderer::layout::resolved_to_text_style(
                styles,
                para.char_shape_id_at(i).unwrap_or(0),
                1,
            );
            let mut buf = [0u8; 4];
            let px = crate::renderer::layout::estimate_text_width(ch.encode_utf8(&mut buf), &style);
            crate::renderer::px_to_hwpunit(px, dpi)
        })
        .collect();
    let cols = (width_hu / grid.cell.max(1)).max(1) as usize;
    let starts = line_starts(
        &chars,
        &natural,
        cols,
        grid.cell,
        indent_cells(indent_hu, grid),
    );
    let follow_tag = para
        .line_segs
        .get(1)
        .map_or(template.tag & !LineSeg::TAG_INDENTATION, |line| line.tag);
    // 줄마다 가장 큰 글자 크기가 줄 높이다(글자가 없는 줄은 첫 줄 높이).
    let text_end = para
        .char_offsets
        .last()
        .zip(chars.last())
        .map_or(0, |(offset, &ch)| {
            offset + crate::model::paragraph::Paragraph::char_stream_len(ch)
        });
    let mut y = template.vertical_pos;
    let mut lines = Vec::with_capacity(starts.len());
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(chars.len());
        let height = (start..end)
            .filter_map(|i| {
                let id = para.char_shape_id_at(i)?;
                styles.char_styles.get(id as usize)
            })
            .map(|style| crate::renderer::px_to_hwpunit(style.font_size, dpi))
            .max()
            .filter(|height| *height > 0)
            .unwrap_or(template.line_height);
        let pitch = line_pitch(height, grid);
        lines.push(LineSeg {
            text_start: if n == 0 {
                template.text_start
            } else {
                para.char_offsets.get(start).copied().unwrap_or(text_end)
            },
            vertical_pos: y,
            line_height: height,
            text_height: height,
            baseline_distance: (f64::from(height) * 0.85).round() as i32,
            line_spacing: pitch - height,
            tag: if n == 0 { template.tag } else { follow_tag },
            ..template.clone()
        });
        y = y.saturating_add(pitch);
    }
    para.line_segs = lines;
    // 줄 단위 공백 폭 기록은 옛 줄을 가리킨다. 칸 배치는 공백도 한 칸이라 쓰지 않는다.
    para.layout_space_metrics.clear();
}

/// 원고지 구역이면 방금 짠 본문 문단을 칸 수로 다시 나누고 구역 줄 간격을 격자에 맞춘다.
/// 본문 문단을 다시 짜는 모든 편집(글·글자/문단 모양·스타일·쪽 설정)이 거친다.
pub(crate) fn fit_body_paragraph(
    section: &mut crate::model::document::Section,
    para_shapes: &[crate::model::style::ParaShape],
    para_idx: usize,
    column_width_px: f64,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) {
    let Some(grid) = section.section_def.cell_grid() else {
        return;
    };
    if let Some(para) = section.paragraphs.get_mut(para_idx) {
        if let (true, Some(style)) = (
            paragraph_takes_grid(para, para_shapes),
            styles.para_styles.get(para.para_shape_id as usize),
        ) {
            let width = crate::renderer::px_to_hwpunit(
                column_width_px - style.margin_left - style.margin_right,
                dpi,
            );
            let indent = crate::renderer::px_to_hwpunit(style.indent, dpi);
            rebreak_paragraph(para, grid, width, indent, styles, dpi);
        }
    }
    fit_section_spacing(&mut section.paragraphs, para_shapes, grid);
}

/// 구역 원고지 문단의 저장 줄 간격을 줄 격자 높이에 맞춘다. 한/글은 글자 높이(줄 간격 0)로
/// 적고 세로 위치만 격자로 띄우는데, 문단 세로 위치를 다시 셈하는 곳은 줄 높이+줄 간격으로
/// 다음 문단을 놓으므로 간격에 격자 나머지를 담아 둔다. 바꾼 줄이 있으면 참이다.
pub(crate) fn fit_section_spacing(
    paragraphs: &mut [crate::model::paragraph::Paragraph],
    para_shapes: &[crate::model::style::ParaShape],
    grid: CellGrid,
) -> bool {
    let mut changed = false;
    for para in paragraphs.iter_mut() {
        if !paragraph_takes_grid(para, para_shapes) {
            continue;
        }
        for line in &mut para.line_segs {
            let spacing = line_pitch(line.line_height, grid) - line.line_height;
            if line.line_spacing != spacing {
                line.line_spacing = spacing;
                changed = true;
            }
        }
    }
    changed
}

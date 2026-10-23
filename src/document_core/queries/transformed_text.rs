//! Editor coordinates use the very same rectangle subtree frame as paint.
//! Line ownership and glyph advances remain those of the composed TextRun.
use crate::document_core::helpers::find_char_at_x;
use crate::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};

type Matrix = [f64; 6];

fn concat(p: Matrix, q: Matrix) -> Matrix {
    let [a, b, c, d, e, f] = p;
    let [g, h, i, j, k, l] = q;
    [
        a * g + c * h,
        b * g + d * h,
        a * i + c * j,
        b * i + d * j,
        a * k + c * l + e,
        b * k + d * l + f,
    ]
}
fn point(m: Matrix, x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}
fn inverse_point(m: Matrix, x: f64, y: f64) -> Option<(f64, f64)> {
    let det = m[0] * m[3] - m[1] * m[2];
    if !det.is_finite() || det.abs() < f64::EPSILON {
        return None;
    }
    let (x, y) = (x - m[4], y - m[5]);
    Some(((m[3] * x - m[2] * y) / det, (-m[1] * x + m[0] * y) / det))
}

fn visit<T>(
    node: &RenderNode,
    parent: Option<Matrix>,
    f: &mut impl FnMut(&RenderNode, &TextRunNode, Matrix) -> Option<T>,
) -> Option<T> {
    if !node.visible {
        return None;
    }
    let frame = match &node.node_type {
        RenderNodeType::Rectangle(r)
            if r.transform.has_transform() && !node.children.is_empty() =>
        {
            let local = r.transform.matrix_about(node.bbox);
            Some(parent.map_or(local, |p| concat(p, local)))
        }
        _ => parent,
    };
    // Later sibling paint is in front. Keep the same composed run/owner, rather
    // than guessing a new line or mapping the physical point to body paragraph 0.
    for child in node.children.iter().rev() {
        if let Some(v) = visit(child, frame, f) {
            return Some(v);
        }
    }
    if let (Some(frame), RenderNodeType::TextRun(run)) = (frame, &node.node_type) {
        if run.cell_context.as_ref().is_some_and(|c| c.in_textbox) {
            return f(node, run, frame);
        }
    }
    None
}

fn cursor_json(
    node: &RenderNode,
    run: &TextRunNode,
    frame: Matrix,
    offset: usize,
    page: u32,
) -> Option<String> {
    let start = run.char_start?;
    let positions = run.replay_positions_for(&run.text);
    let local = offset.checked_sub(start)?;
    let advance = *positions.get(local)?;
    let y = node.bbox.y + run.baseline - run.style.font_size * 0.8;
    let (x, y) = point(frame, node.bbox.x + advance, y);
    let height = run.style.font_size * frame[2].hypot(frame[3]);
    let rotation = (-frame[2]).atan2(frame[3]).to_degrees();
    Some(format!("{{\"pageIndex\":{page},\"x\":{x:.1},\"y\":{y:.1},\"height\":{height:.1},\"rotation\":{rotation:.6}}}"))
}

pub(super) fn hit(root: &RenderNode, page: u32, x: f64, y: f64) -> Option<String> {
    visit(root, None, &mut |node, run, frame| {
        let (local_x, local_y) = inverse_point(frame, x, y)?;
        let b = node.bbox;
        if local_x < b.x || local_x > b.x + b.width || local_y < b.y || local_y > b.y + b.height {
            return None;
        }
        let ctx = run.cell_context.as_ref()?;
        let outer = ctx.path.first()?;
        let positions = run.replay_positions_for(&run.text);
        let offset = run.char_start? + find_char_at_x(&positions, local_x - b.x);
        let rect = cursor_json(node, run, frame, offset, page)?;
        let path:Vec<_>=ctx.path.iter().map(|p| serde_json::json!({"controlIndex":p.control_index,"cellIndex":p.cell_index,"cellParaIndex":p.cell_para_index})).collect();
        Some(format!("{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{offset},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{},\"cellPath\":{},\"isTextBox\":true,\"cursorRect\":{rect}}}",run.section_index?,run.para_index?,ctx.parent_para_index,outer.control_index,outer.cell_index,outer.cell_para_index,serde_json::to_string(&path).ok()?))
    })
}

pub(super) fn cursor(
    root: &RenderNode,
    section: usize,
    parent: usize,
    path: &[(usize, usize, usize)],
    offset: usize,
    page: u32,
) -> Option<String> {
    visit(root, None, &mut |node, run, frame| {
        let ctx = run.cell_context.as_ref()?;
        if run.section_index != Some(section)
            || ctx.parent_para_index != parent
            || ctx.path.len() != path.len()
            || !ctx
                .path
                .iter()
                .zip(path)
                .all(|(a, b)| (a.control_index, a.cell_index, a.cell_para_index) == *b)
        {
            return None;
        }
        cursor_json(node, run, frame, offset, page)
    })
}

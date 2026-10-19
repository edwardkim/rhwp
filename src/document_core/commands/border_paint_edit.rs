//! Explicit native-border replacement of proven source drawing fragments.
//! The journal owns only changed paragraphs, not a Document snapshot.
use crate::{
    document_core::DocumentCore,
    error::HwpError,
    model::{
        border_paint::{
            keep_sides, outside_binding, outside_owner, valid_paint, PaintRef, Resources,
        },
        control::Control,
        paragraph::Paragraph,
        table::Table,
    },
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(crate) const KEYS: [&str; 4] = ["borderLeft", "borderRight", "borderTop", "borderBottom"];

#[derive(Clone, Debug)]
pub(crate) struct ParagraphChange {
    cell: Option<usize>,
    paragraph: usize,
    before: Paragraph,
    after: Paragraph,
}
#[derive(Clone, Debug)]
pub(crate) struct BorderPaintUndo {
    target: (usize, usize, usize),
    table_id: u32,
    changes: Vec<ParagraphChange>,
}
fn error(s: &str) -> HwpError {
    HwpError::RenderError(s.into())
}
fn digest(p: &Paragraph) -> [u8; 32] {
    // Text insert + its inverse may recompose LineSeg without changing text,
    // styles, control slots or drawings. Validate semantic state, then restore
    // the captured source LineSeg together with the removed controls.
    let mut semantic = p.clone();
    semantic.line_segs.clear();
    Sha256::digest(serde_json::to_vec(&semantic).expect("Paragraph is serializable")).into()
}
// Parent journals capture slots/page paint, never another copy of live Table
// contents. Cell/style undo is applied to that live Table independently.
fn parent_capture(p: &Paragraph) -> Paragraph {
    let mut p = p.clone();
    for c in &mut p.controls {
        if let Control::Table(t) = c {
            t.cells.clear();
            t.zones.clear();
        }
    }
    p
}
pub(crate) fn restore_parent(mut capture: Paragraph, current: &Paragraph) -> Paragraph {
    for (i, c) in capture.controls.iter_mut().enumerate() {
        if matches!(c, Control::Table(_)) {
            *c = current.controls[i].clone();
        }
    }
    capture
}
pub(super) fn without(p: &Paragraph, remove: &[usize], updates: &[(usize, u8, bool)]) -> Paragraph {
    let mut after = p.clone();
    for &(i, mask, outside) in updates {
        keep_sides(&mut after.controls[i], mask, outside).expect("validated remaining source role");
    }
    let mut positions = after.logical_control_positions();
    for &i in remove.iter().rev() {
        after.controls.remove(i);
        if i < after.ctrl_data_records.len() {
            after.ctrl_data_records.remove(i);
        }
        if i < positions.len() {
            positions.remove(i);
        }
        for f in &mut after.field_ranges {
            if f.control_idx > i {
                f.control_idx -= 1;
            }
        }
    }
    after.rebuild_char_offsets(&positions);
    after
}
fn mask_of(sides: [bool; 4]) -> u8 {
    sides
        .iter()
        .enumerate()
        .fold(0, |mask, (i, b)| mask | if *b { 1 << i } else { 0 })
}
fn adjacent(table: &Table, ci: usize, peer: usize, dir: usize) -> bool {
    if ci == peer {
        return false;
    }
    let a = &table.cells[ci];
    let b = &table.cells[peer];
    let (x, y, r, d) = (
        u32::from(a.col),
        u32::from(a.row),
        u32::from(a.col) + u32::from(a.col_span),
        u32::from(a.row) + u32::from(a.row_span),
    );
    let (xx, yy, rr, dd) = (
        u32::from(b.col),
        u32::from(b.row),
        u32::from(b.col) + u32::from(b.col_span),
        u32::from(b.row) + u32::from(b.row_span),
    );
    match dir {
        0 => rr == x && yy < d && dd > y,
        1 => xx == r && yy < d && dd > y,
        2 => dd == y && xx < r && rr > x,
        _ => yy == d && xx < r && rr > x,
    }
}

/// Match the same logical shared edges used by the native BorderFill setter.
/// Partial merged-neighbour edges cannot be replaced by one native side.
pub(crate) fn plan(
    table: &Table,
    target: usize,
    sides: [bool; 4],
    parent: &Paragraph,
    table_index: usize,
    resources: &mut Resources<'_>,
) -> Result<Vec<ParagraphChange>, HwpError> {
    let c = table
        .cells
        .get(target)
        .ok_or_else(|| error("셀 인덱스 범위 초과"))?;
    let (r, b, x, e) = (
        u32::from(c.row),
        u32::from(c.row) + u32::from(c.row_span),
        u32::from(c.col),
        u32::from(c.col) + u32::from(c.col_span),
    );
    let mut changes = Vec::new();
    let mut outside = std::collections::BTreeSet::new();
    let mut remove_outside = Vec::new();
    let mut update_outside = Vec::new();
    let last_table = parent
        .controls
        .iter()
        .rposition(|c| matches!(c, Control::Table(_)))
        .unwrap_or(table_index);
    for (i, ctrl) in parent.controls.iter().enumerate() {
        let Some(rect) = PaintRef::from_control(ctrl) else {
            continue;
        };
        let Some((id, row, col, tag)) = outside_binding(rect) else {
            continue;
        };
        if id != table.common.instance_id {
            continue;
        }
        let Some(ci) = table
            .cells
            .iter()
            .position(|c| c.row == row && c.col == col)
        else {
            return Err(error("페이지 테두리 조각의 논리 셀이 없습니다"));
        };
        let cell = &table.cells[ci];
        let (nr, nb, nx, ne) = (
            u32::from(cell.row),
            u32::from(cell.row) + u32::from(cell.row_span),
            u32::from(cell.col),
            u32::from(cell.col) + u32::from(cell.col_span),
        );
        let selected = if ci == target {
            sides
        } else {
            [
                sides[1] && nx == e && nr < b && nb > r,
                sides[0] && ne == x && nr < b && nb > r,
                sides[3] && nr == b && nx < e && ne > x,
                sides[2] && nb == r && nx < e && ne > x,
            ]
        };
        let mask = tag
            .mask()
            .ok_or_else(|| error("잘못된 페이지 경계 변 집합"))?;
        let selected_mask = mask_of(selected);
        if mask & selected_mask == 0 {
            continue;
        }
        if outside_owner(rect, table, resources) != Some((ci, tag)) {
            return Err(error("페이지 테두리 그림 소유가 현재 배치와 다릅니다"));
        }
        if i <= last_table {
            return Err(error("페이지 테두리 제거가 표 슬롯을 이동시킵니다"));
        }
        for dir in 0..4 {
            if mask & (1 << dir) == 0 {
                continue;
            }
            outside.insert((tag.source_id, dir, ci));
            if selected[dir]
                && ci != target
                && ((dir < 2 && (nr < r || nb > b)) || (dir >= 2 && (nx < x || ne > e)))
            {
                return Err(error(
                    "병합 이웃의 공유 변 일부는 native 테두리로 교체할 수 없습니다",
                ));
            }
        }
        let remaining = mask & !selected_mask;
        if remaining == 0 {
            remove_outside.push(i);
        } else {
            update_outside.push((i, remaining, true));
        }
    }
    let mut sources = std::collections::BTreeSet::new();
    for (ci, cell) in table.cells.iter().enumerate() {
        for ctrl in cell.paragraphs.iter().flat_map(|p| &p.controls) {
            if let Some(paint) = PaintRef::from_control(ctrl) {
                if let Some(tag) = valid_paint(table.common.instance_id, cell, paint, resources) {
                    for dir in 0..4 {
                        if tag.mask().unwrap() & (1 << dir) != 0 {
                            sources.insert((tag.source_id, dir, ci));
                        }
                    }
                }
            }
        }
    }
    for (ci, cell) in table.cells.iter().enumerate() {
        let (nr, nb, nx, ne) = (
            u32::from(cell.row),
            u32::from(cell.row) + u32::from(cell.row_span),
            u32::from(cell.col),
            u32::from(cell.col) + u32::from(cell.col_span),
        );
        let shared = [
            sides[1] && nx == e && nr < b && nb > r,
            sides[0] && ne == x && nr < b && nb > r,
            sides[3] && nr == b && nx < e && ne > x,
            sides[2] && nb == r && nx < e && ne > x,
        ];
        let selected = if ci == target { sides } else { shared };
        if !selected.iter().any(|&s| s) {
            continue;
        }
        for (pi, p) in cell.paragraphs.iter().enumerate() {
            let mut remove = Vec::new();
            let mut updates = Vec::new();
            for (i, ctrl) in p.controls.iter().enumerate() {
                let Some(rect) = PaintRef::from_control(ctrl) else {
                    continue;
                };
                let Some(tag) = rect.common().cell_border_paint else {
                    continue;
                };
                let mask = tag.mask().ok_or_else(|| error("잘못된 셀 경계 변 집합"))?;
                let selected_mask = mask_of(selected);
                if mask & selected_mask == 0 {
                    continue;
                }
                if valid_paint(table.common.instance_id, cell, rect, resources) != Some(tag) {
                    return Err(error("셀 테두리 그림 소유가 현재 배치와 다릅니다"));
                }
                for dir in 0..4 {
                    if mask & selected_mask & (1 << dir) == 0 {
                        continue;
                    }
                    if !outside.contains(&(tag.source_id, dir, ci))
                        && !sources
                            .range(
                                (tag.source_id, dir ^ 1, 0)..=(tag.source_id, dir ^ 1, usize::MAX),
                            )
                            .any(|&(_, _, peer)| adjacent(table, ci, peer, dir))
                    {
                        return Err(error("표 밖 잔여 조각의 편집 소유가 확정되지 않은 테두리는 교체할 수 없습니다"));
                    }
                    if ci != target
                        && ((dir < 2 && (nr < r || nb > b)) || (dir >= 2 && (nx < x || ne > e)))
                    {
                        return Err(error(
                            "병합 이웃의 공유 변 일부는 native 테두리로 교체할 수 없습니다",
                        ));
                    }
                }
                let remaining = mask & !selected_mask;
                if remaining == 0 {
                    remove.push(i);
                } else {
                    updates.push((i, remaining, false));
                }
            }
            if remove.is_empty() && updates.is_empty() {
                continue;
            }
            let after = without(p, &remove, &updates);
            changes.push(ParagraphChange {
                cell: Some(ci),
                paragraph: pi,
                before: p.clone(),
                after,
            });
        }
    }
    if !remove_outside.is_empty() || !update_outside.is_empty() {
        let before = parent_capture(parent);
        let after = without(&before, &remove_outside, &update_outside);
        changes.push(ParagraphChange {
            cell: None,
            paragraph: 0,
            before,
            after,
        });
    }
    Ok(changes)
}
impl DocumentCore {
    pub(crate) fn commit_border_paint_plan(
        &mut self,
        target: (usize, usize, usize),
        changes: Vec<ParagraphChange>,
    ) -> Option<u32> {
        if changes.is_empty() {
            return None;
        }
        for change in &changes {
            if change.cell.is_none() {
                let p = &mut self.document.sections[target.0].paragraphs[target.1];
                *p = restore_parent(change.after.clone(), p);
            }
        }
        let table = self
            .get_table_mut(target.0, target.1, target.2)
            .expect("validated table");
        let table_id = table.common.instance_id;
        for change in &changes {
            if let Some(cell) = change.cell {
                table.cells[cell].paragraphs[change.paragraph] = change.after.clone();
            }
        }
        let id = self.next_border_paint_id;
        self.next_border_paint_id += 1;
        self.border_paint_store.push((
            id,
            BorderPaintUndo {
                target,
                table_id,
                changes,
            },
        ));
        Some(id)
    }

    /// Validate the complete batch in reverse order without touching the document.
    /// Repeated edits of one paragraph must form a continuous undo chain.
    pub(crate) fn border_paint_restores(
        &mut self,
        target: (usize, usize, usize),
        ids: &[u32],
    ) -> Result<BTreeMap<(Option<usize>, usize), Paragraph>, HwpError> {
        let mut restores = BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        for &id in ids.iter().rev() {
            if !seen.insert(id) {
                return Err(error("테두리 조각 undo ID 중복"));
            }
            let capture = self
                .border_paint_store
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| error("테두리 조각 undo ID 없음"))?;
            if capture.target != target {
                return Err(error("테두리 조각 undo 대상 불일치"));
            }
            let parent = self
                .document
                .sections
                .get(target.0)
                .and_then(|s| s.paragraphs.get(target.1))
                .ok_or_else(|| error("테두리 조각 문단 없음"))?;
            let Some(Control::Table(table)) = parent.controls.get(target.2) else {
                return Err(error("테두리 조각 표 없음"));
            };
            if table.common.instance_id != capture.table_id {
                return Err(error("테두리 조각 표 신원 불일치"));
            }
            for change in capture.changes {
                let key = (change.cell, change.paragraph);
                let parent_state = parent_capture(parent);
                let current = restores
                    .get(&key)
                    .or_else(|| {
                        if let Some(cell) = change.cell {
                            table
                                .cells
                                .get(cell)
                                .and_then(|c| c.paragraphs.get(change.paragraph))
                        } else {
                            Some(&parent_state)
                        }
                    })
                    .ok_or_else(|| error("테두리 조각 문단 없음"))?;
                if digest(current) != digest(&change.after) {
                    return Err(error("테두리 조각 undo 순서 또는 문단 상태 불일치"));
                }
                restores.insert(key, change.before);
            }
        }
        Ok(restores)
    }
    /// History owns journal lifetime, like discardSectionRaw.
    pub fn discard_border_paint_undo_native(&mut self, id: u32) {
        self.border_paint_store.retain(|(key, _)| *key != id);
    }
}

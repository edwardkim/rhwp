//! Explicit native-border replacement of proven source drawing fragments.
//! The journal owns only changed paragraphs, not a Document snapshot.
use crate::{
    document_core::DocumentCore,
    error::HwpError,
    model::{
        border_paint::{valid_side, CellSide},
        control::Control,
        paragraph::Paragraph,
        shape::ShapeObject,
        table::Table,
    },
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(crate) const KEYS: [&str; 4] = ["borderLeft", "borderRight", "borderTop", "borderBottom"];

#[derive(Clone, Debug)]
pub(crate) struct ParagraphChange {
    cell: usize,
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
fn side_index(side: CellSide) -> usize {
    match side {
        CellSide::Left => 0,
        CellSide::Right => 1,
        CellSide::Top => 2,
        CellSide::Bottom => 3,
    }
}

/// Match the same logical shared edges used by the native BorderFill setter.
/// Partial merged-neighbour edges cannot be replaced by one native side.
pub(crate) fn plan(
    table: &Table,
    target: usize,
    sides: [bool; 4],
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
    let mut sources = std::collections::BTreeSet::new();
    for (ci, cell) in table.cells.iter().enumerate() {
        for ctrl in cell.paragraphs.iter().flat_map(|p| &p.controls) {
            if let Control::Shape(s) = ctrl {
                if let ShapeObject::Rectangle(rect) = s.as_ref() {
                    if let Some(side) = valid_side(table.common.instance_id, cell, rect) {
                        sources.insert((
                            rect.common.cell_border_paint.unwrap().source_id,
                            side_index(side),
                            ci,
                        ));
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
            for (i, ctrl) in p.controls.iter().enumerate() {
                let Control::Shape(s) = ctrl else { continue };
                let ShapeObject::Rectangle(rect) = s.as_ref() else {
                    continue;
                };
                let Some(tag) = rect.common.cell_border_paint else {
                    continue;
                };
                let dir = side_index(tag.side);
                if !selected[dir] {
                    continue;
                }
                if valid_side(table.common.instance_id, cell, rect) != Some(tag.side) {
                    return Err(error("셀 테두리 그림 소유가 현재 배치와 다릅니다"));
                }
                if !sources
                    .range((tag.source_id, dir ^ 1, 0)..=(tag.source_id, dir ^ 1, usize::MAX))
                    .any(|&(_, _, peer)| peer != ci)
                {
                    return Err(error(
                        "표 밖 잔여 조각의 편집 소유가 확정되지 않은 테두리는 교체할 수 없습니다",
                    ));
                }
                if ci != target
                    && ((dir < 2 && (nr < r || nb > b)) || (dir >= 2 && (nx < x || ne > e)))
                {
                    return Err(error(
                        "병합 이웃의 공유 변 일부는 native 테두리로 교체할 수 없습니다",
                    ));
                }
                remove.push(i);
            }
            if remove.is_empty() {
                continue;
            }
            let mut after = p.clone();
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
            changes.push(ParagraphChange {
                cell: ci,
                paragraph: pi,
                before: p.clone(),
                after,
            });
        }
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
        let table = self
            .get_table_mut(target.0, target.1, target.2)
            .expect("validated table");
        let table_id = table.common.instance_id;
        for change in &changes {
            table.cells[change.cell].paragraphs[change.paragraph] = change.after.clone();
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
    ) -> Result<BTreeMap<(usize, usize), Paragraph>, HwpError> {
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
            let table = self.get_table_mut(target.0, target.1, target.2)?;
            if table.common.instance_id != capture.table_id {
                return Err(error("테두리 조각 표 신원 불일치"));
            }
            for change in capture.changes {
                let key = (change.cell, change.paragraph);
                let current = restores
                    .get(&key)
                    .or_else(|| {
                        table
                            .cells
                            .get(change.cell)
                            .and_then(|c| c.paragraphs.get(change.paragraph))
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

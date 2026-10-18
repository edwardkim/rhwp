//! Bind hierarchical source paint to actual nested Table owners, deepest first.
use super::*;

pub(super) fn source_paint_family(grids: &[Value], root: usize) -> Result<bool, String> {
    for (candidate, grid) in grids.iter().enumerate() {
        // Content-backed cell separation can have no ink at a shared side.
        // The uniform-border legacy route would invent that side (or defer on
        // differing partial rules). Use the same per-cell paint/native plan as
        // periodic families; semantic ownership does not imply uniform paint.
        if grid["regularizedPeriodicBoundary"] != true
            && grid["cellRegionEvidence"]["status"] != "Selected"
        {
            continue;
        }
        let mut at = candidate;
        for _ in 0..=16 {
            if at == root {
                return Ok(true);
            }
            let parent = &grids.get(at).ok_or("missing table ancestor")?["hierarchy"]["parent"];
            if parent.is_null() {
                break;
            }
            at = index(&parent["table"])?;
        }
    }
    Ok(false)
}

pub(super) struct Owner {
    pub root: usize,
    pub candidate: usize,
    pub instance_id: u32,
    depth: usize,
}

pub(super) fn owners(
    tables: &[Control],
    roots: &[usize],
    grids: &[Value],
    table_id_base: u32,
) -> Result<Vec<Owner>, String> {
    fn visit(
        table: &Table,
        root: usize,
        depth: usize,
        grids: &[Value],
        base: u32,
        out: &mut Vec<Owner>,
    ) -> Result<(), String> {
        if depth >= 8 || out.len() >= 128 {
            return Err("source paint table family budget exceeded".into());
        }
        let candidate = table
            .common
            .instance_id
            .checked_sub(base)
            .and_then(|i| usize::try_from(i).ok())
            .filter(|&i| i < grids.len())
            .ok_or("source paint table has no candidate owner")?;
        out.push(Owner {
            root,
            candidate,
            instance_id: table.common.instance_id,
            depth,
        });
        for cell in &table.cells {
            for para in &cell.paragraphs {
                for control in &para.controls {
                    if let Control::Table(child) = control {
                        visit(child, root, depth + 1, grids, base, out)?;
                    }
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    for &root in roots {
        let Some(Control::Table(table)) = tables.get(root) else {
            return Err("missing source paint root table".into());
        };
        visit(table, root, 0, grids, table_id_base, &mut out)?;
    }
    // The parent's cell contains the child's region. The remaining physical
    // pieces, after the child consumes them, are the parent's unique paint.
    out.sort_by_key(|o| (std::cmp::Reverse(o.depth), o.root, o.candidate));
    Ok(out)
}

pub(super) fn table_mut(control: &mut Control, instance: u32) -> Option<&mut Table> {
    let Control::Table(table) = control else {
        return None;
    };
    if table.common.instance_id == instance {
        return Some(table);
    }
    for cell in &mut table.cells {
        for para in &mut cell.paragraphs {
            for child in &mut para.controls {
                if let Some(found) = table_mut(child, instance) {
                    return Some(found);
                }
            }
        }
    }
    None
}

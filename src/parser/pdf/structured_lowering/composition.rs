//! Bind hierarchical source paint to actual nested Table owners, deepest first.
use super::*;

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
    allocation: Allocation<'_>,
) -> Result<Vec<Owner>, String> {
    fn visit(
        table: &Table,
        root: usize,
        depth: usize,
        grids: &[Value],
        base: u32,
        out: &mut Vec<Owner>,
        allocation: Allocation<'_>,
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
        allocation
            .push(
                out,
                Owner {
                    root,
                    candidate,
                    instance_id: table.common.instance_id,
                    depth,
                },
            )
            .map_err(message)?;
        for cell in &table.cells {
            for para in &cell.paragraphs {
                for control in &para.controls {
                    if let Control::Table(child) = control {
                        visit(child, root, depth + 1, grids, base, out, allocation)?;
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
        visit(table, root, 0, grids, table_id_base, &mut out, allocation)?;
    }
    // The parent's cell contains the child's region. The remaining physical
    // pieces, after the child consumes them, are the parent's unique paint.
    allocation
        .sort_by(&mut out, |a, b| {
            (std::cmp::Reverse(a.depth), a.root, a.candidate).cmp(&(
                std::cmp::Reverse(b.depth),
                b.root,
                b.candidate,
            ))
        })
        .map_err(message)?;
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

/// Discard whole failed families before any source paint/content is consumed.
pub(super) fn discard_roots(
    tables: &mut Vec<Control>,
    candidates: &mut Vec<usize>,
    consumed: &mut Vec<BTreeSet<usize>>,
    preserve: &mut Vec<usize>,
    held: &mut Vec<Value>,
    rejected: &BTreeSet<usize>,
    reason: &str,
    allocation: Allocation<'_>,
) -> Result<(), String> {
    let keep_paint = allocation
        .set_from(preserve.iter().copied())
        .map_err(message)?;
    preserve.clear();
    for (i, ((table, candidate), removed)) in std::mem::take(tables)
        .into_iter()
        .zip(std::mem::take(candidates))
        .zip(std::mem::take(consumed))
        .enumerate()
    {
        if rejected.contains(&i) {
            allocation
                .push(
                    held,
                    request_json!(allocation, {"candidate":candidate,"reason":reason})?,
                )
                .map_err(message)?;
        } else {
            if keep_paint.contains(&i) {
                allocation.push(preserve, tables.len()).map_err(message)?;
            }
            allocation.push(tables, table).map_err(message)?;
            allocation.push(candidates, candidate).map_err(message)?;
            allocation.push(consumed, removed).map_err(message)?;
        }
    }
    Ok(())
}

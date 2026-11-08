//! Guarded spatial substitution into cell slots, inspired by nonsymmetric operad composition.
use super::allocation::{message, Allocation};
use pdf_oxide::geometry::Rect;

const EPS: f32 = 0.5;
const MAX_DEPTH: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Embedding {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Embedding {
    pub const IDENTITY: Self = Self {
        x: 0.,
        y: 0.,
        width: 1.,
        height: 1.,
    };
    /// self(child(u)): local unit-square embeddings, not absolute PDF translations.
    pub fn compose(self, child: Self) -> Self {
        Self {
            x: self.x + self.width * child.x,
            y: self.y + self.height * child.y,
            width: self.width * child.width,
            height: self.height * child.height,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub parent: Option<(usize, usize)>,
    pub depth: usize,
    pub embedding: Option<Embedding>,
    pub deferred: Option<String>,
}

fn contains(a: Rect, b: Rect) -> bool {
    b.x >= a.x && b.y >= a.y && b.x + b.width <= a.x + a.width && b.y + b.height <= a.y + a.height
}
fn strict(a: Rect, b: Rect) -> bool {
    b.x > a.x + EPS
        && b.y > a.y + EPS
        && b.x + b.width < a.x + a.width - EPS
        && b.y + b.height < a.y + a.height - EPS
}
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

/// Tables and cells must already have passed local geometry and cellular-chain validation.
pub fn compose(
    allocation: Allocation<'_>,
    tables: &[(Rect, Vec<Rect>)],
) -> Result<Vec<Placement>, String> {
    if tables.len() > 128 {
        return Err("table composition budget exceeded".into());
    }
    for (b, cells) in tables {
        if std::iter::once(b).chain(cells.iter()).any(|r| {
            ![r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite())
                || r.width <= 0.
                || r.height <= 0.
        }) {
            return Err("invalid table composition geometry".into());
        }
    }
    let mut placements = allocation.vec(tables.len()).map_err(message)?;
    placements.resize_with(tables.len(), || Placement {
        parent: None,
        depth: 0,
        embedding: None,
        deferred: None,
    });
    for (child, (b, _)) in tables.iter().enumerate() {
        let parent = tables
            .iter()
            .enumerate()
            .filter(|(i, (outer, _))| {
                *i != child
                    && contains(*outer, *b)
                    && outer.width * outer.height > b.width * b.height
            })
            .min_by(|a, b| {
                (a.1 .0.width * a.1 .0.height).total_cmp(&(b.1 .0.width * b.1 .0.height))
            });
        if let Some((pi, (_, cells))) = parent {
            let mut owners = cells.iter().enumerate().filter(|(_, c)| strict(**c, *b));
            let first = owners.next();
            let unique = first.filter(|_| owners.next().is_none());
            if let Some((ci, c)) = unique {
                placements[child].parent = Some((pi, ci));
                placements[child].embedding = Some(Embedding {
                    x: (b.x - c.x) / c.width,
                    y: (b.y - c.y) / c.height,
                    width: b.width / c.width,
                    height: b.height / c.height,
                });
            } else {
                placements[child].deferred = Some(
                    allocation
                        .string("child grid crosses or touches parent cell boundary")
                        .map_err(message)?,
                );
                placements[pi].deferred = Some(
                    allocation
                        .string("unresolved child grid attachment")
                        .map_err(message)?,
                );
            }
        }
    }
    for a in 0..tables.len() {
        for b in a + 1..tables.len() {
            let (aa, bb) = (tables[a].0, tables[b].0);
            if overlaps(aa, bb) && !strict(aa, bb) && !strict(bb, aa) {
                placements[a].deferred = Some(
                    allocation
                        .string("non-laminar or touching table candidates")
                        .map_err(message)?,
                );
                placements[b].deferred = Some(
                    allocation
                        .string("non-laminar or touching table candidates")
                        .map_err(message)?,
                );
            }
        }
    }
    for child in 0..placements.len() {
        let mut at = child;
        let mut depth = 0;
        while let Some((parent, _)) = placements[at].parent {
            depth += 1;
            if depth > MAX_DEPTH {
                placements[child].deferred = Some(
                    allocation
                        .string("table composition depth exceeded")
                        .map_err(message)?,
                );
                break;
            }
            at = parent;
        }
        placements[child].depth = depth;
    }
    // A held subtree cannot silently leave its enclosing candidate eligible for materialization.
    for _ in 0..=MAX_DEPTH {
        let mut changed = false;
        for i in 0..placements.len() {
            if let Some((p, _)) = placements[i].parent {
                if placements[i].deferred.is_some() != placements[p].deferred.is_some() {
                    let held = if placements[i].deferred.is_none() {
                        i
                    } else {
                        p
                    };
                    placements[held].deferred = Some(
                        allocation
                            .string("unresolved table composition family")
                            .map_err(message)?,
                    );
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    Ok(placements)
}

pub fn ancestor(placements: &[Placement], ancestor: usize, mut child: usize) -> bool {
    for _ in 0..=MAX_DEPTH {
        if child == ancestor {
            return true;
        }
        let Some((parent, _)) = placements[child].parent else {
            return false;
        };
        child = parent;
    }
    false
}

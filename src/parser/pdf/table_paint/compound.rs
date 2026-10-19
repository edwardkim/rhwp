//! Editing authority for disconnected axial subpaths of one source stroke.
//! Intersections retain the union of their actual ink owners, not their bbox owners.
use super::*;
use crate::model::{
    bin_data::{self, BinDataContent},
    border_paint::CellBorderPaint,
    pdf_stroke::PdfStroke,
};

type CellPart = (Frame, Option<CellBorderPaint>);
type OutsidePart = (Frame, Option<(usize, usize, CellBorderPaint)>);

struct Band {
    axis: StrokeAxis,
    frame: Frame,
    ink: Vec<Frame>,
}
pub(super) struct Compound {
    target: usize,
    bands: Vec<Band>,
    source_id: u32,
}

fn mapped(rect: [f64; 4], source: Frame, size: [f64; 2]) -> Frame {
    let x = |p: f64| source.x + (p * (source.right - source.x) as f64 / size[0]).round() as i64;
    let y = |p: f64| source.y + (p * (source.bottom - source.y) as f64 / size[1]).round() as i64;
    Frame {
        x: x(rect[0]),
        y: y(rect[1]),
        right: x(rect[2]),
        bottom: y(rect[3]),
    }
}

// A coverage envelope rounds outwards in the native HWPUNIT lattice. This is
// ownership of source ink, not a relocation of its exact vector coordinates.
fn support(rect: [f64; 4], source: Frame, size: [f64; 2]) -> Frame {
    let x = |p: f64| p * (source.right - source.x) as f64 / size[0];
    let y = |p: f64| p * (source.bottom - source.y) as f64 / size[1];
    Frame {
        x: source.x + x(rect[0]).floor() as i64,
        y: source.y + y(rect[1]).floor() as i64,
        right: source.x + x(rect[2]).ceil() as i64,
        bottom: source.y + y(rect[3]).ceil() as i64,
    }
}
#[allow(clippy::too_many_arguments)]

pub(super) fn plan(
    original: &Control,
    source: Frame,
    path: &pdf_oxide::elements::PathContent,
    control: usize,
    targets: &[Target<'_>],
    frames: &[Vec<Frame>],
    required: &[BTreeSet<usize>],
    resources: &[BinDataContent],
    work: &mut usize,
) -> Result<Option<Compound>, String> {
    use pdf_oxide::elements::{LineCap, PathOperation};
    if path.operations.len()<4 || path.operations.len()>64 || !path.operations.len().is_multiple_of(2)
        || path.fill_color.is_some() || path.stroke_color.is_none() || path.matrix.is_some()
        || path.line_cap!=LineCap::Butt
        || !path.operations.chunks_exact(2).all(|p|matches!(p,[PathOperation::MoveTo(x,y),PathOperation::LineTo(a,b)] if (x==a&&y!=b)||(y==b&&x!=a))) {return Ok(None);}
    let Control::Picture(p) = original else {
        return Ok(None);
    };
    let Some(i) = bin_data::content_index(resources, p.image_attr.bin_data_id) else {
        return Ok(None);
    };
    let Some(bytes) = resources[i].data.load_limited(bin_data::MAX_BIN_DATA_BYTES) else {
        return Ok(None);
    };
    let Some(stroke) = PdfStroke::read(&bytes) else {
        return Ok(None);
    };
    if stroke.segments.len() + 1 != path.operations.len() / 2 {
        return Ok(None);
    }
    let mut bands = Vec::new();
    let mut ink_count = 0;
    for e in std::iter::once(stroke.endpoints).chain(stroke.segments.iter().copied()) {
        tick(work)?;
        let horizontal = e[1] == e[3];
        let endpoints = mapped([e[0], e[1], e[2], e[3]], source, stroke.frame);
        let axis = StrokeAxis {
            horizontal,
            at: if horizontal { endpoints.y } else { endpoints.x },
            from: if horizontal {
                endpoints.x.min(endpoints.right)
            } else {
                endpoints.y.min(endpoints.bottom)
            },
            to: if horizontal {
                endpoints.x.max(endpoints.right)
            } else {
                endpoints.y.max(endpoints.bottom)
            },
        };
        let rect = if horizontal {
            [
                e[0].min(e[2]),
                e[1] - stroke.width / 2.,
                e[0].max(e[2]),
                e[1] + stroke.width / 2.,
            ]
        } else {
            [
                e[0] - stroke.width / 2.,
                e[1].min(e[3]),
                e[0] + stroke.width / 2.,
                e[1].max(e[3]),
            ]
        };
        let Some(frame) = intersection(support(rect, source, stroke.frame), source, None) else {
            return Ok(None);
        };
        let Some(rects) = stroke.segment_rectangles(e) else {
            return Ok(None);
        };
        ink_count += rects.len();
        if ink_count > 8192 {
            return Ok(None);
        }
        let ink = rects
            .into_iter()
            .filter_map(|r| intersection(support(r, source, stroke.frame), source, None))
            .collect();
        bands.push(Band { axis, frame, ink });
    }
    let mut target = None;
    for (ti, t) in targets.iter().enumerate() {
        tick(work)?;
        if !t.root || !required[ti].contains(&control) {
            continue;
        }
        // Ownership follows the actual cell perimeters, including merged cells.
        // The segment-coverage proof below rejects ink crossing a cell interior;
        // native side replacement separately rejects a partial merged neighbour.
        let region = bounds(t.grid)?;
        let mut certified = true;
        for band in &bands {
            tick(work)?;
            let a = band.axis;
            let (lo, hi) = if a.horizontal {
                (region.x, region.right)
            } else {
                (region.y, region.bottom)
            };
            if a.from < lo - 1 || a.to > hi + 1 || !frames[ti].iter().any(|c| side(a, *c).is_some())
            {
                certified = false;
                break;
            }
            // The entire segment must follow logical edges, including empty cells.
            let mut covered = Vec::new();
            for c in &frames[ti] {
                tick(work)?;
                if side(a, *c).is_some() {
                    covered.push(if a.horizontal {
                        (c.x, c.right)
                    } else {
                        (c.y, c.bottom)
                    });
                }
            }
            covered.sort_unstable();
            let mut end = a.from;
            for (x, y) in covered {
                if x <= end + 1 && y > end {
                    end = y;
                }
            }
            if end < a.to - 1 {
                certified = false;
                break;
            }
        }
        if certified {
            // A descendant sharing a source boundary needs a separate logical
            // edge-closure contract. Mere geometric contact is not edit ownership.
            for (other, cells) in frames.iter().enumerate() {
                if other == ti {
                    continue;
                }
                for cell in cells {
                    for band in &bands {
                        tick(work)?;
                        if intersection(*cell, band.frame, None).is_some() {
                            certified = false;
                        }
                    }
                }
            }
        }
        if certified {
            if target.is_some() {
                return Ok(None);
            }
            target = Some(ti);
        }
    }
    Ok(target.map(|target| Compound {
        target,
        bands,
        source_id: common(original).unwrap().instance_id,
    }))
}
fn side(axis: StrokeAxis, cell: Frame) -> Option<usize> {
    let (a, b, start, end, first) = if axis.horizontal {
        (cell.y, cell.bottom, cell.x, cell.right, 2)
    } else {
        (cell.x, cell.right, cell.y, cell.bottom, 0)
    };
    if axis.from >= end || axis.to <= start {
        return None;
    }
    match ((axis.at - a).abs() <= 1, (axis.at - b).abs() <= 1) {
        (true, false) => Some(first),
        (false, true) => Some(first + 1),
        _ => None,
    }
}

// Rectangular set overlay. Every output piece has one physical owner; masks
// describe which surviving source axes still require its ink.
fn overlay(
    pieces: &mut Vec<(Frame, u8)>,
    cut: Frame,
    bit: u8,
    work: &mut usize,
) -> Result<(), String> {
    let mut next = Vec::new();
    for (frame, mask) in pieces.drain(..) {
        tick(work)?;
        if let Some(hit) = intersection(frame, cut, None) {
            let mut rest = vec![frame];
            subtract(&mut rest, hit, None, work)?;
            next.extend(rest.into_iter().map(|r| (r, mask)));
            next.push((hit, mask | bit));
        } else {
            next.push((frame, mask));
        }
        if next.len() > MAX_FRAGMENTS {
            return Err("compound paint fragment budget exceeded".into());
        }
    }
    *pieces = next;
    Ok(())
}
impl Compound {
    fn split(
        &self,
        piece: Frame,
        cell: Frame,
        outer: Option<Frame>,
        work: &mut usize,
    ) -> Result<Vec<CellPart>, String> {
        let mut pieces = vec![(piece, 0)];
        let mut candidates = Vec::new();
        for band in &self.bands {
            tick(work)?;
            let Some(dir) = side(band.axis, cell) else {
                continue;
            };
            if let Some(region) = outer {
                if !match dir {
                    0 => cell.x == region.x,
                    1 => cell.right == region.right,
                    2 => cell.y == region.y,
                    _ => cell.bottom == region.bottom,
                } {
                    continue;
                }
            }
            let clip = if band.axis.horizontal {
                Frame {
                    x: cell.x,
                    right: cell.right,
                    ..band.frame
                }
            } else {
                Frame {
                    y: cell.y,
                    bottom: cell.bottom,
                    ..band.frame
                }
            };
            if let Some(cut) = intersection(band.frame, clip, None) {
                overlay(&mut pieces, cut, 1 << dir, work)?;
                candidates.push((band, dir));
            }
        }
        let mut result = Vec::new();
        for (frame, mask) in pieces {
            if mask.count_ones() <= 1 {
                result.push((frame, CellBorderPaint::from_mask(mask, self.source_id)));
                continue;
            }
            // Coarse bands overlap at joints. Refine only these small regions
            // by actual dash ink, so a gap cannot preserve the removed axis.
            let mut joint = vec![(frame, 0)];
            for (band, dir) in &candidates {
                if mask & (1 << dir) == 0 {
                    continue;
                }
                for &ink in &band.ink {
                    tick(work)?;
                    if intersection(frame, ink, None).is_some() {
                        overlay(&mut joint, ink, 1 << dir, work)?;
                    }
                }
            }
            result.extend(
                joint
                    .into_iter()
                    .map(|(r, m)| (r, CellBorderPaint::from_mask(m, self.source_id))),
            );
            if result.len() > MAX_FRAGMENTS {
                return Err("compound paint fragment budget exceeded".into());
            }
        }
        Ok(result)
    }
    pub(super) fn cell(
        &self,
        ti: usize,
        piece: Frame,
        cell: Frame,
        work: &mut usize,
    ) -> Result<Vec<CellPart>, String> {
        if ti != self.target {
            // A path's raster frame can enclose a descendant without any of
            // its ink entering that cell. Do not materialize that empty crop
            // as a selectable, cell-sized picture. Use the certified source
            // stroke envelopes, including width and outward HU rounding;
            // bitmap/frame overlap alone is not evidence of painted content.
            let mut painted = false;
            for band in &self.bands {
                tick(work)?;
                painted |= intersection(band.frame, piece, None).is_some();
            }
            if !painted {
                return Ok(Vec::new());
            }
            return Ok(vec![(piece, None)]);
        }
        // Every certified segment follows this grid. The untagged remainder
        // has no PDF ink. Keeping it as a native crop can reintroduce filtered
        // edge samples after a neighbouring owned stroke is explicitly erased.
        Ok(self
            .split(piece, cell, None, work)?
            .into_iter()
            .filter(|(_, tag)| tag.is_some())
            .collect())
    }
    pub(super) fn outside(
        &self,
        remaining: Vec<Frame>,
        frames: &[Vec<Frame>],
        targets: &[Target<'_>],
        work: &mut usize,
    ) -> Result<Vec<OutsidePart>, String> {
        let ti = self.target;
        let region = bounds(targets[ti].grid)?;
        let mut unowned = remaining;
        let mut out = Vec::new();
        for (ci, cell) in frames[ti].iter().enumerate() {
            let mut next = Vec::new();
            for frame in unowned {
                for (r, tag) in self.split(frame, *cell, Some(region), work)? {
                    if let Some(tag) = tag {
                        out.push((r, Some((ti, ci, tag))));
                    } else {
                        next.push(r);
                    }
                }
                if out.len() + next.len() > MAX_FRAGMENTS {
                    return Err("compound outside fragment budget exceeded".into());
                }
            }
            unowned = next;
        }
        // All segments were certified inside this root. Remaining page regions
        // are geometrically empty source padding, not another paint owner.
        Ok(out)
    }
}

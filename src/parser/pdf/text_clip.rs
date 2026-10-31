//! Carry the actual text-show clip chain into editable TextBox IR.
use super::allocation::{message, Allocation};
use crate::model::text_clip::{ClipCommand as C, TextClipPath};
use pdf_oxide::elements::{PathClip, PathOperation as P};
pub(super) fn has_nonrectangular(mut clip: Option<&PathClip>) -> bool {
    let mut depth = 0;
    while let Some(p) = clip {
        depth += 1;
        if depth > 32 || super::table_inference::rectangle(&p.operations).is_none() {
            return true;
        }
        clip = p.parent.as_deref();
    }
    false
}
pub(super) fn local_paths(
    clip: Option<&PathClip>,
    x: f32,
    top: f32,
    allocation: Allocation<'_>,
) -> Result<Option<Vec<TextClipPath>>, String> {
    local_paths_mapped(
        clip,
        false,
        |a, b| {
            (
                (f64::from(a) - f64::from(x)) * 100.,
                (f64::from(top) - f64::from(b)) * 100.,
            )
        },
        allocation,
    )
}

pub(super) fn local_paths_mapped(
    mut clip: Option<&PathClip>,
    force: bool,
    at: impl Fn(f32, f32) -> (f64, f64),
    allocation: Allocation<'_>,
) -> Result<Option<Vec<TextClipPath>>, String> {
    let mut paths = Vec::new();
    let mut nonrect = force;
    while let Some(p) = clip {
        if paths.len() == 32 {
            return Err("text clip depth exceeded".into());
        }
        nonrect |= super::table_inference::rectangle(&p.operations).is_none();
        if p.operations.len().saturating_mul(5)
            + paths
                .iter()
                .map(|p: &TextClipPath| p.commands.len())
                .sum::<usize>()
            > 16384
        {
            return Err("text clip operation budget exceeded".into());
        }
        let mut commands = Vec::new();
        for op in &p.operations {
            match *op {
                P::MoveTo(a, b) => {
                    let (a, b) = at(a, b);
                    allocation
                        .push(&mut commands, C::Move(a, b))
                        .map_err(message)?;
                }
                P::LineTo(a, b) => {
                    let (a, b) = at(a, b);
                    allocation
                        .push(&mut commands, C::Line(a, b))
                        .map_err(message)?;
                }
                P::CurveTo(a, b, c, d, e, f) => {
                    let (a, b) = at(a, b);
                    let (c, d) = at(c, d);
                    let (e, f) = at(e, f);
                    allocation
                        .push(&mut commands, C::Cubic(a, b, c, d, e, f))
                        .map_err(message)?;
                }
                P::Rectangle(a, b, w, h) => {
                    let (a0, b0) = at(a, b);
                    let (a1, b1) = at(a + w, b);
                    let (a2, b2) = at(a + w, b + h);
                    let (a3, b3) = at(a, b + h);
                    allocation.reserve(&mut commands, 5).map_err(message)?;
                    commands.extend([
                        C::Move(a0, b0),
                        C::Line(a1, b1),
                        C::Line(a2, b2),
                        C::Line(a3, b3),
                        C::Close,
                    ]);
                }
                P::ClosePath => allocation.push(&mut commands, C::Close).map_err(message)?,
            }
        }
        allocation
            .push(
                &mut paths,
                TextClipPath {
                    commands,
                    even_odd: p.even_odd,
                },
            )
            .map_err(message)?;
        clip = p.parent.as_deref();
    }
    if !nonrect || paths.is_empty() {
        return Ok(None);
    }
    if !crate::model::text_clip::valid(&paths) {
        return Err("invalid or excessive text clip geometry".into());
    }
    Ok(Some(paths))
}

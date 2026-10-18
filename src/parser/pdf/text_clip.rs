//! Carry the actual text-show clip chain into editable TextBox IR.
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
    mut clip: Option<&PathClip>,
    x: f32,
    top: f32,
) -> Result<Option<Vec<TextClipPath>>, String> {
    let mut paths = Vec::new();
    let mut nonrect = false;
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
        let at = |a: f32, b: f32| {
            (
                (f64::from(a) - f64::from(x)) * 100.,
                (f64::from(top) - f64::from(b)) * 100.,
            )
        };
        for op in &p.operations {
            match *op {
                P::MoveTo(a, b) => {
                    let (a, b) = at(a, b);
                    commands.push(C::Move(a, b));
                }
                P::LineTo(a, b) => {
                    let (a, b) = at(a, b);
                    commands.push(C::Line(a, b));
                }
                P::CurveTo(a, b, c, d, e, f) => {
                    let (a, b) = at(a, b);
                    let (c, d) = at(c, d);
                    let (e, f) = at(e, f);
                    commands.push(C::Cubic(a, b, c, d, e, f));
                }
                P::Rectangle(a, b, w, h) => {
                    let (a0, b0) = at(a, b);
                    let (a1, b1) = at(a + w, b + h);
                    commands.extend([
                        C::Move(a0, b0),
                        C::Line(a1, b0),
                        C::Line(a1, b1),
                        C::Line(a0, b1),
                        C::Close,
                    ]);
                }
                P::ClosePath => commands.push(C::Close),
            }
        }
        paths.push(TextClipPath {
            commands,
            even_odd: p.even_odd,
        });
        clip = p.parent.as_deref();
    }
    if !nonrect {
        return Ok(None);
    }
    if !crate::model::text_clip::valid(&paths) {
        return Err("invalid or excessive text clip geometry".into());
    }
    Ok(Some(paths))
}

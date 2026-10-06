use std::sync::Arc;

use pdf_writer::types::{PaintType, TilingType};
use pdf_writer::{Chunk, Content, Filter, Ref};
use usvg::{Opacity, Pattern, Transform};

use super::group;
use crate::util::context::Context;
use crate::util::helper::TransformExt;
use crate::util::resources::ResourceContainer;
use crate::Result;

/// Turn a pattern into a PDF tiling pattern.
pub fn create(
    pattern: Arc<Pattern>,
    chunk: &mut Chunk,
    ctx: &mut Context,
    matrix: Transform,
    initial_opacity: Option<Opacity>,
) -> Result<Ref> {
    let pattern_ref = ctx.alloc_ref();
    let mut rc = ResourceContainer::new();

    let pattern_rect = pattern.rect();

    let pattern_matrix = matrix.pre_concat(pattern.transform()).pre_concat(
        Transform::from_row(1.0, 0.0, 0.0, 1.0, pattern_rect.x(), pattern_rect.y()),
    );

    let mut content = Content::new();
    group::render(
        pattern.root(),
        chunk,
        &mut content,
        ctx,
        Transform::default(),
        initial_opacity,
        &mut rc,
    )?;

    let content_stream = ctx.finish_content(content);

    let mut tiling_pattern = chunk.tiling_pattern(pattern_ref, &content_stream);

    if ctx.options.compress {
        tiling_pattern.filter(Filter::FlateDecode);
    }

    rc.finish(&mut tiling_pattern.resources());

    // We already account for the x/y of the pattern by appending it to the matrix above, so here we just need to take the height / width
    // in consideration
    let final_bbox =
        pdf_writer::Rect::new(0.0, 0.0, pattern_rect.width(), pattern_rect.height());

    tiling_pattern
        .tiling_type(TilingType::ConstantSpacing)
        .paint_type(PaintType::Colored)
        .bbox(final_bbox)
        .matrix(pattern_matrix.to_pdf_transform())
        .x_step(final_bbox.x2 - final_bbox.x1)
        .y_step(final_bbox.y2 - final_bbox.y1);

    Ok(pattern_ref)
}

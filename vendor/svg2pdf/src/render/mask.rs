use pdf_writer::{Chunk, Content, Filter, Finish, Ref};
use usvg::{Group, Mask, Transform};

use super::group;
use crate::util::context::Context;
use crate::util::helper::{clip_to_rect, ContentExt, MaskTypeExt, NameExt, RectExt};
use crate::util::resources::ResourceContainer;
use crate::Result;

/// Render a mask into a content stream.
pub fn render(
    group: &Group,
    mask: &Mask,
    chunk: &mut Chunk,
    content: &mut Content,
    ctx: &mut Context,
    rc: &mut ResourceContainer,
) -> Result<()> {
    let mask_ref = create(group, mask, chunk, ctx)?;
    let mask_name = rc.add_graphics_state(mask_ref);
    content.set_parameters(mask_name.to_pdf_name());

    Ok(())
}

/// Create a mask and return the object reference to it.
pub fn create(
    parent: &Group,
    mask: &Mask,
    chunk: &mut Chunk,
    ctx: &mut Context,
) -> Result<Ref> {
    let x_ref = ctx.alloc_ref();
    let mut rc = ResourceContainer::new();

    let mut content = Content::new();
    content.save_state_checked()?;

    if let Some(mask) = mask.mask() {
        render(parent, mask, chunk, &mut content, ctx, &mut rc)?;
    }

    let rect = mask.rect();

    // In addition to setting the bounding box, we also apply a clip path to the mask rect to
    // circumvent a bug in Firefox where the bounding box is not applied properly for some transforms.
    // If we don't do this, the "half-width-region-with-rotation.svg" test case won't render properly.
    clip_to_rect(rect, &mut content);
    group::render(
        mask.root(),
        chunk,
        &mut content,
        ctx,
        Transform::default(),
        None,
        &mut rc,
    )?;

    content.restore_state();
    let content_stream = ctx.finish_content(content);

    let mut x_object = chunk.form_xobject(x_ref, &content_stream);
    rc.finish(&mut x_object.resources());

    if ctx.options.compress {
        x_object.filter(Filter::FlateDecode);
    }

    x_object
        .group()
        .transparency()
        .isolated(false)
        .knockout(false)
        .color_space()
        .icc_based(ctx.srgb_ref());

    x_object.bbox(rect.to_pdf_rect());
    x_object.finish();

    let gs_ref = ctx.alloc_ref();
    let mut gs = chunk.ext_graphics(gs_ref);
    gs.soft_mask().subtype(mask.kind().to_pdf_mask_type()).group(x_ref);

    Ok(gs_ref)
}

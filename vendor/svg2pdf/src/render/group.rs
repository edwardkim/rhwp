use crate::ConversionError::UnknownError;
use pdf_writer::{Chunk, Content, Filter, Finish, Ref};
use std::ops::Mul;
use usvg::{Opacity, Transform};

#[cfg(feature = "filters")]
use super::filter;
use super::{clip_path, mask, Render};
use crate::util::context::Context;
use crate::util::helper::{
    BlendModeExt, ContentExt, GroupExt, NameExt, RectExt, TransformExt,
};
use crate::util::resources::ResourceContainer;
use crate::Result;

/// Render a group into a content stream.
pub fn render(
    group: &usvg::Group,
    chunk: &mut Chunk,
    content: &mut Content,
    ctx: &mut Context,
    accumulated_transform: Transform,
    initial_opacity: Option<Opacity>,
    rc: &mut ResourceContainer,
) -> Result<()> {
    #[cfg(feature = "filters")]
    if !group.filters().is_empty() {
        return filter::render(group, chunk, content, ctx, rc);
    }

    #[cfg(not(feature = "filters"))]
    if !group.filters().is_empty() {
        log::warn!(
            "Failed convert filter because the filters feature was disabled. Skipping."
        )
    }

    let initial_opacity = initial_opacity.unwrap_or(Opacity::ONE);

    if group.is_isolated() || initial_opacity.get() != 1.0 {
        content.save_state_checked()?;
        let gs_ref = ctx.alloc_ref();
        let mut gs = chunk.ext_graphics(gs_ref);
        gs.non_stroking_alpha(group.opacity().mul(initial_opacity).get())
            .stroking_alpha(group.opacity().mul(initial_opacity).get())
            .blend_mode(group.blend_mode().to_pdf_blend_mode());

        gs.finish();
        content.set_parameters(rc.add_graphics_state(gs_ref).to_pdf_name());

        // We need to render the mask here instead of in `create_to_stream` so that
        // if we have a child as an image with a SMask, it won't override
        // the soft mask of the group. Unfortunately, this means we need this ugly
        // hack of setting and then reversing the transform.
        if let Some(mask) = group.mask() {
            content.transform(group.transform().to_pdf_transform());
            mask::render(group, mask, chunk, content, ctx, rc)?;
            content.transform(
                group.transform().invert().ok_or(UnknownError)?.to_pdf_transform(),
            );
        }

        // We don't need to pass the accumulated transform here because if a pattern appears in a
        // XObject, it will be mapped to the coordinate space of where the XObject was invoked, meaning
        // that it will also be affected by the transforms in the content stream. If we passed on the
        // accumulated transform, they would be applied twice.
        let x_ref = create_x_object(group, chunk, ctx, Transform::default())?;
        let x_name = rc.add_x_object(x_ref);
        content.x_object(x_name.to_pdf_name());
        content.restore_state();
    } else {
        create_to_stream(group, chunk, content, ctx, accumulated_transform, rc)?;
    }

    Ok(())
}

/// Turn a group into an XObject.
fn create_x_object(
    group: &usvg::Group,
    chunk: &mut Chunk,
    ctx: &mut Context,
    accumulated_transform: Transform,
) -> Result<Ref> {
    let x_ref = ctx.alloc_ref();
    let mut rc = ResourceContainer::new();

    let pdf_bbox = group
        .layer_bounding_box()
        .transform(group.transform())
        .unwrap()
        .to_pdf_rect();

    let mut content = Content::new();

    create_to_stream(group, chunk, &mut content, ctx, accumulated_transform, &mut rc)?;

    let content_stream = ctx.finish_content(content);

    let mut x_object = chunk.form_xobject(x_ref, &content_stream);
    rc.finish(&mut x_object.resources());

    if ctx.options.compress {
        x_object.filter(Filter::FlateDecode);
    }

    x_object
        .group()
        .transparency()
        .isolated(group.is_isolated())
        .knockout(false)
        .color_space()
        .icc_based(ctx.srgb_ref());

    x_object.bbox(pdf_bbox);
    x_object.finish();

    Ok(x_ref)
}

/// Write a group into a content stream. Opacities will be ignored. If opacities are needed,
/// you should use the `create` method instead.
fn create_to_stream(
    group: &usvg::Group,
    chunk: &mut Chunk,
    content: &mut Content,
    ctx: &mut Context,
    accumulated_transform: Transform,
    rc: &mut ResourceContainer,
) -> Result<()> {
    content.save_state_checked()?;
    content.transform(group.transform().to_pdf_transform());
    let accumulated_transform = accumulated_transform.pre_concat(group.transform());

    if let Some(clip_path) = &group.clip_path() {
        clip_path::render(group, clip_path, chunk, content, ctx, rc)?;
    }

    for child in group.children() {
        child.render(chunk, content, ctx, accumulated_transform, rc)?;
    }

    content.restore_state();

    Ok(())
}

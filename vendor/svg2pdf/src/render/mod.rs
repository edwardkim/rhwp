use pdf_writer::{Chunk, Content, Filter, Finish, Ref};
use usvg::{Node, Transform, Tree};

use crate::util::context::Context;
use crate::util::helper::{ContentExt, RectExt, TransformExt};
use crate::util::resources::ResourceContainer;
use crate::Result;

pub mod clip_path;
#[cfg(feature = "filters")]
pub mod filter;
pub mod gradient;
pub mod group;
#[cfg(feature = "image")]
pub mod image;
pub mod mask;
pub mod path;
pub mod pattern;
#[cfg(feature = "text")]
pub mod text;

/// Write a tree into a stream. Assumes that the stream belongs to transparency group and the object
/// that contains it has the correct bounding box set.
pub fn tree_to_stream(
    tree: &Tree,
    chunk: &mut Chunk,
    content: &mut Content,
    ctx: &mut Context,
    rc: &mut ResourceContainer,
) -> Result<()> {
    content.save_state_checked()?;

    // From PDF coordinate system to SVG coordinate system
    let initial_transform =
        Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, tree.size().height());

    content.transform(initial_transform.to_pdf_transform());

    group::render(tree.root(), chunk, content, ctx, initial_transform, None, rc)?;
    content.restore_state();

    Ok(())
}

/// Convert a tree into a XObject of size 1x1, similar to an image.
pub fn tree_to_xobject(tree: &Tree, chunk: &mut Chunk, ctx: &mut Context) -> Result<Ref> {
    let bbox = tree.size().to_non_zero_rect(0.0, 0.0);
    let x_ref = ctx.alloc_ref();

    let mut rc = ResourceContainer::new();

    let mut content = Content::new();
    tree_to_stream(tree, chunk, &mut content, ctx, &mut rc)?;
    let stream = ctx.finish_content(content);

    let mut x_object = chunk.form_xobject(x_ref, &stream);
    x_object.bbox(bbox.to_pdf_rect());
    x_object.matrix([1.0 / bbox.width(), 0.0, 0.0, 1.0 / bbox.height(), 0.0, 0.0]);

    if ctx.options.compress {
        x_object.filter(Filter::FlateDecode);
    }

    let mut resources = x_object.resources();
    rc.finish(&mut resources);

    resources.finish();
    x_object.finish();

    Ok(x_ref)
}

trait Render {
    fn render(
        &self,
        chunk: &mut Chunk,
        content: &mut Content,
        ctx: &mut Context,
        accumulated_transform: Transform,
        rc: &mut ResourceContainer,
    ) -> Result<()>;
}

impl Render for Node {
    fn render(
        &self,
        chunk: &mut Chunk,
        content: &mut Content,
        ctx: &mut Context,
        accumulated_transform: Transform,
        rc: &mut ResourceContainer,
    ) -> Result<()> {
        match self {
            Node::Path(ref path) => {
                path::render(path, chunk, content, ctx, rc, accumulated_transform)
            }
            Node::Group(ref group) => {
                group::render(group, chunk, content, ctx, accumulated_transform, None, rc)
            }
            #[cfg(feature = "image")]
            Node::Image(ref image) => image::render(
                image.is_visible(),
                image.kind(),
                None,
                chunk,
                content,
                ctx,
                rc,
            ),
            #[cfg(not(feature = "image"))]
            Node::Image(_) => {
                log::warn!("Failed convert image because the image feature was disabled. Skipping.");
                Ok(())
            }
            #[cfg(feature = "text")]
            Node::Text(ref text) => {
                if ctx.options.embed_text {
                    text::render(text, chunk, content, ctx, rc, accumulated_transform)
                } else {
                    group::render(
                        text.flattened(),
                        chunk,
                        content,
                        ctx,
                        accumulated_transform,
                        None,
                        rc,
                    )
                }
            }
            #[cfg(not(feature = "text"))]
            Node::Text(_) => {
                log::warn!("Failed convert text because the text feature was disabled. Skipping.");
                Ok(())
            }
        }
    }
}

use std::rc::Rc;

use crate::ConversionError::InvalidImage;
use image::{ColorType, DynamicImage, ImageFormat, Luma, Rgb, Rgba};
use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
use pdf_writer::{Chunk, Content, Filter, Finish};
use usvg::{ImageKind, Rect, Size, Transform, Tree};

use crate::render::tree_to_xobject;
use crate::util::context::Context;
use crate::util::helper::{ContentExt, NameExt, TransformExt};
use crate::util::resources::ResourceContainer;
use crate::Result;

/// Render an image into a content stream.
pub fn render(
    is_visible: bool,
    kind: &ImageKind,
    view_box: Option<Rect>,
    chunk: &mut Chunk,
    content: &mut Content,
    ctx: &mut Context,
    rc: &mut ResourceContainer,
) -> Result<()> {
    if !is_visible {
        return Ok(());
    }

    let load_with_format = |content, format| {
        image::load_from_memory_with_format(content, format).map_err(|_| InvalidImage)
    };

    // Will return the name of the image (in the Resources dictionary) and the dimensions of the
    // actual image (i.e. the actual image size, not the size in the PDF, which will always be 1x1
    // because that's how ImageXObjects are scaled by default.
    let (image_name, image_size) = match kind {
        ImageKind::JPEG(content) => {
            // JPEGs don't support alphas, so no extra processing is required.
            let image = load_with_format(content, ImageFormat::Jpeg)?;
            create_raster_image(chunk, ctx, content, Filter::DctDecode, &image, None, rc)
        }
        ImageKind::PNG(content) => {
            let image = load_with_format(content, ImageFormat::Png)?;
            create_transparent_image(chunk, ctx, &image, rc)
        }
        ImageKind::GIF(content) => {
            let image = load_with_format(content, ImageFormat::Gif)?;
            create_transparent_image(chunk, ctx, &image, rc)
        }
        ImageKind::WEBP(content) => {
            let image = load_with_format(content, ImageFormat::WebP)?;
            create_transparent_image(chunk, ctx, &image, rc)
        }
        // SVGs just get rendered recursively.
        ImageKind::SVG(tree) => create_svg_image(tree, chunk, ctx, rc)?,
    };

    let view_box = view_box.unwrap_or(
        Rect::from_xywh(0.0, 0.0, image_size.width(), image_size.height()).unwrap(),
    );

    content.save_state_checked()?;

    // Account for the x/y of the viewbox.
    content.transform(
        Transform::from_translate(view_box.x(), view_box.y()).to_pdf_transform(),
    );

    // Scale the image from 1x1 to the actual dimensions.
    content.transform(
        Transform::from_row(
            view_box.width(),
            0.0,
            0.0,
            -view_box.height(),
            0.0,
            view_box.height(),
        )
        .to_pdf_transform(),
    );
    content.x_object(image_name.to_pdf_name());
    content.restore_state();

    Ok(())
}

fn create_transparent_image(
    chunk: &mut Chunk,
    ctx: &mut Context,
    image: &DynamicImage,
    rc: &mut ResourceContainer,
) -> (Rc<String>, Size) {
    let color = image.color();
    let bits = color.bits_per_pixel();
    let channels = color.channel_count() as u16;

    let encoded_image: Vec<u8> = match (channels, bits / channels > 8) {
        (1 | 2, false) => image.to_luma8().pixels().flat_map(|&Luma(c)| c).collect(),
        (1 | 2, true) => image
            .to_luma16()
            .pixels()
            .flat_map(|&Luma(x)| x)
            .flat_map(|x| x.to_be_bytes())
            .collect(),
        (3 | 4, false) => image.to_rgb8().pixels().flat_map(|&Rgb(c)| c).collect(),
        (3 | 4, true) => image
            .to_rgb16()
            .pixels()
            .flat_map(|&Rgb(c)| c)
            .flat_map(|x| x.to_be_bytes())
            .collect(),
        _ => panic!("unknown number of channels={channels}"),
    };

    let encoded_mask: Option<Vec<u8>> = if color.has_alpha() {
        if bits / channels > 8 {
            let image = image.to_rgba16();

            if image.pixels().any(|&Rgba([.., a])| a != u16::MAX) {
                Some(image.pixels().flat_map(|&Rgba([.., a])| a.to_be_bytes()).collect())
            } else {
                None
            }
        } else {
            let image = image.to_rgba8();

            if image.pixels().any(|&Rgba([.., a])| a != u8::MAX) {
                Some(image.pixels().map(|&Rgba([.., a])| a).collect())
            } else {
                None
            }
        }
    } else {
        None
    };

    let compression_level = CompressionLevel::DefaultLevel as u8;
    let compressed_image = compress_to_vec_zlib(&encoded_image, compression_level);
    let compressed_mask =
        encoded_mask.map(|m| compress_to_vec_zlib(&m, compression_level));

    create_raster_image(
        chunk,
        ctx,
        &compressed_image,
        Filter::FlateDecode,
        image,
        compressed_mask.as_deref(),
        rc,
    )
}

fn create_raster_image(
    chunk: &mut Chunk,
    ctx: &mut Context,
    samples: &[u8],
    filter: Filter,
    dynamic_image: &DynamicImage,
    alpha_mask: Option<&[u8]>,
    rc: &mut ResourceContainer,
) -> (Rc<String>, Size) {
    let color = dynamic_image.color();
    let alpha_mask = alpha_mask.map(|mask_bytes| {
        let soft_mask_id = ctx.alloc_ref();
        let mut s_mask = chunk.image_xobject(soft_mask_id, mask_bytes);
        s_mask.filter(filter);
        s_mask.width(dynamic_image.width() as i32);
        s_mask.height(dynamic_image.height() as i32);
        s_mask.color_space().device_gray();
        s_mask.bits_per_component(calculate_bits_per_component(color));
        soft_mask_id
    });

    let image_size =
        Size::from_wh(dynamic_image.width() as f32, dynamic_image.height() as f32)
            .unwrap();
    let image_ref = ctx.alloc_ref();
    let image_name = rc.add_x_object(image_ref);

    let mut image_x_object = chunk.image_xobject(image_ref, samples);
    image_x_object.filter(filter);
    image_x_object.width(dynamic_image.width() as i32);
    image_x_object.height(dynamic_image.height() as i32);

    let color_space = image_x_object.color_space();
    if color.has_color() {
        color_space.device_rgb();
    } else {
        color_space.device_gray();
    }

    image_x_object.bits_per_component(calculate_bits_per_component(color));
    if let Some(soft_mask_id) = alpha_mask {
        image_x_object.s_mask(soft_mask_id);
    }
    image_x_object.finish();
    (image_name, image_size)
}

fn calculate_bits_per_component(color_type: ColorType) -> i32 {
    (color_type.bits_per_pixel() / color_type.channel_count() as u16) as i32
}

fn create_svg_image(
    tree: &Tree,
    chunk: &mut Chunk,
    ctx: &mut Context,
    rc: &mut ResourceContainer,
) -> Result<(Rc<String>, Size)> {
    let image_ref = tree_to_xobject(tree, chunk, ctx)?;
    let image_name = rc.add_x_object(image_ref);
    Ok((image_name, tree.size()))
}

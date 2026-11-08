//! 출력 어댑터는 실제 CTM과 JPEG codec만 제공하고 공통 결과를 재생한다.

use std::collections::{HashMap, HashSet};

use resvg::{tiny_skia, usvg};

use super::{frame, images, text, xml};
use crate::renderer::print_raster::Raster;

const MAX_PIXELS: u64 = 67_108_864;

pub(crate) fn is_prepared(data: &[u8]) -> bool {
    context(data).is_some()
}

fn context(data: &[u8]) -> Option<(frame::Frame, (f64, f64))> {
    if data.len() > xml::MAX_SVG_BYTES {
        return None;
    }
    let source = std::str::from_utf8(data).ok()?;
    let document = roxmltree::Document::parse(source).ok()?;
    let root = document.root_element();
    let frame = frame::Frame::read(root.attribute(frame::ATTRIBUTE)?)?;
    let viewport = (xml::number(root, "width")?, xml::number(root, "height")?);
    (viewport.0 > 0.0 && viewport.1 > 0.0).then_some((frame, viewport))
}

pub(crate) fn rasterize(
    data: &[u8],
    matrix: [f64; 4],
    mut decode_jpeg: impl FnMut(&[u8]) -> Option<image::RgbaImage>,
) -> Option<Raster> {
    let (frame, viewport) = context(data)?;
    let root_matrix = frame.root_matrix(viewport, matrix)?;
    let source = std::str::from_utf8(data).ok()?;
    let document = roxmltree::Document::parse(source).ok()?;
    let definitions = document
        .descendants()
        .filter(|node| node.has_tag_name("image"))
        .filter_map(|node| Some((node.attribute("id")?, node)))
        .collect::<HashMap<_, _>>();
    let mut edits = Vec::new();
    // 비동기 host는 이 순회에서 모든 자원의 로드를 시작한다. 첫 대기 자원에서
    // 반환하면 사진 수만큼 프레임을 다시 요청해야 하므로 성공한 decode만 모은다.
    let mut decoded_images = HashMap::new();
    let mut requested = HashSet::new();
    let mut pending = false;
    let mut pixels = 0u64;
    for node in document
        .descendants()
        .filter(|node| node.has_attribute(images::ATTRIBUTE))
    {
        let image = if node.has_tag_name("image") {
            node
        } else {
            *definitions.get(xml::href(node)?.strip_prefix('#')?)?
        };
        let (mime, bytes) = xml::image_bytes(xml::href(image)?)?;
        if mime != "image/jpeg" {
            return None;
        }
        let key = *blake3::hash(&bytes).as_bytes();
        if !requested.insert(key) {
            continue;
        }
        let header = crate::renderer::image_header::canvaskit_encoded_image_header(&bytes)
            .filter(|header| header.is_within_decode_limits())?;
        pixels =
            pixels.checked_add(u64::from(header.width).checked_mul(u64::from(header.height))?)?;
        if pixels > MAX_PIXELS {
            return None;
        }
        if let Some(decoded) = decode_jpeg(&bytes) {
            if decoded.dimensions() != (header.width, header.height) {
                return None;
            }
            decoded_images.insert(key, decoded);
        } else {
            pending = true;
        }
    }
    if pending {
        return None;
    }
    for node in document.descendants().filter(|node| node.is_element()) {
        if let Some(value) = node.attribute(text::ATTRIBUTE) {
            let marker = text::Marker::read(value)?;
            let device = frame.text_device_matrix(marker.pdf, marker.window, matrix)?;
            let correction = [
                device[0] / root_matrix[0],
                device[1] / root_matrix[1],
                (device[2] - root_matrix[2]) / root_matrix[0],
                (device[3] - root_matrix[3]) / root_matrix[1],
            ];
            if correction.iter().any(|value| !value.is_finite()) {
                return None;
            }
            edits.extend(xml::attributes(
                source,
                node,
                &[(
                    "transform",
                    format!(
                        "matrix({} 0 0 {} {} {})",
                        correction[0], correction[1], correction[2], correction[3]
                    ),
                )],
            )?);
            if marker.synthetic_bold {
                let unit = 2.0 / (device[0].abs() + device[1].abs());
                let width = (marker.em * 0.02).max(unit);
                for path in node.descendants().filter(|child| {
                    child.has_tag_name("path")
                        && child
                            .attribute("stroke")
                            .is_some_and(|value| value != "none")
                }) {
                    edits.extend(xml::attributes(
                        source,
                        path,
                        &[("stroke-width", width.to_string())],
                    )?);
                }
            }
        }
        if let Some(value) = node.attribute(images::ATTRIBUTE) {
            let marker = images::Marker::read(value)?;
            let image = if node.has_tag_name("image") {
                node
            } else {
                let id = xml::href(node)?.strip_prefix('#')?;
                *definitions.get(id)?
            };
            let (mime, bytes) = xml::image_bytes(xml::href(image)?)?;
            if mime != "image/jpeg" {
                return None;
            }
            let mut decoded = decoded_images.get(blake3::hash(&bytes).as_bytes())?.clone();
            if marker.flip_x {
                image::imageops::flip_horizontal_in_place(&mut decoded);
            }
            if marker.flip_y {
                image::imageops::flip_vertical_in_place(&mut decoded);
            }
            let raster = marker.frame.rasterize(
                decoded.as_raw(),
                decoded.dimensions(),
                frame.raster_matrix(matrix),
            )?;
            let output = image::RgbaImage::from_raw(raster.width, raster.height, raster.rgba)?;
            let mut png = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(output)
                .write_to(&mut png, image::ImageFormat::Png)
                .ok()?;
            let (x, y) = (
                (f64::from(raster.left) + matrix[2] - root_matrix[2]) / root_matrix[0],
                (f64::from(raster.top) + matrix[3] - root_matrix[3]) / root_matrix[1],
            );
            let (width, height) = (
                f64::from(raster.width) / root_matrix[0],
                f64::from(raster.height) / root_matrix[1],
            );
            let uri = xml::uri("image/png", png.get_ref())?;
            edits.push((node.range(), format!("<image x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" preserveAspectRatio=\"none\" image-rendering=\"optimizeSpeed\" href=\"{uri}\"/>")));
        }
    }
    let prepared = xml::apply(source, edits)?;
    let options = usvg::Options {
        resources_dir: None,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(&prepared, &options).ok()?;
    let (left, top) = (root_matrix[2].floor(), root_matrix[3].floor());
    let width = (root_matrix[2] + frame.bbox.width * matrix[0]).ceil() - left;
    let height = (root_matrix[3] + frame.bbox.height * matrix[1]).ceil() - top;
    if !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
        || width > f64::from(u32::MAX)
        || height > f64::from(u32::MAX)
        || (width as u64).checked_mul(height as u64)? > MAX_PIXELS
        || left.abs() > f64::from(i32::MAX)
        || top.abs() > f64::from(i32::MAX)
    {
        return None;
    }
    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_row(
            root_matrix[0] as f32,
            0.0,
            0.0,
            root_matrix[1] as f32,
            (root_matrix[2] - left) as f32,
            (root_matrix[3] - top) as f32,
        ),
        &mut pixmap.as_mut(),
    );
    Some(Raster {
        width: width as u32,
        height: height as u32,
        left: left as i32,
        top: top as i32,
        rgba: pixmap.take_demultiplied(),
    })
}

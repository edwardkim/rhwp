//! WMFC의 바깥 WMF에 보존된 얇은 직교 펜을 인쇄 장치 좌표에서 재생한다.

use super::xml;
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::{
    image_resolver, metafile_geometry,
    print_raster::PrintRasterFrame,
    render_tree::{BoundingBox, ImageNode},
};

pub(in crate::paint) fn place_device_strokes(
    image: &ImageNode,
    frame: BoundingBox,
    page: (f64, f64),
    origin: (f64, f64),
    dpi: f64,
    payload: &mut ResolvedImagePayload,
) {
    if payload.mime != "image/svg+xml"
        || payload.kind != ResolvedImageKind::FormatConverted
        || image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.0
        || image.content_inset.is_some()
        || image.transform.has_transform()
        || !matches!(image.fill_mode, None | Some(ImageFillMode::FitToSize))
    {
        return;
    }
    let Some(source) = image.data.as_deref() else {
        return;
    };
    if image_resolver::detect_image_mime_type(source) != "image/x-wmf"
        || !source.windows(4).any(|window| window == b"WMFC")
    {
        return;
    }
    if let Some(svg) = placed_strokes(image, frame, page, origin, dpi, &payload.data) {
        payload.data = svg.into_bytes();
    }
}

fn placed_strokes(
    image: &ImageNode,
    frame: BoundingBox,
    page: (f64, f64),
    origin: (f64, f64),
    dpi: f64,
    data: &[u8],
) -> Option<String> {
    if data.len() > xml::MAX_SVG_BYTES {
        return None;
    }
    let source = std::str::from_utf8(data).ok()?;
    let (width, height) = metafile_geometry::svg_viewport(data)?;
    if image.crop.is_some_and(|crop| {
        crate::renderer::svg::compute_image_crop_src(
            crop,
            metafile_geometry::wmf_crop_reference(
                image.original_size_hu,
                image.data.as_deref(),
                image.crop,
            ),
            width,
            height,
        ) != (0.0, 0.0, width, height)
    }) {
        return None;
    }
    let [paper_w, paper_h, left, top, device_w, device_h, _, _] =
        PrintRasterFrame::new(page, origin, frame, dpi)?.values();
    let sx = device_w / width;
    let sy = device_h / height;
    let ux = page.0 / (paper_w / 12.0).round();
    let uy = page.1 / (paper_h / 12.0).round();
    let local_x = width / frame.width;
    let local_y = height / frame.height;
    let map = |[x, y]: [f64; 2]| {
        // WMF 점은 원본 장치 픽셀을 가리키므로 그 중심을 600dpi 장치에 옮긴다.
        let page_x = (left + (x - 0.5) * sx).round() * ux;
        let page_y = (top + (y - 0.5) * sy).round() * uy;
        [(page_x - frame.x) * local_x, (page_y - frame.y) * local_y]
    };
    let parsed = roxmltree::Document::parse(source).ok()?;
    let root = parsed.root_element();
    if xml::numbers::<4>(root.attribute("viewBox")?)? != [0.0, 0.0, width, height] {
        return None;
    }
    let mut edits = Vec::new();
    for node in root.children().filter(|node| node.is_element()) {
        if node.attribute("fill") != Some("none")
            || node.attribute("stroke-dasharray") != Some("none")
            || node.attribute("stroke-opacity") != Some("1.00")
            || node.has_attribute("transform")
        {
            continue;
        }
        let Some(pen) = xml::number(node, "stroke-width") else {
            continue;
        };
        let pen = (pen * sx.min(sy)).round();
        // 한 인쇄 브러시 픽셀(100dpi)보다 넓은 펜은 원래의 벡터 윤곽을 쓴다.
        if !(1.0..=6.0).contains(&pen) {
            continue;
        }
        let half = pen * dpi / 1200.0;
        if let Some([a, b]) = line_points(node) {
            let [x1, y1] = map(a);
            let [x2, y2] = map(b);
            let stroke = if a[1] == b[1] { local_y } else { local_x };
            edits.extend(xml::attributes(
                source,
                node,
                &[
                    ("d", format!("M {x1} {y1} L {x2} {y2}")),
                    ("stroke-width", (2.0 * half * stroke).to_string()),
                    ("stroke-linecap", "butt".to_string()),
                ],
            )?);
        } else if let Some([a, b]) = rectangle_corners(node) {
            let [x1, y1] = map(a);
            let [x2, y2] = map(b);
            let dx = half * local_x;
            let dy = half * local_y;
            let ring = |inset: f64| {
                let (l, t, r, b) = (
                    x1 + inset * dx,
                    y1 + inset * dy,
                    x2 - inset * dx,
                    y2 - inset * dy,
                );
                format!("M {l} {t} H {r} V {b} H {l} Z")
            };
            if x2 - x1 > 2.0 * dx && y2 - y1 > 2.0 * dy {
                let color = quick_xml::escape::escape(node.attribute("stroke")?);
                let clip = quick_xml::escape::escape(node.attribute("clip-path").unwrap_or("none"));
                let id = quick_xml::escape::escape(node.attribute("id")?);
                let d = format!("{} {}", ring(-1.0), ring(1.0));
                edits.push((node.range(), format!("<path id=\"{id}\" clip-path=\"{clip}\" fill=\"{color}\" fill-rule=\"evenodd\" d=\"{d}\"/>")));
            }
        }
    }
    if edits.is_empty() {
        return None;
    }
    xml::apply(source, edits)
}

pub(super) fn line_points(node: roxmltree::Node<'_, '_>) -> Option<[[f64; 2]; 2]> {
    if !node.has_tag_name("path") {
        return None;
    }
    let points = node.attribute("d")?.strip_prefix("M ")?;
    let (a, b) = points.split_once(" L ")?;
    let a = xml::numbers::<2>(a)?;
    let b = xml::numbers::<2>(b)?;
    ((a[0] == b[0]) != (a[1] == b[1])).then_some([a, b])
}

pub(super) fn rectangle_corners(node: roxmltree::Node<'_, '_>) -> Option<[[f64; 2]; 2]> {
    if !node.has_tag_name("polygon") {
        return None;
    }
    let [l, t, x2, y2, r, b, x4, y4] =
        xml::numbers::<8>(&node.attribute("points")?.replace(',', " "))?;
    (l < r && t < b && [x2, y2, x4, y4] == [r, t, l, b]).then_some([[l, t], [r, b]])
}

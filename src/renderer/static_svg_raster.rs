//! Bounded static SVG raster preparation shared by Native and WASM.
use crate::renderer::render_tree::BoundingBox;

pub(crate) struct StaticSvgRaster {
    pub png: Vec<u8>,
    pub byte_cost: usize,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn prepare_static_svg_raster(
    fragment: &str,
    view_box: BoundingBox,
    bbox: BoundingBox,
    scale: f64,
    current_color: &str,
) -> Option<StaticSvgRaster> {
    use quick_xml::{
        events::{BytesEnd, Event},
        Reader, Writer, XmlVersion,
    };
    use resvg::{tiny_skia, usvg};
    let normalized = crate::renderer::static_svg::normalize_static_svg(fragment)?;
    let fragment = normalized.as_str();
    if !crate::paint::paint_op::bbox_is_finite_positive(bbox)
        || !crate::paint::paint_op::bbox_is_finite_positive(view_box)
        || !scale.is_finite()
        || scale <= 0.0
        || current_color.len() > 256
        || current_color.parse::<svgtypes::Color>().is_err()
    {
        return None;
    }
    let width = (bbox.width * scale).ceil();
    let height = (bbox.height * scale).ceil();
    if !width.is_finite()
        || !height.is_finite()
        || width < 1.0
        || height < 1.0
        || width * height > 16_777_216.0
    {
        return None;
    }
    // The payload viewBox replaces only the outer container's viewport.
    // Nested SVG elements retain their own viewport, scaling and clipping.
    let mut reader = Reader::from_str(fragment);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    loop {
        let event = reader.read_event().ok()?;
        let empty = matches!(&event, Event::Empty(_));
        if let Event::Start(element) | Event::Empty(element) = &event {
            if element.name().as_ref() == "path" {
                if let Some(attribute) = element.try_get_attribute("d").ok()? {
                    let commands = attribute.normalized_value(XmlVersion::Implicit1_0).ok()?;
                    // usvg may discard a malformed path but retain valid siblings.
                    // A strict alternative must not silently lose part of its ink.
                    for (index, segment) in
                        svgtypes::PathParser::from(commands.as_ref()).enumerate()
                    {
                        if index >= 100_000 || segment.is_err() {
                            return None;
                        }
                    }
                }
            }
        }

        let rewrite_container = match &event {
            Event::Start(element) | Event::Empty(element) => {
                depth == 0 && element.name().as_ref() == "svg"
            }
            Event::End(element) => depth == 1 && element.name().as_ref() == "svg",
            _ => false,
        };
        match &event {
            Event::Start(_) => depth = depth.checked_add(1)?,
            Event::End(_) => depth = depth.checked_sub(1)?,
            _ => {}
        }
        match event {
            Event::Eof => {
                if depth != 0 {
                    return None;
                }
                break;
            }
            Event::Start(mut element) | Event::Empty(mut element) if rewrite_container => {
                element.set_name("g");
                writer
                    .write_event(if empty {
                        Event::Empty(element)
                    } else {
                        Event::Start(element)
                    })
                    .ok()?;
            }
            Event::End(_) if rewrite_container => {
                writer.write_event(Event::End(BytesEnd::new("g"))).ok()?;
            }
            event => {
                writer.write_event(event).ok()?;
            }
        }
    }
    let inner = String::from_utf8(writer.into_inner()).ok()?;
    let current_color = current_color
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;");
    let svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"none\" color=\"{current_color}\">{inner}</svg>", view_box.x,view_box.y,view_box.width,view_box.height);
    let mut options = usvg::Options::default();
    options.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
        resolve_string: Box::new(|_, _| None),
    };
    let tree = usvg::Tree::from_str(&svg, &options).ok()?;
    if tree.root().children().is_empty() {
        return None;
    }
    let mut pixels = tiny_skia::Pixmap::new(width as u32, height as u32)?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixels.as_mut());
    let byte_cost = pixels.data().len();
    let encoded = pixels.encode_png().ok()?;
    Some(StaticSvgRaster {
        png: encoded,
        byte_cost,
        width: width as u32,
        height: height as u32,
    })
}

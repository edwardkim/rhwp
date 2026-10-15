//! Bounded static SVG image admission, matching the browser image resource gate.
//! This is not the (narrower) font SVG path contract or a generic RawSvg replay.

use quick_xml::{events::Event, Reader};

use super::{CanvasKitEncodedImageFormat, CanvasKitEncodedImageHeader};

pub(super) fn parse_header(bytes: &[u8]) -> Option<CanvasKitEncodedImageHeader> {
    if bytes.len() > 4 * 1024 * 1024 {
        return None;
    }
    let source = std::str::from_utf8(bytes)
        .ok()?
        .trim_start_matches('\u{feff}');
    let mut reader = Reader::from_str(source);
    let mut depth = 0usize;
    let mut elements = 0usize;
    let mut dimensions = None;
    let mut closed = false;
    loop {
        let event = reader.read_event().ok()?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                elements += 1;
                if closed || elements > 4096 || depth >= 64 {
                    return None;
                }
                let qualified_name = element.name();
                let name = qualified_name.as_ref().rsplit(':').next()?;
                let lower_name = name.to_ascii_lowercase();
                if matches!(
                    lower_name.as_str(),
                    "script" | "set" | "foreignobject" | "image" | "feimage"
                ) || lower_name.starts_with("animate")
                {
                    return None;
                }
                let root = depth == 0;
                if root && (name != "svg" || dimensions.is_some()) {
                    return None;
                }
                let mut width = None;
                let mut height = None;
                let mut view_box = None;
                for attribute in element.attributes() {
                    let attribute = attribute.ok()?;
                    let key = attribute.key.as_ref();
                    let local_key = key.rsplit(':').next()?.to_ascii_lowercase();
                    // The browser's bounded static-image gate rejects XML
                    // references before decoding; keep preflight equally strict.
                    if attribute.value.contains('&') {
                        return None;
                    }
                    let value = quick_xml::escape::unescape(attribute.value.as_ref()).ok()?;
                    if local_key.starts_with("on")
                        || (local_key == "href" && !value.trim().starts_with('#'))
                        || key == "xml:base"
                        || !static_css_value(&value)
                    {
                        return None;
                    }
                    if root {
                        match key {
                            "width" => width = parse_length(&value)?,
                            "height" => height = parse_length(&value)?,
                            "viewBox" => {
                                let values = value
                                    .split(|c: char| c.is_ascii_whitespace() || c == ',')
                                    .filter(|part| !part.is_empty())
                                    .take(5)
                                    .map(str::parse::<f64>)
                                    .collect::<Result<Vec<_>, _>>()
                                    .ok()?;
                                if values.len() != 4
                                    || values.iter().any(|value| !value.is_finite())
                                    || values[2] <= 0.0
                                    || values[3] <= 0.0
                                {
                                    return None;
                                }
                                view_box = Some((values[2], values[3]));
                            }
                            _ => {}
                        }
                    }
                }
                if root {
                    dimensions = Some(resolve_dimensions(width, height, view_box)?);
                }
                if empty {
                    closed = root;
                } else {
                    depth += 1;
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1)?;
                closed = depth == 0;
            }
            Event::Text(text) => {
                let text = quick_xml::escape::unescape(text.as_ref()).ok()?;
                if (depth == 0 && !text.trim().is_empty()) || !static_css_value(&text) {
                    return None;
                }
            }
            Event::CData(text) => {
                if depth == 0 || !static_css_value(text.as_ref()) {
                    return None;
                }
            }
            Event::DocType(_) | Event::GeneralRef(_) | Event::PI(_) => return None,
            Event::Decl(_) if elements != 0 => return None,
            Event::Eof => break,
            Event::Comment(_) | Event::Decl(_) => {}
        }
    }
    if depth != 0 || !closed {
        return None;
    }
    let (width, height) = dimensions?;
    Some(CanvasKitEncodedImageHeader {
        format: CanvasKitEncodedImageFormat::Svg,
        width,
        height,
    })
}

// Reject CSS escapes/comments rather than attempting a second CSS parser. Only
// fragment-local url() references are admitted; no fetches or stylesheet imports.
fn static_css_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.contains('\\') || lower.contains("/*") || lower.contains('@') || lower.contains('&') {
        return false;
    }
    let mut rest = lower.as_str();
    while let Some(offset) = rest.find("url(") {
        rest = &rest[offset + 4..];
        let Some(end) = rest.find(')') else {
            return false;
        };
        let target = rest[..end].trim().trim_matches(['\'', '"']);
        if !target.starts_with('#') {
            return false;
        }
        rest = &rest[end + 1..];
    }
    true
}

fn parse_length(value: &str) -> Option<Option<f64>> {
    let value = value.trim().to_ascii_lowercase();
    if value == "auto" {
        return Some(None);
    }
    for (suffix, scale) in [
        ("px", 1.0),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
        ("%", 0.0),
        ("", 1.0),
    ] {
        if let Some(number) = value.strip_suffix(suffix) {
            let Ok(number) = number.parse::<f64>() else {
                continue;
            };
            if !number.is_finite() || number <= 0.0 {
                return None;
            }
            return Some((scale != 0.0).then_some(number * scale));
        }
    }
    None
}

fn resolve_dimensions(
    width: Option<f64>,
    height: Option<f64>,
    view_box: Option<(f64, f64)>,
) -> Option<(u32, u32)> {
    let (width, height) = match (width, height, view_box) {
        (Some(width), Some(height), _) => (width, height),
        (Some(width), None, Some((vw, vh))) => (width, width * vh / vw),
        (None, Some(height), Some((vw, vh))) => (height * vw / vh, height),
        (None, None, Some(view_box)) => view_box,
        (width, height, None) => (width.unwrap_or(300.0), height.unwrap_or(150.0)),
    };
    if [width, height]
        .into_iter()
        .any(|value| !value.is_finite() || value <= 0.0 || value > f64::from(u32::MAX))
    {
        return None;
    }
    Some((width.ceil() as u32, height.ceil() as u32))
}

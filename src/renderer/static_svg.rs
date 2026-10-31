//! Static OpenType SVG artwork. No file/network resolver or executable nodes.
//! The normalized fragment is the shared resource for Native and browser replay.
use quick_xml::{events::Event, Reader, Writer, XmlVersion};
use std::collections::{HashMap, HashSet};

pub(crate) fn static_svg_fragment_has_path_layer(fragment: &str) -> bool {
    normalize_static_svg(fragment).is_some()
}

pub(crate) fn normalize_static_svg(fragment: &str) -> Option<String> {
    if fragment.len() > 1024 * 1024 {
        return None;
    }
    let mut reader = Reader::from_str(fragment);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut skip_depth = None;
    let mut definitions = Vec::new();
    let mut ids = HashSet::new();
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut owners: Vec<Option<String>> = Vec::new();
    let mut references = Vec::new();
    let mut visible = false;
    let mut weights: HashMap<String, usize> = HashMap::new();
    let mut image_bytes = 0usize;
    let mut root_references = Vec::new();
    for count in 0..=100_000 {
        if count == 100_000 {
            return None;
        }
        let event = reader.read_event().ok()?;
        match &event {
            Event::Start(element) | Event::Empty(element) => {
                let empty = matches!(event, Event::Empty(_));
                let tag = element.name().into_inner();
                if depth > 64 {
                    return None;
                }
                if skip_depth.is_none()
                    && matches!(
                        tag,
                        "script"
                            | "text"
                            | "font"
                            | "foreignObject"
                            | "switch"
                            | "a"
                            | "view"
                            | "style"
                            | "animate"
                            | "animateTransform"
                            | "set"
                            | "title"
                            | "desc"
                            | "metadata"
                            | "color-profile"
                    )
                {
                    if !empty {
                        skip_depth = Some(depth);
                    } else {
                        continue;
                    }
                }
                if skip_depth.is_none() {
                    if !matches!(
                        tag,
                        "svg"
                            | "g"
                            | "path"
                            | "rect"
                            | "circle"
                            | "ellipse"
                            | "polygon"
                            | "polyline"
                            | "line"
                            | "defs"
                            | "linearGradient"
                            | "radialGradient"
                            | "stop"
                            | "clipPath"
                            | "use"
                            | "image"
                            | "symbol"
                            | "pattern"
                            | "mask"
                    ) {
                        return None;
                    }
                    let mut owner = owners.last().cloned().flatten();
                    let mut attrs = Vec::new();
                    for attribute in element.attributes().with_checks(true) {
                        let attribute = attribute.ok()?;
                        let name = attribute.key.as_ref().to_string();
                        let value = attribute
                            .normalized_value(XmlVersion::Implicit1_0)
                            .ok()?
                            .into_owned();
                        if name == "id" {
                            if value.is_empty() || !ids.insert(value.clone()) {
                                return None;
                            }
                            if let Some(parent) = owner.as_ref() {
                                edges.entry(parent.clone()).or_default().push(value.clone());
                            }
                            owner = Some(value.clone());
                        }
                        attrs.push((name, value));
                    }
                    if let Some(owner) = owner.as_ref() {
                        *weights.entry(owner.clone()).or_default() += 1;
                    }
                    for (name, value) in &attrs {
                        if name == "xml:base"
                            || name.starts_with("on")
                            || value.contains('\\')
                            || value
                                .chars()
                                .any(|c| c.is_control() && !c.is_ascii_whitespace())
                        {
                            return None;
                        }
                        if name == "href" || name == "xlink:href" {
                            if let Some(target) = value.strip_prefix('#') {
                                references.push((owner.clone(), target.to_owned()));
                            } else if tag == "image" {
                                image_bytes = image_bytes
                                    .checked_add(validate_embedded_image(value)?)
                                    .filter(|n| *n <= 16 * 1024 * 1024)?;
                            } else {
                                return None;
                            }
                        }
                        let lower = value.to_ascii_lowercase();
                        if lower.contains("@import")
                            || lower.contains("expression(")
                            || lower.contains("javascript:")
                            || lower.contains("var(")
                            || lower.contains("context-fill")
                            || lower.contains("context-stroke")
                        {
                            return None;
                        }
                        let mut rest = value.as_str();
                        while let Some(start) = rest.find("url(") {
                            let after = &rest[start + 4..];
                            let end = after.find(')')?;
                            let target = after[..end]
                                .trim()
                                .trim_matches(['\'', '"'])
                                .strip_prefix('#')?;
                            references.push((owner.clone(), target.to_owned()));
                            rest = &after[end + 1..];
                        }
                        // CSS escapes and comments cannot disguise an external URL.
                        if lower.contains("url(") != value.contains("url(") || lower.contains("/*")
                        {
                            return None;
                        }
                        if references.len() > 4096 {
                            return None;
                        }
                    }
                    let in_definition = definitions.last().copied().unwrap_or(false)
                        || matches!(tag, "defs" | "clipPath" | "mask" | "pattern" | "symbol");
                    if tag == "path"
                        && !static_svg_tag_has_path_layer(tag, element.attributes_raw())
                    {
                        return None;
                    }
                    if !in_definition
                        && (matches!(tag, "use" | "image")
                            || static_svg_tag_has_path_layer(tag, element.attributes_raw()))
                    {
                        visible = true;
                    }
                    if !empty {
                        definitions.push(in_definition);
                        owners.push(owner);
                    }
                    writer.write_event(event.clone()).ok()?;
                }
                if !empty {
                    depth += 1;
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1)?;
                if skip_depth == Some(depth) {
                    skip_depth = None;
                } else if skip_depth.is_none() {
                    owners.pop()?;
                    definitions.pop()?;
                    writer.write_event(event.clone()).ok()?;
                }
            }
            Event::Text(text) => {
                if skip_depth.is_none() && !text.as_ref().trim().is_empty() {
                    return None;
                }
            }
            Event::Decl(declaration) if depth == 0 => {
                if declaration.version().ok()?.as_ref() != "1.0" {
                    return None;
                }
                if declaration
                    .encoding()
                    .transpose()
                    .ok()?
                    .is_some_and(|value| !value.eq_ignore_ascii_case("utf-8"))
                {
                    return None;
                }
            }
            Event::Comment(_) => {}
            Event::Eof => {
                if depth != 0 || !visible {
                    return None;
                }
                for (owner, target) in references {
                    if !ids.contains(&target) {
                        return None;
                    }
                    if let Some(owner) = owner {
                        edges.entry(owner).or_default().push(target);
                    } else {
                        root_references.push(target);
                    }
                }
                fn cost(
                    node: &str,
                    edges: &HashMap<String, Vec<String>>,
                    weights: &HashMap<String, usize>,
                    active: &mut HashSet<String>,
                    cache: &mut HashMap<String, usize>,
                    depth: usize,
                ) -> Option<usize> {
                    if depth > 64 || !active.insert(node.to_owned()) {
                        return None;
                    }
                    let mut total = weights.get(node).copied().unwrap_or(1);
                    for target in edges.get(node).into_iter().flatten() {
                        let next = if let Some(value) = cache.get(target) {
                            *value
                        } else {
                            cost(target, edges, weights, active, cache, depth + 1)?
                        };
                        total = total.checked_add(next).filter(|n| *n <= 100_000)?;
                    }
                    active.remove(node);
                    cache.insert(node.to_owned(), total);
                    Some(total)
                }
                let mut cache = HashMap::new();
                for node in edges.keys() {
                    if !cache.contains_key(node) {
                        cost(node, &edges, &weights, &mut HashSet::new(), &mut cache, 0)?;
                    }
                }
                let mut expanded = 0usize;
                for node in root_references {
                    let size = cost(&node, &edges, &weights, &mut HashSet::new(), &mut cache, 0)?;
                    expanded = expanded.checked_add(size).filter(|n| *n <= 100_000)?;
                }
                return String::from_utf8(writer.into_inner()).ok();
            }
            _ => return None, // No DTD, entities, XSL or processing instructions.
        }
    }
    None
}

fn validate_embedded_image(value: &str) -> Option<usize> {
    use base64::Engine;
    let (mime, encoded) = value.split_once(",")?;
    if !matches!(mime, "data:image/png;base64" | "data:image/jpeg;base64") {
        return None;
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .ok()?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg)
    ) {
        return None;
    }
    let (width, height) = reader.into_dimensions().ok()?;
    (width > 0
        && height > 0
        && width <= 4096
        && height <= 4096
        && u64::from(width) * u64::from(height) <= 4_194_304)
        .then_some(width as usize * height as usize * 4)
}

fn static_svg_tag_has_path_layer(name: &str, raw_attributes: &str) -> bool {
    const MAX_PATH_SEGMENTS: usize = 100_000;
    match name {
        "path" => static_svg_attribute_value(raw_attributes, "d").is_some_and(|value| {
            let mut has_segment = false;
            for (index, segment) in svgtypes::PathParser::from(value.as_str()).enumerate() {
                if index >= MAX_PATH_SEGMENTS {
                    return false;
                }
                if segment.is_err() {
                    return false;
                }
                has_segment = true;
            }
            has_segment
        }),
        "rect" => {
            static_svg_attribute_number(raw_attributes, "width").is_some_and(|value| value > 0.0)
                && static_svg_attribute_number(raw_attributes, "height")
                    .is_some_and(|value| value > 0.0)
        }
        "circle" => {
            static_svg_attribute_number(raw_attributes, "r").is_some_and(|value| value > 0.0)
        }
        "ellipse" => {
            static_svg_attribute_number(raw_attributes, "rx").is_some_and(|value| value > 0.0)
                && static_svg_attribute_number(raw_attributes, "ry")
                    .is_some_and(|value| value > 0.0)
        }
        "polygon" | "polyline" => static_svg_attribute_value(raw_attributes, "points")
            .is_some_and(|value| static_svg_point_number_count(&value) >= 6),
        "line" => true,
        _ => false,
    }
}
fn static_svg_attribute_number(raw_attributes: &str, name: &str) -> Option<f64> {
    static_svg_attribute_value(raw_attributes, name)?
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn static_svg_point_number_count(value: &str) -> usize {
    value
        .split(|ch: char| ch.is_ascii_whitespace() || ch == ',')
        .filter(|token| !token.is_empty())
        .filter_map(|token| token.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .count()
}

fn static_svg_attribute_value(raw_attributes: &str, name: &str) -> Option<String> {
    let bytes = raw_attributes.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_whitespace() || matches!(bytes[cursor], b'/'))
        {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b':' | b'.' | b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == name_start {
            return None;
        }
        let attr_name = raw_attributes[name_start..cursor].to_ascii_lowercase();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            return None;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }
        let value = if bytes[cursor] == b'\'' || bytes[cursor] == b'"' {
            let quote = bytes[cursor];
            cursor += 1;
            let value_start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote {
                cursor += 1;
            }
            if cursor >= bytes.len() {
                return None;
            }
            let value = raw_attributes[value_start..cursor].to_string();
            cursor += 1;
            value
        } else {
            let value_start = cursor;
            while cursor < bytes.len()
                && !bytes[cursor].is_ascii_whitespace()
                && !matches!(bytes[cursor], b'/' | b'>')
            {
                cursor += 1;
            }
            raw_attributes[value_start..cursor].to_string()
        };
        if attr_name == name {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::static_svg_fragment_has_path_layer;

    #[test]
    fn accepts_static_visible_path_layers() {
        assert!(static_svg_fragment_has_path_layer(
            "<!-- ok --><rect x=\"0\" y=\"0\" width=\"16\" height=\"16\"/>"
        ));
        assert!(static_svg_fragment_has_path_layer(
            "<g><circle cx=\"8\" cy=\"8\" r=\"4\"/></g>"
        ));
    }

    #[test]
    fn rejects_nonvisual_or_unsafe_fragments() {
        assert!(!static_svg_fragment_has_path_layer(
            "<svg viewBox=\"0 0 16 16\"><defs><path d=\"M0 0 L1 1\"/></defs></svg>"
        ));
        assert!(static_svg_fragment_has_path_layer(
            "<script>alert(1)</script><path d=\"M0 0 L1 1\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"M0 0 L1 1\" onclick=\"alert(1)\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"M0 0 L1 1\" fill=\"url(https://example.invalid/p.svg#p)\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"M0 0 L1 1\" fill=\"context-fill\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"M0 0 L1 1\" stroke=\"var(--glyph-stroke)\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"M0 0 L1 1\" fill=\"u\\72l(https://example.invalid/p.svg#p)\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"M0 0 L1 1\" style=\"fill:u&#114;l(https://example.invalid/p.svg)\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<rect x=\"0\" y=\"0\" width=\"0\" height=\"16\"/>"
        ));
        assert!(!static_svg_fragment_has_path_layer(
            "<path d=\"not-a-path\"/>"
        ));
        let oversized = format!(
            "<path d=\"M0 0 L1 1\"/><metadata>{}</metadata>",
            "x".repeat(1024 * 1024)
        );
        assert!(!static_svg_fragment_has_path_layer(&oversized));
    }
}

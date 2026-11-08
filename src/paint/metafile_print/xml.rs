//! 자체 생성한 SVG의 속성과 그림 URI를 한계 안에서 읽고 쓴다.

use std::ops::Range;

use base64::Engine;

pub(super) const MAX_SVG_BYTES: usize = 32 * 1024 * 1024;
pub(super) type Edit = (Range<usize>, String);

pub(super) fn numbers<const N: usize>(value: &str) -> Option<[f64; N]> {
    let values = value
        .split_ascii_whitespace()
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let values: [f64; N] = values.try_into().ok()?;
    values
        .iter()
        .all(|v| v.is_finite() && v.abs() < f64::from(i32::MAX))
        .then_some(values)
}

pub(super) fn number(node: roxmltree::Node<'_, '_>, name: &str) -> Option<f64> {
    Some(numbers::<1>(node.attribute(name)?)?[0])
}

pub(super) fn attributes(
    source: &str,
    node: roxmltree::Node<'_, '_>,
    values: &[(&str, String)],
) -> Option<Vec<Edit>> {
    let mut edits = Vec::new();
    let mut missing = String::new();
    for (name, value) in values {
        let escaped = quick_xml::escape::escape(value);
        if let Some(attribute) = node.attribute_node(*name) {
            edits.push((attribute.range_value(), escaped.into_owned()));
        } else {
            missing.push_str(&format!(" {name}=\"{escaped}\""));
        }
    }
    if !missing.is_empty() {
        let start = node.range().start;
        let close = start + source.get(start..)?.find('>')?;
        let at = close - usize::from(source.get(close - 1..close)? == "/");
        edits.push((at..at, missing));
    }
    Some(edits)
}

pub(super) fn apply(source: &str, mut edits: Vec<Edit>) -> Option<String> {
    if source.len() > MAX_SVG_BYTES {
        return None;
    }
    edits.sort_by(|a, b| b.0.start.cmp(&a.0.start).then(b.0.end.cmp(&a.0.end)));
    let mut previous = source.len();
    let mut output = source.to_owned();
    for (range, replacement) in edits {
        if range.start > range.end
            || range.end > previous
            || !source.is_char_boundary(range.start)
            || !source.is_char_boundary(range.end)
        {
            return None;
        }
        previous = range.start;
        output.replace_range(range, &replacement);
        if output.len() > MAX_SVG_BYTES {
            return None;
        }
    }
    Some(output)
}

pub(super) fn href<'a>(node: roxmltree::Node<'a, '_>) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| {
            attribute.name() == "href"
                && matches!(
                    attribute.namespace(),
                    None | Some("http://www.w3.org/1999/xlink")
                )
        })
        .map(|attribute| attribute.value())
}

pub(super) fn image_bytes(uri: &str) -> Option<(&'static str, Vec<u8>)> {
    if uri.len() > MAX_SVG_BYTES {
        return None;
    }
    let (mime, encoded) = if let Some(value) = uri.strip_prefix("data:image/png;base64,") {
        ("image/png", value)
    } else {
        ("image/jpeg", uri.strip_prefix("data:image/jpeg;base64,")?)
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    (bytes.len() <= MAX_SVG_BYTES).then_some((mime, bytes))
}

pub(super) fn uri(mime: &str, bytes: &[u8]) -> Option<String> {
    if bytes.len() > MAX_SVG_BYTES / 4 * 3 {
        return None;
    }
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

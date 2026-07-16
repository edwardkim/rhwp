pub(crate) fn static_svg_fragment_has_path_layer(fragment: &str) -> bool {
    const MAX_STATIC_SVG_FRAGMENT_BYTES: usize = 1024 * 1024;
    if fragment.len() > MAX_STATIC_SVG_FRAGMENT_BYTES {
        return false;
    }
    let Some(markup) = static_svg_markup_without_comments(fragment) else {
        return false;
    };
    if markup.contains("<?") || markup.contains("]]>") {
        return false;
    }

    let bytes = markup.as_bytes();
    let mut cursor = 0;
    let mut has_path_layer = false;
    let mut ignored_element_depth = 0usize;
    let mut open_element_depth = 0usize;
    while cursor < bytes.len() {
        let Some(relative_start) = markup[cursor..].find('<') else {
            break;
        };
        let tag_start = cursor + relative_start;
        let Some(relative_end) = markup[tag_start..].find('>') else {
            return false;
        };
        let tag_end = tag_start + relative_end;
        let mut tag_cursor = tag_start + 1;
        while tag_cursor < tag_end && bytes[tag_cursor].is_ascii_whitespace() {
            tag_cursor += 1;
        }
        if tag_cursor >= tag_end {
            return false;
        }
        if bytes[tag_cursor] == b'!' || bytes[tag_cursor] == b'?' {
            return false;
        }
        if bytes[tag_cursor] == b'/' {
            if ignored_element_depth > 0 {
                ignored_element_depth -= 1;
            } else if open_element_depth > 0 {
                open_element_depth -= 1;
            } else {
                return false;
            }
            cursor = tag_end + 1;
            continue;
        }

        let name_start = tag_cursor;
        while tag_cursor < tag_end
            && (bytes[tag_cursor].is_ascii_alphanumeric()
                || matches!(bytes[tag_cursor], b':' | b'-' | b'_'))
        {
            tag_cursor += 1;
        }
        if tag_cursor == name_start {
            return false;
        }

        let name = markup[name_start..tag_cursor].to_ascii_lowercase();
        if !static_svg_tag_is_supported(&name) {
            return false;
        }
        let raw_attributes = &markup[tag_cursor..tag_end];
        if !static_svg_attributes_are_static_safe(raw_attributes) {
            return false;
        }
        let is_self_closing = raw_attributes.trim_end().ends_with('/');
        if ignored_element_depth > 0 {
            if !is_self_closing {
                ignored_element_depth += 1;
            }
            cursor = tag_end + 1;
            continue;
        }
        if static_svg_tag_is_ignored_subtree(&name) {
            if !is_self_closing {
                ignored_element_depth = 1;
            }
            cursor = tag_end + 1;
            continue;
        }
        if !is_self_closing {
            open_element_depth += 1;
        }
        if static_svg_tag_has_path_layer(&name, raw_attributes) {
            has_path_layer = true;
        }
        cursor = tag_end + 1;
    }
    has_path_layer && ignored_element_depth == 0 && open_element_depth == 0
}

fn static_svg_markup_without_comments(fragment: &str) -> Option<String> {
    let mut stripped = String::new();
    let mut cursor = 0;
    while cursor < fragment.len() {
        let comment_start = fragment[cursor..].find("<!--").map(|index| cursor + index);
        let stray_comment_end = fragment[cursor..].find("-->").map(|index| cursor + index);
        if stray_comment_end.is_some_and(|end| comment_start.is_none_or(|start| end < start)) {
            return None;
        }
        let Some(start) = comment_start else {
            stripped.push_str(&fragment[cursor..]);
            return Some(stripped);
        };
        stripped.push_str(&fragment[cursor..start]);
        let content_start = start + 4;
        let end = fragment[content_start..].find("-->")? + content_start;
        if fragment[content_start..end].contains("--") {
            return None;
        }
        cursor = end + 3;
    }
    Some(stripped)
}

fn static_svg_tag_is_supported(name: &str) -> bool {
    matches!(
        name,
        "svg"
            | "g"
            | "path"
            | "rect"
            | "circle"
            | "ellipse"
            | "polygon"
            | "polyline"
            | "line"
            | "title"
            | "desc"
            | "metadata"
            | "defs"
    )
}

fn static_svg_tag_is_ignored_subtree(name: &str) -> bool {
    matches!(name, "title" | "desc" | "metadata" | "defs")
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

fn static_svg_attributes_are_static_safe(raw_attributes: &str) -> bool {
    let bytes = raw_attributes.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_whitespace() || matches!(bytes[cursor], b'/'))
        {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return true;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b':' | b'.' | b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == name_start {
            return false;
        }
        let attr_name = raw_attributes[name_start..cursor].to_ascii_lowercase();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            return false;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return false;
        }
        let value = if bytes[cursor] == b'\'' || bytes[cursor] == b'"' {
            let quote = bytes[cursor];
            cursor += 1;
            let value_start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote {
                cursor += 1;
            }
            if cursor >= bytes.len() {
                return false;
            }
            let value = &raw_attributes[value_start..cursor];
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
            &raw_attributes[value_start..cursor]
        };
        if static_svg_attribute_is_unsafe(&attr_name, value) {
            return false;
        }
    }
    true
}

fn static_svg_attribute_is_unsafe(name: &str, value: &str) -> bool {
    if name.starts_with("on")
        || name == "href"
        || name.ends_with(":href")
        || name == "src"
        || name == "style"
        || value.contains('&')
        || value.contains('\\')
        || value
            .chars()
            .any(|character| character.is_ascii_control() && !character.is_ascii_whitespace())
    {
        return true;
    }
    let value = value.trim().to_ascii_lowercase();
    if value.contains("javascript:")
        || value.contains("data:")
        || value.contains("url(")
        || value.contains("var(")
        || value.contains("@import")
        || value.contains("expression(")
    {
        return true;
    }
    if matches!(name, "fill" | "stroke" | "color")
        && matches!(
            value.as_str(),
            "context-fill"
                | "context-stroke"
                | "inherit"
                | "initial"
                | "revert"
                | "revert-layer"
                | "unset"
        )
    {
        return true;
    }
    false
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
        assert!(!static_svg_fragment_has_path_layer(
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

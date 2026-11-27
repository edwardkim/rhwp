//! 얇은 직교 WMF 펜의 원본 장치 좌표와 출력 픽셀 피복률을 분리한다.

use super::{device_stroke, frame::Frame, xml};

pub(super) const ATTRIBUTE: &str = "data-rhwp-print-wmf-pen";

pub(super) fn prepare(data: &[u8], frame: Frame, viewport: (f64, f64)) -> Option<Vec<u8>> {
    let (origin, scale, unit) = frame.a4_pen_frame(viewport)?;
    let source = std::str::from_utf8(data).ok()?;
    let parsed = roxmltree::Document::parse(source).ok()?;
    let root = parsed.root_element();
    let mut edits = Vec::new();
    for node in root.children().filter(|n| n.is_element()) {
        if node.attribute("fill") != Some("none")
            || node.attribute("stroke-dasharray") != Some("none")
            || node.attribute("stroke-opacity") != Some("1.00")
            || node.has_attribute("transform")
        {
            continue;
        }
        let Some(width) = xml::number(node, "stroke-width") else {
            continue;
        };
        let device_width = (width * scale[0].min(scale[1])).round();
        if !(1.0..=6.0).contains(&device_width) {
            continue;
        }
        let (ring, [a, b]) = if let Some(points) = device_stroke::line_points(node) {
            (false, points)
        } else if let Some(points) = device_stroke::rectangle_corners(node) {
            (true, points)
        } else {
            continue;
        };
        let x = |value: f64| (origin[0] + value * scale[0]).round() * unit[0];
        let y = |value: f64| (origin[1] + value * scale[1]).round() * unit[1];
        let [left, top, right, bottom] = [
            x(a[0].min(b[0])),
            y(a[1].min(b[1])),
            x(a[0].max(b[0])),
            y(a[1].max(b[1])),
        ];
        let width =
            frame.layout_dpi() * device_width / crate::renderer::print_picture_pen::DEVICE_DPI;
        let kind = u8::from(ring);
        edits.extend(xml::attributes(
            source,
            node,
            &[(
                ATTRIBUTE,
                format!("{left} {top} {right} {bottom} {width} {kind}"),
            )],
        )?);
    }
    if edits.is_empty() {
        return None;
    }
    Some(xml::apply(source, edits)?.into_bytes())
}

#[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
pub(super) fn sampled_path(
    node: roxmltree::Node<'_, '_>,
    marker: &str,
    page: [f64; 4],
    root: [f64; 4],
) -> Option<String> {
    use std::fmt::Write;

    let [left, top, right, bottom, width, ring] = xml::numbers::<6>(marker)?;
    let (dx, dy) = (width * page[0] / 2.0, width * page[1] / 2.0);
    let (left, right) = (left * page[0] + page[2], right * page[0] + page[2]);
    let (top, bottom) = (top * page[1] + page[3], bottom * page[1] + page[3]);
    let (outer, inner) = if ring == 1.0 && right - left > 2.0 * dx && bottom - top > 2.0 * dy {
        (
            [left - dx, top - dy, right + dx, bottom + dy],
            Some([left + dx, top + dy, right - dx, bottom - dy]),
        )
    } else if ring == 0.0 && left == right && top < bottom {
        ([left - dx, top, right + dx, bottom], None)
    } else if ring == 0.0 && top == bottom && left < right {
        ([left, top - dy, right, bottom + dy], None)
    } else {
        return None;
    };

    let total_coverage = |rect: [f64; 4], x0: f64, y0: f64, x1: f64, y1: f64| {
        (rect[2].min(x1) - rect[0].max(x0)).max(0.0) * (rect[3].min(y1) - rect[1].max(y0)).max(0.0)
    };
    // 바깥·안쪽 경계가 포함된 픽셀에서만 alpha가 다르다. 전체 길이를
    // 매 픽셀로 풀지 않고 같은 피복률을 갖는 연속 구간으로 그린다.
    let stops = |a: f64, b: f64, inside: Option<(f64, f64)>| {
        let mut values = vec![a.floor(), a.ceil(), b.floor(), b.ceil()];
        if let Some((l, r)) = inside {
            values.extend([l.floor(), l.ceil(), r.floor(), r.ceil()]);
        }
        values.sort_by(f64::total_cmp);
        values.dedup();
        values
    };
    let xs = stops(outer[0], outer[2], inner.map(|i| (i[0], i[2])));
    let ys = stops(outer[1], outer[3], inner.map(|i| (i[1], i[3])));
    let fill = quick_xml::escape::escape(node.attribute("stroke")?);
    let clip = quick_xml::escape::escape(node.attribute("clip-path").unwrap_or("none"));
    let id = quick_xml::escape::escape(node.attribute("id")?);
    let mut result = format!("<g id=\"{id}\" clip-path=\"{clip}\" fill=\"{fill}\">");
    for py in ys.windows(2) {
        for px in xs.windows(2) {
            let w = px[1] - px[0];
            let h = py[1] - py[0];
            let area = |rect| total_coverage(rect, px[0], py[0], px[1], py[1]);
            let opacity = ((area(outer) - inner.map_or(0.0, area)) / (w * h)).clamp(0.0, 1.0);
            if opacity > 0.0 {
                let (x, y) = ((px[0] - root[2]) / root[0], (py[0] - root[3]) / root[1]);
                let (w, h) = (w / root[0], h / root[1]);
                write!(result, "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill-opacity=\"{opacity}\"/>").ok()?;
            }
        }
    }
    result.push_str("</g>");
    Some(result)
}

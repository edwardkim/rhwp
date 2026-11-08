//! 한컴 2024 Print의 RGB 그림과 별도 coverage mask를 같은 경로로 재생한다.

use std::collections::HashSet;
use std::fmt::Write;

#[derive(Default)]
struct Bounds(Option<(f64, f64, f64, f64)>);

impl Bounds {
    fn include(&mut self, x: f64, y: f64) -> Option<()> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        self.0 = Some(match self.0 {
            Some((left, top, right, bottom)) => {
                (left.min(x), top.min(y), right.max(x), bottom.max(y))
            }
            None => (x, y, x, y),
        });
        Some(())
    }
}

fn coverage_bounds(node: roxmltree::Node<'_, '_>) -> Option<(f64, f64, f64, f64)> {
    let mut bounds = Bounds::default();
    for segment in svgtypes::SimplifyingPathParser::from(node.attribute("d")?) {
        use svgtypes::SimplePathSegment as S;
        match segment.ok()? {
            S::MoveTo { x, y } | S::LineTo { x, y } => bounds.include(x, y)?,
            S::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                bounds.include(x1, y1)?;
                bounds.include(x2, y2)?;
                bounds.include(x, y)?;
            }
            S::Quadratic { x1, y1, x, y } => {
                bounds.include(x1, y1)?;
                bounds.include(x, y)?;
            }
            S::ClosePath => (),
        }
    }
    let (left, top, right, bottom) = bounds.0?;
    let width: f64 = node.attribute("stroke-width")?.parse().ok()?;
    let miter: f64 = node
        .attribute("stroke-miterlimit")
        .unwrap_or("4")
        .parse()
        .ok()?;
    if !width.is_finite() || width <= 0.0 || !miter.is_finite() {
        return None;
    }
    // Bézier 곡선은 제어점의 볼록 껍질 안에 있다. stroke의 miter와 square
    // cap까지 포함하는 여유를 더해 음수 좌표·비등방 변환에서도 선을 자르지 않는다.
    let margin = width * miter.max(std::f64::consts::SQRT_2) / 2.0;
    let rect = (
        left - margin,
        top - margin,
        right - left + 2.0 * margin,
        bottom - top + 2.0 * margin,
    );
    [rect.0, rect.1, rect.2, rect.3]
        .iter()
        .all(|v| v.is_finite())
        .then_some(rect)
}

/// `OutputDevice`의 명시적인 Print 경로에서만 호출한다. 일반 SVG의 alpha나
/// geometric 선을 변경하지 않고 정규화된 cosmetic 경로의 coverage만 분리한다.
pub(super) fn apply(svg: &[u8]) -> Option<Vec<u8>> {
    let source = std::str::from_utf8(svg).ok()?;
    let document = roxmltree::Document::parse(source).ok()?;
    let root = document.root_element();
    if !matches!(root.tag_name().name(), "svg" | "g") {
        return None;
    }
    let mut ids: HashSet<_> = root
        .descendants()
        .filter_map(|node| node.attribute("id"))
        .map(str::to_string)
        .collect();
    let digest = super::resources::resource_digest_hex(svg);
    let mut edits = Vec::new();
    let mut definitions = String::from("<defs>");
    for node in root.descendants().filter(|node| {
        node.has_tag_name("path")
            && node.attribute("data-rhwp-emf-device-stroke") == Some("1")
            && node
                .attribute("stroke")
                .is_some_and(|stroke| stroke != "none")
    }) {
        // 이 변환기의 정규화 경로는 transform을 바깥 그룹으로 옮긴다.
        // 알 수 없는 후속 변환이나 이미 존재하는 mask를 추측해 덮어쓰지 않는다.
        if node.has_attribute("transform") || node.has_attribute("mask") {
            return None;
        }
        let (x, y, width, height) = coverage_bounds(node)?;
        let mut serial = edits.len();
        let id = loop {
            let candidate = format!("rhwp-print-coverage-{digest}-{serial}");
            if ids.insert(candidate.clone()) {
                break candidate;
            }
            serial += 1;
        };
        write!(definitions, "<mask id=\"{id}\" maskUnits=\"userSpaceOnUse\" maskContentUnits=\"userSpaceOnUse\" x=\"{x:.9}\" y=\"{y:.9}\" width=\"{width:.9}\" height=\"{height:.9}\"><path").ok()?;
        for name in [
            "d",
            "stroke-width",
            "stroke-linecap",
            "stroke-linejoin",
            "stroke-miterlimit",
            "fill-rule",
        ] {
            if let Some(value) = node.attribute(name) {
                write!(
                    definitions,
                    " {name}=\"{}\"",
                    quick_xml::escape::escape(value)
                )
                .ok()?;
            }
        }
        let fill = if node.attribute("fill") == Some("none") {
            "none"
        } else {
            "white"
        };
        // 원래 색·opacity는 원본 path에 한 번만 적용한다. mask는 별도의
        // 흰 coverage이며 brush opacity를 복사해 두 번 곱하지 않는다.
        write!(definitions, " fill=\"{fill}\" stroke=\"white\"/></mask>").ok()?;
        let range = node.range();
        let tag = &source[range.clone()];
        let close = tag.find('>')?;
        let position = range.start + close - usize::from(tag[..close].ends_with('/'));
        edits.push((position, format!(" mask=\"url(#{id})\"")));
    }
    if edits.is_empty() {
        return None;
    }
    definitions.push_str("</defs>");
    let root_start = root.range().start;
    let insert = root_start + source[root_start..].find('>')? + 1;
    edits.push((insert, definitions));
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut result = source.to_string();
    for (position, value) in edits {
        result.insert_str(position, &value);
    }
    Some(result.into_bytes())
}

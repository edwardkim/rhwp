//! 비트맵 패턴 마스크만 원본 격자에서 합성하고 벡터 형제는 보존한다.
//!
//! WMF 변환기의 직접 자식인 마스크 개체를 대상으로 한다. 글자를 포함하거나
//! 상위 변환의 해석이 필요한 개체는 기존 SVG 재생에 맡긴다.

use std::collections::HashSet;

use base64::Engine;
use resvg::{tiny_skia, usvg};

const MAX_PIXELS: u64 = 67_108_864;
const MAX_BYTES: usize = 32 * 1024 * 1024;

fn reference(value: &str) -> Option<&str> {
    value.trim().strip_prefix("url(#")?.strip_suffix(')')
}

pub(super) fn flatten_bitmap_masks(svg: &[u8]) -> Option<Vec<u8>> {
    if svg.len() > MAX_BYTES {
        return None;
    }
    let source = std::str::from_utf8(svg).ok()?;
    let document = roxmltree::Document::parse(source).ok()?;
    let root = document.root_element();
    if !root.has_tag_name("svg")
        || root.has_attribute("transform")
        || root.has_attribute("opacity")
        || root.has_attribute("style")
    {
        return None;
    }
    let patterns = root
        .descendants()
        .filter(|n| n.has_tag_name("pattern") && n.descendants().any(|n| n.has_tag_name("image")))
        .filter_map(|n| n.attribute("id"))
        .collect::<HashSet<_>>();
    let masks = root
        .descendants()
        .filter(|n| {
            n.has_tag_name("mask")
                && n.descendants().any(|d| {
                    d.attributes()
                        .filter_map(|a| reference(a.value()))
                        .any(|id| patterns.contains(id))
                })
        })
        .filter_map(|n| n.attribute("id"))
        .collect::<HashSet<_>>();
    if masks.is_empty() {
        return None;
    }
    let definitions = root
        .children()
        .filter(|n| n.has_tag_name("defs"))
        .map(|n| &source[n.range()])
        .collect::<String>();
    let opening = &source
        [root.range().start..root.range().start + source[root.range().start..].find('>')? + 1];
    let options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let mut edits = Vec::new();
    let mut encoded_bytes = 0usize;
    for node in root.children().filter(|n| {
        n.is_element()
            // 내포 SVG는 자체 viewport에서 clip·마스크를 합성한다. 이를 부모
            // 논리 격자의 PNG로 바꾸면 최종 장치 축소 전에 별도 보간이 추가되어
            // 원래 ROP 무늬의 피복률이 달라진다. 해당 소유자는 SVG 재생에 맡긴다.
            && !n.descendants().any(|child| child.has_tag_name("svg"))
            && n.attribute("mask")
                .and_then(reference)
                .is_some_and(|id| masks.contains(id))
            && !n
                .descendants()
                .any(|n| n.has_tag_name("text") || n.has_tag_name("foreignObject"))
    }) {
        if edits.len() >= 512 {
            return None;
        }
        let fragment = format!("{opening}{definitions}{}</svg>", &source[node.range()]);
        let tree = usvg::Tree::from_str(&fragment, &options).ok()?;
        let bounds = tree.root().abs_layer_bounding_box();
        let (left, top) = (bounds.x().floor(), bounds.y().floor());
        let (width, height) = (
            (bounds.right().ceil() - left) as u32,
            (bounds.bottom().ceil() - top) as u32,
        );
        if width == 0
            || height == 0
            || u64::from(width).checked_mul(u64::from(height))? > MAX_PIXELS
        {
            return None;
        }
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
        resvg::render(
            &tree,
            tiny_skia::Transform::from_translate(-left, -top),
            &mut pixmap.as_mut(),
        );
        let png = pixmap.encode_png().ok()?;
        encoded_bytes = encoded_bytes.checked_add(png.len())?;
        if encoded_bytes > MAX_BYTES {
            return None;
        }
        let mut image = format!("<image data-rhwp-wmf-mask-grid=\"1\" x=\"{left}\" y=\"{top}\" width=\"{width}\" height=\"{height}\" preserveAspectRatio=\"none\" href=\"data:image/png;base64,");
        base64::engine::general_purpose::STANDARD.encode_string(&png, &mut image);
        image.push_str("\"/>");
        edits.push((node.range(), image));
    }
    if edits.is_empty() {
        return None;
    }
    let mut output = source.to_owned();
    for (range, replacement) in edits.into_iter().rev() {
        output.replace_range(range, &replacement);
    }
    (output.len() <= MAX_BYTES).then(|| output.into_bytes())
}

//! 출력 페이지의 링크 사각형과 공통 URI 표현 (#6963).
use super::render_tree::BoundingBox;

/// 페이지 좌상단 기준 CSS px. PDF writer에서만 72/96 배율과 y축 반전을 적용한다.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PdfLink {
    pub uri: String,
    pub rect: BoundingBox,
}

/// 웹/메일 링크만 실행 URI로 내보낸다. 내부 목적지는 별도 GoTo 구현이 필요하다.
/// PDF URI는 ASCII 문자열이므로 비ASCII UTF-8 바이트를 percent-encode한다.
pub fn export_uri(uri: &str) -> Option<String> {
    let (scheme, rest) = uri.split_once(':')?;
    if rest.is_empty()
        || !["http", "https", "mailto"]
            .iter()
            .any(|s| scheme.eq_ignore_ascii_case(s))
        || uri.chars().any(char::is_control)
    {
        return None;
    }
    let mut out = String::new();
    for byte in uri.bytes() {
        if byte.is_ascii() && byte != b' ' {
            out.push(byte as char);
        } else {
            use std::fmt::Write;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    Some(out)
}

pub(crate) fn clipped_rect(rect: BoundingBox, clip: BoundingBox) -> Option<BoundingBox> {
    if [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        clip.x,
        clip.y,
        clip.width,
        clip.height,
    ]
    .iter()
    .any(|n| !n.is_finite())
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return None;
    }
    let x = rect.x.max(clip.x);
    let y = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    (right > x && bottom > y).then_some(BoundingBox {
        x,
        y,
        width: right - x,
        height: bottom - y,
    })
}

/// 외부 호출자가 주는 annotation도 페이지 경계와 실행 scheme을 검사한다.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn page_links(links: &[PdfLink], width: f64, height: f64) -> Vec<PdfLink> {
    let page = BoundingBox {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    links
        .iter()
        .filter_map(|link| {
            Some(PdfLink {
                uri: export_uri(&link.uri)?,
                rect: clipped_rect(link.rect, page)?,
            })
        })
        .collect()
}

/// 브라우저 인쇄용 SVG에도 같은 페이지 좌표를 사용한다. 잉크는 추가하지 않는다.
pub(crate) fn append_svg_links(svg: &mut String, links: &[PdfLink]) {
    use std::fmt::Write;
    fn escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
    if links.is_empty() {
        return;
    }
    let Some(end) = svg.rfind("</svg>") else {
        return;
    };
    let mut overlay = String::from("<g data-rhwp-hyperlinks=\"true\">\n");
    for link in links {
        let Some(uri) = export_uri(&link.uri) else {
            continue;
        };
        let b = link.rect;
        let _ = writeln!(overlay, "<a href=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"transparent\" stroke=\"none\"/></a>", escape(&uri), b.x, b.y, b.width, b.height);
    }
    overlay.push_str("</g>\n");
    svg.insert_str(end, &overlay);
}

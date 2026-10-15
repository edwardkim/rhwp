//! SVG 조각 파서 유틸리티 (Task #275)
//!
//! RenderNodeType::RawSvg 에 담긴 SVG 조각 (shape_layout.rs 가 생성) 을
//! 파싱·디코드하기 위한 문자열 유틸. 네이티브/WASM 양쪽에서 사용 가능.

/// SVG 조각에서 `attr="..."` 값을 추출한다.
///
/// 간단한 속성 추출기 — 따옴표 이스케이프는 지원하지 않으나,
/// rhwp 가 만드는 OLE/EMF/OOXML SVG 조각은 모두 단순 따옴표 속성만 사용한다.
///
/// 단어 경계 보장: 속성명 앞에 공백/탭/개행이 있거나 문자열 선두에 위치할 때만 매칭.
/// (예: `href` 검색 시 `xlink:href` 를 잘못 매칭하지 않도록)
pub(crate) fn find_svg_attr_value<'a>(s: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", attr);
    let mut search_from = 0;
    while let Some(idx) = s[search_from..].find(&needle) {
        let pos = search_from + idx;
        let is_boundary = if pos == 0 {
            false
        } else {
            let prev = s.as_bytes()[pos - 1];
            prev == b' ' || prev == b'\t' || prev == b'\n' || prev == b'\r'
        };
        if !is_boundary {
            search_from = pos + needle.len();
            continue;
        }
        let value_start = pos + needle.len();
        let end = s[value_start..].find('"')?;
        return Some(&s[value_start..value_start + end]);
    }
    None
}

/// `<image ... href="data:..." .../>` 단일 요소 조각에서 data URL 추출.
///
/// 조건:
/// - 조각이 `<image` 로 시작하고 `/>` 로 끝남 (trim 후)
/// - 여는 태그 개수 (`<`) 가 정확히 1 (복합 SVG 차단)
/// - `xlink:href` 또는 `href` 속성이 `data:` 스킴
///
/// `xlink:href` 우선 (OLE native_image 경로는 둘 다 동일 값을 넣으므로 무관).
pub(crate) fn try_parse_single_image_data_url(svg: &str) -> Option<&str> {
    let s = svg.trim();
    if !s.starts_with("<image") || !s.ends_with("/>") {
        return None;
    }
    if s.matches('<').count() != 1 {
        return None;
    }
    let href = find_svg_attr_value(s, "xlink:href").or_else(|| find_svg_attr_value(s, "href"))?;
    if !href.starts_with("data:") {
        return None;
    }
    Some(href)
}

/// SVG 프리픽스 감지 — 선행 공백/XML 선언 이후 `<svg` 로 시작하는지.
///
/// RenderNodeType::RawSvg 래퍼 경로에서 생성된 SVG 문서 바이트의 MIME 감지에 사용.
/// (detect_image_mime_type 확장 — Task #275)
pub(crate) fn is_svg_prefix(data: &[u8]) -> bool {
    // 선행 공백 스킵 (최대 64바이트까지만)
    let mut i = 0;
    while i < data.len().min(64) && matches!(data[i], b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }
    if data.len().saturating_sub(i) < 4 {
        return false;
    }
    // `<svg` 직접 시작
    if data[i..].starts_with(b"<svg") {
        return true;
    }
    // `<?xml ... ?>` 선언 후 `<svg`
    if data[i..].starts_with(b"<?xml") {
        // 첫 256바이트 내에 `<svg` 등장하면 SVG 간주
        let search_end = data.len().min(i + 256);
        return data[i..search_end].windows(4).any(|w| w == b"<svg");
    }
    false
}

/// SVG 조각을 완전한 `<svg>` 루트 문서로 래핑한다.
///
/// RenderNodeType::RawSvg 의 조각 (EMF/OOXML 등) 은 좌표계가 **페이지 절대좌표**
/// 이므로, 외부 `<svg>` 의 viewBox 를 bbox 에 맞추고 width/height 도 bbox 크기로
/// 설정하면, 나중에 canvas `drawImage(img, bbox.x, bbox.y, bbox.w, bbox.h)` 로
/// 그릴 때 정확히 원본 좌표 위치에 렌더된다.
pub(crate) fn wrap_svg_fragment(fragment: &str, x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
         width=\"{w:.3}\" height=\"{h:.3}\" viewBox=\"{x:.3} {y:.3} {w:.3} {h:.3}\">\n{fragment}\n</svg>"
    )
}

/// `data:MIME;base64,BASE64` 형식 data URL 을 디코드하여 (mime, bytes) 반환.
///
/// 비-base64 data URL (text/plain 등 percent-encoded) 은 지원하지 않고 None 반환.
pub(crate) fn decode_base64_data_url(data_url: &str) -> Option<(String, Vec<u8>)> {
    use base64::Engine;
    let rest = data_url.strip_prefix("data:")?;
    let comma = rest.find(',')?;
    let header = &rest[..comma];
    let payload = &rest[comma + 1..];
    let (mime, is_base64) = if let Some(m) = header.strip_suffix(";base64") {
        (m, true)
    } else {
        (header, false)
    };
    if !is_base64 {
        return None;
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .ok()?;
    Some((mime.to_string(), bytes))
}

pub(crate) struct SvgTextFontUsage {
    pub family: String,
    pub text: String,
    pub bold: bool,
}

/// 생성 SVG의 명시적 글꼴 속성과 그룹 상속을 따라 실제 text 글자를 수집한다.
/// CSS 선택자나 외부 스타일시트를 해석하는 함수는 아니다.
pub(crate) fn svg_text_font_usage(svg: &str) -> Vec<SvgTextFontUsage> {
    use quick_xml::{events::Event, Reader, XmlVersion};

    let mut reader = Reader::from_str(svg);
    let mut stack: Vec<(Option<String>, bool, bool)> = Vec::new();
    let mut usages = Vec::new();
    loop {
        let Ok(event) = reader.read_event() else {
            return Vec::new();
        };
        let text = match event {
            Event::Start(element) => {
                if stack.len() >= 128 {
                    return Vec::new();
                }
                let mut state = stack.last().cloned().unwrap_or_default();
                for attribute in element.attributes() {
                    let Ok(attribute) = attribute else {
                        return Vec::new();
                    };
                    let Ok(value) = attribute.normalized_value(XmlVersion::Implicit1_0) else {
                        return Vec::new();
                    };
                    match attribute.key.as_ref() {
                        "font-family" if value != "inherit" => state.0 = Some(value.into_owned()),
                        "font-weight" if value != "inherit" => {
                            state.1 = value == "bold"
                                || value.parse::<u16>().is_ok_and(|weight| weight >= 600);
                        }
                        _ => {}
                    }
                }
                state.2 |= element.local_name().as_ref() == "text";
                stack.push(state);
                continue;
            }
            Event::End(_) => {
                if stack.pop().is_none() {
                    return Vec::new();
                }
                continue;
            }
            Event::Text(text) => text.into_inner().into_owned(),
            Event::CData(text) => text.into_inner().into_owned(),
            Event::GeneralRef(reference) => {
                let encoded = format!("&{};", reference.as_ref());
                let Ok(decoded) = quick_xml::escape::unescape(&encoded) else {
                    return Vec::new();
                };
                decoded.into_owned()
            }
            Event::Eof => return if stack.is_empty() { usages } else { Vec::new() },
            _ => continue,
        };
        if let Some((Some(family), bold, true)) = stack.last() {
            for face in family
                .split(',')
                .map(str::trim)
                .filter(|face| !face.is_empty())
            {
                usages.push(SvgTextFontUsage {
                    family: face.trim_matches(['\'', '"']).to_owned(),
                    text: text.clone(),
                    bold: *bold,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_attr_basic() {
        let s = r#"<image x="1.0" y="2.0" width="3.0" href="data:foo" />"#;
        assert_eq!(find_svg_attr_value(s, "x"), Some("1.0"));
        assert_eq!(find_svg_attr_value(s, "y"), Some("2.0"));
        assert_eq!(find_svg_attr_value(s, "width"), Some("3.0"));
        assert_eq!(find_svg_attr_value(s, "href"), Some("data:foo"));
    }

    #[test]
    fn find_attr_word_boundary() {
        // `href` 검색이 `xlink:href` 를 잘못 매칭하지 않아야 함
        let s = r#"<image xlink:href="data:A" href="data:B"/>"#;
        assert_eq!(find_svg_attr_value(s, "href"), Some("data:B"));
        assert_eq!(find_svg_attr_value(s, "xlink:href"), Some("data:A"));
    }

    #[test]
    fn find_attr_missing() {
        let s = r#"<image x="1"/>"#;
        assert_eq!(find_svg_attr_value(s, "nope"), None);
    }

    #[test]
    fn parse_single_image_xlink_href() {
        // shape_layout.rs:1059-1062 가 만드는 실제 형식
        let frag = r#"<image x="10.50" y="20.75" width="100.00" height="50.00" preserveAspectRatio="xMidYMid meet" xlink:href="data:image/png;base64,AAAA" href="data:image/png;base64,AAAA"/>"#;
        assert_eq!(
            try_parse_single_image_data_url(frag),
            Some("data:image/png;base64,AAAA")
        );
    }

    #[test]
    fn parse_single_image_href_only() {
        let frag =
            r#"<image x="0" y="0" width="10" height="10" href="data:image/jpeg;base64,ZZZ"/>"#;
        assert_eq!(
            try_parse_single_image_data_url(frag),
            Some("data:image/jpeg;base64,ZZZ")
        );
    }

    #[test]
    fn parse_single_image_leading_whitespace() {
        let frag = "\n  <image href=\"data:x\"/>\n  ";
        assert_eq!(try_parse_single_image_data_url(frag), Some("data:x"));
    }

    #[test]
    fn parse_single_image_rejects_group() {
        // EMF/OOXML 복합 SVG 는 A 경로로 빠지지 않아야 함
        let g_emf =
            r#"<g transform="matrix(1,0,0,1,0,0)"><rect x="0" y="0" width="10" height="10"/></g>"#;
        assert_eq!(try_parse_single_image_data_url(g_emf), None);

        let g_chart = r#"<g class="hwp-ooxml-chart"><rect/><text>..</text></g>"#;
        assert_eq!(try_parse_single_image_data_url(g_chart), None);
    }

    #[test]
    fn parse_single_image_rejects_non_data_href() {
        let frag = r#"<image href="http://example.com/a.png"/>"#;
        assert_eq!(try_parse_single_image_data_url(frag), None);
    }

    #[test]
    fn parse_single_image_rejects_missing_href() {
        let frag = r#"<image x="0" y="0" width="10" height="10"/>"#;
        assert_eq!(try_parse_single_image_data_url(frag), None);
    }

    #[test]
    fn decode_data_url_png() {
        // PNG 매직 8바이트
        let url = "data:image/png;base64,iVBORw0KGgo=";
        let (mime, bytes) = decode_base64_data_url(url).expect("decode");
        assert_eq!(mime, "image/png");
        assert_eq!(bytes, vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    }

    #[test]
    fn decode_data_url_rejects_non_base64() {
        let url = "data:text/plain,hello";
        assert_eq!(decode_base64_data_url(url), None);
    }

    #[test]
    fn decode_data_url_rejects_malformed() {
        assert_eq!(decode_base64_data_url("not a data url"), None);
        assert_eq!(decode_base64_data_url("data:image/png;base64"), None); // 콤마 없음
        assert_eq!(
            decode_base64_data_url("data:image/png;base64,!!!invalid!!!"),
            None
        );
    }

    #[test]
    fn wrap_svg_fragment_basic() {
        let frag = r#"<g transform="matrix(1,0,0,1,0,0)"><rect x="10" y="20" width="30" height="40"/></g>"#;
        let wrapped = wrap_svg_fragment(frag, 100.0, 200.0, 300.0, 400.0);
        assert!(wrapped.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(wrapped.contains("xmlns:xlink=\"http://www.w3.org/1999/xlink\""));
        assert!(wrapped.contains("width=\"300.000\""));
        assert!(wrapped.contains("height=\"400.000\""));
        assert!(wrapped.contains("viewBox=\"100.000 200.000 300.000 400.000\""));
        assert!(wrapped.contains(frag), "원본 조각이 포함되어야 함");
        assert!(wrapped.ends_with("</svg>"));
    }

    #[test]
    fn wrap_svg_fragment_preserves_fragment_content() {
        // 조각 내부의 특수문자/속성 이스케이프에 손대지 않음
        let frag = r#"<text x="5" y="5">가 &amp; 나</text>"#;
        let wrapped = wrap_svg_fragment(frag, 0.0, 0.0, 10.0, 10.0);
        assert!(wrapped.contains(r#"<text x="5" y="5">가 &amp; 나</text>"#));

        // 같은 조각의 원문 보존과 실제 글꼴/글자 공급을 함께 확인한다.

        let svg = r#"<g font-family="'굴림체', sans-serif" font-weight="bold">
            <text>연금&#x3000;재정<tspan font-family="다른 글꼴" font-weight="normal">&amp;전망</tspan></text>
            <g font-family="숨긴 글꼴"><title>미표시</title></g>
            <text>수입</text><rect data-label="지출"/>
        </g>"#;
        let usage = svg_text_font_usage(svg);
        let collect = |face: &str, bold| {
            usage
                .iter()
                .filter(|u| u.family == face && u.bold == bold)
                .map(|u| u.text.as_str())
                .collect::<String>()
        };
        assert_eq!(collect("굴림체", true), "연금　재정수입");
        assert_eq!(collect("sans-serif", true), "연금　재정수입");
        assert_eq!(collect("다른 글꼴", false), "&전망");
        assert!(!usage.iter().any(|u| u.family == "숨긴 글꼴"));
        assert!(svg_text_font_usage("<g><text font-family=\"굴림체\">미완료").is_empty());
    }

    #[test]
    fn is_svg_prefix_direct() {
        assert!(is_svg_prefix(b"<svg"));
        assert!(is_svg_prefix(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"));
        assert!(is_svg_prefix(b"  \n  <svg>"));
        assert!(is_svg_prefix(b"\t<svg width=\"10\">"));
    }

    #[test]
    fn is_svg_prefix_xml_decl() {
        assert!(is_svg_prefix(b"<?xml version=\"1.0\"?>\n<svg>"));
        assert!(is_svg_prefix(
            b"<?xml version=\"1.0\" encoding=\"utf-8\"?><!DOCTYPE svg>\n<svg>"
        ));
    }

    #[test]
    fn is_svg_prefix_rejects_png() {
        assert!(!is_svg_prefix(b"\x89PNG\r\n\x1a\n"));
        assert!(!is_svg_prefix(b"\xFF\xD8\xFF")); // JPEG
        assert!(!is_svg_prefix(b"BM")); // BMP
    }

    #[test]
    fn is_svg_prefix_rejects_html_etc() {
        assert!(!is_svg_prefix(b"<html>"));
        assert!(!is_svg_prefix(b"<!DOCTYPE html>"));
        assert!(!is_svg_prefix(b""));
        assert!(!is_svg_prefix(b"<?"));
    }

    #[test]
    fn is_svg_prefix_xml_without_svg() {
        // XML 선언은 있지만 256바이트 내에 <svg 가 없으면 거부
        let mut data = b"<?xml version=\"1.0\"?>".to_vec();
        data.extend_from_slice(&[b'a'; 300]);
        data.extend_from_slice(b"<svg>");
        assert!(!is_svg_prefix(&data), "256바이트 창 밖은 매칭하지 않음");
    }
}

// SVG 기준 studio 이미지 발산 전수검사 (#4057 부류).
//
// SVG 내보내기는 정상인데 studio(DOM <img> 좁은 질의·layer tree JSON)만 브라우저가
// 그릴 수 없는 mime 을 내보내는 문서를 찾는다. 문서당 JSONL 1줄을 stdout 에 쓴다.
//
// 사용: audit_studio_image_parity <file> [<file>...]
use std::collections::BTreeMap;

/// 브라우저 <img>/drawImage 가 native 디코드하는 mime — studio 쪽 합격 집합.
const BROWSER_IMG_OK: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/bmp",
    "image/svg+xml",
    "image/x-icon",
];

/// SVG <image href="data:..."> 가 표준 지원하는 mime — SVG 쪽 합격 집합.
/// (BMP 는 SVG <image> 내부 data URI 미지원 — image_resolver.rs 주석 참조)
const SVG_EMBED_OK: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];

type MimeCounts = BTreeMap<String, u32>;

fn is_mime_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'/' | b'+' | b'-' | b'.' | b'_')
}

/// layer tree·flow ops JSON 의 "mime":"..." 값을 센다.
fn count_json_mimes(json: &str, map: &mut MimeCounts) {
    let needle = "\"mime\":\"";
    let mut idx = 0;
    while let Some(p) = json[idx..].find(needle) {
        let start = idx + p + needle.len();
        let Some(len) = json[start..].find('"') else {
            break;
        };
        *map.entry(json[start..start + len].to_string()).or_insert(0) += 1;
        idx = start + len;
    }
}

/// SVG 문자열의 href="data:<mime>;..." 를 센다 (xlink:href 포함).
fn count_svg_data_uri_mimes(svg: &str, map: &mut MimeCounts) {
    let needle = "href=\"data:";
    let mut idx = 0;
    while let Some(p) = svg[idx..].find(needle) {
        let start = idx + p + needle.len();
        let bytes = svg.as_bytes();
        let mut end = start;
        while end < svg.len() && is_mime_char(bytes[end]) {
            end += 1;
        }
        if end > start {
            *map.entry(svg[start..end].to_string()).or_insert(0) += 1;
        }
        idx = end.max(start + 1);
    }
}

fn bad_subset(map: &MimeCounts, ok: &[&str]) -> MimeCounts {
    map.iter()
        .filter(|(mime, _)| !ok.contains(&mime.as_str()))
        .map(|(mime, count)| (mime.clone(), *count))
        .collect()
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn write_map(buf: &mut String, key: &str, map: &MimeCounts) {
    buf.push_str(&format!("\"{key}\":{{"));
    for (i, (mime, count)) in map.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        buf.push_str(&format!("\"{}\":{}", json_escape(mime), count));
    }
    buf.push('}');
}

struct DocResult {
    pages: u32,
    /// studio DOM <img> 좁은 질의(getPageFlowImageOps)가 내보내는 mime.
    flow: MimeCounts,
    /// studio layer tree JSON(screen, omit bytes)이 내보내는 mime.
    layer: MimeCounts,
    /// SVG 내보내기의 data URI mime.
    svg: MimeCounts,
    /// 질의 실패한 페이지 수 (page, kind) — 진단용.
    page_errors: Vec<String>,
}

fn audit(path: &str) -> Result<DocResult, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).map_err(|e| format!("parse: {e:?}"))?;
    let pages = doc.page_count();
    let mut out = DocResult {
        pages,
        flow: MimeCounts::new(),
        layer: MimeCounts::new(),
        svg: MimeCounts::new(),
        page_errors: Vec::new(),
    };
    for page in 0..pages {
        match doc.get_page_flow_image_ops(page) {
            Ok(json) => count_json_mimes(&json, &mut out.flow),
            Err(e) => out.page_errors.push(format!("p{page} flow: {e:?}")),
        }
        match doc.get_page_layer_tree_with_profile(page, "screen", Some(true)) {
            Ok(json) => count_json_mimes(&json, &mut out.layer),
            Err(e) => out.page_errors.push(format!("p{page} layer: {e:?}")),
        }
        match doc.render_page_svg(page) {
            Ok(svg) => count_svg_data_uri_mimes(&svg, &mut out.svg),
            Err(e) => out.page_errors.push(format!("p{page} svg: {e:?}")),
        }
    }
    Ok(out)
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: audit_studio_image_parity <file>...");
        std::process::exit(2);
    }
    for path in &files {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| audit(path)));
        let mut line = String::new();
        line.push_str(&format!("{{\"file\":\"{}\",", json_escape(path)));
        match outcome {
            Ok(Ok(result)) => {
                let studio_bad_flow = bad_subset(&result.flow, BROWSER_IMG_OK);
                let studio_bad_layer = bad_subset(&result.layer, BROWSER_IMG_OK);
                let svg_bad = bad_subset(&result.svg, SVG_EMBED_OK);
                let flagged = !studio_bad_flow.is_empty()
                    || !studio_bad_layer.is_empty()
                    || !svg_bad.is_empty();
                line.push_str(&format!(
                    "\"ok\":true,\"pages\":{},\"flagged\":{},",
                    result.pages, flagged
                ));
                write_map(&mut line, "flow", &result.flow);
                line.push(',');
                write_map(&mut line, "layer", &result.layer);
                line.push(',');
                write_map(&mut line, "svg", &result.svg);
                line.push(',');
                write_map(&mut line, "studioBadFlow", &studio_bad_flow);
                line.push(',');
                write_map(&mut line, "studioBadLayer", &studio_bad_layer);
                line.push(',');
                write_map(&mut line, "svgBad", &svg_bad);
                if !result.page_errors.is_empty() {
                    line.push_str(&format!(",\"pageErrors\":{}", result.page_errors.len()));
                    line.push_str(&format!(
                        ",\"pageErrorSample\":\"{}\"",
                        json_escape(&result.page_errors[0])
                    ));
                }
            }
            Ok(Err(e)) => {
                line.push_str(&format!("\"ok\":false,\"error\":\"{}\"", json_escape(&e)));
            }
            Err(_) => {
                line.push_str("\"ok\":false,\"error\":\"panic\"");
            }
        }
        line.push('}');
        println!("{line}");
    }
}

//! 진단: HWPX/HWP 본문 페이지에 본문 1줄만 배치되고 거대한 빈 공간이 생기는
//! "near-blank page" 결함을 네이티브로 재현한다.
//!
//! WASM 바인딩 `pageCount`/`getPageTextLayout`/`getPageFootnoteInfo` 가 호출하는
//! 동일 내부 경로(`DocumentCore`)를 직접 호출한다.
//!
//! 사용: `cargo run --release --example diag_blank_pages -- <file.hwpx> [from] [to]`
//!   기본 출력 범위: 0-indexed page 8..=20.

use rhwp::wasm_api::HwpDocument;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "사용: cargo run --release --example diag_blank_pages -- <file.hwpx> [from] [to]"
        );
        std::process::exit(1);
    }
    let file = &args[0];

    // --coltype <sec> <from_para> <to_para>: 문단별 column_type/page_break_before/텍스트 덤프
    // (강제 쪽나누기 속성 검증용 — orphan break 의 원인 분류).
    if args.get(1).map(|s| s == "--coltype").unwrap_or(false) {
        let sec: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
        let from_p: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
        let to_p: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(60);
        let data = fs::read(file).expect("read file");
        let doc = HwpDocument::from_bytes(&data).expect("parse document");
        let dpi = 96.0_f64;
        let styles = rhwp::renderer::style_resolver::resolve_styles(&doc.document().doc_info, dpi);
        let section = &doc.document().sections[sec];
        println!(
            "sec {} paragraphs (col_type / page_break_before / text)",
            sec
        );
        for (pi, p) in section.paragraphs.iter().enumerate() {
            if pi < from_p || pi > to_p {
                continue;
            }
            let pbb = styles
                .para_styles
                .get(p.para_shape_id as usize)
                .map(|s| s.page_break_before)
                .unwrap_or(false);
            let text: String = p.text.chars().take(28).collect();
            println!(
                "pi={:>4} col_type={:?} pbb={} lines={} ctrls={} text={:?}",
                pi,
                p.column_type,
                pbb,
                p.line_segs.len(),
                p.controls.len(),
                text.trim()
            );
        }
        return;
    }

    let from: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let to: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);

    let data = fs::read(file).expect("read file");
    let doc = HwpDocument::from_bytes(&data).expect("parse document");

    let total = doc.page_count();
    println!("file: {}", file);
    println!("TOTAL PAGES: {}", total);
    println!(
        "{:>4} | {:>3} | {:>9} | {:>9} | {:>9} | {:>7} | {:>7} | {:>7} | {:>7}",
        "page",
        "sec",
        "bodyLines",
        "bodyMinY",
        "bodyMaxY",
        "fnLines",
        "fnCount",
        "paraMin",
        "paraMax"
    );

    let end = to.min(total.saturating_sub(1));
    for page in from..=end {
        match doc.diag_page_layout_native(page) {
            Ok(json) => {
                let sec = extract_int(&json, "sectionIdx");
                let body_lines = extract_int(&json, "bodyLines");
                let body_min = extract_f64(&json, "bodyMinY");
                let body_max = extract_f64(&json, "bodyMaxY");
                let fn_lines = extract_int(&json, "footnoteLines");
                let fn_count = extract_int(&json, "footnoteCount");
                let para_min = extract_int(&json, "paraMin");
                let para_max = extract_int(&json, "paraMax");
                println!(
                    "{:>4} | {:>3} | {:>9} | {:>9.1} | {:>9.1} | {:>7} | {:>7} | {:>7} | {:>7}",
                    page,
                    sec,
                    body_lines,
                    body_min,
                    body_max,
                    fn_lines,
                    fn_count,
                    para_min,
                    para_max
                );
            }
            Err(e) => {
                println!("{:>4} | ERROR: {:?}", page, e);
            }
        }
    }
}

/// 미니 JSON 정수 추출: `"key":<int>` 패턴.
fn extract_int(json: &str, key: &str) -> i64 {
    extract_raw(json, key)
        .and_then(|s| s.trim().parse::<i64>().ok())
        .unwrap_or(-1)
}

/// 미니 JSON 실수 추출.
fn extract_f64(json: &str, key: &str) -> f64 {
    extract_raw(json, key)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

fn extract_raw(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| c == ',' || c == '}')
        .unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

//! Issue #595 회귀 sweep — 정정 전/후 hit_test_header_footer 결과 비교
//!
//! 각 fixture / 각 페이지에서 다음 영역의 hit 결과를 측정:
//! - **머리말 영역 중앙** — hit:true Header 보장 (기존 동작 보존)
//! - **꼬리말 영역 중앙** — hit:true Footer 보장 (기존 동작 보존)
//! - **본문 영역 중앙** — hit:false 보장 (정정 전 결함 → 정정 후 정상)
//!
//! 페이지의 머리말/꼬리말/본문 영역 좌표는 `getPageInfo` JSON 의 margin 정보로 계산.
//!
//! 실행:
//!   cargo run --release --example inspect_595_regression > /tmp/595_after.txt
//!   git stash -- src/document_core/queries/cursor_rect.rs
//!   cargo run --release --example inspect_595_regression > /tmp/595_before.txt
//!   git stash pop
//!   diff /tmp/595_before.txt /tmp/595_after.txt

use std::fs;

#[derive(Default, Debug)]
struct PageMargins {
    page_width: f64,
    page_height: f64,
    margin_left: f64,
    margin_right: f64,
    margin_top: f64,
    margin_bottom: f64,
    margin_header: f64,
    margin_footer: f64,
}

fn parse_f64(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\":", key);
    let pos = json.find(&needle)?;
    let after = &json[pos + needle.len()..];
    let end = after.find(|c: char| c == ',' || c == '}')?;
    after[..end].trim().parse::<f64>().ok()
}

fn parse_margins(json: &str) -> Option<PageMargins> {
    Some(PageMargins {
        page_width: parse_f64(json, "width")?,
        page_height: parse_f64(json, "height")?,
        margin_left: parse_f64(json, "marginLeft")?,
        margin_right: parse_f64(json, "marginRight")?,
        margin_top: parse_f64(json, "marginTop")?,
        margin_bottom: parse_f64(json, "marginBottom")?,
        margin_header: parse_f64(json, "marginHeader")?,
        margin_footer: parse_f64(json, "marginFooter")?,
    })
}

#[derive(Default)]
struct Stats {
    pages: u32,
    header_hit_pass: u32,
    header_hit_fail: u32,
    footer_hit_pass: u32,
    footer_hit_fail: u32,
    body_no_hit_pass: u32,
    body_no_hit_fail: u32,
    failures: Vec<String>,
}

fn main() {
    let samples_dir = std::path::Path::new("samples");
    let entries: Vec<_> = match fs::read_dir(samples_dir) {
        Ok(rd) => rd.flatten().collect(),
        Err(e) => { eprintln!("read_dir fail: {:?}", e); return; }
    };
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for ent in entries {
        let p = ent.path();
        if p.is_file() {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "hwp" || ext == "hwpx" {
                paths.push(p);
            }
        }
    }
    paths.sort();

    let mut stats = Stats::default();
    let mut total_fixtures = 0usize;

    for path in &paths {
        let bytes = match fs::read(path) { Ok(b) => b, Err(_) => continue };
        let core = match rhwp::document_core::DocumentCore::from_bytes(&bytes) {
            Ok(c) => c, Err(_) => continue
        };
        total_fixtures += 1;
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_string();

        let pc = core.page_count();
        for page in 0..pc {
            stats.pages += 1;
            let info_json = match core.get_page_info_native(page) { Ok(j) => j, Err(_) => continue };
            let m = match parse_margins(&info_json) { Some(m) => m, None => continue };

            // 머리말 영역 중앙: x=중앙, y=margin_top + margin_header/2
            let header_x = m.page_width / 2.0;
            let header_y = m.margin_top + m.margin_header / 2.0;

            // 꼬리말 영역 중앙: x=중앙, y=page_height - margin_bottom - margin_footer/2
            let footer_x = m.page_width / 2.0;
            let footer_y = m.page_height - m.margin_bottom - m.margin_footer / 2.0;

            // 본문 중앙: 페이지 정중앙 (확실히 본문 영역 안)
            let body_x = m.page_width / 2.0;
            let body_y = m.page_height / 2.0;

            // 머리말 영역 hit 검증 (margin_header > 0 인 페이지만 — 정의된 머리말이 있는 경우)
            if m.margin_header > 0.0 {
                if let Ok(r) = core.hit_test_header_footer_native(page, header_x, header_y) {
                    let hit = r.contains("\"hit\":true") && r.contains("\"isHeader\":true");
                    if hit { stats.header_hit_pass += 1; }
                    else {
                        stats.header_hit_fail += 1;
                        if stats.failures.len() < 30 {
                            stats.failures.push(format!(
                                "  [HEADER MISS] {} page {} ({:.1},{:.1}) margin_header={:.1}: {}",
                                name, page, header_x, header_y, m.margin_header, r
                            ));
                        }
                    }
                }
            }

            // 꼬리말 영역 hit 검증 (margin_footer > 0 인 페이지만)
            if m.margin_footer > 0.0 {
                if let Ok(r) = core.hit_test_header_footer_native(page, footer_x, footer_y) {
                    let hit = r.contains("\"hit\":true") && r.contains("\"isHeader\":false");
                    if hit { stats.footer_hit_pass += 1; }
                    else {
                        stats.footer_hit_fail += 1;
                        if stats.failures.len() < 30 {
                            stats.failures.push(format!(
                                "  [FOOTER MISS] {} page {} ({:.1},{:.1}) margin_footer={:.1}: {}",
                                name, page, footer_x, footer_y, m.margin_footer, r
                            ));
                        }
                    }
                }
            }

            // 본문 영역 hit 안 됨 검증
            if let Ok(r) = core.hit_test_header_footer_native(page, body_x, body_y) {
                let no_hit = r.contains("\"hit\":false");
                if no_hit { stats.body_no_hit_pass += 1; }
                else {
                    stats.body_no_hit_fail += 1;
                    if stats.failures.len() < 30 {
                        stats.failures.push(format!(
                            "  [BODY HIT — FALSE POSITIVE] {} page {} ({:.1},{:.1}): {}",
                            name, page, body_x, body_y, r
                        ));
                    }
                }
            }
        }
    }

    println!("=== Issue #595 회귀 sweep 결과 ===");
    println!("총 fixture: {} / 총 페이지: {}", total_fixtures, stats.pages);
    println!();
    println!("[1] 머리말 영역 중앙 hit:true 보장 (기존 동작 보존)");
    println!("    pass: {} / fail: {}", stats.header_hit_pass, stats.header_hit_fail);
    println!();
    println!("[2] 꼬리말 영역 중앙 hit:true 보장 (기존 동작 보존)");
    println!("    pass: {} / fail: {}", stats.footer_hit_pass, stats.footer_hit_fail);
    println!();
    println!("[3] 본문 중앙 hit:false 보장 (Issue #595 결함 정정)");
    println!("    pass: {} / fail: {}", stats.body_no_hit_pass, stats.body_no_hit_fail);
    println!();

    if !stats.failures.is_empty() {
        println!("=== 실패 케이스 (최대 30건) ===");
        for line in &stats.failures {
            println!("{}", line);
        }
    } else {
        println!("=== 모든 케이스 통과 ===");
    }
}

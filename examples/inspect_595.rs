//! Issue #595 진단 — exam_math.hwp page 별 hit_test_header_footer 영역 검증
//!
//! 본질 가설: page 2 (0-based 1) 의 본문 좌표 (654.5, 209.7) 가
//! `wasm.hitTestHeaderFooter` 에서 머리말 hit 으로 인식됨.
//!
//! 본 진단은 다음을 확인한다:
//! 1. 각 페이지의 Header / Footer 렌더 노드 bbox 직접 dump
//! 2. y 좌표 sweep 으로 머리말로 hit 되는 y 범위 식별
//!
//! 실행: cargo run --release --example inspect_595

use std::fs;

fn main() {
    let path = "samples/exam_math.hwp";
    let data = fs::read(path).unwrap_or_else(|e| panic!("read {} failed: {}", path, e));
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");

    println!("=== exam_math.hwp 의 hitTestHeaderFooter 영역 진단 ===\n");

    // 페이지 0~3 의 Header / Footer hit 영역 sweep
    for page in 0..4u32 {
        println!("--- page {} (1-based={}) ---", page, page + 1);

        // x 는 페이지 중앙 514 고정, y 를 50 단위로 sweep (페이지 높이 1489.1)
        // Header 영역, 본문 영역, Footer 영역 모두 커버
        let x = 514.0;
        let mut header_hits: Vec<f64> = Vec::new();
        let mut footer_hits: Vec<f64> = Vec::new();
        for ystep in 0..30 {
            let y = (ystep as f64) * 50.0;
            match core.hit_test_header_footer_native(page, x, y) {
                Ok(json) => {
                    if json.contains("\"hit\":true") {
                        if json.contains("\"isHeader\":true") {
                            header_hits.push(y);
                        } else {
                            footer_hits.push(y);
                        }
                    }
                }
                Err(_) => {}
            }
        }
        if !header_hits.is_empty() {
            let min = header_hits.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = header_hits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            println!("  Header hit y 범위: {:.1} ~ {:.1}  (50px sweep)", min, max);
            println!("    hit y 좌표: {:?}", header_hits);
        } else {
            println!("  Header hit 없음");
        }
        if !footer_hits.is_empty() {
            let min = footer_hits.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = footer_hits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            println!("  Footer hit y 범위: {:.1} ~ {:.1}  (50px sweep)", min, max);
        }

        // 이슈 명세 좌표 직접 검증
        if page == 1 {
            // Page 2 (0-based 1) — paraIdx=65 ci=0 수식 영역 (589.5+65, 191.7+18) = (654.5, 209.7)
            let r = core.hit_test_header_footer_native(page, 654.5, 209.7);
            println!("  [이슈 명세 좌표] page {} (654.5, 209.7) -> {:?}", page, r);
        }
        if page == 0 {
            // Page 1 (0-based 0) — paraIdx=18 ci=0 수식 영역 (180.8, 826.3)
            let r = core.hit_test_header_footer_native(page, 180.8, 826.3);
            println!("  [이슈 명세 좌표] page {} (180.8, 826.3) -> {:?}", page, r);
        }
        println!();
    }

    // Header / Footer 영역 정밀 sweep — 더 작은 step (10px) 으로 범위 좁히기
    println!("\n=== Header hit y 범위 정밀 sweep (step=5px, x=514) ===");
    for page in 0..4u32 {
        let x = 514.0;
        let mut header_min: Option<f64> = None;
        let mut header_max: Option<f64> = None;
        for ystep in 0..300 {
            let y = (ystep as f64) * 5.0;
            if let Ok(json) = core.hit_test_header_footer_native(page, x, y) {
                if json.contains("\"hit\":true") && json.contains("\"isHeader\":true") {
                    if header_min.is_none() { header_min = Some(y); }
                    header_max = Some(y);
                }
            }
        }
        match (header_min, header_max) {
            (Some(min), Some(max)) => {
                println!("  page {}: Header hit y={:.1} ~ {:.1}  (실제 머리말 영역: 0~147)", page, min, max);
            }
            _ => println!("  page {}: Header hit 없음", page),
        }
    }

    // x 도 sweep — page 1 (이슈 발현) 에 대해 (x, y=209.7) 으로 가로 sweep
    println!("\n=== page 1 (0-based, 이슈 발현) y=209.7 가로 sweep (step=50px) ===");
    let page = 1u32;
    for xstep in 0..21 {
        let x = (xstep as f64) * 50.0;
        if let Ok(json) = core.hit_test_header_footer_native(page, x, 209.7) {
            let mark = if json.contains("\"hit\":true") {
                if json.contains("\"isHeader\":true") { " ← Header HIT" } else { " ← Footer HIT" }
            } else { "" };
            println!("  x={:5.1} y=209.7 → {}{}", x, json, mark);
        }
    }

    // 광범위 sweep — samples/ 전체에서 머리말 본문 침범 페이지 식별
    println!("\n=== 광범위 sweep — samples/ 전체에서 본문 영역까지 머리말 hit 되는 fixture/페이지 ===");
    use std::path::Path;
    let samples_dir = Path::new("samples");
    let mut total_fixtures = 0usize;
    let mut affected_fixtures = 0usize;
    let mut total_pages = 0u32;
    let mut affected_pages = 0u32;
    let mut affected_summaries: Vec<String> = Vec::new();

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

    for path in &paths {
        let bytes = match fs::read(path) { Ok(b) => b, Err(_) => continue };
        let core_r = rhwp::document_core::DocumentCore::from_bytes(&bytes);
        let core_local = match core_r { Ok(c) => c, Err(_) => continue };
        total_fixtures += 1;
        // 각 페이지 하한 영역 (y=200, 800) 가 머리말 hit 인지 확인
        let pc = core_local.page_count();
        let mut fixture_affected_pages: Vec<u32> = Vec::new();
        for page in 0..pc {
            total_pages += 1;
            let r1 = core_local.hit_test_header_footer_native(page, 514.0, 800.0);
            if let Ok(j) = r1 {
                if j.contains("\"hit\":true") && j.contains("\"isHeader\":true") {
                    fixture_affected_pages.push(page);
                    affected_pages += 1;
                }
            }
        }
        if !fixture_affected_pages.is_empty() {
            affected_fixtures += 1;
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            let pages_str = if fixture_affected_pages.len() <= 5 {
                format!("{:?}", fixture_affected_pages)
            } else {
                format!("[{}, ..., {}] ({}건)", fixture_affected_pages[0],
                    fixture_affected_pages[fixture_affected_pages.len()-1],
                    fixture_affected_pages.len())
            };
            affected_summaries.push(format!("  {} ({}/{}p): {}", name, fixture_affected_pages.len(), pc, pages_str));
        }
    }

    println!("\n총 {} fixture / {} 페이지 점검", total_fixtures, total_pages);
    println!("머리말 hit 본문 침범 fixture: {} / {} ({:.1}%)", affected_fixtures, total_fixtures, 100.0 * affected_fixtures as f64 / total_fixtures as f64);
    println!("머리말 hit 본문 침범 페이지: {} / {} ({:.1}%)", affected_pages, total_pages, 100.0 * affected_pages as f64 / total_pages as f64);
    println!("\n발현 fixture 목록:");
    for line in &affected_summaries {
        println!("{}", line);
    }
}

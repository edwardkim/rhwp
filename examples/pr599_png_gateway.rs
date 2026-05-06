//! PR #599 PNG 게이트웨이 검증 도구
//!
//! samples/exam_kor.hwp 의 모든 페이지를 PNG 로 내보내서
//! native Skia raster backend 의 동작을 검증한다.

use std::fs;
use std::path::Path;

fn main() {
    let input = "samples/exam_kor.hwp";
    let output_dir = "output/pr599_png/exam_kor";

    fs::create_dir_all(output_dir).expect("create output dir");

    println!("=== PR #599 PNG 게이트웨이 — samples/exam_kor.hwp ===");
    println!();

    let bytes = fs::read(input).expect("read sample");
    let core = rhwp::document_core::DocumentCore::from_bytes(&bytes)
        .expect("parse document");

    let page_count = core.page_count();
    println!("페이지 수: {}", page_count);
    println!();

    let mut total_bytes = 0usize;
    let mut success = 0;
    let mut failed = 0;

    for page_num in 0..page_count {
        match core.render_page_png_native(page_num as u32) {
            Ok(png_bytes) => {
                let out_path = format!("{}/exam_kor_{:03}.png", output_dir, page_num + 1);
                fs::write(&out_path, &png_bytes).expect("write png");
                println!(
                    "page {:3}: {} bytes → {}",
                    page_num + 1,
                    png_bytes.len(),
                    Path::new(&out_path).display()
                );
                total_bytes += png_bytes.len();
                success += 1;
            }
            Err(e) => {
                println!("page {:3}: ❌ FAIL — {:?}", page_num + 1, e);
                failed += 1;
            }
        }
    }

    println!();
    println!("=== 게이트웨이 결과 ===");
    println!("성공: {} / 실패: {} / 총 {} bytes ({:.1} MB)",
        success, failed, total_bytes, total_bytes as f64 / 1024.0 / 1024.0);

    if failed > 0 {
        std::process::exit(1);
    }
}

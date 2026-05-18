//! Issue #836 진단 — column 의 used_height vs hwp_used_height 항목별 차이 분석.
//!
//! 사용: `cargo run --release --example diag_836 -- <file.hwp> [-p PAGE]`
//!
//! 출력: 각 page column 의 item 누적 측정 vs IR vpos 진행 비교.

use rhwp::model::control::Control;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::pagination::PageItem;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("사용: cargo run --release --example diag_836 -- <file.hwp> [-p PAGE]");
        std::process::exit(1);
    }
    let file = &args[0];
    let target_page: Option<u32> = args
        .windows(2)
        .find(|w| w[0] == "-p" || w[0] == "--page")
        .and_then(|w| w[1].parse().ok());

    let data = fs::read(file).expect("read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&data).expect("parse");
    let dpi = 96.0_f64;

    println!("file: {} ({}페이지)", file, doc.page_count());

    // pagination 구조 접근. wasm_api 의 deref 로 DocumentCore.pagination 접근.
    // 테스트 helper 처럼 dump_page_items 를 다시 활용하지 않고 직접 ColumnContent 분석.
    // 간단화: 페이지 별 last item 의 vpos 누적과 첫 item 의 vpos 차이를 비교.

    // dump_page_items 로 raw 정보 + 추가 분석은 직접 트래버스 (renderer crate API 사용).
    // 본 helper 는 확정된 본질 진단 후 제거 가능 (Stage 5 cleanup).

    let mut global = 0u32;
    for sec_idx in 0..doc.document().sections.len() {
        let section = &doc.document().sections[sec_idx];

        // pagination 결과는 DocumentCore 내부 — wasm_api 의 page_count + dump_page_items 로 간접 접근.
        // 본 helper 는 raw IR 분석에 집중: 각 paragraph 의 line_segs vpos+lh+ls 누적 vs measure_paragraph total.

        let composed = rhwp::renderer::composer::compose_section(section);
        let styles = rhwp::renderer::style_resolver::resolve_styles(&doc.document().doc_info, dpi);
        let measurer = rhwp::renderer::height_measurer::HeightMeasurer::new(dpi);
        let measured = measurer.measure_section(&section.paragraphs, &composed, &styles);

        println!(
            "\n=== section {} ({} paragraphs) ===",
            sec_idx,
            section.paragraphs.len()
        );

        let mut total_meas = 0.0_f64;
        let mut prev_ir_vpos: Option<i32> = None;
        let mut total_ir_progress = 0.0_f64;

        let mut shape_meas = 0.0_f64; // measured for shape-only paragraphs
        let mut shape_ir = 0.0_f64;
        let mut empty_meas = 0.0_f64; // measured for empty paragraphs
        let mut empty_ir = 0.0_f64;
        let mut text_meas = 0.0_f64;
        let mut text_ir = 0.0_f64;
        let mut table_meas = 0.0_f64;
        let mut table_ir = 0.0_f64;

        for (pi, para) in section.paragraphs.iter().enumerate() {
            let mp = match measured.get_measured_paragraph(pi) {
                Some(m) => m,
                None => continue,
            };
            // IR contribution: from this paragraph's first line_seg vpos to next paragraph's first vpos
            let ir_h: f64 = para
                .line_segs
                .iter()
                .map(|s| hwpunit_to_px(s.line_height + s.line_spacing, dpi))
                .sum();

            // Categorize
            let has_table = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            let has_shape = para.controls.iter().any(|c| {
                matches!(
                    c,
                    Control::Shape(_) | Control::Picture(_) | Control::Equation(_)
                )
            });
            let is_empty = para.text.trim().is_empty() && para.controls.is_empty();

            total_meas += mp.total_height;
            total_ir_progress += ir_h;

            if has_table {
                table_meas += mp.total_height;
                table_ir += ir_h;
            } else if is_empty {
                empty_meas += mp.total_height;
                empty_ir += ir_h;
            } else if has_shape {
                shape_meas += mp.total_height;
                shape_ir += ir_h;
            } else {
                text_meas += mp.total_height;
                text_ir += ir_h;
            }

            let diff = mp.total_height - ir_h;
            // 차이 큰 항목만 출력 (>5px diff)
            if diff.abs() > 5.0 || pi < 10 {
                let kind = if has_table {
                    "TABLE"
                } else if has_shape {
                    "SHAPE"
                } else if is_empty {
                    "EMPTY"
                } else {
                    "TEXT"
                };
                let preview: String = para
                    .text
                    .chars()
                    .filter(|c| *c > '\u{1F}')
                    .take(30)
                    .collect();
                println!(
                    "  [{:4}] {} meas={:5.1} IR={:5.1} diff={:+6.1} segs={} ctrls={} '{}'",
                    pi,
                    kind,
                    mp.total_height,
                    ir_h,
                    diff,
                    para.line_segs.len(),
                    para.controls.len(),
                    preview
                );
            }
            let _ = prev_ir_vpos;
        }

        println!("\n--- SECTION {} TOTALS ---", sec_idx);
        println!(
            "  TEXT  : meas={:.1}  IR={:.1}  diff={:+.1}",
            text_meas,
            text_ir,
            text_meas - text_ir
        );
        println!(
            "  EMPTY : meas={:.1}  IR={:.1}  diff={:+.1}",
            empty_meas,
            empty_ir,
            empty_meas - empty_ir
        );
        println!(
            "  SHAPE : meas={:.1}  IR={:.1}  diff={:+.1}",
            shape_meas,
            shape_ir,
            shape_meas - shape_ir
        );
        println!(
            "  TABLE : meas={:.1}  IR={:.1}  diff={:+.1}",
            table_meas,
            table_ir,
            table_meas - table_ir
        );
        println!(
            "  TOTAL : meas={:.1}  IR={:.1}  diff={:+.1}",
            total_meas,
            total_ir_progress,
            total_meas - total_ir_progress
        );

        if let Some(_pf) = target_page {
            // for now process whole section
        }
        global += 1;
        let _ = global;
    }
}

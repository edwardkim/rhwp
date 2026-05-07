use std::fs::File;
use std::io::Read;

use rhwp::model::control::Control;
use rhwp::renderer::composer::{compose_paragraph, recompose_for_cell_width, estimate_composed_line_width};
use rhwp::renderer::style_resolver::resolve_styles;

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or_else(|| "samples/계획서.hwp".to_string());
    let mut buf = Vec::new();
    File::open(&path).unwrap().read_to_end(&mut buf).unwrap();
    let doc = rhwp::parser::parse_document(&buf).unwrap();
    let dpi: f64 = 96.0;
    let styles = resolve_styles(&doc.doc_info, dpi);

    let sec = &doc.sections[0];
    for (pi, para) in sec.paragraphs.iter().enumerate() {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Table(tbl) = ctrl {
                println!("=== 표 pi={} ci={} ({}행 × {}열) ===",
                    pi, ci, tbl.row_count, tbl.col_count);
                println!("table.padding (HU): l={} r={} t={} b={}",
                    tbl.padding.left, tbl.padding.right, tbl.padding.top, tbl.padding.bottom);

                // 관심 셀들: [13] r=3,c=1, [21] r=5,c=1, [52] r=13,c=3
                let target_indices = [13usize, 21usize, 52usize];
                for &cell_idx in &target_indices {
                    if cell_idx >= tbl.cells.len() { continue; }
                    let cell = &tbl.cells[cell_idx];
                    println!("\n  셀[{}] r={},c={} rs={},cs={}",
                        cell_idx, cell.row, cell.col, cell.row_span, cell.col_span);
                    println!("    HU: width={} height={} pad=(l{},r{},t{},b{}) aim={}",
                        cell.width, cell.height,
                        cell.padding.left, cell.padding.right,
                        cell.padding.top, cell.padding.bottom,
                        cell.apply_inner_margin);
                    let cell_w_px = cell.width as f64 * dpi / 7200.0;
                    println!("    width_px={:.2}", cell_w_px);

                    // padding 계산 (height_measurer 와 동일 로직)
                    let prefer_cell_axis = |c: i16, t: i16| -> bool {
                        if cell.apply_inner_margin { c != 0 } else { (c as i32) > (t as i32) }
                    };
                    let pad_left = if prefer_cell_axis(cell.padding.left, tbl.padding.left) {
                        cell.padding.left as f64 * dpi / 7200.0
                    } else {
                        tbl.padding.left as f64 * dpi / 7200.0
                    };
                    let pad_right = if prefer_cell_axis(cell.padding.right, tbl.padding.right) {
                        cell.padding.right as f64 * dpi / 7200.0
                    } else {
                        tbl.padding.right as f64 * dpi / 7200.0
                    };
                    let inner_w = (cell_w_px - pad_left - pad_right).max(0.0);
                    println!("    pad: l={:.2} r={:.2}, inner_width={:.2}px", pad_left, pad_right, inner_w);

                    for (pi2, p) in cell.paragraphs.iter().enumerate() {
                        let preview: String = p.text.chars().take(50).collect();
                        println!("    p[{}] text=\"{}{}\" line_segs={}",
                            pi2, preview,
                            if p.text.chars().count() > 50 { "..." } else { "" },
                            p.line_segs.len());

                        let mut comp = compose_paragraph(p);
                        // 분할 전 측정
                        let pre_lines = comp.lines.len();
                        let pre_widths: Vec<f64> = comp.lines.iter()
                            .map(|line| estimate_composed_line_width(line, &styles))
                            .collect();
                        println!("      compose: lines={} widths={:?}", pre_lines, pre_widths);

                        // recompose 적용
                        recompose_for_cell_width(&mut comp, p, inner_w, &styles);
                        let post_lines = comp.lines.len();
                        let post_widths: Vec<f64> = comp.lines.iter()
                            .map(|line| estimate_composed_line_width(line, &styles))
                            .collect();
                        println!("      after recompose: lines={} widths={:?}", post_lines, post_widths);
                    }
                }
            }
        }
    }
}

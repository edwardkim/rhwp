// Task #705: aift.hwp 의 셀 안 PageHide 컨트롤 측정
//
// 메인테이너 권위 측정 데이터 (PR #638 코멘트):
//   section 0 / paragraph 1 / Table[0] / 셀[167] / paragraph[3]
//   text: "       년        월        일"
//   PageHide(header=true, footer=true, master=true,
//            border=true, fill=true, page_num=true)

use std::fs;
use rhwp::model::control::{Control, PageHide};
use rhwp::model::table::Table;

fn dump_pagehide(loc: &str, ph: &PageHide) {
    println!(
        "  {} PageHide(header={} footer={} master={} border={} fill={} page_num={})",
        loc, ph.hide_header, ph.hide_footer, ph.hide_master_page,
        ph.hide_border, ph.hide_fill, ph.hide_page_num
    );
}

fn scan_table(loc_prefix: &str, table: &Table, counter: &mut usize) {
    for (cell_idx, cell) in table.cells.iter().enumerate() {
        for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
            for (ci, ctrl) in cp.controls.iter().enumerate() {
                match ctrl {
                    Control::PageHide(ph) => {
                        let text_preview: String = cp.text.chars().take(40).collect();
                        let loc = format!(
                            "{}/셀[{}]/p[{}]/ctrl[{}] text=\"{}\"",
                            loc_prefix, cell_idx, cp_idx, ci, text_preview
                        );
                        dump_pagehide(&loc, ph);
                        *counter += 1;
                    }
                    Control::Table(inner) => {
                        let inner_loc = format!("{}/셀[{}]/p[{}]/Table[{}]", loc_prefix, cell_idx, cp_idx, ci);
                        scan_table(&inner_loc, inner, counter);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or_else(|| "samples/aift.hwp".to_string());
    println!("=== inspect_705: {} ===", path);

    let data = fs::read(&path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();

    let mut total_body = 0;
    let mut total_cell = 0;

    for (si, section) in doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::PageHide(ph) => {
                        let loc = format!("[BODY] s{}/p[{}]/ctrl[{}]", si, pi, ci);
                        dump_pagehide(&loc, ph);
                        total_body += 1;
                    }
                    Control::Table(table) => {
                        let prefix = format!("[CELL] s{}/p[{}]/Table[{}]", si, pi, ci);
                        scan_table(&prefix, table, &mut total_cell);
                    }
                    _ => {}
                }
            }
        }
    }

    println!();
    println!("=== 요약 ===");
    println!("  본문 PageHide: {} 건", total_body);
    println!("  셀 안 PageHide: {} 건", total_cell);
    println!("  합계: {} 건", total_body + total_cell);
}

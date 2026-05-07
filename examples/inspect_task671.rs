use std::fs::File;
use std::io::Read;

use rhwp::model::control::Control;

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or_else(|| "samples/계획서.hwp".to_string());
    let mut buf = Vec::new();
    File::open(&path).unwrap().read_to_end(&mut buf).unwrap();
    let doc = rhwp::parser::parse_document(&buf).unwrap();

    println!("== 문서: {} ==", path);
    println!("섹션 수: {}", doc.sections.len());

    let sec = &doc.sections[0];
    println!("\n== 섹션 0: 문단 수 = {} ==", sec.paragraphs.len());

    // 표 찾기
    for (pi, para) in sec.paragraphs.iter().enumerate() {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Table(tbl) = ctrl {
                println!("\n=== 표 발견 pi={} ci={} ({}행 × {}열, 셀 수={}) ===",
                    pi, ci, tbl.row_count, tbl.col_count, tbl.cells.len());

                for (cell_idx, cell) in tbl.cells.iter().enumerate() {
                    if cell.paragraphs.is_empty() {
                        continue;
                    }
                    // 두 줄 이상이 될 수 있는 후보만 출력 (text 길이 > 30)
                    let total_text_len: usize = cell.paragraphs.iter()
                        .map(|p| p.text.chars().count()).sum();
                    let max_lineseg = cell.paragraphs.iter()
                        .map(|p| p.line_segs.len()).max().unwrap_or(0);
                    // 모든 셀 출력 (진단용)
                    let _ = total_text_len;
                    let _ = max_lineseg;
                    println!("\n  셀[{}] r={},c={} rs={},cs={} h={} w={}",
                        cell_idx, cell.row, cell.col,
                        cell.row_span, cell.col_span,
                        cell.height, cell.width);
                    for (pi2, p) in cell.paragraphs.iter().enumerate() {
                        let preview: String = p.text.chars().take(40).collect();
                        println!("    p[{}] cc={} text=\"{}{}\" line_segs={}",
                            pi2, p.char_count, preview,
                            if p.text.chars().count() > 40 { "..." } else { "" },
                            p.line_segs.len());
                        for (li, ls) in p.line_segs.iter().enumerate() {
                            println!("      ls[{}] vpos={} lh={} th={} bl={} ls={} cs={} sw={}",
                                li, ls.vertical_pos, ls.line_height, ls.text_height,
                                ls.baseline_distance, ls.line_spacing,
                                ls.column_start, ls.segment_width);
                        }
                    }
                }
            }
        }
    }
}

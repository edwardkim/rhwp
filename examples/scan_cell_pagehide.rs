// Task #705: 전체 샘플 셀 안 PageHide 분포 조사
//
// PR #640 의 H2 측정 ("본문 PageHide 만") 의 누락 영역 (셀 안) 을 재측정.
// 174+ 샘플 sweep 으로 영향 범위 + 회귀 위험 평가.

use std::fs;
use std::path::{Path, PathBuf};
use rhwp::model::control::{Control, PageHide};
use rhwp::model::table::Table;

#[derive(Default)]
struct Counts {
    body: usize,
    cell: usize,
    cell_full_six: usize, // 6 필드 모두 true
    cell_only_pagenum: usize, // page_num 만 true
}

fn scan_table(table: &Table, c: &mut Counts, locs: &mut Vec<String>, sample: &str) {
    for (cell_idx, cell) in table.cells.iter().enumerate() {
        for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
            for (ci, ctrl) in cp.controls.iter().enumerate() {
                match ctrl {
                    Control::PageHide(ph) => {
                        c.cell += 1;
                        if all_six_true(ph) { c.cell_full_six += 1; }
                        if only_pagenum_true(ph) { c.cell_only_pagenum += 1; }
                        let text_preview: String = cp.text.chars().take(20).collect();
                        locs.push(format!(
                            "  {} 셀[{}]/p[{}]/ctrl[{}] {} text=\"{}\"",
                            sample, cell_idx, cp_idx, ci, fmt_ph(ph), text_preview
                        ));
                    }
                    Control::Table(inner) => {
                        scan_table(inner, c, locs, sample);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn all_six_true(ph: &PageHide) -> bool {
    ph.hide_header && ph.hide_footer && ph.hide_master_page
        && ph.hide_border && ph.hide_fill && ph.hide_page_num
}

fn only_pagenum_true(ph: &PageHide) -> bool {
    !ph.hide_header && !ph.hide_footer && !ph.hide_master_page
        && !ph.hide_border && !ph.hide_fill && ph.hide_page_num
}

fn fmt_ph(ph: &PageHide) -> String {
    let mut bits = String::new();
    if ph.hide_header { bits.push('H'); } else { bits.push('-'); }
    if ph.hide_footer { bits.push('F'); } else { bits.push('-'); }
    if ph.hide_master_page { bits.push('M'); } else { bits.push('-'); }
    if ph.hide_border { bits.push('B'); } else { bits.push('-'); }
    if ph.hide_fill { bits.push('I'); } else { bits.push('-'); }
    if ph.hide_page_num { bits.push('P'); } else { bits.push('-'); }
    bits
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() { return; }
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext == "hwp" || ext == "hwpx" {
                out.push(path);
            }
        }
    }
}

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| "samples".to_string());
    let mut files = Vec::new();
    collect_files(Path::new(&root), &mut files);
    files.sort();

    println!("=== scan_cell_pagehide: {} 파일 sweep ===", files.len());

    let mut total = Counts::default();
    let mut hits: Vec<(String, Counts, Vec<String>)> = Vec::new();
    let mut parse_failed: Vec<String> = Vec::new();

    for path in &files {
        let display = path.strip_prefix(&root).unwrap_or(path).display().to_string();
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(_) => { parse_failed.push(display); continue; }
        };
        let core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
            Ok(c) => c,
            Err(_) => { parse_failed.push(display); continue; }
        };
        let doc = core.document();
        let mut c = Counts::default();
        let mut locs = Vec::new();
        for section in &doc.sections {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    match ctrl {
                        Control::PageHide(_) => { c.body += 1; total.body += 1; }
                        Control::Table(table) => {
                            let before_cell = c.cell;
                            scan_table(table, &mut c, &mut locs, &display);
                            let added = c.cell - before_cell;
                            total.cell += added;
                            total.cell_full_six += locs.iter().filter(|_| false).count(); // 별도 계산
                        }
                        _ => {}
                    }
                }
            }
        }
        if c.cell > 0 || c.body > 0 {
            hits.push((display, c, locs));
        }
    }

    // 합계 재계산 (정확)
    total = Counts::default();
    for (_, c, _) in &hits {
        total.body += c.body;
        total.cell += c.cell;
        total.cell_full_six += c.cell_full_six;
        total.cell_only_pagenum += c.cell_only_pagenum;
    }

    println!();
    println!("=== 셀 안 PageHide 가 있는 샘플 ===");
    for (name, c, locs) in &hits {
        if c.cell == 0 { continue; }
        println!("[{}]", name);
        println!("  본문={} 셀안={} (full6={} pagenum_only={})",
            c.body, c.cell, c.cell_full_six, c.cell_only_pagenum);
        for loc in locs {
            println!("{}", loc);
        }
    }

    println!();
    println!("=== 본문 PageHide 만 있는 샘플 ===");
    let body_only: Vec<&(String, Counts, Vec<String>)> = hits.iter()
        .filter(|(_, c, _)| c.cell == 0 && c.body > 0).collect();
    for (name, c, _) in &body_only {
        println!("  {} 본문={}", name, c.body);
    }

    println!();
    println!("=== 전체 합계 ===");
    println!("  파싱 성공: {}", files.len() - parse_failed.len());
    println!("  파싱 실패: {}", parse_failed.len());
    println!("  본문 PageHide 합계: {}", total.body);
    println!("  셀 안 PageHide 합계: {}", total.cell);
    println!("    - 6필드 모두 true: {}", total.cell_full_six);
    println!("    - page_num 만 true: {}", total.cell_only_pagenum);
    println!("  영향 샘플 (셀 안 PageHide 보유): {}", hits.iter().filter(|(_, c, _)| c.cell > 0).count());
}

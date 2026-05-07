//! PR #601 진단 — aift.hwp 의 분할 표 영역 (pi=579, pi=581, pi=584) 에서 is_header 셀 영역 점검
//!
//! 본 PR fix (table_partial.rs +22/-12) 의 활성 분기:
//!   if is_continuation && repeat_header && start_row > 0
//!     → header_rows = is_header 셀이 있는 모든 행 (r < start_row 영역)
//!
//! 다중 제목행 영역 발현 입증을 위해 각 표의 is_header 셀 영역을 출력한다.
//!
//! 실행: cargo run --release --example inspect_pr601

use rhwp::model::control::Control;
use std::fs;

fn main() {
    let path = "samples/aift.hwp";
    let data = fs::read(path).unwrap_or_else(|e| panic!("read {} failed: {}", path, e));
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();

    println!("=== PR #601 권위 영역 점검 — aift.hwp 분할 표 is_header 영역 ===\n");

    // 본 환경 권위 영역 표들 (47 페이지 + 48 페이지 분할 표 + 본 환경 표 영역 광범위 sweep)
    let pi_targets = [579usize, 581, 584];
    for &pi in &pi_targets {
        if let Some(section) = doc.sections.get(2) {
            if let Some(para) = section.paragraphs.get(pi) {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(table) = ctrl {
                        println!("\n--- pi={} ci={} Table {}행×{}열, repeat_header={} ---",
                            pi, ci, table.row_count, table.col_count, table.repeat_header);

                        let mut header_rows: std::collections::BTreeSet<u16> = Default::default();
                        for cell in &table.cells {
                            if cell.is_header {
                                header_rows.insert(cell.row);
                            }
                        }
                        println!("  is_header rows: {:?} ({}개)", header_rows, header_rows.len());

                        if header_rows.is_empty() {
                            println!("  → 본 표는 is_header 셀 부재 영역. 본 PR fix 분기 미발현.");
                        } else if header_rows.len() == 1 {
                            println!("  → 본 표는 단일 제목행 영역. 기존 (devel) 동작과 본 PR fix 동등 (회귀 0).");
                        } else {
                            println!("  → 본 표는 다중 제목행 영역 ({} 행). 본 PR fix 권위 발현 케이스.",
                                header_rows.len());
                        }

                        // is_header 영역 + 행 0~2 영역 셀 명세
                        for cell in &table.cells {
                            if cell.is_header || cell.row < 3 {
                                println!("    cell r={} c={} rs={} cs={} is_header={} bf={}",
                                    cell.row, cell.col, cell.row_span, cell.col_span,
                                    cell.is_header, cell.border_fill_id);
                            }
                        }
                    }
                }
            }
        }
    }

    // 본 환경 모든 fixture 영역 sweep — 다중 제목행 영역 발현 fixture 찾기
    println!("\n\n=== 본 환경 aift.hwp 전체 표의 is_header 영역 sweep ===");
    let mut multi_header_tables: Vec<(usize, usize, u16, u16, usize)> = Vec::new();
    let mut single_header_count = 0;
    let mut zero_header_count = 0;
    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(table) = ctrl {
                    let mut header_rows: std::collections::BTreeSet<u16> = Default::default();
                    for cell in &table.cells {
                        if cell.is_header {
                            header_rows.insert(cell.row);
                        }
                    }
                    if header_rows.is_empty() {
                        zero_header_count += 1;
                    } else if header_rows.len() == 1 {
                        single_header_count += 1;
                    } else {
                        multi_header_tables.push((sec_idx, pi, table.row_count, table.col_count, header_rows.len()));
                    }
                }
            }
        }
    }
    println!("  is_header 부재 표: {} 개", zero_header_count);
    println!("  단일 제목행 표: {} 개", single_header_count);
    println!("  **다중 제목행 표: {} 개**", multi_header_tables.len());
    if !multi_header_tables.is_empty() {
        println!("  → 본 PR fix 권위 발현 영역 (분할 가능 시):");
        for (sec, pi, rows, cols, hr_count) in &multi_header_tables {
            println!("    s{} pi={} {}행×{}열 (제목행 {}개)", sec, pi, rows, cols, hr_count);
        }
    } else {
        println!("  → 본 환경 aift.hwp 의 모든 표는 단일/부재 제목행 영역. 본 PR fix 의 시각적 발현 영역 부재.");
    }
}

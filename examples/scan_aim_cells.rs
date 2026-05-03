use std::fs;
use rhwp::model::control::Control;

fn scan(path: &str) {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => { println!("{}: read failed", path); return; }
    };
    let core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(_) => { println!("{}: parse failed", path); return; }
    };
    let doc = core.document();

    let mut total_cells = 0u32;
    let mut aim_false = 0u32;
    let mut aim_false_cell_gt_table = 0u32;  // Task #347 hack 발동 케이스
    // 옵션 B: aim=false + cell.padding > table.padding + table.padding == 0 인 케이스
    let mut option_b_affected = 0u32;
    // 옵션 B 보존: aim=false + cell > table + table > 0 인 케이스 (Task #347 KTX 유지)
    let mut option_b_preserved = 0u32;
    let mut aim_false_cell_eq_zero = 0u32;
    let mut shifted_axes_max: f64 = 0.0;     // 최대 시프트량 (HU)
    let mut affected_max_cell: Option<(usize, usize, i16, i16, i16, i16)> = None;

    for (si, section) in doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    let tp = &t.padding;
                    for cell in t.cells.iter() {
                        total_cells += 1;
                        if !cell.apply_inner_margin {
                            aim_false += 1;
                            let cp = &cell.padding;
                            let mut hit = false;
                            for (axis, (c, tt)) in [
                                ("L", (cp.left, tp.left)),
                                ("T", (cp.top, tp.top)),
                                ("R", (cp.right, tp.right)),
                                ("B", (cp.bottom, tp.bottom)),
                            ].iter() {
                                let _ = axis;
                                if (*c as i32) > (*tt as i32) {
                                    hit = true;
                                    let shift = (*c as f64) - (*tt as f64);
                                    if shift.abs() > shifted_axes_max.abs() {
                                        shifted_axes_max = shift;
                                        affected_max_cell = Some((si, pi, cp.left, cp.top, cp.right, cp.bottom));
                                    }
                                }
                            }
                            if hit { aim_false_cell_gt_table += 1; }
                            if cp.left == 0 && cp.top == 0 && cp.right == 0 && cp.bottom == 0 {
                                aim_false_cell_eq_zero += 1;
                            }
                            // 옵션 B: hit 가 발생한 축의 table.padding 이 0 인지
                            if hit {
                                let tab_zero_axes = (tp.left == 0) as i32
                                    + (tp.top == 0) as i32
                                    + (tp.right == 0) as i32
                                    + (tp.bottom == 0) as i32;
                                if tab_zero_axes == 4 {
                                    // 옵션 B: 모든 축 table=0 → hack 제거 (옵션 B 적용)
                                    option_b_affected += 1;
                                } else {
                                    // 옵션 B: 일부 축 table>0 → 보존
                                    option_b_preserved += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{}", path);
    println!("  total_cells={} aim_false={} hack_hit={} aim_false_zero={}",
        total_cells, aim_false, aim_false_cell_gt_table, aim_false_cell_eq_zero);
    println!("  option_B: affected (hack 제거)={} preserved (보존)={}",
        option_b_affected, option_b_preserved);
    println!("  max_shift={} HU", shifted_axes_max);
    if let Some((si, pi, l, t, r, b)) = affected_max_cell {
        println!("  max_cell: section={} pi={} pad(l={} t={} r={} b={})",
            si, pi, l, t, r, b);
    }
}

fn main() {
    for p in [
        "samples/21_언어_기출_편집가능본.hwp",
        "samples/exam_kor.hwp",
        "samples/exam_math.hwp",
        "samples/exam_eng.hwp",
        "samples/exam_science.hwp",
        "samples/synam-001.hwp",
    ] {
        scan(p);
    }
}

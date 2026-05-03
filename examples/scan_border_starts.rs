// Task #552 광범위 사전 평가: paragraph border 시작 직전 본문 paragraph 분포.
//
// Task #479 가 본문 paragraph (cell_ctx.is_none()) 마지막 줄에서 trailing ls
// 제외 → 직후 paragraph 가 border 시작이면 박스 top 이 위로 이동 (회귀).
//
// 본 도구는 6 샘플의 paragraph 시퀀스에서 다음 패턴 발견:
//   - prev paragraph border_fill_id=0 (no border)
//   - curr paragraph border_fill_id>0 visible
//   - 즉 회귀 영향 케이스
use std::fs;

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
    let para_shapes = &doc.doc_info.para_shapes;
    let border_fills = &doc.doc_info.border_fills;

    // border_fill_id > 0 + visible (any side has width > 0)
    let is_visible_border = |bf_id: u16| -> bool {
        if bf_id == 0 { return false; }
        let idx = (bf_id as usize).saturating_sub(1);
        let bf = match border_fills.get(idx) { Some(b) => b, None => return false };
        bf.borders.iter().any(|b| b.width > 0)
    };

    let mut total_paragraphs = 0u32;
    let mut total_no_border_to_border = 0u32;  // 회귀 영향 후보
    let mut total_in_border_groups = 0u32;
    let mut total_border_to_no_border = 0u32;
    let mut total_no_border = 0u32;
    let mut samples: Vec<(usize, usize, u16, u16)> = Vec::new();

    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            total_paragraphs += 1;
            let curr_bf = para_shapes.get(para.para_shape_id as usize)
                .map(|s| s.border_fill_id)
                .unwrap_or(0);
            let curr_visible = is_visible_border(curr_bf);

            if pi == 0 { continue; }
            let prev = &sec.paragraphs[pi - 1];
            let prev_bf = para_shapes.get(prev.para_shape_id as usize)
                .map(|s| s.border_fill_id)
                .unwrap_or(0);
            let prev_visible = is_visible_border(prev_bf);

            if !prev_visible && curr_visible {
                total_no_border_to_border += 1;
                if samples.len() < 5 {
                    samples.push((si, pi, prev_bf, curr_bf));
                }
            } else if prev_visible && curr_visible {
                total_in_border_groups += 1;
            } else if prev_visible && !curr_visible {
                total_border_to_no_border += 1;
            } else {
                total_no_border += 1;
            }
        }
    }

    println!("{}", path);
    println!("  total_paragraphs={} no→border={} in_border={} border→no={} no→no={}",
        total_paragraphs, total_no_border_to_border, total_in_border_groups,
        total_border_to_no_border, total_no_border);
    for (si, pi, prev_bf, curr_bf) in samples {
        println!("    sample: section={} pi={} prev_bf={} curr_bf={}",
            si, pi, prev_bf, curr_bf);
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

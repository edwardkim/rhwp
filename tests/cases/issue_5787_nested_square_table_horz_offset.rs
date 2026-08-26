//! [Issue #5787] 칸 안 어울림(SQUARE) 중첩 표가 저장 horzOffset 을 안 써 왼쪽으로
//! 49.9px 치우친다 (#5702 의 가로 축 잔여, 2025571).
//!
//! 원본: `horzRelTo=COLUMN horzAlign=LEFT horzOffset=7975HU(106.33px)`, 셀 안여백
//! 566HU. 한글 2022 = 칸 왼끝 462.31 + 7.55 + 106.33 = **576.17** (0.02px 재현).
//! rhwp 는 #3308 가운데-배치 계약이 과일반화되어 526.23 에 놓았다.
//!
//! 수정(compute_table_x_position depth>0): 양의 horzOffset + LEFT 정렬 + 그 자리로
//! 셀 안에 온전히 들어가는 표는 저장 오프셋을 그대로 쓴다. 오프셋 0 표는 종전
//! 가운데 계약(#3308) 불변.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5787/nested_square_table_horz_offset.hwp";

#[test]
fn issue_5787_nested_square_table_honors_stored_horz_offset() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 중첩 표(이상유무/○✕)의 세로 괘선: 한글 576.17 / 667.96. 결함 시 526.2 / 617.6.
    let mut xs = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x1), Some(x2), Some(y1)) = (attr("x1"), attr("x2"), attr("y1")) {
            if (x1 - x2).abs() < 0.5 && (700.0..950.0).contains(&y1) {
                xs.push(x1);
            }
        }
    }
    assert!(
        xs.iter().any(|x| (x - 576.5).abs() < 2.0),
        "중첩 표 왼쪽 괘선이 한글 자리(576.2)여야 한다 — 결함 시 526.2: {xs:?}"
    );
    assert!(
        !xs.iter().any(|x| (x - 526.2).abs() < 2.0),
        "가운데-배치 결함 위치(526.2)의 괘선이 남아 있으면 안 된다: {xs:?}"
    );
}

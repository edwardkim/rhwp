//! [Issue #5825] 줄인 표 행에서 항목 이름 받침이 아래 괘선을 지나간다
//! (156673604 34쪽, 두 통계표 34행 전부).
//!
//! 근인: 기계생성 통계표의 lineseg 가 **baseline == textheight**(하강부 0)를
//! 저장한다(bl=1100=vertsize·spacing=0). rhwp 는 저장값 그대로 baseline 을 줄
//! 상자 바닥에 놓아 받침 자리가 0.85px 뿐이었다. 한글 2022 는 이 퇴화값을
//! 무시하고 표준 ascent 로 그린다(실측 12.62px = 0.86×; 같은 문서의 정상 표
//! 저장값도 935 = 0.85×1100).
//!
//! 수정: 하강부 0 인 저장 baseline 만 0.85×textheight 로 되돌린다. 정상 저장
//! baseline(bl < th)은 그대로다.
//!
//! 픽스처는 원본 HWPX 구역2 문단 215..222(21행 통계표 2개) 절단 축소본.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5825/shrunk_row_degenerate_baseline.hwpx";

#[test]
fn issue_5825_degenerate_baseline_leaves_descent_room() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 첫 항목 이름 `농림어업` 의 `농` baseline. 결함 시 226.65(줄 바닥 = 저장
    // bl 1100 그대로), 정상 224.45(0.85×th, 한글 실측 224.59).
    let mut ys = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let Some(end) = cap.find("</text>") else {
            continue;
        };
        let node = &cap[..end];
        if !node.ends_with(">농") {
            continue;
        }
        if let Some(y) = node
            .split_once("y=\"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .and_then(|(v, _)| v.parse::<f64>().ok())
        {
            ys.push(y);
        }
    }
    assert!(!ys.is_empty(), "항목 이름 글자가 있어야 한다");
    let first = ys.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        (223.4..225.5).contains(&first),
        "퇴화 저장 baseline 이 0.85×textheight 로 복원돼야 한다 \
         (한글 224.59, 결함 시 226.65): {first:.2}"
    );
}

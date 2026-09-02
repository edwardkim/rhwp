//! [Issue #5756] 위첨자를 70%로 그리면서 전진폭은 100%로 재 칸 밖으로 넘친다.
//!
//! 한글 저장 lineseg 정답지(156732409 3쪽): 칸 안쪽 폭 24,596HU(327.95px)에 담긴
//! 40글자 줄이 rhwp 에서는 382.9px 를 써 칸 오른쪽 괘선(x≈713.6)을 54.9px 넘었다.
//! 초과분 55.5px = 위첨자 run 전진 합(185.0px) × (1 − 0.7). 수정: 첨자 run 의
//! 측정 글꼴 크기를 그리기 배율(0.7)로 통일(`style_params`), fit 배율은 항등으로.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5756/156732409_superscript_advance.hwp";

#[test]
fn issue_5756_superscript_line_stays_inside_cell() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(2).expect("page 3 svg");

    // 3쪽 표 오른쪽 괘선 x≈713.6 — 종전에는 위첨자 줄 글자가 761.7px 까지 넘었다.
    let mut max_text_x = 0.0f64;
    for cap in svg.split("<text x=\"").skip(1) {
        if let Some(end) = cap.find('"') {
            if let Ok(x) = cap[..end].parse::<f64>() {
                max_text_x = max_text_x.max(x);
            }
        }
    }
    assert!(
        max_text_x < 714.5,
        "3쪽 글자가 표 오른쪽 괘선(≈713.6) 안에 있어야 한다: max x={max_text_x:.1} \
         (위첨자 전진폭 결함이면 ≈761.7)"
    );

    // 위첨자 글리프와 보통 글리프가 모두 존재해야 검증이 유효하다.
    // [#5821] 압축 장평(이 문서 ratio=95%)은 세로도 √r 축소 — 10.27→10.00,
    // 14.67→14.29.
    assert!(
        svg.contains("font-size=\"10.0"),
        "위첨자 글리프(0.7배×√r)가 있어야 한다"
    );
    assert!(
        svg.contains("font-size=\"14.2"),
        "본문 글리프가 있어야 한다"
    );
}

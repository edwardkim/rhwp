//! [Issue #5789] 글자처럼 취급 선 도형을 baseline 이 아니라 줄 상자 top 에 놓아
//! 20.4px 위로 떠 제목 글자를 관통한다 (#5711 재발·원인 재지정, 3143955).
//!
//! 근인: 선 도형(hp:line, treatAsChar=1, 높이 1HU)이 빈 run 줄에 홀로 앉는데,
//! 빈 줄은 max_fs=0 이라 `vars.baseline` 이 0 으로 접혀 `(y + 0 - h)` = 줄 상자
//! top(161.99px)에 등록됐다. 한글 2022 는 저장 lineseg baseline(1530HU=20.4px)
//! 자리(중심 182.12px)에 그린다.
//!
//! 수정: 빈 run 줄의 TAC Shape 등록에서 baseline 이 접혔으면(≤0.01) composed 줄의
//! 저장 `baseline_distance` 로 폴백 — 중심 182.09px, 한글과 0.03px 차.
//!
//! 픽스처는 원본 HWPX 의 대형 BinData 를 1×1 스텁으로 바꾼 축소본(14KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5789/tac_line_shape_baseline.hwpx";

#[test]
fn issue_5789_tac_line_sits_on_stored_baseline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 제목 아래 이중선(SLIM_THICK): x≈94.5~699.2 의 두 <line>. 결함 시 y 159.7/163.7
    // (중심 161.7 = 줄 상자 top), 정상 시 180.1/184.1 (중심 182.1 = 저장 baseline).
    let mut ys = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x1), Some(x2), Some(y1)) = (attr("x1"), attr("x2"), attr("y1")) {
            if (x1 - 94.5).abs() < 1.0 && (x2 - 699.2).abs() < 1.0 && y1 < 200.0 {
                ys.push(y1);
            }
        }
    }
    assert_eq!(ys.len(), 2, "제목 밑 이중선 2줄이어야 한다: {ys:?}");
    let center = (ys[0] + ys[1]) / 2.0;
    assert!(
        (center - 182.1).abs() < 1.5,
        "이중선 중심({center:.2})이 저장 baseline(한글 182.12) 자리에 있어야 한다 — \
         결함 시 161.7(줄 상자 top)"
    );
}

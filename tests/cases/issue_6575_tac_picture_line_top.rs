//! [Issue #6575] 줄이 그림보다 크게 저장된 빈 줄의 TAC 그림이 줄 상단 대신
//! baseline 에 바닥을 맞춰 앉아 25.5pt 내려간다 (156489219 5쪽, #6494 잔여).
//!
//! 근인: 빈 run 줄의 TAC Picture 배치가 `(y + baseline - pic_h).max(y)` 로
//! baseline 바닥 맞춤만 알았다. 156489219 5쪽 그림 줄은 저장 lineseg 가
//! lh=21235(283.1px)·bl=18050(240.7px)인데 그림은 205.7px 라 bl−h=+35px 만큼
//! 내려갔다. 한글 2024 는 같은 그림을 저장 lineseg 상단(176.0pt)에 그린다.
//!
//! 수정: 빈 줄에서 저장 줄 높이가 그림보다 4px 넘게 크면 줄 상단에 붙인다.
//! 보통 줄(lh≈h)은 두 규칙이 같은 답이라 기존 경로를 유지하고, 선 도형은
//! baseline 이 정답(#5789)이라 Shape 분기를 건드리지 않는다.
//!
//! 픽스처는 원본 HWP 를 HWPX 변환 후 secPr 문단 + 그림 문단만 남기고
//! BinData 를 1×1 스텁으로 바꾼 축소본(22KB). 결함 lineseg
//! (lh=21235 th=21235 bl=18050) 와 TAC 그림(curSz h=15425HU=205.7px)을
//! 그대로 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6575/tac_picture_line_top.hwpx";

#[test]
fn issue_6575_tac_picture_sits_on_line_top_when_line_is_taller() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 대상: 폭 557.25px 의 TAC 그림 (원본 5쪽 스크린샷 자리). 결함 시
    // y=479.68 (줄 상단 + bl−h = +35.0px), 정상 시 y=444.68 (줄 상단).
    let mut target_y = None;
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(w), Some(y)) = (attr("width"), attr("y")) {
            if (w - 557.25).abs() < 1.0 {
                target_y = Some(y);
            }
        }
    }
    let y = target_y.expect("폭 557.25px 의 TAC 그림이 SVG 에 있어야 한다");
    assert!(
        (y - 444.68).abs() < 1.5,
        "TAC 그림 상단({y:.2})이 줄 상단(444.68) 에 있어야 한다 — \
         결함 시 479.68 (baseline 바닥 맞춤, bl−h=+35px)"
    );
}

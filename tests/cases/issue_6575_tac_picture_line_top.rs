//! [Issue #6575] 줄이 그림보다 크게 저장된 빈 줄의 TAC 그림이 줄 상단 대신
//! baseline 에 바닥을 맞춰 앉아 25.5pt 내려간다 (156489219 5쪽, #6494 잔여).
//!
//! 근인: 빈 run 줄의 TAC Picture 배치가 그림 높이만으로 baseline 에 맞췄다.
//! 그러나 156489219 5쪽 그림에는 Bottom 캡션이 있어 저장 lineseg 는 그림,
//! 캡션 간격, 캡션 문단을 함께 예약한다. 그림만 맞추면 캡션 몫만큼 내려간다.
//! 한글 2024 는 전체 개체 상자를 기준으로 그림을 저장 lineseg 상단(176.0pt)에
//! 그린다.
//!
//! 수정: Top/Bottom 캡션의 간격과 문단 높이를 그림 높이에 더한 전체 개체 상자를
//! baseline 에 맞춘다. 캡션이 없는 그림은 기존 baseline 동작을 유지한다.
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
    let mut target_ys = Vec::new();
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
                target_ys.push(y);
            }
        }
    }
    assert_eq!(
        target_ys.len(),
        1,
        "폭 557.25px 의 TAC 그림은 정확히 하나여야 한다: {target_ys:?}"
    );
    let y = target_ys[0];
    assert!(
        (y - 444.68).abs() < 1.5,
        "TAC 그림 상단({y:.2})이 줄 상단(444.68) 에 있어야 한다 — \
         결함 시 479.68 (baseline 바닥 맞춤, bl−h=+35px)"
    );
}

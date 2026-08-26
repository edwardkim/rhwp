//! [Issue #5700] TAC 표 host 의 쪽-리셋 꼬리(PartialParagraph, 저장 vpos=0)가
//! [#677] TAC PP y-리셋 규칙을 타고 **문단 시작 위**(쪽 상단·심하면 쪽 밖 음수 y)
//! 로 튀어 그리기 순서가 역전되던 결함 — 해양경찰청 p139: pi759 꼬리가 흐름
//! 1004.9 대신 100.1 로 리셋되어 앞 문단(pi758, y388) 위에 그려졌다(INV).
//! 문화예술산업 p327 은 같은 기전으로 y −421(쪽 밖)까지 나간다.
//!
//! 수정: #677 리셋은 **정방향 델타**(seg.vpos ≥ seg0.vpos)에만 적용 — 되감긴
//! 꼬리(한글이 표 뒤에서 쪽을 끊은 흔적)는 순차 흐름(표 바닥)을 유지한다.
//!
//! 재현물은 원본(156쪽)의 구역 2 문단 705..=762 IR 슬라이스(30KB) — 쪽 9 가
//! 원본 p139 와 동형(pi52 표 / pi53 텍스트 / pi54 되감긴 표 host)이다.
//! 결함 상태에서는 pi54 꼬리가 y 100.1(< pi53 의 387.7)로 그려져 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp";

#[test]
fn issue_5700_tac_reset_tail_stays_in_flow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let tree = core.build_page_render_tree(8).expect("page 9 render tree");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON");

    let mut prev_text_y = f64::MIN; // pi53 텍스트 y
    let mut tail_min_y = f64::MAX; // pi54 꼬리 최소 y
    walk(&json, &mut prev_text_y, &mut tail_min_y);
    assert!(prev_text_y > f64::MIN, "pi53 텍스트가 있어야 한다");
    assert!(tail_min_y < f64::MAX, "pi54 꼬리 줄이 있어야 한다");
    assert!(
        tail_min_y > prev_text_y,
        "되감긴 TAC 꼬리는 앞 문단 위로 리셋되면 안 된다 (결함 시 100.1 < 387.7): \
         tail={tail_min_y:.1} prev={prev_text_y:.1}"
    );
}

fn walk(node: &serde_json::Value, prev_text_y: &mut f64, tail_min_y: &mut f64) {
    if let Some(obj) = node.as_object() {
        let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let pi = obj.get("pi").and_then(|v| v.as_i64());
        if ty == "TextRun" {
            if let Some(y) = obj
                .get("bbox")
                .and_then(|b| b.get("y"))
                .and_then(|v| v.as_f64())
            {
                if pi == Some(53) {
                    *prev_text_y = prev_text_y.max(y);
                } else if pi == Some(54) {
                    *tail_min_y = tail_min_y.min(y);
                }
            }
        }
        if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
            for child in children {
                walk(child, prev_text_y, tail_min_y);
            }
        }
    }
}

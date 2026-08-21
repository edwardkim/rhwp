//! [#5801] 저장 사다리가 문단 위 간격을 안 담은 문서에서 쪽 채움 회계가 짧아지지 않는다.
//!
//! `#2279 ①` 의 spacing 트림은 "저장 ladder 가 spacing 을 이미 반영한다"는 전제 위에 선다.
//! 그 게이트(`has_authoritative_seg`)는 lineseg 가 합성인지만 보는데, **합성이 아닌데도
//! 문단 위 간격을 안 담은** 사다리가 있다. 그런 문서에 트림이 걸리면 typeset 이 쪽 채움을
//! 문단마다 `spacing_before` 만큼 짧게 세고, 그 착시 위에서 다음 문단이 이미 꽉 찬 쪽에
//! 얹힌다(#5755 는 그 하류다).
//!
//! `samples/2025 행정업무운영 편람(최종).hwpx` 272쪽(section 9)이 그 형태다. 저장 좌표가
//! 말하는 쪽 채움은 748.2px 인데 트림이 걸리면 729.2px 로 18.9px 짧게 센다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

/// 저장 사다리가 문단 위 간격을 담지 않은 쪽 (0-기반 인덱스).
const LADDER_WITHOUT_SPACING_PAGE: usize = 271;

fn sample_doc() -> HwpDocument {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/2025 행정업무운영 편람(최종).hwpx"),
    )
    .expect("samples/2025 행정업무운영 편람(최종).hwpx");
    HwpDocument::from_bytes(&bytes).expect("샘플이 열려야 한다")
}

#[test]
fn issue_5801_typeset_used_height_matches_stored_ladder() {
    let doc = sample_doc();
    let pages = doc.dump_page_items_json(None);
    let pages = pages.as_array().expect("pages array");
    let page = pages
        .get(LADDER_WITHOUT_SPACING_PAGE)
        .expect("272쪽이 있어야 한다");
    let col = page["columns"]
        .as_array()
        .and_then(|c| c.first())
        .expect("단 0");

    let used = col["usedHeight"].as_f64().expect("usedHeight");
    let hwp_used = col["hwpUsedHeight"].as_f64().expect("hwpUsedHeight");
    let diff = col["usedDiff"].as_f64().expect("usedDiff");

    // 트림이 걸리면 used 가 hwp_used 보다 18.9px 짧아진다. 게이트가 그걸 막는다.
    assert!(
        diff.abs() <= 1.0,
        "저장 사다리가 문단 위 간격을 안 담은 쪽에서는 쪽 채움 회계가 저장 좌표와 맞아야 한다 \
         (used={used:.1} hwp_used={hwp_used:.1} diff={diff:+.1})"
    );
}

#[test]
fn issue_5801_gate_does_not_move_the_page_boundary() {
    // 회계만 고치고 쪽 경계는 건드리지 않는다 — 코퍼스 스윕에서도 쪽수 변화 0건이었다.
    let doc = sample_doc();
    assert_eq!(
        doc.page_count(),
        383,
        "이 샘플의 쪽수는 게이트 전후로 같아야 한다"
    );
}

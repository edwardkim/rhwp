//! [#5543] compat 2024 — 자리차지 스택의 이월 앵커 사다리 계상 계약.
//!
//! 빈 호스트의 자리차지 스택이 쪽 하단에 걸리면 한글 2022 계열은 호스트의 마지막
//! 앵커 줄을 다음 쪽 상단으로 이월해 계상한다(저장 lineseg 마지막 세그 vpos=0).
//! 한글 2024 는 이월 계상을 하지 않아 다음 쪽 사다리가 그만큼(이 샘플 3,597HU)
//! 위로 당겨지고, 하류 쪽 경계(pi42)가 한 쪽 앞으로 온다 — 재저장 오라클 실측.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn sample_bytes() -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue5543_carried_anchor_ladder.hwpx"),
    )
    .expect("sample")
}

/// 문단이 처음 등장하는 1-기반 쪽 번호.
fn para_page(doc: &HwpDocument, target_pi: u64) -> u64 {
    let v = doc.dump_page_items_json(None);
    let pages = v.as_array().expect("pages array");
    for (pg_idx, page) in pages.iter().enumerate() {
        for col in page["columns"].as_array().expect("columns") {
            for item in col["items"].as_array().expect("items") {
                if item["paraIndex"].as_u64() == Some(target_pi) {
                    return (pg_idx as u64) + 1;
                }
            }
        }
    }
    panic!("pi {target_pi} 미배치");
}

#[test]
fn hangul2024_compat_skips_carried_anchor_ladder_charge() {
    let mut doc = HwpDocument::from_bytes(&sample_bytes()).expect("parse");
    // 기본값(2022 계열): 이월 앵커 줄이 다음 쪽 상단을 소비 — pi42 는 4쪽.
    assert_eq!(
        para_page(&doc, 42),
        4,
        "기본값 조판이 바뀌면 안 된다 — 플래그 없는 경로는 2022 계열 유지"
    );
    // compat 2024: 이월 계상을 걷어 pi42 가 3쪽 — 한글 2024 재저장 지문과 일치.
    doc.set_hangul2024_compat(true);
    assert_eq!(
        para_page(&doc, 42),
        3,
        "--compat 2024 는 이월 앵커 계상을 하지 않는다"
    );
    // 끄면 원상 복귀.
    doc.set_hangul2024_compat(false);
    assert_eq!(para_page(&doc, 42), 4);
}

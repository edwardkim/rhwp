//! [#5524] 한글 2024 조판 호환 플래그(Δ1: 자리차지 표 앵커 줄 계상) 계약.
//!
//! 한글 2022 계열은 자리차지(TAC) 표 앵커 문단에 선행 앵커 줄 세그먼트를 흐름에
//! 계상하고, 2024 는 이를 제거했다(재저장 오라클 실측 — 2022 재저장본 앵커 문단
//! lineseg 2개 vs 2024 재저장본 1개). 샘플은 그 차이가 쪽 경계를 가르는 실문서다:
//! 레터헤드 TAC 표(pi0, 선행 세그 1600HU) 때문에 2022 는 pi13 을 2쪽 상단으로,
//! 2024 는 1쪽 말미로 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn sample_bytes() -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue5524_hangul2024_compat_letterhead.hwp"),
    )
    .expect("sample")
}

/// 문단이 처음 등장하는 1-기반 쪽 번호.
fn para_page(doc: &HwpDocument, target_pi: u64) -> u64 {
    let v = doc.dump_page_items_json(None);
    // dump_page_items_json 은 쪽 배열 자체를 돌려준다 (CLI 가 "pages" 로 감싼다).
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
fn hangul2024_compat_reclaims_tac_anchor_line_and_default_stays_2022() {
    let mut doc = HwpDocument::from_bytes(&sample_bytes()).expect("parse");
    // 기본값(2022 계열): 저장 vpos 리셋(pi13)이 쪽 경계로 존중된다.
    assert_eq!(
        para_page(&doc, 13),
        2,
        "기본값 조판이 바뀌면 안 된다 — 플래그 없는 경로는 2022 계열 유지"
    );
    // compat 2024: 레터헤드 TAC 표(pi0)의 선행 앵커 줄을 회수해 pi13 이 1쪽
    // 말미로 들어간다 — 한글 2024 재저장 지문과 일치.
    doc.set_hangul2024_compat(true);
    assert_eq!(
        para_page(&doc, 13),
        1,
        "--compat 2024 는 앵커 줄 회수분만큼 더 채운다"
    );
    // 끄면 재페이지네이션으로 원상 복귀.
    doc.set_hangul2024_compat(false);
    assert_eq!(para_page(&doc, 13), 2);
    assert_eq!(doc.page_count(), 2);
}

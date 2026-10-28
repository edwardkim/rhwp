//! [Issue #6208] 문서에 실린 **인쇄 방식**(모아 찍기)을 rhwp 가 읽지 않아, 한글이
//! 가로 한 장에 두 쪽씩 찍는 문서를 세로 여러 쪽으로 낸다.
//!
//! 모아 찍기 **출력 구현**은 별개 축이다. 이 계약은 이슈가 제시한 최소 수용선 —
//! **문서에 실린 값을 읽어 노출**해서 한글 오라클 대조 시 오판을 막는 것 — 을 잠근다.
//!
//! 저장 위치는 포맷마다 다르고 값은 같다.
//!
//! | 포맷 | 위치 |
//! |---|---|
//! | HWP5 | `DocInfo` / `HWPTAG_DOC_DATA`(tag 27)의 `(u32 key, u32 value)` 중 키 `0x0006_4006` |
//! | HWPX | `settings.xml` 의 `<config:config-item name="PrintMethod">` |
//!
//! **키 순서는 문서마다 다르다** — 인덱스가 아니라 키로 찾아야 한다.
//!
//! 한글 2020 실측(코퍼스 표본 1건씩, COM `FileSaveAs` PDF)으로 값의 의미를 확정했다:
//!
//! | 값 | 한글 출력 | rhwp 출력 |
//! |---|---|---|
//! | 0 · 1 · 3 | 세로, 쪽수 일치 | 일치 |
//! | **4** | **2쪽 841×595 가로** | 4쪽 세로 |
//! | **5** | **1쪽 841×595 가로** | 3쪽 세로 |
//!
//! 즉 0·1·3 은 용지 기하에 영향이 없고 **4·5 만** 장 수·방향을 바꾼다.
//! 코퍼스 591문서 중 121건(20.5%)이 비-기본 값을, 그중 50건이 4·5 를 싣고 있었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::document::print_method_implies_nup;
use rhwp::parser::parse_document;

/// 모아 찍기(PrintMethod=4)를 싣고 있는 HWP5 슬라이스.
const NUP_SAMPLE: &str = "samples/issue6208/print_method_nup.hwp";

#[test]
fn issue_6208_hwp5_doc_data_carries_print_method() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(NUP_SAMPLE);
    let doc = parse_document(&std::fs::read(path).expect("read sample")).expect("parse");

    assert_eq!(
        doc.doc_info.print_method,
        Some(4),
        "HWPTAG_DOC_DATA 의 키 0x0006_4006 을 읽어야 한다 — 인덱스가 아니라 키로 찾을 것"
    );
    assert!(
        print_method_implies_nup(doc.doc_info.print_method),
        "4 는 모아 찍기다 — 한글은 841x595 가로 한 장에 두 쪽을 찍는다"
    );
}

/// 0·1·3 은 용지 기하에 영향이 없다 — 한글 오라클 실측으로 확정한 경계다.
#[test]
fn issue_6208_only_four_and_five_imply_nup() {
    for value in [0u32, 1, 3] {
        assert!(
            !print_method_implies_nup(Some(value)),
            "PrintMethod={value} 는 한글도 세로 1-up 이다 — 모아 찍기로 표시하면 안 된다"
        );
    }
    for value in [4u32, 5] {
        assert!(
            print_method_implies_nup(Some(value)),
            "PrintMethod={value} 는 한글이 가로 N-up 으로 낸다"
        );
    }
    assert!(
        !print_method_implies_nup(None),
        "문서에 항목이 없으면 모아 찍기로 단정하지 않는다"
    );
}

/// 파싱은 **파생**이라 저장 산출물을 건드리지 않는다 — 권위 바이트는
/// `extra_records` 의 `HWPTAG_DOC_DATA` 가 그대로 들고 있다.
#[test]
fn issue_6208_print_method_is_derived_not_authoritative() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(NUP_SAMPLE);
    let doc = parse_document(&std::fs::read(path).expect("read sample")).expect("parse");

    let doc_data = doc
        .doc_info
        .extra_records
        .iter()
        .find(|r| r.tag_id == rhwp::parser::tags::HWPTAG_DOC_DATA)
        .expect("DOC_DATA 원본 레코드가 보존돼야 한다");
    assert!(
        doc_data.data.len() >= 8 && doc_data.data.len() % 8 == 0,
        "DOC_DATA 는 (u32 key, u32 value) 쌍의 목록이다 — 실측 80바이트(10쌍), \
         실제 {}바이트",
        doc_data.data.len()
    );
    assert!(
        doc_data
            .data
            .chunks_exact(8)
            .any(|kv| u32::from_le_bytes([kv[0], kv[1], kv[2], kv[3]])
                == rhwp::model::document::HWP5_DOC_DATA_KEY_PRINT_METHOD),
        "원본 바이트에 인쇄 방식 키가 그대로 남아 있어야 한다(저장 무손실)"
    );
}

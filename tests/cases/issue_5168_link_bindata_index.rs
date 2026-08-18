//! [Issue #5168] HWP5 → HWPX 저장에서 외부 연결(LINK) 그림이 엉뚱한 내장 이미지로 바뀌고
//! 내장 이미지 개수를 넘는 것은 소실되던 문제.
//!
//! HWP5 본문 그림은 `BIN_DATA` 레코드 **순번**(1-based)으로 이미지를 참조한다. 외부 연결
//! (LINK) BinData 는 `storage_id` 가 없어(0) HWPX 직렬화기의 등록 루프들에서 모두 빠졌고,
//! LINK 를 참조하는 그림은 `resolve_bin_id` 의 직접 조회 폴백으로 떨어져
//!   - 순번이 내장 이미지 `storage_id` 범위 안이면 **그 내장 이미지로 충돌**(`image{순번}`),
//!   - 범위를 넘으면 **미등록으로 드롭**(그림 소실)
//!
//! 됐다. 실측 `07605`(2018년 연구용역 결과보고서, LINK 7·EMBEDDING 5): h2x 에서 `gso 12 → pic
//! 10`, `image1`~`image5` 가 각 2회 참조됐다.
//!
//! 수정: LINK 마다 기존 키와 겹치지 않는 새 manifest id 를 배정해 외부 참조 엔트리로 등록하고
//! 순번을 사상한다. 계약: 같은 문단에 LINK 그림과 내장 그림이 섞여도 서로 다른 이미지로
//! 해결돼야 하며, LINK 가 내장 이미지 id 로 충돌하거나 드롭되면 안 된다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::bin_data::{BinData, BinDataContent, BinDataType};
use rhwp::model::document::Document;
use rhwp::serializer::hwpx::context::SerializeContext;

fn link(path: &str) -> BinData {
    BinData {
        data_type: BinDataType::Link,
        abs_path: Some(path.to_string()),
        extension: Some("jpg".to_string()),
        ..Default::default()
    }
}

fn embedding(storage_id: u16) -> BinData {
    BinData {
        data_type: BinDataType::Embedding,
        storage_id,
        extension: Some("jpg".to_string()),
        ..Default::default()
    }
}

#[test]
fn link_pictures_do_not_alias_embedded_images() {
    let mut doc = Document::default();
    // BIN_DATA 레코드 순번(1-based): 1=LINK, 2=LINK, 3=EMBEDDING(storage_id=1).
    // 종전엔 LINK 순번 1 이 내장 이미지 storage_id 1(=image1)과 충돌하고, 순번 2 는
    // 미등록으로 드롭됐다.
    doc.doc_info.bin_data_list = vec![link("A.jpg"), link("B.jpg"), embedding(1)];
    doc.bin_data_content.push(BinDataContent {
        id: 1,
        data: vec![0u8; 4].into(),
        extension: "jpg".to_string(),
    });

    let ctx = SerializeContext::collect_from_document(&doc);
    let r_link1 = ctx.resolve_bin_id(1).map(str::to_string); // LINK 순번 1
    let r_link2 = ctx.resolve_bin_id(2).map(str::to_string); // LINK 순번 2
    let r_embed = ctx.resolve_bin_id(3).map(str::to_string); // EMBEDDING 순번 3 → image1

    assert!(
        r_link1.is_some() && r_link2.is_some() && r_embed.is_some(),
        "세 그림 모두 manifest 로 해결돼야 한다 (LINK 드롭 없음): link1={r_link1:?} link2={r_link2:?} embed={r_embed:?}"
    );
    assert_eq!(
        r_embed.as_deref(),
        Some("image1"),
        "내장 그림은 image{{storage_id}}=image1 을 유지해야 한다 (#1891 불변식)"
    );
    assert_ne!(
        r_link1, r_embed,
        "LINK 순번 1 이 내장 image1 로 충돌하면 안 된다 (#5168 회귀)"
    );
    assert_ne!(
        r_link2, r_embed,
        "LINK 순번 2 가 내장 image1 로 충돌하거나 드롭되면 안 된다 (#5168 회귀)"
    );
    assert_ne!(r_link1, r_link2, "두 LINK 가 같은 id 로 뭉치면 안 된다");
}

//! Issue #2222: 페이지 레이어 트리 JSON 캐시 — renderPage 반복 비용 제거.
//!
//! #2227의 DOM 이미지 경로가 renderPage 마다 `getPageLayerTree`(주보 p2 기준
//! 1.05MB JSON)를 호출하는데, 트리 캐시가 있어도 LayerBuilder 재실행 + 재직렬화
//! 가 회당 15.2ms(실측)로 렌더 자체와 맞먹었다 — 스크롤/입력/토글 후 반복
//! 렌더의 체감 저하 원인. (출력옵션 지문, JSON) 변형 캐시(페이지당 4개)로
//! 반복 호출을 문자열 복제 수준(~65µs)으로 낮춘다.

use std::fs;
use std::path::Path;
use std::time::Instant;

#[test]
fn issue_2222_layer_json_cache_correctness_and_speed() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");

    let j1 = doc.get_page_layer_tree(1).expect("layer tree p2");
    assert!(
        j1.len() > 100_000,
        "p2 레이어 JSON이 비정상적으로 작음: {}",
        j1.len()
    );

    // 1) 캐시 일관성: 반복 호출 결과 동일.
    let j1b = doc.get_page_layer_tree(1).unwrap();
    assert_eq!(j1, j1b, "캐시 히트 결과가 원본과 다름");

    // 2) 옵션 지문: 문단부호 토글 시 다른 JSON, 원복 시 원본과 동일.
    doc.set_show_paragraph_marks(true);
    let j2 = doc.get_page_layer_tree(1).unwrap();
    assert_ne!(j1, j2, "옵션 변경이 캐시에 반영되지 않음 (stale)");
    doc.set_show_paragraph_marks(false);
    let j3 = doc.get_page_layer_tree(1).unwrap();
    assert_eq!(j1, j3, "옵션 원복 후 결과가 원본과 다름");

    // 3) 성능 가드: 히트 20회가 리빌드 1회(≈15ms)보다 확실히 빨라야 한다.
    //    (여유 임계 — CI 변동 감안 회당 3ms, 실측 65µs.)
    let t0 = Instant::now();
    for _ in 0..20 {
        let _ = doc.get_page_layer_tree(1).unwrap();
    }
    let per_call = t0.elapsed() / 20;
    assert!(
        per_call.as_micros() < 3_000,
        "레이어 JSON 캐시 히트가 느림: 회당 {per_call:?} (기대 <3ms, 캐시 부재 시 ≈15ms) — #2222 회귀"
    );
}

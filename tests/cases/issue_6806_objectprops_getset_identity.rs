//! [#6806] `get ∘ set` 은 항등이어야 한다 — 같은 봉지를 되먹여도 문서가 바뀌면 안 된다.
//!
//! # 무엇이 깨져 있었나
//!
//! 게터가 내보낸 속성 봉지를 **그대로** 세터에 되먹이면 문서가 바뀌었다. 되돌리기가
//! 이 세터를 타므로(`ResizeObjectCommand.undo` 가 `{width, height}` 를 보낸다)
//! **undo 가 역연산이 아니게 된다.**
//!
//! 앞선 PR 들이 원본 렌더링 행렬(`raw_rendering`) 손실과 `original_*` 덮어쓰기를
//! 막았고, 이 검사는 남아 있던 두 층을 잠근다 — 둘 다 **rhwp 가 세운 불변식이
//! 한컴 파일과 어긋나는** 경우다.
//!
//! ## (5) 「쪽 영역 제한」이 「겹침 허용」을 끄던 강제
//!
//! 한컴은 두 플래그를 **동시에 켜서 저장한다**.
//! `samples/basic/interview.hwp` 의 본문 그림이 그 상태(`restrictInPage` · `allowOverlap` 둘 다 참)인데, 게터가
//! 내보낸 `allowOverlap:true` 를 되먹이면 그림 세터 끝의 무조건 강제가 false 로
//! 뒤집었다. 코퍼스 계수로 두 플래그가 함께 켜진 개체는 그림 518 중 70건이다.
//!
//! ## (4) 최소 크기 클램프가 되먹임에도 걸리던 것
//!
//! 클램프의 원 목적은 리사이즈 핸들을 반대편으로 넘길 때 studio 가 보내는 **0** 이다.
//! 그런데 한컴 문서에는 높이 0 인 도형이 정당하게 저장되어 있고(아래 재현물),
//! 그 값을 되먹이면 200 으로 부풀었다. 그 자리에서 0 을 올려도 보호되는 것은 없다 —
//! 문서는 이미 0 으로 저장되어 그대로 그려지고 있었다.
//!
//! # 검사 범위
//!
//! 두 재현물의 **실제 좌표**에서 `get → set(같은 봉지) → get` 이 문자열까지 같은지 본다.
//! 그리고 반대쪽을 함께 잠근다 — **값을 실제로 0 으로 떨어뜨리는 편집**은 종전대로
//! 클램프돼야 한다(그게 클램프의 존재 이유다).
//!
//! 이 계약은 `samples/**/*.hwp` 3MB 이하 577문서 · 개체 989건(그림 434 · 도형 555)
//! 전수에서 `get∘set∘get == get` 이 성립함을 확인하고 대표 둘을 고정한 것이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("재현물 읽기 {rel}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("문서 로드 {rel}: {e:?}"))
}

/// 그림 — 「쪽 영역 제한」과 「겹침 허용」이 함께 켜진 개체를 되먹여도 그대로다.
#[test]
fn picture_feedback_keeps_allow_overlap_under_restrict_in_page() {
    const SAMPLE: &str = "samples/basic/interview.hwp";
    let mut core = load(SAMPLE);
    let before = core
        .get_picture_properties_native(0, 0, 2)
        .expect("그림 속성 읽기");
    assert!(
        before.contains("\"restrictInPage\":true") && before.contains("\"allowOverlap\":true"),
        "재현물 전제가 깨졌다 — 두 플래그가 함께 켜진 그림이어야 한다: {before}"
    );

    core.set_picture_properties_native(0, 0, 2, &before)
        .expect("같은 봉지 되먹이기");
    let after = core
        .get_picture_properties_native(0, 0, 2)
        .expect("그림 속성 다시 읽기");

    assert_eq!(
        after, before,
        "같은 봉지를 되먹였는데 그림 속성이 바뀌었다 — 「쪽 영역 제한」이 켜졌다는 이유로 \
         「겹침 허용」을 끄면 안 된다(한컴은 둘을 함께 저장한다). undo 봉지도 이 경로를 탄다."
    );
}

/// 도형 — 문서가 저장한 **0** 높이를 되먹여도 그대로다.
#[test]
fn shape_feedback_keeps_stored_zero_size() {
    const SAMPLE: &str = "samples/issue6023/30269_reform_recommendation.hwp";
    let mut core = load(SAMPLE);
    let before = core
        .get_shape_properties_native(0, 28, 0)
        .expect("도형 속성 읽기");
    assert!(
        before.contains("\"height\":0"),
        "재현물 전제가 깨졌다 — 높이 0 으로 저장된 도형이어야 한다: {before}"
    );

    core.set_shape_properties_native(0, 28, 0, &before)
        .expect("같은 봉지 되먹이기");
    let after = core
        .get_shape_properties_native(0, 28, 0)
        .expect("도형 속성 다시 읽기");

    assert_eq!(
        after, before,
        "같은 봉지를 되먹였는데 도형 크기가 바뀌었다 — 문서가 이미 0 을 저장하고 있으면 \
         «0 으로 떨어뜨린 편집» 이 아니므로 최소 크기 클램프를 걸면 안 된다."
    );
}

/// 반례 — 값을 **실제로** 0 으로 떨어뜨리는 편집은 종전대로 클램프된다.
///
/// 클램프의 존재 이유(리사이즈 핸들을 반대편으로 넘기면 studio 가 0 을 보낸다)를
/// 함께 잠근다. 위 완화를 이 경우까지 넓히면 도형이 화면에서 사라진다.
#[test]
fn shape_edit_that_drops_a_nonzero_size_to_zero_still_clamps() {
    const SAMPLE: &str = "samples/issue6023/30269_reform_recommendation.hwp";
    let mut core = load(SAMPLE);
    let Ok(before) = core.get_shape_properties_native(0, 0, 0) else {
        // 이 재현물에 좌표 (0,0,0) 도형이 없으면 계약 대상이 아니다.
        return;
    };
    let value: serde_json::Value = serde_json::from_str(&before).expect("봉지 파싱");
    let Some(h) = value.get("height").and_then(|v| v.as_u64()) else {
        return;
    };
    if h == 0 {
        return; // 이미 0 이면 이 반례의 전제가 아니다.
    }

    core.set_shape_properties_native(0, 0, 0, r#"{"height":0}"#)
        .expect("0 으로 떨어뜨리는 편집");
    let after = core
        .get_shape_properties_native(0, 0, 0)
        .expect("도형 속성 다시 읽기");
    let after_h = serde_json::from_str::<serde_json::Value>(&after)
        .ok()
        .and_then(|v| v.get("height").and_then(|x| x.as_u64()))
        .expect("높이 읽기");

    assert!(
        after_h >= 200,
        "0 으로 떨어뜨리는 편집은 최소 크기로 올라와야 한다 — 이 보호까지 풀면 도형이 \
         화면에서 사라진다. 이전 높이={h} 이후 높이={after_h}"
    );
}

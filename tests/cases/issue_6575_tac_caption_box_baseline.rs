//! Issue #6575: 위/아래 캡션이 붙은 TAC 그림을 **그림 높이만으로** baseline 에
//! 바닥맞춤해, 캡션 높이만큼 아래로 내려가던 결함의 가드.
//!
//! TAC(글자처럼 취급) 개체는 글자처럼 baseline 에 앉는다. 그런데 캡션이 붙은 그림은
//! 저장 줄이 **그림 + 캡션 간격 + 캡션**을 통째로 예약한다. 그림만 바닥맞춤하면
//! 그림이 캡션 몫만큼 밀려 내려가고, 그 아래 내용이 전부 따라 밀린다.
//!
//! 재현 문서 `samples/issue6575/156489219_satellite_pm_release.hwp` 5쪽 `pi=43`
//! (한글 2024 실측 — COM 이 이 버전으로만 해석된다):
//!
//! ```text
//! y=233.7  baseline=240.7  pic_h=205.7  raw_lh=283.1
//! caption = Bottom(spacing 850HU, 문단 3개)
//!
//! 수정 전  233.7 + 240.7 - 205.7             = 268.7px   (한/글 233.3 대비 +35.4)
//! 수정 후  (233.7 + 240.7 - 283.1).max(233.7) = 233.7px   ✔
//! ```
//!
//! 상자 전체 높이가 baseline 보다 크면 기존 `.max(y)` 클램프가 그대로 줄 상단을 준다 —
//! 한컴이 이런 줄에서 개체를 줄 상단에 붙이는 동작과 같은 답이다.
//!
//! PDF 실측(한글 2024 오라클 대조, 5쪽 매칭 텍스트 편차):
//!
//! ```text
//! median 24.46pt -> 1.79pt      첫 그림 y 201.5pt -> 175.2pt  (한/글 176.0)
//! ```
//!
//! ⚠ 같은 쪽 둘째 그림은 여전히 18pt 낮다(425.9 vs 407.9). 별개 축이라 여기서 닫지
//! 않았다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6575/156489219_satellite_pm_release.hwp";

/// 캡션 몫(35.4px)을 다시 더하면 268.7px 로 돌아간다. 그 사이에 가드를 둔다.
const MAX_FIRST_PICTURE_Y: f64 = 250.0;

fn collect_image_tops(node: &serde_json::Value, out: &mut Vec<(String, f64, f64)>) {
    if let (Some(ty), Some(bbox)) = (node.get("type").and_then(|t| t.as_str()), node.get("bbox")) {
        let get = |k: &str| bbox.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        if ty.contains("Image") || ty.contains("Picture") {
            out.push((ty.to_string(), get("y"), get("h")));
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect_image_tops(child, out);
    }
}

#[test]
fn captioned_tac_picture_aligns_its_whole_box_to_the_baseline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let json = document
        .get_page_render_tree(4)
        .expect("render tree page 5");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut images = Vec::new();
    collect_image_tops(&tree, &mut images);
    assert!(
        !images.is_empty(),
        "5쪽에 그림 노드가 있어야 한다 — 렌더 트리 타입 이름이 바뀌었는지 확인하라"
    );

    let first_y = images
        .iter()
        .map(|(_, y, _)| *y)
        .fold(f64::INFINITY, f64::min);

    assert!(
        first_y < MAX_FIRST_PICTURE_Y,
        "5쪽 첫 그림이 y={first_y:.1}px 에 있다 — #6575 회귀. \
         캡션(Bottom, 문단 3개)이 붙은 TAC 그림을 그림 높이만으로 baseline 에 \
         바닥맞춤하면 캡션 몫 35.4px 만큼 내려가 268.7px 이 된다 \
         (허용 상한 {MAX_FIRST_PICTURE_Y:.1}px, 한/글 2024 실측 ≈233.3px)"
    );
}

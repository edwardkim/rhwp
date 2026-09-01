//! Issue #6551: TAC 글상자를 품은 host 문단의 **항목 전진량**이 `layout_paragraph` 가
//! 이미 적용한 문단 앞 간격(`spacing_before`)을 되돌리던 결함의 가드.
//!
//! `layout_column_item` 의 "TAC Shape 높이 보정" 블록은 개체가 글줄보다 클 때 흐름
//! 하단을 개체 바닥까지 내리는 장치다. 그런데 그 바닥을 `para_start + line_height` 로
//! 잡았다 — `para_start` 는 **`sb` 이전** 위치이고 개체는 `sb` 아래에서 시작하므로,
//! 이 보정이 `layout_paragraph` 의 반환값을 덮으면서 `sb` 를 버렸다.
//!
//! ⚠ 단 상단 문단은 한컴이 `sb` 를 트림하므로(`is_column_top`) 제외한다 — 넣으면 같은
//! 문서 8쪽 `1. 개요` 가 126.82 → 115.82pt 로 11.03pt 어긋난다.
//! ⚠ 줄간격(`ls`)은 더하지 않는다 — 다음 문단이 자기 `sb` 를 따로 적용하므로 이중
//! 계상이고, `issue_1116 sample16` 3쪽 제목이 337.2 → 347.63pt 로 밀린다.
//!
//! 재현 문서 `samples/issue6551/113424_evaluation_guideline.hwpx` 7쪽 `pi=73`
//! (한글 2024 실측 — COM 이 이 버전으로만 해석된다):
//!
//! ```text
//! 조판(dump-pages)  h=68.5 = sb 13.3 + lh 43.5 + ls 11.7
//! 수정 전 렌더       dy=43.5   (= 저장 lh 3261HU 그 자체)
//! 수정 후 렌더       dy=56.8   (= sb 13.3 + lh 43.5; 남은 ls 는 다음 문단 몫)
//! ```
//!
//! 그 결과 다음 제목 `1. 목 적` 이 글상자(`Ⅰ 총 칙`) **안으로** 9.97pt 올라와 붙었다.
//! 한/글은 상자 바닥(449.4pt) 아래 458.41pt 에 둔다.
//!
//! PDF 실측(한글 2024 오라클 대조, 7쪽 매칭 텍스트 편차):
//!
//! ```text
//! 7쪽 max 10.05pt -> 2.99pt      '1. 목 적' 448.44 -> 458.44pt (한/글 458.41)
//! 46쪽 전체 median 합 126.8 불변 — 부수 이동 없음
//! ```

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6551/113424_evaluation_guideline.hwpx";

/// 글상자 아래 첫 제목의 한글 2024 기준 y(px)는 611.2px이다.
/// 종전 회귀값 598.1px과 구분하면서 미세 렌더 편차만 허용한다.
const HEADING_Y_MIN: f64 = 610.0;
const HEADING_Y_MAX: f64 = 612.0;

/// 단 상단 문단은 한글이 `spacing_before`를 트림하므로, 별도 절대 위치를 고정한다.
const COLUMN_TOP_HEADING_Y: f64 = 169.3;
const COLUMN_TOP_TOLERANCE_PX: f64 = 1.0;

fn collect_texts(node: &serde_json::Value, out: &mut Vec<(String, f64)>) {
    if let (Some(text), Some(bbox)) = (node.get("text").and_then(|t| t.as_str()), node.get("bbox"))
    {
        if let Some(y) = bbox.get("y").and_then(|y| y.as_f64()) {
            out.push((text.to_string(), y));
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect_texts(child, out);
    }
}

#[test]
fn tac_shape_height_correction_keeps_paragraph_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let json = document
        .get_page_render_tree(6)
        .expect("render tree page 7");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut texts = Vec::new();
    collect_texts(&tree, &mut texts);

    // 렌더 트리는 이 제목을 `"1. "` 과 `"목 적  "` 두 노드로 나눈다.
    let heading_y = texts
        .iter()
        .find(|(text, _)| text.replace(' ', "") == "목적")
        .map(|(_, y)| *y)
        .expect("7쪽에 '목 적' 제목 노드가 있어야 한다");

    assert!(
        (HEADING_Y_MIN..=HEADING_Y_MAX).contains(&heading_y),
        "'1. 목 적' 이 y={heading_y:.1}px 에 있다 — #6551 회귀. \
         TAC Shape 높이 보정이 개체 바닥을 `para_start + lh` 로 잡으면 문단 앞 간격 13.3px 을 \
         버려 제목이 글상자 안으로 올라온다 \
         (허용 범위 {HEADING_Y_MIN:.1}~{HEADING_Y_MAX:.1}px, 한글 2024 기준 611.4px)"
    );

    let page8_json = document
        .get_page_render_tree(7)
        .expect("render tree page 8");
    let page8_tree: serde_json::Value =
        serde_json::from_str(&page8_json).expect("parse page 8 render tree json");
    let mut page8_texts = Vec::new();
    collect_texts(&page8_tree, &mut page8_texts);
    let overview_y = page8_texts
        .iter()
        .find(|(text, _)| text.replace(' ', "") == "개요")
        .map(|(_, y)| *y)
        .expect("8쪽에 '개요' 제목 노드가 있어야 한다");
    assert!(
        (overview_y - COLUMN_TOP_HEADING_Y).abs() <= COLUMN_TOP_TOLERANCE_PX,
        "8쪽 단 상단 '개요'가 y={overview_y:.1}px 에 있다 — #6551 회귀. \
         단 상단 문단은 `spacing_before`를 트림해야 한다 \
         (기준 {COLUMN_TOP_HEADING_Y:.1}px, 허용 오차 {COLUMN_TOP_TOLERANCE_PX:.1}px)"
    );
}

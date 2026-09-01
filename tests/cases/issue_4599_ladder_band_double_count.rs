//! Issue #4599: 저장 사다리의 줄-간 델타에 **이미 따로 그린 자리차지 밴드**의 공간이
//! 들어 있는데, 문단이 그 델타를 자기 진행량으로 다시 써서 밴드를 두 번 세던 결함의 가드.
//!
//! `inline_table_stored_line_top_offset_px`(#6181)는 인라인 표를 품은 문단의 줄 상단을
//! 첫 줄 대비 저장 `vertical_pos` 델타에 놓는다. #6181 의 표본에서는 그 델타가 문단
//! 자신의 줄 진행량 합과 **정확히** 일치한다(`vertsize 1778 + spacing 976 = 2754 = 델타`).
//!
//! 그런데 앞 문단이 소유한 자리차지 표의 밴드가 이 문단의 두 줄 사이에 놓이면, 델타는
//! 그 밴드까지 품는다. 밴드는 이미 자기 `Table` 항목으로 그려졌으므로 그대로 쓰면
//! 이중 계상이다.
//!
//! 재현 문서 `samples/issue4599/36374873_night_guard_log.hwpx` (1쪽, 한글 2024 실측):
//!
//! ```text
//! LADDER pi=3 ctrls=[Table13x8, Table3x4] segs=[6463/7012/7012/100]
//! LADDER pi=4 ctrls=[Table1x2]            segs=[13575/2414/2414/600  67877/1000/1000/600]
//!
//! pi=4 자신의 진행량 합 = 2414 + 600 = 3014 HU
//! pi=4 사다리 델타       = 67877 - 13575 = 54302 HU   ← 18배
//! 초과 51288 HU (683.8px) = pi=3 소유 13x8 자리차지 표의 밴드
//!                           (그 표는 이미 Table 항목으로 726.7px 소비)
//! ```
//!
//! 수정 전 `FullPara pi=4` 는 745.4px 를 전진했다(조판이 준 높이는 61.5px). 그 결과
//! 뒤따르는 `붙임:` 문단이 y=1839.7 로 용지(1123px) 밖으로 나가 소실됐다.
//! 수정 후 pi=4 전진 49.7px, `붙임:` y=1144.1 이다.
//!
//! ⚠ 이 문서에는 같은 쪽에 **별개 축**의 이중 계상이 하나 더 있다 — `pi=3` 의 저장 줄
//! 하나(7012HU)를 인라인 3x4 TAC 표(89.7px)와 `PartialPara`(94.8px)가 나눠 두 번
//! 소비한다. 그쪽은 표와 글자가 한 줄을 공유하는 #6298 / #6167 계열이라 여기서 닫지
//! 않는다. 그래서 `붙임:` 은 아직 저장 사다리 값(1023.3)까지 내려오지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue4599/36374873_night_guard_log.hwpx";

/// 밴드를 두 번 세면 `붙임:` 은 1839.7 로 간다. 저장 사다리 정답은 1023.3,
/// 남은 별개 축(#6298 계열) 때문에 현재는 1144.1 이다. 그 사이에 가드를 둔다.
const MAX_ATTACHMENT_Y: f64 = 1400.0;

fn collect_text_tops(node: &serde_json::Value, out: &mut Vec<(String, f64)>) {
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
        collect_text_tops(child, out);
    }
}

#[test]
fn stored_ladder_delta_does_not_recharge_an_already_painted_float_band() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let json = document
        .get_page_render_tree(0)
        .expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut tops = Vec::new();
    collect_text_tops(&tree, &mut tops);

    let attachment_y = tops
        .iter()
        .find(|(text, _)| text.starts_with("붙임"))
        .map(|(_, y)| *y)
        .expect("'붙임' 으로 시작하는 꼬리 문단이 렌더 트리에 있어야 한다");

    assert!(
        attachment_y < MAX_ATTACHMENT_Y,
        "꼬리 문단 '붙임:' 이 y={attachment_y:.1} 에 있다 — #4599 회귀. \
         pi=4 의 사다리 델타(54302HU)에 pi=3 소유 13x8 자리차지 표의 밴드가 들어 있는데 \
         그 밴드를 흐름 전진으로 다시 써서 683.8px 를 이중 계상하면 1839.7 로 간다 \
         (허용 상한 {MAX_ATTACHMENT_Y:.1})"
    );
}

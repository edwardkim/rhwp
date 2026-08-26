//! Issue #6030: 내용이 선언 행 높이를 반 줄 미만으로 넘을 때 행을 안 늘려
//! 셀 마지막 선택지 줄이 아래 괘선에 깎인다.
//!
//! 표적 문서(2386771 심사서식)는 코퍼스 파일이라 저장소에 없다. 같은 형상 —
//! TAC 표가 내용+여백 합을 선언 높이로 균일 축소해 마지막 글줄이 clip 되는
//! 경우 — 를 `samples/exam_eng.hwp` 6쪽 선택지 표로 고정한다.
//!
//! `pi=257` 5×13 TAC 표 r0c0 `①`:
//! - 선언 `h=1191 HU` = 15.88px, 줄 `lh=1148 HU` = 15.31px, 상하 여백 `141+141`.
//! - 필요 높이 = 15.31+3.76 = 19.07px. 한글은 이만큼 행을 키운다.
//! - 수정 전: TAC 2%~150% 비례 축소 폴백이 하한 아래까지 눌러 셀 clip 15.88px.
//!   선택지 `①` 글줄 바닥이 clip 을 1.3px 넘는다.
//! - 수정 후: 하한 합이 선언을 넘으면 축소하지 않고 내용+여백으로 유지.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/exam_eng.hwp";
const PAGE: u32 = 5; // 6쪽 (0-based)

fn find_choice_cell<'a>(node: &'a serde_json::Value, mark: &str) -> Option<&'a serde_json::Value> {
    if node.get("type").and_then(|t| t.as_str()) == Some("Cell") {
        let mut found = false;
        fn has_text(n: &serde_json::Value, mark: &str, found: &mut bool) {
            if n.get("type").and_then(|t| t.as_str()) == Some("TextRun")
                && n.get("text").and_then(|t| t.as_str()) == Some(mark)
            {
                *found = true;
            }
            if let Some(children) = n.get("children").and_then(|c| c.as_array()) {
                for c in children {
                    has_text(c, mark, found);
                }
            }
        }
        has_text(node, mark, &mut found);
        if found {
            return Some(node);
        }
    }
    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for c in children {
            if let Some(hit) = find_choice_cell(c, mark) {
                return Some(hit);
            }
        }
    }
    None
}

fn last_textline_bottom(node: &serde_json::Value) -> Option<f64> {
    let mut best = None;
    fn walk(n: &serde_json::Value, best: &mut Option<f64>) {
        if n.get("type").and_then(|t| t.as_str()) == Some("TextLine") {
            if let (Some(y), Some(h)) = (
                n.get("bbox")
                    .and_then(|b| b.get("y"))
                    .and_then(|v| v.as_f64()),
                n.get("bbox")
                    .and_then(|b| b.get("h"))
                    .and_then(|v| v.as_f64()),
            ) {
                let bot = y + h;
                *best = Some(best.map_or(bot, |cur: f64| cur.max(bot)));
            }
        }
        if let Some(children) = n.get("children").and_then(|c| c.as_array()) {
            for c in children {
                walk(c, best);
            }
        }
    }
    walk(node, &mut best);
    best
}

#[test]
fn issue_6030_exam_eng_choice_last_line_not_clipped() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    let json = doc.get_page_render_tree(PAGE).expect("render tree page 6");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let cell = find_choice_cell(&tree, "①").expect("6쪽 선택지 ① 셀");
    let bbox = cell.get("bbox").expect("cell bbox");
    let y = bbox.get("y").and_then(|v| v.as_f64()).expect("y");
    let h = bbox.get("h").and_then(|v| v.as_f64()).expect("h");
    let clip_bot = y + h;
    let line_bot = last_textline_bottom(cell).expect("① 글줄");

    assert!(
        h >= 18.5,
        "① 셀 높이는 내용+여백(≈19.07px)이어야 한다 — 선언 15.88px 로 눌렸다면 \
         TAC 균일 축소 폴백이 하한을 무시해 마지막 줄을 깎은 것이다, got {h}"
    );
    assert!(
        line_bot <= clip_bot + 0.4,
        "① 글줄 바닥({line_bot:.2})이 셀 clip({clip_bot:.2}) 안에 있어야 한다 \
         — 수정 전 over≈1.3px"
    );
}

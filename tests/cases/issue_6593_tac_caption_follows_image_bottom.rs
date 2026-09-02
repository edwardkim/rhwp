//! Issue #6593: 캡션 붙은 글자처럼(TAC) 그림의 아래 캡션이 그림 바닥이 아니라
//! `그림 상단 + baseline` 에 붙어, 저장 줄을 넘고 뒤 내용을 18pt 밀던 결함의 가드.
//!
//! `layout_shape_item` 은 캡션 y 를 `image_bottom + spacing` 으로 잡는데, 종전에는
//! `image_bottom = 그림 상단 + max(baseline, 그림 높이)` 였다. 저장 줄이 그림 + 캡션을
//! 통째로 예약한 줄은 baseline 이 그림 높이보다 커서, 캡션이 `baseline − 그림 높이`
//! 만큼 내려가 줄 바닥을 넘고 `result_y` 가 캡션 끝으로 밀린다.
//!
//! 재현 문서 `samples/issue6575/156489219_satellite_pm_release.hwp` 5쪽
//! (한글 2020 PDF `pdf/156489219_satellite_pm_release-2020.pdf` 대조, px = pt × 96/72):
//!
//! ```text
//! 그림 pi=43   y=233.7  h=205.7  baseline=240.7  raw_lh=283.1  caption=Bottom(spacing 850HU)
//!
//!                     수정 전     수정 후    한/글 2020
//! 캡션 첫 줄          492.3       457.3      458.3
//! 다음 표 pi=44       553.7       529.1      529.9
//! ```
//!
//! 수정 전 캡션 블록 끝(551.8)이 줄 바닥(516.8)을 35px 넘겨 표가 24.6px 내려갔다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6575/156489219_satellite_pm_release.hwp";
/// 쪽 상단 제목표에도 같은 문구가 있어 꺾쇠까지 포함해 캡션만 고른다.
const CAPTION_TEXT: &str = "< 환경위성센터 누리집 주요 화면 >";
const TABLE_PARA_INDEX: u64 = 44;

/// 한/글 2020 실측 458.3px. 결함 시 492.3px (baseline − 그림 높이 = +35.0).
const EXPECTED_CAPTION_Y: f64 = 457.3;
/// 한/글 2020 실측 529.9px. 결함 시 553.7px (캡션 넘침 +24.6).
const EXPECTED_TABLE_Y: f64 = 529.1;
const TOLERANCE_PX: f64 = 1.5;

#[derive(Default)]
struct Found {
    caption_ys: Vec<(String, f64)>,
    table_y: Option<f64>,
}

fn walk(node: &serde_json::Value, found: &mut Found) {
    let ty = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let y = node
        .get("bbox")
        .and_then(|b| b.get("y"))
        .and_then(|v| v.as_f64());
    match ty {
        "TextRun" => {
            let text = node.get("text").and_then(|t| t.as_str()).unwrap_or("");
            if text.contains(CAPTION_TEXT) {
                found
                    .caption_ys
                    .push((text.to_string(), y.unwrap_or(f64::NAN)));
            }
        }
        "Table" => {
            if node.get("pi").and_then(|v| v.as_u64()) == Some(TABLE_PARA_INDEX) {
                assert!(
                    found.table_y.is_none(),
                    "표 pi={TABLE_PARA_INDEX} 가 5쪽에 두 번 나온다"
                );
                found.table_y = y;
            }
        }
        _ => {}
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        walk(child, found);
    }
}

#[test]
fn bottom_caption_of_tac_picture_starts_right_below_the_drawn_picture() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    let json = document
        .get_page_render_tree(4)
        .expect("render tree page 5");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut found = Found::default();
    walk(&tree, &mut found);

    assert_eq!(
        found.caption_ys.len(),
        1,
        "캡션 글줄 `{CAPTION_TEXT}` 은 5쪽에 한 번 있어야 한다: {:?}",
        found.caption_ys
    );
    let caption_y = found.caption_ys[0].1;
    assert!(
        (caption_y - EXPECTED_CAPTION_Y).abs() < TOLERANCE_PX,
        "캡션 첫 줄 y={caption_y:.1}px — 그림 바닥(439.4) + 간격 아래 {EXPECTED_CAPTION_Y:.1} 이어야 한다. \
         결함 시 492.3 (그림 상단 + baseline 기준, +35.0px)"
    );

    let table_y = found.table_y.expect("5쪽에 표 pi=44 가 있어야 한다");
    assert!(
        (table_y - EXPECTED_TABLE_Y).abs() < TOLERANCE_PX,
        "표 pi=44 y={table_y:.1}px — 저장 줄 바닥 + gap + outer margin 인 {EXPECTED_TABLE_Y:.1} 이어야 한다. \
         결함 시 553.7 (넘친 캡션 끝을 따라 +24.6px)"
    );
}

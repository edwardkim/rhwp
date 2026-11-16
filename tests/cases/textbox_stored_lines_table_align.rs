//! 저장 줄이 있는 세로 가운데 정렬 글상자는 끝 문단의 비글자처럼 표(글 앞으로·자리차지)까지
//! 바깥 여백을 넣어 내용 높이를 잡고, 자리차지 표도 자기 세로 오프셋 자리에 놓는다.
//!
//! 재현물 `samples/textbox_stored_table/textbox_stored_{top,center}_{none,front_v*,tb_v*}.{hwpx,hwp}`:
//! 공개 샘플 `samples/issue8513/textbox_floating_table_offset.hwpx` 의 글상자(높이 30000 HU) 안을
//! `T1`~`T4` + 표 문단(빈 문단 + 표 5669 HU, 바깥 여백 283 HU, 세로 오프셋 0·2000 HU)으로 바꾸고 한컴
//! 저장 형식의 줄 정보(flags 0x60000, 줄 피치 1600)를 채운 합성본이다. HWP 는 같은 HWPX 를 rhwp 로
//! 변환한 것으로 저장 줄이 한컴 저장 표지 그대로 남는다.
//!
//! 기준: `pdf/textbox_stored_table/<이름>-webhwp-print.pdf`, `<이름>-hwp5-webhwp-print.pdf`(웹한글기안기
//! 인쇄). 96dpi px 실측:
//! - 가운데 정렬 `T1` 이 표 없는 대조군보다 위로: 오프셋 0 에서 34.9, 2000 에서 48.0(글 앞으로·자리차지 같음)
//!   = (표 바닥 + 바깥 아래 여백 − 끝 줄 바닥) / 2
//! - 위 정렬 표 칸 글자는 `T1` 아래 120.3(오프셋 0)·146.9(2000) — 자리차지 표도 오프셋을 따른다
//!
//! - 띠가 앵커 줄을 민 판(`_pushed`: 앵커 줄을 앞 문단 끝 + 띠 높이에 저장, 한컴 저장본과 같은 관계)도
//!   표는 앞 문단 끝 + 오프셋에 놓인다: 위 정렬 `T1`→표 칸 글자 77.8(오프셋 0)·104.3(2000)
//!   (웹한글기안기는 뒤 줄을 다시 짜므로 `A1` 자리는 비교하지 않는다)
//!
//! 수정 전 rhwp 는 가운데 정렬 높이에서 표를 빼(그림·도형만 넣음) `T1` 을 대조군 자리에 두었고,
//! 저장 줄 있는 글상자의 자리차지 표는 세로 오프셋과 바깥 위 여백을 버렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn first_y(node: &RenderNode, text: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.trim() == text {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| first_y(child, text))
}

fn ys(name: &str) -> (f64, Option<f64>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/textbox_stored_table")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "{name}: 한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    (
        first_y(&tree.root, "T1").expect("T1 줄"),
        first_y(&tree.root, "BOX"),
    )
}

fn assert_near(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.5,
        "{what}: {actual:.1}px, 웹한글기안기 인쇄 {expected:.1}px"
    );
}

const EXTS: [&str; 2] = ["hwpx", "hwp"];

#[test]
fn centered_stored_textbox_counts_trailing_floating_table() {
    for ext in EXTS {
        let (none, _) = ys(&format!("textbox_stored_center_none.{ext}"));
        for kind in ["front", "tb"] {
            for (voff, rise) in [(0, 34.9), (2000, 48.0)] {
                let name = format!("textbox_stored_center_{kind}_v{voff}.{ext}");
                let (t1, _) = ys(&name);
                assert_near(&format!("{name} T1 이 대조군보다 위로"), none - t1, rise);
            }
        }
    }
}

#[test]
fn stored_textbox_topbottom_table_follows_vertical_offset() {
    for ext in EXTS {
        for kind in ["front", "tb"] {
            for (voff, below) in [(0, 120.3), (2000, 146.9)] {
                let name = format!("textbox_stored_top_{kind}_v{voff}.{ext}");
                let (t1, cell) = ys(&name);
                let cell = cell.expect("표 칸 글자");
                assert_near(&format!("{name} T1→칸 글자"), cell - t1, below);
            }
        }
    }
}

#[test]
fn pushed_anchor_line_places_topbottom_table_at_previous_paragraph_end() {
    for ext in EXTS {
        for (voff, below) in [(0, 77.8), (2000, 104.3)] {
            let name = format!("textbox_stored_top_tb_v{voff}_pushed.{ext}");
            let (t1, cell) = ys(&name);
            let cell = cell.expect("표 칸 글자");
            assert_near(&format!("{name} T1→칸 글자"), cell - t1, below);
        }
    }
}

//! 저장 줄 없는 글상자의 빈 앵커 문단에 놓인 문단 기준 자리차지(위아래) 표는 글상자 흐름을 차지한다.
//!
//! 재현물 `samples/textbox_tb_table/textbox_tb_table_{top,center}_v{0,2000}.{hwpx,hwp}`:
//! 공개 샘플 `samples/issue8513/textbox_floating_table_offset.hwpx` 의 글상자(높이 40000 HU) 안을
//! `T1`, `T2`, 빈 문단 + 자리차지 표(높이 5669 HU, 바깥 여백 283 HU, 세로 오프셋 0·2000 HU), `A1`, `A2`
//! 로 바꾸고 글상자 세로 정렬(위·가운데)을 바꾼 합성본이다. HWP 는 같은 HWPX 를 rhwp 로 변환한 것으로
//! 글상자 문단에 저장 줄이 없다(실문서와 같은 경로).
//!
//! 기준: `pdf/textbox_tb_table/<이름>-webhwp-print.pdf`, `<이름>-hwp5-webhwp-print.pdf`(웹한글기안기 인쇄,
//! 화면 캡처와 0.5pt 안에서 같다). 단어 위끝 실측(96dpi px):
//! - 위 정렬: `T1`→`A1` 거리는 오프셋 0 에서 125.8, 2000 에서 152.7(오프셋 20pt 만큼 더 밀림)
//! - 가운데 정렬: `T1` 이 위 정렬보다 오프셋 0 에서 182.7, 2000 에서 169.3 아래(띠가 내용 높이에 듦)
//!
//!
//! 표 문단에 글 `ANCHOR` 가 있는 4벌(`_text`)은 띠에 걸리는 앵커 줄만 띠 아래로 간다:
//! - 오프셋 0: 표가 문단 위끝에 놓이고 `ANCHOR` 가 띠 아래(빈 앵커의 `A1` 자리), `A1` 은 그 다음 줄
//!   (`T1`→`ANCHOR` 125.8, `T1`→`A1` 147.2)
//! - 오프셋 2000: 띠가 앵커 줄 아래에서 시작해 `ANCHOR` 는 제자리(`T1`→`ANCHOR` 42.9),
//!   `A1` 은 띠 아래(`T1`→`A1` 152.7, 빈 앵커와 같음)
//!
//! 수정 전 rhwp 는 표를 흐름에 넣지 않아 `A1` 이 표 문단 바로 다음 줄에 놓이고(표와 겹침),
//! 세로 오프셋을 버렸으며, 가운데 정렬 높이에서도 표를 뺐다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/textbox_tb_table")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

#[derive(Default)]
struct Found {
    t1: Option<f64>,
    anchor: Option<f64>,
    a1: Option<f64>,
    table: Option<BoundingBox>,
}

fn collect(node: &RenderNode, found: &mut Found) {
    match &node.node_type {
        RenderNodeType::TextRun(run) => match run.text.trim() {
            "T1" if found.t1.is_none() => found.t1 = Some(node.bbox.y),
            "A1" if found.a1.is_none() => found.a1 = Some(node.bbox.y),
            "ANCHOR" if found.anchor.is_none() => found.anchor = Some(node.bbox.y),
            _ => {}
        },
        RenderNodeType::Table(_) if found.table.is_none() => {
            found.table = Some(node.bbox.clone());
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, found);
    }
}

fn positions(name: &str) -> (f64, f64, BoundingBox) {
    let core = load(name);
    assert_eq!(core.page_count(), 1, "{name}: 한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut found = Found::default();
    collect(&tree.root, &mut found);
    (
        found.t1.expect("T1 줄"),
        found.a1.expect("A1 줄"),
        found.table.expect("글상자 안 표"),
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
fn top_aligned_textbox_flows_below_topbottom_table() {
    for ext in EXTS {
        for (voff, t1_to_a1) in [(0, 125.8), (2000, 152.7)] {
            let name = format!("textbox_tb_table_top_v{voff}.{ext}");
            let (t1, a1, table) = positions(&name);
            assert_near(&format!("{name} T1→A1"), a1 - t1, t1_to_a1);
            assert!(
                table.y + table.height <= a1 + 0.5,
                "{name}: A1({a1:.1}) 이 표 바닥({:.1}) 아래여야 한다",
                table.y + table.height
            );
        }
    }
}

#[test]
fn vertical_offset_moves_table_and_following_flow() {
    for ext in EXTS {
        let (_, a0, t0) = positions(&format!("textbox_tb_table_top_v0.{ext}"));
        let (_, a2, t2) = positions(&format!("textbox_tb_table_top_v2000.{ext}"));
        // 2000 HU = 26.67px
        assert_near(&format!("{ext} 표 위 이동"), t2.y - t0.y, 26.67);
        assert_near(&format!("{ext} A1 이동"), a2 - a0, 26.9);
    }
}

#[test]
fn centered_textbox_counts_topbottom_table_band() {
    for ext in EXTS {
        for (voff, shift) in [(0, 182.7), (2000, 169.3)] {
            let (top_t1, _, _) = positions(&format!("textbox_tb_table_top_v{voff}.{ext}"));
            let (center_t1, _, _) = positions(&format!("textbox_tb_table_center_v{voff}.{ext}"));
            assert_near(
                &format!("{ext} v{voff} 가운데 정렬 T1 이동"),
                center_t1 - top_t1,
                shift,
            );
        }
    }
}

#[test]
fn text_anchor_lines_overlapping_band_move_below_table() {
    for ext in EXTS {
        for (voff, t1_to_anchor, t1_to_a1) in [(0, 125.8, 147.2), (2000, 42.9, 152.7)] {
            let name = format!("textbox_tb_table_top_v{voff}_text.{ext}");
            let core = load(&name);
            assert_eq!(core.page_count(), 1, "{name}: 한 쪽");
            let tree = core.build_page_render_tree(0).unwrap();
            let mut found = Found::default();
            collect(&tree.root, &mut found);
            let (t1, anchor, a1) = (
                found.t1.expect("T1 줄"),
                found.anchor.expect("ANCHOR 줄"),
                found.a1.expect("A1 줄"),
            );
            let table = found.table.expect("글상자 안 표");
            assert_near(&format!("{name} T1→ANCHOR"), anchor - t1, t1_to_anchor);
            assert_near(&format!("{name} T1→A1"), a1 - t1, t1_to_a1);
            let table_bottom = table.y + table.height;
            assert!(
                anchor + 0.5 >= table_bottom || anchor + 21.0 <= table.y + 0.5,
                "{name}: ANCHOR({anchor:.1}) 가 표({:.1}~{table_bottom:.1})와 겹치면 안 된다",
                table.y
            );
        }
        // 가운데 정렬: 오프셋 0 은 띠 + 앵커 줄, 2000 은 앵커 줄 + 띠가 내용 높이에 든다.
        for (voff, shift) in [(0, 172.0), (2000, 169.3)] {
            let (top_t1, _, _) = positions(&format!("textbox_tb_table_top_v{voff}_text.{ext}"));
            let (center_t1, _, _) =
                positions(&format!("textbox_tb_table_center_v{voff}_text.{ext}"));
            assert_near(
                &format!("{ext} v{voff} 글 앵커 가운데 정렬 T1 이동"),
                center_t1 - top_t1,
                shift,
            );
        }
    }
}

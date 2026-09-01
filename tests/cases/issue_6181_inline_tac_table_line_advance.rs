//! Issue #6181: 글자처럼 취급되는 표를 **줄 안에** 품은 문단의 둘째 줄이 저장
//! `vertpos` 대신 **표 하단**에 붙어, 앞 줄의 `vertsize` 초과분과 `spacing` 을 함께
//! 버린다.
//!
//! `layout_inline_table_paragraph` 의 첫 줄바꿈은 `current_y = max_table_bottom` 으로
//! 갔다. 표 폭 때문에 글이 표 아래로 밀리는 형상에는 맞지만, `신규` 배지처럼 표가 줄
//! **안**에 들어가는 문단에서는 줄이 바짝 붙는다.
//!
//! 실측(156562368 인쇄 4쪽 para#114, `paraPr 39` = `lineSpacing PERCENT 120`):
//!
//! ```text
//! ls[0] vertpos=24112 vertsize=1778 baseline=1511 spacing=976
//! ls[1] vertpos=26866 vertsize=1500 baseline=1275 spacing=976
//!
//! 저장 줄 간격  2754 HU = 36.72px  ( = 앞 줄 vertsize 1778 + spacing 976 )
//! rhwp 종전     20.00px            ( = 표 하단 = 뒤 줄 vertsize 1500 만 )
//! ```
//!
//! 13.6px(40%)이 사라져 인쇄 4~9쪽에서 압축 줄이 25개 나왔다 — 한/글은 같은 여섯 쪽에서
//! 0개다. 줄 수가 양쪽 같으므로 되감김 차이가 아니라 순수한 줄 간격 결함이다.
//!
//! 쪽수는 12/12 로 한/글과 같고 넘침·겹침 지표도 전후가 같아 게이트가 전부 침묵한다.
//! 그래서 좌표를 직접 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6181/156562368_inline_tac_table_line_advance.hwpx";
const PAGE_INDEX: u32 = 4; // 0-based — 인쇄 4쪽(물리 5쪽)

/// 저장 `vertpos` 델타 2754 HU = 36.72px @96dpi. 회귀 시에는 20.00px 였다.
const STORED_ADVANCE_PX: f64 = 36.72;
const TOLERANCE_PX: f64 = 1.5;

/// `ㅇ 신규(기록강화) …` 와 `ㅇ 신규(기재 일원화) …` — 둘 다 첫 줄에 1×1 TAC 표(`신규`
/// 배지)를 품고 둘째 줄로 넘어간다.
const PARAS: [usize; 2] = [64, 67];

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 이 문단이 만든 글줄들의 상단 y — 6px 안쪽은 같은 줄로 묶는다.
fn line_tops(nodes: &[&RenderNode], para_index: usize) -> Vec<f64> {
    let mut ys: Vec<f64> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(para_index) && !run.text.trim().is_empty() =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .collect();
    ys.sort_by(f64::total_cmp);
    let mut grouped: Vec<f64> = Vec::new();
    for y in ys {
        if grouped.last().map(|&last| y - last > 6.0).unwrap_or(true) {
            grouped.push(y);
        }
    }
    grouped
}

#[test]
fn issue_6181_inline_tac_table_line_uses_stored_advance() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6181 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p5");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    for para_index in PARAS {
        let tops = line_tops(&nodes, para_index);
        assert!(
            tops.len() >= 2,
            "pi={para_index} 은 두 줄이어야 한다 — 실측 {}줄 {tops:?}",
            tops.len()
        );
        let advance = tops[1] - tops[0];
        assert!(
            (advance - STORED_ADVANCE_PX).abs() <= TOLERANCE_PX,
            "pi={para_index} 둘째 줄이 저장 vertpos 가 아니라 표 하단에 붙었다 — \
             실측 {advance:.2}px, 저장 {STORED_ADVANCE_PX:.2}px (회귀 시 20.00px)"
        );
    }
}

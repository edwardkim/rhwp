//! [#7330] 공백만 있는 **단일** host 줄은 "표 뒤 본문"으로 나가지 않는다.
//!
//! ## 종전 결함
//!
//! 한 문단에 표가 둘이고 마지막 표가 비-TAC 이면 `post_table_start` 가
//! `is_last_table && !is_first_table` 갈래로 **0** 이 된다. 그러면 host 문단의 줄 전체가
//! `PartialParagraph { start_line: 0, … }` 로 **표 뒤에** 실려 나간다. 그 줄은 저장 `vpos`
//! 가 아니라 흐름 꼬리를 받으므로 자리차지 표 **아래**에 그려지고, 그 높이만큼 뒤 문단이
//! 밀려 용지 밖으로 나간다.
//!
//! 기존 가드 `whitespace_only_single_tac_host_line` 이 그 자리를 노렸지만
//! `treat_as_char && pre_table_end_line == 0` 으로 좁혀 있어 이 형상(마지막 표가 비-TAC,
//! `pre_table_end_line == 1`)을 못 잡았다.
//!
//! ## 이 문서 (`samples/issue4599/36374873_night_guard_log.hwpx`, 1쪽)
//!
//! `#4599` 백로그가 이미 들여온 fixture 다 — 새 표본을 더하지 않는다.
//!
//! ```text
//! 문단 0.3  ls[0] vpos=6463(86.2px) lh=7012(93.5px)   텍스트 = 공백 42칸, 줄 1개
//!   [1] 표 3×4  treat_as_char=true                     ← 결재란
//!   [0] 표 13×8 treat_as_char=false  vert=문단(134.4px)
//! ```
//!
//! 실측(base `4653189bb`, 같은 빌드에서 수정만 껐다 켬):
//!
//! | | 수정 전 | 수정 후 |
//! | --- | ---: | ---: |
//! | `pi=3` host 줄 | 978.2 (13×8 표 **뒤**, 표 바닥 976.9) | **251.5** (3×4 표 뒤·13×8 표 **앞**) |
//! | `pi=5` / `pi=6` / `pi=7` | 1125.5 / 1146.8 / 1168.1 | **1030.6 / 1052.0 / 1073.3** |
//!
//! host 줄이 **사라지는 게 아니라** 저장이 정한 자리로 돌아간다. 표 뒤로 밀려 있던
//! 93.5px 짜리 줄이 흐름에서 빠지면서 뒤 문단 셋이 정확히 94.9px 올라온다.
//!
//! 용지는 1122.5px 다 — 수정 전에는 `pi=5..7` 세 줄이 전부 그 밖에 있었다.
//! 한/글 2022 정본은 `pi=6` 을 1022.0 에 둔다(저장 사다리 1023.3). 수정 후 1052.0 은
//! 아직 +30.0px 남고 그 잔여는 `#4599` 의 다른 축이다 — 이 시험은 **용지 밖 이탈 해소**를 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue4599/36374873_night_guard_log.hwpx";
/// 용지 높이 (px).
const PAPER_H: f64 = 1122.5;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn whitespace_host_line_does_not_push_body_off_the_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 1, "#7330: 이 fixture 는 1쪽이다");

    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    // 1. 어떤 글줄도 용지를 넘지 않는다 (수정 전 1146.8 · 1168.1 이 넘었다).
    let worst = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(0.0f64, f64::max);
    assert!(
        worst <= PAPER_H + 0.5,
        "#7330: 글줄이 용지({PAPER_H}) 밖으로 나가면 안 된다 — 수정 전 1181.5: {worst:.1}"
    );

    // 2. 공백뿐인 host 문단(`pi=3`)의 줄이 13×8 표 아래에 그려지지 않는다.
    let big_table_bottom = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.row_count == 13 && t.col_count == 8 => {
                Some(n.bbox.y + n.bbox.height)
            }
            _ => None,
        })
        .next()
        .expect("13×8 자리차지 표");
    let host_line_below_table = nodes.iter().any(|n| match &n.node_type {
        RenderNodeType::TextLine(line) => {
            line.para_index == Some(3) && n.bbox.y > big_table_bottom - 0.5
        }
        _ => false,
    });
    assert!(
        !host_line_below_table,
        "#7330: 공백뿐인 host 줄이 표({big_table_bottom:.1}) 아래에 그려지면 안 된다 \
         — 수정 전 978.2"
    );

    // 3. 마지막 본문 줄이 저장 사다리 쪽으로 붙는다 (수정 전 1146.8 · 사다리 1023.3).
    let last_text = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) if run.text.contains("끝.") => Some(n.bbox.y),
            _ => None,
        })
        .next()
        .expect("마지막 본문 줄");
    assert!(
        last_text < 1100.0,
        "#7330: 마지막 본문 줄은 용지 안에 있어야 한다 — 수정 전 1146.8: {last_text:.1}"
    );
}

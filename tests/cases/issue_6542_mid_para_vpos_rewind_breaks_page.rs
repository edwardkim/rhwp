//! [Issue #6542] 문단 **안**에서 저장 `LINE_SEG` 의 `vertical_pos` 가 되감기면 그 줄부터
//! 다음 쪽이다. 조판이 그 신호를 쓰지 않아 뒷줄들이 본문 하한을 넘어 **쪽번호 아래**에
//! 그려졌다.
//!
//! `156678235`(사망보험금 유동화 보도자료) 물리 6쪽, 구역 0 문단 59 실측:
//!
//! ```text
//! 본문       y=94.5  h=933.6      → 하한 1028.1px
//! 종전       PartialParagraph pi=59 lines=0..3  used=1008.3px   ← +74.7px
//!            넘침 3건  y 1031.7 / 1065.3 / 1098.9  (+3.7 / +37.3 / +70.9)
//! 저장 사다리 vpos 68896 → 5040 으로 line1 에서 되감김
//! 수정 후    PartialParagraph pi=59 lines=0..1  used=941.1px
//! ```
//!
//! 한/글 2022 PDF 대조로도 7쪽 글자의 y 차 중앙값이 **−75.6pt → −25.2pt** 로 줄었다.
//! 쪽수는 7/7 로 전후 불변이다.
//!
//! `dump-pages` 는 이 되감김을 이미 `[vpos-rewind@line1]` 로 **검출해 찍고 있었지만**
//! 그것은 진단 문자열일 뿐 조판 입력이 아니었다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6542/156678235_mid_para_vpos_rewind.hwp";
/// 0-based — 되감김 문단이 놓이는 물리 6쪽.
const PAGE_INDEX: u32 = 5;
/// `body_area y=94.5 + h=933.6`. 한 줄 높이(33.6px)보다 작은 여유만 허용한다.
const BODY_BOTTOM_PX: f64 = 1028.1;
const TOLERANCE_PX: f64 = 8.0;

/// 꼬리말(쪽번호)은 본문 하한 아래에 있는 것이 정상이다 — `Body` 아래만 모은다.
fn collect_body_runs<'a>(node: &'a RenderNode, in_body: bool, out: &mut Vec<&'a RenderNode>) {
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    if in_body {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if !run.text.trim().is_empty() {
                out.push(node);
            }
        }
    }
    for child in &node.children {
        collect_body_runs(child, in_body, out);
    }
}

#[test]
fn issue_6542_stored_rewind_keeps_lines_inside_the_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6542 sample");
    assert_eq!(
        document.page_count(),
        7,
        "쪽수는 한/글과 같은 7쪽이어야 한다"
    );

    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p6");
    let mut nodes = Vec::new();
    collect_body_runs(&tree.root, false, &mut nodes);

    let worst = nodes
        .iter()
        .map(|node| node.bbox.y + node.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        worst.is_finite(),
        "6쪽에 글자가 있어야 한다 — 표본이나 쪽 인덱스가 어긋났다"
    );
    assert!(
        worst <= BODY_BOTTOM_PX + TOLERANCE_PX,
        "6쪽 글자가 본문 하한을 넘었다 — 최하단 {worst:.1}px, 하한 {BODY_BOTTOM_PX:.1}px \
         (회귀 시 1098.9px = +70.9px, 쪽번호 아래에 그려진다)"
    );
}

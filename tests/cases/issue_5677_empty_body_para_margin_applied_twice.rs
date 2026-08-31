//! [#5677] 여백 있는 **빈** 본문 문단이 편집 후 `margin_left` 를 두 번 먹지 않는다.
//!
//! `samples/hwp3-sample.hwp` 의 구역 0 문단 53 이 그 형상이다 — 텍스트도 컨트롤도 없고
//! `ParaShape.margin_left > 0` 이며 `head_type = None`(목록 아님).
//!
//! **경로.** 편집이 걸리면 `reflow_line_segs_impl` 이
//! `column_start = snap_base_left(margin_left)` 을 발행한다(종전에는 0). 그 값이
//! `ComposedLine` 으로 복사되고 `paragraph_layout` 의 `empty_stored_wrap_line` 이
//! 발동해 `effective_col_x = col_area.x + column_start` 가 된다. 그런데 같은 자리의
//! `hwp5_stored_line_start_eligible` 은 `!uses_stored_segment_geometry` 를 요구하므로
//! 거짓이 되어 `margin_left` 가 **또** 더해진다.
//!
//! **차단이 `head_type` 을 키로 잡고 있었다.** `ParagraphBox::body_for_style` 은
//! 목록(`HeadType != None|Outline`)일 때만 원점 발행을 막는데, 그 주석이 스스로 적어
//! 두었듯 **위험은 목록이 아니라 "비어 있음"** 에 있다(`issue_1329_bullet_caret`:
//! 같은 저장 기록인데 빈 줄만 캐럿이 26.6px 어긋난다). `HeadType::None` 인 빈 본문
//! 문단은 차단에서 빠져 있어 이 경로가 열려 있었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::style_resolver::resolve_styles_with_variant;
use rhwp::renderer::DEFAULT_DPI;

/// 이 형상을 가진 문단 (구역 0).
const EMPTY_MARGIN_PARA: usize = 53;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample.hwp");
    let bytes = std::fs::read(path).expect("read hwp3-sample.hwp");
    DocumentCore::from_bytes(&bytes).expect("parse hwp3-sample.hwp")
}

/// 문단이 실제로 "여백 있는 빈 본문 문단"인지 — 전제가 상하면 이 시험은 무의미하다.
fn assert_shape(core: &DocumentCore) -> f64 {
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[EMPTY_MARGIN_PARA];
    assert!(
        para.text.is_empty() && para.controls.is_empty(),
        "문단 {EMPTY_MARGIN_PARA} 이 비어 있어야 한다"
    );
    let styles = resolve_styles_with_variant(&doc.doc_info, DEFAULT_DPI, true);
    let style = styles
        .para_styles
        .get(para.para_shape_id as usize)
        .expect("문단 모양");
    assert!(
        style.margin_left > 0.0,
        "문단 {EMPTY_MARGIN_PARA} 은 왼쪽 여백이 있어야 한다: {}",
        style.margin_left
    );
    assert!(
        matches!(style.head_type, rhwp::model::style::HeadType::None),
        "이 시험은 목록이 아닌 본문 문단을 다룬다"
    );
    style.margin_left
}

/// `pi` 문단이 그린 첫 줄의 좌단과, 그 줄이 속한 단(Column)의 좌단.
fn line_and_column_left(root: &RenderNode, pi: usize) -> Option<(f64, f64)> {
    fn walk(node: &RenderNode, pi: usize, col_x: Option<f64>, out: &mut Option<(f64, f64)>) {
        let col_x = match &node.node_type {
            RenderNodeType::Column(_) => Some(node.bbox.x),
            _ => col_x,
        };
        if out.is_none() {
            if let RenderNodeType::TextLine(line) = &node.node_type {
                if line.para_index == Some(pi) {
                    if let Some(cx) = col_x {
                        *out = Some((node.bbox.x, cx));
                    }
                }
            }
        }
        for child in &node.children {
            walk(child, pi, col_x, out);
        }
    }
    let mut out = None;
    walk(root, pi, None, &mut out);
    out
}

fn find_line_left(core: &DocumentCore, pi: usize) -> (f64, f64) {
    for page in 1..=12u32 {
        let Ok(tree) = core.build_page_render_tree(page) else {
            continue;
        };
        if let Some(found) = line_and_column_left(&tree.root, pi) {
            return found;
        }
    }
    panic!("문단 {pi} 의 줄을 어느 쪽에서도 찾지 못했다");
}

/// 편집으로 재조판이 걸려도 줄 좌단이 `단 좌단 + margin_left` 를 넘지 않는다.
///
/// 종전에는 `col_area.x + margin_left + margin_left` 로 한 번 더 밀렸다.
#[test]
fn empty_body_paragraph_keeps_single_margin_after_edit() {
    let mut core = core();
    let margin_left = assert_shape(&core);
    let (before_x, before_col) = find_line_left(&core, EMPTY_MARGIN_PARA);

    // 편집 → 되돌리기. 문단은 다시 비지만 LINE_SEG 는 재발행된 상태다.
    core.insert_text_native(0, EMPTY_MARGIN_PARA, 0, "X")
        .expect("insert");
    core.delete_text_native(0, EMPTY_MARGIN_PARA, 0, 1)
        .expect("delete");
    assert!(
        core.document().sections[0].paragraphs[EMPTY_MARGIN_PARA]
            .text
            .is_empty(),
        "되돌린 뒤에도 빈 문단이어야 한다"
    );

    let (after_x, after_col) = find_line_left(&core, EMPTY_MARGIN_PARA);
    let before_inset = before_x - before_col;
    let after_inset = after_x - after_col;
    assert!(
        after_inset <= margin_left + 1.0,
        "편집 뒤 줄 좌단이 여백을 한 번만 먹어야 한다: \
         margin_left={margin_left:.2}px, 편집 전 들여쓴 폭={before_inset:.2}px, \
         편집 후={after_inset:.2}px"
    );
}

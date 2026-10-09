//! 입력한 탭은 저장 데이터가 없어 HWP·HWPX 저장본에 자리표(#7170, `tab_ext_is_placeholder`)로
//! 남는다. 다시 연 문서도 이 탭을 문단 탭 정의로 재서, 탭 뒤 글자를 저장 전과 같은 자리에 그려야
//! 한다.
//!
//! 글자 자리(`compute_char_positions`)는 자리표를 걸러 탭 정지까지 나아가지만, run 폭은 탭 전진
//! 없이 잡혔다. 그래서 다음 run 이 탭 뒤 글자 위에 겹쳤다. `가\t나다 ABC` 를 저장했다 열면
//! `ABC` 가 x 199.3 에서 158.9 로 당겨져 `나`(166.7)·`다`(179.7) 위에 그려졌다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const TEXT: &str = "가\t나다 ABC";

#[derive(Clone, Copy, Debug)]
enum Format {
    Hwp,
    Hwpx,
}

fn reopen(doc: &HwpDocument, format: Format) -> HwpDocument {
    let bytes = match format {
        Format::Hwp => doc.export_hwp().expect("HWP 저장"),
        Format::Hwpx => doc.export_hwpx().expect("HWPX 저장"),
    };
    HwpDocument::from_bytes(&bytes).expect("저장본 다시 열기")
}

fn collect_runs(node: &RenderNode, out: &mut Vec<(String, f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.is_empty() {
            out.push((run.text.clone(), node.bbox.x, node.bbox.width));
        }
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 첫 쪽의 글자 run `(글자, x, 폭)`.
fn text_runs(doc: &HwpDocument) -> Vec<(String, f64, f64)> {
    let tree = doc.build_page_render_tree(0).expect("첫 쪽 render tree");
    let mut runs = Vec::new();
    collect_runs(&tree.root, &mut runs);
    runs
}

/// 저장했다 다시 연 문서의 글자 run 이 저장 전과 같은 자리·폭인지 본다.
fn assert_runs_survive_reopen(place: &str, doc: &HwpDocument) {
    let before = text_runs(doc);
    assert!(
        before.iter().any(|(text, _, _)| text == "ABC"),
        "{place} 저장 전 run: {before:?}"
    );
    let mut moved = Vec::new();
    for format in [Format::Hwp, Format::Hwpx] {
        let after = text_runs(&reopen(doc, format));
        assert_eq!(
            after.iter().map(|(text, _, _)| text).collect::<Vec<_>>(),
            before.iter().map(|(text, _, _)| text).collect::<Vec<_>>(),
            "{place} {format:?} 저장본의 run 나눔"
        );
        for ((text, x_before, w_before), (_, x_after, w_after)) in before.iter().zip(&after) {
            if (x_after - x_before).abs() >= 0.01 || (w_after - w_before).abs() >= 0.01 {
                moved.push(format!(
                    "{format:?} {text:?}: x {x_before:.1}·폭 {w_before:.1} → x {x_after:.1}·폭 {w_after:.1}"
                ));
            }
        }
    }
    assert!(
        moved.is_empty(),
        "{place} 저장본의 run 이 저장 전 자리·폭에서 바뀌었다: {moved:#?}"
    );
}

fn blank() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    doc
}

#[test]
fn body_tab_keeps_width_after_reopen() {
    let mut doc = blank();
    doc.insert_text_native(0, 0, 0, TEXT).expect("본문 입력");
    assert_runs_survive_reopen("본문", &doc);
}

#[test]
fn header_tab_keeps_width_after_reopen() {
    let mut doc = blank();
    doc.create_header_footer_native(0, true, 0).expect("머리말");
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, TEXT)
        .expect("머리말 입력");
    assert_runs_survive_reopen("머리말", &doc);
}

#[test]
fn cell_tab_keeps_width_after_reopen() {
    let mut doc = blank();
    doc.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    let (para_idx, control_idx) = doc.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(para_idx, para)| {
            para.controls
                .iter()
                .position(|control| matches!(control, Control::Table(_)))
                .map(|control_idx| (para_idx, control_idx))
        })
        .expect("표 컨트롤");
    doc.insert_text_in_cell_native(0, para_idx, control_idx, 0, 0, 0, TEXT)
        .expect("셀 입력");
    assert_runs_survive_reopen("표 셀", &doc);
}

//! 저장본을 다시 편집해도 원형의 후행 표가 다음 복제 문단 뒤로 넘어가지 않는다.
//! 한컴에서 복사한 독립 입력과 native block API 출력으로 같은 소유 순서를 검사한다.
use rhwp::{
    document_core::{DocumentCore, ParagraphBlockLimits, RepeatParagraphBlockRequest},
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
};

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/rnote")
        .join(name);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("required real fixture")).unwrap()
}

fn collect_tables(node: &RenderNode, in_column: bool, out: &mut Vec<(usize, usize)>) {
    if in_column {
        if let RenderNodeType::Table(t) = &node.node_type {
            if let (Some(pi), Some(ci)) = (t.para_index, t.control_index) {
                if pi >= 12 {
                    out.push((pi, ci));
                }
            }
            return; // 셀 내부의 표는 본문 형제 표 순서에 섞지 않는다.
        }
    }
    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        collect_tables(child, in_column, out);
    }
}

fn assert_owner_order(doc: &DocumentCore, blocks: usize) {
    let mut order = Vec::new();
    for page in 0..doc.page_count() {
        collect_tables(
            &doc.build_page_render_tree(page).unwrap().root,
            false,
            &mut order,
        );
    }
    for pi in 12..12 + blocks {
        for ci in 0..3 {
            assert!(order.contains(&(pi, ci)), "missing ({pi},{ci}): {order:?}");
        }
    }
    assert!(
        order.windows(2).all(|pair| pair[0] <= pair[1]),
        "형제 표/문단 순서 역전: {order:?}"
    );
}

fn grow(doc: &mut DocumentCore, pi: usize) {
    for n in 0..15 {
        doc.split_paragraph_in_cell_native(0, pi, 1, 5, n, 0, None)
            .unwrap();
    }
    let Control::Table(t) = &doc.document().sections[0].paragraphs[pi].controls[1] else {
        panic!("본문 표 누락");
    };
    assert_eq!(t.cells[5].paragraphs.len(), 16);
}

#[test]
fn original_table_growth_preserves_sibling_order() {
    let mut doc = load("labnote-001.hwp");
    grow(&mut doc, 12);
    assert_owner_order(&doc, 1);
}

#[test]
fn hancom_copies_keep_owner_order_after_enter() {
    for (name, blocks) in [("labnote-001-cp-01.hwp", 2), ("labnote-001-cp-02.hwp", 3)] {
        for edited in 12..12 + blocks {
            let mut doc = load(name);
            assert_owner_order(&doc, blocks);
            grow(&mut doc, edited);
            assert_owner_order(&doc, blocks);
        }
    }
}

#[test]
fn native_repeat_save_reopen_keeps_owner_order_after_enter() {
    for count in [1, 2] {
        let mut source = load("labnote-001.hwp");
        source
            .repeat_paragraph_block_native(&RepeatParagraphBlockRequest {
                section_index: 0,
                source_start: 12,
                source_end: 13,
                insert_before: 13,
                count,
                limits: ParagraphBlockLimits::default(),
            })
            .unwrap();
        for bytes in [
            source.export_hwp_native().unwrap(),
            source.export_hwpx_native().unwrap(),
        ] {
            for edited in 12..13 + count {
                let mut doc = DocumentCore::from_bytes(&bytes).unwrap();
                grow(&mut doc, edited);
                assert_owner_order(&doc, 1 + count);
            }
        }
    }
}

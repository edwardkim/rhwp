//! 연속 그림자의 기본 문서·host 공급·실제 paint run을 공유하는 검사 fixture다.
//! 기존 검사 본문을 유지하고 정식 Native CI 모듈과 기본 회귀가 같은 자료를 사용한다.
//! Native 전용 소비자가 feature 없이 수집되는 suite에서는 이 fixture만 미사용일 수 있다.
#![cfg(not(target_arch = "wasm32"))]
#![cfg_attr(not(feature = "native-skia"), allow(dead_code))]

use rhwp::{
    model::bin_data::{BinData, BinDataContent, BinDataType},
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::TextRunNode,
    DocumentCore,
};

pub(crate) fn fixture(current: bool, kind: u8) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "A").unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = "RHWP Host Fixture".into();
            font.is_embedded = true;
            font.resolved_bin_data_id = Some(1);
        }
    }
    for style in &mut doc.doc_info.char_shapes {
        style.base_size = 900;
        style.shadow_type = kind;
        style.shadow_offset_x = 7;
        style.shadow_offset_y = 3;
    }
    doc.doc_info.bin_data_list.push(BinData {
        data_type: BinDataType::Embedding,
        storage_id: 1,
        extension: Some("ttf".into()),
        ..Default::default()
    });
    doc.bin_data_content.push(BinDataContent {
        id: 1,
        extension: "ttf".into(),
        data: include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf")
            .to_vec()
            .into(),
    });
    core.set_document(doc);
    core.set_hangul2024_compat(current);
    core
}

fn collect(node: &LayerNode, runs: &mut Vec<TextRunNode>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, runs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, runs)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { run, .. } = op {
                    if run.display_or_text() == "A" {
                        runs.push(run.as_ref().clone());
                    }
                }
            }
        }
    }
}

pub(crate) fn actual_run(core: &DocumentCore, profile: RenderProfile) -> TextRunNode {
    let original = core.export_hwp_native().unwrap();
    let tree = core.build_page_layer_tree_with_profile(0, profile).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    assert_eq!(runs.len(), 1, "본문 A의 누락·중복이 없다");
    assert_eq!(core.page_count(), 1);
    assert_eq!(
        core.export_hwp_native().unwrap(),
        original,
        "paint 계획이 저장 원본을 바꾸지 않는다"
    );
    runs.pop().unwrap()
}

pub(crate) fn host_request_document(current: bool, kind: u8) -> DocumentCore {
    let mut core = fixture(current, kind);
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.is_embedded = false;
            font.resolved_bin_data_id = None;
        }
    }
    doc.doc_info.bin_data_list.clear();
    doc.bin_data_content.clear();
    core.set_document(doc);
    core.set_hangul2024_compat(current);
    core
}

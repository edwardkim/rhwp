//! #8153: 같은 런의 오른쪽 탭은 명시적 정지와 쪽번호 순서를 보존한다.
//! HWP5 §4.2.7 표 36·37과 독립 Print 280쪽, 관련 계획의 원본 관측 참조.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a LayerNode, runs: &mut Vec<(&'a BoundingBox, &'a TextRunNode)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, runs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, runs);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { bbox, run, .. } = op {
                    if matches!(run.text.as_str(), "\t285" | "\t286") {
                        runs.push((bbox, run));
                    }
                }
            }
        }
    }
}

fn verify(extension: &str) {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(source).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let tree = core
        .build_page_layer_tree_with_profile(279, RenderProfile::Print)
        .unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    assert_eq!(runs.len(), 2, "대응 쪽번호의 누락·중복 금지");
    assert_eq!(runs[0].1.text, "\t285");
    assert_eq!(runs[1].1.text, "\t286");
    assert!(runs[0].0.y + runs[0].0.height < runs[1].0.y);
    assert_ne!(runs[0].1.para_index, runs[1].1.para_index);
    for (bbox, run) in runs {
        assert!(run.section_index.is_some() && run.para_index.is_some());
        let style = &run.style;
        assert!(!style.auto_tab_right, "원본은 자동 우측 탭이 아니다");
        let stop = style
            .tab_stops
            .iter()
            .find(|stop| stop.tab_type == 1)
            .unwrap();
        let column_origin = bbox.x - style.line_x_offset;
        let declared_right = column_origin + stop.position;
        let body_right = column_origin + style.text_start_offset + style.available_width;
        assert!(
            declared_right < body_right,
            "셀 안에 유효한 명시적 정지가 있다"
        );
        let actual_right = bbox.x + bbox.width;
        assert!(
            (actual_right - declared_right).abs() < 1e-6,
            "같은 런의 오른쪽 탭은 셀 우단 대신 원본 정지에 맞춘다: actual={actual_right}, declared={declared_right}, body={body_right}"
        );
    }
}

#[test]
fn hwp_intrarun_right_tab_preserves_declared_stop() {
    verify("hwp");
}

#[test]
fn hwpx_intrarun_right_tab_preserves_declared_stop() {
    verify("hwpx");
}

//! 검사 유형: 관계형 — 글자 위치와 아래 밑줄의 줄 기준선 소유를 구분한다.
//! 주석 갱신: 2026-11-14T21:29:06+09:00.
//! exam_eng.hwp 전8쪽은 보정 전 Native/fresh WASM 최저97.12%/97.11%다.
//! 문서의 절대 픽셀 좌표를 고정하지 않는다. 독립 한컴2020 Print 대조군의
//! 글자 위치 -12/0/+12에서 밑줄은 줄 간격만 따라가고 글리프만 추가 이동했다.
//! 원본 run·장식의 canonical 소유는 동일하며 재생 좌표만 이 계약을 소비해야 한다.
//! 글자 위치 0과 위 밑줄·취소선은 기존 글리프 기준선을 유지하는 대조군이다.
//! 시스템 글꼴 없이 명시적 글자 전진으로 같은 JSON 소비 경로를 검증한다.

use rhwp::{
    model::style::UnderlineType,
    paint::{LayerNode, PageLayerTree, PaintOp, TextDecorationKind},
    renderer::{
        render_tree::{BoundingBox, FieldMarkerType, TextRunNode},
        TextStyle,
    },
};
use serde_json::Value;

fn layer(position: i8, underline: UnderlineType, kind: TextDecorationKind) -> PageLayerTree {
    let bounds = BoundingBox::new(10.0, 20.0, 12.0, 16.0);
    let run = TextRunNode {
        text: "A\u{a0}".to_owned(),
        style: TextStyle {
            font_size: 16.0,
            font_position: Some(position),
            underline,
            strikethrough: kind == TextDecorationKind::Strikethrough,
            ..Default::default()
        },
        char_shape_id: None,
        para_shape_id: None,
        section_index: Some(0),
        para_index: Some(0),
        char_start: Some(0),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: FieldMarkerType::None,
        layout_positions: Some(vec![0.0, 8.0, 12.0]),
        display_text: None,
    };
    PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::leaf(
            bounds,
            None,
            vec![
                PaintOp::text_run(bounds, run.clone()),
                PaintOp::text_decoration(bounds, run, kind),
            ],
        ),
    )
}

#[test]
fn bottom_underline_keeps_the_line_baseline_and_original_text_ownership() {
    for position in [-12, 0, 12] {
        let tree = layer(
            position,
            UnderlineType::Bottom,
            TextDecorationKind::Underline,
        );
        let LayerNode {
            kind: rhwp::paint::LayerNodeKind::Leaf { ops },
            ..
        } = &tree.root
        else {
            unreachable!()
        };
        let PaintOp::TextRun { run: text, .. } = &ops[0] else {
            unreachable!()
        };
        let PaintOp::TextDecoration {
            run: decoration, ..
        } = &ops[1]
        else {
            unreachable!()
        };
        assert_eq!(
            text.as_ref(),
            decoration.as_ref(),
            "원본 소유·variant 검증용 run은 동일해야 한다"
        );
        assert_eq!(text.style.font_position, Some(position));
        let value: Value = serde_json::from_str(&tree.to_json()).unwrap();
        let serialized = &value["root"]["ops"];
        assert_eq!(
            serialized[1]["decoration"]["baseline"], 12.0,
            "아래 밑줄은 글자 위치와 무관하게 줄 기준선을 소비한다"
        );
        let (_, glyph_baseline) = text
            .style
            .script_draw_metrics(text.style.font_size, text.baseline);
        assert!(
            (glyph_baseline - (12.0 + 16.0 * f64::from(position) / 100.0)).abs() < 0.001,
            "본문 글리프는 원본 글자 위치를 계속 소비해야 한다"
        );
        assert_eq!(
            serialized[1]["decoration"]["positions"],
            serde_json::json!([0.0, 8.0, 12.0]),
            "밑줄은 같은 글자 전진·소유 범위를 유지한다"
        );
    }
}

#[test]
fn top_underline_and_strike_keep_their_existing_glyph_baselines() {
    for (underline, kind) in [
        (UnderlineType::Top, TextDecorationKind::Underline),
        (UnderlineType::None, TextDecorationKind::Strikethrough),
    ] {
        let tree = layer(-12, underline, kind);
        let value: Value = serde_json::from_str(&tree.to_json()).unwrap();
        let ops = &value["root"]["ops"];
        let expected = 12.0 - 16.0 * 12.0 / 100.0;
        assert!((ops[1]["decoration"]["baseline"].as_f64().unwrap() - expected).abs() < 0.001);
    }
}

/// 관계형: 저장 줄의 실제 전진 슬롯으로 압축하며 NBSP 밑줄 끝을 줄 프레임에 맞춘다.
#[test]
fn nbsp_compression_counts_the_advances_actually_replayed() {
    use rhwp::{
        renderer::render_tree::{RenderNode, RenderNodeType},
        DocumentCore,
    };
    let bytes =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/samples/exam_eng.hwp")).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(1).unwrap();
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::TextLine(_))
            && node.children.iter().any(|child| {
                matches!(&child.node_type,
                RenderNodeType::TextRun(run) if run.para_index == Some(68))
            })
        {
            out.push(node);
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let mut lines = Vec::new();
    collect(&tree.root, &mut lines);
    assert_eq!(lines.len(), 1, "Woman 밑줄은 저장한 한 줄에만 속한다");
    let line = lines[0];
    let runs: Vec<_> = line
        .children
        .iter()
        .filter(|child| matches!(child.node_type, RenderNodeType::TextRun(_)))
        .collect();
    assert!(runs
        .iter()
        .any(|node| matches!(&node.node_type,RenderNodeType::TextRun(run)
        if run.text.chars().all(|ch|ch=='\u{a0}') && run.text.chars().count()>1)));
    let right = runs
        .iter()
        .map(|node| node.bbox.x + node.bbox.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let expected = line.bbox.x + line.bbox.width;
    let quantum = 96.0 / 7200.0;
    assert!(
        (right - expected).abs() <= quantum * (runs.len() + 1) as f64,
        "NBSP 전진을 압축 분모에서 빼면 밑줄이 줄 끝에 못 미친다: {right} / {expected}"
    );
}

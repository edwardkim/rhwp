//! 저장 rowspan 조각이 원본 문단 범위를 RenderTree와 LayerTree에 그대로 전달한다.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Cell;
use rhwp::paint::layer_tree::{GroupKind, LayerNode, LayerNodeKind};
use rhwp::paint::profile::RenderProfile;
use rhwp::paint::LayerBuilder;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TableCellNode};
use serde_json::{json, Value};

fn source_cell(doc: &DocumentCore) -> &Cell {
    let Control::Table(outer) = &doc.document().sections[0].paragraphs[674].controls[0] else {
        panic!("674번 본문 문단의 바깥 표");
    };
    let Control::Table(inner) = &outer.cells[2].paragraphs[0].controls[1] else {
        panic!("바깥 셀[2]의 안쪽 표");
    };
    &inner.cells[39]
}

fn target_cell(cell: &TableCellNode) -> bool {
    cell.model_cell_index == Some(39) && cell.row == 18 && cell.col == 0 && cell.row_span == 6
}

fn render_cells<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TableCell(cell) if target_cell(cell)) {
        result.push(node);
    }
    for child in &node.children {
        render_cells(child, result);
    }
}

fn layer_cells<'a>(node: &'a LayerNode, result: &mut Vec<&'a TableCellNode>) {
    match &node.kind {
        LayerNodeKind::Group {
            group_kind,
            children,
            ..
        } => {
            if let GroupKind::TableCell(cell) = group_kind {
                if target_cell(cell) {
                    result.push(cell);
                }
            }
            for child in children {
                layer_cells(child, result);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            layer_cells(child, result)
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

fn rendered_paragraphs(
    node: &RenderNode,
    inherited_paragraph: Option<usize>,
    result: &mut BTreeMap<usize, String>,
) {
    let paragraph = match &node.node_type {
        RenderNodeType::TextLine(line) => line.para_index.or(inherited_paragraph),
        _ => inherited_paragraph,
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        // is_para_end는 이 run이 문단 끝임을 뜻하며 실제 텍스트도 포함한다.
        // 대상 셀 subtree의 TextLine 번호로 묶어 스타일별 run 전체를 보존한다.
        if let Some(paragraph) = paragraph.or(run.para_index) {
            result.entry(paragraph).or_default().push_str(&run.text);
        }
    }
    for child in &node.children {
        rendered_paragraphs(child, paragraph, result);
    }
}

fn json_cells<'a>(value: &'a Value, result: &mut Vec<&'a Value>) {
    if value.get("kind") == Some(&json!("tableCell"))
        && value.get("modelCellIndex") == Some(&json!(39))
        && value.get("row") == Some(&json!(18))
        && value.get("col") == Some(&json!(0))
        && value.get("rowSpan") == Some(&json!(6))
    {
        result.push(value);
    }
    match value {
        Value::Array(values) => {
            for value in values {
                json_cells(value, result);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                json_cells(value, result);
            }
        }
        _ => {}
    }
}

#[test]
fn stored_rowspan_fragments_preserve_source_paragraph_ranges_and_contents() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("rhwp-studio/public/samples/kps-ai.hwp");
    let doc =
        DocumentCore::from_bytes(&fs::read(path).expect("기존 kps-ai 샘플")).expect("문서 열기");
    let paragraphs = &source_cell(&doc).paragraphs;
    let before = serde_json::to_value(paragraphs).expect("원본 문단 직렬화");
    assert_eq!(
        paragraphs
            .iter()
            .map(|para| para.text.as_str())
            .collect::<Vec<_>>(),
        ["3. 민간", "소프트웨어", "", "시장침해", " 가능성"]
    );

    // 쪽 번호는 0부터 센다. 두 번째 조각의 로컬 문단 0은 원본 문단 3이다.
    for (page_index, (start, end)) in [(64, (0, 3)), (65, (3, 5))] {
        let page = doc.build_page_render_tree(page_index).expect("쪽 조판");
        let mut cells = Vec::new();
        render_cells(&page.root, &mut cells);
        assert_eq!(cells.len(), 1, "{page_index}쪽 대상 셀은 하나");
        let cell_node = cells[0];
        let RenderNodeType::TableCell(cell) = &cell_node.node_type else {
            unreachable!();
        };
        assert_eq!(cell.model_para_range, Some((start, end)));
        assert!(cell.clip, "분할 쪽의 셀 클립을 유지한다");

        // 스타일별로 나뉜 run을 합쳐 복제된 문단의 텍스트를 원본 slice와 대조한다.
        let mut actual = BTreeMap::new();
        rendered_paragraphs(cell_node, None, &mut actual);
        actual.retain(|_, text| !text.is_empty());
        let expected: BTreeMap<_, _> = paragraphs[start..end]
            .iter()
            .enumerate()
            .filter(|(_, para)| !para.text.is_empty())
            .map(|(local, para)| (local, para.text.clone()))
            .collect();
        assert_eq!(actual, expected, "{page_index}쪽 문단 내용 보존");

        let layer = LayerBuilder::new(RenderProfile::Screen).build(&page);
        let mut rich_cells = Vec::new();
        layer_cells(&layer.root, &mut rich_cells);
        assert_eq!(rich_cells.len(), 1);
        assert_eq!(rich_cells[0].model_para_range, Some((start, end)));
        let serialized: Value = serde_json::from_str(&layer.to_json()).expect("layer JSON");
        let mut metadata = Vec::new();
        json_cells(&serialized, &mut metadata);
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0]["modelParaRange"], json!([start, end]));
    }

    assert_eq!(
        serde_json::to_value(&source_cell(&doc).paragraphs).expect("조판 후 문단 직렬화"),
        before,
        "읽기 전용 조판과 layer 변환은 원본 Paragraph의 서식·컨트롤·저장 줄 정보를 변경하지 않는다"
    );
}

//! #7445/#7642: 원본 글줄·분수 셀·후속 표가 같은 저장 프레임을 소비한다.
//! 독립 Hancom2020 재저장이 보존한 원본 줄/표와 Print 전4쪽 대조를 근거로 한다.
//! 전4쪽 최저 Native96.14438% / fresh WASM96.06833%, 쪽수4/4.
//! Stage20 repair772/775 TSV와 3·4쪽 review/overlay를 직접 판독했다.
//! 절대 px를 고정하지 않고 줄/셀 포함, 문단 소유와 원본 상대 간격을 검사한다.
//! 완료 페이지의 별도 px 검사는 각 함수에 독립 Print 근거를 기록한다.
use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_science.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("검증 원본 읽기")).unwrap();
    core.set_hangul2024_compat(false);
    core
}
fn visit<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        visit(child, out);
    }
}
fn text(node: &RenderNode) -> String {
    match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => node.children.iter().map(text).collect::<Vec<_>>().join(""),
    }
}
fn equal_ratio(actual: f64, expected: f64) {
    let roundoff = actual.abs().max(expected.abs()).max(1.0) * f64::EPSILON * 512.0;
    assert!(
        (actual - expected).abs() <= roundoff,
        "actual={actual}, source={expected}"
    );
}

#[test]
fn fraction_text_lines_are_contained_by_their_own_cell_rows() {
    let core = load();
    let tree = core.build_page_render_tree(2).unwrap();
    let mut all = vec![];
    visit(&tree.root, &mut all);
    let tables: Vec<_> = all
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t) if t.row_count == 2 && t.col_count == 1)
                && text(n).contains("오비탈")
        })
        .collect();
    assert_eq!(tables.len(), 1, "원문 분수 표의 소유/누락/중복");
    let mut cells = vec![];
    visit(tables[0], &mut cells);
    let cells: Vec<_> = cells
        .into_iter()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), 2);
    for cell in cells {
        let mut children = vec![];
        visit(cell, &mut children);
        let lines: Vec<_> = children
            .into_iter()
            .filter(|n| matches!(&n.node_type, RenderNodeType::TextLine(_)))
            .collect();
        assert!(!lines.is_empty());
        for line in lines {
            assert!(line.bbox.y >= cell.bbox.y);
            assert!(
                line.bbox.y + line.bbox.height
                    <= cell.bbox.y + cell.bbox.height + f64::EPSILON * cell.bbox.y.abs() * 512.0,
                "글줄 {:?}가 소유 행 {:?}를 벗어난다",
                line.bbox,
                cell.bbox
            );
        }
    }
}

#[test]
fn inline_fraction_keeps_the_owned_paragraph_left_edge() {
    let core = load();
    let tree = core.build_page_render_tree(3).unwrap();
    let mut all = vec![];
    visit(&tree.root, &mut all);
    let lines: Vec<_> = all
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(t)
        if t.para_index == Some(118) && t.line_index == Some(0))
        })
        .collect();
    let tables: Vec<_> = all
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
        if t.cell_context.is_none() && t.para_index == Some(118))
        })
        .collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(tables.len(), 1);
    equal_ratio(tables[0].bbox.x / lines[0].bbox.x, 1.0);
}

#[test]
fn following_table_and_text_keep_the_preceding_multicolumn_source_gap() {
    let core = load();
    assert_eq!(core.page_count(), 4, "독립 Print 전체 쪽수");
    let paragraphs = &core.document().sections[0].paragraphs;
    let tree = core.build_page_render_tree(3).unwrap();
    let mut all = vec![];
    visit(&tree.root, &mut all);
    let source = &paragraphs[118];
    let last_index = source.line_segs.len() - 1;
    let last = &source.line_segs[last_index];
    let host = &paragraphs[119];
    let [saved] = host.line_segs.as_slice() else {
        panic!("표의 한 줄 저장 프레임");
    };
    let lines: Vec<_> = all
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(t)
        if t.para_index == Some(118) && t.line_index == Some(last_index as u32))
        })
        .collect();
    let tables: Vec<_> = all
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
        if t.cell_context.is_none() && t.para_index == Some(119))
        })
        .collect();
    let following: Vec<_> = all
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(t)
        if t.para_index == Some(120) && t.line_index == Some(0))
        })
        .collect();
    assert_eq!(lines.len(), 1, "앞 마지막 글줄의 소유/누락/중복");
    assert_eq!(tables.len(), 1, "뒤 표의 소유/누락/중복");
    assert_eq!(following.len(), 1, "표 뒤 첫 글줄의 소유/누락/중복");
    let before = lines[0];
    let table = tables[0];
    let after = following[0];
    assert!(before.bbox.y + before.bbox.height < table.bbox.y);
    assert!(table.bbox.y + table.bbox.height <= after.bbox.y);
    let source_gap = saved.vertical_pos - last.vertical_pos - last.line_height;
    assert!(source_gap > 0);
    equal_ratio(
        (table.bbox.y - before.bbox.y - before.bbox.height) / before.bbox.height,
        f64::from(source_gap) / f64::from(last.line_height),
    );
    let after_gap =
        paragraphs[120].line_segs[0].vertical_pos - saved.vertical_pos - saved.line_height;
    assert!(after_gap > 0);
    equal_ratio(
        (after.bbox.y - table.bbox.y - table.bbox.height) / table.bbox.height,
        f64::from(after_gap) / f64::from(saved.line_height),
    );
    let emitted: Vec<_> = all
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextLine(line) if line.para_index == Some(118) => line.line_index,
            _ => None,
        })
        .collect();
    assert_eq!(
        emitted,
        (0..source.line_segs.len() as u32).collect::<Vec<_>>(),
        "원문 줄 순서/누락/중복"
    );
}

/// 완료 2쪽 Native98.97662% / fresh WASM98.91532%, 양쪽 review/overlay 판독.
/// 독립 Print PDF(1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14)
/// 2쪽의 실제 구분선 연산: (422.88,249.172)→(570.72,249.172)pt,
/// MediaBox 높이1116pt. CSS96으로 변환한 값이며 후보 bbox에서 얻지 않았다.
#[test]
fn completed_page_two_fraction_divider_keeps_independent_print_endpoints() {
    use rhwp::paint::RenderProfile;
    use serde_json::Value;
    let core = load();
    let tree = core.build_page_render_tree(1).unwrap();
    let mut all = vec![];
    visit(&tree.root, &mut all);
    let tables: Vec<_> = all
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.cell_context.is_none() && table.row_count == 2 && table.col_count == 1)
                && text(node).contains("중성자수")
        })
        .collect();
    assert_eq!(tables.len(), 1, "분수의 원문 소유/누락/중복");
    let table = tables[0];
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), 2);
    assert!(cells[0].bbox.y < cells[1].bbox.y);
    assert!(cells.iter().all(|cell| !text(cell).trim().is_empty()));

    fn owned<'a>(value: &'a Value, id: u64, out: &mut Vec<&'a Value>) {
        match value {
            Value::Object(map) => {
                if value["sourceNodeId"].as_u64() == Some(id)
                    && value["groupKind"]["kind"] == "table"
                {
                    out.push(value);
                }
                for child in map.values() {
                    owned(child, id, out);
                }
            }
            Value::Array(values) => {
                for child in values {
                    owned(child, id, out);
                }
            }
            _ => {}
        }
    }
    fn lines<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
        match value {
            Value::Object(map) => {
                if value["type"] == "line" {
                    out.push(value);
                }
                for child in map.values() {
                    lines(child, out);
                }
            }
            Value::Array(values) => {
                for child in values {
                    lines(child, out);
                }
            }
            _ => {}
        }
    }
    let paint: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(1, RenderProfile::Print)
            .unwrap(),
    )
    .unwrap();
    let mut owners = vec![];
    owned(&paint, u64::from(table.id), &mut owners);
    assert_eq!(owners.len(), 1, "최종 Print paint의 같은 표 소유");
    let mut dividers = vec![];
    lines(owners[0], &mut dividers);
    assert_eq!(dividers.len(), 1, "분수 구분선을 한 번만 그린다");
    let line = dividers[0];
    let x1 = line["x1"].as_f64().unwrap();
    let y1 = line["y1"].as_f64().unwrap();
    let x2 = line["x2"].as_f64().unwrap();
    let y2 = line["y2"].as_f64().unwrap();
    assert!(x1 < x2 && y1 == y2);
    assert!(
        cells[0].bbox.y < y1 && y1 < cells[1].bbox.y + cells[1].bbox.height,
        "구분선은 자신의 분자·분모 사이에 있다"
    );
    for (actual, expected) in [
        (x1, 563.84),
        (y1, 1155.7706666666666),
        (x2, 760.96),
        (y2, 1155.7706666666666),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "완료 2쪽 독립 Print CSS96 좌표: {actual} vs {expected}"
        );
    }
}

/// shortcut 전7쪽 Native/fresh WASM 최저91.78413/91.99857%, 미달0.
/// 완료7쪽은99.65777/99.79309%이며 양쪽 review/overlay를 판독했다.
/// 독립 shortcut-2022.pdf의7쪽 MediaBox841×595pt에서 마지막 두 단의
/// '글 앞으로'와 '개체 풀기' 기준선을 CSS96으로 변환했다.
/// 후보 위치로 문단을 고르지 않고 원문·쪽·단 소유와 누락/중복을 함께 검사한다.
#[test]
fn completed_shortcut_last_page_keeps_both_column_tails_and_print_baselines() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/basic/shortcut.hwp")).expect("단축키 원본"),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 7, "독립 Print의 전체 쪽수");
    let anchors = [
        ("글 앞으로", "Shift+Home", 121.28),
        ("개체 풀기", "Ctrl+U", 589.92),
    ];
    let mut positions = Vec::new();
    for (label, shortcut, expected_x) in anchors {
        let source: Vec<_> = core.document().sections[0]
            .paragraphs
            .iter()
            .enumerate()
            .filter(|(_, para)| para.text.starts_with(label))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(source.len(), 1, "원본 항목의 단일 문단 소유");
        let mut found = Vec::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut nodes = Vec::new();
            visit(&tree.root, &mut nodes);
            for line in nodes.iter().filter(|node| {
                matches!(node.node_type, RenderNodeType::TextLine(_))
                    && text(node).starts_with(label)
            }) {
                assert_eq!(page, 6, "두 단의 마지막 항목은 마지막 쪽이 소유한다");
                let mut runs = Vec::new();
                visit(line, &mut runs);
                let owned: Vec<_> = runs
                    .iter()
                    .filter_map(|node| match &node.node_type {
                        RenderNodeType::TextRun(run) => Some((*node, run)),
                        _ => None,
                    })
                    .collect();
                assert!(!owned.is_empty());
                assert!(owned
                    .iter()
                    .all(|(_, run)| run.para_index == Some(source[0])));
                let actual: String = text(line).chars().filter(|c| !c.is_whitespace()).collect();
                let expected: String = format!("{label}{shortcut}")
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                assert_eq!(actual, expected, "내용 순서와 단축키의 누락/중복");
                let (node, run) = owned[0];
                found.push((node.bbox.x, node.bbox.y + run.baseline));
            }
        }
        assert_eq!(found.len(), 1, "원문 항목이 전체 문서에서 한 번만 배치된다");
        let (x, baseline) = found[0];
        assert!((x - expected_x).abs() <= 2.0, "독립 Print의 단별 시작 x");
        assert!(
            (baseline - 461.76).abs() <= 2.0,
            "독립 Print의 마지막 줄 기준선"
        );
        positions.push((x, baseline));
    }
    assert!(positions[0].0 < positions[1].0, "좌/우 단 순서");
    equal_ratio(positions[0].1, positions[1].1);
}

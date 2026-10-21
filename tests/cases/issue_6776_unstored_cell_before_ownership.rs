//! #6776 저장 줄이 없는 셀의 문단 앞 간격 소유.
//! 원본 HWP ParaShape raw400과 독립 한컴2020 재저장 case prev200HU를 대조한다.
//! 절대 쪽 좌표 대신 원본 간격의 단일 소비·내용 보존·메타데이터 중립성을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn text(n: &RenderNode) -> String {
    let mut s = String::new();
    if let RenderNodeType::TextRun(r) = &n.node_type {
        s.push_str(&r.text)
    }
    for c in &n.children {
        s.push_str(&text(c))
    }
    s
}
fn parent(n: &RenderNode) -> Option<&RenderNode> {
    if matches!(&n.node_type,RenderNodeType::Table(t)if t.section_index==Some(0)&&t.para_index==Some(331)&&t.control_index==Some(0))
    {
        return Some(n);
    }
    n.children.iter().find_map(parent)
}
fn first(n: &RenderNode) -> Option<&RenderNode> {
    if matches!(n.node_type, RenderNodeType::TextLine(_)) && text(n).starts_with("① 법") {
        return Some(n);
    }
    n.children.iter().find_map(first)
}
fn gap(c: &DocumentCore) -> f64 {
    for page in 0..c.page_count() {
        let p = c.build_page_render_tree(page).unwrap();
        if let Some(t) = parent(&p.root) {
            if let Some(cell) = t
                .children
                .iter()
                .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(x)if x.row==1&&x.col==2))
            {
                if let Some(line) = first(cell) {
                    println!(
                        "page={} cell={:?} first={:?}",
                        page + 1,
                        cell.bbox,
                        line.bbox
                    );
                    return line.bbox.y - cell.bbox.y;
                }
            }
        }
    }
    panic!("source cell/first line missing")
}
#[test]
fn source_unstored_cell_keeps_its_declared_before_space() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let original = DocumentCore::from_bytes(&bytes).unwrap();
    let mut d = original.document().clone();
    let Control::Table(t) = &d.sections[0].paragraphs[331].controls[0] else {
        panic!("source table")
    };
    let cell = t.cells.iter().find(|c| c.row == 1 && c.col == 2).unwrap();
    let para = &cell.paragraphs[0];
    assert!(para.line_segs.is_empty());
    assert!(para.controls.is_empty());
    let expected_text = para
        .text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let style = para.para_shape_id as usize;
    let before = d.doc_info.para_shapes[style].spacing_before;
    assert!(before > 0);
    d.doc_info.para_shapes[style].spacing_before = 0;
    let mut zero = DocumentCore::new_empty();
    zero.set_document(d);
    let mut actual = String::new();
    let mut owners = 0;
    for page in 0..original.page_count() {
        let tree = original.build_page_render_tree(page).unwrap();
        if let Some(t) = parent(&tree.root) {
            if let Some(cell) = t
                .children
                .iter()
                .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(x)if x.row==1&&x.col==2))
            {
                actual.push_str(&text(cell));
                owners += 1;
            }
        }
    }
    assert_eq!(
        owners, 1,
        "source first paragraph/cell must remain on one page"
    );
    let actual = actual
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    assert_eq!(
        actual.matches(&expected_text).count(),
        1,
        "source paragraph must appear once without loss or duplication"
    );
    let a = gap(&original);
    let b = gap(&zero);
    // HWP ParaShape 간격은 물리 HWPUNIT의 두 배다(400→200HU).
    // 독립 한컴 재저장과 기존 #1749 간격 계약으로 확인한 단위 변환이다.
    assert_eq!(before % 2, 0);
    let expected = f64::from(before) / 150.0;
    println!(
        "gapOriginal={a} zero={b} sourceBefore={before} delta={} expected={expected}",
        a - b
    );
    assert!(
        ((a - b) - expected).abs() < 0.05,
        "missing source first-paragraph before-space"
    );
}

#[test]
fn a_single_column_definition_does_not_replace_text_before_space() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let original = DocumentCore::from_bytes(&bytes).unwrap();
    let mut doc = original.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[331].controls[0] else {
        panic!("source table")
    };
    let para = &mut table
        .cells
        .iter_mut()
        .find(|c| c.row == 1 && c.col == 2)
        .unwrap()
        .paragraphs[0];
    assert!(para.controls.is_empty());
    para.controls
        .push(Control::ColumnDef(rhwp::model::page::ColumnDef {
            column_count: 1,
            same_width: true,
            ..Default::default()
        }));
    let mut metadata = DocumentCore::new_empty();
    metadata.set_document(doc);
    assert!(
        (gap(&metadata) - gap(&original)).abs() < 0.05,
        "one-column metadata does not own or consume the text paragraph's before-space"
    );
}

/// 독립 원본 Print37쪽: 중첩 종료와 후속 두 행의 소유·내용·최종 baseline.
/// 같은 원본 전74쪽이 양쪽90% 이상이며37쪽은99.98294/99.98291%로 직접 판독했다.
/// 독립 Print의 원점·기준선을 ±2 CSSpx로 보호하고 source 내용/조각 관계도 유지한다.
#[test]
fn continued_parent_closes_after_the_child_and_its_owned_bottom_margins() {
    use rhwp::paint::{LayerJsonOptions, RenderProfile};
    use serde_json::Value;
    fn normalized(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_whitespace() && !c.is_control())
            .collect()
    }
    fn painted<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
        if v["type"] == "textRun" && v["source"]["stableSourceKey"] == key {
            out.push(v);
        }
        match v {
            Value::Object(map) => {
                for child in map.values() {
                    painted(child, key, out);
                }
            }
            Value::Array(values) => {
                for child in values {
                    painted(child, key, out);
                }
            }
            _ => {}
        }
    }
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 74, "독립 원본 Print의 전체 쪽수");
    let Control::Table(source) = &core.document().sections[0].paragraphs[331].controls[0] else {
        panic!("source parent")
    };
    let sc = source
        .cells
        .iter()
        .find(|c| c.row == 2 && c.col == 2)
        .unwrap();
    let child = sc.paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .unwrap();
    assert!(!child.repeat_header);
    let expected = (f64::from(child.outer_margin_bottom)
        + f64::from(sc.effective_padding(&source.padding).bottom))
        / 75.0;
    let page = core.build_page_render_tree(36).unwrap();
    let table = parent(&page.root).unwrap();
    let cell = table
        .children
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TableCell(t) if t.row == 2 && t.col == 2))
        .unwrap();
    let nested = cell
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .unwrap();
    let last: Vec<_> = nested.children.iter().filter(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row + 1 == child.row_count)).collect();
    assert_eq!(last.len(), usize::from(child.col_count));
    let bottom = last
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);
    let gap = cell.bbox.y + cell.bbox.height - bottom;
    assert!(
        (gap - expected).abs() < 0.05,
        "시작 띠를 다시 더하지 않은 원본 닫힘 여백: {gap} vs {expected}"
    );
    let paint: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                36,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    // 원본 PDF SHA674ef63b…의 MediaBox와 text matrix를 CSS96으로 변환한 값이다.
    // 현재 후보의 bbox에서 기대값을 복사하지 않는다.
    for (row, expected_baseline, expected_pages) in [
        (3, 609.893472, vec![36]),
        (4, 903.9720853333333, vec![36, 37]),
    ] {
        let (source_index, source_cell) = source
            .cells
            .iter()
            .enumerate()
            .find(|(_, c)| c.row == row && c.col == 2)
            .unwrap();
        let expected_text: String = source_cell
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        let mut owners = vec![];
        let mut actual = String::new();
        for index in 0..core.page_count() {
            let tree = core.build_page_render_tree(index).unwrap();
            if let Some(t) = parent(&tree.root) {
                for c in &t.children {
                    if matches!(&c.node_type, RenderNodeType::TableCell(v) if v.row == row && v.col == 2)
                    {
                        owners.push(index);
                        actual.push_str(&text(c));
                    }
                }
            }
        }
        assert_eq!(
            owners, expected_pages,
            "후속 셀의 전체 조각을 독립 Print와 같은 쪽에 한 번씩 배치한다"
        );
        assert_eq!(
            normalized(&actual),
            normalized(&expected_text),
            "후속 셀의 내용·순서·누락·중복"
        );
        let key = format!("section:0/para:0/char:0/cell:331:0:{source_index}:0:0");
        let mut runs = vec![];
        painted(&paint, &key, &mut runs);
        assert_eq!(
            runs.len(),
            1,
            "원문 첫 글리프를 최종 Print에서 한 번만 그린다"
        );
        let run = runs[0];
        assert_eq!(run["text"].as_str().unwrap().trim(), "①");
        let matrix = &run["placement"]["runToPage"];
        let x = matrix["e"].as_f64().unwrap();
        let baseline = matrix["f"].as_f64().unwrap();
        assert!(
            (x - 348.45115333333337).abs() <= 2.0,
            "독립 Print의 원문 첫 글리프 x"
        );
        assert!(
            (baseline - expected_baseline).abs() <= 2.0,
            "독립 Print 최종 기준선: {baseline} vs {expected_baseline}"
        );
        assert!(
            bottom < baseline,
            "중첩 표의 실제 종료 다음에 후속 행을 배치한다"
        );
    }
}

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
    // 원본 PDF SHA 674ef63b…의 MediaBox와 text matrix를 CSS 96으로 변환한 값이다.
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

// 원자 TAC 표와 후속 문단의 같은 원본 간격을 보호한다. 보정 64의 전쪽 시각 증적 참조.
#[test]
fn atomic_tac_host_keeps_its_source_line_spacing_in_the_following_flow() {
    fn find_table(node: &RenderNode, para: usize) -> Option<&RenderNode> {
        if matches!(&node.node_type,RenderNodeType::Table(t)if t.section_index==Some(0)&&t.para_index==Some(para))
        {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_table(child, para))
    }
    fn following_top(core: &DocumentCore) -> f64 {
        assert_eq!(core.page_count(), 74);
        let tree = core.build_page_render_tree(20).unwrap();
        find_table(&tree.root, 203)
            .expect("같은21쪽의 후속 간접편익 표")
            .bbox
            .y
    }

    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let source = DocumentCore::from_bytes(&bytes).unwrap();
    let mut doc = source.document().clone();
    let Control::Table(parent) = &mut doc.sections[0].paragraphs[193].controls[0] else {
        panic!("원본 부모 표")
    };
    let owner = parent
        .cells
        .iter_mut()
        .find(|cell| cell.row == 6 && cell.col == 1)
        .unwrap();
    let Control::Table(child) = owner.paragraphs[0]
        .controls
        .iter_mut()
        .find(|control| matches!(control, Control::Table(_)))
        .unwrap()
    else {
        unreachable!()
    };
    let host = &mut child.cells[0].paragraphs[30];
    assert!(host.line_segs.is_empty());
    assert!(host.text.trim().is_empty());
    let Control::Table(atom) = &host.controls[0] else {
        panic!("원본 원자 표")
    };
    assert!(atom.common.treat_as_char);
    assert_eq!((atom.row_count, atom.col_count), (11, 3));
    let original_style = doc.doc_info.para_shapes[host.para_shape_id as usize].clone();
    let base_size = doc.doc_info.char_shapes[host.char_shapes[0].char_shape_id as usize].base_size;
    assert_eq!(base_size, 1300);
    assert_eq!(original_style.line_spacing, 120);
    let expected = (f64::from(base_size) / 100.0)
        * (96.0 / 72.0)
        * (f64::from(original_style.line_spacing) / 100.0 - 1.0);
    let mut style = original_style.clone();
    style.line_spacing = 100;
    style.line_spacing_v2 = 100;
    style.raw_data = None;
    host.para_shape_id = doc.doc_info.para_shapes.len() as u16;
    doc.doc_info.para_shapes.push(style);
    let mut original = DocumentCore::new_empty();
    original.set_document(source.document().clone());
    let mut zero = DocumentCore::new_empty();
    zero.set_document(doc);
    let before = following_top(&original);
    let without = following_top(&zero);
    println!(
        "with_source_spacing={before:.9} without={without:.9} delta={:.9} expected={expected:.9}",
        before - without
    );
    assert!(
        ((before - without) - expected).abs() < 0.05,
        "TAC 원자 표 뒤의 원본 줄간격을 후속 흐름에서 한 번 소비한다"
    );
}

/// 같은 원본 전 74쪽 Native/fresh WASM이 각각 90% 이상이고 21쪽은 양쪽 100%다.
/// 21쪽 review·overlay를 직접 판독한 원자 표→후속 제목의 소속·내용·기준선을 보호한다.
/// 독립 Print PDF SHA 674ef63b…의 text matrix 원점이며 현재 후보 bbox를 기대값으로 쓰지 않는다.
#[test]
fn following_title_preserves_the_independent_print_baseline_after_an_atomic_tac_table() {
    use rhwp::paint::{LayerJsonOptions, RenderProfile};
    use serde_json::Value;
    fn normalize(s: &str) -> String {
        s.chars()
            .filter(|ch| !ch.is_whitespace() && !ch.is_control())
            .collect()
    }
    fn text(n: &RenderNode) -> String {
        let mut s = String::new();
        if let RenderNodeType::TextRun(run) = &n.node_type {
            s.push_str(&run.text)
        }
        for child in &n.children {
            s.push_str(&text(child))
        }
        s
    }
    fn outer_table(n: &RenderNode, para: usize) -> Option<&RenderNode> {
        if let RenderNodeType::Table(t) = &n.node_type {
            return (t.section_index == Some(0)
                && t.para_index == Some(para)
                && t.control_index == Some(0))
            .then_some(n);
        }
        n.children.iter().find_map(|child| outer_table(child, para))
    }
    fn painted<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
        if v["type"] == "textRun" && v["source"]["stableSourceKey"] == key {
            out.push(v)
        }
        match v {
            Value::Object(map) => {
                for child in map.values() {
                    painted(child, key, out)
                }
            }
            Value::Array(values) => {
                for child in values {
                    painted(child, key, out)
                }
            }
            _ => {}
        }
    }
    let data = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let core = DocumentCore::from_bytes(&data).unwrap();
    assert_eq!(core.page_count(), 74);
    let Control::Table(source) = &core.document().sections[0].paragraphs[203].controls[0] else {
        panic!("원본의 후속 제목 표")
    };
    let expected = source.cells[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<String>();
    let mut owners = vec![];
    let mut actual = String::new();
    for index in 0..core.page_count() {
        let page = core.build_page_render_tree(index).unwrap();
        if let Some(table) = outer_table(&page.root, 203) {
            owners.push(index);
            actual.push_str(&text(table));
        }
    }
    assert_eq!(owners, vec![20], "후속 제목의 쪽 소속과 단일 출현");
    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "제목의 내용·순서·누락·중복"
    );
    let page = core.build_page_render_tree(20).unwrap();
    let title = outer_table(&page.root, 203).unwrap();
    let parent = outer_table(&page.root, 193).unwrap();
    assert!(
        parent.bbox.y + parent.bbox.height <= title.bbox.y,
        "부모의 실제 종료 뒤에 제목을 배치한다"
    );
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                20,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let mut runs = vec![];
    painted(
        &layer,
        "section:0/para:0/char:0/cell:203:0:0:0:0",
        &mut runs,
    );
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["text"].as_str().unwrap().trim(), "□");
    // 독립 PDF SHA 674ef63b… 21쪽의 text matrix 원점을 CSS 96으로 변환했다.
    let matrix = &runs[0]["placement"]["runToPage"];
    let x = matrix["e"].as_f64().unwrap();
    let y = matrix["f"].as_f64().unwrap();
    let expected_x = 56.609317779541016 * 96.0 / 72.0;
    let expected_y = 354.212890625 * 96.0 / 72.0;
    println!("title origin=({x},{y}) independent=({expected_x},{expected_y})");
    assert!((x - expected_x).abs() <= 2.0, "독립 Print 제목의 x");
    assert!(
        (y - expected_y).abs() <= 2.0,
        "독립 Print 제목의 최종 기준선"
    );
}

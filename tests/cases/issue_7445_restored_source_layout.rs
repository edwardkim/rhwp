//! 관계형·고정형: 한컴 Print의 전체 쪽수와 원본에서 복원한 줄·표·그림 소유, 순서, 포함, 점유를 검사한다.
//! 주석 갱신: 2026-11-28T19:58:50+09:00.
//! 2026-11-28 한컴 Print 대조(96dpi, PDFium, 2px raw). 아래 수치는 Native / fresh WASM 순이다.
//! #7821 1쪽 99.97% / 99.97%, 2쪽 100.00% / 100.00%, 3쪽 98.16% / 98.12%. 전 3쪽 최저 98.16% / 98.12%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7821.tsv 및 wasm/7821.tsv.
//! #7830 8쪽 100.00% / 100.00%, 9쪽 99.99% / 99.99%. 전 29쪽 최저 98.88% / 98.95%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7830.tsv 및 wasm/7830.tsv.
//! #7831 19쪽 99.77% / 99.65%. 전 39쪽 최저 98.25% / 98.12%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7831.tsv 및 wasm/7831.tsv.
//! #7834 6쪽 99.99% / 99.99%, 7쪽 100.00% / 100.00%, 8쪽 100.00% / 100.00%. 전 15쪽 최저 99.98% / 99.98%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7834.tsv 및 wasm/7834.tsv.
//! #7864 1쪽 100.00% / 100.00%. 전 1쪽 최저 100.00% / 100.00%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7864.tsv 및 wasm/7864.tsv.
//! #7955 1쪽 99.87% / 99.85%. 전 3쪽 최저 99.87% / 99.85%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7955.tsv 및 wasm/7955.tsv.
//!
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn find(node: &RenderNode, pred: impl Fn(&RenderNode) -> bool + Copy) -> Option<&RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

fn top_table(node: &RenderNode, para: usize, control: usize) -> &RenderNode {
    find(node, |node| {
        matches!(&node.node_type, RenderNodeType::Table(table)
            if table.cell_context.is_none()
                && table.para_index == Some(para)
                && table.control_index == Some(control))
    })
    .expect("원본 문단의 독립 표")
}

fn body(node: &RenderNode) -> &RenderNode {
    find(node, |node| {
        matches!(node.node_type, RenderNodeType::Column(_))
    })
    .expect("본문 단")
}

#[test]
fn coanchored_float_continues_before_inline_table() {
    let core = open("samples/issue5807/coanchored_float_tac_order.hwp");
    assert_eq!(core.page_count(), 3);
    let first_page = core.build_page_render_tree(0).unwrap();
    let occupied = top_table(&first_page.root, 1, 0);
    let first_fragment = top_table(&first_page.root, 2, 0);
    assert!(occupied.bbox.y + occupied.bbox.height <= first_fragment.bbox.y);
    let previous = core.dump_page_items_json(Some(1));
    let following = core.dump_page_items_json(Some(2));
    let last = &previous[0]["columns"][0]["items"][0];
    let first = &following[0]["columns"][0]["items"][0];
    let inline = &following[0]["columns"][0]["items"][1];
    for chunk in [last, first] {
        assert_eq!(chunk["kind"], "partialTable");
        assert_eq!(chunk["paraIndex"], 2);
        assert_eq!(chunk["controlIndex"], 0);
    }
    assert!(last["endCut"].as_array().is_some_and(|cut| !cut.is_empty()));
    assert_eq!(last["endCut"], first["startCut"]);
    assert_eq!(inline["kind"], "table");
    assert_eq!(inline["paraIndex"], 2);
    assert_eq!(inline["controlIndex"], 1);
    let second_page = core.build_page_render_tree(1).unwrap();
    let second_body = body(&second_page.root);
    let opening = top_table(second_body, 2, 0);
    assert!(
        opening.bbox.y + opening.bbox.height <= second_body.bbox.y + second_body.bbox.height + 1.0
    );
    let third_page = core.build_page_render_tree(2).unwrap();
    let closing = top_table(&third_page.root, 2, 0);
    let inline = top_table(&third_page.root, 2, 1);
    assert!(closing.bbox.y + closing.bbox.height <= inline.bbox.y);
}

#[test]
fn saved_title_and_nested_table_belong_to_adjacent_fragments() {
    let core = open("samples/hwpx_sample2.hwp");
    assert_eq!(core.page_count(), 29);
    let title = |node: &RenderNode| {
        matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.text.contains("부도임대(경주금장로얄)"))
    };
    let nested_table = |node: &RenderNode| {
        matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(21) && table.control_index == Some(0)
                && table.cell_context.is_some())
    };
    let first = core.build_page_render_tree(7).unwrap();
    let next = core.build_page_render_tree(8).unwrap();
    let opening = top_table(&first.root, 74, 0);
    let closing = top_table(&next.root, 74, 0);
    let heading = find(opening, |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_)) && node.children.iter().any(title)
    })
    .expect("앞 조각 마지막 제목 줄");
    let blank = find(opening, |node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(20))
            && node.children.iter().all(|child| {
                matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text.trim().is_empty())
            })
    }).expect("제목 앞의 원본 빈 줄");
    let Control::Table(source) = &core.document().sections[0].paragraphs[74].controls[0] else {
        panic!("제목과 안쪽 표를 소유한 바깥 표");
    };
    let paragraphs = &source.cells[0].paragraphs;
    let blank_to_heading =
        paragraphs[21].line_segs[0].vertical_pos - paragraphs[20].line_segs[0].vertical_pos;
    assert!((heading.bbox.y - blank.bbox.y - f64::from(blank_to_heading) / 75.0).abs() < 0.1);
    let child = find(closing, nested_table).expect("뒤 조각 처음 중첩 표");
    assert!(opening.bbox.contains(&heading.bbox));
    assert!(closing.bbox.contains(&child.bbox));
    assert!(find(opening, nested_table).is_none());
    assert!(find(closing, title).is_none());
}

#[test]
fn page_number_host_keeps_both_tables_and_following_picture_in_order() {
    let core = open("mydocs/pr/assets/semanticist21-20261008/legacy-cropped-fixtures-v448/6575/tac_picture_line_top.hwpx");
    assert_eq!(core.page_count(), 1);
    let paragraphs = &core.document().sections[0].paragraphs;
    let Control::Picture(picture) = &paragraphs[1].controls[0] else {
        panic!("뒤 문단의 그림")
    };
    let page = core.build_page_render_tree(0).unwrap();
    let first = top_table(&page.root, 0, 4);
    let second = top_table(&page.root, 0, 5);
    let Control::Table(source) = &paragraphs[0].controls[4] else {
        panic!("번호 다음 표")
    };
    let [number_band, first_flow_line] = paragraphs[0].line_segs.as_slice() else {
        panic!("번호 제어가 있는 호스트의 저장 줄");
    };
    let physical_top = first_flow_line.vertical_pos - number_band.vertical_pos
        + i32::from(source.outer_margin_top);
    assert!((first.bbox.y - body(&page.root).bbox.y - f64::from(physical_top) / 75.0).abs() < 0.1);
    let owns_picture = |node: &RenderNode| {
        matches!(&node.node_type, RenderNodeType::Image(image)
            if image.para_index == Some(1)
                && image.bin_data_id == picture.image_attr.bin_data_id)
    };
    let line = find(&page.root, |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
            && node.children.iter().any(owns_picture)
    })
    .expect("원본 그림을 소유한 뒤 문단의 글줄");
    let picture = line
        .children
        .iter()
        .find(|node| owns_picture(node))
        .unwrap();
    assert!(
        (picture.bbox.y - line.bbox.y).abs() < 0.1,
        "글자처럼 취급하는 그림은 자기 줄의 상단에 붙는다"
    );
    assert!(first.bbox.y + first.bbox.height <= second.bbox.y);
    assert!(second.bbox.y + second.bbox.height <= picture.bbox.y);
    assert!(body(&page.root).bbox.contains(&picture.bbox));
}

fn owned_cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell)
            if cell.row == row && cell.col == col)
        })
        .expect("행과 열을 소유한 직접 셀")
}

#[test]
fn final_blank_cell_line_closes_the_accepted_continuation_box() {
    let core = open("samples/task2430/1382000_domestic_violence_survey.hwp");
    assert_eq!(core.page_count(), 39);
    let Control::Table(source) = &core.document().sections[0].paragraphs[90].controls[0] else {
        panic!("원본 이어받기 표");
    };
    let [cell] = source.cells.as_slice() else {
        panic!("단일 셀")
    };
    let blank = cell.paragraphs.last().unwrap();
    assert!(blank.text.is_empty() && blank.controls.is_empty());
    let [line] = blank.line_segs.as_slice() else {
        panic!("종료 빈 줄")
    };
    let occupied =
        line.vertical_pos + line.line_height + cell.effective_vertical_padding_hu(&source.padding);
    let page = core.build_page_render_tree(18).unwrap();
    let table = top_table(&page.root, 90, 0);
    let actual = owned_cell(table, 0, 0).bbox.height;
    assert!(
        (actual - f64::from(occupied) / 75.0).abs() < 0.1,
        "이어받기 셀은 마지막 원본 빈 줄의 높이까지 소유한다: {actual}"
    );
}

#[test]
fn ordinary_row_keeps_its_boxes_while_a_neighboring_span_continues() {
    let core = open("samples/issue6854/22037757-chuncheon-personnel-rule-annex13.hwpx");
    assert_eq!(core.page_count(), 15);
    let Control::Table(source) = &core.document().sections[0].paragraphs[4].controls[0] else {
        panic!("원본 행과 병합 칸을 소유한 표");
    };
    let cell = source
        .cells
        .iter()
        .find(|cell| cell.row == 48 && cell.col == 1)
        .unwrap();
    assert_eq!(cell.row_span, 1);
    let first = core.build_page_render_tree(6).unwrap();
    let second = core.build_page_render_tree(7).unwrap();
    let opening = owned_cell(top_table(&first.root, 4, 0), 48, 1);
    let following = top_table(&second.root, 4, 0);
    let closing = owned_cell(following, 48, 1);
    let next = owned_cell(following, 49, 1);
    assert!(opening.bbox.height > 0.0 && closing.bbox.height > 0.0);
    let height = opening.bbox.height + closing.bbox.height;
    assert!(
        (height - f64::from(cell.height) / 75.0).abs() < 0.1,
        "보이는 두 상자는 원본 일반 행의 선언 높이만 소유한다: {height}"
    );
    assert!(closing.bbox.y + closing.bbox.height <= next.bbox.y + 0.01);
}

#[test]
fn paragraph_anchored_decoration_tables_keep_the_hosts_left_margin() {
    let core = open("tests/fixtures/issue_7047/housing-lease-standard-form.hwp");
    assert_eq!(core.page_count(), 3);
    let doc = core.document();
    let page = core.build_page_render_tree(0).unwrap();
    let column = body(&page.root);
    let mut previous_bottom = column.bbox.y;
    for para_index in [6, 14] {
        let para = &doc.sections[0].paragraphs[para_index];
        let Control::Table(source) = &para.controls[0] else {
            panic!("임대차 계약 문단의 장식 표");
        };
        let style = &doc.doc_info.para_shapes[para.para_shape_id as usize];
        // HWP5 ParaShape의 좌우 여백 저장 단위는 1/2 HWPUNIT이다.
        let source_margin = f64::from(style.margin_left) / 2.0;
        let table_offset = f64::from(source.common.horizontal_offset as i32);
        let outer_margin = f64::from(source.outer_margin_left);
        let expected_offset = (source_margin + table_offset + outer_margin) / 75.0;
        let table = top_table(&page.root, para_index, 0);
        assert!(table.bbox.y >= previous_bottom);
        previous_bottom = table.bbox.y + table.bbox.height;
        assert!(
            (table.bbox.x - column.bbox.x - expected_offset).abs() < 0.1,
            "문단 여백과 표 오프셋이 물리 원점에 한 번씩 들어가야 한다: {para_index}"
        );
    }
}

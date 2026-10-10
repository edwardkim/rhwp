//! #7445 저장 줄 프레임은 측정과 실제 셀 배치에서 같은 물리 폭을 사용한다.
//! 원본의 부족한 cellSz를 한컴이 이미 넓은 프레임으로 인쇄한다는 독립 근거와
//! 전60쪽 Native/fresh WASM 수용 뒤 복구한 관계 검사다. 고정 x/y는 없다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, document::Document, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::DocumentCore;

fn target_table(document: &Document) -> &Table {
    match &document.sections[1].paragraphs[3].controls[0] {
        Control::Table(table) => table,
        _ => panic!("부실채권 관리대장 표"),
    }
}

fn find_node<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, predicate))
}

#[test]
fn saved_auction_frame_is_owned_by_both_measurement_and_painted_cell() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6031/3249937_asset_management_rules.hwpx");
    let bytes = std::fs::read(path).expect("정식 원본");
    let source = rhwp::parser::parse_document(&bytes).expect("원본 저장 속성");
    let table = target_table(&source);
    let index = table
        .cells
        .iter()
        .position(|cell| {
            cell.paragraphs.iter().any(|para| {
                para.text
                    .chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
                    == "경매"
            })
        })
        .expect("정본의 경매 칸");
    let cell = &table.cells[index];
    let source_frame = cell
        .paragraphs
        .iter()
        .flat_map(|para| &para.line_segs)
        .map(|seg| seg.column_start + seg.segment_width)
        .max()
        .expect("실제 저장 글줄 프레임");
    let padding = cell.effective_padding(&table.padding);
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core
        .build_page_render_tree(50)
        .expect("정본51쪽의 실제 배치");
    let painted_table = find_node(&tree.root, &|node| {
        matches!(
            &node.node_type,
            RenderNodeType::Table(data) if data.section_index == Some(1)
                && data.para_index == Some(3) && data.control_index == Some(0)
        )
    })
    .expect("관리대장의 실제 표");
    let painted = find_node(painted_table, &|node| {
        matches!(
            &node.node_type,
            RenderNodeType::TableCell(data) if data.model_cell_index == Some(index as u32)
        )
    })
    .expect("경매 칸의 실제 paint 소유");
    let inner_width = painted.bbox.width
        - hwpunit_to_px(
            i32::from(padding.left) + i32::from(padding.right),
            DEFAULT_DPI,
        );
    let frame_width = hwpunit_to_px(source_frame, DEFAULT_DPI);
    let roundoff = painted.bbox.width.abs() * 1e-9;
    assert!(
        inner_width + roundoff >= frame_width,
        "실제 셀 내부가 저장 글줄 프레임을 잘라서는 안 된다: {inner_width} < {frame_width}"
    );
    let measured_width = hwpunit_to_px(
        target_table(core.document()).cells[index].width as i32,
        DEFAULT_DPI,
    );
    assert!(
        (painted.bbox.width - measured_width).abs() <= roundoff,
        "측정/편집과 paint가 서로 다른 셀 폭을 소유한다"
    );

    // 독립 Print 60쪽의 내용 소속과 정상 저장 제목의 단일 글줄을 함께 보존한다.
    assert_eq!(core.page_count(), 60, "정본 문서와 같은 60쪽");
    let original_title = source
        .sections
        .iter()
        .find_map(|section| find_paragraph(&section.paragraphs, "해지증서"))
        .expect("원본 해지증서 제목");
    let loaded_title = core
        .document()
        .sections
        .iter()
        .find_map(|section| find_paragraph(&section.paragraphs, "해지증서"))
        .expect("로드한 해지증서 제목");
    assert_eq!(original_title.line_segs.len(), 1, "원본 한 줄 제목");
    assert!(
        original_title.line_segs[0].text_height > original_title.line_segs[0].line_height,
        "텍스트 테두리와 줄 높이가 다른 유효한 원본"
    );
    assert_eq!(
        loaded_title.line_segs.len(),
        1,
        "정상 저장 제목을 재줄바꿈하지 않는다"
    );
    assert_eq!(
        loaded_title.line_segs[0].text_start,
        original_title.line_segs[0].text_start
    );
    assert_eq!(
        loaded_title.line_segs[0].tag
            & rhwp::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY,
        0,
        "원본 제목을 합성 줄로 대체하지 않는다"
    );
    let title_page = core.build_page_render_tree(49).expect("해지증서 소속 쪽");
    let title_line = find_node(&title_page.root, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
            && compact(&node_text(node)) == "해지증서"
    })
    .expect("전체 제목이 한 실제 글줄에 보존된다");
    let title_cell = find_node(&title_page.root, &|node| {
        matches!(node.node_type, RenderNodeType::TableCell(_))
            && find_node(node, &|child| std::ptr::eq(child, title_line)).is_some()
    })
    .expect("제목을 소유한 칸");
    assert!(
        title_line.bbox.x >= title_cell.bbox.x
            && title_line.bbox.x + title_line.bbox.width
                <= title_cell.bbox.x + title_cell.bbox.width,
        "제목 글줄은 원래 칸 안에 포함된다"
    );

    // 서명 기호는 원문을 보존하면서 같은 글줄의 한글과 같은 전각을 소유한다.
    let signature_page = core.build_page_render_tree(53).expect("계약포기 각서 쪽");
    let signature = find_node(&signature_page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run)
            if compact(&run.text) == "각서자\u{F00E1}")
    })
    .expect("서명과 인 기호가 같은 글줄에 있다");
    let RenderNodeType::TextRun(signature_run) = &signature.node_type else {
        unreachable!()
    };
    let measurer = rhwp::renderer::layout::EmbeddedTextMeasurer;
    use rhwp::renderer::layout::TextMeasurer;
    let seal = measurer.estimate_text_width("\u{F00E1}", &signature_run.style);
    let hangul = measurer.estimate_text_width("각", &signature_run.style);
    assert!(
        (seal - hangul).abs() <= hwpunit_to_px(1, DEFAULT_DPI) + hangul.abs() * 1e-9,
        "인 기호를 반각으로 좁혀 가운데 정렬의 글줄 전체를 옮기지 않는다"
    );
    let footer = find_node(&signature_page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run)
            if compact(&run.text) == "매립지등관리자귀하")
    })
    .expect("수신인 내용 보존");
    let RenderNodeType::TextRun(footer_run) = &footer.node_type else {
        unreachable!()
    };
    assert!(
        footer_run.style.bold && footer_run.style.hft_fullwidth_dot,
        "원본 HFT 굵은 글자모양을 사용하는 수신인"
    );
    let mut regular = footer_run.style.clone();
    regular.bold = false;
    assert!(
        measurer.estimate_text_width("매립지등", &footer_run.style)
            > measurer.estimate_text_width("매립지등", &regular),
        "HFT 굵은 한글은 일반체와 다른 전진 간격을 소유한다"
    );
    assert!(
        (measurer.estimate_text_width(" ", &footer_run.style)
            - measurer.estimate_text_width(" ", &regular))
        .abs()
            <= footer_run.style.font_size * 1e-9,
        "공백에 굵은 한글의 추가 간격을 전파하지 않는다"
    );
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn find_paragraph<'a>(
    paragraphs: &'a [rhwp::model::paragraph::Paragraph],
    text: &str,
) -> Option<&'a rhwp::model::paragraph::Paragraph> {
    for paragraph in paragraphs {
        if compact(&paragraph.text) == text {
            return Some(paragraph);
        }
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    if let Some(found) = find_paragraph(&cell.paragraphs, text) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

fn node_text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.text.clone();
    }
    node.children.iter().map(node_text).collect()
}

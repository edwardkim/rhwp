//! Issue #5598: 좁은 표 칸에서 내어쓰기가 줄 상자를 다 먹으면 이어지는 줄의 글자가 칸 밖으로
//! 밀려 나간다.
//!
//! 재현 문서(코퍼스 00539, `자치수형자 선정 절차`)의 `분류처우위원회 심의ㆍ의결` 칸은 안쪽 폭이
//! 107.7px 인데 문단 내어쓰기가 −104.4px 다. 종전에는 둘째 줄 상자가 `x=193.3 w=3.3` 으로 무너져
//! `의결` 이 칸 오른쪽(206.1) 밖에 그려졌다. 한글은 같은 문단의 두 줄을 모두 칸 안쪽 폭으로
//! 조판한다(저장 LINE_SEG 두 줄 모두 `cs=200 sw=8076`).
//!
//! 첫 줄은 내어쓰기의 기준선이므로 그대로 두고, 이어지는 줄에서 한 글자도 못 담게 되는 경우에만
//! 내어쓰기를 접는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{CharShape, Font, ParaShape};
use rhwp::model::table::{Cell, Table};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 합성 대조군의 셀 폭(HWPUNIT). 안쪽 폭은 원본 문단 여백에서 계산된다.
const CELL_WIDTH: u32 = 9499;
/// 칸 폭에 육박하는 내어쓰기.
const HANGING_INDENT: i32 = -7832;

fn document_with_narrow_cell(indent: i32) -> Document {
    let mut shape = ParaShape {
        margin_left: 200,
        margin_right: 200,
        indent,
        ..Default::default()
    };
    shape.line_spacing = 160;

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![shape];
    // 독립 Print가 읽을 수 있는 명시적 글꼴·글자 크기와 정상 용지를 사용한다.
    doc.doc_info.font_faces = vec![
        vec![Font {
            name: "바탕".into(),
            alt_type: 1,
            ..Default::default()
        }];
        7
    ];
    doc.doc_info.char_shapes = vec![CharShape {
        base_size: 900,
        ..Default::default()
    }];

    let mut cell_para = Paragraph::default();
    cell_para.text = "분류처우위원회 심의ㆍ의결".to_string();
    cell_para.para_shape_id = 0;

    let cell = Cell {
        width: CELL_WIDTH,
        height: 6317,
        paragraphs: vec![cell_para],
        ..Default::default()
    };

    let table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell],
        ..Default::default()
    };

    let mut host = Paragraph::default();
    host.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.section_def.page_def.width = 59528;
    section.section_def.page_def.height = 84188;
    section.paragraphs.push(host);
    doc.sections.push(section);
    doc
}

/// 실제 페인트의 가시 글자 점유가 소유 셀을 넘는 최대 폭. 말미 공백의
/// 논리 advance를 잉크로 오인하지 않고 내용·순서도 함께 보존한다.
fn worst_overflow(doc: &Document, printed_input: &[u8]) -> f64 {
    use quick_xml::{events::Event, Reader};
    use rhwp::renderer::render_tree::RenderNodeType;
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    assert_eq!(bytes, printed_input, "독립 Print에 사용한 입력과 동일하다");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 한 쪽");
    let tree = core.build_page_render_tree(0).expect("render tree");
    fn find_cell(
        node: &rhwp::renderer::render_tree::RenderNode,
    ) -> Option<&rhwp::renderer::render_tree::RenderNode> {
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            return Some(node);
        }
        node.children.iter().find_map(find_cell)
    }
    let cell = find_cell(&tree.root).expect("소유 셀");
    let left = cell.bbox.x;
    let right = left + cell.bbox.width;
    let source_em = f64::from(doc.doc_info.char_shapes[0].base_size) * 96.0 / 7200.0;
    // 선언한 바탕의 한글/아래아는 1em 전진 상자다. 실제 SVG의 글자 원점과
    // 원본 크기의 점유를 비교하며 문서의 절대 위치를 기대값으로 고정하지 않는다.
    let svg = core
        .render_page_svg_layer_native(0)
        .expect("실제 layer 페인트");
    let mut reader = Reader::from_str(&svg);
    let mut pending = None;
    let mut rendered = String::new();
    let mut worst = f64::NEG_INFINITY;
    let mut title_end = None;
    loop {
        match reader.read_event().expect("SVG 파싱") {
            Event::Start(element) if element.name().as_ref() == "text" => {
                let mut x = None;
                let mut size = None;
                for attribute in element.attributes() {
                    let attribute = attribute.expect("SVG 글자 속성");
                    let value = attribute.value.as_ref();
                    match attribute.key.as_ref() {
                        "x" => x = Some(value.parse::<f64>().unwrap()),
                        "font-size" => size = Some(value.parse::<f64>().unwrap()),
                        _ => {}
                    }
                }
                pending = Some((x.expect("글자 원점"), size.expect("글자 크기")));
            }
            Event::Text(text) if pending.is_some() => {
                let text = text.as_ref();
                if text.trim().is_empty() {
                    continue;
                }
                let (x, size) = pending.unwrap();
                assert!((size - source_em).abs() <= f64::EPSILON * 128.0);
                assert_eq!(text.chars().count(), 1, "배분된 글자의 실제 페인트 원점");
                let ch = text.chars().next().unwrap();
                assert!(
                    ('가'..='힣').contains(&ch) || ch == 'ㆍ',
                    "독립 1em 글꼴 계약의 글자"
                );
                rendered.push(ch);
                if ch == '회' {
                    title_end = Some(x);
                }
                worst = worst.max((x + source_em) - right).max(left - x);
            }
            Event::End(element) if element.name().as_ref() == "text" => pending = None,
            Event::Eof => break,
            _ => {}
        }
    }
    assert_eq!(
        rendered, "분류처우위원회심의ㆍ의결",
        "가시 내용·순서·누락·중복"
    );
    assert!(worst.is_finite(), "실제 글자를 찾았다");
    // 동일 Rust builder/독립 한컴2020 Print에서 마지막 ‘회’의 원점은
    // 84.9145813pt다. 두 대조군 페이지의 Native/fresh WASM100% 및 확대
    // 판독 후 ±2px로 보호한다. 이전 말미 공백 배분은 약10.7px 짧게 끝났다.
    let print_title_end = 84.91458129882812 * 96.0 / 72.0;
    assert!(
        (title_end.expect("첫 줄의 끝 글자") - print_title_end).abs() <= 2.0,
        "양쪽 정렬이 소비한 말미 공백 때문에 가시 글자를 압축했다"
    );
    worst
}

#[test]
fn issue_5598_hanging_indent_does_not_push_text_out_of_cell() {
    let over = worst_overflow(
        &document_with_narrow_cell(HANGING_INDENT),
        include_bytes!("../fixtures/issue5598/narrow-cell-indent--7832-defined.hwpx"),
    );
    assert!(
        over <= 2.0,
        "칸 폭에 육박하는 내어쓰기 때문에 글자가 칸 밖으로 {over:.1}px 나갔다 (#5598)"
    );
}

#[test]
fn issue_5598_no_indent_case_is_unchanged() {
    let over = worst_overflow(
        &document_with_narrow_cell(0),
        include_bytes!("../fixtures/issue5598/narrow-cell-indent-0-defined.hwpx"),
    );
    assert!(
        over <= 2.0,
        "내어쓰기가 없는 문단인데 글자가 칸 밖으로 {over:.1}px 나갔다"
    );
}

//! #7445: 17쪽 전수 Native/fresh WASM 수용 후 원본 줄 소유 계약을 복구한다.
//! 시각 증적은 mydocs/pr/assets/issue7445/resolution-20261005/에 있다.
//! 절대 페이지 좌표 대신 원본 글자모양, 캡션 줄 상자와 최종 문단 소속을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue7445/80250_regulatory_analysis.hwp";

fn bytes() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("보존된 원본 HWP")
}

fn lines_for_para<'a>(node: &'a RenderNode, para_index: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(para_index))
    {
        out.push(node);
    }
    for child in &node.children {
        lines_for_para(child, para_index, out);
    }
}

fn table_for_para(node: &RenderNode, para_index: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(para_index))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_for_para(child, para_index))
}

#[test]
fn empty_caption_keeps_its_source_font_box_and_table_page_owner() {
    let input = bytes();
    let source = rhwp::parse_document(&input).expect("원본 파싱");
    let Control::Table(source_table) = &source.sections[0].paragraphs[71].controls[0] else {
        panic!("예비분석 결과표 소유 문단");
    };
    let source_caption = &source_table.caption.as_ref().expect("위쪽 캡션").paragraphs[0];
    assert!(source_caption.text.is_empty() && source_caption.line_segs.is_empty());
    let shape = source_caption.char_shapes.last().expect("종료 글자모양");
    let font_box_hu = source.doc_info.char_shapes[shape.char_shape_id as usize].base_size;
    let core = DocumentCore::from_bytes(&input).expect("문서 로드");
    assert_eq!(core.page_count(), 17, "독립 PDF와 같은 페이지 수");
    let Control::Table(table) = &core.document().sections[0].paragraphs[71].controls[0] else {
        panic!("로드 후 표 소속 유지");
    };
    let caption = &table.caption.as_ref().expect("캡션 유지").paragraphs[0];
    assert_eq!(
        caption.line_segs.len(),
        1,
        "빈 캡션도 원본 줄 상자를 소유한다"
    );
    assert_eq!(
        caption.line_segs[0].line_height, font_box_hu,
        "원본 글자모양의 em 상자"
    );
    let tree = core.build_page_render_tree(10).expect("독립 PDF 11쪽");
    let rendered_table = table_for_para(&tree.root, 71).expect("예비분석 결과표는 11쪽 소속");
    let mut title = Vec::new();
    lines_for_para(&tree.root, 71, &mut title);
    let title = title.first().expect("표 제목 유지");
    assert!(
        title.bbox.y + title.bbox.height <= rendered_table.bbox.y,
        "표는 제목 뒤에 배치한다"
    );

    // 같은 원본 전 17쪽의 Native/fresh WASM 수용 후 보완한다.
    // HMKMG의 실제 periodcentered hmtx(512/512)와 독립 Print 10쪽은
    // 글자 크기 대비 전각 전진을 입증한다. 절대 x 좌표를 고정하지 않는다.
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(9)
            .expect("점검항목 본문 10쪽의 실제 글자 원점"),
    )
    .expect("text-layout JSON");
    let mut verified_dots = 0;
    for run in layout["runs"].as_array().expect("실제 글줄") {
        let family = run["fontFamily"].as_str().unwrap_or_default();
        if family
            .split(',')
            .next()
            .unwrap_or(family)
            .trim_matches('\'')
            != "휴먼고딕"
        {
            continue;
        }
        let text: Vec<char> = run["text"].as_str().expect("글줄 내용").chars().collect();
        let font_size = run["fontSize"].as_f64().expect("원본 글자 크기");
        let origins = run["charX"].as_array().expect("글자별 원점");
        assert!(font_size > 0.0 && origins.len() > text.len());
        for (i, ch) in text.iter().enumerate() {
            if *ch != '\u{00B7}' {
                continue;
            }
            let advance = origins[i + 1].as_f64().expect("다음 글자 원점")
                - origins[i].as_f64().expect("가운데점 원점");
            assert!(
                (advance / font_size - 1.0).abs() < 0.05,
                "휴먼고딕 가운데점은 원본 프로그램의 전각을 소비한다: {}em",
                advance / font_size
            );
            verified_dots += 1;
        }
    }
    assert!(
        verified_dots > 0,
        "점검항목의 실제 가운데점 경로를 실행한다"
    );
}

#[test]
fn paragraph_end_font_is_owned_only_by_the_final_line() {
    let input = bytes();
    let source = rhwp::parse_document(&input).expect("원본 파싱");
    let para = &source.sections[0].paragraphs[118];
    let end_pos = para.char_count.checked_sub(1).expect("원본 종료 문자");
    let shape = para
        .char_shapes
        .iter()
        .rev()
        .find(|shape| shape.start_pos <= end_pos)
        .expect("종료 문자 글자모양");
    let font_hu = source.doc_info.char_shapes[shape.char_shape_id as usize].base_size;
    // 좌표 실측값을 고정하지 않고 원본 글자모양과 실제 줄 상자의 포함 관계를 대조한다.
    let source_font_box = rhwp::renderer::hwpunit_to_px(font_hu, rhwp::renderer::DEFAULT_DPI);
    let core = DocumentCore::from_bytes(&input).expect("문서 로드");
    let tree = core.build_page_render_tree(14).expect("독립 PDF 15쪽");
    let mut lines = Vec::new();
    lines_for_para(&tree.root, 118, &mut lines);
    assert_eq!(lines.len(), 2, "문단118의 두 글줄 소속 유지");
    let last = lines.last().expect("마지막 줄");
    assert!(
        last.bbox.height >= source_font_box,
        "비표시 종료 문자의 원본 글자모양도 마지막 줄 높이에 포함한다"
    );
    assert!(
        lines[0].bbox.height < last.bbox.height,
        "종료 글자모양을 앞줄까지 적용하지 않는다"
    );
    let mut next = Vec::new();
    lines_for_para(&tree.root, 119, &mut next);
    assert!(!next.is_empty(), "다음 문단도 같은 쪽에 남아야 한다");
    assert!(
        next.iter()
            .all(|line| line.bbox.y >= last.bbox.y + last.bbox.height),
        "다음 문단은 마지막 줄 뒤에 배치한다"
    );
}

#[test]
fn click_here_value_spaces_share_distribution_after_the_authored_prefix() {
    let input = bytes();
    let source = rhwp::parse_document(&input).expect("원본 파싱");
    let (index, paragraph) = source.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, para)| para.text.contains("해당없음(교통약자 등 국민생명"))
        .expect("일몰설정 누름틀의 원본 값");
    let field_index = paragraph
        .controls
        .iter()
        .position(|control| {
            matches!(control, Control::Field(field)
                if field.field_type == rhwp::model::control::FieldType::ClickHere)
        })
        .expect("누름틀 시작 소유");
    let field_start = paragraph.control_text_positions()[field_index];
    assert!(field_start > 0);
    assert!(paragraph.text.chars().take(field_start).all(|ch| ch == ' '));
    let core = DocumentCore::from_bytes(&input).expect("문서 로드");
    assert_eq!(core.page_count(), 17);
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(11)
            .expect("독립 Print 12쪽의 실제 글자 원점"),
    )
    .expect("글줄 JSON");
    let runs: Vec<_> = layout["runs"]
        .as_array()
        .expect("실제 글줄")
        .iter()
        .filter(|run| run["secIdx"] == 0 && run["paraIdx"] == index)
        .collect();
    let prefix = runs
        .iter()
        .find(|run| run["charStart"] == 0)
        .expect("필드 전 원본 들여쓰기 공백");
    let field_space = runs
        .iter()
        .find(|run| run["charStart"] == field_start)
        .expect("필드 값의 첫 공백");
    assert_eq!(
        prefix["text"].as_str().unwrap().chars().count(),
        field_start
    );
    assert_eq!(field_space["text"], " ");
    assert_eq!(prefix["fontSize"], field_space["fontSize"]);
    let natural_prefix_space = prefix["w"].as_f64().unwrap() / field_start as f64;
    assert!(
        field_space["w"].as_f64().unwrap() > natural_prefix_space + 0.1,
        "필드 전 접두 공백은 자연 폭, 값 안 공백은 독립 Print와 같은 본문 정렬 배분을 갖는다"
    );

    // 전17쪽 Native/fresh 수용 뒤12쪽은99.53545/99.53641%이며 확대 판독했다.
    // 독립 Print 첫 '해'의 trace 원점은722×0.12pt다. 허용된 ±2 CSSpx
    // 좌표 보호와 별도로 위의 실제 공백 소유 관계가 수정 전 결함을 검출한다.
    let value_byte_offset = paragraph.text.find("해당없음(교통약자").unwrap();
    let value_offset = paragraph.text[..value_byte_offset].chars().count();
    let value = runs
        .iter()
        .find(|run| {
            let start = run["charStart"].as_u64().unwrap() as usize;
            let count = run["text"].as_str().unwrap().chars().count();
            start <= value_offset && value_offset < start + count
        })
        .expect("값 본문의 실제 글줄");
    let glyph_offset = value_offset - value["charStart"].as_u64().unwrap() as usize;
    assert_eq!(
        value["text"].as_str().unwrap().chars().nth(glyph_offset),
        Some('해')
    );
    let first_glyph_x =
        value["x"].as_f64().unwrap() + value["charX"][glyph_offset].as_f64().unwrap();
    let independent_x = 722.0 * 0.12 * rhwp::renderer::DEFAULT_DPI / 72.0;
    assert!(
        (first_glyph_x - independent_x).abs() <= 2.0,
        "누름틀 값 시작: 실제{first_glyph_x}, 독립 Print{independent_x}"
    );
    let compact = |text: &str| {
        text.chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
    };
    let painted = runs
        .iter()
        .map(|run| run["text"].as_str().unwrap())
        .collect::<String>();
    assert_eq!(
        compact(&painted),
        compact(&paragraph.text),
        "필드 앞뒤 원문은 같은 쪽에 누락·중복 없이 보존된다"
    );
}

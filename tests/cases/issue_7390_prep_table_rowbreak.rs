//! #7390 / PR #7406: KoPub 줄 폭 변경 뒤 PrEP 69쪽의 다행 RowBreak 표가
//! 선행 본문과 캡션 위로 끌려 올라오지 않고 70쪽에 이어져야 한다.
//!
//! 독립 기준: `pdf/issue2006/1790387_prep_final_report-2024.pdf` (원 HWPX를
//! 한컴 2024 엔진으로 변환). 물리 69쪽은 `<표 27>` 캡션 뒤에 표가 시작하고,
//! 물리 70쪽에서 마지막 행들을 이어 그린다. 수정 전에는 38행 전체가 69쪽
//! 본문 상단 y=94.5px에 놓여 앞 글·캡션을 덮고 70쪽 후속 조각은 없다.

use std::collections::BTreeSet;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue2006/1790387_prep_final_report.hwpx";

// 2024 Print 전체 140쪽의 기존 96dpi Native/fresh WASM 최저 91.93033%를
// 확인하고 대응 review를 직접 판독했다. 절대 위치 대신 실제 소유 영역과
// 앞뒤 내용 관계를 보호한다. 원본 여백·저장 줄의 간격 계약은 유지한다.
fn find<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find(child, predicate))
}

fn text(node: &RenderNode) -> String {
    let mut value = String::new();
    line_text(node, &mut value);
    value
}

fn line<'a>(root: &'a RenderNode, needle: &str) -> &'a RenderNode {
    find(root, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(needle)
    })
    .unwrap_or_else(|| panic!("대응 줄: {needle}"))
}

fn body(root: &RenderNode) -> &RenderNode {
    find(root, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역")
}

fn owner<'a>(node: &'a RenderNode, needle: &str, cell: bool) -> Option<&'a RenderNode> {
    // 중첩 표에서는 가장 안쪽의 실제 소유 칸을 선택한다.
    node.children
        .iter()
        .find_map(|child| owner(child, needle, cell))
        .or_else(|| {
            let kind = if cell {
                matches!(node.node_type, RenderNodeType::TableCell(_))
            } else {
                matches!(node.node_type, RenderNodeType::Table(_))
            };
            (kind && text(node).contains(needle)).then_some(node)
        })
}

fn inside_y(inner: &RenderNode, outer: &RenderNode) {
    let epsilon = outer.bbox.height.max(1.0) * 1e-9;
    assert!(
        inner.bbox.y >= outer.bbox.y - epsilon
            && inner.bbox.y + inner.bbox.height <= outer.bbox.y + outer.bbox.height + epsilon,
        "내용 {:?}은 소유 영역 {:?} 안에 있어야 한다",
        inner.bbox,
        outer.bbox
    );
}

fn before(first: &RenderNode, next: &RenderNode) {
    let epsilon = first.bbox.height.max(next.bbox.height).max(1.0) * 1e-9;
    assert!(
        first.bbox.y + first.bbox.height <= next.bbox.y + epsilon,
        "앞 내용 {:?}이 뒤 내용 {:?}을 덮는다",
        first.bbox,
        next.bbox
    );
}

fn saved_distance(core: &DocumentCore, first: &RenderNode, next: &RenderNode) {
    let source = |node: &RenderNode| {
        let RenderNodeType::TextLine(data) = &node.node_type else {
            panic!("저장 본문 글줄")
        };
        assert!(
            data.caption_owner.is_none(),
            "개체 캡션의 로컬 축과 본문 축을 섞지 않는다"
        );
        let section = data.section_index.expect("구역 소유");
        let para = data.para_index.expect("문단 소유");
        let index = data.line_index.expect("저장 줄 소유") as usize;
        core.document().sections[section].paragraphs[para].line_segs[index].vertical_pos
    };
    let expected = f64::from(source(next) - source(first)) / 75.0;
    let actual = next.bbox.y - first.bbox.y;
    assert!(
        (actual - expected).abs() <= expected.abs().max(1.0) * 1e-9,
        "같은 쪽의 두 원본 줄 사이 간격 {expected}와 실제 상대 간격 {actual}이 다르다"
    );
}

#[test]
fn prep_page_76_single_cell_does_not_reserve_last_line_spacing() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(75).expect("물리 76쪽");
    let first = line(&page.root, "대상자 1 (30대 후반");
    let table = owner(&page.root, "대상자 1 (30대 후반", false).expect("심층 인터뷰 표");
    let last = line(&page.root, "대상자 9 (30대 후반");
    let following = line(&page.root, "다양한 트랜지션 과정을");
    inside_y(
        first,
        owner(&page.root, "대상자 1 (30대 후반", true).unwrap(),
    );
    inside_y(last, table);
    before(table, following);
    assert!(
        text(table).contains("성확정 수술을 하지 못했음"),
        "마지막 응답 보존"
    );
    // 원본 선언 높이는 마지막 줄 끝과 칸 여백으로 닫힌다. 후행 줄간격을 재예약하지 않는다.
    let rhwp::model::control::Control::Table(source) =
        &core.document().sections[match &table.node_type {
            RenderNodeType::Table(meta) => meta.section_index.expect("표 구역 소유"),
            _ => unreachable!(),
        }]
        .paragraphs[14]
            .controls[0]
    else {
        panic!("원본 단일 셀 표")
    };
    let declared = f64::from(source.common.height) / 75.0;
    assert!((table.bbox.height - declared).abs() <= declared * 1e-9);
}

fn target_table(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(27)
            && table.control_index == Some(0)
            && table.row_count == 38
            && table.col_count == 4
        {
            return Some(node);
        }
    }
    node.children.iter().find_map(target_table)
}

fn line_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        line_text(child, text);
    }
}

fn caption_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        line_text(node, &mut text);
        if text.contains("표 27") {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children.iter().find_map(caption_bottom)
}

fn line_top_containing(node: &RenderNode, needle: &str) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        line_text(node, &mut text);
        if text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| line_top_containing(child, needle))
}

fn visible_rows(table: &RenderNode) -> BTreeSet<u16> {
    table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TableCell(cell) => Some(cell.row),
            _ => None,
        })
        .collect()
}

fn paragraph_lines(node: &RenderNode, para_index: usize, lines: &mut Vec<String>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            let mut text = String::new();
            line_text(node, &mut text);
            lines.push(text);
        }
    }
    for child in &node.children {
        paragraph_lines(child, para_index, lines);
    }
}

fn footnote_text(node: &RenderNode) -> Option<String> {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        let mut text = String::new();
        line_text(node, &mut text);
        return Some(text);
    }
    node.children.iter().find_map(footnote_text)
}

#[test]
fn prep_kopub_paragraph_does_not_orphan_final_syllable() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(66).expect("물리 67쪽");
    let mut lines = Vec::new();
    paragraph_lines(&page.root, 14, &mut lines);
    assert_eq!(
        lines.len(),
        4,
        "한컴 2024 PDF 물리 67쪽의 문단은 네 줄: {lines:?}"
    );
    assert!(
        lines.last().is_some_and(|line| line.ends_with("하였음")),
        "마지막 음절은 네 번째 줄에 있어야 한다: {lines:?}"
    );
    let mut next_paragraph = Vec::new();
    paragraph_lines(&page.root, 16, &mut next_paragraph);
    assert!(
        next_paragraph
            .last()
            .is_some_and(|line| line.ends_with("연령을 정리하면 다음과 같음.")),
        "한컴 PDF에서는 다음 문단의 마지막 줄까지 물리 67쪽에 있다: {next_paragraph:?}"
    );
}

#[test]
fn prep_single_cell_continuation_keeps_vertical_padding_and_following_flow() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(38).expect("물리 39쪽");
    let next = core.build_page_render_tree(39).expect("물리 40쪽");
    let a1 = line(&first.root, "A1. 귀하의 출생년도를");
    let a8_tail = line(&next.root, "2) 없다");
    let following = line(&next.root, "관심집단과 대조군은");
    inside_y(
        a1,
        owner(&first.root, "A1. 귀하의 출생년도를", true).unwrap(),
    );
    inside_y(a8_tail, owner(&next.root, "2) 없다", true).unwrap());
    let table = owner(&next.root, "A9. 지난 1년간", false).expect("질문 이어받기 표");
    assert!(!text(&first.root).contains("A9. 지난 1년간"));
    before(table, following);
    assert!(!text(&next.root).contains("A1. 귀하의 출생년도를"));
}

#[test]
fn prep_intra_paragraph_cell_split_keeps_last_line_on_page_34() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(33).expect("물리 34쪽");
    let next = core.build_page_render_tree(34).expect("물리 35쪽");
    // 원본 1×1 CELL 표의 마지막 문단은 저장 두 줄의 vertpos가
    // 28200/29800HU이다. 한컴 PDF p34는 두 번째 줄을 표 하단에 두고,
    // p35는 바로 (4) Jenness 항목으로 시작한다.
    let tail = line(&first.root, "는 경향을 보임");
    inside_y(
        tail,
        owner(&first.root, "는 경향을 보임", true).expect("마지막 줄 소유 칸"),
    );
    fn find_cell_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.row_count == 1
                && table.col_count == 1
                && line_top_containing(node, "는 경향을 보임").is_some()
            {
                return Some(node);
            }
        }
        node.children.iter().find_map(find_cell_table)
    }
    fn body_bottom(node: &RenderNode) -> Option<f64> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return Some(node.bbox.y + node.bbox.height);
        }
        node.children.iter().find_map(body_bottom)
    }
    let table = find_cell_table(&first.root).expect("34쪽 단일 셀 표");
    let bottom = body_bottom(&first.root).expect("34쪽 본문 바닥");
    assert!(
        table.bbox.y + table.bbox.height <= bottom + 0.5,
        "34쪽 표 바닥 {:.2} > 본문 바닥 {bottom:.2}",
        table.bbox.y + table.bbox.height
    );
    assert!(line_top_containing(&next.root, "는 경향을 보임").is_none());
    let next_item = line(&next.root, "(4) Jenness");
    inside_y(
        next_item,
        owner(&next.root, "(4) Jenness", true).expect("이어지는 원본 칸"),
    );
}

#[test]
fn prep_population_table_keeps_last_fitting_rows_with_first_fragment() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(49).expect("물리 50쪽");
    let next = core.build_page_render_tree(50).expect("물리 51쪽");
    fn population_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(480) && table.row_count == 21 && table.col_count == 5 {
                return Some(node);
            }
        }
        node.children.iter().find_map(population_table)
    }
    let first_rows = visible_rows(population_table(&first.root).expect("50쪽 인구집단 표"));
    let next_rows = visible_rows(population_table(&next.root).expect("51쪽 인구집단 표"));
    // 한컴 2024 PDF p50은 치과의사·약사 두 행까지 놓고, p51은
    // 반복 제목행 뒤 간호사 행부터 재개한다.
    assert!(
        first_rows.contains(&11) && first_rows.contains(&12),
        "50쪽 행: {first_rows:?}"
    );
    assert!(!next_rows.contains(&11) && !next_rows.contains(&12));
    assert!(next_rows.contains(&13), "51쪽 행: {next_rows:?}");
}

#[test]
fn prep_centered_inline_table_uses_host_paragraph_margin() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(56).expect("물리 57쪽");
    fn demographic_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(535) && table.row_count == 29 && table.col_count == 2 {
                return Some(node);
            }
        }
        node.children.iter().find_map(demographic_table)
    }
    let table = demographic_table(&page.root).expect("성별 정체성 표");
    let column = find(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Column { .. })
    })
    .expect("표 소유 단");
    let doc = core.document();
    let source = &doc.sections[0].paragraphs[535];
    let shape = &doc.doc_info.para_shapes[source.para_shape_id as usize];
    assert_eq!(shape.alignment, rhwp::model::style::Alignment::Center);
    let left_gap = table.bbox.x - column.bbox.x;
    let right_gap = column.bbox.x + column.bbox.width - table.bbox.x - table.bbox.width;
    let margin_difference = f64::from(shape.margin_left - shape.margin_right) / 150.0;
    assert!(
        (left_gap - right_gap - margin_difference).abs() <= column.bbox.width * 1e-9,
        "가운데 표의 좌우 공간은 원본 문단 여백 차이를 반영한다"
    );
    inside_y(table, column);
    assert!(text(table).contains("성별 정체성") && text(table).contains("414"));
}

#[test]
fn prep_paragraph_relative_chart_image_uses_host_left_margin() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(60).expect("물리 61쪽");
    fn chart_image(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Image(image) = &node.node_type {
            if image.section_index == Some(1)
                && image.para_index == Some(7)
                && image.control_index == Some(0)
            {
                return Some(node);
            }
        }
        node.children.iter().find_map(chart_image)
    }
    let image = chart_image(&page.root).expect("성 파트너 수 분포 그림");
    let column = find(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Column { .. })
    })
    .unwrap();
    let doc = core.document();
    let host = &doc.sections[1].paragraphs[7];
    let shape = &doc.doc_info.para_shapes[host.para_shape_id as usize];
    let rhwp::model::control::Control::Picture(source) = &host.controls[0] else {
        panic!("원본 분포 그림")
    };
    let expected_offset =
        f64::from(shape.margin_left) / 150.0 + f64::from(source.common.horizontal_offset) / 75.0;
    assert!(
        (image.bbox.x - column.bbox.x - expected_offset).abs() <= column.bbox.width * 1e-9,
        "문단 기준 그림은 소유 문단의 왼쪽 여백과 개체 오프셋을 반영한다"
    );
    let caption = line(&page.root, "[그림 11]");
    let next = line(&page.root, "HIV 검사를 한 적 있는 사람은");
    let following = line(&page.root, "성연결망 분석은");
    before(image, caption);
    before(caption, next);
    before(next, following);
    saved_distance(&core, next, following);
}

#[test]
fn prep_saved_table_tail_keeps_subject_seven_on_page_81() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(80).expect("물리 81쪽");
    let next = core.build_page_render_tree(81).expect("물리 82쪽");
    // 한컴 2024 PDF는 대상자 7의 라벨·응답을 81쪽 표의 마지막에,
    // 저장 vpos가 0으로 돌아간 대상자 8은 82쪽 표 처음에 둔다.
    assert!(line_top_containing(&first.root, "대상자 7 (여, 40대)").is_some());
    assert!(line_top_containing(&first.root, "물 중단 후 금단 증상으로").is_some());
    assert!(line_top_containing(&next.root, "대상자 7 (여, 40대)").is_none());
    assert!(line_top_containing(&next.root, "대상자 8 (남, 30대)").is_some());
}

#[test]
fn prep_page_90_keeps_saved_leading_spacing_after_page_start() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(89).expect("물리 90쪽");
    let first = line(&page.root, "전파경로에 대한 응답률이 적어");
    let next = line(&page.root, "HIV 감염인 대상으로 감염 경로 확인");
    let heading = line(&page.root, "3. 생존 HIV 감염인의 인구학적 특성");
    let caption = line(&page.root, "<표 37> 2024년 HIV 내국인");
    inside_y(first, body(&page.root));
    before(first, next);
    before(next, heading);
    before(heading, caption);
    saved_distance(&core, first, next);
    saved_distance(&core, heading, caption);
    let source = &core.document().sections[3].paragraphs[189];
    let shape = &core.document().doc_info.para_shapes[source.para_shape_id as usize];
    // 원본의 앞 간격(문단모양은 2 HU 단위)과 본문 시작 사이의 관계를 보존한다.
    let expected = f64::from(shape.spacing_before) / 150.0;
    assert!((first.bbox.y - body(&page.root).bbox.y - expected).abs() <= expected.max(1.0) * 1e-9);
}

#[test]
fn prep_page_79_keeps_last_quote_before_saved_cell_reset() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let before = core.build_page_render_tree(78).expect("물리 79쪽");
    let after = core.build_page_render_tree(79).expect("물리 80쪽");
    // 한컴 2024 PDF p79는 표의 '필로폰이 제게' 한 줄을 같은 쪽에 두고,
    // p80은 저장 vpos가 0으로 되감긴 다음 문단부터 시작한다.
    assert!(
        line_top_containing(&before.root, "필로폰이 제게 잘 맞는다고").is_some(),
        "저장 프레임 마지막 응답은 79쪽에 남아야 함"
    );
    assert!(
        line_top_containing(&after.root, "필로폰이 제게 잘 맞는다고").is_none(),
        "80쪽에 마지막 응답이 중복되거나 이월되면 안 됨"
    );
    assert!(
        line_top_containing(&after.root, "채팅 어플로 사람들을 만나기도").is_some(),
        "80쪽은 저장 reset 뒤의 다음 응답에서 재개"
    );
}

#[test]
fn prep_page_92_caption_keeps_saved_negative_empty_line_advance() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(91).expect("물리 92쪽");
    let caption = line(&page.root, "[그림 14] QUANTPrEP을");
    let empty = find(&page.root, &|node| matches!(&node.node_type,
        RenderNodeType::TextLine(data) if data.section_index == Some(3) && data.para_index == Some(215)))
        .expect("그림 뒤 원본 빈 줄");
    saved_distance(&core, empty, caption);
    let source = &core.document().sections[3].paragraphs[215].line_segs[0];
    assert!(source.line_spacing < 0, "원본 음수 후행 간격");
    let expected = f64::from(source.line_height + source.line_spacing) / 75.0;
    assert!((caption.bbox.y - empty.bbox.y - expected).abs() <= expected * 1e-9);
    inside_y(caption, body(&page.root));
}

#[test]
fn prep_page_93_caption_starts_after_saved_picture_bottom() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(92).expect("물리 93쪽");
    let caption = line(&page.root, "[그림 15] QUANTPrEP을");
    let first = line(&page.root, "2024년 주민등록연앙인구를 활용하였을 때");
    let cdc = line(&page.root, "다음으로 미국 CDC에서");
    let image = find(&page.root, &|node| matches!(&node.node_type,
        RenderNodeType::Image(data) if data.para_index == Some(218) && data.control_index == Some(0)))
        .expect("그림 15 원본 그림");
    before(image, caption);
    before(caption, first);
    before(first, cdc);
    saved_distance(&core, caption, first);
    saved_distance(&core, first, cdc);
}

#[test]
fn prep_page_105_merged_budget_table_keeps_saved_row_boundaries() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(104).expect("물리 105쪽");
    fn budget_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(336) && table.row_count == 20 && table.col_count == 6 {
                return Some(node);
            }
        }
        node.children.iter().find_map(budget_table)
    }
    let table = budget_table(&page.root).expect("연구비 사용 내역 표");
    let cells: Vec<_> = table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TableCell(cell) if cell.col == 3 => Some((cell.row, child)),
            _ => None,
        })
        .collect();
    assert_eq!(
        cells.iter().map(|(row, _)| *row).collect::<Vec<_>>(),
        (0..20).collect::<Vec<_>>(),
        "금액 열의 원본 20행을 같은 쪽에 한 번씩 보존"
    );
    for (_, cell) in &cells {
        inside_y(cell, table);
    }
    for pair in cells.windows(2) {
        before(pair[0].1, pair[1].1);
        let gap = pair[1].1.bbox.y - pair[0].1.bbox.y - pair[0].1.bbox.height;
        assert!(
            gap.abs() <= table.bbox.height * 1e-9,
            "행 괘선은 빈 밴드 없이 이어진다"
        );
    }
    inside_y(table, body(&page.root));
    assert!(text(table).contains("4,447,084") && text(table).contains("95,000,000"));
    let next = core.build_page_render_tree(105).expect("연구비 뒤 다음 쪽");
    assert!(
        budget_table(&next.root).is_none(),
        "완결 연구비 표를 다음 쪽에 중복하지 않는다"
    );
}

#[test]
fn prep_footnote_four_starts_on_next_physical_page() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let marker_page = core.build_page_render_tree(66).expect("물리 67쪽");
    let note_page = core.build_page_render_tree(67).expect("물리 68쪽");
    let chart_caption = line(&note_page.root, "연도별 성주체성장애 진단 건수");
    let chart = find(&note_page.root, &|node| {
        matches!(&node.node_type,
        RenderNodeType::Image(data) if data.para_index == Some(20) && data.control_index == Some(0))
    })
    .expect("각주 앞 진단 건수 차트");
    before(chart, chart_caption);
    let notes = find(&note_page.root, &|node| {
        matches!(node.node_type, RenderNodeType::FootnoteArea)
    })
    .expect("독립 Print의 각주 영역");
    before(chart_caption, notes);
    let marker_page_notes = footnote_text(&marker_page.root).unwrap_or_default();
    let next_page_notes = footnote_text(&note_page.root).unwrap_or_default();
    assert!(
        !marker_page_notes.contains("F64 코드를 가장 처음 진단받은"),
        "한컴 PDF의 각주 4 본문은 67쪽에 없다: {marker_page_notes}"
    );
    assert!(
        next_page_notes.contains("F64 코드를 가장 처음 진단받은"),
        "한컴 PDF의 각주 4 본문은 68쪽에 있다: {next_page_notes}"
    );
    assert!(
        next_page_notes.contains("주상병 및 배제 상병"),
        "각주 5도 같은 68쪽에 남아야 한다: {next_page_notes}"
    );

    let mut lines = Vec::new();
    paragraph_lines(&note_page.root, 25, &mut lines);
    assert_eq!(lines.len(), 3, "각주 위 본문은 세 줄이어야 한다: {lines:?}");
    assert!(
        lines
            .last()
            .is_some_and(|line| line.trim_end().ends_with("105,534명이")),
        "한컴 PDF에서 각주 위 본문 세 번째 줄도 68쪽에 들어간다: {lines:?}"
    );
}

#[test]
fn prep_table_starts_after_caption_and_continues_next_page() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(68).expect("물리 69쪽");
    let next = core.build_page_render_tree(69).expect("물리 70쪽");

    let first_table = target_table(&first.root).expect("69쪽 표 27의 첫 조각");
    let caption_end = caption_bottom(&first.root).expect("69쪽 표 27 캡션");
    assert!(
        first_table.bbox.y + 0.5 >= caption_end,
        "표 첫 조각이 선행 캡션을 덮는다: 표 y={:.1}, 캡션 끝={caption_end:.1}",
        first_table.bbox.y
    );

    let first_rows = visible_rows(first_table);
    let next_table = target_table(&next.root).expect("70쪽 표 27의 후속 조각");
    let next_rows = visible_rows(next_table);
    assert!(first_rows.contains(&0), "첫 조각은 첫 행을 소유한다");
    assert!(
        !first_rows.contains(&37),
        "마지막 행은 첫 조각 밖에 있어야 한다"
    );
    assert!(
        next_rows.contains(&37),
        "다음 쪽에서 마지막 행을 이어 그린다"
    );
    assert!(
        (1..38).all(|row| first_rows.contains(&row) ^ next_rows.contains(&row)),
        "헤더 외 모든 원본 행은 두 쪽에서 정확히 한 번 소유한다: 첫={first_rows:?}, 다음={next_rows:?}"
    );
}

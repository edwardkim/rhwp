//! #8160: sample16 회귀 검사 통합. 이전 등록: tests/cases/verified_sample16_all_pages.rs
//! #8160: 새 Hancom Print와 Native/fresh 표준 WASM 전 384쪽 raw 100% 뒤 등록.
//! 세 원본 각각 64쪽의 내용·기준선·가로 원점과 중요 표·그림·산식·꼬리말의
//! 독립 px 좌표, 원본 본문/셀/글상자 주소를 함께 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use rhwp::DocumentCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Deserialize)]
struct Contract {
    sample: String,
    sample_sha256: String,
    pdf: String,
    pdf_sha256: String,
    compat2024: bool,
    dpi: u32,
    coordinate_tolerance_px: u32,
    pages: Vec<Page>,
}

#[derive(Deserialize)]
struct Page {
    page: usize,
    anchors: Vec<Anchor>,
    origin_x: f64,
    regions: Vec<Region>,
    footer: Option<Footer>,
    price_equations: Option<[[f64; 3]; 2]>,
}

#[derive(Deserialize)]
struct Footer {
    text: String,
    x: f64,
    baseline: f64,
}

#[derive(Deserialize)]
struct Region {
    kind: String,
    owner: [usize; 3],
    cell: Option<u32>,
    edges: [f64; 4],
}

#[derive(Deserialize)]
struct Anchor {
    text: String,
    section: usize,
    paragraph: usize,
    parent: usize,
    path: Vec<[usize; 3]>,
    textbox: bool,
    start: usize,
    baseline: f64,
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn runs(line: &RenderNode) -> Vec<(&RenderNode, &TextRunNode)> {
    line.children
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                Some((node, run))
            } else {
                None
            }
        })
        .collect()
}

fn lines<'a>(node: &'a RenderNode, body: bool, out: &mut Vec<&'a RenderNode>) {
    let body = body || matches!(node.node_type, RenderNodeType::Body { .. });
    if body && matches!(node.node_type, RenderNodeType::TextLine(_)) && !runs(node).is_empty() {
        out.push(node);
    }
    for child in &node.children {
        lines(child, body, out);
    }
}

fn owned(run: &TextRunNode, anchor: &Anchor) -> bool {
    if (run.section_index, run.para_index) != (Some(anchor.section), Some(anchor.paragraph)) {
        return false;
    }
    match &run.cell_context {
        None => anchor.path.is_empty() && anchor.parent == anchor.paragraph && !anchor.textbox,
        Some(context) => {
            context.parent_para_index == anchor.parent
                && context.in_textbox == anchor.textbox
                && context
                    .path
                    .iter()
                    .map(|e| [e.control_index, e.cell_index, e.cell_para_index])
                    .eq(anchor.path.iter().copied())
        }
    }
}

fn source<'a>(core: &'a DocumentCore, anchor: &Anchor) -> &'a Paragraph {
    let mut paragraph = &core.document().sections[anchor.section].paragraphs[anchor.parent];
    for &[control, cell, inner] in &anchor.path {
        paragraph = match &paragraph.controls[control] {
            Control::Table(table) => &table.cells[cell].paragraphs[inner],
            Control::Shape(shape) => {
                assert_eq!(cell, 0, "글상자 원본 주소");
                let drawing = match shape.as_ref() {
                    ShapeObject::Rectangle(shape) => &shape.drawing,
                    ShapeObject::Ellipse(shape) => &shape.drawing,
                    ShapeObject::Polygon(shape) => &shape.drawing,
                    ShapeObject::Curve(shape) => &shape.drawing,
                    _ => panic!("기준 내용의 실제 원본 글상자"),
                };
                &drawing
                    .text_box
                    .as_ref()
                    .expect("원본 글상자 내용")
                    .paragraphs[inner]
            }
            _ => panic!("기준 내용의 실제 원본 컨테이너"),
        };
    }
    paragraph
}

fn verify(fixture: &str) {
    let contract: Contract = serde_json::from_str(fixture).expect("독립 Print 계약");
    assert_eq!((contract.dpi, contract.coordinate_tolerance_px), (96, 2));
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join(&contract.sample)).unwrap();
    assert_eq!(sha256(&bytes), contract.sample_sha256);
    let pdf = std::fs::read(root.join(&contract.pdf)).unwrap();
    assert_eq!(sha256(&pdf), contract.pdf_sha256);
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(contract.compat2024);
    // 좌표의 독립 Print·전쪽 PNG를 산출한 세션과 같은 프로그램 선택이다.
    // 실제 글꼴 바이트의 공급·paint는 별도 Native/fresh WASM 검증으로 확인한다.
    // 기본 대체 환경의 가운데 정렬 폭을 원본 프로그램의 PDF 좌표로 고정하지 않는다.
    let body_program = if contract.compat2024 {
        "휴먼명조"
    } else {
        "Gulim"
    };
    core.set_font_environment(Some(
        rhwp::renderer::font_environment::FontEnvironment::from_json(
            &serde_json::json!({
                "id": "sample16-independent-print-coordinate-programs",
                "substitutions": {
                    "한양신명조": "한양신명조",
                    "휴먼명조": body_program,
                    "HCI Poppy": "HCI Poppy"
                },
                "trueTypeFaces": [body_program],
                "glyphFallbacks": { (body_program): ["Haansoft Batang"] }
            })
            .to_string(),
        )
        .unwrap(),
    ))
    .unwrap();
    assert_eq!(contract.pages.len(), 64, "독립 PDF의 전쪽 계약 누락 없음");
    assert_eq!(
        core.page_count() as usize,
        contract.pages.len(),
        "내용 보존과 함께 확인하는 쪽수"
    );

    verify_table_picture_rows(&core);
    let mut seen = BTreeSet::new();
    for (index, page) in contract.pages.iter().enumerate() {
        assert_eq!(page.page, index + 1, "전체 쪽 순서·중복·누락");
        assert!(
            !page.anchors.is_empty(),
            "표지·목차·본문·서식 모두 중요 내용 보호"
        );
        let tree = core.build_page_render_tree(index as u32).unwrap();
        verify_regions(&core, &tree.root, page);
        let mut body = Vec::new();
        lines(&tree.root, false, &mut body);
        let mut previous_baseline = f64::NEG_INFINITY;
        for (anchor_index, anchor) in page.anchors.iter().enumerate() {
            let found: Vec<_> = body
                .iter()
                .copied()
                .filter(|line| {
                    let text: String = runs(line)
                        .iter()
                        .map(|(_, run)| run.display_or_text())
                        .collect();
                    compact(&text) == anchor.text
                        && runs(line).iter().any(|(_, run)| {
                            owned(run, anchor) && run.char_start == Some(anchor.start)
                        })
                })
                .collect();
            assert_eq!(
                found.len(),
                1,
                "{} {}쪽 원문 소유·누락·중복: {}",
                contract.sample,
                page.page,
                anchor.text
            );
            let line = found[0];
            if anchor_index == 0 {
                // 선행 공백 런은 PDF의 가시 글자 원점에 포함되지 않는다.
                let (first, _) = runs(line)
                    .into_iter()
                    .find(|(_, run)| !compact(run.display_or_text()).is_empty())
                    .expect("독립 내용의 첫 가시 런");
                near(
                    first.bbox.x,
                    page.origin_x,
                    page.page,
                    "핵심 글줄 가로 원점",
                );
            }
            let original = source(&core, anchor);
            let mut prior_end = None;
            for (node, run) in runs(line) {
                assert!(owned(run, anchor), "같은 줄의 원본 본문/셀/글상자 소유");
                let start = run.char_start.expect("원본 문자 위치");
                let end = start + run.text.chars().count();
                let source_text: String = original
                    .text
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect();
                assert_eq!(run.text, source_text, "원본 런 내용·소유·문자 순서");
                if let Some(prior) = prior_end {
                    assert!(start >= prior, "문자 범위 중복·역전 없음");
                }
                prior_end = Some(end);
                let actual_baseline = node.bbox.y + run.baseline;
                assert!(
                    (actual_baseline - anchor.baseline).abs() <= 2.0,
                    "{} {}쪽 독립 Print 기준선 ±2px: {}, expected={}, actual={}",
                    contract.sample,
                    page.page,
                    anchor.text,
                    anchor.baseline,
                    actual_baseline
                );
                assert!(
                    node.bbox.x >= tree.root.bbox.x
                        && node.bbox.x + node.bbox.width <= tree.root.bbox.x + tree.root.bbox.width,
                    "원문 런의 가로 점유는 실제 쪽에 포함"
                );
            }
            assert!(
                anchor.baseline >= previous_baseline,
                "독립 Print의 위아래 내용 순서"
            );
            previous_baseline = anchor.baseline;
            assert!(
                seen.insert((
                    anchor.section,
                    anchor.parent,
                    anchor.path.clone(),
                    anchor.paragraph,
                    anchor.start
                )),
                "선정 원문을 다른 쪽에 중복 배치하지 않는다"
            );
        }
    }
}

fn near(actual: f64, expected: f64, page: usize, label: &str) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 2.0,
        "{page}쪽 {label} ±2px: expected={expected}, actual={actual}"
    );
}

fn verify_regions(core: &DocumentCore, root: &RenderNode, page: &Page) {
    let expected_count = match page.page {
        6 => 16,
        18 => 3,
        35 => 4,
        38 => 44,
        58 => 143,
        _ => 0,
    };
    assert_eq!(page.regions.len(), expected_count, "중요 셀·그림 계약 누락");
    let mut nodes = Vec::new();
    collect_picture_nodes(root, &mut nodes);
    assert_eq!(
        page.price_equations.is_some(),
        page.page == 6,
        "중요 산식 좌표 계약 누락"
    );
    if let Some(expected) = &page.price_equations {
        let mut rules = Vec::new();
        let mut owners = BTreeSet::new();
        for node in &nodes {
            let RenderNodeType::Equation(equation) = &node.node_type else {
                continue;
            };
            let owner = (
                equation.section_index.unwrap(),
                equation.para_index.unwrap(),
                equation.control_index.unwrap(),
            );
            assert!(owners.insert(owner), "산식 원본 소유 중복 없음");
            let Control::Equation(source) =
                &core.document().sections[owner.0].paragraphs[owner.1].controls[owner.2]
            else {
                panic!("원본 산식 제어")
            };
            assert_eq!(compact(&source.script), compact(&equation.script));
            assert!(source.common.treat_as_char);
            let mut fraction = Vec::new();
            fraction_rules(
                &equation.layout_box,
                node.bbox.x,
                node.bbox.y,
                &mut fraction,
            );
            assert_eq!(fraction.len(), 1, "각 산식의 한 분수선");
            let first =
                first_glyph_x(&equation.layout_box, node.bbox.x).expect("산식 첫 가시 글자");
            rules.push([first, fraction[0][1], fraction[0][2] - fraction[0][0]]);
        }
        assert_eq!(owners.len(), 2, "두 원본 산식의 누락 없음");
        rules.sort_by(|a, b| a[1].total_cmp(&b[1]));
        assert_eq!(rules.len(), expected.len(), "분수선 누락·중복 없음");
        for (actual, expected) in rules.iter().zip(expected) {
            for ((actual, expected), label) in
                actual
                    .iter()
                    .zip(expected)
                    .zip(["산식 첫 글자", "분수선 높이", "분수선 길이"])
            {
                near(*actual, *expected, page.page, label);
            }
        }
    }
    assert_eq!(
        page.footer.is_some(),
        page.page == 38,
        "중요 꼬리말 계약 누락"
    );
    if let Some(footer) = &page.footer {
        let found: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run)
                if compact(run.display_or_text()) == footer.text)
            })
            .collect();
        assert_eq!(found.len(), 1, "꼬리말 내용·누락·중복");
        let node = found[0];
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert_eq!(
            (run.section_index, run.para_index),
            (None, None),
            "본문으로 소유 이동 없음"
        );
        assert!(run.cell_context.is_none());
        near(node.bbox.x, footer.x, page.page, "꼬리말 가로 원점");
        near(
            node.bbox.y + run.baseline,
            footer.baseline,
            page.page,
            "꼬리말 기준선",
        );
    }
    let mut seen = BTreeSet::new();
    for region in &page.regions {
        assert!(seen.insert((region.kind.as_str(), region.owner, region.cell)));
        let [section, paragraph, control] = region.owner;
        let owner = (Some(section), Some(paragraph), Some(control));
        let found: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|node| match &node.node_type {
                RenderNodeType::Table(table) if region.kind == "cell" => {
                    (table.section_index, table.para_index, table.control_index) == owner
                }
                RenderNodeType::Image(image) if region.kind == "image" => {
                    (image.section_index, image.para_index, image.control_index) == owner
                }
                _ => false,
            })
            .collect();
        assert_eq!(found.len(), 1, "{}쪽 중요 개체의 단일 원본 소유", page.page);
        let source = &core.document().sections[section].paragraphs[paragraph].controls[control];
        let node = match source {
            Control::Table(table) if region.kind == "cell" => {
                let cell = region.cell.expect("독립 PDF clip과 대응한 원본 셀");
                let cells: Vec<_> = found[0]
                    .children
                    .iter()
                    .filter(|node| {
                        matches!(&node.node_type, RenderNodeType::TableCell(c)
                        if c.model_cell_index == Some(cell))
                    })
                    .collect();
                assert_eq!(cells.len(), 1, "원본 셀의 누락·중복 없음");
                let original = &table.cells[cell as usize];
                let RenderNodeType::TableCell(actual) = &cells[0].node_type else {
                    unreachable!()
                };
                assert_eq!(
                    (actual.row, actual.col, actual.row_span, actual.col_span),
                    (
                        original.row,
                        original.col,
                        original.row_span,
                        original.col_span
                    ),
                    "병합 셀의 원본 행·열·점유 관계"
                );
                cells[0]
            }
            Control::Picture(_) if region.kind == "image" => {
                assert!(region.cell.is_none());
                found[0]
            }
            _ => panic!("독립 중요 영역과 원본 개체 종류 일치"),
        };
        let bbox = node.bbox;
        let actual = [bbox.x, bbox.y, bbox.x + bbox.width, bbox.y + bbox.height];
        for ((actual, expected), edge) in
            actual
                .into_iter()
                .zip(region.edges)
                .zip(["왼쪽", "위쪽", "오른쪽", "아래쪽"])
        {
            near(actual, expected, page.page, edge);
        }
    }
}

// 원본 글꼴을 공급하지 않는 일반 검사에서도 같은 의미인 첫 가시 글자의 원점.
fn first_glyph_x(layout: &LayoutBox, x: f64) -> Option<f64> {
    let x = x + layout.x;
    match &layout.kind {
        LayoutKind::Row(children) => children.iter().find_map(|child| first_glyph_x(child, x)),
        LayoutKind::FontStyle { body, .. }
        | LayoutKind::Paren { body, .. }
        | LayoutKind::Color { body, .. } => first_glyph_x(body, x),
        LayoutKind::HancomGlyphs(run) => run.positions.first().map(|offset| x + offset),
        LayoutKind::Text(text)
        | LayoutKind::Number(text)
        | LayoutKind::Symbol(text)
        | LayoutKind::MathSymbol(text)
        | LayoutKind::Function(text)
        | LayoutKind::FontGlyph { text, .. }
            if !compact(text).is_empty() =>
        {
            Some(x)
        }
        _ => None,
    }
}

// 최종 EquationNode의 공통 배치가 발행한 분수선. PDF 기대값이나
// source script로 실제 좌표를 재계산하지 않는다.
fn fraction_rules(layout: &LayoutBox, x: f64, y: f64, out: &mut Vec<[f64; 3]>) {
    let (x, y) = (x + layout.x, y + layout.y);
    match &layout.kind {
        LayoutKind::Row(children) => {
            for child in children {
                fraction_rules(child, x, y, out);
            }
        }
        LayoutKind::Fraction {
            numer,
            denom,
            axis_height,
            rule_start,
            rule_end,
            ..
        } => {
            out.push([
                x + rule_start,
                y + layout.baseline - axis_height,
                x + rule_end,
            ]);
            fraction_rules(numer, x, y, out);
            fraction_rules(denom, x, y, out);
        }
        LayoutKind::FontStyle { body, .. }
        | LayoutKind::Paren { body, .. }
        | LayoutKind::Color { body, .. } => fraction_rules(body, x, y, out),
        _ => {}
    }
}

#[test]
fn issue_8160_hwpx_all_64_pages_preserve_independent_print_content_and_owners() {
    verify(include_str!("../fixtures/issue8160/hwpx.json"));
}

#[test]
fn issue_8160_hwp2024_all_64_pages_preserve_independent_print_content_and_owners() {
    verify(include_str!("../fixtures/issue8160/hwp2024.json"));
}

#[test]
fn issue_8160_hwp2022_all_64_pages_preserve_independent_print_content_and_owners() {
    verify(include_str!("../fixtures/issue8160/hwp2022.json"));
}

// 기존 HWPX 18쪽 보호를 전쪽 case에 병합해 세 저장본 모두에서 실행한다.
fn collect_picture_nodes<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect_picture_nodes(child, nodes);
    }
}
fn compact_picture_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}
fn picture_row_text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    collect_picture_nodes(node, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.display_or_text()),
            _ => None,
        })
        .collect()
}

fn verify_table_picture_rows(core: &DocumentCore) {
    let source = core.document();
    let title = "가. 주전산센터 목표시스템 구성(안)";
    let owners: Vec<_> = source.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            matches!(p.controls.first(),Some(Control::Table(t)) if t.cells.iter().any(|cell|
            cell.paragraphs.iter().any(|para|compact_picture_text(&para.text)==compact_picture_text(title))))
        })
        .collect();
    assert_eq!(owners.len(), 1, "원본 제목을 소유한 문단은 하나다");
    let (owner, paragraph) = owners[0];
    assert_eq!(paragraph.controls.len(), 3, "원본 표·그림·표의 세 컨트롤");
    assert!(matches!(&paragraph.controls[0],Control::Table(t) if t.common.treat_as_char));
    assert!(matches!(&paragraph.controls[1],Control::Picture(p) if p.common.treat_as_char));
    assert!(matches!(&paragraph.controls[2],Control::Table(t) if t.common.treat_as_char));
    let page = core.build_page_render_tree(17).unwrap();
    let mut nodes = Vec::new();
    collect_picture_nodes(&page.root, &mut nodes);
    let table = |control| {
        let found:Vec<_>=nodes.iter().copied().filter(|node|matches!(&node.node_type,
            RenderNodeType::Table(t) if t.section_index==Some(0)&&t.para_index==Some(owner)&&t.control_index==Some(control))).collect();
        assert_eq!(
            found.len(),
            1,
            "같은 원본 표를 중복 배치하거나 누락하면 안 된다"
        );
        found[0]
    };
    let first = table(0);
    let last = table(2);
    let pictures:Vec<_>=nodes.iter().copied().filter(|node|matches!(&node.node_type,
        RenderNodeType::Image(p) if p.section_index==Some(0)&&p.para_index==Some(owner)&&p.control_index==Some(1))).collect();
    assert_eq!(pictures.len(), 1, "그림은 원본 둘째 줄에 한 번만 배치한다");
    let picture = pictures[0];
    assert_eq!(
        compact_picture_text(&picture_row_text(first)),
        compact_picture_text(title),
        "글꼴별로 런이 나뉘어도 제목 전체를 보존한다"
    );
    assert_eq!(
        compact_picture_text(&picture_row_text(last)),
        compact_picture_text("나. 주요 과업내용")
    );
    assert!(
        first.bbox.y + first.bbox.height <= picture.bbox.y,
        "첫 표 뒤에 그림 줄이 온다"
    );
    assert!(
        picture.bbox.y + picture.bbox.height <= last.bbox.y,
        "그림 아래에 마지막 표가 온다"
    );
    assert!(
        picture.bbox.x >= page.root.bbox.x
            && picture.bbox.x + picture.bbox.width <= page.root.bbox.x + page.root.bbox.width
    );
    for (table_node, control) in [(first, 0), (last, 2)] {
        let mut title_nodes = Vec::new();
        collect_picture_nodes(table_node, &mut title_nodes);
        let mut end = 0;
        for node in title_nodes {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                continue;
            };
            let context = run
                .cell_context
                .as_ref()
                .expect("제목은 원본 표 셀에 속한다");
            assert_eq!(context.parent_para_index, owner);
            assert_eq!(context.path[0].control_index, control);
            let start = run.char_start.expect("원문 문자 시작");
            assert!(start >= end, "글꼴별 분할 후에도 문자 순서를 유지한다");
            end = start + run.text.chars().count();
        }
    }
}

//! 원본 sample16의 독립 한컴2020 Print64쪽과 C94 Native/fresh WASM64쪽
//! (최저91.52306/91.77199%)을 확인한 뒤 추가한 원본 프레임 계약.
//! 33쪽 raw99.25546/98.53604%: 중요한 표 시작만 ±2CSSpx로 보호한다.
//! C144 최신 전64쪽 최저94.38750/94.46126%,33쪽99.57555/98.83569%로
//! 재검증한 뒤 중요한 첫 글머리표와 뒤 본문 원점도 독립±2CSSpx로 보호한다.
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn original() -> HwpDocument {
    let bytes = std::fs::read("samples/hwp3-sample16.hwp").expect("원본 HWP3");
    HwpDocument::from_bytes(&bytes).expect("원본 파싱")
}

fn owned<'a>(node: &'a RenderNode, paragraph: usize, tables: bool, out: &mut Vec<&'a RenderNode>) {
    let matches = match &node.node_type {
        RenderNodeType::Table(table) if tables => table.para_index == Some(paragraph),
        RenderNodeType::TextLine(line) if !tables => {
            line.section_index == Some(0) && line.para_index == Some(paragraph)
        }
        _ => false,
    };
    if matches {
        out.push(node);
        return;
    }
    for child in &node.children {
        owned(child, paragraph, tables, out);
    }
}

#[test]
fn empty_opening_line_and_following_tables_keep_the_same_physical_page() {
    let doc = original();
    assert_eq!(doc.page_count(), 64, "독립 Print64쪽");
    let paragraphs = &doc.document().sections[0].paragraphs;
    // 원본 저장 줄·개체 순서로 대상을 확인한다. 화면 좌표로 선택하지 않는다.
    let empty = &paragraphs[535];
    assert!(empty.text.trim().is_empty() && empty.controls.is_empty());
    assert_eq!(empty.line_segs[0].vertical_pos, 0);
    assert_eq!(
        paragraphs[536].line_segs[0].vertical_pos,
        empty.line_segs[0].line_height + empty.line_segs[0].line_spacing
    );
    let Control::Table(title) = &paragraphs[536].controls[0] else {
        panic!("제목 표")
    };
    let title_text: String = title
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|para| para.text.as_str())
        .collect();
    assert!(title_text.contains("SAN"), "독립 Print의 SAN 스위치 제목");
    let previous = doc.build_page_render_tree(31).expect("32쪽");
    let page = doc.build_page_render_tree(32).expect("33쪽");
    let mut lines = Vec::new();
    owned(&previous.root, 535, false, &mut lines);
    assert!(lines.is_empty(), "빈 시작 줄이 앞 쪽에 남으면 안 된다");
    owned(&page.root, 535, false, &mut lines);
    assert_eq!(lines.len(), 1, "33쪽 빈 줄 소속·중복");
    let mut titles = Vec::new();
    let mut bodies = Vec::new();
    owned(&page.root, 536, true, &mut titles);
    owned(&page.root, 537, true, &mut bodies);
    assert_eq!(titles.len(), 1, "제목 표 소속·누락·중복");
    assert_eq!(bodies.len(), 1, "뒤 본문 표 소속·누락·중복");
    assert!(lines[0].bbox.y + lines[0].bbox.height <= titles[0].bbox.y);
    assert!(titles[0].bbox.y + titles[0].bbox.height <= bodies[0].bbox.y);
    // PDF 본문 표 윗선100.57pt를96dpi로 환산. 제목·빈 줄 점유를 함께 검사한다.
    assert!(
        (bodies[0].bbox.y - 100.57 * 4.0 / 3.0).abs() <= 2.0,
        "독립 Print 본문 표 시작 ±2CSSpx: {}",
        bodies[0].bbox.y
    );
}

#[test]
fn completed_san_table_marker_and_following_text_keep_independent_print_origins() {
    let doc = original();
    let page = doc.build_page_render_tree(32).expect("완료33쪽");
    let mut bodies = Vec::new();
    owned(&page.root, 537, true, &mut bodies);
    assert_eq!(bodies.len(), 1, "SAN 본문 표 소속·누락·중복");
    fn runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::TextRun(_)) {
            out.push(node);
        }
        for child in &node.children {
            runs(child, out);
        }
    }
    fn find_line<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut children = Vec::new();
            runs(node, &mut children);
            let text: String = children
                .iter()
                .filter_map(|node| {
                    if let RenderNodeType::TextRun(run) = &node.node_type {
                        Some(run.text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            // 화면 좌표가 아니라 원본 내용·소유 표로 대상을 선택한다.
            if text.contains("SUN") && text.contains("Solaris") && text.contains("HP-UX") {
                out.push(node);
            }
        }
        for child in &node.children {
            find_line(child, out);
        }
    }
    let mut lines = Vec::new();
    find_line(bodies[0], &mut lines);
    assert_eq!(lines.len(), 1, "중요 첫 글줄 누락·중복");
    let mut children = Vec::new();
    runs(lines[0], &mut children);
    let markers: Vec<_> = children
        .iter()
        .copied()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "◦ "))
        .collect();
    let labels: Vec<_> = children.iter().copied().filter(|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.starts_with("SUN Solaris"))
    }).collect();
    assert_eq!(markers.len(), 1, "원본 글머리표 소속·누락·중복");
    assert_eq!(labels.len(), 1, "뒤 본문 소속·누락·중복");
    let marker = markers[0];
    let label = labels[0];
    let RenderNodeType::TextRun(run) = &marker.node_type else {
        unreachable!()
    };
    assert!(run.cell_context.is_some(), "원본 표 셀의 글머리표");
    assert!(marker.bbox.x < label.bbox.x, "글머리표와 뒤 본문 읽기 순서");
    assert!(
        marker.bbox.x + marker.bbox.width <= label.bbox.x + 1e-6,
        "뒤 본문과 겹치지 않음"
    );
    // 독립 한컴2020 Print33쪽: ◦ 원점194.76/129.48pt, SUN 원점209.28pt.
    // 조판·paint가 소비하는 baseline을96dpi CSSpx에서 직접 대조한다.
    for (actual, expected) in [
        (marker.bbox.x, 194.76 * 4.0 / 3.0),
        (marker.bbox.y + run.baseline, 129.48 * 4.0 / 3.0),
        (label.bbox.x, 209.28 * 4.0 / 3.0),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 원점±2CSSpx: 실제{actual},기준{expected}"
        );
    }
}

#[test]
fn inline_equations_keep_source_size_baseline_and_single_line_ownership() {
    let doc = original();
    let paragraphs = &doc.document().sections[0].paragraphs;
    let equations: Vec<_> = paragraphs
        .iter()
        .flat_map(|para| {
            para.controls.iter().filter_map(move |control| {
                if let Control::Equation(eq) = control {
                    Some((para, eq))
                } else {
                    None
                }
            })
        })
        .collect();
    assert_eq!(equations.len(), 2);
    for (para, equation) in equations {
        assert!(equation.common.treat_as_char && equation.common.flow_with_text);
        assert_eq!(equation.font_size, 1200, "원본 스크립트 셀300×4HU");
        assert_eq!(equation.font_name, "HYhwpEQ");
        // 원본 baseline465×4HU / 상자2808HU를 백분율로 올림한 값.
        assert_eq!(equation.baseline, 67);
        assert_eq!(para.line_segs.len(), 1);
        assert!(para.line_segs[0].baseline_distance < para.line_segs[0].line_height);
        assert!(equation.script.contains("입찰가격"));
    }
}

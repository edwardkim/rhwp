//! 표 셀 안에 둔 쪽 번호 위치 컨트롤도 쪽 번호를 켠다.
//! 보도자료 제목표처럼 첫 문단 표의 셀에 쪽 번호 위치를 넣은 문서에서 쪽 번호가 빠졌다.
//! 공개 API로 생성한 동일 계약의 입력을 독립 Print와 양쪽 raw100으로 검증했다.
//! 증적: mydocs/pr/assets/pr_8031_page_number_cell/README.md
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::BoundingBox;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn fixture() -> DocumentCore {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../fixtures/pr8031_page_number_cell/pr8031-api-defined-page-number-cell.hwpx"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    core
}

fn text_runs(node: &RenderNode, out: &mut Vec<(String, BoundingBox, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((run.display_or_text().to_owned(), node.bbox, run.baseline));
    }
    for child in &node.children {
        text_runs(child, out);
    }
}

#[test]
fn printed_cell_declaration_preserves_body_and_footer_baseline() {
    let mut core = fixture();
    // 최상위 선언으로 우연히 통과하지 않도록 원본의 셀 소유를 확인한다.
    assert!(core.document().sections[0].paragraphs.iter().all(|para| {
        !para
            .controls
            .iter()
            .any(|c| matches!(c, Control::PageNumberPos(_)))
    }));
    let nested: Vec<_> = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .flat_map(|t| &t.cells)
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::PageNumberPos(_)))
        .collect();
    assert_eq!(nested.len(), 1, "원본 셀의 유일한 쪽 번호 위치 선언");
    assert_eq!(core.page_count(), 1);
    let mut runs = Vec::new();
    text_runs(&core.build_page_render_tree(0).unwrap().root, &mut runs);
    let body: Vec<_> = runs.iter().filter(|r| r.0 == "BODY").collect();
    let page_number: Vec<_> = runs
        .iter()
        .filter(|r| {
            r.0.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                == "-1-"
        })
        .collect();
    assert_eq!(body.len(), 1, "후속 본문 보존·중복 없음");
    assert_eq!(page_number.len(), 1, "쪽 번호 표시·중복 없음");
    assert!(
        page_number[0].1.y > body[0].1.y + body[0].1.height,
        "셀의 선언은 본문 뒤 꼬리말의 쪽 번호를 소유함"
    );
    // 독립 Print의 587.6424560546875pt 기준선, 96dpi CSS px로 환산한다.
    let baseline = page_number[0].1.y + page_number[0].2;
    assert!(
        (baseline - 783.5232747395834).abs() <= 2.0,
        "완료 페이지의 독립 Print 꼬리말 기준선: {baseline}"
    );
}

#[test]
fn printed_cell_declaration_reaches_legacy_pagination() {
    use rhwp::renderer::style_resolver::resolve_styles;
    use rhwp::renderer::{composer::compose_section, pagination::Paginator};
    let core = fixture();
    let doc = core.document();
    let section = &doc.sections[0];
    let (result, _) = Paginator::new(96.0).paginate(
        &section.paragraphs,
        &compose_section(section),
        &resolve_styles(&doc.doc_info, 96.0),
        &section.section_def.page_def,
        &Default::default(),
        0,
    );
    assert!(result.pages[0].page_number_pos.is_some());
    let position = result.pages[0].page_number_pos.as_ref().unwrap();
    assert_eq!(position.position, 5);
    // 이 HWPX는 pageNum pos=BOTTOM_CENTER, sideChar="-"로 저장되어 있다.
    assert_eq!(position.dash_char, '-');
}

//! [#6312] 빈 앵커 문단의 자기 줄과 각 셀의 개체 줄 소유를 분리한다.
//!
//! 공개 fixture는 원본156721992의 구조 보존 슬라이스다. BinData만8×8 PNG로
//! 바꿨으며 선언 객체 크기·저장 줄·표는 그대로다. 한컴오피스2020 저장 정보에
//! 따라 같은 입력을2020 엔진으로 변환한 세로4쪽 기준 PDF를 사용한다.
//!
//! 호스트 원본 사다리는19829−17129=2700=1500+1200HU다. 구역의0높이
//! 줄 정규화가 바꾼 vertical_pos가 아닌 source_line_seg_vertical_pos를 검사한다.
//! 본문 간격은 과거 rhwp 출력 상수가 아니라 실제 마지막 표 하단에 대해 검사한다.
//! 다른 셀의 큰 글줄은 로고 셀의 빈 앵커 줄을 별도로 쌓는 근거가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6312/fiscal_trend_float_table_anchor.hwpx");
    DocumentCore::from_bytes(&std::fs::read(path).expect("원본 공개 슬라이스")).unwrap()
}

fn first_body_line(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.para_index == Some(1) && line.line_index == Some(0))
    {
        return Some(node);
    }
    // 셀 내부의 문단 번호와 본문 문단 번호를 혼동하지 않는다.
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    node.children.iter().find_map(first_body_line)
}

fn band_table(node: &RenderNode, control: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(0) && table.control_index == Some(control))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| band_table(child, control))
}

#[test]
fn anchor_paragraph_line_advances_the_flow() {
    let core = core();
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(
        paragraphs[0].source_line_seg_vertical_pos.as_ref().unwrap()[0],
        17129
    );
    assert_eq!(
        paragraphs[1].source_line_seg_vertical_pos.as_ref().unwrap()[0],
        19829
    );
    let tree = core.build_page_render_tree(0).unwrap();
    let line = first_body_line(&tree.root).expect("첫 본문 줄의 첫 쪽 소유");
    // 독립 저장 본문 원점7085HU와 원본 문단 시작19829HU.
    let expected = (7085.0 + 19829.0) / 75.0;
    assert!(
        (line.bbox.y - expected).abs() < 0.2,
        "원본 본문 논리 시작 {expected}, 실제 {:?}",
        line.bbox
    );
    assert_eq!(core.page_count(), 4, "동일 입력의 한컴2020 기준4쪽");
}

#[test]
fn paragraph_advance_matches_its_actual_table_band() {
    let core = core();
    let tree = core.build_page_render_tree(0).unwrap();
    let line = first_body_line(&tree.root).unwrap();
    let table = band_table(&tree.root, 5).expect("마지막 제목 표");
    // 자기 줄2700HU와 마지막 표 아래 바깥 여백283HU를 한 번 계상한다.
    let expected_gap = (2700.0 + 283.0) / 75.0;
    assert!(
        (line.bbox.y - table.bbox.y - table.bbox.height - expected_gap).abs() < 0.1,
        "실제 표 하단과 본문 간격: 표 {:?}, 줄 {:?}",
        table.bbox,
        line.bbox
    );
}

#[test]
fn object_anchor_line_is_not_stacked_when_a_sibling_cell_expands_the_row() {
    let core = core();
    let tree = core.build_page_render_tree(0).unwrap();
    let table = band_table(&tree.root, 3).expect("로고·제목·그림의 머리 표");
    // 오른쪽 저장 줄3194HU+실효 안 여백282HU=3476HU.
    // PDF 실제 괘선 높이46.1907px가 같은46.35px 행을 독립 증언한다.
    assert!(
        (table.bbox.height - 3476.0 / 75.0).abs() < 0.1,
        "다른 셀의 확장은 로고 앵커 줄 중복 계상의 근거가 아니다: {:?}",
        table.bbox
    );
    assert!(
        (table.bbox.height - 46.1907).abs() < 0.3,
        "한컴2020 머리 표의 실제 높이: {:?}",
        table.bbox
    );
}

/// 수동 IR의 셀 저장 순서 변형이다. 공간의 소유는 행/열 위치를 따르며
/// 직렬화 배열의 순서로 다른 셀의 저장 줄을 소비하지 않는다.
#[test]
fn anchor_ownership_follows_the_cell_after_serialization_order_changes() {
    let mut core = core();
    let mut document = core.document().clone();
    let rhwp::model::control::Control::Table(table) =
        &mut document.sections[0].paragraphs[0].controls[3]
    else {
        panic!("머리 표");
    };
    table.cells.reverse();
    table.rebuild_grid();
    core.set_document(document);
    let tree = core.build_page_render_tree(0).unwrap();
    let table = band_table(&tree.root, 3).unwrap();
    assert!(
        (table.bbox.height - 3476.0 / 75.0).abs() < 0.1,
        "셀 배열 순서로 빈 줄 소유가 바뀌지 않는다: {:?}",
        table.bbox
    );
    let line = first_body_line(&tree.root).unwrap();
    assert!((line.bbox.y - (7085.0 + 19829.0) / 75.0).abs() < 0.2);
    assert_eq!(core.page_count(), 4);
}

/// 같은 셀 소유 규칙이 적용되는 기존 그림5의 실제 최종 높이·캡션 대조다.
/// 전체11쪽의 다른 배치 차이를 이 정상 대조의 통과로 면제하지 않는다.
#[test]
fn liver_picture_row_and_caption_keep_the_independent_pdf_frame() {
    fn host_table(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(237) && table.control_index == Some(0))
        {
            return Some(node);
        }
        node.children.iter().find_map(host_table)
    }
    fn caption_line(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && node.children.iter().any(|child| {
                matches!(&child.node_type,
                RenderNodeType::TextRun(run) if run.text.contains("WHO"))
            })
        {
            return Some(node);
        }
        node.children.iter().find_map(caption_line)
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx",
    );
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let tree = core.build_page_render_tree(10).unwrap();
    let table = host_table(&tree.root).expect("11쪽 그림5 표");
    let picture_cells: Vec<_> = table
        .children
        .iter()
        .filter(|child| {
            matches!(
        &child.node_type, RenderNodeType::TableCell(cell) if cell.row == 0)
        })
        .collect();
    assert_eq!(picture_cells.len(), 2);
    // 큰 그림13135HU와 실효 셀 여백282HU가 공유 행의 최소 점유를 정한다.
    for cell in picture_cells {
        assert!(
            (cell.bbox.height - (13135.0 + 282.0) / 75.0).abs() < 0.1,
            "다른 셀의 빈 앵커 줄을 그림 위에 더하지 않는다: {:?}",
            cell.bbox
        );
    }
    let caption = caption_line(table).expect("그림5 캡션 보존");
    // 원본 한컴2024 PDF11쪽 가시 글자 상단267.621053px.
    assert!(
        (caption.bbox.y - 267.621053).abs() < 0.5,
        "독립 그림5 캡션 위치: {:?}",
        caption.bbox
    );
}

/// 실제 빈 줄의 점유 끝과 저장 간격을 함께 검사한다. 빈 문자열도 공간을 소유한다.
#[test]
fn blank_line_spent_spacing_is_not_added_again_to_the_lazy_origin() {
    fn find_line(node: &RenderNode, para: usize) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(para) && line.line_index == Some(0))
        {
            return Some(node);
        }
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            return None;
        }
        node.children
            .iter()
            .find_map(|child| find_line(child, para))
    }
    let core = core();
    let tree = core.build_page_render_tree(2).unwrap();
    let blank = find_line(&tree.root, 39).expect("첫 표 다음 빈 줄");
    let body = find_line(&tree.root, 40).expect("빈 줄 다음 채무 본문");
    let later = find_line(&tree.root, 46).expect("두 번째 표 뒤 국고채 본문");
    // 독립 원본의 쪽 시작과 저장 문단 좌표. PDF 가시 글자 시작도457.9773px다.
    assert!((blank.bbox.y - (7085.0 + 25155.0) / 75.0).abs() < 0.1);
    assert!((blank.bbox.height - 1400.0 / 75.0).abs() < 0.1);
    assert!(
        (body.bbox.y - (7085.0 + 27327.0) / 75.0).abs() < 0.1,
        "빈 줄이 소비한772HU를 지연 기준에 다시 더하지 않는다: {:?}",
        body.bbox
    );
    assert!((body.bbox.y - blank.bbox.y - blank.bbox.height - 772.0 / 75.0).abs() < 0.1);
    assert!((later.bbox.y - (7085.0 + 49846.0) / 75.0).abs() < 0.1);
    assert_eq!(core.page_count(), 4);
}

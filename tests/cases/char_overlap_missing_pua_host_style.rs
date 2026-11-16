//! 관계형: 별도 글자모양 ID가 없는 PUA 글자겹침은 원문 제어 슬롯의 서식을 상속한다.
//! 주석 갱신: 2026-11-15 03:01:22 KST. 신규 파일이며 직전 파일 변경 커밋은 없다.
//! 2026-11-15, compat2022·96DPI·독립 한컴 Print·PDFium·보정 전2px raw.
//! issue2007_nested_cell_pagination_42065.hwp 전17쪽: 보정 전 Native 최저98.30%,
//! fresh WASM 최저98.36%; 대상12쪽 Native99.86%, fresh WASM99.83%.
//! 원본·PDF SHA와 제어 슬롯·독립 전진 근거: mydocs/pr/assets/issue_9060/missing-pua-overlap-style-analysis.md.
//! 실제 문서의 절대 좌표를 고정하지 않으며 설치 글꼴 없이 원본 모양·내용·인접 관계를 검사한다.

use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode},
    DocumentCore,
};

fn marker_control(paragraphs: &mut [Paragraph]) -> bool {
    for paragraph in paragraphs {
        for control in &mut paragraph.controls {
            match control {
                Control::CharOverlap(overlap) if overlap.chars.contains(&'\u{F02B3}') => {
                    assert!(
                        overlap.char_shape_ids.is_empty(),
                        "실제 원본에는 내부 서식 ID가 없다"
                    );
                    overlap.char_shape_ids.push(0);
                    return true;
                }
                Control::Table(table) => {
                    for cell in &mut table.cells {
                        if marker_control(&mut cell.paragraphs) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn run<'a>(node: &'a RenderNode, text: &str) -> Option<(&'a RenderNode, &'a TextRunNode)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(89) && run.text.contains(text) {
            return Some((node, run));
        }
    }
    node.children.iter().find_map(|child| run(child, text))
}

fn core() -> DocumentCore {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}

#[test]
fn missing_pua_overlap_reference_keeps_host_size_and_following_title_origin() {
    let core = core();
    let tree = core.build_page_render_tree(11).unwrap();
    let (marker_node, marker) = run(&tree.root, "\u{F02B3}").expect("원본 겹침 번호");
    let (title_node, title) = run(&tree.root, "중앙선거관리위원회").expect("바로 뒤 제목");
    assert_eq!(
        marker.style.font_size, title.style.font_size,
        "내부 ID 누락은 기본 0번을 뜻하지 않는다"
    );
    assert_eq!(
        marker.char_shape_id, title.char_shape_id,
        "같은 원문 글자모양 106의 소유"
    );
    assert!((marker_node.bbox.width - marker.style.font_size).abs() < 1e-6);
    assert!(
        (title_node.bbox.x - (marker_node.bbox.x + marker_node.bbox.width)).abs() < 1e-6,
        "제어 한 칸 뒤의 제목 순서·원점"
    );
    assert_eq!(
        title.text, " 중앙선거관리위원회",
        "제어 슬롯 뒤 제목의 원문 보존"
    );
}

#[test]
fn explicit_zero_overlap_reference_is_preserved() {
    let mut core = core();
    let mut document = core.document().clone();
    assert!(marker_control(&mut document.sections[0].paragraphs));
    core.set_document(document);
    let tree = core.build_page_render_tree(11).unwrap();
    let (_, marker) = run(&tree.root, "\u{F02B3}").unwrap();
    let (_, title) = run(&tree.root, "중앙선거관리위원회").unwrap();
    assert_eq!(
        marker.char_shape_id,
        Some(0),
        "명시적 0번은 누락과 구분한다"
    );
    assert!(marker.style.font_size < title.style.font_size);
}

//! 관계형: 재조판하는 서식의 원문 보존과 독립 Print의 어절별 줄 소유를 확인한다.
//! 주석 갱신: 2026-11-10T02:09:25+09:00 (Asia/Seoul).
//! 1쪽은 독립 Print의 구조 전제이며 px 좌표를 고정하지 않는다.
//! 최소 글꼴 전1쪽 Native 97.75797%, fresh WASM 97.78365%에서 발견한 잔여다.
//! 근거: mydocs/pr/assets/issue8856/stage3-legacy-shadow/validation.json.
//! 같은 원본의 한컴 2024 재인쇄와 저장 줄만 바꾼 대조군으로 줄 경계를 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const SOURCE: &[u8] =
    include_bytes!("../../samples/issue8639/policy_research_review_request-trackchange0.hwpx");
const MARKER: &str = "근무성적평정";

fn paragraph(paragraphs: &[Paragraph]) -> &Paragraph {
    for p in paragraphs {
        if p.text.contains(MARKER) {
            return p;
        }
        for control in &p.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    if cell.paragraphs.iter().any(|p| p.text.contains(MARKER)) {
                        return paragraph(&cell.paragraphs);
                    }
                }
            }
        }
    }
    panic!("대상 문단을 찾을 수 없다");
}

fn collect_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, text);
    }
}

fn collect_lines(node: &RenderNode, lines: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        collect_text(node, &mut text);
        lines.push(text);
    }
    for child in &node.children {
        collect_lines(child, lines);
    }
}

fn ink(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn trailing_fallback_symbol_does_not_freeze_the_preceding_verified_text() {
    let core = DocumentCore::from_bytes(SOURCE).unwrap();
    assert_eq!(core.document().doc_info.track_change_flags() & 0x08, 0);
    assert_eq!(core.page_count(), 1);
    let source = paragraph(&core.document().sections[0].paragraphs);
    assert!(
        source.text.contains('․'),
        "대체 프로그램이 필요한 원문 기호 유지"
    );
    let tree = core.build_page_render_tree(0).unwrap();
    let mut lines = Vec::new();
    collect_lines(&tree.root, &mut lines);
    let row = lines.iter().position(|line| line.contains(MARKER)).unwrap();
    let first = ink(&lines[row]);
    let second = ink(&lines[row + 1]);
    assert_eq!(
        format!("{first}{second}"),
        ink(&source.text),
        "원문 누락·중복이 없다"
    );
    assert!(
        first.ends_with("포상금") && second.starts_with("지급("),
        "독립 Print는 포상금을 앞줄에, 지급을 다음 줄에 둔다: first={first:?}, second={second:?}"
    );
}

fn paragraph_mut(paragraphs: &mut [Paragraph]) -> Option<&mut Paragraph> {
    for p in paragraphs {
        if p.text.contains(MARKER) {
            return Some(p);
        }
        for control in &mut p.controls {
            if let Control::Table(table) = control {
                for cell in &mut table.cells {
                    if let Some(found) = paragraph_mut(&mut cell.paragraphs) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

fn assert_stored_rows_preserved(core: &DocumentCore) {
    assert_eq!(core.page_count(), 1);
    let source = paragraph(&core.document().sections[0].paragraphs);
    let chars: Vec<_> = source.text.chars().collect();
    let cut = source.line_seg_text_start(1) as usize;
    let expected_first = ink(&chars[..cut].iter().collect::<String>());
    let expected_second = ink(&chars[cut..].iter().collect::<String>());
    let mut lines = Vec::new();
    collect_lines(&core.build_page_render_tree(0).unwrap().root, &mut lines);
    let row = lines.iter().position(|line| line.contains(MARKER)).unwrap();
    assert_eq!(ink(&lines[row]), expected_first, "원본 첫 줄 경계 유지");
    assert_eq!(
        ink(&lines[row + 1]),
        expected_second,
        "원본 다음 줄 소유 유지"
    );
}

#[test]
fn stored_row_mode_keeps_its_own_boundary_with_the_same_fallback_symbol() {
    let mut core = DocumentCore::from_bytes(SOURCE).unwrap();
    let mut doc = core.document().clone();
    let record = doc
        .doc_info
        .extra_records
        .iter_mut()
        .find(|record| record.tag_id == rhwp::parser::tags::HWPTAG_TRACKCHANGE)
        .unwrap();
    record.data[..4].copy_from_slice(&8_u32.to_le_bytes());
    core.set_document(doc);
    assert_ne!(core.document().doc_info.track_change_flags() & 0x08, 0);
    assert_stored_rows_preserved(&core);
}

#[test]
fn an_unverified_symbol_does_not_certify_the_whole_paragraph() {
    let mut core = DocumentCore::from_bytes(SOURCE).unwrap();
    let mut doc = core.document().clone();
    let p = paragraph_mut(&mut doc.sections[0].paragraphs).unwrap();
    p.text = p.text.replace('․', "\u{0378}");
    core.set_document(doc);
    assert_eq!(core.document().doc_info.track_change_flags() & 0x08, 0);
    assert_stored_rows_preserved(&core);
}

//! 관계형·고정형: 변경 추적 설정 0x08 이 있는 문서는 가로 상자가 같은 저장 줄 경계를 그대로 쓰고
//! (관계형: 렌더 줄 경계 = 저장 LineSeg 경계, 내용 보존), 0x08 이 없는 같은 문서는 원본 글꼴로 다시
//! 조판한다(고정형: 한컴 Print 의 첫 줄 끝 어절). 두 입력 모두 전체 쪽수 1쪽은 한컴 Print 값(고정형).
//! 주석 갱신: 2026-11-06T10:34:27+09:00 (새 파일).
//! 측정: 2026-11-06, `scripts/visual_sweep.py --silhouette-only --dpi 96`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣), 글꼴은 PDF 내장과 같은
//! KoPub Dotum Light·Haansoft Batang·Malgun Gothic.
//! - `policy_research_review_request.hwp`(flags 56): 기준 `pdf/issue8639/policy_research_review_request-print-2020.pdf`
//!   (Hwp 2020 Print, `--compat 2022`) p1 Native 99.94% / fresh WASM 99.96%, 2024 Print
//!   (`…-print-2024.pdf`, `--compat 2024`) p1 99.94% / 99.96%. 전체 1쪽 = 한컴 1쪽, 전쪽 최저 같은 값.
//!   이 수정 전 head(원본 프로그램 재조판이 켜짐)는 2020 기준 97.33% / 97.41%.
//! - `policy_research_review_request-trackchange0.hwpx`(flags 0 대조군): 기준
//!   `pdf/issue8639/policy_research_review_request-trackchange0-print-2024.pdf`(Hancom 13.0.0.3901)
//!   p1 Native 97.73% / fresh WASM 97.82% (devel 95.21% / 95.40%). 전체 1쪽 = 한컴 1쪽.
//!
//! 증거 `mydocs/pr/assets/issue_8639/trackchange/fixture/`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::PathBuf;

/// 정책연구과제심의신청서(10k `6540000-202500141`). 한/글 2020 저장본, flags 56.
const SOURCE: &str = "samples/issue8639/policy_research_review_request.hwp";
/// 위 원본의 HWPX 변환본에서 `trackchageConfig flags` 만 0 으로 바꾼 대조군
/// (`mydocs/tech/investigations/issue-8639/probes/make_trackchange0_variant.py`).
const TRACKCHANGE0: &str = "samples/issue8639/policy_research_review_request-trackchange0.hwpx";
/// 두 한컴 Print(2020·2024)와 대조군 Print 의 전체 쪽수.
const PRINT_PAGES: u32 = 1;
/// 대상 문단을 고르는 원문 조각.
const MARKER: &str = "공정성과 형평성 확보를 위해";

fn read(path: &str) -> Vec<u8> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path)
}

fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 표 셀 안에서 원문 조각을 담은 문단.
fn find_paragraph<'a>(paragraphs: &'a [Paragraph], marker: &str) -> Option<&'a Paragraph> {
    for paragraph in paragraphs {
        if paragraph.text.contains(marker) {
            return Some(paragraph);
        }
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    if let Some(found) = find_paragraph(&cell.paragraphs, marker) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

fn line_texts(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        collect_runs(node, &mut text);
        out.push(text);
    }
    for child in &node.children {
        line_texts(child, out);
    }
}

fn collect_runs(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        collect_runs(child, text);
    }
}

/// 대상 문단이 그린 두 글줄(잉크 문자만).
fn rendered_rows(bytes: &[u8]) -> (String, String) {
    let core = DocumentCore::from_bytes(bytes).expect("문서 로드");
    assert_eq!(core.page_count(), PRINT_PAGES, "한컴 Print 의 전체 쪽수");
    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut lines = Vec::new();
    line_texts(&tree.root, &mut lines);
    let first = lines
        .iter()
        .position(|line| line.contains(MARKER))
        .expect("대상 문단의 첫 줄");
    (ink(&lines[first]), ink(&lines[first + 1]))
}

#[test]
fn trackchange_08_keeps_the_stored_row_boundary() {
    let bytes = read(SOURCE);
    let document = rhwp::parse_document(&bytes).expect("원본 파싱");
    assert_ne!(
        document.doc_info.track_change_flags() & 0x08,
        0,
        "전제: 원본 변경 추적 설정에 0x08 이 있다"
    );
    let paragraph = find_paragraph(&document.sections[0].paragraphs, MARKER).expect("대상 문단");
    assert_eq!(paragraph.line_segs.len(), 2, "전제: 저장 줄 두 개");
    let chars: Vec<char> = paragraph.text.chars().collect();
    let stored = paragraph.line_seg_text_start(1) as usize;
    let stored_first = ink(&chars[..stored].iter().collect::<String>());
    let stored_second = ink(&chars[stored..].iter().collect::<String>());

    let (first, second) = rendered_rows(&bytes);
    // 한/글 2020·2024 Print 는 저장 줄 그대로 「…성격」/「평가를…」로 끊는다.
    assert_eq!(first, stored_first, "첫 줄은 저장 줄 경계에서 끝난다");
    assert_eq!(second, stored_second, "둘째 줄은 저장 줄 경계에서 시작한다");
    assert!(first.ends_with("성격") && second.starts_with("평가를"));
    assert_eq!(
        format!("{first}{second}"),
        ink(&paragraph.text),
        "두 줄이 문단 내용을 순서대로 모두 담는다"
    );
}

#[test]
fn trackchange_without_08_recomposes_with_the_source_font() {
    let bytes = read(TRACKCHANGE0);
    let document = rhwp::parse_document(&bytes).expect("대조군 파싱");
    assert_eq!(
        document.doc_info.track_change_flags() & 0x08,
        0,
        "전제: 대조군 변경 추적 설정에 0x08 이 없다"
    );
    let paragraph = find_paragraph(&document.sections[0].paragraphs, MARKER).expect("대상 문단");
    let chars: Vec<char> = paragraph.text.chars().collect();
    let stored = paragraph.line_seg_text_start(1) as usize;
    let stored_first = ink(&chars[..stored].iter().collect::<String>());

    let (first, second) = rendered_rows(&bytes);
    // 대조군 한/글 2024 Print 는 원본 KoPub 으로 다시 조판해 「평가를」까지 첫 줄에 담는다.
    assert_ne!(first, stored_first, "저장 줄 경계를 쓰지 않는다");
    assert!(
        first.ends_with("성격평가를"),
        "첫 줄 끝은 한컴 Print 와 같은 「평가를」: {first}"
    );
    assert!(second.starts_with("실시하여"), "둘째 줄 시작: {second}");
    assert_eq!(
        format!("{first}{second}"),
        ink(&paragraph.text),
        "두 줄이 문단 내용을 순서대로 모두 담는다"
    );
}

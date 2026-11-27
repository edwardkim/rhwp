//! 관계형 — [#9426] 줄 끝 소프트 하이픈은 그 저장 줄의 auto-hyphenation 표지(LineSeg bit 19)가 있을 때만 보인다.
//! 주석 갱신: 2026-11-27 23:25 KST (직전 파일 커밋 9a81140ebf, 2026-11-27 22:03 KST — #9441 위 리베이스)
//!
//! 한/글은 하이픈(HWP5 코드 24, HWP3 특수 문자 24) 자리에서 나눈 세그먼트에만 bit 19 를 세우고,
//! 그 비트가 있는 줄 끝에서만 `-` 를 그린다. 근거(모두 이 저장소 밖 실물 대조, 측정 2026-11-27):
//! - 한/글 2024(13.0.0.4225)가 HWP3 원본 「독일의 법령체계와 입법심사기준」(264쪽)을 HWP5 로 저장한 본은
//!   8,186개 세그먼트 중 bit 19 가 선 111개와 끝 글자가 코드 24 인 111개가 정확히 같다.
//! - 같은 원본을 rhwp 가 변환한 HWP5(비트 없음)를 한/글 2020(11.0.0.9172)·2024 로 Print 하면 같은 줄
//!   나눔에서 하이픈을 그리지 않고, bit 19 를 세운 변환본은 한/글 HWP3 Print 와 같게 하이픈을 그린다.
//! - 저장소 fixture `samples/issue8482/germany_law_ch3_resaved2020.hwp` 와 한/글 2020 Print
//!   `pdf/issue8482/germany_law_ch3-2020.pdf`: bit 19 가 선 각주 줄은 `Demo-`·`Gesetz-` 로, 비트 없는 본문
//!   9쪽 줄은 `inneres Ge` 로 끝난다(하이픈 없음). 같은 fixture 를 한/글 2024(13.0.0.4225) Print 로
//!   뽑아도 같다(각주 4줄 하이픈, 본문 `inneres Ge` 하이픈 없음) — 판본 차이가 아니라 표지 차이다.
//!
//! 측정(2026-11-27, PR #9441 head ad5db4fb45 위 이 수정, `scripts/visual_sweep.py --silhouette-only`
//! silhouette_raw_match_percent, 96 DPI, compat 2022, 11쪽 = 한/글 11쪽):
//! - 기준 `pdf/issue8482/germany_law_ch3-2020.pdf`(한/글 2020 Print): Native 전쪽 최저 99.12(11쪽), 9쪽 99.64;
//!   fresh WASM 전쪽 최저 98.99(11쪽), 9쪽 99.27. (#9441 head 9쪽: Native 99.22, WASM 98.86)
//! - 기준 한/글 2024 Print(13.0.0.4225, sha256 4e5ea22c…): Native 최저 98.87(11쪽), 9쪽 99.54;
//!   fresh WASM 최저 98.95(11쪽), 9쪽 99.22.
//! - 9쪽 `(inneres Ge` 줄 끝은 한/글·Native·WASM 셋 다 하이픈 없음(확대 판독).
//! - 전쪽 TSV: `mydocs/pr/assets/issue_9426/hyphen/{native,wasm}/`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const CH3: &str = "samples/issue8482/germany_law_ch3_resaved2020.hwp";
const CH3_HANCOM_PAGES: u32 = 11;

/// HWP3 특수 문자 하이픈(§10.18 표 59): 코드·너비·코드 세 hchar.
const HWP3_HYPHEN: [u16; 3] = [24, 125, 24];

/// 「ABCD‐」「EF‐GH」「IJKL」 세 저장 줄의 HWP3 문서. 첫 줄은 하이픈 자리에서 나뉘고,
/// 둘째 줄의 하이픈은 줄 중간이다.
fn hwp3_document_bytes() -> Vec<u8> {
    let mut hchars: Vec<u16> = "ABCD".encode_utf16().collect();
    hchars.extend(HWP3_HYPHEN);
    let second_line = hchars.len() as u16;
    hchars.extend("EF".encode_utf16());
    hchars.extend(HWP3_HYPHEN);
    hchars.extend("GH".encode_utf16());
    let third_line = hchars.len() as u16;
    hchars.extend("IJKL".encode_utf16());
    let line_starts = [0, second_line, third_line];

    let mut bytes = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05".to_vec();
    bytes.extend_from_slice(&[0; 128 + 1008]);
    for _ in 0..7 {
        bytes.extend_from_slice(&1u16.to_le_bytes());
        let mut name = [0; 40];
        name[..4].copy_from_slice(&[0xB9, 0xD9, 0xC5, 0xC1]); // 바탕
        bytes.extend_from_slice(&name);
    }
    bytes.extend_from_slice(&0u16.to_le_bytes()); // 스타일 없음
    bytes.push(0); // 이전 문단 모양을 상속하지 않음
    bytes.extend_from_slice(&(hchars.len() as u16 + 1).to_le_bytes());
    bytes.extend_from_slice(&(line_starts.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&[0; 7]);
    let mut char_shape = [0; 31];
    char_shape[..2].copy_from_slice(&250u16.to_le_bytes());
    char_shape[9..16].fill(100);
    bytes.extend_from_slice(&char_shape);
    let mut para_shape = [0; 187];
    para_shape[6..8].copy_from_slice(&160u16.to_le_bytes());
    para_shape[172] = 1; // 한 단
    bytes.extend_from_slice(&para_shape);
    for (index, start) in line_starts.iter().enumerate() {
        for value in [*start, 0, 250, 100 + index as u16 * 400, 0, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    for hchar in hchars.into_iter().chain(std::iter::once(13)) {
        bytes.extend_from_slice(&hchar.to_le_bytes());
    }
    bytes.extend_from_slice(&[0; 43]); // 문단 목록 끝
    bytes
}

/// 세그먼트 `index` 의 마지막 글자가 소프트 하이픈이고 다음 세그먼트가 이어지는가.
fn line_ends_at_soft_hyphen(paragraph: &Paragraph, index: usize) -> bool {
    let Some(next) = paragraph.line_segs.get(index + 1) else {
        return false;
    };
    let line = &paragraph.line_segs[index];
    paragraph
        .char_offsets
        .iter()
        .zip(paragraph.text.chars())
        .filter(|(offset, _)| **offset >= line.text_start && **offset < next.text_start)
        .last()
        .is_some_and(|(_, character)| character == '\u{00AD}')
}

fn hyphenation_marks(paragraph: &Paragraph) -> Vec<bool> {
    paragraph
        .line_segs
        .iter()
        .map(|line| line.tag & LineSeg::TAG_AUTO_HYPHENATION != 0)
        .collect()
}

fn assert_marks_follow_line_breaks(paragraph: &Paragraph, context: &str) {
    let ends: Vec<bool> = (0..paragraph.line_segs.len())
        .map(|index| line_ends_at_soft_hyphen(paragraph, index))
        .collect();
    assert_eq!(
        hyphenation_marks(paragraph),
        ends,
        "{context}: bit 19 는 소프트 하이픈에서 나뉜 세그먼트에만 선다"
    );
}

#[test]
fn hwp3_import_marks_lines_broken_at_a_hyphen_and_hwp5_keeps_the_mark() {
    let mut core = DocumentCore::from_bytes(&hwp3_document_bytes()).expect("합성 HWP3");
    let paragraph = &core.document().sections[0].paragraphs[0];
    assert_eq!(
        paragraph.text, "ABCD\u{00AD}EF\u{00AD}GHIJKL",
        "하이픈은 제어 표기"
    );
    assert_eq!(paragraph.line_segs.len(), 3, "저장 줄 셋");
    assert_eq!(
        hyphenation_marks(paragraph),
        [true, false, false],
        "하이픈에서 나뉜 첫 줄만 표지, 줄 중간 하이픈·마지막 줄은 아님"
    );
    assert_marks_follow_line_breaks(paragraph, "HWP3 가져오기");

    let hwp5 = core.export_hwp_native().expect("HWP5 저장");
    let reopened = DocumentCore::from_bytes(&hwp5).expect("HWP5 다시 열기");
    assert_eq!(
        hyphenation_marks(&reopened.document().sections[0].paragraphs[0]),
        [true, false, false],
        "HWP5 저장본이 표지를 싣는다 — 없으면 한/글은 하이픈을 지운다"
    );

    // 반례: 줄을 다시 나누면 표지는 견본 줄에서 복사되지 않고 새 경계를 따른다.
    core.insert_text_native(0, 0, 0, "XY ")
        .expect("문단 앞 입력");
    assert_marks_follow_line_breaks(&core.document().sections[0].paragraphs[0], "재조판");
}

fn open_ch3() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CH3);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {CH3}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {CH3}: {e:?}"));
    assert_eq!(core.page_count(), CH3_HANCOM_PAGES, "한컴 PDF 의 전체 쪽수");
    core
}

fn line_text(line: &RenderNode) -> String {
    let mut text = String::new();
    collect_runs(line, &mut text);
    text
}

fn collect_runs(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        // 그리기는 표시 문자열을 쓴다(모델 글자 U+00AD 는 `text` 에 남는다).
        text.push_str(run.display_text.as_deref().unwrap_or(&run.text));
    }
    for child in &node.children {
        collect_runs(child, text);
    }
}

fn find_line(node: &RenderNode, needle: &str) -> Option<String> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text = line_text(node);
        return text.contains(needle).then_some(text);
    }
    node.children
        .iter()
        .find_map(|child| find_line(child, needle))
}

/// 표지 비트를 바꾼 뒤 그 쪽에서 `needle` 을 담은 글줄의 글을 돌려준다.
fn rendered_line(core: &mut DocumentCore, page: u32, needle: &str) -> String {
    let tree = core.build_page_render_tree(page).unwrap();
    find_line(&tree.root, needle).unwrap_or_else(|| panic!("{}쪽 「{needle}」 글줄", page + 1))
}

#[test]
fn stored_line_end_hyphen_is_drawn_only_with_the_hyphenation_mark() {
    let mut core = open_ch3();
    // 9쪽 본문(구역 0 문단 23): `…(inneres Ge‐setz‐gebungsverfahren…` 의 첫 하이픈에서 줄이 나뉘고,
    // 한/글 2020 이 저장한 그 세그먼트에는 bit 19 가 없다.
    let paragraph = &core.document().sections[0].paragraphs[23];
    let line = (0..paragraph.line_segs.len())
        .find(|&index| line_ends_at_soft_hyphen(paragraph, index))
        .expect("소프트 하이픈에서 끝나는 저장 줄");
    assert!(
        !hyphenation_marks(paragraph)[line],
        "fixture 의 표지 없는 줄"
    );

    let without_mark = rendered_line(&mut core, 8, "inneres Ge");
    assert!(
        without_mark.trim_end().ends_with("inneres Ge"),
        "표지 없는 저장 줄 끝에는 하이픈이 없다(한/글 2020 Print 와 같다): {without_mark:?}"
    );

    // 반례: 같은 저장 줄에 표지를 세우면 줄 끝에 하이픈을 그린다.
    let mut document = core.document().clone();
    document.sections[0].paragraphs[23].line_segs[line].tag |= LineSeg::TAG_AUTO_HYPHENATION;
    core.set_document(document);
    let with_mark = rendered_line(&mut core, 8, "inneres Ge");
    assert!(
        with_mark.trim_end().ends_with("inneres Ge-"),
        "표지가 있는 저장 줄 끝에는 하이픈을 그린다: {with_mark:?}"
    );
}

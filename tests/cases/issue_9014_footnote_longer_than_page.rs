//! 관계형·고정형: #9014 쪽보다 긴 각주의 쪽별 줄 소속·순서와 본문/각주 영역의 겹침 없음, 전체 쪽수.
//! 주석 갱신: 2026-11-18 05:46 +0900 (KST)
//!
//! 입력은 이슈 재현 절차를 그대로 실행해 만든다(본문 30문단 scaffold, 문단 25에 LF 60줄 각주).
//! 독립 기준은 각 저장본을 한/글 2020(11.0.0.9136) Print 경로
//! (`pdf_output_mode=hancom2020_pdf_driver_one_up`)로 출력한 PDF다.
//! - 60줄(3쪽): 1쪽 본문 0~25·각주 0~14, 2쪽 본문 없음·각주 15~55, 3쪽 본문 26~29·각주 56~59
//! - 100줄(4쪽): 각주 15/41/41/3줄, 2·3쪽 본문 없음, 4쪽 본문 26~29
//! - 45줄(2쪽): 각주 15/30줄, 2쪽 본문 26~29 / 30줄(2쪽): 각주 15/15줄, 2쪽 본문 26~29
//!
//! 기준 PDF는 모두 Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`
//! (SHA-256 앞 16자: 60줄 5101649246f07c48, 100줄 9aa1041343621691, 45줄 5eaf16c92ca2381d,
//! 30줄 5444171b3637dac8)이다.
//!
//! 시각 측정(2026-11-18): `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣 일치율)다.
//! 측정한 source는 base `1e2488ebd4`에 이 수정을 더한 작업 트리다.
//! - 60줄(3쪽): Native 100.00/100.00/100.00, fresh WASM 100.00/100.00/100.00
//! - 100줄(4쪽): Native 100.00×4, fresh WASM 100.00×4
//! - 45줄(2쪽)·30줄(2쪽): Native 100.00×2, fresh WASM 100.00×2
//! - 두 경로 모두 전쪽 최저 100.00이고 쪽수가 기준 PDF와 같다.
//!   수정 전 Native는 60줄 2·3쪽 91.71·45.80, 100줄 3쪽 산출(기준 4쪽)·3쪽 3.47이었다.
//! - 증거: `mydocs/pr/assets/issue_9014/{native,wasm,native-base}/footnote_*_silhouette.tsv`
//!
//! 절대 px 좌표는 고정하지 않는다. 쪽수와 각주 줄 수는 독립 PDF에서 읽은 고정값이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const NOTE_LINES: usize = 60;
const BODY_PARAS: usize = 30;
const MARKER_PARA: usize = 25;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().expect("rhwp")
}

fn temp(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-9014-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn ok(output: Output, step: &str) {
    assert_eq!(output.status.code(), Some(0), "{step}: {output:?}");
}

/// 이슈 재현 절차로 만든 저장본(한컴 기준 PDF의 입력과 같은 절차).
fn long_footnote_doc(note_lines: usize) -> PathBuf {
    let blocks: Vec<String> = (0..BODY_PARAS)
        .map(|k| {
            format!(r#"{{"type":"paragraph","text":"본문 문단 {k}. 이 문장은 쪽을 채우기 위한 본문이다."}}"#)
        })
        .collect();
    let spec = temp("spec", "json");
    std::fs::write(
        &spec,
        format!(r#"{{"version":"1","blocks":[{}]}}"#, blocks.join(",")),
    )
    .unwrap();
    let a = temp("a", "hwpx");
    let b = temp("b", "hwpx");
    let c = temp("c", "hwpx");
    ok(
        run(&[
            "scaffold",
            spec.to_str().unwrap(),
            "-o",
            a.to_str().unwrap(),
        ]),
        "scaffold",
    );
    let para = MARKER_PARA.to_string();
    ok(
        run(&[
            "edit",
            "insert-footnote",
            a.to_str().unwrap(),
            "--para",
            &para,
            "--offset",
            "5",
            "-o",
            b.to_str().unwrap(),
        ]),
        "insert-footnote",
    );
    let text = (0..note_lines)
        .map(|k| format!("풀이 {k}줄: 식을 정리하면 값이 나온다."))
        .collect::<Vec<_>>()
        .join("\n");
    ok(
        run(&[
            "edit",
            "insert-footnote-text",
            b.to_str().unwrap(),
            "--para",
            &para,
            "--ctrl",
            "0",
            "--fn-para",
            "0",
            "--offset",
            "2",
            "--text",
            &text,
            "-o",
            c.to_str().unwrap(),
        ]),
        "insert-footnote-text",
    );
    for path in [&spec, &a, &b] {
        let _ = std::fs::remove_file(path);
    }
    c
}

fn load(path: &Path) -> DocumentCore {
    let bytes = std::fs::read(path).expect("재현 저장본");
    let _ = std::fs::remove_file(path);
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

/// 쪽의 (본문 줄, 각주 줄) — 각 줄의 (글, 위끝, 아래끝).
#[derive(Default)]
struct PageLines {
    body: Vec<(String, f64, f64)>,
    notes: Vec<(String, f64, f64)>,
    note_area_top: Option<f64>,
}

fn page_lines(core: &DocumentCore, page: u32) -> PageLines {
    fn walk(node: &RenderNode, in_notes: bool, in_body: bool, out: &mut PageLines) {
        let in_notes = in_notes || matches!(node.node_type, RenderNodeType::FootnoteArea);
        let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
        if matches!(node.node_type, RenderNodeType::FootnoteArea) {
            out.note_area_top = Some(
                out.note_area_top
                    .map_or(node.bbox.y, |top: f64| top.min(node.bbox.y)),
            );
        }
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let line = (text(node), node.bbox.y, node.bbox.y + node.bbox.height);
            if in_notes {
                out.notes.push(line);
            } else if in_body {
                out.body.push(line);
            }
            return;
        }
        for child in &node.children {
            walk(child, in_notes, in_body, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
    let mut out = PageLines::default();
    walk(&tree.root, false, false, &mut out);
    out
}

fn note_numbers(lines: &[(String, f64, f64)]) -> Vec<usize> {
    lines
        .iter()
        .filter_map(|(line, _, _)| {
            let rest = line.split("풀이 ").nth(1)?;
            rest.split('줄').next()?.trim().parse().ok()
        })
        .collect()
}

fn body_numbers(lines: &[(String, f64, f64)]) -> Vec<usize> {
    lines
        .iter()
        .filter_map(|(line, _, _)| {
            // 참조 문단은 `본문 문단` 과 번호 사이에 각주 표시가 끼므로 마침표 앞 마지막 낱말을 쓴다.
            let head = line.strip_prefix("본문 문단")?.split('.').next()?;
            head.split_whitespace().last()?.parse().ok()
        })
        .collect()
}

/// 각주 줄은 문서 순서대로 한 번씩 보존되고, 어느 쪽에서도 본문 줄과 각주 영역이 겹치지 않는다.
fn assert_notes_preserved_without_body_overlap(core: &DocumentCore, note_lines: usize) {
    let mut all_notes = Vec::new();
    for page in 0..core.page_count() {
        let lines = page_lines(core, page);
        if let Some(top) = lines.note_area_top {
            for (line, _, bottom) in &lines.body {
                assert!(
                    *bottom <= top + 0.5,
                    "{}쪽 본문 줄 `{line}`(아래끝 {bottom:.1})이 각주 영역(위끝 {top:.1})과 겹친다",
                    page + 1
                );
            }
        }
        all_notes.extend(note_numbers(&lines.notes));
    }
    assert_eq!(
        all_notes,
        (0..note_lines).collect::<Vec<_>>(),
        "각주 줄이 순서대로 한 번씩 이어져야 한다"
    );
}

/// 한/글 2020 Print 기준: 3쪽, 2쪽은 각주만 있는 쪽이다.
#[test]
fn footnote_longer_than_page_continues_over_note_only_page() {
    let core = load(&long_footnote_doc(NOTE_LINES));
    assert_eq!(core.page_count(), 3, "한/글 2020 Print 기준 쪽수");
    assert_notes_preserved_without_body_overlap(&core, NOTE_LINES);

    let pages: Vec<PageLines> = (0..3).map(|page| page_lines(&core, page)).collect();
    assert_eq!(
        body_numbers(&pages[0].body),
        (0..=MARKER_PARA).collect::<Vec<_>>(),
        "1쪽 본문은 참조 문단까지"
    );
    assert!(pages[1].body.is_empty(), "2쪽은 각주만 있는 쪽이다");
    assert_eq!(
        body_numbers(&pages[2].body),
        (MARKER_PARA + 1..BODY_PARAS).collect::<Vec<_>>(),
        "남은 본문은 3쪽"
    );
    // 각 쪽은 앞 쪽에 이어 남은 각주 줄을 받는다.
    for page in &pages {
        assert!(!page.notes.is_empty(), "각 쪽에 이어받은 각주 줄이 있다");
    }
}

/// 한/글 2020 Print 기준(100줄): 4쪽, 각주 줄 15/41/41/3, 2·3쪽은 각주만 있는 쪽이다.
/// 이어받은 조각을 쪽마다 다시 나누는 반복(한 번 나눈 뒤 남은 조각도 쪽보다 긴 경우)을 검사한다.
#[test]
fn footnote_spanning_two_note_only_pages_keeps_splitting() {
    let core = load(&long_footnote_doc(100));
    assert_eq!(core.page_count(), 4, "한/글 2020 Print 기준 쪽수");
    assert_notes_preserved_without_body_overlap(&core, 100);

    let pages: Vec<PageLines> = (0..4).map(|page| page_lines(&core, page)).collect();
    let per_page: Vec<Vec<usize>> = pages.iter().map(|page| note_numbers(&page.notes)).collect();
    assert_eq!(
        per_page.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![15, 41, 41, 3],
        "쪽별 각주 줄 수"
    );
    assert!(
        pages[1].body.is_empty() && pages[2].body.is_empty(),
        "2·3쪽은 각주만 있는 쪽이다"
    );
    assert_eq!(
        body_numbers(&pages[3].body),
        (MARKER_PARA + 1..BODY_PARAS).collect::<Vec<_>>(),
        "남은 본문은 4쪽"
    );
}

/// 대조군(한/글 2020 Print 2쪽): 이어받는 조각이 한 쪽에 들어가면 남은 본문과 같은 쪽에 놓인다.
/// 30줄은 각주 줄 15/15, 45줄은 15/30이며 두 경우 모두 2쪽에 본문 26~29가 있다.
#[test]
fn footnote_continuation_that_fits_one_page_keeps_body() {
    for (note_lines, second_page_lines) in [(30, 15), (45, 30)] {
        let core = load(&long_footnote_doc(note_lines));
        assert_eq!(
            core.page_count(),
            2,
            "{note_lines}줄: 한/글 2020 Print 기준 쪽수"
        );
        assert_notes_preserved_without_body_overlap(&core, note_lines);
        let second = page_lines(&core, 1);
        assert_eq!(
            note_numbers(&second.notes).len(),
            second_page_lines,
            "{note_lines}줄: 2쪽 각주 줄 수"
        );
        assert_eq!(
            body_numbers(&second.body),
            (MARKER_PARA + 1..BODY_PARAS).collect::<Vec<_>>(),
            "{note_lines}줄: 2쪽 본문"
        );
    }
}

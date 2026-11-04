//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8487의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:41:03+09:00 (`1b635b32684ac4ba1eabeaa96d0d582c2452146a`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8479/missing_lines.hwpx`; 독립 Print `pdf/issue8479_missing_lines-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8487_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 담았을 때만 쪽 경계다. 저장 줄 없는 보이는 문단에 자리를 남기지 않은 사다리(그 문단의
//! 높이를 세지 않았다)나 보이는 글줄이 본문 아래에서 시작하는 사다리는 이 쪽 기하의 쪽
//! 배분이 아니므로 한/글처럼 다시 흘린다.
//!
//! - 입력: `samples/issue8479/missing_lines.hwpx` — `gen_stored_ladder.py` 로 생성(저장 줄은 스크립트 값).
//! - 독립 기준: 한/글 2022(12.0.0.4547) 파일 → 인쇄 → Hancom PDF(한 쪽씩)
//!   `pdf/issue8479_missing_lines-2020.pdf`. 2쪽 — 1쪽 끝 `NOLINE 07`, 2쪽 `NOLINE 08` … `LAST 03`,
//!   `RESET`, `TAIL 10`.
//! - 수정 전 rhwp 는 `RESET` 에서 쪽을 넘겨 3쪽(2쪽에 6 문단)이었다.
//! - 측정(`silhouette_raw_match_percent` 1/2쪽): Native 100/100 · fresh WASM 100/100.
//!   증거 경로 `mydocs/pr/assets/issue_8479/{native,wasm}/fixture.tsv`.
//! - 반례(경계를 지킴): 저장 줄만으로 쪽을 넘어 누적되는 사다리의 vpos 0 은 한/글도 쪽
//!   경계로 쓴다 — `issue_8160_all_pages` 의 `hwp3-sample16-hwp5.hwpx` 45쪽이 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn collect_text(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextLine(_) = &node.node_type {
        let mut text = String::new();
        collect_runs(node, &mut text);
        let text = text.replace(' ', "");
        if !text.is_empty() {
            out.push(text);
        }
        return;
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn collect_runs(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 쪽마다 그려진 글줄(공백 제거) 목록.
fn page_lines(core: &DocumentCore) -> Vec<Vec<String>> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("렌더 트리");
            let mut out = Vec::new();
            collect_text(&tree.root, &mut out);
            out
        })
        .collect()
}

fn page_of(pages: &[Vec<String>], label: &str) -> usize {
    let hits: Vec<usize> = pages
        .iter()
        .enumerate()
        .flat_map(|(page, lines)| lines.iter().filter(|l| *l == label).map(move |_| page))
        .collect();
    assert_eq!(hits.len(), 1, "{label} 는 한 번만 그려져야 한다: {hits:?}");
    hits[0]
}

/// 모든 라벨이 한 번씩, 원본 순서대로 그려진다.
fn assert_order(pages: &[Vec<String>], labels: &[String]) {
    let drawn: Vec<&String> = pages
        .iter()
        .flatten()
        .filter(|l| labels.contains(l))
        .collect();
    let expected: Vec<&String> = labels.iter().collect();
    assert_eq!(drawn, expected, "본문 글줄 순서·누락·중복");
}

fn labels(prefix: &str, n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("{prefix}{i:02}")).collect()
}

#[test]
fn ladder_without_room_for_unstored_paragraphs_does_not_force_reset_page() {
    let core = core("samples/issue8479/missing_lines.hwpx");
    let pages = page_lines(&core);
    let mut all = labels("STORED", 36);
    all.extend(labels("NOLINE", 10));
    all.extend(labels("LAST", 3));
    all.push("RESET".into());
    all.extend(labels("TAIL", 10));
    assert_order(&pages, &all);
    // 한/글 2쪽: 저장 줄 없는 문단이 1쪽을 넘긴 뒤 `RESET` 은 같은 쪽에서 이어진다.
    assert_eq!(pages.len(), 2, "한/글 Print 쪽수");
    assert_eq!(
        page_of(&pages, "LAST03"),
        page_of(&pages, "RESET"),
        "사다리가 세지 않은 내용 뒤의 vpos 0 은 쪽 경계가 아니다"
    );
    assert_eq!(page_of(&pages, "NOLINE01"), 0);
    assert_eq!(page_of(&pages, "TAIL10"), 1);
}

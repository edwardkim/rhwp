//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8519의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:54+09:00 (`72754779c6b1e8e1898565b065d09fc51ba10c44`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8518/table_host_top.hwp`; 독립 Print `pdf/issue8518_table_host_top-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8519_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 표)이 새 쪽 첫 문단으로 저장돼 있으면(첫 줄 vpos = 문단 위 간격, 앞 문단은 쪽 하단) 그
//! 저장 쪽 경계에서 새 쪽을 연다. 표 경로가 따로 쪽을 넘기며 host 줄의 간격을 달리 세지 않는다.
//!
//! - 입력: `samples/issue8518/table_host_top.hwp` — `gen_table_host_top.py 20 28 830` 의 입력을
//!   한/글 2022(12.0.0.4547)가 다시 저장한 HWP. host 문단 저장 vpos 830(= 문단 위 간격).
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8518_table_host_top-2020.pdf` — 2쪽.
//!   2쪽에 표(ROW 01~20)와 `AFTER` 가 있고, `AFTER` 는 표 아래끝 바로 밑(3.2px)에서 시작한다.
//! - 수정 전 rhwp 는 `AFTER` 를 표 아래끝에서 20px 밑에 두었다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// (글자, 위 y, 아래 y) 와 표 노드의 (위 y, 아래 y).
fn collect(node: &RenderNode, runs: &mut Vec<(String, f64, f64)>, tables: &mut Vec<(f64, f64)>) {
    match &node.node_type {
        RenderNodeType::TextRun(run) if !run.text.trim().is_empty() => runs.push((
            run.text.trim().to_string(),
            node.bbox.y,
            node.bbox.y + node.bbox.height,
        )),
        RenderNodeType::Table(_) => tables.push((node.bbox.y, node.bbox.y + node.bbox.height)),
        _ => {}
    }
    for child in &node.children {
        collect(child, runs, tables);
    }
}

#[test]
fn empty_table_host_saved_at_page_top_opens_the_page() {
    let core = core("samples/issue8518/table_host_top.hwp");
    assert_eq!(core.page_count(), 2, "한/글 Print 쪽수");
    let tree = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let (mut runs, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut runs, &mut tables);
    assert_eq!(tables.len(), 1, "2쪽에 표 하나");
    let (table_top, table_bottom) = tables[0];
    let find = |text: &str| {
        runs.iter()
            .find(|(t, _, _)| t == text)
            .unwrap_or_else(|| panic!("2쪽에 {text}"))
            .clone()
    };
    // 표의 행은 모두 2쪽 표 안에 순서대로 있다.
    let mut last = table_top;
    for row in 1..=20 {
        let (_, top, bottom) = find(&format!("ROW {row:02}"));
        assert!(
            top >= last - 0.5 && bottom <= table_bottom + 0.5,
            "ROW {row:02} 은 표 안·순서"
        );
        last = top;
    }
    // 뒤 문단은 표 아래에서 시작하되, 자기 문단 위 간격(830HU = 11.07px)보다 멀리 떨어지지
    // 않는다 — host 줄 간격을 표 경로에서 한 번 더 세지 않는다.
    let (_, after_top, _) = find("AFTER");
    let gap = after_top - table_bottom;
    assert!(gap >= -0.5, "AFTER 는 표 아래에서 시작한다: gap={gap:.2}");
    assert!(
        gap < 11.07,
        "AFTER 는 표 아래끝에서 문단 위 간격 안에 시작한다: gap={gap:.2}"
    );
    // 1쪽은 표를 담지 않는다.
    let first = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let (mut first_runs, mut first_tables) = (Vec::new(), Vec::new());
    collect(&first.root, &mut first_runs, &mut first_tables);
    assert!(first_tables.is_empty(), "1쪽에 표 조각이 없다");
    assert!(
        first_runs.iter().any(|(t, _, _)| t == "PRE 28"),
        "1쪽 끝은 PRE 28"
    );
}

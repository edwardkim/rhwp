//! 관계형·고정형(독립 쪽수): PR #8433의 기본 개요 예약 영역 회귀.
//! 주석 갱신: 2026-11-05T06:29:07+09:00; 직전 파일 변경: 2026-11-05T04:07:14+09:00 (`d5f1feb9132a0a3cc57378eb9924da53ea6c5a50`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-05 KST, 제품 코드 `c183a1ac61c2aa4188989fd93032cae68d3538c1`.
//! 같은 원본·기존 독립 Print를 실제 새 Native/fresh WASM 제품으로 다시 측정했다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `mydocs/pr/assets/issue_8425/probe/levels.hwpx`; 독립 Print `pdf/issue-8425/levels-2024.pdf`.
//! 전체 1쪽, 전쪽 최저와 검사 1쪽 Native·fresh WASM 각각 100.00%.
//! 입력 `mydocs/pr/assets/issue_8425/probe/levels.hwpx`; 독립 Print `pdf/issue-8425/levels-2020.pdf`.
//! 전체 1쪽, 전쪽 최저와 검사 1쪽 Native·fresh WASM 각각 100.00%.
//! 입력 `samples/task3307/issue3307_outline_number.hwpx`; 독립 Print `pdf/task3307/issue3307_outline_number-2020.pdf`.
//! 전체 9쪽, 전쪽 최저 Native 99.99% / fresh WASM 99.94%.
//! 주장 7쪽 raw: Native 100.00% / fresh WASM 99.99%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8433_review.md`.
//! 실제 새 TSV·직접 판독: `mydocs/pr/assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T19:05:00+09:00 (새 파일).
//! 2026-11-01 측정 raw(`visual_sweep.py --silhouette-only`, 96dpi, pdftoppm, `--compat 2022`),
//! `samples/task3307/issue3307_outline_number.hwpx` ↔ `pdf/task3307/issue3307_outline_number-2020.pdf`:
//! 검사 쪽 7쪽 Native 99.80% / fresh WASM 99.80% (수정 전 Native 97.42%); 전9쪽 최저
//! Native 99.43%(2쪽) / fresh WASM 99.42%(2쪽), 한컴 9쪽 = rhwp 9쪽.
//! 글꼴: rhwp `RHWP_FONT_PATH=ttfs/opensource`(문서 글꼴 돋움·한양신명조 등 미설치, 대체).
//! 근거: mydocs/pr/assets/issue_8425/README.md.
//!
//! [#8425] 번호 정의 없는 개요 문단(#3307 기본 모양 합성)의 머리 모양을 한컴 내장 값으로 합성한다.
//!
//! ## 무엇이 문제였나
//!
//! `default_outline_numbering()` 은 번호 형식(전 수준 `^N`)만 합성하고 머리 모양(`heads`)은
//! `Numbering::default()` 그대로 두었다. 본문과의 거리 0·자동 내어쓰기 없음이라 rhwp 는 번호
//! 바로 뒤에 본문을 붙여 그리고("1.인적사항"), 두 줄로 넘친 개요 문단의 둘째 줄을 번호 자리에서
//! 시작했다.
//!
//! ## 기대값의 출처 — 수정과 독립적인 근거
//!
//! - `pdf/task3307/issue3307_outline_number-2020.pdf`(한컴 2020 Print) 7쪽: "1." 은 x=117.54px,
//!   "인적사항" 은 137.53px 에서 시작하고(10pt 셀의 참조 형식 1em + 거리 50%), "4. 감경 대상 공적 유무 및"
//!   의 둘째·셋째 줄은 첫 줄 본문과 같은 137.53px 에서 이어진다. 개요가 아닌 리터럴
//!   "5. 혐의자의" 는 번호와 본문 사이에 거리를 두지 않는다.
//! - 2026-11-05 정정: 두 독립 Print의 수준별 제목은 실제 글리프 폭이 아니라 `^N` 참조
//!   형식의 수준당 1em과 거리 50%를 예약한다. 아래 검사도 이 독립 관계를 사용한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/task3307/issue3307_outline_number.hwpx";
/// `pdf/task3307/issue3307_outline_number-2020.pdf` 의 7쪽 (0부터 6).
const PAGE: u32 = 6;

#[derive(Debug, Clone)]
struct Run {
    text: String,
    bbox: BoundingBox,
    font_size: f64,
    /// 런을 감싼 가장 안쪽 표 칸 (행, 열).
    cell: Option<(u16, u16)>,
}

fn page_runs() -> Vec<Run> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("3307 픽스처")).expect("문서 로드");
    runs_in_page(&core, PAGE)
}

fn runs_in_page(core: &DocumentCore, page: u32) -> Vec<Run> {
    let tree = core.build_page_render_tree(page).expect("render tree");
    fn walk(node: &RenderNode, cell: Option<(u16, u16)>, out: &mut Vec<Run>) {
        let cell = match &node.node_type {
            RenderNodeType::TableCell(c) => Some((c.row, c.col)),
            _ => cell,
        };
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push(Run {
                text: run.text.clone(),
                bbox: node.bbox,
                font_size: run.style.font_size,
                cell,
            });
        }
        for child in &node.children {
            walk(child, cell, out);
        }
    }
    let mut runs = Vec::new();
    walk(&tree.root, None, &mut runs);
    runs
}

fn find<'a>(runs: &'a [Run], text: &str) -> &'a Run {
    runs.iter()
        .find(|r| r.text.trim() == text)
        .unwrap_or_else(|| panic!("7쪽에서 {text:?} 런을 못 찾았다"))
}

/// 번호 런과 같은 줄에서 그 오른쪽에 처음 오는 런.
fn text_after<'a>(runs: &'a [Run], number: &Run) -> &'a Run {
    runs.iter()
        .filter(|r| {
            (r.bbox.y - number.bbox.y).abs() < 0.5
                && r.bbox.x > number.bbox.x + 0.5
                && !r.text.trim().is_empty()
        })
        .min_by(|a, b| a.bbox.x.total_cmp(&b.bbox.x))
        .unwrap_or_else(|| panic!("{:?} 뒤 본문 런이 없다", number.text))
}

/// 참조 형식 1em에는 원본 장평 97%를 적용하고 본문 거리 50%는 별도로 둔다.
#[test]
fn default_outline_number_reserves_one_em_form_and_half_em_distance() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("3307 원본")).expect("문서 로드");
    // 원본 HWPX header.xml의 charPr 25·34는 모두 장평 97%다.
    // 독립 Print의 번호와 제목 원점은 이 참조 폭과 50% 본문 거리로 대응한다.
    let shapes = &core.document().doc_info.char_shapes;
    assert_eq!(
        shapes[25].ratios[0], 97,
        "인적사항·비위 유형·징계부가금의 원본 장평"
    );
    assert_eq!(shapes[34].ratios[0], 97, "감경 대상의 원본 장평");
    let expected_em = f64::from(shapes[25].ratios[0]) / 100.0 + 0.5;
    let runs = runs_in_page(&core, PAGE);
    for number in ["1.", "2.", "3.", "4."] {
        let num = find(&runs, number);
        let body = text_after(&runs, num);
        let offset_em = (body.bbox.x - num.bbox.x) / num.font_size;
        assert!(
            (offset_em - expected_em).abs() <= 0.001,
            "{number} {:?}: 실제 글리프 폭과 무관하게 참조 형식 1em × 원본 장평 + 거리 0.5em이어야 한다: {offset_em}em != {expected_em}em",
            body.text
        );
    }
}

/// 제목 1~7의 본문은 참조 수준 경로의 예약 칸 뒤에서 시작한다.
#[test]
fn builtin_outline_levels_reserve_reference_form_instead_of_glyph_width() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/issue_8425/probe/levels.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("한컴 탐침 원본"))
        .expect("기본 개요 탐침");
    assert_eq!(core.page_count(), 1, "두 독립 Print의 전체 쪽수");
    let runs = runs_in_page(&core, 0);
    for level in 1..=7 {
        let marker_text = "1.".repeat(level);
        let marker = find(&runs, &marker_text);
        let body = text_after(&runs, marker);
        assert!(
            body.text.trim_start().starts_with("수준"),
            "run 분할과 무관하게 탐침의 실제 제목 시작이어야 한다: {:?}",
            body.text
        );
        let expected = level as f64 + 0.5;
        let actual = (body.bbox.x - marker.bbox.x) / marker.font_size;
        assert!(
            (actual - expected).abs() <= 0.001,
            "수준{level}: 실제 글리프가 아니라 한 자리 참조 형식과 50% 거리를 예약: {actual}em != {expected}em"
        );
    }
}

/// 두 줄 이상으로 넘친 개요 문단의 다음 줄은 번호가 아니라 첫 줄 본문 시작에서 이어진다.
#[test]
fn default_outline_wrapped_lines_hang_at_text_start() {
    let runs = page_runs();
    let num = find(&runs, "4.");
    let first = text_after(&runs, num);
    assert!(num.cell.is_some(), "\"4.\" 는 표 칸 안에 있어야 한다");
    let mut in_cell: Vec<&Run> = runs
        .iter()
        .filter(|r| r.cell == num.cell && !r.text.trim().is_empty())
        .collect();
    in_cell.sort_by(|a, b| {
        a.bbox
            .y
            .total_cmp(&b.bbox.y)
            .then(a.bbox.x.total_cmp(&b.bbox.x))
    });
    let text: String = in_cell
        .iter()
        .flat_map(|r| r.text.chars())
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(
        text, "4.감경대상공적유무및감경대상비위해당여부",
        "개요 문단 글자가 누락·중복 없이 그려져야 한다"
    );

    let mut line_starts: Vec<&Run> = Vec::new();
    for r in in_cell.iter().filter(|r| r.bbox.y > first.bbox.y + 0.5) {
        match line_starts.last() {
            Some(last) if (r.bbox.y - last.bbox.y).abs() < 0.5 => {}
            _ => line_starts.push(r),
        }
    }
    assert!(
        !line_starts.is_empty(),
        "\"4.\" 개요 문단은 둘째 줄로 넘쳐야 한다"
    );
    for line in line_starts {
        assert!(
            (line.bbox.x - first.bbox.x).abs() <= 1.0,
            "{:?} 줄의 시작 {:.2}px 가 첫 줄 본문 시작 {:.2}px 와 같아야 한다 (번호 시작 {:.2}px)",
            line.text,
            line.bbox.x,
            first.bbox.x,
            num.bbox.x
        );
    }
}

/// 반례: 개요가 아닌 리터럴 번호 문단 "5. 혐의자의" 에는 머리 모양 거리가 붙지 않는다.
#[test]
fn literal_numbered_text_is_not_indented() {
    let runs = page_runs();
    let literal = runs
        .iter()
        .find(|r| r.text.starts_with("5."))
        .expect("리터럴 '5.' 런");
    let body = find(&runs, "혐의자의");
    let gap = body.bbox.x - (literal.bbox.x + literal.bbox.width);
    assert!(
        gap.abs() <= 0.5,
        "리터럴 번호 뒤 본문은 이어 붙어야 한다 (거리 {gap:.2}px)"
    );
    let outline = find(&runs, "1.");
    assert!(
        (literal.bbox.x - outline.bbox.x).abs() <= 0.5,
        "리터럴 번호와 개요 번호의 시작 열이 같아야 한다"
    );
}

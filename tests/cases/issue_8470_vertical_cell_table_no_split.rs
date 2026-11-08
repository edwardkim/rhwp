//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8475의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:38:52+09:00 (`5d7e42138e721f03abf79a3ab6ddc796525fb2e8`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8470/vertical_fit.hwpx`; 독립 Print `pdf/issue8470/vertical_fit-2020.pdf`.
//! 전체 5쪽, 전쪽 최저 Native 98.75% / fresh WASM 98.72%.
//! 주장 5쪽 raw: Native 98.75% / fresh WASM 98.72%.
//! 입력 `samples/issue8470/vertical_oversize.hwpx`; 독립 Print `pdf/issue8470/vertical_oversize-2020.pdf`.
//! 전체 5쪽, 전쪽 최저 Native 98.94% / fresh WASM 98.92%.
//! 주장 5쪽 raw: Native 98.94% / fresh WASM 98.92%.
//! 입력 `samples/issue8470/42565_입법예고문.hwp`; 독립 Print `pdf/issue8470/42565_입법예고문-2020.pdf`.
//! 전체 16쪽, 전쪽 최저 Native 94.92% / fresh WASM 96.96%.
//! 주장 5쪽 raw: Native 99.27% / fresh WASM 99.24%.
//! 주장 10쪽 raw: Native 99.26% / fresh WASM 99.25%.
//! 주장 11쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 주장 13쪽 raw: Native 99.88% / fresh WASM 99.84%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8475_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02 19:16 KST (신규 파일).
//! 2026-11-02 측정 raw(`silhouette_raw_match_percent`, 96dpi, PDFium, `--compat 2022`, 한/글 2020 엔진 Print 정본):
//! vertical_oversize.hwpx: 3쪽(H3 둘째 조각) Native 100.00% / fresh WASM 100.00%; 5쪽(V3 통째) Native 98.94% / fresh WASM 98.92%; 전5쪽 최저 98.94% / 98.92%.
//! vertical_fit.hwpx: 2쪽(H6 첫 조각) Native 100.00% / fresh WASM 100.00%; 5쪽(V6 통째) Native 98.75% / fresh WASM 98.72%; 전5쪽 최저 98.75% / 98.72%.
//! 근거: mydocs/pr/assets/issue_8470/{native,wasm}/vertical_{oversize,fit}_silhouette.tsv,
//! mydocs/pr/assets/issue_8470/png/vertical_{oversize,fit}_p5_{review,overlay}_{native,wasm}.png.
//!
//! #8470 세로쓰기 칸이 있는 표는 쪽 경계에서 나누지 않는다.
//!
//! 독립 기준은 같은 HWPX 의 한/글 2020 엔진 Print PDF(`pdf/issue8470/*-2020.pdf`)다. 생성기는
//! `samples/issue8470/gen_vertical_cell_table.py` 이며, 아래 쪽 소속은 PDF 본문 글자(`H3 행N` 등)에서 읽었다.
//! 두 표는 0열 걸침 칸의 글 방향만 다르다(가로쓰기 H / 세로쓰기 V). 쪽 나눔은 둘 다 셀 단위로 나눔.
//! - `vertical_oversize`: 쪽 맨 위 자리차지 표(행 36개)가 본문보다 943HU 길다.
//!   한/글은 H3 을 행 1..35 | 36 으로 나누고, V3 은 통째로 다음 쪽에 넘쳐 그린다(뒤 문단 `V3 끝` 이 앞 쪽에 남는다).
//!   docs#333(입법예고문 21행×22열 표, 세로쓰기 「가족관계」·「임용전경력」)과 같은 꼴이다.
//! - `vertical_fit`: 채움 20줄 뒤 표(행 30개). H6 은 쪽 바닥에서 행 1..17 | 18..30 으로 나뉘고,
//!   V6 은 남은 공간이 있어도 나누지 않고 통째로 다음 쪽에 놓인다.
//!
//! 검사는 쪽 소속(어느 행·문단이 어느 쪽에 있나)·누락·중복이다. 좌표는 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn line_texts(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                text.push_str(&run.text);
            }
            stack.extend(n.children.iter().rev());
        }
        out.push(text.trim().to_string());
        return;
    }
    for child in &node.children {
        line_texts(child, out);
    }
}

fn page_lines(core: &DocumentCore) -> Vec<Vec<String>> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            let mut lines = Vec::new();
            line_texts(&tree.root, &mut lines);
            lines
        })
        .collect()
}

/// 표 `label` 의 행 번호를 쪽마다 모은다. 행이 없는 쪽은 빠진다: `(쪽 번호, 행 번호들)`.
fn rows_by_page(pages: &[Vec<String>], label: &str) -> Vec<(usize, Vec<u32>)> {
    let prefix = format!("{label} 행");
    pages
        .iter()
        .enumerate()
        .filter_map(|(page, lines)| {
            let mut rows: Vec<u32> = lines
                .iter()
                .filter_map(|line| line.strip_prefix(&prefix)?.trim().parse().ok())
                .collect();
            rows.sort_unstable();
            (!rows.is_empty()).then_some((page, rows))
        })
        .collect()
}

fn page_of(pages: &[Vec<String>], text: &str) -> usize {
    let found: Vec<usize> = (0..pages.len())
        .filter(|&page| pages[page].iter().any(|line| line == text))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "`{text}` 는 한 쪽에 한 번 나와야 한다: {found:?}"
    );
    found[0]
}

fn all_rows(count: u32) -> Vec<u32> {
    (1..=count).collect()
}

#[test]
fn oversize_vertical_cell_table_is_placed_whole_on_next_page() {
    let core = open("samples/issue8470/vertical_oversize.hwpx");
    let pages = page_lines(&core);

    // 가로쓰기 대조군: 쪽 끝에서 행 단위로 나뉜다(한/글 2·3쪽).
    let h3 = rows_by_page(&pages, "H3");
    assert_eq!(
        h3.iter().map(|(_, rows)| rows.clone()).collect::<Vec<_>>(),
        vec![all_rows(35), vec![36]],
        "H3 은 행 1..35 | 36 으로 나뉜다: {h3:?}"
    );
    assert_eq!(h3[1].0, h3[0].0 + 1, "H3 두 조각은 이웃 쪽이다: {h3:?}");
    assert_eq!(
        page_of(&pages, "H3 끝"),
        h3[1].0,
        "H3 끝은 둘째 조각 쪽에 있다"
    );

    // 세로쓰기: 행 36개가 모두 한 쪽에 있고(나누지 않음), 뒤 문단은 그 앞 쪽에 남는다(한/글 4·5쪽).
    let v3 = rows_by_page(&pages, "V3");
    assert_eq!(v3.len(), 1, "V3 은 나뉘지 않는다: {v3:?}");
    assert_eq!(v3[0].1, all_rows(36), "V3 행 누락·중복 없음");
    let v3_end = page_of(&pages, "V3 끝");
    assert!(v3_end > h3[1].0, "V3 은 H3 뒤 쪽 나누기로 시작한다");
    assert_eq!(
        v3[0].0,
        v3_end + 1,
        "V3 표는 V3 끝 문단 다음 쪽에 통째로 놓인다"
    );
    assert_eq!(
        core.page_count() as usize,
        v3[0].0 + 1,
        "V3 표 쪽이 마지막 쪽이다(한/글 5쪽)"
    );
}

#[test]
fn vertical_cell_table_moves_whole_instead_of_splitting_at_page_bottom() {
    let core = open("samples/issue8470/vertical_fit.hwpx");
    let pages = page_lines(&core);

    // 가로쓰기 대조군: 채움 뒤 쪽 바닥에서 1..17 | 18..30(한/글 2·3쪽).
    let h6 = rows_by_page(&pages, "H6");
    assert_eq!(
        h6.iter().map(|(_, rows)| rows.clone()).collect::<Vec<_>>(),
        vec![all_rows(17), (18..=30).collect::<Vec<_>>()],
        "H6 은 행 1..17 | 18..30 으로 나뉜다: {h6:?}"
    );
    assert_eq!(
        page_of(&pages, "H6 머리"),
        h6[0].0,
        "H6 첫 조각은 채움 쪽에 있다"
    );

    // 세로쓰기: 같은 자리에서 나누지 않고 통째로 다음 쪽(한/글 4·5쪽).
    let v6 = rows_by_page(&pages, "V6");
    let v6_head = page_of(&pages, "V6 머리");
    assert_eq!(v6.len(), 1, "V6 은 나뉘지 않는다: {v6:?}");
    assert_eq!(v6[0].1, all_rows(30), "V6 행 누락·중복 없음");
    assert_eq!(
        v6[0].0,
        v6_head + 1,
        "V6 표는 채움 쪽 다음 쪽에 통째로 놓인다"
    );
    assert_eq!(
        page_of(&pages, "V6 끝"),
        v6_head,
        "V6 끝은 채움 쪽에 남는다"
    );
    assert_eq!(
        core.page_count() as usize,
        v6[0].0 + 1,
        "V6 표 쪽이 마지막 쪽이다(한/글 5쪽)"
    );
}

#[test]
fn layout_page_break_keeps_stored_value_for_save() {
    let core = open("samples/issue8470/vertical_fit.hwpx");
    let tables: Vec<&rhwp::model::table::Table> = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .filter_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 2);
    for table in tables {
        let vertical = table.cells.iter().any(|cell| cell.text_direction != 0);
        // 저장 값은 원본(CELL)을 지킨다 — 조판 값만 세로쓰기 표에서 나누지 않음.
        assert_eq!(
            table.page_break,
            rhwp::model::table::TablePageBreak::RowBreak
        );
        let expected = if vertical {
            rhwp::model::table::TablePageBreak::None
        } else {
            rhwp::model::table::TablePageBreak::RowBreak
        };
        assert_eq!(table.layout_page_break(), expected);
    }
}

#[test]
fn layout_page_break_follows_cell_direction_edits() {
    let core = open("samples/issue8470/vertical_fit.hwpx");
    let mut horizontal = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            rhwp::model::control::Control::Table(table)
                if table.cells.iter().all(|cell| cell.text_direction == 0) =>
            {
                Some(table.as_ref().clone())
            }
            _ => None,
        })
        .expect("가로쓰기 대조 표");
    // 판정을 한 번 캐시한 뒤 칸을 세로쓰기로 바꾸면 조판 값도 따라 바뀐다.
    assert_eq!(
        horizontal.layout_page_break(),
        rhwp::model::table::TablePageBreak::RowBreak
    );
    horizontal.cells[0].text_direction = 1;
    horizontal.invalidate_layout_page_break();
    assert_eq!(
        horizontal.layout_page_break(),
        rhwp::model::table::TablePageBreak::None
    );
    horizontal.cells[0].text_direction = 0;
    horizontal.rebuild_grid();
    assert_eq!(
        horizontal.layout_page_break(),
        rhwp::model::table::TablePageBreak::RowBreak
    );
}

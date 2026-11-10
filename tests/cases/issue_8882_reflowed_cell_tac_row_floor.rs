//! 관계형·고정형 — [#8882] 저장 줄 없는 칸이 섞인 글자처럼 취급 표의 행은 재조판 내용 아래로 눌리지 않는다.
//! 주석 갱신: 2026-11-10 18:20 KST (새 파일, devel 5f4e95c187 로 rebase 후 재측정)
//!
//! 입력: 공개 결재문서 36404364 원본 `samples/issue8882/36404364-reflowed-cell-tac-row-floor.hwpx`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8882/36404364-reflowed-cell-tac-row-floor-2020.pdf`(4쪽).
//! 3쪽 4×5 TAC 표(`pi=35`): 가린 칸 3개(첫 열 1~3행)는 `linesegarray` 가 없어 재조판되고 두 줄로
//! 줄바꿈된다(한/글과 같은 줄바꿈). 표 선언 10505HU, 행 선언 2630HU. 한/글은 행을 내용에 맞춰
//! 키워 표가 약 174px 이고, 수정 전 rhwp 는 선언 140.1px 에 가둬 머리 행을 29.5px 로 눌렀다.
//! - 고정형: 전체 쪽수 4.
//! - 관계형 3쪽: 표의 모든 칸 글줄은 제 칸 안에 있다(위끝 ≥ 칸 위끝, 아래끝 ≤ 칸 아래끝).
//! - 관계형 3쪽: 머리 행은 원본 행 선언(2630HU) 아래로 줄지 않는다.
//! - 관계형 3쪽: 표 뒤 첫 문단(`pi=36`)은 표 아래끝보다 위에서 시작하지 않는다.
//!
//! 합성 IR 계약(한컴 출력 증거 아님, 시각 측정 비해당 — 기대값은 같은 측정기의 내용 높이와의 관계):
//! - 재조판 칸 내용이 선언을 48px 넘게(1.5배 안으로) 넘는 혼합 TAC 표는 균일 축소하지 않고 내용
//!   높이를 지킨다(공개 결재문서 36479973·36303317 의 같은 형상. 두 문서는 fresh WASM 에서 가린
//!   글자 그림 위치 차이로 일부 쪽이 90% 미만이라 fixture 로 쓰지 않았다).
//! - 저장 줄 없는 칸 문단의 TAC 중첩 표는 칸 높이에 한 번만 들어간다(공개 샘플
//!   `samples/hwpx/opengov/36384689…` 1쪽의 같은 형상, 그 쪽은 수정 후에도 다른 원인으로 84.20%).
//!
//! 측정(2026-11-10, 이 파일과 같은 커밋의 코드, base = upstream/devel 5f4e95c187,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 4쪽 / 4쪽, p1 97.72 · p2 98.42 · p3 99.52 · p4 98.17, 최저 97.72
//!   (base: p3 62.09, 나머지 같음)
//! - fresh WASM: 4쪽 / 4쪽, p1 94.04 · p2 96.37 · p3 90.04 · p4 99.88, 최저 90.04
//! - 증거: `mydocs/pr/assets/issue_8882/`(native·wasm·native_base TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::{
    composer::compose_section, height_measurer::HeightMeasurer, style_resolver::resolve_styles,
};
use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8882/36404364-reflowed-cell-tac-row-floor.hwpx";

/// HWPUNIT → px(96 DPI).
fn px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 원본 저장값: 36404364 3쪽 표 머리 행의 칸 선언 높이.
const HEADER_ROW_DECLARED_HU: i32 = 2630;

#[derive(Debug, Clone)]
struct Extent {
    page: usize,
    indent: usize,
    kind: String,
    top: f64,
    bottom: f64,
    rest: String,
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn parse_extent(page: usize, line: &str) -> Option<Extent> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let kind = trimmed.split_whitespace().next()?.to_string();
    let y_start = trimmed.find("y=")? + 2;
    let (top, after_top) = trimmed[y_start..].trim_start().split_once("..")?;
    let bottom = after_top.split_whitespace().next()?;
    Some(Extent {
        page,
        indent,
        kind,
        top: top.trim().parse().ok()?,
        bottom: bottom.trim().parse().ok()?,
        rest: trimmed.to_string(),
    })
}

fn extents(sample: &str) -> (usize, Vec<Extent>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    assert!(path.is_file(), "fixture 누락: {}", path.display());
    let out = Command::new(rhwp_bin())
        .arg("dump-extents")
        .arg(&path)
        .output()
        .expect("dump-extents 실행");
    assert!(out.status.success(), "dump-extents 실패");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut page = 0usize;
    let mut pages = 0usize;
    let mut items = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("=== ") {
            page = rest
                .split('쪽')
                .next()
                .and_then(|n| n.trim().parse().ok())
                .expect("쪽 머리");
            pages = pages.max(page);
            continue;
        }
        if page > 0 {
            if let Some(extent) = parse_extent(page, line) {
                items.push(extent);
            }
        }
    }
    (pages, items)
}

/// 쪽의 본문 표(`pi=<para> ci=0`)와 그 뒤 항목들(표 하위 노드 포함) 범위.
fn table_block<'a>(items: &'a [Extent], page: usize, para: usize) -> (&'a Extent, &'a [Extent]) {
    let tag = format!("pi={para} ci=0 ");
    let start = items
        .iter()
        .position(|e| e.page == page && e.kind == "Table" && e.rest.contains(&tag))
        .unwrap_or_else(|| panic!("{page}쪽 표 `{tag}` 없음"));
    let table = &items[start];
    let end = items[start + 1..]
        .iter()
        .position(|e| e.page != page || e.indent <= table.indent)
        .map_or(items.len(), |offset| start + 1 + offset);
    (table, &items[start + 1..end])
}

/// 표의 직속 칸과 그 칸의 직속 글줄이 모두 칸 안에 있는지 검사하고 칸 목록을 돌려준다.
fn assert_cell_lines_contained<'a>(
    label: &str,
    table: &Extent,
    body: &'a [Extent],
) -> Vec<&'a Extent> {
    let cell_indent = body
        .iter()
        .filter(|e| e.kind == "TableCell")
        .map(|e| e.indent)
        .min()
        .expect("칸 없음");
    let mut cells = Vec::new();
    let mut current: Option<&Extent> = None;
    let mut lines = 0usize;
    for item in body {
        if item.kind == "TableCell" && item.indent == cell_indent {
            current = Some(item);
            cells.push(item);
            continue;
        }
        let Some(cell) = current else { continue };
        if item.kind == "TextLine" && item.indent == cell_indent + 2 {
            lines += 1;
            assert!(
                item.top + 0.5 >= cell.top && item.bottom <= cell.bottom + 0.5,
                "{label}: 글줄 {:.1}..{:.1} 이 칸 {:.1}..{:.1} 밖으로 나간다 ({})",
                item.top,
                item.bottom,
                cell.top,
                cell.bottom,
                cell.rest
            );
        }
    }
    assert!(lines > 0, "{label}: 검사한 칸 글줄이 없다");
    for cell in &cells {
        assert!(
            cell.bottom <= table.bottom + 0.5,
            "{label}: 칸 아래끝 {:.1} 이 표 아래끝 {:.1} 밖",
            cell.bottom,
            table.bottom
        );
    }
    cells
}

fn first_line_top(items: &[Extent], page: usize, para: usize) -> f64 {
    let tag = format!("pi={para} line=0 ");
    items
        .iter()
        .find(|e| e.page == page && e.kind == "TextLine" && e.indent == 6 && e.rest.contains(&tag))
        .unwrap_or_else(|| panic!("{page}쪽 `{tag}` 없음"))
        .top
}

#[test]
fn reflowed_cells_keep_their_rows_in_a_mixed_tac_table() {
    let (pages, items) = extents(SAMPLE);
    assert_eq!(pages, 4, "한컴 Print PDF 는 4쪽");

    let (table, body) = table_block(&items, 3, 35);
    let cells = assert_cell_lines_contained("36404364 3쪽 표", table, body);
    let header = cells
        .iter()
        .find(|cell| cell.rest.contains("r=0 c=0"))
        .expect("머리 행 칸");
    assert!(
        header.bottom - header.top + 0.5 >= px(HEADER_ROW_DECLARED_HU),
        "머리 행 {:.1}px 은 행 선언 {:.1}px 아래로 줄지 않는다",
        header.bottom - header.top,
        px(HEADER_ROW_DECLARED_HU)
    );
    let next = first_line_top(&items, 3, 36);
    assert!(
        next + 0.5 >= table.bottom,
        "표 뒤 문단 위끝 {next:.1} 은 표 아래끝 {:.1} 보다 위에서 시작하지 않는다",
        table.bottom
    );
}

/// 1×1 TAC 표. 표 선언 `declared`, 칸 선언 `cell_h`(HWPUNIT). 칸 문단은 `paragraphs` 그대로다.
fn tac_table(declared: u32, cell_h: u32, paragraphs: Vec<Paragraph>) -> Table {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        ..Default::default()
    };
    table.common.width = 20000;
    table.common.height = declared;
    table.common.treat_as_char = true;
    table.cells = vec![Cell {
        row_span: 1,
        col_span: 1,
        width: 20000,
        height: cell_h,
        paragraphs,
        ..Default::default()
    }];
    table.rebuild_grid();
    table
}

fn measured_table_heights(outer: Table) -> f64 {
    let section = Section {
        paragraphs: vec![Paragraph {
            controls: vec![Control::Table(Box::new(outer))],
            ..Default::default()
        }],
        ..Default::default()
    };
    let styles = resolve_styles(&Document::default().doc_info, 96.0);
    let composed = compose_section(&section);
    HeightMeasurer::new(96.0)
        .measure_section(&section.paragraphs, &composed, &styles, None)
        .tables[0]
        .total_height
}

#[test]
fn tac_nested_table_in_unstored_cell_counts_once() {
    // 안쪽 표: 2행(칸 선언 3000HU 씩), 표 선언은 행 합보다 큰 6400HU, 바깥 여백 141HU —
    // 실측이 선언보다 작은 형상(36384689 의 안쪽 8행 표: 실측 27613HU, 선언 27878HU).
    // 안쪽 칸 문단은 저장 줄을 가진다(36384689 의 안쪽 표처럼 칸 자체는 재조판하지 않는다).
    let stored = || Paragraph {
        line_segs: vec![LineSeg {
            line_height: 1000,
            baseline_distance: 850,
            segment_width: 20000,
            ..Default::default()
        }],
        ..Default::default()
    };
    let inner = || {
        let mut table = tac_table(6400, 3000, vec![stored()]);
        table.row_count = 2;
        table.outer_margin_top = 141;
        table.outer_margin_bottom = 141;
        table.cells.push(Cell {
            row: 1,
            row_span: 1,
            col_span: 1,
            width: 20000,
            height: 3000,
            paragraphs: vec![stored()],
            ..Default::default()
        });
        table.rebuild_grid();
        table
    };
    let inner_alone = measured_table_heights(inner());
    // 같은 host 문단에 저장 줄 없는 작은 TAC 표를 하나 더 둔다 — 그 표가 host 줄을 재조판하게
    // 만들어, 저장 칸 표(안쪽 표)도 합성 줄에 실측 높이로 실리는 경로를 탄다.
    let reflowed_small = tac_table(1000, 1000, vec![Paragraph::default()]);
    let host = Paragraph {
        controls: vec![
            Control::Table(Box::new(inner())),
            Control::Table(Box::new(reflowed_small)),
        ],
        char_count: 17,
        ..Default::default()
    };
    // 바깥 표: 저장 줄 없는 칸에 안쪽 표 host 문단만 둔다. 선언은 작게 잡아 내용이 정한다.
    // 1×1 감싸개 지름길을 타지 않도록 빈 칸을 하나 더 둔 1×2 표로 만든다.
    let mut outer = tac_table(1000, 1000, vec![host]);
    outer.col_count = 2;
    outer.cells[0].width = 10000;
    outer.cells.push(Cell {
        col: 1,
        row_span: 1,
        col_span: 1,
        width: 10000,
        height: 1000,
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    });
    outer.rebuild_grid();
    let outer = measured_table_heights(outer);
    assert!(
        outer + 0.5 >= inner_alone,
        "바깥 표 {outer:.1}px 는 안쪽 표 {inner_alone:.1}px 를 담는다"
    );
    assert!(
        outer < inner_alone * 1.5,
        "바깥 표 {outer:.1}px 에 안쪽 표 {inner_alone:.1}px 가 두 번 들어갔다"
    );
}

/// 저장 칸(1행)과 재조판 칸(2행)이 섞인 2×1 TAC 표. 표 선언은 `declared`.
fn mixed_tac_table(declared: u32) -> Table {
    let stored_row = Paragraph {
        text: "가".into(),
        char_count: 2,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            segment_width: 20000,
            ..Default::default()
        }],
        ..Default::default()
    };
    let reflowed_line = || Paragraph {
        text: "가".into(),
        char_count: 2,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        ..Default::default()
    };
    let mut table = tac_table(declared, 1500, vec![stored_row]);
    table.row_count = 2;
    table.cells.push(Cell {
        row: 1,
        row_span: 1,
        col_span: 1,
        width: 20000,
        height: 1000,
        paragraphs: (0..15).map(|_| reflowed_line()).collect(),
        ..Default::default()
    });
    table.rebuild_grid();
    table
}

#[test]
fn reflowed_overflow_beyond_cap_is_not_uniformly_shrunk() {
    // 선언 0 이면 축소 갈래를 타지 않으므로 내용 높이 그대로다.
    let content = measured_table_heights(mixed_tac_table(0));
    // 내용이 선언의 1.39배(축소 상한 1.5배 안)이고 넘는 양이 48px 를 넘게 선언을 잡는다.
    let declared_hu = (content * 0.72 * 7200.0 / 96.0) as u32;
    let declared_px = f64::from(declared_hu) * 96.0 / 7200.0;
    assert!(
        content - declared_px > 48.0 && content <= declared_px * 1.5,
        "전제: 내용 {content:.1}px 는 선언 {declared_px:.1}px 를 48px 넘게, 1.5배 안으로 넘는다"
    );
    let measured = measured_table_heights(mixed_tac_table(declared_hu));
    assert!(
        measured + 0.5 >= content,
        "재조판 칸 내용 {content:.1}px 를 담은 표가 선언 {declared_px:.1}px 쪽으로 {measured:.1}px 까지 눌렸다"
    );
}

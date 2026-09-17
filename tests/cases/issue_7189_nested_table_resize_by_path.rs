//! [Issue #7189] 중첩 표의 칸 경계에서 표 크기 조절이 동작하지 않는다.
//!
//! studio 의 hover/리사이즈 경로는 평면 좌표계 `(구역, 문단, 컨트롤)` 로만 표를 가리켰다.
//! 그 좌표는 `hitTest` 의 `cellPath[0]`, 즉 **최외곽** 표다. 중첩 표는 그 좌표계로 지칭할 수
//! 없어 가이드가 뜨지 않았고, 엔진에도 경로로 셀을 조절하는 명령이 **없었다**.
//!
//! 그래서 hover 만 열어 주는 수정은 더 나쁘다 — 내부 표 bbox 에서 나온 셀 번호가 바깥 표에
//! 적용돼 엉뚱한 행이 조절된다. 엔진에 경로 명령을 먼저 세워야 한다.
//!
//! # 이 시험이 잠그는 것
//!
//! - 깊이 2 경로가 **안쪽 표만** 바꾼다 (바깥 표의 셀·표 크기는 불변).
//! - 깊이 1 경로는 평면 API 와 같은 결과다 (`*_by_path` 계열의 공통 규약).
//! - 폭을 넓히면 안쪽 셀의 문단이 **안쪽 셀 폭 기준**으로 다시 흐른다.
//! - 잘못된 경로는 조용히 성공하지 않는다.
//!
//! # 잠그지 않는 것
//!
//! studio 쪽 배선(hover 가 경로를 넘기는지)과 최소 셀 크기 정책은 범위 밖이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table};

const OUTER_CELL_WIDTH: u32 = 30000;
const INNER_LEFT_WIDTH: u32 = 4000;
const INNER_RIGHT_WIDTH: u32 = 6000;

/// 바깥 1×1 표의 칸 안에 2열 표가 든 문서. `#7189` 신고 문서(3026219)와 같은 형상이다.
///
/// 반환 경로는 studio `hitTest` 의 `cellPath` 와 같은 모양 —
/// `[(바깥 컨트롤, 바깥 셀, 그 셀의 문단), (안쪽 컨트롤, 안쪽 셀, 그 셀의 문단)]`.
fn nested_core() -> (DocumentCore, Vec<(usize, usize, usize)>) {
    let cell = |col: u16, width: u32, text: &str| Cell {
        row: 0,
        col,
        col_span: 1,
        row_span: 1,
        width,
        height: 3000,
        paragraphs: vec![Paragraph {
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
            char_count: text.chars().count() as u32,
            has_para_text: true,
            ..Default::default()
        }],
        ..Default::default()
    };

    let inner_table = Table {
        row_count: 1,
        col_count: 2,
        cells: vec![
            cell(
                0,
                INNER_LEFT_WIDTH,
                "안쪽 왼쪽 칸의 글이 길어지면 다시 흐른다",
            ),
            cell(1, INNER_RIGHT_WIDTH, "안쪽 오른쪽"),
        ],
        ..Default::default()
    };

    let mut outer_cell_para = Paragraph::default();
    outer_cell_para
        .controls
        .push(Control::Table(Box::new(inner_table)));

    let outer_table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row: 0,
            col: 0,
            col_span: 1,
            row_span: 1,
            width: OUTER_CELL_WIDTH,
            height: 9000,
            paragraphs: vec![outer_cell_para],
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut body_para = Paragraph::default();
    body_para
        .controls
        .push(Control::Table(Box::new(outer_table)));

    let mut document = Document::default();
    document.sections.push(Section {
        paragraphs: vec![body_para],
        ..Default::default()
    });

    let mut core = DocumentCore::new_empty();
    core.set_document(document);
    (core, vec![(0, 0, 0), (0, 0, 0)])
}

fn outer_table(core: &DocumentCore) -> &Table {
    match core.document().sections[0].paragraphs[0].controls.first() {
        Some(Control::Table(table)) => table,
        _ => panic!("바깥 표를 찾지 못했다"),
    }
}

fn inner_table(core: &DocumentCore) -> &Table {
    match outer_table(core).cells[0].paragraphs[0].controls.first() {
        Some(Control::Table(table)) => table,
        _ => panic!("안쪽 표를 찾지 못했다"),
    }
}

#[test]
fn nested_path_resizes_the_inner_table_only() {
    let (mut core, path) = nested_core();
    let before_outer_width = outer_table(&core).cells[0].width;

    core.resize_table_cells_by_cell_path_native(
        0,
        0,
        &path,
        r#"[{"cellIdx":0,"widthDelta":1200},{"cellIdx":1,"widthDelta":-1200}]"#,
    )
    .expect("중첩 표 셀 크기 조절이 성공해야 한다");

    let inner = inner_table(&core);
    assert_eq!(
        inner.cells[0].width,
        INNER_LEFT_WIDTH + 1200,
        "안쪽 왼쪽 칸이 넓어져야 한다"
    );
    assert_eq!(
        inner.cells[1].width,
        INNER_RIGHT_WIDTH - 1200,
        "안쪽 오른쪽 칸이 그만큼 좁아져야 한다"
    );
    assert_eq!(
        outer_table(&core).cells[0].width,
        before_outer_width,
        "바깥 표의 칸은 손대지 않는다 — 평면 API 로는 이 값이 대신 바뀌었다",
    );
}

#[test]
fn nested_resize_reflows_with_the_inner_cell_width() {
    let (mut core, path) = nested_core();
    let before_lines = inner_table(&core).cells[0].paragraphs[0].line_segs.len();

    // 안쪽 왼쪽 칸을 크게 좁히면 그 칸 글이 여러 줄로 접혀야 한다. 바깥 칸 폭(30000)이나
    // 본문 폭으로 재면 접히지 않는다 — 리플로우 기준이 안쪽 칸 폭임을 가른다.
    core.resize_table_cells_by_cell_path_native(
        0,
        0,
        &path,
        r#"[{"cellIdx":0,"widthDelta":-3000}]"#,
    )
    .expect("중첩 표 셀 크기 조절이 성공해야 한다");

    let after_lines = inner_table(&core).cells[0].paragraphs[0].line_segs.len();
    assert!(
        after_lines > before_lines,
        "안쪽 칸을 좁히면 그 칸 글이 더 많은 줄로 흘러야 한다 (이전 {before_lines}줄 → 이후 {after_lines}줄)",
    );
}

#[test]
fn depth_one_path_targets_the_outer_table() {
    let (mut core, _) = nested_core();
    let before_outer = outer_table(&core).cells[0].height;
    let before_inner = inner_table(&core).cells[0].height;

    // 깊이 1 경로는 최외곽 표를 가리킨다 — 다른 `*_by_path` 명령과 같은 규약이며,
    // 이 경로는 평면 API 에 위임한다.
    core.resize_table_cells_by_cell_path_native(
        0,
        0,
        &[(0, 0, 0)],
        r#"[{"cellIdx":0,"heightDelta":500}]"#,
    )
    .expect("깊이 1 경로가 성공해야 한다");

    assert_eq!(
        outer_table(&core).cells[0].height,
        before_outer + 500,
        "깊이 1 은 바깥 표의 칸을 조절한다"
    );
    assert_eq!(
        inner_table(&core).cells[0].height,
        before_inner,
        "안쪽 표는 건드리지 않는다"
    );
}

#[test]
fn a_path_that_does_not_end_at_a_table_is_refused() {
    let (mut core, _) = nested_core();
    // 안쪽 표의 컨트롤 번호를 1 로 두면 그 자리에 컨트롤이 없다.
    let result = core.resize_table_cells_by_cell_path_native(
        0,
        0,
        &[(0, 0, 0), (1, 0, 0)],
        r#"[{"cellIdx":0,"widthDelta":100}]"#,
    );
    assert!(
        result.is_err(),
        "표가 아닌 경로는 조용히 성공하면 안 된다 — 엉뚱한 표가 바뀐다"
    );
    assert_eq!(
        inner_table(&core).cells[0].width,
        INNER_LEFT_WIDTH,
        "거부된 요청은 아무것도 바꾸지 않는다"
    );
}

#[test]
fn an_empty_path_is_refused() {
    let (mut core, _) = nested_core();
    let result = core.resize_table_cells_by_cell_path_native(
        0,
        0,
        &[],
        r#"[{"cellIdx":0,"widthDelta":100}]"#,
    );
    assert!(result.is_err(), "빈 경로는 거부해야 한다");
}

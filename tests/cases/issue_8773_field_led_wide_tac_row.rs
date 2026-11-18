//! [#8773] 관계형·고정형 회귀 — 누름틀 시작 뒤에 남은 줄 폭보다 넓은 글자처럼 취급 표가 오면
//! 한/글은 누름틀 시작만 담은 첫 줄을 두고 표를 다음 줄에서 시작한다.
//!
//! 주석 갱신: 2026-11-08 15:24 KST(+09:00), 새 파일.
//!
//! - 입력 A: `samples/issue8773/field_led_wide_table.hwpx` — 공개 결재문서 36479199(HWPX, 2쪽).
//!   문단 5 원시 순서 = 누름틀 시작 → 글자처럼 취급 16×16 표(바깥 여백 포함 50346HU) → 누름틀
//!   끝 → 공백 1자. 저장 줄 1개(높이 1100HU)가 표를 담지 않아 rhwp 가 줄을 재조판한다.
//! - 입력 B(반례): `samples/issue8773/field_led_wide_table_nofield.hwpx` — A 의 `section0.xml` 에서
//!   이 문단의 `fieldBegin`·`fieldEnd` 제어만 지우고 다시 압축한 합성본. 다른 저장값은 그대로다.
//! - 독립 기준: `pdf/issue8773/field_led_wide_table-2020.pdf`, `..._nofield-2020.pdf`(로컬 한/글
//!   2022 12.0.0.4547 Print → Hancom PDF, 각 2쪽). 표 위 괘선 A 255.7px, B 238.1px — 차 17.6px =
//!   글자 1100HU × 줄 간격 120% = 1320HU. 표 폭만 줄인 변형도 255.7px 이다(한/글은 칸 폭으로 표
//!   폭을 다시 정한다). 한/글이 저장한 공개 결재문서 사다리 26건도 누름틀 시작 뒤 넓은 표를 둘째
//!   줄(textpos 8)에 둔다.
//!
//! 측정: 2026-11-08, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `80fc0c6b44` + 이 수정.
//! - A: 1쪽 Native 100.00 / fresh WASM 99.99(수정 전 Native 50.58), 2쪽 94.25 / 94.28. 전체 2쪽 =
//!   PDF 2쪽, 전쪽 최저 Native 94.25(2쪽)·fresh WASM 94.28(2쪽).
//! - B: 1쪽 Native 94.71 / fresh WASM 94.04, 2쪽 94.25 / 94.28(수정 전과 같음). 전체 2쪽 = PDF 2쪽,
//!   전쪽 최저 Native 94.25(2쪽)·fresh WASM 94.04(1쪽). B 1쪽의 남은 차이는 표 뒤 주석 줄이다 — 합성
//!   과정에서 뒤 문단 저장 vpos 를 고치지 않아 rhwp 는 저장 위치(PDF 보다 19px 아래)를 따른다.
//! - 증거: `mydocs/pr/assets/issue_8773/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: A 의 줄 소속(첫 줄에 글자 없음, 공백은 표 줄)과 표 위끝 − 문단 첫 줄 = 글자 모양 크기 ×
//! 줄 간격 + 바깥 위 여백(관계형), 앞 결재 표와 겹치지 않음(관계형), A 1쪽 표 위끝 255.7px ±2(고정형,
//! 양쪽 raw 99.99% 이상·확대 판독). B 는 A 보다 정확히 한 줄 위(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIELD_LED: &str = "samples/issue8773/field_led_wide_table.hwpx";
const NO_FIELD: &str = "samples/issue8773/field_led_wide_table_nofield.hwpx";
const DPI: f64 = 96.0;
/// 표를 담은 본문 문단(구역 0).
const HOST: usize = 5;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn find_table(node: &RenderNode, rows: u16, cols: u16) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows && table.col_count == cols {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, rows, cols))
}

fn host_text_line_tops(node: &RenderNode, out: &mut Vec<f64>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(HOST) {
            out.push(node.bbox.y);
        }
    }
    for child in &node.children {
        host_text_line_tops(child, out);
    }
}

fn previous_tables_bottom(node: &RenderNode) -> f64 {
    // 문단 4 의 두 3행 표(결재란)가 본문 표 바로 앞 흐름이다.
    let mut bottom: f64 = 0.0;
    for (rows, cols) in [(3, 2), (3, 4)] {
        let table = find_table(node, rows, cols).expect("문단 4 결재 표");
        bottom = bottom.max(table.bbox.y + table.bbox.height);
    }
    bottom
}

/// 문단 글자 모양 크기에 문단 줄 간격(비율)을 적용한 한 줄의 전진량(HU).
/// 저장·재조판 줄이 아니라 원본 글자·문단 모양에서 계산한다.
fn text_row_advance_hu(core: &DocumentCore) -> i32 {
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[HOST];
    let shape = &doc.doc_info.para_shapes[para.para_shape_id as usize];
    let char_shape_id = para.char_shapes.first().expect("글자 모양").char_shape_id;
    let base = doc.doc_info.char_shapes[char_shape_id as usize].base_size;
    assert!(
        matches!(
            shape.line_spacing_type,
            rhwp::model::style::LineSpacingType::Percent
        ),
        "비율 줄 간격"
    );
    base * shape.line_spacing / 100
}

#[test]
fn wide_table_after_a_field_start_begins_on_the_next_row() {
    let core = load(FIELD_LED);
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[HOST];
    assert!(
        matches!(para.controls.first(), Some(Control::Field(_))),
        "원시 순서: 누름틀 시작이 표보다 앞"
    );

    // 줄 소속: 첫 줄은 글자를 하나도 갖지 않고(누름틀 시작만), 표와 뒤 공백은 둘째 줄이다.
    assert_eq!(para.line_segs.len(), 2, "누름틀 줄 + 표 줄");
    let second = para.line_segs[1].text_start;
    assert!(
        para.char_offsets.iter().all(|&offset| offset >= second),
        "뒤 공백은 표 줄 소속: {:?} ≥ {second}",
        para.char_offsets
    );

    let page = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let table = find_table(&page.root, 16, 16).expect("16×16 본문 표");
    let Some(Control::Table(model)) = para.controls.get(1) else {
        panic!("문단 5 컨트롤 1 = 표");
    };
    let outer_top = px(i32::from(model.outer_margin_top));
    let mut tops = Vec::new();
    host_text_line_tops(&page.root, &mut tops);
    let host_top = tops.into_iter().fold(f64::INFINITY, f64::min);
    let row = px(text_row_advance_hu(&core));

    // 표는 누름틀 줄 한 줄(글자 크기 × 줄 간격) 아래, 바깥 위 여백만큼 안쪽에서 시작한다.
    let gap = table.bbox.y - host_top;
    assert!(
        (gap - (row + outer_top)).abs() <= 1.0,
        "표 위끝 − 문단 첫 줄 {gap:.1} = 누름틀 줄 {row:.1} + 바깥 위 여백 {outer_top:.1}"
    );
    // 문단 첫 줄은 앞 결재 표 흐름 바로 뒤다(표가 앞 흐름에 겹치지 않는다).
    assert!(
        host_top >= previous_tables_bottom(&page.root) - 0.5,
        "문단 첫 줄 {host_top:.1} 은 앞 결재 표 아래"
    );
    // 한/글 2022 Print: 표 위 괘선 255.7px(96DPI).
    assert!(
        (table.bbox.y - 255.7).abs() <= 2.0,
        "표 위끝 {:.1} = 한/글 255.7",
        table.bbox.y
    );
}

#[test]
fn wide_table_without_a_leading_field_stays_on_the_first_row() {
    let core = load(NO_FIELD);
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[HOST];
    assert!(
        !para
            .controls
            .iter()
            .any(|control| matches!(control, Control::Field(_))),
        "반례: 누름틀 없음"
    );
    let page = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let table = find_table(&page.root, 16, 16).expect("16×16 본문 표");
    let Some(Control::Table(model)) = para
        .controls
        .iter()
        .find(|control| matches!(control, Control::Table(_)))
    else {
        panic!("문단 5 표");
    };
    let outer_top = px(i32::from(model.outer_margin_top));
    let field = load(FIELD_LED);
    let field_page = field.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let field_table = find_table(&field_page.root, 16, 16).expect("16×16 본문 표");
    let row = px(text_row_advance_hu(&field));

    // 줄 맨 앞 개체는 줄보다 넓어도 그 줄에 남는다: 누름틀 있는 원본보다 정확히 한 줄 위.
    let lift = field_table.bbox.y - table.bbox.y;
    assert!(
        (lift - row).abs() <= 1.0,
        "누름틀 유무의 표 위끝 차 {lift:.1} = 누름틀 줄 {row:.1}"
    );
    assert!(
        table.bbox.y - outer_top >= previous_tables_bottom(&page.root) - 0.5,
        "표는 앞 결재 표 아래"
    );
}

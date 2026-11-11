//! [#8782] 관계형 회귀 — 글자 없는 host 에서 한/글이 저장한 필드 시작 줄(누름틀 시작만 담은 빈
//! 줄)은 표 앞 줄이다. 표는 그 줄 다음 저장 줄에서 시작하고, 쪽을 넘길 때 그 줄은 앞 쪽에 남는다.
//!
//! 주석 갱신: 2026-11-10 05:10 KST(+09:00), 최신 devel 재측정.
//!
//! - 입력 A: `samples/issue8782/stored_field_line_table.hwpx` — 공개 결재문서 36368775(HWPX, 2쪽).
//!   2쪽 문단 9: 글자 없음, 원시 순서 누름틀 시작 → 글자처럼 취급 8×8 표 → 누름틀 끝. 한/글 저장
//!   줄 2개 = 필드 줄(ts 0, vpos 1652, 높이 1100 + 간격 552) + 표 줄(ts 8, vpos 3304).
//! - 입력 B: `samples/issue8782/stored_field_line_page_split.hwpx` — 공개 결재문서 36454509(HWPX,
//!   3쪽). 1쪽 끝 문단 2: 같은 구성, 필드 줄(vpos 64942)은 1쪽 본문(69790HU)에 들고 표 줄(vpos
//!   67102 + 6408)은 넘친다.
//! - 독립 기준: `pdf/issue8782/*-2020.pdf`(한/글 2022 12.0.0.4547 Print, A 2쪽·B 3쪽). A 2쪽 표 위
//!   괘선 102.4px = 본문 위 + (3304 + 바깥 위 여백 140)/75. B 는 필드 줄을 1쪽에 두고 2쪽 제목 표를
//!   본문 맨 위에서 시작한다.
//!
//! 측정: 2026-11-10, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), head `cb6ed9b4b5`(devel `b5064fae1e`
//! 위 이 수정), 글꼴은 합계 64MiB 이내의 굴림·한컴바탕·HY견고딕·맑은고딕 폴더.
//! - A: 2쪽 Native 95.31 / fresh WASM 95.79(수정 전 Native 36.60), 1쪽 100.00 / 99.31. 전체 2쪽 = PDF
//!   2쪽, 전쪽 최저 Native 95.31(2쪽)·fresh WASM 95.79(2쪽).
//! - B: 1쪽 90.91 / 99.26, 2쪽 91.93 / 94.18(수정 전 Native 43.16), 3쪽 99.08 / 100.00(수정 전
//!   0.00). 전체 3쪽 = PDF 3쪽, 전쪽 최저 Native 90.91(1쪽)·fresh WASM 94.18(2쪽).
//! - 증거: `mydocs/pr/assets/issue_8782/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사(관계형): 두 입력 모두 원본 계약(글자 없음, 필드 시작이 표보다 앞, 한/글 저장 줄 2개)을
//! 확인한다. A 는 표 위끝 − 필드 줄 위끝 = 저장 필드 줄 전진 + 바깥 위 여백. B 는 필드 줄이 1쪽에
//! 한 번, 표가 2쪽에 한 번(중복·누락 없음), 2쪽 표 위끝 = 본문 위 + 바깥 위 여백.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const STACKED: &str = "samples/issue8782/stored_field_line_table.hwpx";
const PAGE_SPLIT: &str = "samples/issue8782/stored_field_line_page_split.hwpx";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 문단 `para` 의 표 노드와 글줄 위끝들.
fn owned_nodes(node: &RenderNode, para: usize, tables: &mut Vec<f64>, lines: &mut Vec<f64>) {
    // 본문 흐름만 본다. 표 칸 안 문단도 자기 칸 안 번호를 `para_index` 로 싣는다.
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if table.para_index == Some(para) {
                tables.push(node.bbox.y);
            }
            return;
        }
        RenderNodeType::TextLine(line) if line.para_index == Some(para) => lines.push(node.bbox.y),
        _ => {}
    }
    for child in &node.children {
        owned_nodes(child, para, tables, lines);
    }
}

fn body_top(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(body_top)
}

/// 원본 계약: 글자 없음, 필드 시작이 표보다 앞, 한/글 저장 줄 2개(필드 줄 ts 0 + 표 줄).
fn assert_field_led_stored_host(core: &DocumentCore, para: usize) -> (i32, i32, i16) {
    let host = &core.document().sections[0].paragraphs[para];
    assert!(host.text.is_empty(), "문단 {para}: 글자 없는 host");
    assert!(
        matches!(host.controls.first(), Some(Control::Field(_))),
        "문단 {para}: 필드 시작이 표보다 앞"
    );
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.common.treat_as_char => Some(table),
            _ => None,
        })
        .expect("글자처럼 취급 표");
    assert_eq!(host.line_segs.len(), 2, "문단 {para}: 필드 줄 + 표 줄");
    let (field, row) = (&host.line_segs[0], &host.line_segs[1]);
    assert!(
        field.tag & (1 << 31) == 0 && row.tag & (1 << 31) == 0,
        "한/글 저장 줄"
    );
    assert_eq!(field.text_start, 0);
    assert!(row.text_start > 0, "표 줄은 필드 시작 뒤");
    (
        field.line_height,
        field.line_spacing,
        table.outer_margin_top,
    )
}

#[test]
fn table_starts_on_the_row_after_the_stored_field_line() {
    const HOST: usize = 9;
    let core = load(STACKED);
    let (field_height, field_spacing, outer_top) = assert_field_led_stored_host(&core, HOST);
    let page = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let (mut tables, mut lines) = (Vec::new(), Vec::new());
    owned_nodes(&page.root, HOST, &mut tables, &mut lines);
    let table = *tables.first().expect("2쪽 문단 9 표");
    let field_line = lines.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(field_line.is_finite(), "필드 줄이 2쪽에 그려진다");
    // 표 위끝 − 필드 줄 위끝 = 저장 필드 줄 전진(높이 + 간격) + 바깥 위 여백.
    let expected = px(field_height + field_spacing) + px(i32::from(outer_top));
    let gap = table - field_line;
    assert!(
        (gap - expected).abs() <= 1.0,
        "표 위끝 − 필드 줄 {gap:.1} = 필드 줄 전진 + 바깥 위 여백 {expected:.1}"
    );
}

#[test]
fn stored_field_line_stays_on_the_page_where_it_fits() {
    const HOST: usize = 2;
    let core = load(PAGE_SPLIT);
    let (_, _, outer_top) = assert_field_led_stored_host(&core, HOST);
    let first = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let second = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let (mut first_tables, mut first_lines) = (Vec::new(), Vec::new());
    owned_nodes(&first.root, HOST, &mut first_tables, &mut first_lines);
    let (mut second_tables, mut second_lines) = (Vec::new(), Vec::new());
    owned_nodes(&second.root, HOST, &mut second_tables, &mut second_lines);

    // 쪽 소속: 필드 줄은 1쪽, 표는 2쪽. 중복·누락 없음.
    assert_eq!(first_lines.len(), 1, "필드 줄은 1쪽에 한 번");
    assert!(first_tables.is_empty(), "표는 1쪽에 없다");
    assert_eq!(second_tables.len(), 1, "표는 2쪽에 한 번");
    assert!(second_lines.is_empty(), "필드 줄을 2쪽에 다시 쌓지 않는다");
    // 2쪽 표는 본문 맨 위에서 바깥 위 여백만큼 안쪽에서 시작한다.
    let top = body_top(&second.root).expect("2쪽 본문");
    let expected = top + px(i32::from(outer_top));
    assert!(
        (second_tables[0] - expected).abs() <= 1.0,
        "2쪽 표 위끝 {:.1} = 본문 위 {top:.1} + 바깥 위 여백",
        second_tables[0]
    );
    // 필드 줄은 1쪽 본문 안이다.
    let first_top = body_top(&first.root).expect("1쪽 본문");
    assert!(first_lines[0] > first_top, "필드 줄은 1쪽 본문 안");
}

//! 관계형·고정형 — [#9279] 글자처럼 취급 표 줄 뒤에 같은 문단의 도형 줄이 있으면 문단 뒤 간격은 표 줄 뒤가 아니라 도형 줄 뒤에 온다.
//! 주석 갱신: 2026-11-21 18:23 KST.
//!
//! 입력: 산림청 보도자료 공개 문서(「DMZ의 숨결을 품은 북방계식물, 한 달간 국민과 만나다」, HWPX,
//! `appVersion="10, 0, 0, 13462"`, 3쪽) `samples/issue9279/forest-press-release-tac-table-shape-tail.hwpx`.
//! 기준: 한컴 2024 Print PDF `pdf/issue9279/forest-press-release-tac-table-shape-tail-2024.pdf`(3쪽).
//!
//! 구역 0 문단 14 는 글이 없고 컨트롤 둘을 가진다. 컨트롤 0 은 글자처럼 취급 표(2×6 「담당 부서」)로
//! 저장 줄 0(`vpos=20020`, `lh=3696`, `ls=840`)을, 컨트롤 1 은 글자처럼 취급 사각형(로고 줄)으로 저장
//! 줄 1(`vpos=24556`, `lh=4287`)을 차지한다. 문단 뒤 간격은 1400HU(9.33px)다. 한/글은 두 줄을 저장
//! 간격 4536HU(60.48px)로 놓고 문단 뒤 간격을 마지막 줄 뒤에 둔다.
//! 수정 전 rhwp 는 문단 뒤 간격을 표 줄 뒤에 더해 사각형이 9.33px 아래(간격 69.8px)에 놓였다.
//! - 고정형: 전체 쪽수 3과 표·도형의 위쪽 경계(96 DPI, ±1px).
//! - 관계형: 2쪽에서 문단 14 의 표와 사각형이 한 번씩 그려지고, 표 줄 위(표 위 − 표 바깥 위 여백)에서
//!   사각형 위까지의 거리가 두 저장 줄의 `vpos` 차와 같다(±1px). 사각형은 표 아래에 겹치지 않게 놓인다.
//!
//! 측정(2026-11-21, 통합 코드 `cef9ff9acc`, Visual Sweep 96 DPI·compat 2024):
//! Native/fresh WASM 양쪽 3쪽 모두 raw 100.00%. 2쪽 확대 판독에서 표와 아래 도형의
//! 간격을 확인했다. 독립 PDF 2쪽 페이지 좌상단 기준으로 표 선 y=273.54pt, 도형 채움
//! y=315.86pt를 96 DPI px로 환산했다. 검사는 개체 소유 관계와 함께 해당 좌표를 비교한다.
//! 증거: `mydocs/pr/assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9279/forest-press-release-tac-table-shape-tail.hwpx";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 3;
/// 글자처럼 취급 표와 사각형을 함께 가진 문단.
const HOST_PARA: usize = 14;
const TABLE_CONTROL: usize = 0;
const SHAPE_CONTROL: usize = 1;
const DPI: f64 = 96.0;
const PRINT_TABLE_TOP_PX: f64 = 364.72;
const PRINT_SHAPE_TOP_PX: f64 = 421.14;

/// 본문 흐름에서 문단 14 가 소유한 표와 사각형의 상자를 모은다.
fn collect_owned(node: &RenderNode, tables: &mut Vec<BoundingBox>, shapes: &mut Vec<BoundingBox>) {
    match &node.node_type {
        RenderNodeType::Header | RenderNodeType::Footer => return,
        RenderNodeType::Table(table) => {
            if table.para_index == Some(HOST_PARA) && table.control_index == Some(TABLE_CONTROL) {
                tables.push(node.bbox);
            }
            return;
        }
        RenderNodeType::Rectangle(rect) => {
            if rect.para_index == Some(HOST_PARA) && rect.control_index == Some(SHAPE_CONTROL) {
                shapes.push(node.bbox);
            }
            return;
        }
        _ => {}
    }
    for child in &node.children {
        collect_owned(child, tables, shapes);
    }
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    core.set_hangul2024_compat(true);
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    core
}

#[test]
fn shape_line_after_tac_table_line_keeps_the_stored_line_advance() {
    let core = open();
    let para = &core.document().sections[0].paragraphs[HOST_PARA];
    let Some(Control::Table(table)) = para.controls.get(TABLE_CONTROL) else {
        panic!("문단 14 의 컨트롤 0 은 표여야 한다");
    };
    assert!(
        table.common.treat_as_char,
        "문단 14 의 표는 글자처럼 취급이어야 한다"
    );
    assert_eq!(
        para.line_segs.len(),
        2,
        "문단 14 는 표 줄과 도형 줄, 저장 줄 둘을 가져야 한다"
    );
    let stored_advance_px = hwpunit_to_px(
        para.line_segs[1].vertical_pos - para.line_segs[0].vertical_pos,
        DPI,
    );
    let table_margin_top_px = hwpunit_to_px(i32::from(table.outer_margin_top), DPI);

    let page = core.build_page_render_tree(1).expect("2쪽");
    let (mut tables, mut shapes) = (Vec::new(), Vec::new());
    collect_owned(&page.root, &mut tables, &mut shapes);
    assert_eq!(tables.len(), 1, "문단 14 의 표는 2쪽에 한 번 그려져야 한다");
    assert_eq!(
        shapes.len(),
        1,
        "문단 14 의 사각형은 2쪽에 한 번 그려져야 한다"
    );
    let (table_box, shape_box) = (tables[0], shapes[0]);

    assert!(
        (table_box.y - PRINT_TABLE_TOP_PX).abs() <= 1.0,
        "표 위 {:.2}px, 한컴 Print {PRINT_TABLE_TOP_PX:.2}px",
        table_box.y
    );
    assert!(
        (shape_box.y - PRINT_SHAPE_TOP_PX).abs() <= 1.0,
        "도형 위 {:.2}px, 한컴 Print {PRINT_SHAPE_TOP_PX:.2}px",
        shape_box.y
    );

    assert!(
        shape_box.y >= table_box.y + table_box.height,
        "사각형 위 {:.2} 가 표 아래 {:.2} 보다 위다",
        shape_box.y,
        table_box.y + table_box.height
    );
    let table_line_top = table_box.y - table_margin_top_px;
    let advance = shape_box.y - table_line_top;
    assert!(
        (advance - stored_advance_px).abs() <= 1.0,
        "표 줄 위 {table_line_top:.2} 에서 사각형 위 {:.2} 까지 {advance:.2}px 는 저장 줄 전진 \
         {stored_advance_px:.2}px 와 같아야 한다(문단 뒤 간격은 도형 줄 뒤에 온다)",
        shape_box.y
    );
}

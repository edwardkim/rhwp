// Issue #7441 — 제보 조건·수치를 옮긴 합성 재현 probe (회귀 테스트 아님).
//
// 제보 원본 HWP 가 제공되지 않아, 이슈 본문의 조건과 수치를 공개 편집 API 로 그대로 옮긴
// 문서를 만든다. 한컴 기준 PDF 가 없는 합성 입력이라 정식 회귀(`tests/cases/`)로 두지 않고
// 조사 근거로 보존한다. 원본을 받으면 같은 출력을 원본과 대조하는 데 쓴다.
//
// - 용지: A4 세로, 한컴 기본 여백(머리말 15·위 20·아래 15·꼬리말 15mm)
//   → 본문 시작 132.3px, 꼬리말 시작 1009.1px(제보의 "꼬리말 시작 y≈1009px").
// - 표: 5행×2열, 쪽나눔 RowBreak, 글자처럼 아님, 글 뒤로, 용지 기준 위 정렬, 쪽 영역 안으로
//   제한, 용지 y 728px, 행 높이 48·48·48·59·59px(합 262px, 앞 3행 144px — 제보의
//   "표 높이≈262px", "앞 3행(약 144px)"). 표 뒤 강제 쪽 나누기로 2쪽을 만든다.
//
// 관측(2026-09-28, base eb9142dd7): 수정 전에는 1쪽에 행 0–2, 2쪽 상단에 행 3–4
// (`RHWP_TABLE_DRIFT` 의 OVERLAY_CONT start_row=3 remaining=118.0 room=148.8) — 제보의
// "실제 결과"와 같다. 수정 후에는 1쪽에 행 0–4 가 모두 있고 2쪽에 조각이 없다.
//
// 출력: output/poc/issue_7441/reporter_synthetic.hwp 와 쪽별 표 행 목록(stdout).
// 실행: 루트 Cargo.toml 에 등록된 example 이 아니다. 필요하면 임시 [[example]] 로 등록해
// `cargo run --release --example repro_7441_reporter_geometry` 로 실행하고 등록은 되돌린다.
use std::collections::BTreeSet;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const ROW_HEIGHTS_PX: [f64; 5] = [48.0, 48.0, 48.0, 59.0, 59.0];
const TABLE_TOP_PX: f64 = 728.0;
const OUT_DIR: &str = "output/poc/issue_7441";

fn px_to_hu(v: f64) -> i64 {
    (v * 7200.0 / 96.0).round() as i64
}

fn table_rows(node: &RenderNode, key: (usize, usize), rows: &mut BTreeSet<u16>, inside: bool) {
    let inside = inside
        || matches!(&node.node_type, RenderNodeType::Table(t)
            if t.cell_context.is_none() && t.para_index == Some(key.0) && t.control_index == Some(key.1));
    if inside {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            rows.extend(cell.row..cell.row + cell.row_span.max(1));
            return;
        }
    }
    for child in &node.children {
        table_rows(child, key, rows, inside);
    }
}

fn main() {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.create_table_native(0, 0, 0, 5, 2)
        .expect("create table");
    let (para, ctrl) = doc.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .expect("inserted table");
    // 표 아래 빈 문단 앞에서 쪽을 나눠 2쪽을 만든다.
    doc.insert_page_break_native(0, para + 1, 0)
        .expect("page break");

    let props = serde_json::json!({
        "treatAsChar": false,
        "textWrap": "BehindText",
        "vertRelTo": "Paper",
        "vertAlign": "Top",
        "vertOffset": px_to_hu(TABLE_TOP_PX),
        "pageBreak": 2,
        "restrictInPage": true,
    });
    doc.set_table_properties_native(0, para, ctrl, &props.to_string())
        .expect("table props");
    for (row, height) in ROW_HEIGHTS_PX.iter().enumerate() {
        for col in 0..2 {
            let cell = serde_json::json!({ "height": px_to_hu(*height) }).to_string();
            doc.set_cell_properties_native(0, para, ctrl, row * 2 + col, &cell)
                .expect("cell height");
        }
    }
    let page = serde_json::json!({
        "width": 59528, "height": 84188, "landscape": false,
        "marginLeft": 8504, "marginRight": 8504,
        "marginHeader": 4252, "marginTop": 5669,
        "marginBottom": 4252, "marginFooter": 4252,
    });
    doc.set_page_def_native(0, &page.to_string())
        .expect("page def");

    std::fs::create_dir_all(OUT_DIR).expect("output dir");
    let dst = format!("{OUT_DIR}/reporter_synthetic.hwp");
    std::fs::write(&dst, doc.export_hwp_native().expect("export hwp")).expect("write hwp");
    println!("saved: {dst} (pages={})", doc.page_count());

    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        let mut rows = BTreeSet::new();
        table_rows(&tree.root, (para, ctrl), &mut rows, false);
        println!("page {}: table rows {:?}", page + 1, rows);
    }
}

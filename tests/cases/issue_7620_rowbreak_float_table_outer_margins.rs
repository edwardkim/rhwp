//! Issue #7620 — 쪽 경계 나눔(RowBreak)이 켜진 자리 차지 표만 든 문단의 통째 배치.
//!
//! 쪽 경계 나눔은 분할을 허용할 뿐 그 표가 실제로 나뉜다는 뜻이 아니다. 통째로 들어가는
//! 빈 개체 앵커 표는 쪽 나눔 None 표처럼 문단 윗변 + 바깥 여백 위에 서고, 다음 줄은
//! 표 아랫변 + 바깥 여백 아래에서 시작한다.
//!
//! 가드하는 축:
//!   ① 재현 문서 `samples/issue7620/a-rowbreak-float.hwp`(devel 의 web WASM `generate.mjs` 로 만든
//!      HWP5). 기준은 같은 문서의 한컴독스 인쇄 `pdf/issue7620/a-rowbreak-float-hancomdocs.pdf`
//!      (표 괘선 157–191px, 표 뒤 문단 잉크 217px, 96dpi). 종전에는 표가 문단 윗변에 붙고
//!      다음 줄이 표 아랫변에서 시작했다(153.6 / 187.8px)
//!   ② 문단 위·아래 간격이 있는 한컴 저장본 대조군 `hwpctl_API_v2.4.hwp` 60쪽 문단 1465
//!      (RowBreak, 간격 위·아래 500HU). 저장 첫 줄 vpos 는 위 간격을 포함하므로 표는 그보다
//!      위인 문단 윗변에 서고, 문단 아래 간격은 표 상자에 들어가지 않는다. 기대값은 한컴이
//!      저장한 사다리 `다음 문단 vpos = 문단 윗변 + 바깥 여백 위 + 표 높이 + 바깥 여백 아래` 에서
//!      정한다(한컴 인쇄 표 윗변 210.5px). 종전 코드는 이 관계를 지켰다
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::wasm_api::HwpDocument;

/// 위치 비교 허용치(px). 저장 HWPUNIT 정수와 렌더 트리 소수 한 자리 반올림을 덮는다.
const TOL_PX: f64 = 0.6;

fn load(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("{rel} 파싱: {e:?}"))
}

/// 96dpi 에서 HWPUNIT → px.
fn hu_px(hu: i32) -> f64 {
    f64::from(hu) / 75.0
}

/// 고정 입력의 전제: 비TAC · 자리 차지 · 문단 기준 · 오프셋 0 · RowBreak · 폭 0 앵커 줄의 빈 문단.
fn host_table(doc: &HwpDocument, rel: &str, host: usize) -> Table {
    let para = &doc.document().sections[0].paragraphs[host];
    let t = match &para.controls[..] {
        [Control::Table(t)] => (**t).clone(),
        other => panic!("{rel}: 전제 — 표 문단에 표 하나만 있어야 한다 ({other:?})"),
    };
    assert!(para.text.trim().is_empty(), "{rel}: 전제 — 빈 문단");
    assert!(!t.common.treat_as_char, "{rel}: 전제 — 비TAC");
    assert_eq!(
        t.common.text_wrap,
        TextWrap::TopAndBottom,
        "{rel}: 전제 — 자리 차지"
    );
    assert_eq!(
        t.common.vert_rel_to,
        VertRelTo::Para,
        "{rel}: 전제 — 문단 기준"
    );
    assert_eq!(t.common.vertical_offset, 0, "{rel}: 전제 — 세로 오프셋 0");
    assert_eq!(
        t.page_break,
        TablePageBreak::RowBreak,
        "{rel}: 전제 — 쪽 나눔 RowBreak"
    );
    assert_eq!(
        para.line_segs[0].segment_width, 0,
        "{rel}: 전제 — 폭 0 앵커 줄"
    );
    t
}

/// 문단 위·아래 간격(HWPUNIT). HWP5 문단 모양은 두 배 값으로 저장한다.
fn paragraph_spacing_hu(doc: &HwpDocument, para: usize) -> (i32, i32) {
    let document = doc.document();
    let shape = &document.doc_info.para_shapes
        [document.sections[0].paragraphs[para].para_shape_id as usize];
    (shape.spacing_before / 2, shape.spacing_after / 2)
}

/// `page` 쪽의 (본문 윗변, 표 상자 (y, h), 표 문단 다음 문단의 첫 줄 윗변) px.
fn geometry(doc: &HwpDocument, page: u32, host: usize) -> (f64, (f64, f64), f64) {
    fn bbox(node: &serde_json::Value) -> (f64, f64) {
        let b = &node["bbox"];
        (
            b["y"].as_f64().unwrap_or(f64::NAN),
            b["h"].as_f64().unwrap_or(f64::NAN),
        )
    }
    fn walk(
        node: &serde_json::Value,
        host: u64,
        body: &mut Option<f64>,
        table: &mut Option<(f64, f64)>,
        next: &mut Option<f64>,
    ) {
        match node["type"].as_str() {
            Some("Body") if body.is_none() => *body = Some(bbox(node).0),
            Some("Table") if node["pi"].as_u64() == Some(host) && table.is_none() => {
                *table = Some(bbox(node));
                return; // 셀 안의 줄은 본문 문단이 아니다
            }
            Some("TextLine") if next.is_none() && node["pi"].as_u64() == Some(host + 1) => {
                *next = Some(bbox(node).0);
            }
            _ => {}
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, host, body, table, next);
        }
    }
    let json = doc.get_page_render_tree(page).expect("render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
    let (mut body, mut table, mut next) = (None, None, None);
    walk(&tree, host as u64, &mut body, &mut table, &mut next);
    (
        body.expect("Body 노드가 없다"),
        table.unwrap_or_else(|| panic!("{}쪽에 문단 {host} 표 노드가 없다", page + 1)),
        next.expect("표 문단 다음 문단의 줄이 없다"),
    )
}

fn assert_near(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOL_PX,
        "{what}: {actual:.1}px 이 기대 {expected:.1}px 과 다르다",
    );
}

#[test]
fn issue_7620_rowbreak_float_table_keeps_outer_margins() {
    const REL: &str = "samples/issue7620/a-rowbreak-float.hwp";
    const HOST: usize = 1;
    let doc = load(REL);
    let t = host_table(&doc, REL, HOST);
    // 위 간격이 없으므로 저장 첫 줄 vpos 가 곧 문단 윗변이다.
    assert_eq!(
        paragraph_spacing_hu(&doc, HOST).0,
        0,
        "{REL}: 전제 — 문단 위 간격 0"
    );
    let host_vpos = doc.document().sections[0].paragraphs[HOST].line_segs[0].vertical_pos;

    let (body_y, (table_y, table_h), next_y) = geometry(&doc, 0, HOST);
    assert_near(
        "표 윗변 = 문단 윗변 + 바깥 여백 위",
        table_y,
        body_y + hu_px(host_vpos) + hu_px(i32::from(t.outer_margin_top)),
    );
    assert_near(
        "다음 문단 윗변 = 표 아랫변 + 바깥 여백 아래",
        next_y,
        table_y + table_h + hu_px(i32::from(t.outer_margin_bottom)),
    );
}

/// 대조군: 문단 위·아래 간격이 있는 한컴 저장본. 종전에도 한컴과 같았다.
#[test]
fn issue_7620_spaced_host_keeps_paragraph_top_control() {
    const REL: &str = "samples/hwpctl_API_v2.4.hwp";
    const PAGE: u32 = 59;
    const HOST: usize = 1465;
    let doc = load(REL);
    let t = host_table(&doc, REL, HOST);
    let (sb, sa) = paragraph_spacing_hu(&doc, HOST);
    assert!(
        sb > 0 && sa > 0,
        "{REL}: 전제 — 문단 위·아래 간격이 있다 ({sb}, {sa})"
    );
    let paras = &doc.document().sections[0].paragraphs;
    let next_vpos = paras[HOST + 1].line_segs[0].vertical_pos;
    // 한컴 저장 사다리에서 문단 윗변을 정한다. 저장 첫 줄 vpos 는 그보다 위 간격만큼 아래다.
    let paragraph_top = next_vpos
        - i32::from(t.outer_margin_top)
        - t.common.height as i32
        - i32::from(t.outer_margin_bottom);
    assert_eq!(
        paras[HOST].line_segs[0].vertical_pos - paragraph_top,
        sb,
        "{REL}: 전제 — 저장 첫 줄 vpos = 문단 윗변 + 위 간격"
    );

    let (body_y, (table_y, table_h), next_y) = geometry(&doc, PAGE, HOST);
    assert_near(
        "표 윗변 = 문단 윗변 + 바깥 여백 위",
        table_y,
        body_y + hu_px(paragraph_top + i32::from(t.outer_margin_top)),
    );
    assert_near(
        "다음 문단 윗변 = 표 아랫변 + 바깥 여백 아래 (문단 아래 간격 없음)",
        next_y,
        table_y + table_h + hu_px(i32::from(t.outer_margin_bottom)),
    );
}

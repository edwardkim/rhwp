//! Issue #7621 — 자리 차지(TopAndBottom) 표만 든 문단 바로 뒤 글자 문단의 재열기 배치.
//!
//! 재현 문서는 `samples/issue7621/`(devel 의 web WASM `generate.mjs` 로 만든 HWP5), 기준은
//! 같은 문서를 한컴독스 인쇄로 출력한 `pdf/issue7621/*-hancomdocs.pdf` 다. 두 문서 모두
//! 앞 문단 · 표 문단(폭 0 앵커 줄) · 다음 문단 · 뒤 문단 구성이고, 표는 비TAC · 문단 기준 ·
//! 오프셋 0 · 바깥 여백 283HU · 쪽 나눔 None 이다. 한컴은 다음 문단이 빈 문단이든 글자
//! 문단이든 표 아랫변 + 바깥 여백 아래에서 시작한다.
//!
//! 가드하는 축:
//!   ① 폭 0 앵커 줄 문단 바로 뒤가 글자 문단이어도 띠 아래 host 줄을 더하지 않는다.
//!      종전에는 저장 사다리(다음 vpos − host vpos = lh + ls)를 #6147 의 띠 아래 host 줄
//!      증거로 읽어 한 줄 아래에 그렸다(216.7px, 한컴 195px). 폭 0 앵커 줄은 #7470 대로
//!      띠에 흡수되어 별도로 전진하지 않는다
//!   ② 다음 문단이 빈 문단인 정상 대조군은 종전과 같다
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::wasm_api::HwpDocument;

/// 표 문단 번호. 두 문서 모두 앞 문단 다음이다.
const HOST: usize = 1;
/// 위치 비교 허용치(px). 저장 HWPUNIT 정수와 렌더 트리 소수 한 자리 반올림을 덮는다.
const TOL_PX: f64 = 0.6;

fn load(name: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7621")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("{name} 파싱: {e:?}"))
}

/// 96dpi 에서 HWPUNIT → px.
fn hu_px(hu: i32) -> f64 {
    f64::from(hu) / 75.0
}

fn host_table(doc: &HwpDocument) -> Table {
    match &doc.document().sections[0].paragraphs[HOST].controls[..] {
        [Control::Table(t)] => (**t).clone(),
        other => panic!("전제: 표 문단에 표 하나만 있어야 한다 ({other:?})"),
    }
}

/// 첫 쪽의 (표 상자 (y, h), 표 문단 다음 문단의 첫 줄 윗변) px.
fn geometry(doc: &HwpDocument) -> ((f64, f64), f64) {
    fn bbox(node: &serde_json::Value) -> (f64, f64) {
        let b = &node["bbox"];
        (
            b["y"].as_f64().unwrap_or(f64::NAN),
            b["h"].as_f64().unwrap_or(f64::NAN),
        )
    }
    fn walk(node: &serde_json::Value, table: &mut Option<(f64, f64)>, next: &mut Option<f64>) {
        match node["type"].as_str() {
            Some("Table") if node["pi"].as_u64() == Some(HOST as u64) && table.is_none() => {
                *table = Some(bbox(node));
                return; // 셀 안의 줄은 본문 문단이 아니다
            }
            Some("TextLine") if next.is_none() && node["pi"].as_u64() == Some(HOST as u64 + 1) => {
                *next = Some(bbox(node).0);
            }
            _ => {}
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, table, next);
        }
    }
    let json = doc.get_page_render_tree(0).expect("render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
    let (mut table, mut next) = (None, None);
    walk(&tree, &mut table, &mut next);
    (
        table.expect("표 노드가 없다"),
        next.expect("표 문단 다음 문단의 줄이 없다"),
    )
}

/// 한컴 인쇄와 같은 관계: 표 문단 다음 문단 = 표 아랫변 + 바깥 여백 아래.
fn assert_next_starts_below_band(name: &str, next_has_text: bool) {
    let doc = load(name);
    let t = host_table(&doc);
    let paras = &doc.document().sections[0].paragraphs;
    let host = &paras[HOST].line_segs[0];
    // 고정 입력의 전제.
    assert!(!t.common.treat_as_char, "{name}: 전제 — 비TAC");
    assert_eq!(
        t.common.text_wrap,
        TextWrap::TopAndBottom,
        "{name}: 전제 — 자리 차지"
    );
    assert_eq!(
        t.common.vert_rel_to,
        VertRelTo::Para,
        "{name}: 전제 — 문단 기준"
    );
    assert_eq!(t.common.vertical_offset, 0, "{name}: 전제 — 세로 오프셋 0");
    assert_eq!(
        t.page_break,
        TablePageBreak::None,
        "{name}: 전제 — 쪽 나눔 None"
    );
    assert_eq!(host.segment_width, 0, "{name}: 전제 — 폭 0 앵커 줄");
    assert_eq!(
        paras[HOST + 1].text.trim().is_empty(),
        !next_has_text,
        "{name}: 전제 — 다음 문단의 글자 유무"
    );

    let ((table_y, table_h), next_y) = geometry(&doc);
    let expected_next = table_y + table_h + hu_px(i32::from(t.outer_margin_bottom));
    assert!(
        (next_y - expected_next).abs() <= TOL_PX,
        "{name}: 다음 문단 윗변 {next_y:.1}px 이 표 아랫변 + 바깥 여백 아래 {expected_next:.1}px 과 다르다",
    );
}

#[test]
fn issue_7621_text_after_float_only_host_starts_below_band() {
    assert_next_starts_below_band("b-text-after-float.hwp", true);
}

/// 정상 대조군: 다음 문단이 빈 문단이면 종전에도 한컴과 같았다.
#[test]
fn issue_7621_empty_after_float_only_host_control() {
    assert_next_starts_below_band("control-float.hwp", false);
}

//! Issue #4326 — 중첩 표 PartialTable 조각의 행 좌표계가 데이터에 없어
//! 렌더러 되추론이 틀리는 문제 회귀.
//!
//! `samples/76076_regulatory_analysis.hwp` para 36은 빈 셀 하나짜리 투명 1×1
//! RowBreak 래퍼가 3×3 중첩 표(`구분/장점/단점` 제목행 + `현행유지안`/`규제대안`
//! 본문 2행)를 감싼다. 페이지네이터는 이 래퍼를 벗기고 중첩 표의 행 기준으로
//! 페이지를 나누지만, 예전에는 그 `PartialTable` 조각이 바깥 컨트롤의
//! `para_index`/`control_index`로만 식별되고 "이 행 좌표가 바깥 1행 래퍼
//! 자신의 것인지 벗겨낸 중첩 표의 것인지"는 어디에도 기록되지 않았다.
//! 렌더러는 `end_row <= table.row_count`로 값을 되추론했는데, 중첩 표의 첫
//! 행만 담은 조각(`end_row == 1`)이 바깥 1행 래퍼 자신의 행 0과 값으로
//! 구별되지 않아 잘못 라우팅됐다 — 표 전체가 앞 페이지에 다시 그려지고
//! (본문 영역을 넘어 꼬리말까지 침범), 뒤 페이지에 마지막 행들이 중복
//! 인쇄됐다.
//!
//! `page_def.margin_bottom`를 조금 늘리면(+500 HWPUNIT, 약 1.8mm) 조각 경계가
//! 중첩 표의 행 1 바로 앞으로 옮겨가 이 경로를 그대로 재현한다. 이 테스트는
//! 그 여백에서 앞 쪽에 중첩 표의 행 하나만 담기고, 뒤 쪽에 나머지 행이
//! 중복 없이 이어지는지 고정한다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/76076_regulatory_analysis.hwp";
const TARGET_PARA: usize = 36;

fn load() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

/// 여백을 조정한 재-페이지네이션 결과. `set_document`는 테스트/네이티브
/// 전용 API로, IR을 다시 넣고 dirty 표시 + 전체 재페이지네이션까지 수행한다.
fn load_with_margin_bottom_delta(delta_hu: i32) -> HwpDocument {
    let mut core = load();
    let mut doc = core.document().clone();
    for section in &mut doc.sections {
        let pd = &mut section.section_def.page_def;
        pd.margin_bottom = (pd.margin_bottom as i32 + delta_hu).max(0) as u32;
    }
    core.set_document(doc);
    core
}

fn collect_table_text(node: &RenderNode, para_index: usize, out: &mut String) {
    if let RenderNodeType::Table(meta) = &node.node_type {
        if meta.para_index == Some(para_index) {
            collect_all_text(node, out);
            return;
        }
    }
    for child in &node.children {
        collect_table_text(child, para_index, out);
    }
}

fn collect_all_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_all_text(child, out);
    }
}

/// para=36 표 fragment의 렌더 텍스트(공백 제거) — 페이지에 없으면 빈 문자열.
fn table_fragment_text(doc: &HwpDocument, page: u32) -> String {
    let tree = doc
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page {page}: {e}"));
    let mut text = String::new();
    collect_table_text(&tree.root, TARGET_PARA, &mut text);
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// [Issue #4326] 여백을 +1.8mm 늘려 조각 경계가 중첩 표의 행 1 앞으로
/// 옮겨가는 조건(`end_row == 1`이 중첩 표 좌표인 경우)을 직접 재현한다.
/// 고친 코드는 앞 쪽에 제목행만, 뒤 쪽에 나머지 두 행만 중복 없이 담아야 한다.
#[test]
fn nested_wrapper_row_cursor_does_not_duplicate_or_overflow() {
    let doc = load_with_margin_bottom_delta(500);

    let mut owner_page = None;
    let mut combined = String::new();
    for page in 0..doc.page_count() {
        let text = table_fragment_text(&doc, page);
        if text.is_empty() {
            continue;
        }
        if owner_page.is_none() {
            owner_page = Some(page);
        }
        combined.push_str(&text);
    }
    let first_page = owner_page.expect("para=36 표가 어느 페이지에도 렌더되지 않음");
    let page_a = table_fragment_text(&doc, first_page);
    let page_b = table_fragment_text(&doc, first_page + 1);

    // 제목행(구분/장점/단점)은 앞 조각에만 있어야 한다 — 중첩 표는 항상 행0부터 시작.
    assert!(
        page_a.contains("구분") && page_a.contains("장점") && page_a.contains("단점"),
        "표 제목행이 첫 fragment(page {first_page})에 있어야 함: {page_a:?}"
    );
    assert!(
        !page_b.contains("구분"),
        "표 제목행이 다음 fragment(page {})에 중복되면 안 됨: {page_b:?}",
        first_page + 1
    );

    // 본문 두 행("현행유지안"/"규제대안")은 어느 한쪽에만 있어야 한다 —
    // 고친 코드는 이 여백에서 둘 다 뒤 fragment로 보낸다(제목행만 앞).
    let row1_in_a = page_a.contains("현행유지안");
    let row1_in_b = page_b.contains("현행유지안");
    assert_ne!(
        row1_in_a, row1_in_b,
        "'현행유지안' 행이 두 fragment에 걸쳐 중복되거나 누락됨: page_a={page_a:?}, page_b={page_b:?}"
    );
    let row2_in_a = page_a.contains("규제대안");
    let row2_in_b = page_b.contains("규제대안");
    assert_ne!(
        row2_in_a, row2_in_b,
        "'규제대안' 행이 두 fragment에 걸쳐 중복되거나 누락됨: page_a={page_a:?}, page_b={page_b:?}"
    );

    // [핵심 회귀] 되추론 버그는 앞 fragment에 표 세 행을 통째로(제목+본문 2행)
    // 다시 그렸다 — 앞 fragment 텍스트 길이가 뒤 fragment와 맞먹거나 넘었다.
    // 고친 코드는 앞 fragment에 제목행 하나만 담아 훨씬 짧다.
    assert!(
        page_a.chars().count() < page_b.chars().count(),
        "앞 fragment가 뒤 fragment만큼 길다 — 표 전체가 앞 페이지에 다시 그려진 \
         되추론 버그로 회귀했을 수 있음: page_a={page_a:?} ({} chars), page_b={page_b:?} ({} chars)",
        page_a.chars().count(),
        page_b.chars().count()
    );

    // 각 마커는 문서 전체에서 정확히 한 번만 나타나야 한다(위치 무관 총 중복 검사).
    for marker in ["구분", "현행유지안", "규제대안"] {
        let count = combined.matches(marker).count();
        assert_eq!(
            count, 1,
            "'{marker}' 마커가 {count}번 나타남(정확히 1번이어야 함): combined={combined:?}"
        );
    }
}

/// 기준 여백(문서 원본 그대로)에서는 이미 정상이었다 — 회귀 없음을 함께 고정.
#[test]
fn nested_wrapper_row_cursor_baseline_margin_stays_correct() {
    let doc = load_with_margin_bottom_delta(0);

    let mut combined = String::new();
    for page in 0..doc.page_count() {
        combined.push_str(&table_fragment_text(&doc, page));
    }
    for marker in ["구분", "현행유지안", "규제대안"] {
        let count = combined.matches(marker).count();
        assert_eq!(
            count, 1,
            "기준 여백에서 '{marker}' 마커가 {count}번 나타남(정확히 1번이어야 함)"
        );
    }
}

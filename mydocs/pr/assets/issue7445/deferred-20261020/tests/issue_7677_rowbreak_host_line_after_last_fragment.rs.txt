//! [#7677] 쪽을 넘긴 문단(PARA) 기준 자리차지(TOP_AND_BOTTOM) RowBreak 표의 **마지막 조각 뒤
//! host 글줄**.
//!
//! 한/글 규칙: host 첫 글줄은 마지막 표 조각 하단 + 표 바깥 아래 여백에 선다. 문단 앞 간격은
//! 문단 상단(첫 조각이 있던 쪽)에서 이미 소비됐으므로 다시 더하지 않는다.
//!
//! 입력 `samples/issue7677/float_push_spacing_before_continuation-2024.hwp` 는 저장소 생성기
//! `samples/float_push_spacing_before/make_float_push_spacing_before_boundaries.py --continuation-s`
//! 의 HWPX(동봉, 저장 줄 없음)를 한/글 2024(13.0.0.564)로 HWP 저장한 것이고(저장 줄 있음), 정본
//! `pdf/issue7677/float_push_spacing_before_continuation-2024.pdf` 는 같은 한/글의 Print 출력이다.
//! S 쪽(6쪽): 앞 간격 1400HU host 의 20행 표(바깥 여백 위 140/아래 852HU)가 5쪽 아래에서 시작해
//! 6쪽으로 넘어간다. (`--continuation` 의 C 쪽은 #7678 의 쪽 끝 넘침이라 여기 넣지 않았다.)
//!
//! ```text
//!   6쪽 (96dpi)            정본      수정 전 rhwp
//!   마지막 조각 하단        362.5     363.2
//!   「S. 넘어간 표 조각 뒤」 373.8     381.7   (+11.3 = 852HU  ↔  +18.5 = 앞 간격 1400HU)
//!   「S 뒤 문단이다.」       395.2     403.0
//! ```
//!
//! 수정 전에는 typeset 이 그 host 줄을 예약하지 않아(6쪽 항목이 `PartialTable` 다음 바로
//! `FullParagraph`) layout 만 조각 끝에서 문단을 새로 시작하듯 앞 간격을 더해 그렸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue7677/float_push_spacing_before_continuation-2024.hwp";
/// host 문단(「S. …」)과 다음 문단의 구역 내 문단 번호.
const HOST_PI: u64 = 87;
const NEXT_PI: u64 = 88;
/// 표 바깥 아래 여백 852HU = 11.36px (96dpi).
const OUTER_BOTTOM_PX: f64 = 852.0 * 96.0 / 7200.0;

fn page_tree(page: u32) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("표본 파싱");
    assert_eq!(doc.page_count(), 6, "정본 PDF 와 같은 6쪽");
    serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap()
}

/// 본문 직속 Table 과 TextLine 을 (종류, pi, 윗변, 아랫변) 으로 모은다. 표 안은 내려가지 않는다.
fn items(tree: &Value) -> Vec<(&'static str, u64, f64, f64)> {
    fn walk(node: &Value, in_table: bool, out: &mut Vec<(&'static str, u64, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        let bbox = |n: &Value| {
            let y = n["bbox"]["y"].as_f64().unwrap();
            (y, y + n["bbox"]["h"].as_f64().unwrap())
        };
        let mut inside = in_table;
        if kind == "Table" {
            let (y, b) = bbox(node);
            out.push(("Table", node["pi"].as_u64().unwrap_or(u64::MAX), y, b));
            inside = true;
        } else if kind == "TextLine" && !in_table {
            let (y, b) = bbox(node);
            out.push(("Line", node["pi"].as_u64().unwrap_or(u64::MAX), y, b));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, inside, out);
        }
    }
    let mut out = Vec::new();
    walk(tree, false, &mut out);
    out
}

#[test]
fn host_line_after_the_last_fragment_sits_at_outer_bottom_not_spacing_before() {
    let tree = page_tree(5);
    let items = items(&tree);
    let fragment_bottom = items
        .iter()
        .find(|i| i.0 == "Table" && i.1 == HOST_PI)
        .map(|i| i.3)
        .unwrap_or_else(|| panic!("6쪽에 표 조각(pi={HOST_PI})이 있어야 한다: {items:?}"));
    let host = items
        .iter()
        .find(|i| i.0 == "Line" && i.1 == HOST_PI)
        .unwrap_or_else(|| panic!("6쪽에 host 글줄(pi={HOST_PI})이 있어야 한다: {items:?}"));
    let next = items
        .iter()
        .find(|i| i.0 == "Line" && i.1 == NEXT_PI)
        .unwrap_or_else(|| panic!("6쪽에 다음 문단(pi={NEXT_PI})이 있어야 한다: {items:?}"));
    let gap = host.2 - fragment_bottom;
    assert!(
        (gap - OUTER_BOTTOM_PX).abs() <= 1.0,
        "host 첫 글줄은 마지막 조각 하단 + 바깥 아래 여백 {OUTER_BOTTOM_PX:.1}px 에 서야 한다 — \
         조각 하단 {fragment_bottom:.1}, host {:.1} (gap {gap:.1}; 수정 전 18.5 = 앞 간격 1400HU)",
        host.2
    );
    assert!(
        next.2 >= host.3 - 0.5 && next.2 - host.2 <= 24.0,
        "다음 문단 {:.1} 은 host 줄 {:.1}~{:.1} 바로 뒤여야 한다",
        next.2,
        host.2,
        host.3
    );
}

#[test]
fn host_text_is_not_drawn_twice_across_the_split() {
    // 호스트 글줄은 마지막 조각이 있는 6쪽에만 있고, 첫 조각 쪽(5쪽)에는 없다.
    let p5 = items(&page_tree(4));
    assert!(
        p5.iter().any(|i| i.0 == "Table" && i.1 == HOST_PI),
        "5쪽에 첫 조각이 있어야 한다: {p5:?}"
    );
    assert!(
        !p5.iter().any(|i| i.0 == "Line" && i.1 == HOST_PI),
        "5쪽에 host 글줄이 그려지면 안 된다(표 뒤 글): {p5:?}"
    );
    let p6 = items(&page_tree(5));
    assert_eq!(
        p6.iter()
            .filter(|i| i.0 == "Line" && i.1 == HOST_PI)
            .count(),
        1,
        "6쪽 host 글줄은 한 번만 그린다: {p6:?}"
    );
}

/// 저장 줄이 없는 HWPX(생성기 출력 그대로)도 같은 규칙이다 — 쪽 나눔은 한/글 저장본과 다를 수
/// 있어 좌표가 아니라 **조각 하단과 host 줄의 관계**만 본다.
#[test]
fn hwpx_without_stored_lines_follows_the_same_outer_bottom_rule() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7677/float_push_spacing_before_continuation.hwpx");
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    let mut seen = 0;
    for page in 0..doc.page_count() {
        let tree: Value =
            serde_json::from_str(&doc.get_page_render_tree(page as u32).unwrap()).unwrap();
        let items = items(&tree);
        let Some(fragment) = items.iter().find(|i| i.0 == "Table" && i.1 == HOST_PI) else {
            continue;
        };
        let Some(host) = items.iter().find(|i| i.0 == "Line" && i.1 == HOST_PI) else {
            continue;
        };
        seen += 1;
        let gap = host.2 - fragment.3;
        assert!(
            (gap - OUTER_BOTTOM_PX).abs() <= 1.0,
            "{}쪽: host 글줄은 마지막 조각 하단 + 바깥 아래 여백에 서야 한다 (gap {gap:.1})",
            page + 1
        );
    }
    assert_eq!(seen, 1, "표 조각과 host 글줄이 같은 쪽에 한 번 있어야 한다");
}

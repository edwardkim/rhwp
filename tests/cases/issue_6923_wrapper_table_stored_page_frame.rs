//! [#6923] 본문을 감싼 1칸 표가 한/글이 적어 둔 쪽 프레임을 지나쳐 용지 밖까지 채운다.
//!
//! ## 무엇이 문제였나
//!
//! `148738070` 보도자료는 본문 전체(87문단)를 1칸 `RowBreak` 표로 감싼 서식이라 쪽 나눔이
//! **칸 안에서** 일어난다. 한/글이 저장한 사다리는 4쪽 끝을 **빈 문단의 vpos 되감김**으로
//! 적어 두었다.
//!
//! ```text
//!   p[70] vpos=66790 lh=1400 → 끝 68190HU (= 909px, 예산의 98%)
//!   p[71] vpos=0                      ← 한/글이 여기서 쪽을 넘긴다
//!   p[72] vpos=2088  (4 기대효과 및 향후계획 제목 상자)
//! ```
//!
//! 그런데 빈 문단의 되감김은 `#1488` 계약대로 하드 브레이크로 올리지 않는다(기계 문서의
//! 촘촘한 오버레이 리셋이 쪽을 양산했다). 그래서 조각 컷이 그 경계를 지나쳐 다음 쪽 몫인
//! 제목 상자와 5줄을 4쪽에 실었고, 마지막 줄이 **용지 밖**(1134.9px, 용지 1122.5)까지 갔다.
//!
//! ## 수정
//!
//! HWPX 쪽에 이미 있던 판별(`#5880`: 되감김까지 쌓인 높이가 예산의 ≥70%면 저장 쪽 프레임)을
//! HWP5 저장 조판에도 준다. 신호는 `stored_frame_break_before` 가 아니라 **되감김 기하**
//! (`page_frame_reset_before`)다 — 전자는 겹치는 줄 상자(이 문서 p46→p47)도 참이라 쪽
//! 경계가 아닌 자리에서 끊겨 쪽수가 7→8로 늘었다(실측).
//!
//! ## 정답지
//!
//! `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf`
//! (한/글 2020 11.0.0.9136, 7쪽). 4쪽은 법조문 표(`1. 거짓·과장의표시·광고`)에서 끝나고
//! **5쪽이 `4 기대효과 및 향후계획` 제목으로 시작**한다.
//!
//! ## 중첩 표의 저장 줄 소속 (같은 이슈의 둘째 축)
//!
//! 한/글이 저장한 사다리는 표를 소유한 줄을 따로 적는다(p69: ls[0] 49113HU 글줄 ·
//! ls[1] 51229HU 표 밴드). 종전 렌더는 문단 첫 줄 좌표에 표를 앉혀 앞 글줄 위로
//! 28.2px 올라왔다 — 4쪽 `□ 적용법조` 줄(735.0..753.7)과 법조문 표(740.0)가 겹쳤다
//! (글자 겹침 13건). 정본은 그 둘을 34.8px 띄운다(줄 765.9px · 표 800.7px).
//! 저장 델타(33.3px)를 더해 앉히면 겹침이 사라진다.
//!
//! ## 이 시험이 잠그지 않는 것
//!
//! 같은 4쪽에서 제목 상자(선언 148.0px, 페인트 32.2px) 뒤로 **116px 빈 띠**가 남는다.
//! 흐름 누적이 저장 사다리보다 짧게 쌓이다가(p49~p61 구간 72.8px 압축) p64 가 저장
//! vpos 로 스냅하면서 생기는 두 높이 모델의 어긋남이며, 이 이슈의 남은 범위다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_document, scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("픽스처")).expect("문서 로드")
}

/// 본문(Body) 상자.
fn body_box(root: &RenderNode) -> (f64, f64) {
    fn find<'a>(node: &'a RenderNode, out: &mut Option<&'a RenderNode>) {
        if out.is_some() {
            return;
        }
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            *out = Some(node);
            return;
        }
        for child in &node.children {
            find(child, out);
        }
    }
    let mut body = None;
    find(root, &mut body);
    let body = body.expect("Body");
    (body.bbox.y, body.bbox.y + body.bbox.height)
}

/// 감싼 칸 안의 직계 내용(줄·중첩 표)을 위에서 아래로.
fn wrapper_cell_items(root: &RenderNode) -> Vec<(&'static str, f64, f64)> {
    fn find_cell<'a>(node: &'a RenderNode, out: &mut Option<&'a RenderNode>) {
        if out.is_some() {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            *out = Some(node);
            return;
        }
        for child in &node.children {
            find_cell(child, out);
        }
    }
    let mut cell = None;
    find_cell(root, &mut cell);
    let Some(cell) = cell else {
        return Vec::new();
    };
    cell.children
        .iter()
        .filter_map(|child| match child.node_type {
            RenderNodeType::TextLine(_) => {
                Some(("TextLine", child.bbox.y, child.bbox.y + child.bbox.height))
            }
            RenderNodeType::Table(_) => {
                Some(("Table", child.bbox.y, child.bbox.y + child.bbox.height))
            }
            _ => None,
        })
        .collect()
}

/// 쪽수가 정본과 같다.
#[test]
fn page_count_matches_the_hancom_oracle() {
    assert_eq!(core().page_count(), 7, "한/글 2020 정본은 7쪽이다");
}

/// 4쪽 내용이 본문 상자 안에서 끝난다 — 용지 밖으로 나가지 않는다.
#[test]
fn page4_content_stays_inside_the_body() {
    let core = core();
    let tree = core.build_page_render_tree(3).expect("4쪽 render tree");
    let (_body_top, body_bottom) = body_box(&tree.root);
    let items = wrapper_cell_items(&tree.root);
    assert!(!items.is_empty(), "감싼 칸의 내용을 찾지 못했다");

    let lowest = items
        .iter()
        .map(|(_, _, bottom)| *bottom)
        .fold(f64::MIN, f64::max);
    assert!(
        lowest <= body_bottom + 0.5,
        "4쪽 내용이 본문 바닥을 넘는다: 최하단 {lowest:.1}px, 본문 바닥 {body_bottom:.1}px \
         (수정 전 1134.9px — 용지 1122.5px 밖)"
    );
}

/// 5쪽은 저장 쪽 프레임이 가리키는 `4 기대효과 및 향후계획` 제목 상자로 시작한다.
#[test]
fn page5_starts_at_the_stored_page_frame() {
    let core = core();
    let tree = core.build_page_render_tree(4).expect("5쪽 render tree");
    let items = wrapper_cell_items(&tree.root);
    let first = items.first().expect("5쪽 첫 내용");
    assert_eq!(
        first.0, "Table",
        "정본 5쪽은 제목 상자(1×3 표)로 시작한다 — 수정 전에는 그 상자가 4쪽에 있었다. got {items:?}"
    );

    fn first_text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(run.display_or_text());
        }
        for child in &node.children {
            first_text(child, out);
        }
    }
    let mut text = String::new();
    first_text(&tree.root, &mut text);
    assert!(
        text.contains("기대효과"),
        "5쪽 첫머리가 '4 기대효과 및 향후계획' 이어야 한다: {:?}",
        text.chars().take(40).collect::<String>()
    );
}

/// 4쪽에서 글자 겹침이 없다 — 중첩 표가 자기 저장 줄에 앉는다.
#[test]
fn page4_has_no_text_overlap() {
    let core = core();
    let tree = core.build_page_render_tree(3).expect("4쪽 render tree");
    let anomalies = scan_page(3, &tree.root, core.page_count(), &AnomalyOptions::default());
    assert!(
        anomalies.text_overlap.is_empty(),
        "글자 겹침 {}건 (수정 전 13건: `□ 적용법조` 줄 위로 법조문 표가 올라왔다)",
        anomalies.text_overlap.len()
    );
}

/// 문서 전체에 쪽 밖 요소가 없다.
#[test]
fn no_off_canvas_in_the_document() {
    let core = core();
    let anomalies = scan_document(&core, &AnomalyOptions::default()).expect("layout-anomaly");
    let off: Vec<String> = anomalies
        .pages
        .iter()
        .flat_map(|page| {
            page.off_canvas
                .iter()
                .map(move |a| format!("{}쪽 {} {:.1}px", page.page + 1, a.path, a.max_over()))
        })
        .collect();
    assert!(
        off.is_empty(),
        "쪽 밖 요소가 남아 있다 (수정 전 4쪽 표 12.4px): {off:?}"
    );
}

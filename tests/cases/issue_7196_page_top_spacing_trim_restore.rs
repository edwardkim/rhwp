//! [#7196] 복원되지 않을 문단 간격 트림을 조판이 하지 않는다 — 쪽 말미 표가 본문 바닥을 넘지 않는다.
//!
//! `#2279 ①` 은 문단 전진을 `height_for_fit`(문단 위 간격·끝 줄 간격 제외)으로 깎고, 다음 저장
//! anchor 의 vpos-snap 이 좌표를 되돌린다고 전제한다. 그런데 HWPX 저장 레이아웃에서 다음 경계의
//! 저장 사다리가 굵은 문단 위 간격을 담지 않으면 `#6031` 이 그 스냅을 철회(dirty)해 트림이
//! 영영 복원되지 않는다. 렌더는 순차 흐름을 쓰므로 조판(쪽 끊기)과 렌더(그리기)가 갈라진다.
//!
//! 재현 문서 `samples/issue7196/156760012_page_top_spacing_trim.hwpx` (한/글 2022 저장본):
//! - 10쪽 첫 문단 pi=66(쪽나누기, sb 26.7px) 이 조판에서 52.3px(sb 26.7 + 끝 줄 ls 25.6) 짧게
//!   계상되고, pi=67 경계(`sb 33.3px`, 저장 gap 1920HU = 줄 간격뿐)에서 `#6031` 이 철회한다.
//! - 결과: 조판은 pi=72 `붙임 3` 표(40.2px)가 10쪽에 들어간다고 보지만 렌더는 표를
//!   본문 바닥 +18.3px, 뒤 제목 줄을 +55.2px 넘겨 그렸다.
//!
//! 정답지 — 한/글 2024 MCP PDF(`…-2024.pdf`; 저장 제품 2022 는 규약상 `engine 2020` 이지만
//! 그 profile 이 이 문서에서 EOF 없는 불완전 PDF 로 2회 실패해 2024 로 받았다):
//! - `□ 아울러 …` 로 시작하는 쪽의 본문 줄 기준선이 rhwp 렌더와 줄마다 +18.0px 로 일치
//!   (렌더 흐름이 옳다), 그 쪽 마지막 줄은 `감사합니다.`
//! - `붙임 3` 표는 **다음 쪽 맨 위**에서 시작한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7196/156760012_page_top_spacing_trim.hwpx";

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

fn subtree_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        subtree_text(child, out);
    }
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 본문 흐름에 직접 놓인 표·글줄의 (종류, bbox, 글자).
fn flow_items(node: &RenderNode, out: &mut Vec<(&'static str, BoundingBox, String)>) {
    let kind = match node.node_type {
        RenderNodeType::Table(_) => Some("Table"),
        RenderNodeType::TextLine(_) => Some("TextLine"),
        _ => None,
    };
    if let Some(kind) = kind {
        let mut text = String::new();
        subtree_text(node, &mut text);
        out.push((kind, node.bbox, text));
        return;
    }
    for child in &node.children {
        flow_items(child, out);
    }
}

#[test]
fn issue_7196_attachment_table_starts_next_page_without_body_overflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let mut attachment_page = None;
    let mut pages = Vec::new();
    for page_num in 0..core.page_count() {
        let tree = core.build_page_render_tree(page_num).expect("page");
        let body = body(&tree.root).expect("Body").clone();
        let mut items = Vec::new();
        flow_items(&body, &mut items);
        if attachment_page.is_none()
            && items
                .iter()
                .any(|(kind, _, text)| *kind == "Table" && squash(text).contains("붙임3"))
        {
            attachment_page = Some(pages.len());
        }
        pages.push((body.bbox, items));
    }
    let at = attachment_page.expect("`붙임 3` 표가 있는 쪽");
    assert!(at > 0, "`붙임 3` 표가 첫 쪽에 있을 수 없다");

    // 한/글: `붙임 3` 표는 쪽 맨 위에서 시작한다.
    let (body_box, items) = &pages[at];
    let (kind, first_box, first_text) = &items[0];
    assert!(
        *kind == "Table" && squash(first_text).contains("붙임3"),
        "`붙임 3` 표가 {}쪽의 첫 흐름 항목이어야 한다(한/글 PDF), got {kind} {first_text:?}",
        at + 1
    );
    assert!(
        first_box.y - body_box.y < 5.0,
        "`붙임 3` 표가 쪽 맨 위에 있어야 한다: body_top={:.1} table_y={:.1}",
        body_box.y,
        first_box.y
    );

    // 한/글: 앞 쪽은 `감사합니다.` 로 끝나고, 어떤 흐름 항목도 본문 바닥을 넘지 않는다.
    let (prev_body, prev_items) = &pages[at - 1];
    let body_bottom = prev_body.y + prev_body.height;
    let (_, _, last_text) = prev_items.last().expect("앞 쪽 흐름 항목");
    assert_eq!(
        squash(last_text),
        "감사합니다.",
        "앞 쪽은 `감사합니다.` 로 끝나야 한다(한/글 PDF)"
    );
    for (kind, bbox, text) in prev_items {
        assert!(
            bbox.y + bbox.height <= body_bottom + 0.5,
            "앞 쪽 {kind} {text:?} 바닥 {:.1} 이 본문 바닥 {body_bottom:.1} 을 넘었다",
            bbox.y + bbox.height
        );
    }
}

/// 독립 한컴 2020 PDF 3쪽. 음수 Percent TAC host와 양수 줄간격 TAC 표가
/// 한 문서에 섞여 있어, 트림 복원을 고치며 앞쪽 표를 다음 쪽으로 밀면 안 된다.
#[test]
fn negative_percent_tac_hosts_preserve_the_three_page_physical_layout() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "tests/fixtures/planet_review_20260917/156676190_[교육부 02-27(목) 조간보도자료] 2026년 국립대 임대형 민자사업(BTL) 기숙사 추진.hwpx"
    );
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("read counterexample")).expect("open");
    assert_eq!(core.page_count(), 3, "동일 원문의 한컴 2020 기준 PDF는 3쪽");
    let tree = core.build_page_render_tree(0).expect("page 1");
    let page_body = body(&tree.root).expect("body");
    let mut items = Vec::new();
    flow_items(page_body, &mut items);
    let first_body = items
        .iter()
        .find(|(kind, _, text)| *kind == "TextLine" && squash(text).starts_with("교육부("))
        .expect("first body line");
    assert!(
        (first_body.1.y - 371.8).abs() <= 1.0,
        "원문 vertpos 20799 HU / PDF 첫 본문 줄: {:?}",
        first_body.1
    );
    let last_table = items
        .iter()
        .filter(|(kind, _, _)| *kind == "Table")
        .last()
        .expect("photo table");
    assert!(
        (last_table.1.y - 811.0).abs() <= 1.5,
        "PDF 사진 표 윗변 811px: {:?}",
        last_table.1
    );
    for (kind, bbox, text) in &items {
        assert!(
            bbox.y + bbox.height <= page_body.bbox.y + page_body.bbox.height + 0.5,
            "{kind} {text:?}: 본문 바닥을 넘음 {bbox:?}"
        );
    }
    let second = core.build_page_render_tree(1).expect("page 2");
    let mut second_items = Vec::new();
    flow_items(body(&second.root).expect("second body"), &mut second_items);
    let (_, first_box, first_text) = &second_items[0];
    assert!(squash(first_text).starts_with("박성민"));
    // 원본 pi8 vertpos=1500 HU, PDF 2쪽 첫 줄 top=114.47px.
    assert!(
        (first_box.y - 114.47).abs() <= 1.0,
        "쪽 위 문단 간격 보존: {first_box:?}"
    );
    let first_seg = &core.document().sections[0].paragraphs[0].line_segs[0];
    assert_eq!(
        first_seg.line_height,
        2994 + 283 + 283,
        "개체 줄 상자는 상하 바깥여백을 포함"
    );
    assert_eq!(first_seg.line_spacing, -600, "1500 HU 글자모양에 60% 간격");
}

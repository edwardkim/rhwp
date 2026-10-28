//! Issue #1755: 지연 이월 표의 host 텍스트 줄은 이월 전 쪽에 pre-emit 되어야 한다.
//!
//! Regression shape (samples/task1753/deferred_takeplace_fill_ahead.hwpx):
//! - pi=51 host 제목("1) 투입인원수 산정기준")을 한글은 9쪽 하단(anchor 흐름 위치)에
//!   렌더하는데, 수정 전 rhwp 는 layout 의 defer_visible_rowbreak_host_text 경로로
//!   마지막 fragment 뒤(11쪽)에 렌더 — 제목이 자기 표/후속 문단 뒤에 나타나는 순서 결함.
//! - 수정 후: typeset 이 이월 직전 PartialParagraph{51, 0..1} 로 9쪽에 pre-emit 하고
//!   layout 은 pre_emitted_host_paras 신호로 fragment 쪽 host 렌더를 억제한다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1753/deferred_takeplace_fill_ahead.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1755_host_heading_pre_emitted_on_page_9() {
    let doc = load_doc();
    let page9 = doc.dump_page_items(Some(8));

    assert!(
        page9.contains("PartialParagraph  pi=51"),
        "host 제목 줄은 이월 전 쪽(9쪽)에 PartialParagraph 로 pre-emit 되어야 한다\n--- page 9 ---\n{}",
        page9
    );
    // 한글 순서: pi50 → pi51(제목) → pi52 → pi53
    let i51 = page9.find("PartialParagraph  pi=51").unwrap();
    let i52 = page9.find("FullParagraph  pi=52").unwrap_or(usize::MAX);
    assert!(
        i51 < i52,
        "제목 줄(pi=51)은 후속 문단(pi=52)보다 앞에 배치되어야 한다\n--- page 9 ---\n{}",
        page9
    );
}

// ── Issue #6969: 데코레이션(글앞/글뒤) 표 host 문단의 텍스트 방출 ──────────
//
// #703 단축은 글앞으로/글뒤로 표를 Shape 로만 방출하고 `continue` 한다. 그
// 문단에 가시 텍스트(제목 등)가 있으면 그 텍스트를 위한 PageItem 이 typeset
// 어디에서도 발행되지 않아 렌더 트리에 TextRun 이 생기지 않았다 — 제목이
// 통째로 사라지고, 텍스트 높이가 흐름에 실리지 않아 뒤 내용이 위로 붙었다.

const DECORATION_HOST_SAMPLE: &str = "samples/issue6969/synth_decoration_host_title.hwp";

fn load_decoration_host_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DECORATION_HOST_SAMPLE);
    let bytes =
        fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", DECORATION_HOST_SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", DECORATION_HOST_SAMPLE, e))
}

#[test]
fn issue_6969_decoration_table_host_paragraph_text_is_emitted() {
    let doc = load_decoration_host_doc();

    // 전제: pi=0 은 가시 텍스트와 비인라인 wrap 표를 함께 가진 host 문단이다.
    let title: String = doc.document().sections[0].paragraphs[0]
        .text
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert!(
        !title.is_empty(),
        "전제: 픽스처 pi=0 은 가시 텍스트를 가져야 한다(픽스처 드리프트 감지)"
    );

    let items = doc.dump_page_items(Some(0));
    assert!(
        items.contains("pi=0"),
        "데코레이션 표 host 문단의 텍스트가 1쪽 항목으로 방출되어야 한다\n--- page 1 ---\n{items}"
    );

    // 방출된 텍스트가 실제로 렌더 트리의 TextRun 까지 도달한다.
    let core = rhwp::document_core::DocumentCore::from_bytes(
        &fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(DECORATION_HOST_SAMPLE))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    let tree = core.build_page_render_tree(0).expect("render tree");

    fn collect_text(node: &rhwp::renderer::render_tree::RenderNode, out: &mut String) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            collect_text(child, out);
        }
    }
    let mut rendered = String::new();
    collect_text(&tree.root, &mut rendered);
    let rendered: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();

    assert!(
        rendered.contains(&title),
        "host 문단 제목이 렌더 트리의 TextRun 으로 나와야 한다: {title:?}"
    );
}

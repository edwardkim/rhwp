//! [#8105] 쪽 나눔 표 조각의 바깥 위 여백과 저장 되감김 경계의 끝 줄간격.
//!
//! - A. 이어지는 조각은 문단 기준 세로 오프셋이 있는 표도 바깥 위 여백을 다시 연다. 오프셋은
//!   첫 조각에서 소진된다. 1342000 구역 1 표(`pi=0`, 오프셋 5280HU, 바깥 위 141HU)의 한/글 2020
//!   이어지는 조각 윗변은 26–29쪽 모두 본문 위(위 여백 5669 + 머리말 4252 HU) + 141HU 다.
//! - B. 다음 문단이 자기 앞 간격 원점으로 되감기는 저장 경계(55쪽 행 21: p1 vpos 1800,
//!   p2 vpos 500 = 앞 간격)에서 앞 줄의 뒤 줄간격은 다음 프레임 몫이다. 한/글 2020 은 p0·p1 을
//!   55쪽에, p2 «제10조(학생예방교육)» 를 56쪽에 둔다. 수정 전 rhwp 는 p1 까지 담지 못해 행
//!   21 을 통째로 넘겼다(A 와 함께일 때).
//! - C. 앵커 뒤에 나눌 수 없는 0행이 들어가지 않아 다음 쪽으로 미뤄진 첫 조각은 문단 오프셋을
//!   소진하고, 그 쪽에서 바깥 위 여백을 연다. 한/글 2024 `hwpx_sample2.hwp` 표 `pi=69`(앵커
//!   1016.48 + 오프셋 300HU, 0행 한 줄 28.41px, 바깥 위 283HU) 6쪽 윗변은 본문 위 2835 + 283HU,
//!   아래는 본문 아래 81353 − 283 − 100HU 이다(저장 첫 프레임 1038.01px 와 같다). 수정 전
//!   rhwp 는 예산이 오프셋과 여백을 둘 다 빼 4px 일찍 닫았다.
//!   반례: 156555538 9쪽(#6143, 바깥 위 140HU)은 종전부터 본문 위 + 140HU — 여백을 두 번 열지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";
const SAMPLE_HWPX_SAMPLE2: &str = "samples/hwpx_sample2.hwp";
const SAMPLE_156555538: &str = "samples/issue6143/156555538_securities_settlement_review.hwpx";

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn px(hu: f64) -> f64 {
    hu / 75.0
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽 `page`(0-기준)에서 가장 위·아래 표 칸 상자 경계.
fn table_extent(core: &DocumentCore, page: u32) -> (f64, f64) {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .fold((f64::MAX, f64::MIN), |(top, bottom), node| {
            (
                top.min(node.bbox.y),
                bottom.max(node.bbox.y + node.bbox.height),
            )
        })
}

fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>()
        .replace(' ', "")
}

fn assert_near(got: f64, expected: f64, what: &str) {
    assert!(
        (got - expected).abs() <= 0.5,
        "{what}: 기대 {expected:.2}px, 실제 {got:.2}px"
    );
}

#[test]
fn continued_offset_table_reopens_its_outer_top_margin() {
    let core = core(SAMPLE_1342000);
    let top = px(5669.0 + 4252.0 + 141.0);
    for page in 25..=28 {
        let (got, _) = table_extent(&core, page);
        assert_near(got, top, &format!("{}쪽 이어지는 조각 윗변", page + 1));
    }
}

#[test]
fn own_before_restart_keeps_the_closing_line_on_the_page() {
    let core = core(SAMPLE_1342000);
    let p55 = page_text(&core, 54);
    let p56 = page_text(&core, 55);
    let closing = "[시행2017.7.31.][충청남도조례제4271호";
    assert!(p55.contains(closing), "55쪽이 행 21 의 p1 줄을 담아야 한다");
    assert!(!p56.contains(closing), "p1 줄이 56쪽에 다시 나오면 안 된다");
    assert!(
        p56.contains("제10조(학생예방교육)"),
        "56쪽은 되감긴 p2 «제10조(학생예방교육)» 를 담는다"
    );
    assert!(
        !p55.contains("제10조(학생예방교육)"),
        "p2 는 55쪽에 들지 않는다"
    );
}

#[test]
fn deferred_first_fragment_spends_its_offset_and_opens_the_outer_top() {
    let core = core(SAMPLE_HWPX_SAMPLE2);
    let (top, bottom) = table_extent(&core, 5);
    assert_near(top, px(2835.0 + 283.0), "6쪽 첫 조각 윗변");
    assert_near(
        bottom,
        px(84188.0 - 2835.0 - 283.0 - 100.0),
        "6쪽 첫 조각 아래 닫는 선",
    );
}

#[test]
fn consumed_offset_does_not_reopen_an_already_opened_margin() {
    let core = core(SAMPLE_156555538);
    let (top, _) = table_extent(&core, 8);
    assert_near(
        top,
        px(2835.0 + 2835.0 + 140.0),
        "156555538 9쪽 첫 조각 윗변",
    );
}

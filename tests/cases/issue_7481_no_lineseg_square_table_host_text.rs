//! [#7481] 저장 줄이 없는 문단의 어울림(Square) 표 host 글자 — 띠 도출과 흐름 전진 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! 어울림 표를 앵커한 host 문단의 글자는 표 옆 띠로 흐르고, 흐름은 max(글자, 표) 만큼만
//! 전진한다(#439). 두 장부가 모두 그 띠를 **첫 저장 줄**(cs/sw)과 저장 줄 높이에서
//! 읽었기 때문에, 저장 LINE_SEG 가 없는 문서(기계 생성 서식)에서는
//!
//! - typeset 이 글자를 표 **아래** post-text 로 한 번 더 쌓아 흐름이 표+글자만큼 전진했고
//!   (뒤따르는 쪽 높이급 표가 다음 쪽으로 통째 이월 — 1쪽이 거의 백지),
//! - layout 은 띠 폭 0(또는 저장 줄 부재로 조기 반환)이라 글자를 **그리지 않았다**
//!   (제목 소실).
//!
//! # 기대값의 출처
//!
//! 두 재현물은 실문서(저장 LINE_SEG 없음)를 글자 치환·그림 교체로 익명화한 것이다.
//!
//! - `synth_square_host_title_no_ls.hwp`: 원본을 웹한글기안기로 연 화면은 1쪽이다.
//!   제목은 오른쪽 위 작은 어울림 표(617..731px) 왼쪽에 놓이고(212.8..549.7px, 59.6..83.1px),
//!   본문 표는 그 표 바로 아래(127.7px)에서 같은 쪽에 시작한다.
//! - `synth_square_host_full_width_table_no_ls.hwp`(반례): 어울림 표가 단 폭을 거의 채워
//!   옆에 글자 띠가 없다. 표의 세로 오프셋이 양수여도 웹한글기안기 화면에서 host 글자
//!   (199.3..214.8px)는 표(123.9..194.7px) **아래**에 놓인다.
//!
//! # 검사하는 것
//!
//! 제목 글줄이 존재하는지, 띠가 있으면 표 옆에·없으면 표 아래에 놓이는지, 뒤 표가 같은
//! 쪽에서 앞 표 아래에 시작하는지(최종 좌표)와 쪽 수.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const BESIDE: &str = "samples/issue7481/synth_square_host_title_no_ls.hwp";
const FULL_WIDTH: &str = "samples/issue7481/synth_square_host_full_width_table_no_ls.hwp";

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

#[derive(Default)]
struct Found {
    /// para_index 별 본문(표 바깥) 글줄 상자
    body_lines: Vec<(usize, BoundingBox)>,
    /// para_index 별 본문 표 상자
    tables: Vec<(usize, BoundingBox)>,
}

fn collect(node: &RenderNode, in_table: bool, found: &mut Found) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(t) => {
            if !in_table {
                if let Some(pi) = t.para_index {
                    found.tables.push((pi, node.bbox));
                }
            }
            inside = true;
        }
        RenderNodeType::TextLine(line) if !in_table => {
            if let Some(pi) = line.para_index {
                found.body_lines.push((pi, node.bbox));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, inside, found);
    }
}

fn page0(sample: &str) -> Found {
    let tree = load(sample)
        .build_page_render_tree(0)
        .expect("1쪽 렌더 트리");
    let mut found = Found::default();
    collect(&tree.root, false, &mut found);
    found
}

fn table_of(found: &Found, pi: usize) -> BoundingBox {
    found
        .tables
        .iter()
        .find(|(p, _)| *p == pi)
        .map(|(_, b)| *b)
        .unwrap_or_else(|| panic!("1쪽에서 문단 {pi} 의 표를 찾지 못했다"))
}

fn host_lines(found: &Found, pi: usize) -> Vec<BoundingBox> {
    found
        .body_lines
        .iter()
        .filter(|(p, b)| *p == pi && b.width > 1.0)
        .map(|(_, b)| *b)
        .collect()
}

#[test]
fn title_beside_square_table_is_painted_in_the_band() {
    let found = page0(BESIDE);
    let square = table_of(&found, 0);
    let lines = host_lines(&found, 0);
    assert!(
        !lines.is_empty(),
        "어울림 표 host 제목 글줄이 없다 — 저장 줄이 없으면 layout 이 띠 폭 0 으로 제목을 버린다"
    );
    for line in &lines {
        assert!(
            line.x + line.width <= square.x + 0.5 && line.y < square.y + square.height,
            "제목 글줄이 표 옆 띠 밖이다 — 글줄 x {:.1}..{:.1} y {:.1}, 표 x {:.1}.. y ..{:.1}",
            line.x,
            line.x + line.width,
            line.y,
            square.x,
            square.y + square.height
        );
    }
}

#[test]
fn following_table_starts_below_the_square_table_on_the_same_page() {
    let doc = load(BESIDE);
    assert_eq!(
        doc.page_count(),
        1,
        "웹한글기안기 기준 1쪽 — host 글자를 표 아래에 다시 쌓으면 뒤 표가 2쪽으로 이월한다"
    );
    let found = page0(BESIDE);
    let square = table_of(&found, 0);
    let body = table_of(&found, 1);
    assert!(
        body.y + 0.5 >= square.y + square.height,
        "뒤 표가 어울림 표와 겹친다 — 어울림 표 ..{:.1}, 뒤 표 {:.1}..",
        square.y + square.height,
        body.y
    );
}

#[test]
fn full_width_square_table_sends_host_text_below_the_table() {
    // 반례: 옆 띠가 없으면 띠를 만들지 않고, 글자는 표 아래로 흐른다(양수 세로 오프셋이어도).
    const HOST: usize = 2;
    let found = page0(FULL_WIDTH);
    let square = table_of(&found, HOST);
    let lines = host_lines(&found, HOST);
    assert!(!lines.is_empty(), "단 폭 어울림 표 host 글줄이 없다");
    for line in &lines {
        assert!(
            line.y + 0.5 >= square.y + square.height,
            "띠가 없는 host 글자가 표 아래로 가지 않았다 — 글줄 y {:.1}..{:.1}, 표 ..{:.1}",
            line.y,
            line.y + line.height,
            square.y + square.height
        );
    }
}

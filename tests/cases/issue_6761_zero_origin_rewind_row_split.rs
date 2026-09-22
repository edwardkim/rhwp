//! [#6761] 저장 사다리가 표의 한 행을 **첫 줄 뒤에서** 나눈 자리를 25px 고아 기준이 버린다.
//!
//! ## 무엇이 문제였나
//!
//! `1480000-201900042` 의 `<표 2-5> 어린이보호에 대한 경고문구 표시 사례`(RowBreak, 15행)는
//! 한 쪽 끝에서 r=3 `시아노아크릴레이트 함유 혼합물` 행을 만난다. 한컴 2010 저장본은 그 행의
//! 두 칸 모두 **첫 줄만 이 쪽에 두고 둘째 줄을 새 쪽 원점에서** 재개했다고 적는다.
//!
//! ```text
//!   셀[8] r=3,c=1  p[0] ls[0] vpos=0  ls[1] vpos=0
//!   셀[9] r=3,c=2  p[0] ls[0] vpos=0  ls[1] vpos=0
//!                  p[1] ls[0] vpos=1320 = 1100 + 220   ← 되감긴 줄에서 한 줄 전진
//! ```
//!
//! rhwp 의 컷도 정확히 그 자리(`end_cut=[1,1]`, 17.6px)에 떨어지지만 25px 고아 기준이 이를
//! 기각해 행 전체를 다음 쪽으로 넘겼다. 그 쪽의 조각이 한 줄만큼 길어져 마지막 문단
//! `하나 이상의 피부과민성물질…` 이 본문 바닥(1028.0px)을 13px 넘었다.
//!
//! ## 기대값의 독립 근거
//!
//! - 저장 사다리: 위의 셀 되감김(`rhwp dump -s 1 -p 199`)
//! - 한컴 정본 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` 39쪽 마지막
//!   행이 `시아노아크릴레이트 | – 순식간에 피부 및 안구가 접착됨. 어` 이고 40쪽이
//!   `함유 혼합물 | 린이 손에 닿게 하지 마시오.` 로 시작한다
//!
//! ## 반례 — 저장값이 모두 0 인 입력은 증거가 아니다
//!
//! 같은 행의 모든 줄을 0 으로 바꾸면 "되감김"은 남아도 사다리가 이어진다는 확인(다음 seg 의
//! `lh + ls` 전진)이 사라진다. 그때는 종전 고아 기준대로 행을 통째로 넘겨야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
/// r=3 첫 칸의 글자. 행을 나누면 앞 조각에 `시아노아크릴레이트`, 뒤 조각에 `함유 혼합물` 이 간다.
const ROW_HEAD: &str = "시아노아크릴레이트";
/// 표가 시작하는 쪽에만 있는 캡션.
const CAPTION: &str = "어린이보호에대한경고문구";
/// 본문 바닥: 본문 상자 y 132.28 + h 895.73.
const BODY_BOTTOM: f64 = 1028.0;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn first_page_with_all(core: &DocumentCore, needles: &[&str]) -> u32 {
    (0..core.page_count() as u32)
        .find(|page| {
            let text = flat(&core.extract_page_text_native(*page).unwrap_or_default());
            needles.iter().all(|needle| text.contains(needle))
        })
        .unwrap_or_else(|| panic!("{needles:?} 를 모두 담은 쪽이 없다"))
}

fn first_page_with(core: &DocumentCore, needle: &str) -> u32 {
    first_page_with_all(core, &[needle])
}

/// 표가 시작하는 쪽. 캡션은 앞쪽 표 차례에도 있으므로 첫 행 `EU CLP` 와 함께 찾는다.
fn table_start_page(core: &DocumentCore) -> u32 {
    first_page_with_all(core, &[CAPTION, "EUCLP"])
}

/// 본문(`Body`) 아래 노드의 최대 하단. 꼬리말 줄은 본문 밖이라 세지 않는다.
fn body_bottom(node: &RenderNode, in_body: bool) -> f64 {
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    let own = if in_body && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        node.bbox.y + node.bbox.height
    } else {
        f64::MIN
    };
    node.children
        .iter()
        .map(|child| body_bottom(child, in_body))
        .fold(own, f64::max)
}

/// 저장 사다리가 적은 자리에서 행을 나눈다 — 첫 줄이 표의 첫 쪽에 남는다.
#[test]
fn stored_first_line_split_keeps_row_head_on_table_page() {
    let core = open();
    let table_page = table_start_page(&core);
    let head_page = first_page_with(&core, ROW_HEAD);
    assert_eq!(
        head_page,
        table_page,
        "`{ROW_HEAD}` 행 첫 줄이 {}쪽에 있다 — 저장 되감김(ls[1] vpos 0)과 정본 39쪽대로 표의 \
         첫 쪽({}쪽)에 남아야 한다. 25px 고아 기준이 17.6px 한 줄을 기각하면 행 전체가 넘어간다",
        head_page + 1,
        table_page + 1
    );
    let next = core
        .extract_page_text_native(table_page + 1)
        .unwrap_or_default();
    assert!(
        flat(&next).starts_with(&flat("국외 관련 제도"))
            && flat(&next).contains(&flat("함유 혼합물")),
        "다음 쪽이 반복 제목행 뒤 `함유 혼합물` 조각으로 이어져야 한다: {:?}",
        next.chars().take(80).collect::<String>()
    );
}

/// 이어받은 쪽의 마지막 문단이 본문 바닥 안에 든다.
#[test]
fn continuation_page_stays_inside_body() {
    let core = open();
    let page = table_start_page(&core) + 1;
    let tree = core.build_page_render_tree(page).expect("render tree");
    let bottom = body_bottom(&tree.root, false);
    assert!(
        bottom <= BODY_BOTTOM + 0.5,
        "{}쪽 본문 마지막 줄 하단이 {bottom:.1}px 로 본문 바닥 {BODY_BOTTOM}px 를 넘는다 (#6761)",
        page + 1
    );
}

/// 반례 — 되감김 뒤 사다리가 이어지지 않으면(다음 seg 가 되감긴 줄의 `lh + ls` 만큼 전진하지
/// 않으면) 저장 분할의 증거가 아니다. 행 형상과 컷은 그대로 두고 `셀[9] p[1]` 의 저장 위치만
/// 80HU(약 1px) 밀어 그 확인 하나만 깨뜨린다 — 종전 고아 기준대로 행을 통째 넘겨야 한다.
#[test]
fn rewind_without_ladder_continuation_is_not_split_evidence() {
    let mut core = open();
    let table_page = table_start_page(&core);
    let mut doc = core.document().clone();
    let mut touched = 0;
    for section in &mut doc.sections {
        for paragraph in &mut section.paragraphs {
            for control in &mut paragraph.controls {
                let Control::Table(table) = control else {
                    continue;
                };
                for cell in table.cells.iter_mut().filter(|cell| cell.row == 3) {
                    let is_response = cell
                        .paragraphs
                        .first()
                        .is_some_and(|p| flat(&p.text).starts_with(&flat("- 순식간에")));
                    if !is_response {
                        continue;
                    }
                    for seg in &mut cell.paragraphs[1].line_segs {
                        seg.vertical_pos += 80;
                        touched += 1;
                    }
                }
            }
        }
    }
    assert_eq!(touched, 2, "대상 셀[9] p[1] 의 저장 줄 2개를 찾지 못했다");
    core.set_document(doc);
    let head_page = first_page_with(&core, ROW_HEAD);
    assert_eq!(
        head_page,
        table_page + 1,
        "사다리 전진 확인이 없으면 `{ROW_HEAD}` 행은 종전 고아 기준대로 다음 쪽({}쪽)으로 가야 \
         한다 — {}쪽에 남았다",
        table_page + 2,
        head_page + 1
    );
}

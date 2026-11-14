//! 관계형: 저장 줄 없는 문단에서 쪽 너비를 채운 글자처럼 표 뒤 꼬리 공백이 넘치면,
//! 뒤에 부동 앵커(글앞으로 표)만 남았을 때도 일반 글줄의 공백 걸기(#7168·#8639)를 따른다.
//!
//! # 기대값의 출처
//!
//! 웹한글기안기 인쇄 PDF `pdf/tac_space_tail/<이름>-webhwp-print.pdf`. 합성본
//! `samples/tac_space_tail/*.hwp`(생성기 `gen_tac_space_tail.py`, 공개 샘플만 조합):
//! 문단 0 "Q" / 문단 1 = 16pt 런의 글자처럼 표 + 20pt 굵은 공백 N개 + 13pt 공백 40개 +
//! 문단 끝 작은 글앞으로 표 / 문단 2 빈 문단 / 문단 3 "TEXT".
//! 표 폭은 단 폭에서 `rest` 만 남긴다(at_table: 20pt 공백 하나도 들지 않음, after_3fit: 셋이
//! 들어감, after_1fit: 하나가 들어감).
//!
//! PDF 의 "TEXT" 위 끝(96dpi px)과 같은 세트 첫 문서 대비 변화:
//!
//! | 문서 | TEXT y | 변화 | 읽기 |
//! |---|---|---|---|
//! | at_table_n0 | 265.7 | 0 | 13pt 공백만 |
//! | at_table_n2 | 268.9 | +3.2 | 넘친 20pt 공백 둘 다 표 줄 끝에 걸림(표 바로 뒤라 하나 더) |
//! | at_table_n3 | 283.6 | +17.9 | 셋째가 다음 줄로 넘어가 그 줄이 20pt 높이 |
//! | after_3fit_n3 | 268.9 | 0 | 셋 다 들어감 |
//! | after_3fit_n4 | 268.9 | 0 | 넘친 첫 공백 하나는 걸림 |
//! | after_3fit_n5 | 283.6 | +14.7 | 넘친 둘째부터 다음 줄 |
//! | after_1fit_n1 | 268.9 | 0 | 하나 들어감 |
//! | after_1fit_n2 | 268.9 | 0 | 넘친 첫 공백 하나는 걸림 |
//! | after_1fit_n3 | 283.6 | +14.7 | 넘친 둘째부터 다음 줄 |
//!
//! +3.2 는 20pt 공백이 표 줄에 걸려 그 줄 줄간격이 16pt 기준에서 20pt 기준이 된 몫이다
//! (같은 PDF 의 글앞으로 표 위치도 3.2 내려간다). 13pt 공백만 건 줄의 줄간격은 표 런의
//! 16pt 그대로다(at_table_n0).
//!
//! # 무엇이 깨져 있었나
//!
//! 인라인 흐름 계획의 공백 분류는 문단 끝 공백(`trailing`)과 뒤에 흐름 내용이 있는
//! 공백(`wrap`)만 줄 끝에 걸었다. 뒤에 부동 앵커가 있으면 둘 다 아니어서, 넘친 공백을
//! 하나도 걸지 않고 모두 다음 줄로 넘겼다. 그래서 넘친 20pt 공백 하나가 다음 줄을
//! 20pt 높이로 만들어 뒤 문단이 15px 가까이 내려갔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

/// 렌더 줄 위치 비교 여유(px). PDF 좌표는 0.1px 로 반올림했다.
const EPS: f64 = 0.6;

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/tac_space_tail")
        .join(format!("{name}.hwp"));
    let bytes = std::fs::read(&path).expect("tac_space_tail 합성본");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn text_line_top(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextLine(_) = &node.node_type {
        let has = node.children.iter().any(|child| {
            matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        });
        if has {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| text_line_top(child, needle))
}

/// 뒤 문단 "TEXT" 줄 위 끝(px).
fn text_top(name: &str) -> f64 {
    let core = load(name);
    assert_eq!(core.page_count(), 1, "{name}: 웹한글기안기는 1쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    text_line_top(&tree.root, "TEXT").expect("TEXT 줄")
}

fn assert_shift(base: &str, name: &str, hancom_shift: f64) {
    let shift = text_top(name) - text_top(base);
    assert!(
        (shift - hancom_shift).abs() < EPS,
        "{name}: {base} 대비 TEXT 이동 {shift:.2}px, 웹한글기안기 {hancom_shift:.1}px"
    );
}

#[test]
fn overflow_right_after_table_hangs_two_spaces() {
    assert_shift("at_table_n0", "at_table_n2", 3.2);
}

#[test]
fn third_overflowing_space_after_table_starts_next_line() {
    assert_shift("at_table_n0", "at_table_n3", 17.9);
}

#[test]
fn first_overflowing_space_after_fitting_spaces_hangs() {
    assert_shift("after_3fit_n3", "after_3fit_n4", 0.0);
    assert_shift("after_1fit_n1", "after_1fit_n2", 0.0);
}

#[test]
fn second_overflowing_space_after_fitting_spaces_starts_next_line() {
    assert_shift("after_3fit_n3", "after_3fit_n5", 14.7);
    assert_shift("after_1fit_n1", "after_1fit_n3", 14.7);
}

//! 관계형·고정형: 꼬리 공백의 줄 끝 걸기와 TEXT의 상대 이동, 독립 Print의 한 쪽 소유.
//! 주석 갱신: 2026-11-14 12:32 KST. 직전 파일 변경: 2026-11-14 01:59:51 +09:00,
//! commit 350a038c344c0cead98eeb1cdf1d366ff596a3cf(Git 확인; 측정 시각과 구분).
//! 측정: 2026-11-14 KST, source e2d7fb08b5, 96 DPI/compat 2022,
//! `scripts/visual_sweep.py --silhouette-only`, 색상 경계 보정 전 2px raw.
//! 공개 원본 9문서 전 9쪽: Native Skia/fresh 최적화 WASM 각각 전쪽 100.00%.
//! 독립 기준은 `pdf/pr9063/`의 실제 데스크톱 한컴 2020 11.0.0.9136 Print(method 0).
//! pdftotext bbox의 TEXT 잉크 위 끝을 96 DPI로 환산해 상대 이동을 확인했다.
//! render-tree 줄 상자와 PDF 잉크의 서브픽셀 차이는 기존 0.6px 여유 안에서 대조한다.
//! 증거: `mydocs/pr/assets/pr9063/evidence.md`, `native/`, `wasm/`, 최종 `correction/` TSV.
//!
//! 관계형: 저장 줄 없는 문단에서 쪽 너비를 채운 글자처럼 표 뒤 꼬리 공백이 넘치면,
//! 뒤에 부동 앵커(글앞으로 표)만 남았을 때도 일반 글줄의 공백 걸기(#7168·#8639)를 따른다.
//!
//! # 기대값의 출처
//!
//! 데스크톱 한컴 2020 Print PDF `pdf/pr9063/<이름>-2020.pdf`. 합성본
//! `samples/tac_space_tail/*.hwp`(생성기 `gen_tac_space_tail.py`, 공개 샘플만 조합):
//! 문단 0 "Q" / 문단 1 = 16pt 런의 글자처럼 표 + 20pt 굵은 공백 N개 + 13pt 공백 40개 +
//! 문단 끝 작은 글앞으로 표 / 문단 2 빈 문단 / 문단 3 "TEXT".
//! 표 폭은 단 폭에서 `rest` 만 남긴다(at_table: 20pt 공백 하나도 들지 않음, after_3fit: 셋이
//! 들어감, after_1fit: 하나가 들어감).
//!
//! PDF 의 "TEXT" 잉크 위 끝(96dpi px)과 같은 세트 첫 문서 대비 변화:
//!
//! | 문서 | TEXT y | 변화 | 읽기 |
//! |---|---|---|---|
//! | at_table_n0 | 269.5230 | 0 | 13pt 공백만 |
//! | at_table_n2 | 272.7196 | +3.2 | 넘친 20pt 공백 둘 다 표 줄 끝에 걸림(표 바로 뒤라 하나 더) |
//! | at_table_n3 | 287.7431 | +18.22 | 셋째가 다음 줄로 넘어가 그 줄이 20pt 높이 |
//! | after_3fit_n3 | 272.7196 | 0 | 셋 다 들어감 |
//! | after_3fit_n4 | 272.7196 | 0 | 넘친 첫 공백 하나는 걸림 |
//! | after_3fit_n5 | 287.7431 | +15.02 | 넘친 둘째부터 다음 줄 |
//! | after_1fit_n1 | 272.7196 | 0 | 하나 들어감 |
//! | after_1fit_n2 | 272.7196 | 0 | 넘친 첫 공백 하나는 걸림 |
//! | after_1fit_n3 | 287.7431 | +15.02 | 넘친 둘째부터 다음 줄 |
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
    assert_eq!(core.page_count(), 1, "{name}: 데스크톱 한컴 Print는 1쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    text_line_top(&tree.root, "TEXT").expect("TEXT 줄")
}

fn assert_shift(base: &str, name: &str, hancom_shift: f64) {
    let shift = text_top(name) - text_top(base);
    assert!(
        (shift - hancom_shift).abs() < EPS,
        "{name}: {base} 대비 TEXT 이동 {shift:.2}px, 데스크톱 한컴 Print {hancom_shift:.1}px"
    );
}

#[test]
fn overflow_right_after_table_hangs_two_spaces() {
    assert_shift("at_table_n0", "at_table_n2", 3.2);
}

#[test]
fn third_overflowing_space_after_table_starts_next_line() {
    assert_shift("at_table_n0", "at_table_n3", 18.22);
}

#[test]
fn first_overflowing_space_after_fitting_spaces_hangs() {
    assert_shift("after_3fit_n3", "after_3fit_n4", 0.0);
    assert_shift("after_1fit_n1", "after_1fit_n2", 0.0);
}

#[test]
fn second_overflowing_space_after_fitting_spaces_starts_next_line() {
    assert_shift("after_3fit_n3", "after_3fit_n5", 15.02);
    assert_shift("after_1fit_n1", "after_1fit_n3", 15.02);
}

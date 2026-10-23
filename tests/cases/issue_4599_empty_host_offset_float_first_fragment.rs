//! Issue #4599: 빈 호스트 문단의 **글자처럼 표 뒤** 양수 오프셋 자리차지 표가 현재 쪽 잔여에
//! 첫 조각을 두지 않고 통째로 다음 쪽으로 이월되던 결함의 가드.
//!
//! 재현 문서 `samples/issue4599/1790387_offset_float_split_min.hwpx` 는 코퍼스
//! `1790387-202300151_D0150014-1-001_정책연구용역 중간진도보고서.hwpx` 의 구역 2 머리
//! 문단 17개(pi0..pi16)만 남긴 것이다(그림 없음, `secCnt` 1).
//!
//! ```text
//! 문단 0.6  빈 호스트, 줄 1개(lh 23412) — controls 2
//!   [0] 표 2x2  treat_as_char=false wrap=자리차지 vert=문단(23973 = 319.6px)
//!   [1] 표 6x4  treat_as_char=true  (호스트 줄 안, 문단 시작부터)
//!   [0] 셀[2] p[0..6] vpos 1000..16320, p[7] vpos=700  ← 한/글이 p[7] 에서 쪽을 나눠 저장
//! ```
//!
//! 한/글(한컴 2020 변환 PDF)은 2x2 표를 `문단 시작 + 오프셋` — 글자처럼 표 바로 아래 — 에
//! 두고 1쪽에서 p[0..6] 까지, 2쪽에서 p[7..] 을 이어 그린다(원본 64쪽). 수정 전 rhwp 는
//! 이월 판정이 오프셋을 문단 시작이 아니라 **글자처럼 표가 내린 현재 흐름 높이**에서 빼
//! 잔여를 0 으로 보고 표 전체를 다음 쪽으로 넘겼다(원본 65쪽, 이 표본 3쪽). 분할 예산은
//! 이미 문단 시작 기준(#1860)이어서 두 판정의 원점이 달랐다.
//!
//! 검사는 관계로 한다: 전체 쪽수, 첫 조각의 쪽·위치(글자처럼 표 아래, 본문 안),
//! 셀 문단의 쪽 소속(저장 분할 지점과 같음, 누락·중복 없음), 뒤 문단이 이어받기 조각 아래.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue4599/1790387_offset_float_split_min.hwpx";
const HOST_PI: u64 = 6;
const FLOAT_CI: u64 = 0;
const TAC_CI: u64 = 1;
const NEXT_PI: u64 = 8;

#[derive(Debug, Clone, Copy)]
struct Bbox {
    y: f64,
    h: f64,
}

impl Bbox {
    fn bottom(self) -> f64 {
        self.y + self.h
    }
}

fn bbox(node: &serde_json::Value) -> Option<Bbox> {
    let b = node.get("bbox")?;
    Some(Bbox {
        y: b.get("y")?.as_f64()?,
        h: b.get("h")?.as_f64()?,
    })
}

fn children(node: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    node.get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
}

fn is(node: &serde_json::Value, kind: &str) -> bool {
    node.get("type").and_then(|t| t.as_str()) == Some(kind)
}

fn find_tables<'a>(node: &'a serde_json::Value, ci: u64, out: &mut Vec<&'a serde_json::Value>) {
    if is(node, "Table")
        && node.get("pi").and_then(|p| p.as_u64()) == Some(HOST_PI)
        && node.get("ci").and_then(|c| c.as_u64()) == Some(ci)
    {
        out.push(node);
        return;
    }
    for child in children(node) {
        find_tables(child, ci, out);
    }
}

fn find_body(node: &serde_json::Value) -> Option<Bbox> {
    if is(node, "Body") {
        return bbox(node);
    }
    children(node).find_map(find_body)
}

/// 표 안 TextLine 의 문단 번호(셀 문단 인덱스).
fn cell_line_pis(node: &serde_json::Value, out: &mut BTreeSet<u64>) {
    if is(node, "TextLine") {
        if let Some(pi) = node.get("pi").and_then(|p| p.as_u64()) {
            out.insert(pi);
        }
    }
    for child in children(node) {
        cell_line_pis(child, out);
    }
}

/// 표 밖 본문 TextLine.
fn body_lines(node: &serde_json::Value, pi: u64, out: &mut Vec<Bbox>) {
    if is(node, "Table") {
        return;
    }
    if is(node, "TextLine") && node.get("pi").and_then(|p| p.as_u64()) == Some(pi) {
        if let Some(b) = bbox(node) {
            out.push(b);
        }
    }
    for child in children(node) {
        body_lines(child, pi, out);
    }
}

fn page_tree(document: &rhwp::wasm_api::HwpDocument, page: u32) -> serde_json::Value {
    let json = document
        .get_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree page {page}: {e:?}"));
    serde_json::from_str(&json).expect("parse render tree json")
}

fn single_table(tree: &serde_json::Value, ci: u64, page: u32) -> &serde_json::Value {
    let mut found = Vec::new();
    find_tables(tree, ci, &mut found);
    assert_eq!(
        found.len(),
        1,
        "{}쪽에 pi={HOST_PI} ci={ci} 표(조각)가 하나 있어야 한다",
        page + 1
    );
    found[0]
}

#[test]
fn empty_host_offset_float_starts_below_tac_sibling_on_current_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    assert_eq!(
        document.page_count(),
        2,
        "한/글은 2쪽 — #4599 회귀: 오프셋 표가 첫 조각 없이 다음 쪽으로 이월되어 쪽이 늘었다"
    );

    // 1쪽: 글자처럼 표 아래에 오프셋 표의 첫 조각, 본문 안.
    let first = page_tree(&document, 0);
    let body = find_body(&first).expect("1쪽 Body");
    let tac = bbox(single_table(&first, TAC_CI, 0)).expect("글자처럼 표 bbox");
    let head_node = single_table(&first, FLOAT_CI, 0);
    let head = bbox(head_node).expect("첫 조각 bbox");
    assert!(
        head.y >= tac.bottom() - 0.5,
        "첫 조각(y={:.1})이 글자처럼 표(바닥 {:.1})와 겹친다",
        head.y,
        tac.bottom()
    );
    assert!(
        head.bottom() <= body.bottom() + 0.5,
        "첫 조각 바닥 {:.1} 이 본문 바닥 {:.1} 을 넘는다",
        head.bottom(),
        body.bottom()
    );

    // 2쪽: 이어받기 조각, 그 아래 뒤 문단.
    let second = page_tree(&document, 1);
    let tail_node = single_table(&second, FLOAT_CI, 1);
    let tail = bbox(tail_node).expect("이어받기 조각 bbox");
    let mut next = Vec::new();
    body_lines(&second, NEXT_PI, &mut next);
    assert_eq!(next.len(), 1, "뒤 문단 pi={NEXT_PI} 은 2쪽에 한 줄");
    assert!(
        next[0].y >= tail.bottom() - 0.5,
        "뒤 문단(y={:.1})이 이어받기 조각(바닥 {:.1})보다 위에 있다",
        next[0].y,
        tail.bottom()
    );

    // 셀 문단의 쪽 소속: 저장본이 p[7] 에서 쪽을 나눴다(vpos 16320 → 700).
    // 머리 행 두 셀(p[0])과 본문 셀 p[0..6] 은 1쪽, p[7..17] 은 2쪽 — 누락·중복 없이.
    let mut head_pis = BTreeSet::new();
    cell_line_pis(head_node, &mut head_pis);
    let mut tail_pis = BTreeSet::new();
    cell_line_pis(tail_node, &mut tail_pis);
    assert_eq!(
        head_pis,
        (0..=6).collect::<BTreeSet<u64>>(),
        "첫 조각의 셀 문단"
    );
    assert_eq!(
        tail_pis,
        (7..=17).collect::<BTreeSet<u64>>(),
        "이어받기 조각의 셀 문단"
    );
}

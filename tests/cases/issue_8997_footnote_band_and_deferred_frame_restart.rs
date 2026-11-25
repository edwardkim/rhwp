//! 관계형 — [#8997] 각주 구분선 띠·이월 표 뒤 빈 재시작 문단으로 90~93쪽 흐름을 한컴과 맞춘다.
//! 주석 갱신: 2026-11-24 06:22 KST (devel 28a6afc1eb 로 rebase 뒤 재측정; 처음 작성 2026-11-23 17:26 KST)
//!
//! 입력: `mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp`
//! (한컴 2020 Print `pdf/issue8997/1543000-202100035-food-security-report-2020.pdf`, 141쪽).
//! - 구역 2 문단 893 은 공백 한 칸 host 와 자리차지·문단 기준 표(그림 2-3, RowBreak, 세로 오프셋
//!   2985HU)를 가진다. 표는 앵커 쪽에 들어가지 않는다. 한컴은 host 줄과 뒤 빈 문단 둘을 89쪽에
//!   두고 표만 90쪽 단 위로 옮긴다. 저장 사다리: pi894·pi895 vpos 55334(비전진 쌍), pi896 vpos
//!   22586 = 위 여백 283 + 표 22020 + 아래 여백 283, pi897 vpos 24616 = pi896 + 줄 하나.
//!   수정 전 rhwp 는 표를 문단째 넘기고 세로 오프셋·빈 줄 둘을 더해 90쪽 흐름이 약 97px 아래였다.
//!   표만 넘긴 뒤에도 「빈 줄 감추기」가 이월 상자 아래 빈 문단 pi896 을 쪽 첫 빈 줄로 감췄다.
//! - 구역 2 문단 925 의 표 2-5(4행×3열, 행 나눔)는 92쪽에서 행 3 첫 줄까지 담고 나머지를 93쪽에
//!   잇는다. 수정 전 rhwp 는 각주 구역(구분선 굵기 0.7mm 를 높이에 더함)이 한컴보다 높아 행 3 첫 줄
//!   조각이 0.4px 모자랐고, 표가 통째로 93쪽에 갔다.
//! - 관계형: 표 그림 2-3 은 90쪽, 그 host 줄은 89쪽에 있다. 이월 상자 아래 빈 문단 pi896 의 줄은
//!   감춰지지 않고 상자 아래에, 다음 문단 pi897 첫 줄은 그 줄 아래에 놓인다. 표 2-5 는 두 쪽으로
//!   나뉘고 앞 조각은 행 3 의 첫 줄(「국민이최저한도로…」)과 각주 21 을 담고, 행 3 의 나머지
//!   (「험성이있는경우」)는 다음 쪽에서 시작한다.
//! - 반례(관계형): 재시작 문단 pi896 에 앞 간격(5000)을 준 사본에서도 빈 재시작 줄의 자리(표 아래끝
//!   → 줄 위끝)와 다음 글 줄까지의 간격은 원본과 같다. 저장 줄 vpos 22586 = 바깥 상자 아래끝이
//!   그 줄의 원점이고 앞 간격은 그 앞에서 소비된다. 10k A/B 에서 찾은 1490000-201400162 46쪽
//!   (pi106 앞 간격 2500HU)이 이 경우다. 빈 재시작 문단을 받아들인 앞 커밋에서 99.17 → 85.85 로
//!   떨어졌고, 재시작 줄의 앞 간격을 상자 아래끝 앞에서 소비하게 고쳐 99.17 로 되돌렸다.
//!
//! 측정: 2026-11-24, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! (`silhouette_raw_match_percent`, 색상 경계 보정 전 2px 내용 실루엣), 글꼴은 맑은 고딕(Regular·Bold)·
//! 바탕 + `ttfs/opensource` 를 Native·WASM 두 경로에 공급. source `36a9dc8ad9`(코드, devel `28a6afc1eb` 위).
//!
//! - 대상 쪽(89~93): Native 95.49·100.00·100.00·99.86·100.00, fresh WASM 95.64·99.96·100.00·99.84·99.99.
//!   수정 전(devel `28a6afc1eb`) Native 95.49·29.74·35.83·29.30·24.29. 89쪽 95.49 는 수정 전과
//!   같은 값이고 이 수정이 바꾸지 않는 쪽 위 글자 차이다.
//! - 전체 141쪽(한컴 Print 와 같음). 전쪽 최저 Native 63.66(85쪽)·WASM 64.78(85쪽). 90 미만 4쪽
//!   (4·6·68·85)은 수정 전과 같은 기존 미달로 #9326 에서 다룬다.
//! - 증거: `mydocs/pr/assets/issue_8997/footnote/{native,wasm,base-native}/food_silhouette.tsv`, 대표 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp";
const SECTION: usize = 2;
/// 그림 2-3 표의 host, 이월 상자 아래 빈 재시작 문단, 그 다음 글 문단.
const DEFERRED_TABLE_PARA: usize = 893;
const BLANK_RESTART_PARA: usize = 896;
const NEXT_TEXT_PARA: usize = 897;
/// 표 2-5(행 나눔) 문단.
const SPLIT_TABLE_PARA: usize = 925;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 열기")
}

#[derive(Default)]
struct PageBoxes {
    tables: Vec<(Option<usize>, Option<usize>, BoundingBox)>,
    lines: Vec<(Option<usize>, Option<usize>, BoundingBox)>,
}

/// 본문 흐름의 표와 줄을 모은다. 표 칸 안과 머리말·꼬리말·각주는 본문 문단이 아니다.
fn collect(node: &RenderNode, out: &mut PageBoxes) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            out.tables
                .push((table.section_index, table.para_index, node.bbox));
            return;
        }
        RenderNodeType::Header
        | RenderNodeType::Footer
        | RenderNodeType::MasterPage
        | RenderNodeType::FootnoteArea => return,
        RenderNodeType::TextLine(line) => {
            out.lines
                .push((line.section_index, line.para_index, node.bbox))
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn page_boxes(core: &DocumentCore, page: u32) -> PageBoxes {
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut out = PageBoxes::default();
    collect(&tree.root, &mut out);
    out
}

/// 쪽 본문(머리말·꼬리말 제외, 각주 포함)의 글자를 표 칸까지 그려진 순서로 이어 붙인다.
fn painted_text(node: &RenderNode, out: &mut String) {
    match &node.node_type {
        RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage => return,
        RenderNodeType::TextRun(run) => out.extend(run.text.chars().filter(|c| !c.is_whitespace())),
        _ => {}
    }
    for child in &node.children {
        painted_text(child, out);
    }
}

fn compact(core: &DocumentCore, page: u32) -> String {
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut out = String::new();
    painted_text(&tree.root, &mut out);
    out
}

fn pages_with_table(core: &DocumentCore, para: usize) -> Vec<u32> {
    (0..core.page_count())
        .filter(|&page| {
            page_boxes(core, page)
                .tables
                .iter()
                .any(|(s, p, _)| *s == Some(SECTION) && *p == Some(para))
        })
        .collect()
}

fn first_line(boxes: &PageBoxes, para: usize) -> Option<BoundingBox> {
    boxes
        .lines
        .iter()
        .filter(|(s, p, _)| *s == Some(SECTION) && *p == Some(para))
        .map(|(_, _, b)| *b)
        .min_by(|a, b| a.y.total_cmp(&b.y))
}

#[test]
fn deferred_frame_keeps_the_blank_restart_line_below_it() {
    let core = core();
    let table_pages = pages_with_table(&core, DEFERRED_TABLE_PARA);
    assert_eq!(table_pages.len(), 1, "그림 2-3 표는 한 쪽에 통째로 놓인다");
    let page = table_pages[0];
    let boxes = page_boxes(&core, page);
    assert!(
        first_line(&boxes, DEFERRED_TABLE_PARA).is_none(),
        "표만 넘어가고 host 줄은 앞 쪽에 남는다"
    );
    assert!(
        compact(&core, page - 1).contains("그림2-2.일본의식량자급력개념"),
        "host 앞 그림 2-2 는 표 앞 쪽에 있다"
    );
    let table = boxes
        .tables
        .iter()
        .find(|(s, p, _)| *s == Some(SECTION) && *p == Some(DEFERRED_TABLE_PARA))
        .map(|(_, _, b)| *b)
        .expect("그림 2-3 표");
    let blank = first_line(&boxes, BLANK_RESTART_PARA).expect("빈 재시작 문단 줄이 표 쪽에 있다");
    let next = first_line(&boxes, NEXT_TEXT_PARA).expect("다음 글 문단 줄");
    assert!(
        blank.y + 0.5 >= table.y + table.height,
        "빈 재시작 줄 {:.2} 은 표 아래끝 {:.2} 아래다",
        blank.y,
        table.y + table.height
    );
    assert!(
        next.y >= blank.y + blank.height - 0.5,
        "다음 글 문단 첫 줄 {:.2} 은 빈 재시작 줄({:.2}, 높이 {:.2}) 아래다 — 빈 줄을 감추지 않는다",
        next.y,
        blank.y,
        blank.height
    );
    assert!(
        compact(&core, page).contains("그림2-3.식량자급력의4가지유형"),
        "표 쪽은 그림 2-3 제목을 담는다"
    );
}

/// 이월 상자 아래 빈 재시작 줄과 그 위치를 잰다: (쪽, 표 아래끝 → 빈 줄 위끝, 빈 줄 → 다음 줄).
fn blank_restart_gaps(core: &DocumentCore) -> (u32, f64, f64) {
    let table_pages = pages_with_table(core, DEFERRED_TABLE_PARA);
    assert_eq!(table_pages.len(), 1, "그림 2-3 표는 한 쪽에 통째로 놓인다");
    let page = table_pages[0];
    let boxes = page_boxes(core, page);
    let table = boxes
        .tables
        .iter()
        .find(|(s, p, _)| *s == Some(SECTION) && *p == Some(DEFERRED_TABLE_PARA))
        .map(|(_, _, b)| *b)
        .expect("그림 2-3 표");
    let blank = first_line(&boxes, BLANK_RESTART_PARA).expect("빈 재시작 문단 줄");
    let next = first_line(&boxes, NEXT_TEXT_PARA).expect("다음 글 문단 줄");
    (page, blank.y - (table.y + table.height), next.y - blank.y)
}

/// 반례: 재시작 문단에 앞 간격이 있어도 저장 줄(vpos 22586 = 바깥 상자 아래끝)의 자리는
/// 그대로다. 앞 간격은 상자 아래끝에 다시 더해지지 않는다(1490000-201400162 s2 pi106:
/// 앞 간격 2500HU 의 빈 재시작 문단이 상자 아래끝보다 33.3px 아래에 그려지던 결함).
#[test]
fn spacing_before_does_not_move_the_stored_blank_restart_line() {
    let plain = core();
    let (page, table_gap, next_gap) = blank_restart_gaps(&plain);

    let mut spaced = core();
    {
        let doc = spaced.document_mut();
        let para = &mut doc.sections[SECTION].paragraphs[BLANK_RESTART_PARA];
        let mut shape = doc.doc_info.para_shapes[para.para_shape_id as usize].clone();
        assert_eq!(shape.spacing_before, 0, "원본 재시작 문단은 앞 간격이 없다");
        shape.spacing_before = 5000;
        shape.raw_data = None;
        doc.doc_info.para_shapes.push(shape);
        para.para_shape_id = (doc.doc_info.para_shapes.len() - 1) as u16;
    }
    let document = spaced.document().clone();
    spaced.set_document(document);
    let (spaced_page, spaced_table_gap, spaced_next_gap) = blank_restart_gaps(&spaced);

    assert_eq!(spaced_page, page, "표와 재시작 줄은 같은 쪽에 있다");
    assert!(
        (spaced_table_gap - table_gap).abs() < 0.5,
        "앞 간격을 준 재시작 줄도 상자 아래끝에서 {table_gap:.2}px 에 놓인다(실제 {spaced_table_gap:.2}px)"
    );
    assert!(
        (spaced_next_gap - next_gap).abs() < 0.5,
        "다음 글 줄은 재시작 줄에서 저장 간격 {next_gap:.2}px 뒤다(실제 {spaced_next_gap:.2}px)"
    );
}

#[test]
fn split_table_first_fragment_takes_the_first_line_of_row_three() {
    let core = core();
    let table_pages = pages_with_table(&core, SPLIT_TABLE_PARA);
    assert_eq!(
        table_pages.len(),
        2,
        "표 2-5 는 두 쪽으로 나뉜다: {table_pages:?}"
    );
    assert_eq!(
        table_pages[1],
        table_pages[0] + 1,
        "두 조각은 이어진 쪽이다"
    );
    let first = compact(&core, table_pages[0]);
    let rest = compact(&core, table_pages[1]);
    assert!(
        first.contains("국민이최저한도로필요로하는열량의공급이곤란해질위"),
        "앞 조각은 행 3 의 첫 줄을 담는다"
    );
    assert!(
        first.contains("대책본부장의직위는각사태에따라유동적임"),
        "표 2-5 앞 조각 쪽은 각주 21 을 담는다"
    );
    assert!(
        !first.contains("험성이있는경우"),
        "행 3 의 둘째 줄은 앞 조각에 없다"
    );
    assert!(
        rest.contains("험성이있는경우"),
        "행 3 의 나머지는 다음 쪽에 있다"
    );
    assert!(
        !rest.contains("국민이최저한도로필요로하는열량의공급이곤란해질위"),
        "행 3 의 첫 줄은 다음 쪽에 다시 나오지 않는다"
    );
}

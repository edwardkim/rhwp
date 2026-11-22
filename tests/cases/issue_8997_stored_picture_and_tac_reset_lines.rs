//! 관계형·고정형 — [#8997] 캡션 그림이 흡수한 빈 host 줄과 리셋 앞 TAC 그림 줄을 저장 사다리대로 조판한다.
//! 주석 갱신: 2026-11-23 04:19 KST (새 파일)
//!
//! 입력: `mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp`
//! (한컴 2020 Print `pdf/issue8997/1543000-202100035-food-security-report-2020.pdf`, 141쪽).
//! 수정 전(devel `622af208ee`) rhwp 는 143쪽이었다.
//! - 구역 2 문단 952 는 빈 host 줄(`sw=0`) 하나와 캡션 위 그림(비TAC·위아래·문단 기준 오프셋 0·위아래
//!   여백 283HU)을 가진다. 한/글은 host 줄을 그림 띠에 흡수해 다음 문단(953) 저장 줄을 그림 상자
//!   아래끝(20984 = 앞 간격 원점 0 + 여백 283 + 캡션 그림 + 여백 283)에 둔다. 조판은 캡션 그림이면
//!   흡수를 돌려주지 않아 흐름이 30px 아래로 내려가, 뒤 표 2-6(34행, 행 나눔)의 첫 조각이 두 행
//!   (2007·2008)을 덜 담고 「자료」 한 줄만 다음 쪽에 남겼다.
//! - 구역 0 문단 11 은 저장 줄 0(vpos 42652, TAC 로고)과 줄 1(vpos 0 리셋, TAC 목차 표)을 가진다.
//!   한컴은 로고를 1쪽 아래, 목차 표를 2쪽에 둔다. 조판은 리셋 앞 줄 fit 판정이 컨트롤 순번과
//!   저장 줄이 1:1 이길 요구해(쪽번호위치·머리말·감추기 컨트롤) 문단 전체를 다음 쪽으로 넘겨,
//!   2쪽이 로고만 있는 쪽이 되었다.
//! - 관계형: 그림 문단 뒤 첫 글줄은 그림 상자 아래끝(그림 아래 + 아래 여백) 바로 뒤에 놓이고,
//!   표 2-6 의 쪽 나눔은 한컴과 같은 행(2008 년 행까지 앞 쪽)에서 일어난다. 로고는 1쪽, 목차 표는
//!   2쪽의 첫 본문 개체다.
//! - 고정형: 전체 쪽수 141(독립 한컴 Print PDF).
//!
//! 측정: 2026-11-23, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! (`silhouette_raw_match_percent`, 색상 경계 보정 전 2px 내용 실루엣), 글꼴은 맑은 고딕(Regular·Bold)·
//! 바탕 + `ttfs/opensource` 를 Native·WASM 두 경로에 공급. source `6ae490cdba`(코드) 위 시험 커밋.
//!
//! - 대상 쪽(1·2·97·98): Native 98.55·99.75·98.89·100.00, fresh WASM 99.46·99.84·98.88·100.00.
//! - 전체 141쪽(한컴 Print 와 같음). 전쪽 최저 Native 24.29(93쪽)·WASM 25.16(93쪽). 90 미만 9쪽
//!   (4·6·68·85·90~93·132)은 본문이 수정 전 devel 의 대응 쪽과 같은 기존 미달이며(쪽 번호만 다름)
//!   90~93쪽은 #8997 의 남은 범위, 나머지는 #9326 으로 분리했다.
//! - 증거: `mydocs/pr/assets/issue_8997/{native,wasm}/food_silhouette.tsv`, 대표 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp";
/// 한컴 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 141;
/// 캡션 그림 host(구역 2) 와 그 다음 문단.
const PICTURE_SECTION: usize = 2;
const PICTURE_PARA: usize = 952;
const NEXT_PARA: usize = 953;
/// 그림의 아래 바깥 여백(283HU) — 원본 개체 속성.
const PICTURE_MARGIN_BOTTOM_PX: f64 = 283.0 * 96.0 / 7200.0;
/// 리셋 앞 TAC 로고와 목차 표의 문단(구역 0).
const LOGO_SECTION: usize = 0;
const LOGO_PARA: usize = 11;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 열기")
}

#[derive(Default)]
struct PageBoxes {
    images: Vec<(Option<usize>, Option<usize>, BoundingBox)>,
    tables: Vec<(Option<usize>, Option<usize>, BoundingBox)>,
    lines: Vec<(Option<usize>, Option<usize>, BoundingBox)>,
}

/// 본문 흐름의 그림·표·줄을 모은다. 표 칸 안과 머리말·꼬리말은 본문 문단이 아니다.
fn collect(node: &RenderNode, out: &mut PageBoxes) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            out.tables
                .push((table.section_index, table.para_index, node.bbox));
            return;
        }
        RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage => return,
        RenderNodeType::Image(image) => {
            out.images
                .push((image.section_index, image.para_index, node.bbox))
        }
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

/// 쪽 본문(머리말·꼬리말 제외)의 글자를 표 칸까지 그려진 순서로 이어 붙인다.
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

fn page_of_text(core: &DocumentCore, needle: &str) -> u32 {
    (0..core.page_count())
        .find(|&page| compact(core, page).contains(needle))
        .unwrap_or_else(|| panic!("「{needle}」 를 담은 쪽이 없다"))
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(core().page_count(), PDF_PAGES, "한컴 Print 141쪽");
}

#[test]
fn captioned_picture_host_line_is_absorbed_and_table_splits_like_hancom() {
    let core = core();
    let page = page_of_text(&core, "표2-6.중국의주요식량생산량추이");
    let boxes = page_boxes(&core, page);
    let picture = boxes
        .images
        .iter()
        .find(|(s, p, _)| *s == Some(PICTURE_SECTION) && *p == Some(PICTURE_PARA))
        .map(|(_, _, b)| *b)
        .expect("표 2-6 쪽에 캡션 그림이 있다");
    let next_line = boxes
        .lines
        .iter()
        .filter(|(s, p, _)| *s == Some(PICTURE_SECTION) && *p == Some(NEXT_PARA))
        .map(|(_, _, b)| *b)
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("그림 다음 문단 줄");
    let frame_bottom = picture.y + picture.height + PICTURE_MARGIN_BOTTOM_PX;
    assert!(
        (next_line.y - frame_bottom).abs() <= 0.5,
        "그림 다음 줄 위끝 {:.2} 은 그림 상자 아래끝 {frame_bottom:.2} 바로 뒤여야 한다(빈 host 줄 흡수)",
        next_line.y
    );

    // 한컴 Print: 표 2-6 첫 조각은 2008 년 행까지, 다음 쪽은 2009 년 행부터.
    let first = compact(&core, page);
    let rest = compact(&core, page + 1);
    assert!(
        first.contains("200853434.9") && first.contains("200750413.9"),
        "표 2-6 첫 조각은 2007·2008 년 행을 담아야 한다"
    );
    assert!(
        rest.starts_with("200953940.9"),
        "다음 쪽은 2009 년 행부터 이어진다"
    );
    assert!(!rest.contains("200853434.9"), "2008 년 행은 다음 쪽에 없다");
    assert!(
        rest.contains("자료:중국해관"),
        "그림 2-7 의 자료 줄은 표 나머지와 같은 쪽이다"
    );
}

#[test]
fn tac_logo_line_before_stored_reset_stays_on_the_title_page() {
    let core = core();
    let title = page_boxes(&core, 0);
    assert!(
        title
            .images
            .iter()
            .any(|(s, p, _)| *s == Some(LOGO_SECTION) && *p == Some(LOGO_PARA)),
        "리셋 앞 TAC 로고 줄은 1쪽(표지)에 남는다"
    );
    let contents = page_boxes(&core, 1);
    assert!(
        contents
            .images
            .iter()
            .all(|(s, p, _)| !(*s == Some(LOGO_SECTION) && *p == Some(LOGO_PARA))),
        "로고만 다음 쪽으로 넘어가지 않는다"
    );
    assert!(
        contents
            .tables
            .iter()
            .any(|(s, p, _)| *s == Some(LOGO_SECTION) && *p == Some(LOGO_PARA)),
        "리셋 뒤 목차 표는 2쪽에 놓인다"
    );
    assert!(
        compact(&core, 1).starts_with("목차"),
        "2쪽은 목차로 시작한다(한컴 Print 2쪽)"
    );
}

//! 관계형·고정형 — [#8997] 캡션 그림이 흡수한 빈 host 줄과 리셋 앞 TAC 그림 줄을 저장 사다리대로 조판한다.
//! 주석 갱신: 2026-11-23 05:36 KST (새 파일, devel `93e8615236` 위 rebase 뒤 재측정)
//!
//! 입력: `mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp`
//! (한컴 2020 Print `pdf/issue8997/1543000-202100035-food-security-report-2020.pdf`, 141쪽).
//! 수정 전(devel `93e8615236`) rhwp 는 143쪽이었다.
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
//! 바탕 + `ttfs/opensource` 를 Native·WASM 두 경로에 공급. source `ee3bc54011`(코드) 위 시험 커밋.
//!
//! - 대상 쪽(1·2·97·98): Native 98.55·99.75·98.89·100.00, fresh WASM 99.46·99.84·98.88·100.00.
//! - 전체 141쪽(한컴 Print 와 같음). 전쪽 최저 Native 24.29(93쪽)·WASM 25.16(93쪽). 90 미만 9쪽
//!   (4·6·68·85·90~93·132)은 본문이 수정 전 devel 의 대응 쪽과 같은 기존 미달이며(쪽 번호만 다름)
//!   90~93쪽은 #8997 의 남은 범위, 나머지는 #9326 으로 분리했다.
//! - 증거: `mydocs/pr/assets/issue_8997/{native,wasm}/food_silhouette.tsv`, 대표 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::hwpunit_to_px;
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

/// 관계형 — 한컴 2020 Print 90~93쪽에서 그림 뒤 본문과 갈린 표의 글줄·각주 소속을 확인한다.
#[test]
fn relocated_chart_and_split_rows_keep_their_print_page_owners() {
    let core = core();
    let pages: Vec<_> = (89..93).map(|page| compact(&core, page)).collect();
    let print_owners = [
        (90, "그림2-3.식량자급력의4가지유형"),
        (90, "일본정부는외국으로부터의수입에의존"),
        (91, "표2-4.일본의주요농산물비축기준"),
        (92, "국민이최저한도로필요로하는열량의공급"),
        (92, "대책본부장의직위는각사태에따라유동적임"),
        (93, "험성이있는경우"),
    ];
    for (expected_page, text) in print_owners {
        let actual: Vec<_> = pages
            .iter()
            .enumerate()
            .filter(|(_, contents)| contents.contains(text))
            .map(|(index, _)| 90 + index)
            .collect();
        assert_eq!(actual, [expected_page], "「{text}」의 한컴 Print 쪽 소속");
    }
}

const DEFERRED_TABLE_PARA: usize = 893;
const BLANK_RESTART_PARA: usize = 896;
const BODY_AFTER_RESTART: usize = 897;

fn first_food_line(boxes: &PageBoxes, paragraph: usize) -> Option<BoundingBox> {
    boxes
        .lines
        .iter()
        .filter(|(section, index, _)| *section == Some(2) && *index == Some(paragraph))
        .map(|(_, _, bounds)| *bounds)
        .min_by(|left, right| left.y.total_cmp(&right.y))
}

fn restart_gaps(boxes: &PageBoxes) -> (f64, f64) {
    let (_, _, table) = boxes
        .tables
        .iter()
        .find(|(section, index, _)| *section == Some(2) && *index == Some(DEFERRED_TABLE_PARA))
        .expect("90쪽에 그림 2-3 표가 있다");
    let blank = first_food_line(boxes, BLANK_RESTART_PARA).expect("표 뒤에 빈 재시작 줄이 있다");
    let next = first_food_line(boxes, BODY_AFTER_RESTART).expect("빈 줄 뒤에 본문이 있다");
    assert!(blank.y >= table.y + table.height);
    assert!(next.y >= blank.y + blank.height);
    (blank.y - (table.y + table.height), next.y - blank.y)
}

/// 관계형 — 그림 2-3의 공백 호스트는 89쪽, 표와 빈 재시작 줄은 90쪽을 소유한다.
#[test]
fn deferred_table_keeps_whitespace_host_on_previous_print_page() {
    let core = core();
    let before = page_boxes(&core, 88);
    let after = page_boxes(&core, 89);
    assert!(first_food_line(&before, DEFERRED_TABLE_PARA).is_some());
    assert!(first_food_line(&after, DEFERRED_TABLE_PARA).is_none());
    assert!(before
        .tables
        .iter()
        .all(|(section, index, _)| *section != Some(2) || *index != Some(DEFERRED_TABLE_PARA)));
    restart_gaps(&after);
}

/// 관계형 — 원본 저장 줄이 인증한 표 아래 원점을 유지한다.
#[test]
fn stored_restart_line_consumes_paragraph_spacing_before_its_origin() {
    let mut core = core();
    let baseline = restart_gaps(&page_boxes(&core, 89));
    let document = core.document_mut();
    let paragraph = &mut document.sections[2].paragraphs[BLANK_RESTART_PARA];
    let mut shape = document.doc_info.para_shapes[paragraph.para_shape_id as usize].clone();
    assert_eq!(shape.spacing_before, 0);
    shape.spacing_before = 5000;
    shape.raw_data = None;
    document.doc_info.para_shapes.push(shape);
    paragraph.para_shape_id = (document.doc_info.para_shapes.len() - 1) as u16;
    let document = core.document().clone();
    core.set_document(document);
    let with_spacing = restart_gaps(&page_boxes(&core, 89));
    assert!((baseline.0 - with_spacing.0).abs() <= 0.5);
    assert!((baseline.1 - with_spacing.1).abs() <= 0.5);
}

/// 관계형 — 6쪽 목차 표는 빈 줄 전진을 포함한 원본 내용 전체를 개체 선언 안에 배치한다.
#[test]
fn centered_contents_table_uses_its_declared_object_frame() {
    let core = core();
    let page = 5;
    let section = 2;
    let paragraph = 25;
    let [Control::Table(source)] = core.document().sections[section].paragraphs[paragraph]
        .controls
        .as_slice()
    else {
        panic!("원본 목차 표가 없다");
    };
    assert!(source.cells[0].height > source.common.height);
    let frame = page_boxes(&core, page)
        .tables
        .into_iter()
        .find(|(s, p, _)| *s == Some(section) && *p == Some(paragraph))
        .map(|(_, _, bounds)| bounds)
        .expect("한컴 Print와 같은 6쪽에 목차 표가 있다");
    let declared = hwpunit_to_px(source.common.height as i32, 96.0);
    assert!(
        (frame.height - declared).abs() < 1e-6,
        "표의 실제 높이 {}가 원본 개체 선언 {declared}을 따른다",
        frame.height,
    );
    let contents = compact(&core, page);
    assert!(contents.contains("제1장서론") && contents.contains("생산,소비"));
}

fn picture_between_text_rows(
    core: &DocumentCore,
    page: u32,
    paragraph: usize,
) -> (BoundingBox, BoundingBox) {
    let page = core.build_page_render_tree(page).expect("한컴 Print 쪽");
    let mut remaining = vec![&page.root];
    let table = loop {
        let node = remaining.pop().expect("그림을 담은 원본 표");
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if table.section_index == Some(2) && table.para_index == Some(paragraph))
        {
            break node;
        }
        remaining.extend(&node.children);
    };
    let picture_row = table
        .children
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 1))
        .expect("가운데 그림 행");
    let picture = picture_row
        .children
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::Image(_)))
        .expect("그림 행의 원본 그림");
    (picture_row.bbox, picture.bbox)
}

/// 관계형 — 85쪽의 그림은 설명·출처 행 사이에서 자신의 전체 높이를 보존한다.
#[test]
fn picture_between_caption_and_source_stays_inside_its_row() {
    let (row, picture) = picture_between_text_rows(&core(), 84, 845);
    assert!(picture.y > row.y, "그림 위 안 여백");
    assert!(
        picture.y + picture.height < row.y + row.height,
        "그림의 전체 높이가 출처 행 전에 들어가야 한다",
    );
}

/// 관계형 — 89쪽은 폭 없는 저장 줄의 위치와 무관하게 그림이 행의 가운데에 놓인다.
#[test]
fn declared_picture_row_centers_the_picture_instead_of_its_empty_line() {
    let (row, picture) = picture_between_text_rows(&core(), 88, 884);
    let above = picture.y - row.y;
    let below = row.y + row.height - picture.y - picture.height;
    assert!(above > 0.0 && below > 0.0, "그림 위·아래 안 여백");
    assert!(
        (above - below).abs() < 1e-6,
        "그림 위·아래 여백 {above} / {below}"
    );
}

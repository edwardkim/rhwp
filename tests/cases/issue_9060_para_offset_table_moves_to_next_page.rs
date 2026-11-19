//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-19 08:07:10 KST (Asia/Seoul).
//! 직전 파일 변경: f28dbe339ce74531e83e0a3242c283955a322b0c 2026-11-19T00:28:25+09:00.
//!
//! 입력 1: `samples/issue9060/m2-full-t2tall-para-offset-next-page.hwp`.
//! 입력 1 SHA-256: e774998d8ceb0807f435c62f878c1cd7f91b2be758f572239ded1e735f3385f9.
//! 생성: 합성 HWPX(`gen_m2.py` `M2_full_t2tall`)를 한/글 2020 이 HWP 로 다시 저장한 원본.
//! 저장 제품: 한컴오피스 2020 11.0.0.9136(`rhwp info` lastSavedWith).
//! 독립 Print: `pdf/issue9060/m2-full-t2tall-para-offset-next-page-2020.pdf`, 4쪽.
//! PDF SHA-256: 95979023e1e60854e2e6183301034ca5fa671f3c3857c5824eb52f9037620949.
//! PDF 출처: MCP engine 2020(HOffice110 11.0.0.9136) Print, `pdf_print_method` 0,
//! `hancom2020_pdf_driver_one_up`, job a67a8eb1-3c14-452f-80fb-4b743d22b180. Creator Hwp 2020.
//!
//! 입력 2(반례, 앵커가 쪽 위끝에 놓인 표): `samples/issue9060/a1-small-top-para-offset.hwpx`.
//! 입력 2 SHA-256: 83b6a5577e3541c3c49c10609b6881878dc38e76091c59b092f0c11c67e9d524.
//! 저장 제품: 한컴오피스 2024 13.0.0.3622.
//! 독립 Print: `pdf/issue9060/a1-small-top-para-offset-2024.pdf`, 2쪽.
//! PDF SHA-256: 12c88cc4100a91788bbaed4e85dead2696186f01b6b99ad47a6a3e8ea6b580f3.
//! PDF 출처: MCP engine 2024(HOffice130 13.0.0.3901) Print, `pdf_print_method` 0,
//! `frame_print_to_pdf_ex_one_up`, job 40da0591-1fe2-45e2-8905-e84c5fc6d85d.
//!
//! 입력 3(반례, 오프셋 자리에서 그대로 나뉘는 표): `samples/issue9060/t34-rows4000-para-offset-split.hwpx`.
//! 입력 3 SHA-256: e3a546b5b49a7d1bd34cb7532ebd5cbc6d25b7894eab9754133a0b4f9c113e16.
//! 저장 제품: 한컴오피스 2024 13.0.0.3622.
//! 독립 Print: `pdf/issue9060/t34-rows4000-para-offset-split-2024.pdf`, 2쪽.
//! PDF SHA-256: d584d2d4a71f4f98efb3eb5ba1248b6dfcaa64fb7c7dcf4c530b11820523e10f.
//! PDF 출처: MCP engine 2024(HOffice130 13.0.0.3901) Print, `pdf_print_method` 0,
//! `frame_print_to_pdf_ex_one_up`, job e968af42-d75b-458e-aa30-5045db4bb3ea.
//! 같은 생성기의 T35(앵커 한 줄 아래)는 한/글이 셀 행 안을 자르는 형상이라 2쪽 raw 29.41% 로
//! 회귀 추가 선행 조건(90%)을 충족하지 않아 쓰지 않는다(행 중간 분할은 #9060 범위 밖).
//!
//! 측정일: 2026-11-19. 소스 2c5461ff24(코드), Native release-test native-skia
//! SHA-256 ae3fd109ac31a5e96144806eeebd3d62d96eee05689eb74452559f31e465216f,
//! fresh WASM `rhwp_bg.wasm` SHA-256 dab60baecf026184632f99e840b963eb3894531825811143ed9f08db5d539b2c.
//! 방법: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 색상 경계 보정 전 2px raw.
//! 글꼴: ODA 보고서 측정과 같은 `RHWP_FONT_PATH` 설치 글꼴 폴더(`direct-font-programs`). 회귀 실행에는 요구하지 않는다.
//! 입력 1(`--compat 2022`): Native 전 4쪽 각각 100.00%, fresh WASM 전 4쪽 각각 100.00%.
//! 입력 2(`--compat 2024`): Native 100.00·100.00%, fresh WASM 99.49·98.18%.
//! 입력 3(`--compat 2024`): Native 100.00·100.00%, fresh WASM 99.33·100.00%.
//! 전쪽 최저: Native 100.00%, fresh WASM 98.18%(입력 2 의 2쪽). 쪽수는 독립 PDF와 같다(4·2·2).
//! 증거: `mydocs/pr/assets/issue_9060_oda/native/{m2,a1,t34}_silhouette.tsv`,
//! `mydocs/pr/assets/issue_9060_oda/wasm/{m2,a1,t34}_silhouette.tsv`.
//! 고정형은 세 독립 PDF의 전체 쪽수(4·2·2)다. 절대 px 좌표는 고정하지 않는다.
//! 관계형은 표 첫 조각의 쪽 소유, 쪽 본문 위끝에서 표 위끝까지의 거리와 원본 문단 기준
//! 세로 오프셋(`vertical_offset`)의 관계, 앞 글줄·뒤 문단과의 순서·겹침 없음이다.
//! 한/글 규칙: 문단 기준 자리차지 표의 첫 유닛이 앵커 쪽 `문단 위치 + 오프셋` 자리에 들어가지
//! 않으면 앵커 줄은 그 쪽에 남고 표만 다음 쪽에서 오프셋 없이 본문 위끝 + 바깥 위 여백 + 위
//! 캡션에서 시작한다. 앵커가 실제로 그 쪽 위끝에 있으면 오프셋을 그대로 쓰고(입력 2), 첫
//! 유닛이 들어가면 오프셋 자리에서 나눈다(입력 3).
//! 실물의 절대 px 좌표를 고정하지 않으며 회귀 실행에 설치된 OTF·시스템 글꼴을 요구하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const MOVED: &str = "samples/issue9060/m2-full-t2tall-para-offset-next-page.hwp";
const PAGE_TOP: &str = "samples/issue9060/a1-small-top-para-offset.hwpx";
const SPLIT_HERE: &str = "samples/issue9060/t34-rows4000-para-offset-split.hwpx";

/// 이월 대상 표의 문단(빈 앵커)과 그 앞 표의 문단.
const MOVED_PARA: usize = 2;
const PREVIOUS_TABLE_PARA: usize = 1;
const MOVED_FOLLOWING_PARA: usize = 3;
/// 반례 입력의 표 문단.
const PAGE_TOP_PARA: usize = 4;
const SPLIT_PARA: usize = 35;

fn document(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {sample}: {e:?}"))
}

/// 원본 표의 문단 기준 세로 오프셋(px, 96 DPI)과 바깥 위 여백(px).
fn offset_and_outer_top_px(core: &DocumentCore, para: usize) -> (f64, f64) {
    let host = &core.document().sections[0].paragraphs[para];
    let Some(Control::Table(table)) = host
        .controls
        .iter()
        .find(|c| matches!(c, Control::Table(_)))
    else {
        panic!("문단 {para} 에 표가 없다");
    };
    assert!(
        !table.common.treat_as_char,
        "문단 {para} 표는 자리차지 개체다"
    );
    let offset = table.common.vertical_offset as i32;
    assert!(
        offset > 0,
        "문단 {para} 표의 세로 오프셋은 양수다: {offset}"
    );
    let px = |hu: i32| f64::from(hu) * 96.0 / 7200.0;
    (px(offset), px(i32::from(table.outer_margin_top)))
}

#[derive(Default)]
struct Page {
    body_top: Option<f64>,
    body_bottom: Option<f64>,
    /// 표 밖 본문 글줄(문단, 위끝, 아래끝).
    lines: Vec<(usize, f64, f64)>,
    /// 최상위 표(문단, 위끝, 아래끝).
    tables: Vec<(usize, f64, f64)>,
}

impl Page {
    fn of(core: &DocumentCore, page: u32) -> Self {
        fn walk(node: &RenderNode, in_table: bool, out: &mut Page) {
            let mut inside = in_table;
            match &node.node_type {
                RenderNodeType::Body { .. } if out.body_top.is_none() => {
                    out.body_top = Some(node.bbox.y);
                    out.body_bottom = Some(node.bbox.y + node.bbox.height);
                }
                RenderNodeType::Table(table) => {
                    if let Some(para) = table.para_index.filter(|_| !in_table) {
                        out.tables
                            .push((para, node.bbox.y, node.bbox.y + node.bbox.height));
                    }
                    inside = true;
                }
                RenderNodeType::TextLine(line) if !in_table => {
                    if let Some(para) = line.para_index {
                        out.lines
                            .push((para, node.bbox.y, node.bbox.y + node.bbox.height));
                    }
                }
                _ => {}
            }
            for child in &node.children {
                walk(child, inside, out);
            }
        }
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
        let mut out = Page::default();
        walk(&tree.root, false, &mut out);
        out
    }

    fn table(&self, para: usize) -> Option<(f64, f64)> {
        self.tables
            .iter()
            .filter(|(p, _, _)| *p == para)
            .map(|(_, top, bottom)| (*top, *bottom))
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
    }

    fn body(&self) -> (f64, f64) {
        (
            self.body_top.expect("본문 영역 노드"),
            self.body_bottom.expect("본문 영역 노드"),
        )
    }
}

/// 문단 표가 처음 나타나는 쪽(0 기준)과 그 쪽.
fn first_page_with_table(core: &DocumentCore, para: usize) -> (u32, Page) {
    (0..core.page_count())
        .map(|page| (page, Page::of(core, page)))
        .find(|(_, page)| page.table(para).is_some())
        .unwrap_or_else(|| panic!("문단 {para} 표가 어느 쪽에도 없다"))
}

#[test]
fn page_counts_match_hancom_print() {
    for (sample, pages) in [(MOVED, 4), (PAGE_TOP, 2), (SPLIT_HERE, 2)] {
        assert_eq!(
            document(sample).page_count(),
            pages,
            "{sample}: 첨부 독립 한컴 Print PDF는 {pages}쪽"
        );
    }
}

/// [#9060] 앞 표가 2쪽 바닥을 채워 첫 유닛이 들어가지 않는 표는 3쪽에서 오프셋 없이 시작한다.
#[test]
fn table_without_room_for_first_unit_opens_next_page_without_paragraph_offset() {
    let core = document(MOVED);
    let (offset, outer_top) = offset_and_outer_top_px(&core, MOVED_PARA);
    let (page, third) = first_page_with_table(&core, MOVED_PARA);
    assert_eq!(page, 2, "독립 Print처럼 표는 3쪽에서 처음 나타난다");

    // 앞 표는 1·2쪽에서 끝나고 3쪽에 오지 않는다.
    let second = Page::of(&core, 1);
    let (_, previous_bottom) = second
        .table(PREVIOUS_TABLE_PARA)
        .expect("2쪽에 앞 표의 이어진 조각이 있다");
    assert!(third.table(PREVIOUS_TABLE_PARA).is_none());
    let (_, second_body_bottom) = second.body();
    let room = second_body_bottom - previous_bottom;
    assert!(
        room < offset + outer_top,
        "2쪽에 남은 {room:.2}px 는 오프셋 {offset:.2}px + 바깥 위 여백 {outer_top:.2}px 보다 작다"
    );

    let (body_top, body_bottom) = third.body();
    let (top, bottom) = third.table(MOVED_PARA).expect("3쪽 표 조각");
    let gap = top - body_top;
    assert!(
        gap >= outer_top - 0.5,
        "표는 바깥 위 여백 아래에서 시작한다: 거리 {gap:.2}px, 여백 {outer_top:.2}px"
    );
    assert!(
        gap < offset,
        "이월한 표는 문단 오프셋을 다시 더하지 않는다: 본문 위끝에서 {gap:.2}px, 오프셋 {offset:.2}px"
    );
    assert!(
        bottom <= body_bottom + 0.5,
        "3쪽 표 조각은 본문 안에서 끝난다"
    );
    assert!(
        third.lines.iter().all(|(_, line_top, line_bottom)| {
            *line_bottom <= top + 0.5 || *line_top >= bottom - 0.5
        }),
        "3쪽 본문 글줄은 표와 겹치지 않는다"
    );

    // 뒤 문단은 표 뒤 4쪽에 한 번 놓인다.
    let owners: Vec<u32> = (0..core.page_count())
        .filter(|&page| {
            Page::of(&core, page)
                .lines
                .iter()
                .any(|(para, _, _)| *para == MOVED_FOLLOWING_PARA)
        })
        .collect();
    assert_eq!(owners, [3], "뒤 문단은 4쪽에만 있다");
}

/// 반례: 쪽 나눔으로 앵커가 2쪽 위끝에 놓인 표는 그 쪽에서 오프셋을 그대로 쓴다.
/// 두 입력은 같은 생성기의 표(바깥 여백·위 캡션·오프셋 2668HU)이므로 본문 위끝에서 표
/// 위끝까지의 거리 차가 곧 오프셋이다.
#[test]
fn page_top_anchor_keeps_offset_that_the_moved_table_drops() {
    let page_top_core = document(PAGE_TOP);
    let (offset, _) = offset_and_outer_top_px(&page_top_core, PAGE_TOP_PARA);
    let (page, second) = first_page_with_table(&page_top_core, PAGE_TOP_PARA);
    assert_eq!(page, 1, "쪽 나눔 문단의 표는 2쪽에 있다");
    let (body_top, _) = second.body();
    let (anchored_top, _) = second.table(PAGE_TOP_PARA).expect("2쪽 표");
    let anchored_gap = anchored_top - body_top;
    assert!(
        anchored_gap >= offset,
        "앵커가 쪽 위끝에 있으면 오프셋을 쓴다: {anchored_gap:.2}px >= {offset:.2}px"
    );
    let anchor = second
        .lines
        .iter()
        .find(|(para, _, _)| *para == PAGE_TOP_PARA)
        .expect("2쪽에 앵커 글줄이 있다");
    assert!(anchor.1 < anchored_top, "앵커 글줄은 표보다 위에 있다");

    let moved_core = document(MOVED);
    let (moved_offset, _) = offset_and_outer_top_px(&moved_core, MOVED_PARA);
    assert!(
        (moved_offset - offset).abs() < 0.01,
        "두 표의 오프셋은 같다"
    );
    let third = Page::of(&moved_core, 2);
    let (moved_body_top, _) = third.body();
    let (moved_top, _) = third.table(MOVED_PARA).expect("3쪽 표");
    let difference = anchored_gap - (moved_top - moved_body_top);
    assert!(
        (difference - offset).abs() <= 1.0,
        "두 표의 본문 위끝 거리 차 {difference:.2}px 는 오프셋 {offset:.2}px 와 같다"
    );
}

/// 반례: 오프셋 자리에 첫 행이 들어가면 표는 앵커 쪽에서 오프셋을 쓰고 그 자리에서 나뉜다.
#[test]
fn first_unit_that_fits_below_the_offset_splits_on_the_anchor_page() {
    let core = document(SPLIT_HERE);
    let (offset, _) = offset_and_outer_top_px(&core, SPLIT_PARA);
    let (page, first) = first_page_with_table(&core, SPLIT_PARA);
    assert_eq!(page, 0, "표 첫 조각은 앵커와 같은 1쪽에 있다");
    let anchor_top = first
        .lines
        .iter()
        .filter(|(para, _, _)| *para == SPLIT_PARA)
        .map(|(_, top, _)| *top)
        .reduce(f64::min)
        .expect("1쪽에 앵커 글줄이 있다");
    let (top, bottom) = first.table(SPLIT_PARA).expect("1쪽 표 조각");
    assert!(
        top - anchor_top >= offset,
        "표 위끝은 앵커 글줄 위끝 + 오프셋 아래다: {:.2}px >= {offset:.2}px",
        top - anchor_top
    );
    let (_, body_bottom) = first.body();
    assert!(bottom <= body_bottom + 0.5, "1쪽 조각은 본문 안에서 끝난다");
    assert!(
        first
            .lines
            .iter()
            .filter(|(para, _, _)| *para < SPLIT_PARA)
            .all(|(_, _, line_bottom)| *line_bottom <= top + 0.5),
        "앞 문단 글줄은 표 위에 있다"
    );
    assert!(
        Page::of(&core, 1).table(SPLIT_PARA).is_some(),
        "나머지 행은 2쪽으로 이어진다"
    );
}

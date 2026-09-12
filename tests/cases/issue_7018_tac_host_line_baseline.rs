//! [Issue #7018] 자리차지 표가 **자기 저장 줄**을 가질 때, 그 앞 텍스트가 표 줄의 baseline
//! 을 받아 165.4px 아래로 내려가 표와 겹쳤다.
//!
//! `2769535` 2쪽 문단 27 — 텍스트 `  마. 행정박물류` 뒤에 `treatAsChar` 표가 붙는다.
//! 한/글이 저장한 사다리는 **두 줄**이다.
//!
//! ```text
//!   seg[0] vpos=39764 lh=1200  th=1200  bl=1020   textpos=0    ← 글자 줄
//!   seg[1] vpos=41924 lh=15792 th=15792 bl=13423  textpos=11   ← 표 줄
//! ```
//!
//! `textpos=11` 은 본문 길이와 같다 — 표가 줄 하나를 통째로 가졌다는 한/글의 기록이다.
//! 본문 시작 오프셋 75.6px 을 더하면 글자 줄 상단 605.8 · baseline 619.4, 표 상단 636.4 이고
//! 기준 PDF 잉크(605.5..620.5)와 맞는다.
//!
//! ⭐ **원인은 baseline 출처 하나다.** `layout_inline_table_paragraph` 는 `table_seg`(표 줄)와
//! `text_seg`(글자 줄)를 이미 갈라 놓고도, `wrapped_below_table` 이 아니면 텍스트 런에
//! **표 줄의 baseline** 을 썼다. 그래서 런 상자 높이가 179.0px(=13423HU)이 되고 baseline 이
//! 상자 바닥 784.8px 에 찍혔다.
//!
//! ⭐ **판정은 크기가 아니라 저장 증거다.** 표 컨트롤이 앉은 글자 위치와 표 seg 의 시작이
//! 같을 때만 "표가 줄을 가졌다"로 본다. 표가 글자 사이에 진짜로 끼어드는 인라인 형상은 그
//! 줄이 표보다 **앞**에서 시작하므로 이 분기에 들어오지 않는다.
//!
//! ⚠ 돌연변이 4종이 원인을 한 모델로 닫는다(`ensure_min_baseline` 하한 0.8×글자크기 포함):
//!
//! ```text
//!   원본                 표 줄 bl 13423 → 179.0px   관측 179.0
//!   글자 줄 bl 1020→3000 변화 없음                   관측 179.0   ← 글자 줄 값은 안 읽었다
//!   표 줄 bl →5000       5000HU = 66.7px             관측  66.7
//!   표 줄 bl →500        하한 0.8×16 = 12.8px        관측  12.8
//!   표 줄 lineseg 삭제   글자 줄 값으로 폴백          관측  13.6 = 오라클
//! ```
//!
//! ⭐ **판정은 문단 단위가 아니라 런별 줄 소속이다** (PR #7044 검토 반영). 종전 구현은
//! 첫 표로 문단 단위 불린을 만들어 여러 런에 재사용했는데, 그러면 ① 마지막 run 출력 경로
//! (`remaining_bbox_h`)에 보정이 빠지고 ② 여러 줄·여러 표 문단에서 첫 표의 판정이 다른
//! 줄로 번질 수 있다. 지금은 런의 글자 위치가 속한 저장 줄을 찾아 그 줄의 baseline 을 쓴다.
//!
//! ⭐ **축**: `LineSeg.textpos` 와 `para.char_offsets` 는 같은 문단 UTF-16 축이고 **컨트롤
//! 슬롯을 포함**한다. `para.text` 문자만 합산하면 선행 컨트롤이 있는 문단에서 어긋난다.
//! 이 fixture 로 그 눈금이 같음을 아래 `stored_ladder_axis_matches_char_offsets` 가 잠근다.
//!
//! ⚠ 이 문단은 제목(`charPrIDRef=16`)과 후행 공백·표(`=17`)로 나뉜다. 제목은 글자 스타일
//! 변경 지점에서 중간 flush 되어 다른 경로로 나오고, **후행 공백은 마지막 run 경로**로
//! 나온다. 그래서 제목만 보는 시험은 그 경로의 누락을 못 잡는다 — 아래 시험은 두 런을
//! 함께 단정한다.
//!
//! 기준: 한/글 2020(저장 버전) — `pdf/2769535-records-inspection-plan-2020.pdf`
//! (`hwp2024Convert` engine 2020, 2쪽).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7018/2769535-records-inspection-plan.hwpx";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#7018 정식 HWPX fixture 읽기")
}

fn collect_runs<'a>(node: &'a RenderNode, needle: &str, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            out.push(node);
        }
    }
    for child in &node.children {
        collect_runs(child, needle, out);
    }
}

fn host_run(core: &DocumentCore) -> RenderNode {
    let page = core.build_page_render_tree(1).expect("2쪽 render tree");
    let mut found = Vec::new();
    collect_runs(&page.root, "행정박물", &mut found);
    assert_eq!(found.len(), 1, "`마. 행정박물류` 런은 2쪽에 하나여야 한다");
    found[0].clone()
}

/// 글자 런의 상자 높이는 **글자 줄의 baseline**(1020HU = 13.6px)이어야 한다.
/// 수정 전에는 표 줄의 13423HU = 179.0px 을 받았다.
#[test]
fn tac_host_text_uses_its_own_stored_line_baseline() {
    let core = DocumentCore::from_bytes(&sample()).expect("문서 로드");
    let run = host_run(&core);
    assert!(
        (run.bbox.height - 13.6).abs() <= 1.0,
        "글자 런 상자 높이는 글자 줄 baseline 13.6px 이어야 한다 — 실측 {:.1}px \
         (표 줄 값 179.0px 을 받으면 #7018 회귀)",
        run.bbox.height,
    );
}

/// baseline(상자 바닥)이 한/글 2020 잉크 구간 안에 있어야 한다 — 수정 전 784.8px.
#[test]
fn tac_host_text_baseline_matches_the_hangul_oracle() {
    let core = DocumentCore::from_bytes(&sample()).expect("문서 로드");
    let run = host_run(&core);
    let baseline = run.bbox.y + run.bbox.height;
    assert!(
        (run.bbox.y - 605.8).abs() <= 1.0,
        "줄 상자 상단은 저장 사다리가 말하는 605.8px 이어야 한다 — 실측 {:.1}px",
        run.bbox.y,
    );
    assert!(
        (605.0..=622.0).contains(&baseline),
        "baseline 은 한/글 2020 잉크 구간(605.5..620.5) 안이어야 한다 — 실측 {baseline:.1}px \
         (수정 전 784.8px, 165.4px 하강)",
    );
}

/// 표 자체는 움직이지 않는다 — 이 수정은 **글자 런의 baseline 출처**만 바꾼다.
#[test]
fn the_table_placement_is_unchanged() {
    let core = DocumentCore::from_bytes(&sample()).expect("문서 로드");
    let page = core.build_page_render_tree(1).expect("2쪽 render tree");
    let mut tallest = 0.0f64;
    let mut top = 0.0f64;
    fn walk(node: &RenderNode, tallest: &mut f64, top: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) && node.bbox.height > *tallest {
            *tallest = node.bbox.height;
            *top = node.bbox.y;
        }
        for child in &node.children {
            walk(child, tallest, top);
        }
    }
    walk(&page.root, &mut tallest, &mut top);
    assert!(
        (top - 636.4).abs() <= 2.5,
        "표 상단은 수정 전과 같은 636.4px 이어야 한다 — 실측 {top:.1}px          (이 수정은 글자 런의 baseline 출처만 바꾼다)",
    );
}

/// 쪽수는 한/글 2020 과 같은 2쪽이다.
#[test]
fn page_count_matches_the_oracle() {
    let core = DocumentCore::from_bytes(&sample()).expect("문서 로드");
    assert_eq!(core.page_count(), 2, "한/글 2020 과 같은 2쪽이어야 한다");
}

/// 같은 줄의 **모든** 런이 그 줄의 baseline 을 쓴다 — 후행 공백 런까지.
///
/// 제목은 글자 스타일 변경으로 중간 flush 되지만 후행 공백은 마지막 run 경로로 나온다.
/// 그 경로에 보정이 빠지면 이 시험만 빨강이 된다 (PR #7044 검토 지적 1).
#[test]
fn every_run_on_the_host_line_uses_that_lines_baseline() {
    let core = DocumentCore::from_bytes(&sample()).expect("문서 로드");
    let page = core.build_page_render_tree(1).expect("2쪽 render tree");
    let mut host = Vec::new();
    fn collect(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            // 호스트 줄(상단 605.8px)에 놓인 런만
            if (node.bbox.y - 605.8).abs() <= 1.0 {
                out.push((node.bbox.height, node.bbox.y, run.text.clone()));
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    collect(&page.root, &mut host);
    assert!(
        host.len() >= 2,
        "호스트 줄에는 제목 런과 후행 공백 런이 함께 있어야 한다 — 실측 {}개: {host:?}",
        host.len(),
    );
    for (h, y, text) in &host {
        assert!(
            (h - 13.6).abs() <= 1.0,
            "호스트 줄 런 {text:?} (y={y:.1}) 의 상자 높이는 글자 줄 baseline 13.6px 이어야              한다 — 실측 {h:.1}px (표 줄 179.0px 이면 그 출력 경로에 보정이 빠진 것)",
        );
    }
}

/// 저장 사다리의 `textpos` 와 `char_offsets` 가 같은 눈금임을 잠근다 (검토 지적 3).
///
/// 이 문단은 본문 11 글자(제목 10 + 후행 공백 1) 뒤에 표 컨트롤이 온다. 표 줄의 `textpos`
/// 는 11 이고 `char_offsets` 의 마지막 글자 위치도 10 이다 — 두 값이 같은 축이라는 뜻이다.
/// `para.char_count`(20)는 컨트롤 슬롯을 포함해 본문 길이와 다르므로 이 비교의 기준이
/// 될 수 없다.
#[test]
fn stored_ladder_axis_matches_char_offsets() {
    let bytes = sample();
    let doc = rhwp::parse_document(&bytes).expect("파싱");
    let para = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .find(|p| p.text.contains("행정박물"))
        .expect("호스트 문단");

    assert_eq!(para.line_segs.len(), 2, "글자 줄과 표 줄 두 줄이어야 한다");
    let text_units = para.text.chars().count();
    assert_eq!(text_units, 11, "본문은 11 글자다 (제목 10 + 후행 공백 1)");
    assert_eq!(
        para.char_offsets.len(),
        text_units,
        "char_offsets 는 글자마다 하나다"
    );
    assert_eq!(
        para.line_seg_text_start(1),
        11,
        "표 줄은 본문 끝에서 시작한다 — 표가 줄을 통째로 가졌다는 기록"
    );
    assert_eq!(
        *para.char_offsets.last().expect("마지막 글자"),
        10,
        "마지막 글자의 축 위치는 10 — 표 줄 시작 11 과 같은 눈금이다"
    );
    assert_ne!(
        u32::try_from(text_units).unwrap_or(0),
        para.char_count,
        "char_count 는 컨트롤 슬롯을 포함해 본문 길이와 다르다 — 판정 기준으로 쓸 수 없다"
    );
}

//! 관계형 — [#9396] 문단 마지막 줄에 앵커된 자리차지 그림이 다음 문단을 민 저장 띠는 host 의 줄과 같은 쪽에 있고, 세로 오프셋은 앵커 줄 자리에서 잰다.
//! 주석 갱신: 2026-11-25 22:38 KST (처음 작성)
//!
//! 입력: `samples/issue9396/fire_inspection_manual_p71-75.hwp` — 10k 공개 코퍼스의 소방청
//! 「소방방재 시스템 점검메뉴얼」 Ch2(HWP 5.0.0.6, 558쪽)에서 `rhwp extract-pages --from 71 --to 75`
//! 로 71~75쪽 문단만 남기고 표지 제목 글 상자 하나를 지운 7쪽 파생본(`samples/issue9396/README.md`).
//! 구역 1 문단 28(원본 1211)은 글 415자 뒤에 그림 control 하나를 둔다. 그 control 은 마지막 줄
//! (ts 415, vpos 18000)의 유일한 글자다. 그림은 글자처럼 취급하지 않는 자리차지, 문단 기준 세로
//! 오프셋 2161, 높이 9900 이다.
//! - 저장 사다리: 문단 28 첫 줄 vpos 0(5쪽에서 다시 시작), 다음 문단 29 vpos 30061 =
//!   앵커 줄 18000 + 오프셋 2161 + 그림 9900. 띠는 문단 28 의 줄과 같은 사다리, 곧 5쪽에 있다.
//! - 수정 전 rhwp 는 저장 리셋 경계에서 그림을 떠나는 4쪽 말미로 흘렸다(#6146 규칙). 그림이
//!   꼬리말 선 밖으로 넘쳤고 5쪽에는 그림이 없었다. 그림 위치도 문단 첫 줄에서 오프셋을 재어
//!   문단 글줄과 겹쳤다.
//! - 반례: #6146 156583583 문단 24(로고 3736, 줄 끝 = 다음 문단 3236)는 띠가 다음 사다리에
//!   없어 떠나는 쪽에 남는다(`issue_6146_page_tail_float_band_spill`).
//!
//! 측정: 2026-11-25 21:53 KST, `scripts/visual_sweep.py` 의 `silhouette_raw_match_percent`
//! (색상 경계 보정 전 2px 내용 실루엣), 96 DPI. 기준은 `pdf/issue9396/fire_inspection_manual_p71-75-2020.pdf`
//! (한/글 2020 Print, 7쪽). 수정 후 rhwp 도 Native·fresh WASM 모두 7쪽.
//! - 4쪽(떠나는 쪽): Native 99.32 / fresh WASM 99.88 (수정 전 Native 85.63)
//! - 5쪽(host 쪽): Native 99.73 / fresh WASM 99.93 (수정 전 Native 85.92)
//! - 전쪽 최저: Native 99.32(2·4쪽) / fresh WASM 99.88(4쪽)
//! - 증거: `mydocs/pr/assets/issue_9396/{native,wasm,base-native}/` 의 TSV·review PNG
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9396/fire_inspection_manual_p71-75.hwp";
const PX_PER_HU: f64 = 96.0 / 7200.0;
const SECTION: usize = 1;
const HOST: usize = 28;
/// 0 기반 쪽 번호: 4쪽(떠나는 쪽)과 5쪽(host 줄이 다시 시작하는 쪽).
const LEAVING_PAGE: u32 = 3;
const HOST_PAGE: u32 = 4;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 열기")
}

#[derive(Default)]
struct PageBoxes {
    body: Option<BoundingBox>,
    lines: Vec<(Option<usize>, BoundingBox)>,
    images: Vec<(Option<usize>, BoundingBox)>,
}

fn collect(node: &RenderNode, out: &mut PageBoxes) {
    match &node.node_type {
        RenderNodeType::Body { .. } => out.body = Some(node.bbox),
        RenderNodeType::Header
        | RenderNodeType::Footer
        | RenderNodeType::MasterPage
        | RenderNodeType::FootnoteArea => return,
        RenderNodeType::TextLine(line) => out.lines.push((line.para_index, node.bbox)),
        RenderNodeType::Image(image) => out.images.push((image.para_index, node.bbox)),
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

fn of(items: &[(Option<usize>, BoundingBox)], para: usize) -> Vec<BoundingBox> {
    items
        .iter()
        .filter(|(p, _)| *p == Some(para))
        .map(|(_, b)| *b)
        .collect()
}

/// 저장 사다리가 이 형상인지 원본 값으로 확인하고 (앵커 줄 vpos − 첫 줄 vpos + 오프셋)을 돌려준다.
fn stored_band_origin_hu(core: &DocumentCore) -> i32 {
    let paragraphs = &core.document().sections[SECTION].paragraphs;
    let host = &paragraphs[HOST];
    let picture = host
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Picture(p) => Some(p.as_ref()),
            _ => None,
        })
        .expect("전제: host 문단에 그림이 있다");
    assert!(
        !picture.common.treat_as_char,
        "전제: 글자처럼 취급하지 않는 그림"
    );
    let positions = host.control_text_positions();
    let last = host.line_segs.last().expect("host 저장 줄");
    assert_eq!(
        positions.first().copied(),
        Some(host.text.chars().count()),
        "전제: 그림 control 은 글 끝에 있다"
    );
    assert_eq!(
        last.text_start as usize,
        host.text.chars().count(),
        "전제: 마지막 줄은 그림 control 만 담는다"
    );
    let first = host.line_segs.first().expect("host 첫 줄");
    let next = paragraphs[HOST + 1]
        .line_segs
        .first()
        .expect("다음 문단 저장 줄");
    let offset = picture.common.vertical_offset as i32;
    let frame = offset
        + picture.common.height as i32
        + i32::from(picture.common.margin.top)
        + i32::from(picture.common.margin.bottom);
    assert_eq!(
        first.vertical_pos, 0,
        "전제: host 는 새 쪽에서 저장 사다리를 다시 시작한다"
    );
    assert_eq!(
        next.vertical_pos - last.vertical_pos,
        frame,
        "전제: 다음 문단 = 앵커 줄 + 그림 띠"
    );
    last.vertical_pos - first.vertical_pos + offset
}

#[test]
fn last_line_anchor_picture_stays_with_host_lines_on_next_page_p5() {
    let core = core();
    stored_band_origin_hu(&core);
    let leaving = page_boxes(&core, LEAVING_PAGE);
    assert!(
        of(&leaving.lines, HOST).is_empty(),
        "전제: host 문단의 줄은 4쪽에 없다"
    );
    assert!(
        of(&leaving.images, HOST).is_empty(),
        "4쪽에 host 문단의 그림이 남아서는 안 된다"
    );
    let leaving_body = leaving.body.expect("4쪽 본문");
    for (para, image) in &leaving.images {
        assert!(
            image.y + image.height <= leaving_body.y + leaving_body.height + 0.5,
            "4쪽 그림(문단 {para:?})은 본문 아래끝 안에 있다"
        );
    }

    let host = page_boxes(&core, HOST_PAGE);
    assert!(
        !of(&host.lines, HOST).is_empty(),
        "host 문단의 줄은 5쪽에 있다"
    );
    assert_eq!(
        of(&host.images, HOST).len(),
        1,
        "host 문단의 그림은 5쪽에 한 번 그려진다"
    );
}

#[test]
fn last_line_anchor_picture_offset_is_measured_from_anchor_line_p5() {
    let core = core();
    let origin_hu = stored_band_origin_hu(&core);
    let boxes = page_boxes(&core, HOST_PAGE);
    let lines = of(&boxes.lines, HOST);
    let first_line = lines.first().expect("host 첫 줄");
    let image = *of(&boxes.images, HOST).first().expect("host 그림");
    let expected = first_line.y + f64::from(origin_hu) * PX_PER_HU;
    assert!(
        (image.y - expected).abs() <= 0.5,
        "그림 윗변 {:.2} 은 host 첫 줄 + (앵커 줄 − 첫 줄 + 오프셋) {expected:.2} 이다",
        image.y
    );
    let text_bottom = lines
        .iter()
        .map(|b| b.y + b.height)
        .fold(f64::MIN, f64::max);
    assert!(
        image.y >= text_bottom - 0.5,
        "그림 윗변 {:.2} 은 host 글줄 아래끝 {text_bottom:.2} 아래다(겹치지 않는다)",
        image.y
    );
    let next = of(&boxes.lines, HOST + 1);
    let next_top = next.first().expect("다음 문단 줄").y;
    assert!(
        image.y + image.height <= next_top + 0.5,
        "그림 아래끝 {:.2} 은 다음 문단 첫 줄 {next_top:.2} 위다",
        image.y + image.height
    );
}

//! [Issue #7047] 글자 없는 떠있는 개체 host 문단이 흐름을 전진시키지 않아, 뒤따르는
//! 개체가 저장 사다리보다 위에 놓이고 제목 표가 글상자 첫 줄과 겹치던 결함의 가드.
//!
//! 제보는 "큰 글꼴 제목 문단 다음 줄의 y 가 제목 줄 높이를 반영하지 않는다" 였지만, 겹치는
//! 두 줄은 같은 흐름의 연속 문단이 아니다 — 하나는 쪽 직속 떠있는 표, 하나는 글상자 안
//! 문단이다. 본문 흐름은 정본과 dy 0.00 으로 일치한다. 실제 원인은 **개체를 매단 빈 문단이
//! 흐름에서 자기 줄을 차지하지 않는 것**이고, 세 관문이 겹쳐 있었다.
//!
//! ① `textless_host_ladder_line_advance` 의 기대값이 `줄높이 + 줄간격` 뿐이었다. 저장
//!    델타에는 **문단 간격**(host 뒤 + 다음 앞)까지 실려 있어서, 줄 높이가 작은 host 만
//!    `델타/기대` 가 1.5 를 넘어 stale 가드에 걸려 판별 불가로 물러났다.
//! ② `#703` 데코레이션(글앞/글뒤) 표 단축은 표만 방출하고 흐름을 0 소비한다. 가시 텍스트가
//!    있는 host 는 보완됐지만 **글자 없는 host** 는 `PageItem` 이 하나도 없어 렌더가 그
//!    문단을 건너뛰고 저장 vpos 보정조차 받지 못했다.
//! ③ 렌더의 두 술어(`para_has_visible_textless_float_shape_item` · `has_overlay_float`)가
//!    Picture/Shape 만 매칭해 **표 host** 는 ②로 항목을 얻어도 사다리 전진을 못 받았다.
//!
//! 셋 중 하나만 고치면 닫히지 않는다 — ③ 을 한쪽 술어에만 넣으면 사다리 질의가 돌지 않아
//! 휴리스틱이 "전진 없음"으로 답해 **오히려 나빠진다**(최대 |dy| 19.6 → 27.6px 실측).
//!
//! 재현체 3쪽 빈 개체 host 일곱 개 전량에서 저장 델타가 정확히 그 합이다.
//!
//! ```text
//!   host      델타 = 줄높이 + 줄간격 + host 뒤간격 + 다음 앞간격
//!   rec#829   1720 =  1100 +  220 +  200 +  200
//!   rec#847   2120 =  1400 +  420 +    0 +  300
//!   rec#947    886 =   450 +  136 +    0 +  300
//!   rec#958    494 =   150 +   44 +    0 +  300
//!   rec#1041  1794 =  1150 +  344 +    0 +  300     (rec#1052 · rec#1092 동형)
//! ```
//!
//! 인과는 돌연변이 검정으로 단독 확정했다 — host 문단에 글자 한 자를 넣어 "빈 host" 분기를
//! 벗어나게 하면 아래 전체가 정확히 그 문단의 저장 델타만큼 내려온다(rec#947 +11.8px =
//! 886 HU, rec#958 +6.6px = 494 HU, 0.01px 일치).
//!
//! 정본(engine 2020 새 PDF)과 3쪽 글자 1,276자를 전량 정합한 최대 |dy| 는
//! **57.9px → 1.79px**로 개선됐다는 과거 관측이다. 이후 문단 기준 overlay 표의
//! 바깥여백 반영으로 제목 표는 독립 Print에 더 가까워졌다. 과거 표 y 핀과
//! "Print보다 반드시 위"라는 부호 조건2건은 현재 정상 출력의 기대값이 아니다.
//! 원본1쪽이 고정 devel에서도90% 미만이므로 기대값을 현재 px로 갱신하지 않고
//! #7955에 보존했다. 현재 전쪽90% 이상 확인 뒤 글상자 사다리의 남은 절대 y 핀도
//! 단계 제목·상자의 소유·순서·비겹침 관계로 교체했다. 하단 로고 관계도 유지한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "tests/fixtures/issue_7047/housing-lease-standard-form.hwp";

fn page3() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).expect("재현체 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(2)
        .expect("3쪽 render tree")
        .root
}

/// 원본 안내 상자는 둥근 외곽선이다. 화면의 고정 좌표·크기로 영역을 고르지 않는다.
fn is_wide_textbox(node: &RenderNode) -> bool {
    matches!(&node.node_type, RenderNodeType::Rectangle(shape) if shape.corner_radius > 0.0)
}

fn compact_text(node: &RenderNode) -> String {
    let mut text = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        text.push_str(&compact_text(child));
    }
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 독립 Print의 세 계약 단계 제목으로 소유 표를 식별한다.
fn is_title_table(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Table(_))
        && [
            "계약체결시꼭확인하세요",
            "계약기간중꼭확인하세요",
            "계약종료시꼭확인하세요",
        ]
        .iter()
        .any(|title| compact_text(node).contains(title))
}

fn bands(node: &RenderNode, pick: fn(&RenderNode) -> bool, out: &mut Vec<(f64, f64)>) {
    if pick(node) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        bands(child, pick, out);
    }
}

fn sorted_bands(root: &RenderNode, pick: fn(&RenderNode) -> bool) -> Vec<(f64, f64)> {
    let mut result = Vec::new();
    bands(root, pick, &mut result);
    result.sort_by(|a, b| a.0.total_cmp(&b.0));
    result
}

/// 이 부분트리 안 `TextLine` 의 (상단, 하단).
fn text_line_bands(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        text_line_bands(child, out);
    }
}

/// 전쪽 Native/fresh WASM 최저95% 이상, 이 쪽은 양쪽98% 미만이다.
/// 과거 y 핀을 유지하지 않고 세 단계의 소유·순서와 다음 제목 전 점유 종료를 보호한다.
#[test]
fn contract_stage_boxes_keep_owned_titles_and_disjoint_flow() {
    let root = page3();
    let boxes = sorted_bands(&root, is_wide_textbox);
    let titles = sorted_bands(&root, is_title_table);
    assert_eq!(
        boxes.len(),
        3,
        "계약 체결·기간 중·종료의 안내 상자: {boxes:?}"
    );
    assert_eq!(titles.len(), 3, "각 단계에 하나씩 있는 제목: {titles:?}");
    for (index, ((top, bottom), (title_top, title_bottom))) in boxes.iter().zip(&titles).enumerate()
    {
        assert!(
            title_top < top && top < bottom,
            "제목과 안내 상자의 순서: {index}"
        );
        assert!(
            title_bottom < bottom,
            "제목만 남기고 본문 점유를 잃으면 안 된다: {index}"
        );
        if let Some(next_title) = titles.get(index + 1) {
            assert!(
                *bottom < next_title.0,
                "안내 상자가 다음 단계 제목을 침범함: {index}"
            );
        }
    }
}

/// 제보된 겹침이 사라져야 한다 — 쪽 하단 제목 표의 글줄이 그 아래 글상자 첫 줄을 침범하지
/// 않는다. 수정 전에는 제목 줄 바닥 828.7 이 글상자 첫 줄 상단 821.0 을 7.7px 파고들었다.
#[test]
fn the_title_table_line_no_longer_overlaps_the_textbox_first_line() {
    let root = page3();

    let mut title: Option<(f64, f64)> = None;
    fn scan_titles(node: &RenderNode, out: &mut Option<(f64, f64)>) {
        if is_title_table(node) {
            let mut bands = Vec::new();
            text_line_bands(node, &mut bands);
            if let Some(band) = bands
                .into_iter()
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            {
                if out.map(|cur| band.0 > cur.0).unwrap_or(true) {
                    *out = Some(band);
                }
            }
        }
        for child in &node.children {
            scan_titles(child, out);
        }
    }
    scan_titles(&root, &mut title);
    let (title_top, title_bottom) = title.expect("제목 표 글줄");

    let mut first_below: Option<f64> = None;
    fn scan_boxes(node: &RenderNode, after: f64, out: &mut Option<f64>) {
        if is_wide_textbox(node) {
            let mut bands = Vec::new();
            text_line_bands(node, &mut bands);
            for (top, _) in bands {
                if top > after && out.map(|cur| top < cur).unwrap_or(true) {
                    *out = Some(top);
                }
            }
        }
        for child in &node.children {
            scan_boxes(child, after, out);
        }
    }
    scan_boxes(&root, title_top, &mut first_below);
    let box_line_top = first_below.expect("제목 아래 글상자 첫 글줄");

    assert!(
        box_line_top > title_bottom,
        "제목 줄[{title_top:.1}..{title_bottom:.1}] 이 글상자 첫 줄 {box_line_top:.1} 을 \
         침범한다 (겹침 {:.1}px)",
        title_bottom - box_line_top
    );
}

/// 한컴 engine 2020 기준 PDF의 하단 이미지 3개는 모두 y=784.901pt에서 시작한다.
/// TopAndBottom 그림 host 뒤의 빈 문단도 저장 사다리에 참여해야 한다. 원 통합
/// 후보는 pi=81의 줄을 전진하지 않아 pi=83 법무부 로고만 17.9px 위에 놓였고,
/// 보정된 글상자의 아래 테두리와 겹쳤다. 절대 좌표 보정 대신 공통 상단을 검증한다.
#[test]
fn footer_logos_share_the_hancom_top_and_clear_the_textbox() {
    fn images(node: &RenderNode, out: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::Image(_)) {
            out.push(node.bbox.y);
        }
        for child in &node.children {
            images(child, out);
        }
    }
    fn last_box_bottom(node: &RenderNode) -> f64 {
        let own = if is_wide_textbox(node) {
            node.bbox.y + node.bbox.height
        } else {
            0.0
        };
        node.children
            .iter()
            .map(last_box_bottom)
            .fold(own, f64::max)
    }
    let root = page3();
    let mut ys = Vec::new();
    images(&root, &mut ys);
    assert_eq!(ys.len(), 3, "하단의 기관 로고 세 개: {ys:?}");
    let min = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let max = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // 한컴 이미지 rect는 같은 상단이다. 허용치는 SVG 좌표 소수 첫째 자리 반올림뿐이다.
    assert!(max - min < 0.1, "로고 상단이 서로 갈렸다: {ys:?}");
    assert!(
        min > last_box_bottom(&root),
        "기관 로고가 글상자 테두리 위에 겹쳤다: {ys:?}"
    );
}

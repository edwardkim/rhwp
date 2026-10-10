//! [#7413] 칸 안 여백 축소는 **줄바꿈으로 해결되지 않을 때만** 발화한다.
//!
//! # 무엇이 깨져 있었나
//!
//! `shrunk_cell_horizontal_padding` 은 칸 문단의 **자연 폭**(줄바꿈하지 않은 한 줄 폭)이
//! 가용 너비의 1.15배를 넘으면 선언된 칸 안 여백을 양쪽 1.0px 까지 깎았다. 그 조건은
//! "이 문단은 줄바꿈이 필요하다"는 뜻일 뿐이라, **줄바꿈하면 칸에 들어가는 문단까지 전부**
//! 걸렸다.
//!
//! 축소가 실제로 지탱하는 계약은 "칸 안 글자가 칸 밖으로 새지 않는다" 하나다. 축소를
//! 통째로 끄고 `samples/**` 전수를 재면 겹침이 4건 늘고, 그 넷이 전부 **줄바꿈 결과가
//! 행 높이를 넘는** 자리였다.
//!
//! 기존 영월군 검사는 `월별누계`를 한 줄로 요구했다. 원본 HWP3의 해당 줄은
//! 시작·좌표·구분 정보가 모두 비어 있는 글자 높이 레코드라 저장 줄 구성의 근거가 아니다.
//! 독립 한컴 2020 Print와 같은 원본의 한컴 HWPX 변환본은 `월별` / `누계` 두 줄이다.
//! 마지막 줄 뒤 간격을 제외한 두 줄은 원본 셀 높이에 들어간다.
//!
//! 그래서 묻는 것을 바꿨다 — **줄바꿈한 결과가 칸 높이를 넘는가.** 넘지 않으면 줄바꿈이
//! 답이고 선언 여백을 그대로 둔다.
//!
//! # 현재 검사 범위
//!
//! 이 파일은 `table-text.hwpx`의 선언 여백 검사만 유지한다.
//! 영월 서식의 고정 위치·한 줄 기대값은 독립 한컴 2020 Print와 상충한다.
//! Native/fresh WASM 전 4쪽 최저 99.29% 확인 후 새 파일
//! `issue_7642_yeongwol_monthly_cell_relations.rs`에서 원본 소유·월별/누계 두 줄·
//! 순서·비겹침·셀 내부 포함을 검사한다. 다른 입력의 검사는 일괄 제거하지 않는다.
//!
//! `table-text.hwpx`의 기존 선언 여백 검사는 별도의 Print 출처 확인 대상이다.
//! 영월 검증 결과를 이 입력의 시각 근거로 사용하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 줄바꿈이 칸 높이에 들어가는 칸 — 선언 여백이 남아야 한다.
const FITS: &str = "samples/hwpx/table-text.hwpx";

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// `(칸 bbox, 글자 왼끝, 글자 오른끝)` 를 칸 단위로 모은다.
fn cell_text_extents(
    node: &RenderNode,
    cell: Option<&RenderNode>,
    out: &mut Vec<(f64, f64, f64, f64)>,
) {
    let next_cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if let (RenderNodeType::TextRun(_), Some(c)) = (&node.node_type, next_cell) {
        out.push((
            c.bbox.x,
            c.bbox.width,
            node.bbox.x,
            node.bbox.x + node.bbox.width,
        ));
    }
    for child in &node.children {
        cell_text_extents(child, next_cell, out);
    }
}

/// 줄바꿈으로 해결되는 칸은 선언 안 여백을 지킨다.
#[test]
fn a_cell_that_wraps_within_its_height_keeps_its_declared_padding() {
    let core = load(FITS);
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut rows = Vec::new();
    cell_text_extents(&page.root, None, &mut rows);

    // 대상 칸: 숫자 한 줄이 들어가는 폭 76.4px 칸.
    let target: Vec<_> = rows
        .iter()
        .filter(|(cx, cw, ..)| (*cx - 115.30).abs() < 0.2 && (*cw - 76.40).abs() < 0.2)
        .collect();
    assert!(
        !target.is_empty(),
        "정답지 전제가 깨졌다 — 대상 칸(x=115.30, w=76.40)을 못 찾았다. 쪽 구성이 바뀌었으면          이 검사의 좌표부터 다시 정해야 한다."
    );

    let left = target
        .iter()
        .map(|(_, _, x0, _)| *x0)
        .fold(f64::MAX, f64::min);
    let right = target
        .iter()
        .map(|(_, _, _, x1)| *x1)
        .fold(f64::MIN, f64::max);
    let (cx, cw) = (target[0].0, target[0].1);
    let pad_left = left - cx;
    let pad_right = cx + cw - right;

    assert!(
        pad_left > 3.0 && pad_right > 3.0,
        "줄바꿈이 칸 높이에 들어가는데도 안 여백이 깎였다 — 축소 하한(1.0px)까지 내려가면          여기서 1.0 근처가 나온다. 좌 {pad_left:.2}px / 우 {pad_right:.2}px"
    );
}

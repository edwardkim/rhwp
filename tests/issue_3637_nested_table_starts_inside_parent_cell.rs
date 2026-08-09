//! Issue #3637: 셀 안 중첩 표가 부모 셀 밖에서 시작해 쪽을 벗어난다.
//!
//! ## 근인
//!
//! `table_layout.rs` 는 셀 안 중첩 표의 시작 y 를 `para_y`(그 셀에서 앞 텍스트가 끝난
//! 자리)로 잡는다. 그 텍스트가 이미 셀 밖으로 밀려 있으면 **중첩 표 컨테이너가 통째로
//! 셀 아래에 놓여** 쪽 밖으로 나간다. 셀 바닥으로 상한을 두어 막는다.
//!
//! [`issue_3637_split_cell_nested_table_vpos`] 가 거는 상한은 `table_partial.rs` 의 셀
//! 경로에만 있다. 그 경로를 지나는 중첩1 은 이미 셀 안에 있고, 탈출하는 것은 **중첩1 의
//! 셀 안에 있는 중첩2** 라 일반 표 경로(`table_layout.rs`)를 지난다. 그래서 형제 테스트로는
//! 이 변경의 회귀를 잡지 못한다.
//!
//! ## 계약
//!
//! 컨테이너가 쪽 아래로 흘러내린 **깊이**를 계약한다. 클램프가 풀리면 중첩 표가 통째로
//! 셀 밑으로 내려가므로 이 값이 곧바로 되돌아간다.
//!
//! ## 임계값 근거
//!
//! 표본을 수정 전후로 재서 그 사이에 둔다(쪽 높이 1,122.5px).
//!
//! ```text
//!                              수정 전    수정 후
//!   문서 최하단 렌더 y          2,416.1    2,073.6
//!   쪽 아래로 넘어간 깊이        1,293.6      951.1
//!   시작 y 가 셀 밖인 중첩 표       46건       42건
//! ```
//!
//! 수정 후에도 42건이 남는 것은 **다른 축**이다 — 중첩 표가 부모 셀보다 큰 형상으로,
//! 쪼개려면 쪽보다 큰 행의 분할 정책을 넓혀야 하는데 그 축은 3회 반증됐다(이슈 #3637 ·
//! #3932 코멘트). 0 을 요구하면 이 테스트가 그 축의 결함까지 지게 되므로 두 상태 사이의
//! 값으로 계약한다. 같은 이유로 "최대 이탈"(2,717.9px, 전후 불변)은 계약에 넣지 않는다 —
//! 그 값은 이 수정이 건드리는 양이 아니다.
//!
//! 건수(46→42)도 계약하지 않는다. 밴드가 ±2건뿐이라 폰트 환경에 따라 뒤집힐 수 있다
//! (#3458 에서 박은 좌표가 CI 전용 실패를 냈다). 깊이는 342.5px 밴드라 그 위험이 없다.
//! 대신 실패 메시지에 건수를 함께 실어 진단에 쓴다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 농림축산식품부 규제영향분석서. 표 안에 표가 2단으로 겹쳐 있고, 그 안쪽 표가 부모 셀
/// 아래로 나간다.
///
/// 국민참여입법센터 입법예고 공개 자료다.
const SAMPLE: &str = "samples/issue3637/regulatory_impact_nested_table_escape.hwpx";

/// 쪽 아래로 넘어가도 되는 깊이의 상한 — 수정 전 1,293.6 · 수정 후 951.1 의 사이.
///
/// 쪽 높이에 상대적인 값이라 용지 크기가 달라져도 뜻이 유지된다.
const MAX_OVERFLOW_BELOW_PAGE_PX: f64 = 1_100.0;

/// 부모 셀 경계를 벗어난 것으로 셀 때 무시할 오차.
const TOLERANCE_PX: f64 = 0.5;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 시작 y 가 부모 셀 밖인 중첩 표를 모은다 — `(시작 y, 부모 셀 상단, 부모 셀 하단)`.
fn nested_tables_starting_outside_cell(
    node: &RenderNode,
    cell: Option<&RenderNode>,
    found: &mut Vec<(f64, f64, f64)>,
) {
    if let (RenderNodeType::Table(_), Some(parent)) = (&node.node_type, cell) {
        let top = parent.bbox.y;
        let bottom = parent.bbox.y + parent.bbox.height;
        let start = node.bbox.y;
        if start > bottom + TOLERANCE_PX || start < top - TOLERANCE_PX {
            found.push((start, top, bottom));
        }
    }
    let inner = match node.node_type {
        RenderNodeType::TableCell(_) => Some(node),
        _ => cell,
    };
    for child in &node.children {
        nested_tables_starting_outside_cell(child, inner, found);
    }
}

/// 그 노드 아래에서 가장 깊이 내려간 y.
fn deepest_bottom(node: &RenderNode) -> f64 {
    let own = node.bbox.y + node.bbox.height;
    node.children
        .iter()
        .map(deepest_bottom)
        .fold(own, |a, b| if b > a { b } else { a })
}

/// 실제 Cell clip 안에 온전히 남은 text만 누적한다. RenderTree에는 clip 바깥으로
/// 배치된 다음 조각의 노드도 진단용으로 남을 수 있으므로, raw node text만 보면
/// p26 하단에서 잘린 p27 source owner를 오탐한다.
fn fully_visible_text(node: &RenderNode, clip_top: f64, clip_bottom: f64, out: &mut String) {
    let (next_top, next_bottom) = if matches!(&node.node_type, RenderNodeType::TableCell(_)) {
        (
            clip_top.max(node.bbox.y),
            clip_bottom.min(node.bbox.y + node.bbox.height),
        )
    } else {
        (clip_top, clip_bottom)
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let bottom = node.bbox.y + node.bbox.height;
        if node.bbox.y >= next_top - TOLERANCE_PX && bottom <= next_bottom + TOLERANCE_PX {
            out.push_str(&run.text);
        }
    }
    for child in &node.children {
        fully_visible_text(child, next_top, next_bottom, out);
    }
}

/// page 밖에 시작한 셀 TextLine은 SVG clip으로 숨겨도 사용자에게 보이지 않는다.
/// 다음 페이지 소유 줄을 현재 page의 RenderTree에 생성하지 않는지를 직접 고정한다.
fn cell_lines_starting_below_page(
    node: &RenderNode,
    inside_cell: bool,
    page_bottom: f64,
    found: &mut Vec<f64>,
) {
    let inside_cell = inside_cell || matches!(&node.node_type, RenderNodeType::TableCell(_));
    if inside_cell
        && matches!(&node.node_type, RenderNodeType::TextLine(_))
        && node.bbox.y >= page_bottom - TOLERANCE_PX
    {
        found.push(node.bbox.y);
    }
    for child in &node.children {
        cell_lines_starting_below_page(child, inside_cell, page_bottom, found);
    }
}

/// 중첩 표가 부모 셀 안에서 시작해, 쪽 아래로 컨테이너째 흘러내리지 않는다.
#[test]
fn nested_table_starts_inside_its_parent_cell() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    let page_count = doc.page_count();
    // HWP 2020 PrintToPDFEx fresh oracle (2026-08-06) is 31 pages.  This also
    // prevents the terminal run of empty paragraphs after the last nested table
    // from materializing as a 32nd blank page.
    assert_eq!(
        page_count, 31,
        "HWP 2020 기준 31쪽과 달라졌다 — 중첩 표 조각 또는 문서 말미 빈 문단의 쪽 소유를 확인하라"
    );

    // HWP 2020 PDF p26의 마지막 source line은 "시간당 근로임금…"이고,
    // p27은 바로 다음 "사업체노동력조사…"로 시작한다. p26의 painted tail을
    // 좁히기만 하면 p27에서 첫 줄이 중복되고, pagination만 앞당기면 p27의
    // 첫 줄이 사라진다. 실제 Cell clip 안의 소유를 양쪽에서 함께 고정한다.
    let mut p26_text = String::new();
    let p26 = doc
        .build_page_render_tree(25)
        .expect("HWP 2020 p26 render tree");
    fully_visible_text(&p26.root, f64::NEG_INFINITY, f64::INFINITY, &mut p26_text);
    assert!(
        p26_text.contains("시간당 근로임금은"),
        "p26은 HWP 2020이 소유한 마지막 임금 기준 줄을 보여야 한다"
    );
    assert!(
        !p26_text.contains("사업체노동력조사"),
        "p26에 p27 source owner가 가시 상태로 남았다"
    );

    let mut p27_text = String::new();
    let p27 = doc
        .build_page_render_tree(26)
        .expect("HWP 2020 p27 render tree");
    fully_visible_text(&p27.root, f64::NEG_INFINITY, f64::INFINITY, &mut p27_text);
    assert!(
        p27_text.contains("사업체노동력조사"),
        "p27은 HWP 2020이 소유한 다음 사업체 조사 줄부터 재개해야 한다"
    );
    assert!(
        !p27_text.contains("시간당 근로임금은"),
        "p27에 p26의 마지막 source line이 중복됐다"
    );

    // p28 하단의 12×3 손자 표는 이 쪽에 들어오는 두 행만 그리고, 나머지는 p29
    // continuation이 소유한다. 전체 표를 먼저 만든 뒤 조상 Cell clip으로 숨기면 SVG는
    // 겉보기에는 맞아도 쪽 하단 밖 TextLine이 남아 overflow-cell gate를 우회한다.
    let p28 = doc
        .build_page_render_tree(27)
        .expect("HWP 2020 p28 render tree");
    let mut p28_hidden_cell_lines = Vec::new();
    let p28_bottom = p28.root.bbox.y + p28.root.bbox.height;
    cell_lines_starting_below_page(&p28.root, false, p28_bottom, &mut p28_hidden_cell_lines);
    assert!(
        p28_hidden_cell_lines.is_empty(),
        "p28 Cell clip 아래에 다음 쪽 소유 TextLine이 {}개 남았다: {p28_hidden_cell_lines:?}",
        p28_hidden_cell_lines.len(),
    );

    let mut escapes = Vec::new();
    let mut deepest_overflow = 0.0_f64;
    let mut deepest_page = 0;
    let mut deepest_y = 0.0_f64;
    let mut page_height = 0.0_f64;
    for page in 0..page_count {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
        let mut page_escapes = Vec::new();
        nested_tables_starting_outside_cell(&tree.root, None, &mut page_escapes);
        for (start, top, bottom) in page_escapes {
            escapes.push(format!(
                "{}쪽 중첩 표 시작 y={start:.1} 부모 셀=[{top:.1}..{bottom:.1}]",
                page + 1
            ));
        }
        let height = tree.root.bbox.height;
        let bottom = deepest_bottom(&tree.root);
        let overflow = bottom - height;
        if overflow > deepest_overflow {
            deepest_overflow = overflow;
            deepest_page = page + 1;
            deepest_y = bottom;
            page_height = height;
        }
    }

    assert!(
        deepest_overflow <= MAX_OVERFLOW_BELOW_PAGE_PX,
        "렌더 트리가 쪽 아래로 {deepest_overflow:.1}px 넘어갔다(상한 \
         {MAX_OVERFLOW_BELOW_PAGE_PX}, {deepest_page}쪽 y={deepest_y:.1} / 쪽 높이 \
         {page_height:.1}). 셀 안 중첩 표의 시작 y 를 부모 셀 바닥으로 상한하지 않으면 \
         컨테이너가 통째로 셀 아래에 놓여 흘러내린다(수정 전 1,293.6px).\n\
         참고 — 시작 y 가 부모 셀 밖인 중첩 표 {}건(수정 전 46 · 수정 후 42, 이 값은 \
         계약이 아니다):\n  {}",
        escapes.len(),
        escapes
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

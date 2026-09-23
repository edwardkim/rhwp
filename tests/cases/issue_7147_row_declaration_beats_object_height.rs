//! [#7147] 행 높이의 권위는 표 개체 높이가 아니라 **행 선언**이다.
//!
//! # 무엇이 깨져 있었나
//!
//! `fit_measured_table_to_declared_height` 는 측정 행 높이를 표 개체 높이
//! (`common.height`)에 맞춰 **모든 행을 비례 축소**했다. 이 보정이 화해시키려는 것은
//! "측정이 저장 선언에서 근소하게 어긋났다"(#1510)인데, 이 문서는 어긋나지 않았다 —
//! 측정 행 높이가 문서의 **행별 선언 높이와 행 단위로 이미 일치**한다.
//!
//! ```text
//!   측정 행합  453.1px = 행 선언 합 453.1px   (20행 전부 일치)
//!   개체 common.height     366.6px           → 0.8092 배 일괄 축소
//! ```
//!
//! 그래서 맞던 행 높이가 19% 작아졌고, 줄어든 표가 한 쪽에 "들어간다"고 판정돼
//! 쪽 경계 분할도 일어나지 않았다.
//!
//! # 기대값의 출처 — 한/글 정본의 벡터 괘선
//!
//! `pdf/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사-2022.pdf`
//! 94쪽(표시 83쪽) `<표 Ⅵ-5>` 의 제목부 괘선 간격을 재면
//!
//! | 행 | 원본 선언 | 한/글 정본 | 수정 전 rhwp |
//! |---|---:|---:|---:|
//! | IR row=0 | 6.004mm | **6.005mm** | 4.859mm |
//! | IR row=1 | 13.391mm | **13.405mm** | 10.836mm |
//!
//! 정본은 행 선언을 지키고 그 표를 **94→95쪽으로 나눈다**. 개체 높이 366.6px 는
//! 표 전체가 아니라 **첫 조각**의 높이다.
//!
//! # 검사하는 것
//!
//! 「축소하지 않는다」를 helper 단위로 묻지 않고, 그 쪽에 **실제로 그려진 셀 상자**의
//! 높이를 정본 실측값과 대조한다. 대조군으로 표가 쪽 경계에서 나뉘는지도 함께 본다 —
//! 축소가 되살아나면 표가 한 쪽에 들어가 이 검사가 같이 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp";
/// 정본 94쪽에 대응하는 0-based 쪽.
const PAGE: u32 = 94;
/// `<표 Ⅵ-5>` 를 소유한 문단.
const TABLE_PARA: usize = 934;
const DPI: f64 = 96.0;

/// 한/글 정본 94쪽 괘선 실측(mm). 원본 선언은 6.004 / 13.391mm 다.
const ORACLE_ROW0_MM: f64 = 6.005;
const ORACLE_ROW1_MM: f64 = 13.405;
/// 원본 선언과 정본 실측의 차(0.014mm)보다 넉넉하되, 종전 결함(1.1mm·2.6mm)은
/// 확실히 걸러내는 폭.
const TOLERANCE_MM: f64 = 0.15;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 쪽 `page` 에 그려진 표 `pi` 의 `rowspan == 1` 셀 상자 높이를 행 번호별로 모은다.
fn row_box_heights_mm(doc: &DocumentCore, page: u32) -> std::collections::BTreeMap<u16, f64> {
    fn walk(
        node: &RenderNode,
        in_target: bool,
        out: &mut std::collections::BTreeMap<u16, f64>,
    ) {
        let mut inside = in_target;
        match &node.node_type {
            RenderNodeType::Table(t) if t.para_index == Some(TABLE_PARA) => inside = true,
            RenderNodeType::TableCell(c) if in_target && c.row_span == 1 => {
                out.entry(c.row)
                    .or_insert(node.bbox.height / DPI * 25.4);
            }
            _ => {}
        }
        for child in &node.children {
            walk(child, inside, out);
        }
    }
    let mut out = std::collections::BTreeMap::new();
    if let Ok(tree) = doc.build_page_render_tree(page) {
        walk(&tree.root, false, &mut out);
    }
    out
}

/// 제목부 두 행의 실제 상자 높이가 한/글 정본과 같다.
#[test]
fn header_row_boxes_match_the_hancom_oracle() {
    let doc = load();
    let rows = row_box_heights_mm(&doc, PAGE);
    assert!(
        !rows.is_empty(),
        "쪽 {PAGE} 에서 표 pi={TABLE_PARA} 의 셀을 찾지 못했다 — 시험 설정 오류"
    );
    for (row, oracle) in [(0u16, ORACLE_ROW0_MM), (1u16, ORACLE_ROW1_MM)] {
        let got = *rows
            .get(&row)
            .unwrap_or_else(|| panic!("행 {row} 의 셀 상자를 찾지 못했다. 잡힌 행: {:?}", rows.keys()));
        assert!(
            (got - oracle).abs() <= TOLERANCE_MM,
            "행 {row} 상자 높이가 정본과 다르다 — 개체 높이에 맞춘 비례 축소가 살아 있으면 \
             약 0.809 배로 작아진다. 실제={got:.3}mm 정본={oracle:.3}mm (허용 {TOLERANCE_MM}mm)"
        );
    }
}

/// 축소하지 않은 표는 한 쪽에 안 들어가므로 쪽 경계에서 나뉜다 — 정본과 같다.
#[test]
fn the_unshrunk_table_splits_across_the_page_boundary() {
    let doc = load();
    let here = row_box_heights_mm(&doc, PAGE);
    let next = row_box_heights_mm(&doc, PAGE + 1);
    assert!(
        !here.is_empty() && !next.is_empty(),
        "정본은 이 표를 {}쪽과 {}쪽에 나눠 그린다(0-based). 한쪽이 비었다면 축소로 표가 \
         한 쪽에 들어간 것이다. {}쪽 행수={} {}쪽 행수={}",
        PAGE,
        PAGE + 1,
        PAGE,
        here.len(),
        PAGE + 1,
        next.len()
    );
}

/// 반례 — 개체 높이가 **행 중간**에서 끊기면 조각 경계일 수 없으므로 종전대로 화해한다.
///
/// `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` 쪽 53 의 3행 표는
/// 측정·행 선언이 둘 다 144.7px 로 같지만 개체 높이 130.2px 는 누적 행 경계
/// `[47.0, 99.4, 144.7]` 어디에도 맞지 않는다. 한/글 정본(전체 205쪽 PDF 55쪽)의
/// 가로 괘선은 899 / 946 / 998 / 1029px 로 표 높이가 **130px** — 개체 높이 쪽이다.
/// 이 갈래까지 넓히면 그 표가 14.5px 자라 본문 하단을 9.4px 넘는다.
#[test]
fn object_height_that_ends_mid_row_still_reconciles() {
    const COUNTER: &str = "samples/issue5941/1480000-201900698-native-neartop-reset.hwp";
    const COUNTER_PAGE: u32 = 53;
    const COUNTER_PARA: usize = 303;
    /// 정본 괘선 899..1029px.
    const ORACLE_TABLE_PX: f64 = 130.0;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(COUNTER);
    let bytes = std::fs::read(&path).expect("반례 재현물 읽기");
    let doc = DocumentCore::from_bytes(&bytes).expect("반례 문서 로드");
    let tree = doc
        .build_page_render_tree(COUNTER_PAGE)
        .expect("반례 쪽 렌더 트리");

    fn find(node: &RenderNode, out: &mut Option<(f64, f64)>) {
        if let RenderNodeType::Table(t) = &node.node_type {
            if t.para_index == Some(COUNTER_PARA) {
                *out = Some((node.bbox.y, node.bbox.height));
            }
        }
        for child in &node.children {
            find(child, out);
        }
    }
    let mut found = None;
    find(&tree.root, &mut found);
    let (top, height) = found.expect("반례 표를 찾지 못했다 — 시험 설정 오류");

    assert!(
        (height - ORACLE_TABLE_PX).abs() <= 1.5,
        "개체 높이가 행 중간에서 끊기는 표까지 화해를 건너뛰면 안 된다 — 정본은 이 표를 \
         {ORACLE_TABLE_PX}px 로 그린다. 실제={height:.1}px (상단 {top:.1})"
    );
}

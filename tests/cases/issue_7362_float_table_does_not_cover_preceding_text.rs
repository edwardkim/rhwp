//! [#7362] 자리차지 표의 최종 원점과 조각 바닥을 지키는 좌표 회귀 가드.
//!
//! 원 PR #7366의 검사다. #7437 병합 devel에서는 원 정책 변경 없이도 두 검사가
//! 통과한다. 현재 신고 결함은 실제 painted RowBreak 경계와 행 소유를 보정한
//! #7437에서 해결했으며, 아래는 당시 결함의 설명과 독립 좌표 계약을 보존한다.
//! 선언·실측 중 큰 값을 모든 표에 선택해야 한다는 일반 규칙의 증거는 아니다.
//!
//! # 무엇이 깨져 있었나
//!
//! 표 개체의 **선언** 높이가 실제 조판된 높이보다 148px 작았다(809.3px vs 957.3px).
//! 통째 배치 판정이 작은 쪽인 선언값을 썼기 때문에
//!
//! ```text
//!   흐름 113.5 + 선언 809.3 = 922.8  ≤ 본문 926.0   → "들어간다"
//!   흐름 113.5 + 실측 957.3 = 1070.8 >  본문 926.0   ← 실제로는 144.9px 넘친다
//! ```
//!
//! 가 되어 표가 쪽을 넘긴 채 통째로 들어갔다. 그러면 페인트의 세로 클램프 상한
//! `본문상단 + 본문높이 − 표높이` 가 하한(본문 상단)보다 **위**로 내려가
//! `clamp(94.5, 94.5)` 로 퇴화하고, 표가 제 흐름 위치 208.0 에서 단 상단 94.5 로
//! 끌려와 이미 그려진 앞 문단(94.5..201.0)을 덮었다.
//!
//! # 기대값의 출처 — 한/글 정본
//!
//! `pdf/issue2006/1790387_prep_final_report-hwp2020-20260814.pdf` 는 140쪽으로
//! rhwp 140쪽과 짝이 맞는다. 그 69쪽(0-based 68)에서 이 표의 행 상단은
//!
//! ```text
//!   207.3  232.7  256.7  282.0  306.0  331.3 … 972.7  996.7   (px, 96DPI 환산)
//! ```
//!
//! 으로 **행 피치 ≈ 24.66px** — rhwp 가 실제로 배치한 피치와 같고, 선언 높이가
//! 함의하는 피치(809.3/38 ≈ 21.3px)가 아니다. 정본은 같은 표를 **69→70쪽으로
//! 나눈다**(70쪽이 100.7 / 110.0 / 120.7 … 로 이어받는다). 곧 실측 쪽이 옳고
//! 쪽 경계 분할이 정답이다.
//!
//! # 검사하는 것
//!
//! 「겹치지 않는다」가 아니라 **최종 좌표**를 본다 — 표 상단이 같은 쪽에 이미
//! 놓인 본문 줄의 아래끝 이상이어야 하고, 표 상자가 본문 영역을 넘지 않아야 한다.
//! 쪽수 핀(`issue_2006_1790387_prep_pagination_pin`)이 140쪽을 따로 잠근다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue2006/1790387_prep_final_report.hwpx";
/// 신고된 쪽 (0-based). 정본 69쪽에 대응한다.
const PAGE: u32 = 68;
/// 표를 소유한 문단.
const TABLE_PARA: usize = 27;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// (본문 줄 아래끝 최대, 표 상단, 표 아래끝, 본문 영역 상·하단)
struct PageGeometry {
    last_text_bottom: f64,
    table_top: f64,
    table_bottom: f64,
    body_top: f64,
    body_bottom: f64,
}

fn collect(node: &RenderNode, in_table: bool, geo: &mut PageGeometry) {
    let mut inside_table = in_table;
    match &node.node_type {
        RenderNodeType::Body { .. } => {
            geo.body_top = node.bbox.y;
            geo.body_bottom = node.bbox.y + node.bbox.height;
        }
        RenderNodeType::Table(t) if t.para_index == Some(TABLE_PARA) => {
            geo.table_top = node.bbox.y;
            geo.table_bottom = node.bbox.y + node.bbox.height;
            inside_table = true;
        }
        // 표 **바깥**의 본문 줄만 센다 — 칸 안 줄은 표를 따라 움직인다.
        RenderNodeType::TextLine(line) if !in_table => {
            if line.para_index.is_some_and(|pi| pi < TABLE_PARA) {
                geo.last_text_bottom = geo.last_text_bottom.max(node.bbox.y + node.bbox.height);
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, inside_table, geo);
    }
}

fn page_geometry() -> PageGeometry {
    let doc = load();
    let tree = doc.build_page_render_tree(PAGE).expect("쪽 렌더 트리");
    let mut geo = PageGeometry {
        last_text_bottom: f64::MIN,
        table_top: f64::MIN,
        table_bottom: f64::MIN,
        body_top: 0.0,
        body_bottom: 0.0,
    };
    collect(&tree.root, false, &mut geo);
    assert!(
        geo.table_top > f64::MIN && geo.last_text_bottom > f64::MIN,
        "쪽 {PAGE} 에서 표 pi={TABLE_PARA} 또는 앞 본문 줄을 찾지 못했다 — 시험 설정 오류"
    );
    geo
}

/// 자리차지 표는 같은 쪽에 이미 놓인 본문 줄 **아래**에서 시작한다.
#[test]
fn float_table_starts_below_the_text_already_on_the_page() {
    let geo = page_geometry();
    assert!(
        geo.table_top + 0.5 >= geo.last_text_bottom,
        "자리차지 표가 앞 본문 글줄을 덮는다 — 적합 판정이 선언 높이를 써서 쪽을 넘긴 표를 \
         통째로 받으면 페인트 클램프가 표를 단 상단으로 끌어올린다. \
         표 상단={:.1} 앞 본문 아래끝={:.1} (겹침 {:.1}px)",
        geo.table_top,
        geo.last_text_bottom,
        geo.last_text_bottom - geo.table_top
    );
}

/// 그 쪽에 놓인 표 조각은 본문 영역 안에서 끝난다 — 넘칠 양을 받지 않는다.
#[test]
fn placed_fragment_stays_inside_the_body_area() {
    let geo = page_geometry();
    assert!(
        geo.table_bottom <= geo.body_bottom + 0.5,
        "표 조각이 본문 하단을 넘겼다 — 선언 높이로 적합을 판정하면 실측 초과분을 \
         아무도 책임지지 않는다. 표 {:.1}..{:.1} 본문 {:.1}..{:.1} (초과 {:.1}px)",
        geo.table_top,
        geo.table_bottom,
        geo.body_top,
        geo.body_bottom,
        geo.table_bottom - geo.body_bottom
    );
}

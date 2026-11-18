//! [#8606] 관계형·고정형 회귀 — 글자 없는 host 에 걸린 문단 기준 자리차지 표 둘 가운데, 자기
//! 세로 오프셋이 앞 머리 표의 점유 끝보다 아래인 여러 쪽 본표는 머리 표에 밀린 표가 아니다.
//! 한/글은 같은 쪽 자기 오프셋 자리에서 첫 조각을 시작한다. 두 표가 모두 오프셋 0 이라 둘째
//! 표가 실제로 밀린 #1831 형상(`samples/float-stack-defer.hwp`)은 종전대로 다음 쪽으로 넘긴다.
//!
//! 주석 갱신: 2026-11-05 06:29 (KST)
//!
//! - 입력: `samples/issue8606/delegation_trimmed.hwpx` — 대전광역시 사무위임 조례 [별표] 위임하는
//!   사항(한컴 2020 저장 HWPX, 24쪽)에서 구역 정의와 문단 0(별표 1)만 남겨 한컴 2022 로 다시
//!   저장한 축소본. 문단 0 은 글자가 없고 컨트롤 3 = 자리차지 표 120×4(세로 오프셋 6646,
//!   바깥 여백 0, 나눔), 컨트롤 4 = 자리차지 머리 표 2×3(세로 오프셋 0, 바깥 위·아래 141).
//!   host 저장 줄 vpos 4604 = 머리 표 바닥.
//! - 독립 기준: 같은 입력의 한컴 2022 Print PDF `pdf/issue8606_delegation_trimmed-2020.pdf`
//!   6쪽. 1쪽에 머리 표와 본표 행 0~11 이 있고, 본표 윗변은 문단 위 + 6646HU 다.
//! - 수정 전 rhwp(devel 7fdd5441ff): 7쪽. 1쪽에 머리 표만 두고 본표를 2쪽에서 시작했다
//!   (원본 전체 문서는 24쪽 대 26쪽).
//!
//! 측정(2026-11-05, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣, 수정 head 코드):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | Native 쪽별 | fresh WASM 쪽별 |
//! |---|---|---|---|
//! | delegation_trimmed | 6 | 93.98 98.45 97.67 98.75 99.00 98.84 | 100.00 ×6 |
//! | float-stack-defer(반례, #1831) | 2 | 99.97 99.98 | 99.96 99.97 |
//!
//! 두 경로 전쪽 최저: 93.98(delegation_trimmed 1쪽 Native, 「」 등 글자 폭 차이 — 표 원점·행
//! 구성·괘선은 한컴과 같다). 수정 전 Native 는 delegation_trimmed 가 7쪽으로 한컴과 쪽수가
//! 달라 쪽별 대조가 성립하지 않았다.
//!
//! 증거: `mydocs/pr/assets/issue_8606/{native,wasm}/*_silhouette.tsv`, `review/*.png`.
//! 검사는 두 표의 쪽 소속과 원점 차이를 저장 오프셋·바깥 여백과 비교하는 관계형과, 독립 PDF
//! 전체 쪽수를 고정하는 고정형이다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const DPI: f64 = 96.0;

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// 본문 최상위 표: (문단, 컨트롤, 상자).
fn top_tables(node: &RenderNode, out: &mut Vec<(usize, usize, BoundingBox)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if let (Some(para), Some(control)) = (table.para_index, table.control_index) {
            out.push((para, control, node.bbox));
        }
        return;
    }
    for child in &node.children {
        top_tables(child, out);
    }
}

fn page_tables(core: &DocumentCore, page: u32) -> Vec<(usize, usize, BoundingBox)> {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut out = Vec::new();
    top_tables(&tree.root, &mut out);
    out
}

fn hu(value: i32) -> f64 {
    f64::from(value) * DPI / 7200.0
}

#[test]
fn own_offset_body_table_starts_below_head_table_on_same_page() {
    let core = core("samples/issue8606/delegation_trimmed.hwpx");
    assert_eq!(core.page_count(), 6, "한컴 Print PDF 6쪽");
    let first = page_tables(&core, 0);
    let bbox = |control: usize| {
        first
            .iter()
            .find(|(para, ctrl, _)| *para == 0 && *ctrl == control)
            .map(|(_, _, bbox)| *bbox)
            .unwrap_or_else(|| panic!("1쪽에 문단 0 컨트롤 {control} 표가 없다: {first:?}"))
    };
    let (body, head) = (bbox(3), bbox(4));
    // 두 표는 같은 문단 원점에서 각자 오프셋을 잰다: 머리 표 윗변 = 원점 + 바깥 위 141,
    // 본표 윗변 = 원점 + 6646(바깥 위 0).
    let expected = hu(6646) - hu(141);
    assert!(
        (body.y - head.y - expected).abs() <= 1.0,
        "본표 윗변 - 머리 표 윗변 = {:.2}px, 저장 오프셋 차 {:.2}px",
        body.y - head.y,
        expected
    );
    assert!(
        body.y >= head.y + head.height,
        "본표({:.1})는 머리 표 바닥({:.1}) 아래",
        body.y,
        head.y + head.height
    );
    for page in 1..core.page_count() {
        assert!(
            page_tables(&core, page)
                .iter()
                .all(|(para, ctrl, _)| *para == 0 && *ctrl == 3),
            "{}쪽은 본표 이어받기 조각만",
            page + 1
        );
    }
}

#[test]
fn table_pushed_below_offset_zero_sibling_still_moves_whole() {
    // #1831 근거 형상: 두 표 모두 세로 오프셋 0 — 둘째 표는 첫 표 바닥까지 밀린다.
    let core = core("samples/float-stack-defer.hwp");
    assert_eq!(core.page_count(), 2, "한컴 2022 PDF 2쪽");
    let owner = |page: u32| {
        page_tables(&core, page)
            .iter()
            .map(|(para, ctrl, _)| (*para, *ctrl))
            .collect::<Vec<_>>()
    };
    assert!(owner(0).contains(&(1, 0)), "1쪽에 첫 표: {:?}", owner(0));
    assert!(
        !owner(0).contains(&(1, 1)),
        "밀린 둘째 표는 1쪽에 조각을 남기지 않는다: {:?}",
        owner(0)
    );
    assert!(owner(1).contains(&(1, 1)), "2쪽에 둘째 표: {:?}", owner(1));
}

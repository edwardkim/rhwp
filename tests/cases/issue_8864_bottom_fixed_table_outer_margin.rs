//! 관계형·고정형 — [#8864] 쪽 아래 고정 표의 배타 영역은 아래 바깥 여백까지 포함한다.
//! 주석 갱신: 2026-11-10 06:31 KST (새 파일)
//!
//! 입력: 공개 결재문서 36338125(한컴오피스 2020 저장) 원본. 마지막 문단 `pi=23` 은 쪽 기준·아래 정렬
//! 자리차지 표 둘을 갖는다. `ci=0` 은 쪽 맨 아래(오프셋 0·여백 0), `ci=2` 는 세로 오프셋 9591HU·
//! 아래 바깥 여백 140HU 위에 놓이는 수신자 표다. 본문 끝은 `ci=2` 위끝을 1.09px 넘는다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8864/36338125-bottom-fixed-table-margin-2020.pdf`(2쪽).
//! - 고정형: 전체 쪽수 2.
//! - 관계형: `ci=0` 은 1쪽에 남고 `ci=2` 만 2쪽으로 간다. 2쪽의 `ci=2` 아래끝은 본문 아래끝에서
//!   세로 오프셋과 아래 바깥 여백을 뺀 자리다.
//!
//! 측정(2026-11-10, 코드 head 7875b6e673, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 글꼴은 문서 face 만 모은 폴더로 공급):
//! - Native: 2쪽 / 2쪽, p1 98.93, p2 100.00, 전쪽 최저 98.93
//! - fresh WASM: 2쪽 / 2쪽, p1 99.34, p2 100.00, 전쪽 최저 99.34
//! - 증거: `mydocs/pr/assets/issue_8864/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8864/36338125-bottom-fixed-table-margin.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(para) && table.control_index == Some(control))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

#[test]
fn table_overlapping_body_end_through_its_bottom_margin_moves_to_next_page() {
    let core = open();
    assert_eq!(core.page_count(), 2, "한컴 engine 2020 Print PDF 는 2쪽");
    let first = core.build_page_render_tree(0).expect("1쪽");
    let second = core.build_page_render_tree(1).expect("2쪽");
    assert!(
        find_table(&first.root, 23, 0).is_some(),
        "본문과 겹치지 않는 쪽 맨 아래 표 ci=0 은 1쪽에 남는다"
    );
    assert!(
        find_table(&first.root, 23, 2).is_none(),
        "본문 끝과 겹치는 수신자 표 ci=2 는 1쪽에 남지 않는다"
    );
    assert!(
        find_table(&second.root, 23, 0).is_none(),
        "ci=0 은 2쪽으로 옮겨지지 않는다"
    );
    let table = find_table(&second.root, 23, 2).expect("2쪽 수신자 표 ci=2");
    let body = body(&second.root).expect("2쪽 Body");
    let expected = body.bbox.y + body.bbox.height - hu_to_px(9591 + 140);
    let bottom = table.bbox.y + table.bbox.height;
    assert!(
        (bottom - expected).abs() <= TOLERANCE_PX,
        "ci=2 아래끝 {bottom:.1} 은 본문 아래끝에서 오프셋·아래 여백을 뺀 {expected:.1} 이어야 한다"
    );
}

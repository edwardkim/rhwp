//! [#6344] 용지 기준 표만 있는 쪽을 `empty_page` 로 오판하지 않는다.
//!
//! `layout_anomaly::scan_page` 의 콘텐츠 판정(`has_content`)은 `Body` 서브트리만 돌았다.
//! 그런데 용지 기준으로 배치된 표·도형은 `Body` 가 아니라 **페이지 직계 자식**으로
//! 그려지므로, `Body` 만 보면 그 쪽이 통째로 비어 보인다.
//!
//! # 실측 — 세 방향이 전부 "내용 있음" 인데 판정만 빈 쪽
//!
//! `samples/table-ipc.hwp` 는 10 쪽 문서인데 8 쪽이 빈 쪽으로 잡혔다.
//!
//! | 근거 | 결과 |
//! | --- | --- |
//! | 한컴 정답지 `pdf/table-ipc-2022.pdf` | 10 쪽 모두 861~1,026자 |
//! | `rhwp export-text` | 10 쪽 모두 700~860자 |
//! | 종전 `layout-anomaly` | **8 쪽이 빈 쪽** |
//!
//! 렌더 트리를 열면 원인이 보인다 — 표(181 칸, 글자 154 개)와 쪽번호가 페이지 직계
//! 자식이고 `Body` 는 비어 있다.
//!
//! ```text
//! Page
//! ├─ Body    글자 0개          <- 종전에는 여기만 순회
//! ├─ Table   글자 154개, 181칸
//! ├─ Rect    '3/10'
//! └─ Footer
//! ```
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/table-ipc.hwp";

fn load() -> Option<DocumentCore> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).ok()?;
    DocumentCore::from_bytes(&bytes).ok()
}

/// 용지 기준 표가 있는 쪽은 빈 쪽이 아니다.
#[test]
fn paper_anchored_table_page_is_not_reported_empty() {
    let Some(doc) = load() else {
        return;
    };
    let page_count = doc.page_count();
    let opts = AnomalyOptions::default();

    let mut empty_pages = Vec::new();
    for page in 0..page_count {
        let Ok(tree) = doc.build_page_render_tree(page) else {
            continue;
        };
        if scan_page(page, &tree.root, page_count, &opts)
            .empty_page
            .is_some()
        {
            empty_pages.push(page);
        }
    }

    assert!(
        empty_pages.is_empty(),
        "{SAMPLE} 는 한컴 정답지·export-text 모두 10쪽 전부 내용이 있다. \
         빈 쪽으로 잡힌 쪽: {empty_pages:?}"
    );
}

/// 콘텐츠 판정만 넓혔고 다른 판정의 기준 상자는 그대로다.
///
/// `overflow` 는 본문 여백, `off-canvas` 는 페이지 상자라는 뜻이 분명하므로 이 변경이
/// 그 수치를 건드리면 안 된다. 이 문서의 종전 실측은 셋 다 0 이다.
#[test]
fn other_verdicts_are_unchanged() {
    let Some(doc) = load() else {
        return;
    };
    let page_count = doc.page_count();
    let opts = AnomalyOptions::default();

    let (mut overflow, mut off_canvas, mut overlap) = (0usize, 0usize, 0usize);
    for page in 0..page_count {
        let Ok(tree) = doc.build_page_render_tree(page) else {
            continue;
        };
        let pa = scan_page(page, &tree.root, page_count, &opts);
        overflow += pa.overflow.len();
        off_canvas += pa.off_canvas.len();
        overlap += pa.overlap.len();
    }

    assert_eq!(
        (overflow, off_canvas, overlap),
        (0, 0, 0),
        "콘텐츠 판정을 넓힌 것이 컨테이너 판정에 새어 들어갔다 \
         (overflow, off_canvas, overlap) = ({overflow}, {off_canvas}, {overlap})"
    );
}

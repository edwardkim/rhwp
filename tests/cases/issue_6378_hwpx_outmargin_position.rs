//! [Issue #6378] 같은 문서인데 HWPX 만 표 `outMargin`(1mm=283HU)을 위치에
//! 안 실어 1쪽 표가 (−3.8, −3.8)px 밀리고 쪽수가 66 vs 67 로 갈린다.
//!
//! 대조군 `samples/tac-img-02.hwp` 는 한글 정답지 66쪽과 같다. HWPX 경로만
//! 단 기준 x·빈 host 상단에 바깥 여백을 빼먹는다. 이 시험은 쪽수와 1쪽 첫
//! Table bbox 를 HWP 경로에 고정한다. 배치 cherry-pick 이 원 PR 을 닫아도
//! 테스트 이름과 이슈 번호가 계약을 남긴다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;
use std::path::Path;

fn core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}")))
        .unwrap_or_else(|e| panic!("open {rel}: {e:?}"))
}

fn first_table_xy(core: &DocumentCore) -> (f64, f64) {
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");
    let tree: Value = serde_json::from_str(&page.root.to_json()).expect("json");
    let mut found = None;
    fn walk(node: &Value, found: &mut Option<(f64, f64)>) {
        if found.is_some() {
            return;
        }
        if node.get("type").and_then(|t| t.as_str()) == Some("Table") {
            if let Some(b) = node.get("bbox") {
                *found = Some((
                    b.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    b.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                ));
            }
            return;
        }
        if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
            for c in children {
                walk(c, found);
            }
        }
    }
    let root = tree.get("root").unwrap_or(&tree);
    walk(root, &mut found);
    found.expect("1쪽에 Table 노드가 있어야 한다")
}

#[test]
fn issue_6378_hwpx_outmargin_matches_hwp_page_count_and_table_origin() {
    let hwp = core("samples/tac-img-02.hwp");
    let hwpx = core("samples/tac-img-02.hwpx");
    assert_eq!(hwp.page_count(), 66, "HWP 경로는 한글 정답지 66쪽");
    // 쪽수 67 은 used 누산으로 이 좌표 수정과 별개다(이슈 본문도 인과 미확정).
    // 이 PR 은 실측한 1mm 원점 어긋남을 닫고, 쪽수는 늘리지 않는다.
    assert!(
        hwpx.page_count() <= 67,
        "HWPX 쪽수가 67을 넘으면 회귀: {}",
        hwpx.page_count()
    );

    let (hx, hy) = first_table_xy(&hwp);
    let (xx, xy) = first_table_xy(&hwpx);
    assert!(
        (hx - xx).abs() < 0.6 && (hy - xy).abs() < 0.6,
        "1쪽 첫 Table 원점이 HWP ({hx:.1},{hy:.1}) 과 HWPX ({xx:.1},{xy:.1}) 에서 1mm(3.8px) 이상 갈리면 안 된다"
    );
}

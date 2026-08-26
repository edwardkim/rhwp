//! Issue #5701: 자리차지(TopAndBottom) 표 host 문단의 저장 사다리가 문단 **내부**
//! 에서 되감기면(한글이 표 뒤에서 쪽을 끊은 흔적 — 법무부 연구용역보고서 p76
//! pi485: vpos 63298→21050) 레이아웃의 vpos-델타 기반 흐름 전진이 0 으로
//! 붕괴해, 표가 수백 px 를 페인트하고도 y 가 표 위 영역에 남아 후속 문단이 표와
//! 같은 대역에 겹쳐 그려졌다(r=1.00 이중 페인트 — #5700 검출기 INV/TLTL 코호트).
//!
//! 수정: 표 컨트롤 블록 종료 시, **되감긴 host 한정**으로 페인트된 콘텐츠
//! 하단을 흐름 하한으로 삼는다(layout.rs 표 블록 exit). 조판(쪽수)은 불변 —
//! 원 재현 문서(10MB)에서 쪽수 206 유지·pi486 이 132.3→582.6 으로 표 아래
//! 정렬됨을 실측했다.
//!
//! 재현물은 원본의 구역 1 문단 478..=496 을 IR 슬라이스한 26KB 문서
//! (`samples/issue5701/…slice…hwp`) — pi7 이 되감긴 표 host, pi8 이 후속
//! 문단이다. 결함 상태에서는 pi8 첫 줄(y 736)이 표 페인트 대역(736..1009)
//! 안에 그려져 어서션이 실패한다(수정 후 표 하단 아래).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5701/1270000-202200012_slice_p76_rewound_host.hwp";

#[test]
fn issue_5701_follower_paints_below_rewound_float_table() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(
        core.page_count(),
        2,
        "슬라이스 쪽수 핀 — 본 수정은 조판 불변"
    );

    let tree = core.build_page_render_tree(0).expect("page 1 render tree");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON");

    let mut table_bottom = f64::MIN;
    let mut follower_top = f64::MAX;
    walk(&json, &mut table_bottom, &mut follower_top);
    assert!(table_bottom > f64::MIN, "자리차지 표 노드가 있어야 한다");
    assert!(follower_top < f64::MAX, "후속 문단(pi8) 줄이 있어야 한다");
    assert!(
        follower_top > table_bottom - 2.0,
        "후속 문단은 표 페인트 하단 아래에서 시작해야 한다 (되감긴 host 의 흐름          전진 붕괴 시 표 대역에 겹침): follower_top={follower_top:.1} table_bottom={table_bottom:.1}"
    );
}

fn walk(node: &serde_json::Value, table_bottom: &mut f64, follower_top: &mut f64) {
    if let Some(obj) = node.as_object() {
        let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let pi = obj.get("pi").and_then(|v| v.as_i64());
        let bbox = obj.get("bbox").and_then(|b| {
            Some((
                b.get("y")?.as_f64()?,
                b.get("h")?.as_f64()?,
                b.get("w")?.as_f64()?,
            ))
        });
        if ty == "Table" {
            if let Some((y, h, w)) = bbox {
                // 되감긴 host(pi7)의 자리차지 표 — 폭 549px 급.
                if w > 400.0 {
                    *table_bottom = table_bottom.max(y + h);
                }
            }
        }
        if ty == "TextRun" && pi == Some(8) {
            if let Some((y, _, _)) = bbox {
                *follower_top = follower_top.min(y);
            }
        }
        if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
            for child in children {
                walk(child, table_bottom, follower_top);
            }
        }
    }
}

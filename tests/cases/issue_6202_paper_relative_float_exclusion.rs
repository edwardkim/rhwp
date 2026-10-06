//! [Issue #6202] 용지 기준(`Paper`/`Page`) 어울림 개체의 배제 밴드를 **계산한다**.
//!
//! 종전 `resolve_picture_exclusion` 은 `horz_rel`/`vert_rel` 이 `Paper|Page` 면 바로
//! `None` 이라, 이 형상의 되감김은 오직 저장 `LINE_SEG` 가 준 것이었다. 그래서 studio 에서
//! 개체를 옮겨도 본문이 옛 밴드로 되감겨 그림이 글자를 덮었다.
//!
//! `156483689` 실측 — 계산이 저장 사다리를 **1 HU 오차로 재현**한다:
//!
//! ```text
//!   그림  w=12482 h=9366  wrap=Square  vert=Paper(36844)  horz=Paper(42333)
//!   본문  왼쪽 5670 HU · 폭 48188 HU
//!
//!   가로  깎인 줄 오른끝 = 5670 + 36664 = 42334   vs  그림 왼쪽 42333   (1 HU)
//!   세로  밴드(본문좌표) = 36844 − 5670 .. +9366 = 31174 .. 40540
//!         pi=5 vpos 29631..32031 깎임 · pi=6 34431 깎임
//!         pi=7 36831..44031 에서 40540 을 지나며 전폭 복귀
//! ```
//!
//! 코퍼스 1,997건 표본에서 Square float 개체를 막던 관문은 사실상 `Paper|Page` 기준
//! 뿐이었다(`horz_rel` 57 · `vert_rel` 57). 계산식은 `Column`/`Para` 와 **같고 기준점만**
//! 용지 원점으로 바뀐다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6202/156483689-turmeric-industry-standardization.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6202 정식 HWP fixture 읽기")
}

/// 1쪽에서 그림 밴드에 깎인 줄들의 오른쪽 끝.
fn carved_right_edges(core: &DocumentCore) -> Vec<f64> {
    let Ok(tree) = core.build_page_render_tree(0) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    fn walk(n: &RenderNode, out: &mut Vec<(f64, f64)>) {
        if matches!(n.node_type, RenderNodeType::TextLine(_)) {
            out.push((n.bbox.y, n.bbox.x + n.bbox.width));
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let mut rows = Vec::new();
    walk(&tree.root, &mut rows);
    // 그림 밴드가 걸치는 y 구간(492~610px)의 줄만 본다.
    rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for (y, right) in rows {
        if (480.0..640.0).contains(&y) {
            out.push((right * 10.0).round() / 10.0);
        }
    }
    out
}

/// 정적 렌더는 그대로여야 한다 — 계산이 저장 사다리를 재현하므로 우단 564.5px.
#[test]
fn static_render_keeps_the_stored_band() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let edges = carved_right_edges(&core);
    assert!(
        !edges.is_empty(),
        "1쪽 그림 밴드 구간의 줄을 못 찾았다 — 시험 설정 오류"
    );
    let carved = edges.iter().filter(|r| (**r - 564.5).abs() <= 1.5).count();
    assert!(
        carved >= 3,
        "그림을 피해 우단 564.5px 에서 끊긴 줄이 3개 이상이어야 한다 — #6202 회귀. {edges:?}"
    );
}

/// 개체를 왼쪽으로 옮기면 본문이 **새 자리**를 피해 되감겨야 한다.
///
/// 종전에는 옛 밴드(우단 564.5)가 그대로 남아 그림이 글자를 덮었다.
#[test]
fn moving_the_float_reflows_the_body() {
    let bytes = sample();
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    // 이슈의 재현 그대로 — horzOffset 42333 → 22333 (266.7px 왼쪽으로).
    core.set_picture_properties_native(0, 5, 0, r#"{"horzOffset":22333}"#)
        .expect("개체 이동 뒤 Picture band 재투영");
    let edges = carved_right_edges(&core);
    assert!(
        !edges.is_empty(),
        "이동 후 1쪽 줄을 못 찾았다 — 시험 설정 오류"
    );
    // 새 그림 왼쪽 = 22333 HU = 297.8px. 옛 자리(564.5)로 남아 있으면 회귀다.
    let stale = edges.iter().filter(|r| (**r - 564.5).abs() <= 1.5).count();
    assert_eq!(
        stale, 0,
        "개체를 옮겼는데 본문이 옛 밴드(우단 564.5px)로 되감겨 있다 — #6202 회귀. {edges:?}"
    );
}

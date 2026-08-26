//! [#5870] 빈 host 문단의 자리차지 표는 흐름을 `v_off + outer_top + 표높이 +
//! outer_bottom` 만큼 전진시켜야 한다 — 단, 저장 사다리가 그 물리 공식과 정확히
//! 일치하는 문단에서만(#2097 광역 반증 회피 게이트).
//!
//! `empty_host_float_flow_advance.hwp`(10645 부패행위 신고사무 운영지침) 40쪽
//! [별지 제11호서식]: 결재란(2×3, pi=45)과 본표(10×6, pi=46)가 각각 빈 문단에
//! 고정된 자리차지 float 다. 저장 사다리 델타 8413HU = v_off 1840 + outer 140 +
//! 표높이 6293 + outer 140 정확 일치. 수정 전 rhwp 는 표 높이(6293)만 전진시켜
//! 본표가 288.0px 에서 시작 — 결재란 하단(307.7px)을 19.7px 파고들었다.
//! 수정 후 316.2px(한글 2022 정답지 317.7px, 잔차 1.5px).
//!
//! 같은 문서의 통제군: 구역2 문단 62 는 형상이 같지만 host 에 글자가 6자 있어
//! `is_visible_para_float` 정상 경로를 탄다 — 45쪽 pi=63 표 상단 297.4px 불변.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5870/empty_host_float_flow_advance.hwp";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

/// 페이지 SVG 에서 가로 괘선(수평선)의 y 좌표를 모은다.
fn hline_ys(svg: &str) -> Vec<f64> {
    let mut ys = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x1), Some(x2), Some(y1), Some(y2)) =
            (attr("x1"), attr("x2"), attr("y1"), attr("y2"))
        {
            if (y1 - y2).abs() < 0.5 && (x2 - x1).abs() > 50.0 {
                ys.push(y1);
            }
        }
    }
    ys
}

#[test]
fn issue_5870_page40_main_table_clears_approval_box() {
    let doc = load_doc();
    let svg = doc.render_page_svg_native(39).expect("page 40 svg");
    let ys = hline_ys(&svg);
    // 본표 첫 가로 괘선: 한글 317.7 부근 — 결함 시 288.0 (결재란 하단 307.7 침범).
    assert!(
        ys.iter().any(|y| (y - 316.2).abs() < 3.0),
        "#5870: 40쪽 본표 상단 괘선이 결재란 아래(≈316)여야 한다 — 결함 시 288.0: {ys:?}"
    );
    assert!(
        !ys.iter().any(|y| (y - 288.0).abs() < 2.0),
        "#5870: 결함 위치(288.0)의 본표 상단 괘선이 남아 있으면 안 된다: {ys:?}"
    );
}

#[test]
fn issue_5870_page45_visible_host_control_group_unchanged() {
    let doc = load_doc();
    let svg = doc.render_page_svg_native(44).expect("page 45 svg");
    let ys = hline_ys(&svg);
    // 통제군(host 에 글자 있는 같은 형상): pi=63 표 상단 297.4 — 저장 기준 297.45.
    assert!(
        ys.iter().any(|y| (y - 297.4).abs() < 2.0),
        "#5870 통제군: 45쪽 pi=63 표 상단(297.4)은 불변이어야 한다: {ys:?}"
    );
}

#[test]
fn issue_5870_page_count_stays_63() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        63,
        "#5870: 흐름 가산은 저장 사다리와 동기이므로 쪽수 63 은 그대로여야 한다"
    );
}

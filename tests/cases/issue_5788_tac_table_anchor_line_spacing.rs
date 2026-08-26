//! [Issue #5788] 글자처럼 취급 표 아래 문단이 9.2px 붙는다 — 18곳 전부 11.46px 고정
//! (한글 20.7px), 누적되어 쪽수까지 어긋난다 (3190263).
//!
//! 근인: 저장 lineseg 가 없는(기계생성) 문서에서 TAC 표 앵커 줄의 trailing
//! line_spacing 을 실을 seg 가 없어, 표 아래 y 전진이 표 높이+바깥여백에서 끝났다.
//! 저장 lineseg 보유 문서는 seg.line_spacing 경로(#1116 핀)가 이미 싣는다.
//!
//! 수정(layout_table_item): lineseg-부재 문단 한정으로 문단 스타일의 퍼센트
//! 줄간격 × 앵커 글꼴 크기(+8.0px, 160%×13.3px)를 보충 — 표→다음 문단 간격
//! 11.46 → 19.5px (한글 20.26~21.23, 잔여 ~1px 는 ascent 축).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5788/tac_table_missing_anchor_line_spacing.hwp";

#[test]
fn issue_5788_paragraph_after_tac_table_gets_anchor_line_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 1쪽 첫 표의 아래 괘선(y≈462.9, 종전 464.8-8? → 실측로 판별)과 그 아래 "(주)"
    // 첫 글줄 baseline 의 간격이 앵커 줄간격을 포함해 19px 이상이어야 한다
    // (결함 시 11.46px 고정).
    let mut rule_ys = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(y1), Some(y2), Some(x1), Some(x2)) =
            (attr("y1"), attr("y2"), attr("x1"), attr("x2"))
        {
            if (y1 - y2).abs() < 0.3 && (x2 - x1) > 300.0 {
                rule_ys.push(y1);
            }
        }
    }
    rule_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // 첫 표의 마지막(가장 아래) 가로 괘선: 400~480 구간의 최댓값.
    let table_bottom = rule_ys
        .iter()
        .cloned()
        .filter(|y| (380.0..500.0).contains(y))
        .fold(f64::MIN, f64::max);
    assert!(
        table_bottom > 0.0,
        "첫 표 아래 괘선을 찾아야 한다: {rule_ys:?}"
    );

    // 그 아래 첫 글자 baseline.
    let mut next_baseline = f64::MAX;
    for cap in svg.split("<text ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        if let Some(s) = head.find("y=\"") {
            let s = s + 3;
            if let Some(e) = head[s..].find('"') {
                if let Ok(y) = head[s..s + e].parse::<f64>() {
                    if y > table_bottom + 1.0 && y < next_baseline {
                        next_baseline = y;
                    }
                }
            }
        }
    }
    let gap = next_baseline - table_bottom;
    assert!(
        gap >= 18.0,
        "표 아래 괘선→다음 baseline 간격({gap:.2})이 앵커 줄간격 포함 19px 급이어야 한다 \
         — 결함 시 11.46 고정 (한글 20.7)"
    );
}

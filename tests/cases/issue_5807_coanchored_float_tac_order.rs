//! [Issue #5807] 같은 빈 문단에 co-anchored 된 자리차지 표(4행)와 TAC 표(2행)의
//! 위·아래 순서가 뒤집혀, 2쪽에서 표가 쪽 밖으로 354.6px 넘쳤다 (1880690 서식).
//!
//! 근인: 빈 host 의 float 표 v_off 오름차순 정렬(#986/#1088)이 TAC 표에 정렬 키
//! 0 을 주어, 양수 v_off(937) float 앞으로 끌어왔다. v_off 는 자리차지 개체의
//! 문단 상대 위치이고 TAC 의 흐름 위치와 다른 축이다. 저장 배열 순서·float
//! offset·TAC 저장 줄 vpos(12924)·한글 배치 모두 float 먼저다.
//!
//! 수정: TAC 표와 양수 v_off float 표가 혼재한 host 는 #1639(음수 혼재)/#2287
//! (vpos 리셋)과 같이 정렬을 끄고 배열(저장) 순서를 보존한다.
//!
//! 픽스처는 원본 HWP5 의 구역0 문단 0..4 절단 + BinData 스텁 축소본(9KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5807/coanchored_float_tac_order.hwp";

fn rule_ys(svg: &str) -> Vec<f64> {
    let mut ys = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        if let Some(v) = head
            .split_once("y1=\"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .and_then(|(v, _)| v.parse::<f64>().ok())
        {
            ys.push(v);
        }
    }
    ys
}

#[test]
fn issue_5807_float_table_precedes_coanchored_tac_table() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 3, "한글과 같은 3쪽이어야 한다");

    // 2쪽: 결함 시 자리차지 표가 y=1439.4 까지 그려져 본문 하단(1084.7)을 354.6px
    // 넘쳤다. 수정 후 최대 괘선 y 는 본문 안이다.
    let p2 = core.render_page_svg_native(1).expect("page 2 svg");
    let p2_max = rule_ys(&p2).into_iter().fold(f64::MIN, f64::max);
    assert!(
        p2_max < 1085.0,
        "2쪽 괘선이 본문 하단 안에 있어야 한다 (결함 시 1439.4): {p2_max:.1}"
    );

    // 3쪽: 자리차지 표 꼬리(≈332) **아래에** TAC 표(≈375~704)가 놓인다.
    // 결함 시 TAC 표는 2쪽으로 앞당겨져 3쪽 최대 괘선이 ≈332 에 그친다.
    let p3 = core.render_page_svg_native(2).expect("page 3 svg");
    let p3_max = rule_ys(&p3).into_iter().fold(f64::MIN, f64::max);
    assert!(
        (690.0..730.0).contains(&p3_max),
        "3쪽에서 TAC 표가 자리차지 표 꼬리 아래(하단 괘선 ≈704)에 있어야 한다: {p3_max:.1}"
    );
}

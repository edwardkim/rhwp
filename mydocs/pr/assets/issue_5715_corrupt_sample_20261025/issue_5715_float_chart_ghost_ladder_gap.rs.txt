//! [Issue #5715] 어울림(T&B) 차트가 지면 밖(y=-533)이나 본문 위에 그려진다 —
//! 그림 있는 26쪽 중 9쪽(34.6%) (베트남노동시장1125).
//!
//! 근인: #1079 gap 휴리스틱(문단 앞 저장 vpos gap ≥ 그림 높이 → 그림 바닥을 문단
//! 줄에 정렬)이 **쪽 리셋 뒤 유령 사다리**가 만든 가짜 gap 에 발동했다. 리셋된
//! 0-기저 쪽(vpos 0..17040)에 앞 lineage 의 vpos(65410)가 섞인 문단에서 gap
//! +47170HU 가 실재 예약으로 오인돼, 바닥-정렬이 차트를 단 상단 위(y=83.9, 본문
//! top 117.2)나 지면 밖(-533)으로 밀었다.
//!
//! 수정: 바닥-정렬 목표가 단 상단을 넘으면 #1079 를 기각하고, 같은 문단의 선행
//! float(표)이 진행시킨 흐름(y_offset) 뒤로 배치한다. 원문서 124쪽 전수 census:
//! 본문 상단 위 그림 8쪽 → **0쪽**, 쪽수 124 불변.
//!
//! 잔여(별개 축): p72 같은 앵커 두 차트 겹침, p73 음수 offset 차트 41px 부분 겹침.
//!
//! 픽스처는 원본 HWP5 구역1 문단 914..926 절단 + BinData 1×1 스텁(21.5KB, 2쪽).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5715/float_chart_ghost_ladder_gap.hwp";

#[test]
fn issue_5715_chart_stays_below_body_top_after_ghost_gap_reject() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 2);
    let svg = core.render_page_svg_native(1).expect("page 2 svg");

    // 차트(높이 302.8px 그림)의 y. 결함 시 83.9(본문 top 117.2 위), 정상 699.4
    // (같은 문단 선행 표 370..682.7 아래).
    let mut chart_y = None;
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(y), Some(h)) = (attr("y"), attr("height")) {
            if (295.0..310.0).contains(&h) {
                chart_y = Some(y);
            }
        }
    }
    let chart_y = chart_y.expect("302.8px 차트 그림이 있어야 한다");
    assert!(
        chart_y >= 117.0,
        "차트가 본문 상단(117.2) 아래에 있어야 한다 (결함 시 83.9): {chart_y:.1}"
    );
    assert!(
        (690.0..710.0).contains(&chart_y),
        "차트가 선행 표(682.7) 아래 흐름 자리에 있어야 한다: {chart_y:.1}"
    );
}

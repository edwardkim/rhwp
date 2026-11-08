//! [Issue #5833] 한 셀 안에 문단마다 자리차지(T&B) 그림이 하나씩 든 표에서 그림이
//! 흐름으로 쌓이지 않고 같은 기준선에 겹쳐, 앞 그림이 뒤 그림에 덮여 사라진다
//! (156684746 6쪽 표4: 머리기사 줄 그림이 본문 그림 안쪽 +60px 에 파묻힘).
//!
//! 근인: 그림-단위 셀 valign 강제(#2071)가 다문단 float-그림 셀까지 덮었다. 한글은
//! 그림을 저장 vpos 사다리로 직렬 적층하고 셀 valign 은 **블록 전체**에 적용한다.
//! text_y_start 의 센터링은 저장 extent 신뢰가 그림 높이를 안 담는 사다리(빈 줄
//! lh 만, 실측 34.7px vs 블록 135.9px)로 접힐 수 있어, 블록 extent(문단 vpos +
//! 그림 높이 최대)를 자체 계산해 valign 을 적용한다.
//!
//! 잔여: 9쪽 표8(rowspan 조각 + 자리차지 제한 OFF)은 별개 좌표 축으로 추적.
//!
//! 픽스처는 원본 HWP5 구역0 문단 63..65 절단 + BinData 1×1 스텁 축소본(20KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5833/cell_multi_para_float_pics.hwp";

#[test]
fn issue_5833_stacked_cell_pics_flow_without_overlap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // r1c0 셀의 그림 두 장(x≈84~86): 머리기사 줄(h=14.7)과 기사 본문(h=114.6).
    let mut pics = Vec::new();
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x), Some(y), Some(h)) = (attr("x"), attr("y"), attr("height")) {
            if (84.0..87.0).contains(&x) && h < 130.0 {
                pics.push((y, h));
            }
        }
    }
    pics.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!(pics.len() >= 2, "r1c0 그림 2장이 있어야 한다: {pics:?}");
    let (y0, h0) = pics[0];
    let (y1, _) = pics[1];

    // 머리기사 줄 그림이 위(블록 센터 시작, 한글 151.5), 본문 그림이 그 아래
    // (한글 172.9). 결함 시 y0=211.8 로 본문 그림 상자(162.5~277.1) 안에 파묻힌다.
    assert!(
        (148.0..154.0).contains(&y0),
        "머리기사 줄 그림이 블록 상단에 있어야 한다 (한글 151.5, 결함 시 211.8): {y0:.1}"
    );
    assert!(
        y0 + h0 <= y1 + 0.5,
        "앞 그림이 뒤 그림 위에서 끝나야 한다(겹침 금지): p0 {y0:.1}+{h0:.1} vs p1 {y1:.1}"
    );
    assert!(
        (170.0..176.0).contains(&y1),
        "본문 그림이 앞 그림 아래 저장 vpos 자리에 있어야 한다 (한글 172.9): {y1:.1}"
    );
}

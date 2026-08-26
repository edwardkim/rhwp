//! [Issue #5808] 어울림(Square) 묶음 그림의 왼쪽 바깥여백 566HU 가 가로 위치에
//! 빠져 그림이 7.2px 왼쪽에 그려진다 (156518601 1쪽).
//!
//! 근인: `compute_object_position` 이 float 개체 x 에 `margin.left`(바깥여백)를
//! 가산하지 않았다. 한글 실측 = 본문 왼쪽 + 오프셋 29138HU + 바깥여백 566HU
//! = 471.6px (실측 471.3). 실측이 Left 정렬뿐이라 Square+Left/Inside 한정으로
//! 가산하고 Center/Right 는 종전 유지.
//!
//! 픽스처는 원본 HWPX 의 구역0 문단 7(묶음 앵커) 절단 + BinData 1×1 스텁 축소본.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5808/square_group_left_outer_margin.hwpx";

#[test]
fn issue_5808_square_group_x_includes_left_outer_margin() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 묶음 child 그림 2장의 x. 결함 시 464.1/593.4(여백 미가산), 정상 471.6/600.9.
    let mut xs = Vec::new();
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x), Some(w)) = (attr("x"), attr("width")) {
            if (w - 124.7).abs() < 1.0 {
                xs.push(x);
            }
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(xs.len(), 2, "묶음 child 그림 2장이 있어야 한다: {xs:?}");
    assert!(
        (xs[0] - 471.6).abs() < 1.0 && (xs[1] - 600.9).abs() < 1.0,
        "묶음 그림 x 가 오프셋+왼쪽 바깥여백 자리에 있어야 한다 (한글 471.3/600.6, \
         결함 시 464.1/593.4): {xs:?}"
    );
}

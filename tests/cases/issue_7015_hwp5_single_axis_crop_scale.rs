//! [Issue #7015] HWP5 그림 자르기에서 **한 축만** 진짜 자르기일 때 적응 배율이 틀려
//! 로고가 절반만 보인다
//! (`samples/issue7015/30442-acrc-recommendation-business-burden.hwp` 3쪽).
//!
//! 기전: `compute_image_crop_src` 의 적응 폴백(#3239)은 crop `right`/`bottom` 이
//! **전체 좌표 범위**라는 가정 위에 선다. 그 가정은 그 축을 자르지 않았을 때만
//! 성립하는데, 축을 나눠 보지 않고 두 값을 늘 전체 범위로 썼다. `Picture::img_dim`
//! 은 HWPX 파서만 적재하므로 HWP5 는 항상 이 폴백을 탄다.
//!
//! 이 문서의 원값(`HWPTAG_SHAPE_COMPONENT_PICTURE`)과 디코딩 크기:
//!
//! ```text
//!   crop  left=0  top=20745  right=88560  bottom=45453      이미지 1181 × 945 px
//!
//!   x 축  left = 0    → right 가 전체 범위   88560 / 1181 = 75.0  (표준과 일치)
//!   y 축  top  > 0    → bottom 은 자르기 경계  45453 /  945 = 48.1  ← 36% 작다
//! ```
//!
//! 배율이 작아지면 자르기 창이 아래로 밀리고 길어진다. 결함 상태의 `viewBox` 는
//! `y 431.3 .. 945.0` 인데 로고 잉크는 `y 324 .. 562` 라, 위쪽 107행이 잘리고 아래
//! 383행은 흰 여백만 들어왔다 — "로고가 절반만 보인다".
//!
//! 수정: 적응 배율은 **그 축의 자르기가 0 에서 시작할 때만** 유효하다. 한 축만
//! 확인되면 그 배율을 두 축에 쓰고(HWP5 crop 좌표는 등방), 둘 다 없으면 [Task #477]
//! 표준 75 HU/px 로 떨어진다. 여기서는 x 축의 75.0 이 y 에도 적용돼
//! `viewBox y 276.6 .. 606.1` 로 로고를 온전히 감싼다.
//!
//! 정답지: 문서 작성 버전 `7.5.12.754` → engine 2020 버킷으로 새로 생성한 PDF 가
//! 로고 전체를 그린다. `layout-anomaly` 는 이 쪽에 아무 신호도 내지 않는다 —
//! 기하가 아니라 그림 자르기 축이기 때문이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue7015/30442-acrc-recommendation-business-burden.hwp";

/// 원본 이미지(1181 × 945) 안에서 권익위 로고 잉크가 차지하는 세로 범위.
const LOGO_INK_TOP: f64 = 324.0;
const LOGO_INK_BOTTOM: f64 = 562.0;

#[test]
fn issue_7015_single_axis_crop_keeps_the_whole_logo() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let svg = core.render_page_svg_native(2).expect("page 3 svg");
    let logo_box = crop_view_box_for_image(&svg, 1181.0, 945.0)
        .expect("3쪽에 1181×945 로고 그림의 자르기 viewBox 가 있어야 한다");
    let (_, y, _, h) = logo_box;

    assert!(
        y < LOGO_INK_TOP,
        "자르기 창 위끝이 로고 잉크(y={LOGO_INK_TOP}) 위여야 한다 (결함 시 431.3): {y:.1}"
    );
    assert!(
        y + h > LOGO_INK_BOTTOM,
        "자르기 창 아래끝이 로고 잉크(y={LOGO_INK_BOTTOM}) 아래여야 한다: {:.1}",
        y + h
    );
    // x 축 배율 75.0 을 y 에 적용한 값 — 결함 시 431.3 / 513.7.
    assert!((y - 276.6).abs() < 0.5, "viewBox y={y:.1}, 기대 276.6");
    assert!((h - 329.5).abs() < 0.5, "viewBox h={h:.1}, 기대 329.5");
}

/// `width`/`height` 가 주어진 `<image>` 를 감싼 `<svg>` 의 `viewBox` 를 찾는다.
fn crop_view_box_for_image(svg: &str, img_w: f64, img_h: f64) -> Option<(f64, f64, f64, f64)> {
    let needle = format!("width=\"{img_w}\" height=\"{img_h}\"");
    let image_at = svg.find(&needle)?;
    // 그 `<image>` 바로 앞의 `<svg ... viewBox="...">` 를 거슬러 찾는다.
    let open_at = svg[..image_at].rfind("<svg ")?;
    let head_end = svg[open_at..].find('>')? + open_at;
    let head = &svg[open_at..head_end];
    let vb_at = head.find("viewBox=\"")? + "viewBox=\"".len();
    let vb_end = head[vb_at..].find('"')? + vb_at;
    let nums: Vec<f64> = head[vb_at..vb_end]
        .split_whitespace()
        .filter_map(|n| n.parse().ok())
        .collect();
    match nums[..] {
        [x, y, w, h] => Some((x, y, w, h)),
        _ => None,
    }
}

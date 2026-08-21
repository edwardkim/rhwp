//! [Issue #5798] 본문만 +203px 내려가 제자리의 결재란과 겹친다 (2401225 근무일지).
//!
//! 근인: 단(그리고 용지) **밖에 통째로** 놓인 자리차지(T&B) 결재란 표 2개
//! (horz=단 offset 64328HU → x=933px, 용지 793px)가 각 ~101.7px 씩 흐름 밴드를
//! 예약해 본문을 2×101.7=+203px 밀었다. 한글은 가로로 글과 겹칠 수 없는 표에
//! 밴드를 주지 않는다(그림의 #959 가드와 동일 시멘틱).
//!
//! 수정: T&B·Left/Inside·Column/Para 기준 표의 가로 구간이 단과 전혀 겹치지
//! 않으면 #703 데코레이션 경로(흐름 무예약, 절대 배치)로 보낸다. 하이픈 소실(②)은
//! kevin9327 의 PR #5826 이 다룬다.
//!
//! 픽스처는 원본 HWP5 구역0 문단 0..8 절단 + BinData 스텁(8.7KB, 1쪽).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5798/offcolumn_float_table_no_band.hwp";

#[test]
fn issue_5798_offcolumn_approval_tables_reserve_no_flow_band() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 제목 `근 무 일지` 첫 글자의 y. 결함 시 +203px(338.7 부근), 정상 ~120(한글
    // baseline 135.4 와 같은 대역).
    let mut title_y = None;
    for cap in svg.split("<text ").skip(1) {
        let Some(end) = cap.find("</text>") else {
            continue;
        };
        let node = &cap[..end];
        if !node.ends_with(">근") {
            continue;
        }
        let y = node
            .split_once("translate(")
            .and_then(|(_, rest)| rest.split_once(')'))
            .and_then(|(args, _)| args.split(',').nth(1))
            .and_then(|v| v.trim().parse::<f64>().ok())
            .or_else(|| {
                node.split_once("y=\"")
                    .and_then(|(_, rest)| rest.split_once('"'))
                    .and_then(|(v, _)| v.parse::<f64>().ok())
            });
        if let Some(y) = y {
            title_y = Some(title_y.map_or(y, |t: f64| t.min(y)));
        }
    }
    let title_y = title_y.expect("제목 글자가 있어야 한다");
    assert!(
        title_y < 200.0,
        "제목이 쪽 상단 대역에 있어야 한다 (한글 135.4, 결함 시 338.7): {title_y:.1}"
    );
}

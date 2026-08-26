//! [Issue #5820 축1 부분] 이어지는 partial 로 시작하는 쪽에서 레이아웃이 vpos
//! 기준(base)을 전면 리셋한 뒤, 나중에 lazy 역산이 빈 문단 trailing-ls bridge 로
//! base 를 ls 만큼 낮춰 쪽 전체 저장-vpos 스냅이 +ls(11.2px) 밀렸다 — 조판
//! 패스는 같은 자리에서 텍스트-연속 역산으로 올바른 base(67902)를 얻어 두 패스가
//! 발산(#5801 코어 클래스). 156560092 2쪽 글상자(Rect) y 400.1 vs 한글 2022
//! 358.3 중 +11.2px 성분(잔여 +30.6 은 NEO 저장 사다리 ↔ 2022 신선 조판 발산
//! 으로 별도 축).
//!
//! 수정: 쪽 첫 항목이 연속 partial 이면 base 를 그 연속 줄의 저장 vpos 로 직접
//! 시드한다(빈 문단 bridge 자체를 고치는 접근은 복학원서·#4514 계열 keep-핀
//! 4건이 반증 — 전량 게이트 실측 3회).
//!
//! 결함 상태에서는 Rect y ≈ 400.1 로 어서션이 실패한다(수정 후 388.9 =
//! 저장 사다리 함의 좌표와 정합).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5820/156560092_ecard_meeting_press.hwpx";

#[test]
fn issue_5820_partial_page_start_seeds_vpos_base() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 2, "한글 2022 정본은 2쪽이다");

    let svg = core.render_page_svg_native(1).expect("page 2 svg");
    // 글상자(Rect, w≈342.6) 의 y — 저장 사다리 함의 388.9. 결함 시 400.1(+11.2).
    let rect_y = shape_rect_y(&svg).expect("글상자 rect");
    assert!(
        (385.0..=392.0).contains(&rect_y),
        "글상자 y 는 저장 사다리 함의 좌표(388.9)여야 한다 (결함 시 400.1): {rect_y:.1}"
    );
}

/// w 340~346 인 rect 의 y (글상자 프레임).
fn shape_rect_y(svg: &str) -> Option<f64> {
    for chunk in svg.split("<rect ").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..end];
        let (Some(w), Some(y)) = (attr(head, "width"), attr(head, "y")) else {
            continue;
        };
        if (340.0..=346.0).contains(&w) {
            return Some(y);
        }
    }
    None
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

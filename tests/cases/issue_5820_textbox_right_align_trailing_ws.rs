//! [Issue #5820 축3] 글상자(drawText) 안 오른쪽 정렬 문단의 인라인 로고가
//! 말미 공백 폭만큼 좌측으로 이탈 — [로고A][로고B][공백5] RIGHT 문단에서
//! shape_layout 의 인라인 배치가 말미 공백(32.7px)을 정렬 폭에 포함했다.
//! 한글 2022 실측(156560092 2쪽): 로고 B 우변 여백(글상자 상자 상대) 4.1px,
//! rhwp 는 36.8px. 한글은 셀 밖 오른쪽 정렬에서 말미 공백을 제외한다 —
//! paragraph_layout 의 기존 Right 제외 규칙과 같은 계약을 글상자 인라인
//! 배치에도 적용한다.
//!
//! 결함 상태에서는 로고 B x=559.3 으로 어서션이 실패한다(수정 후 592.3 —
//! 우변 여백 3.8px, 한글 4.1px 정합).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5820/156560092_ecard_meeting_press.hwpx";

#[test]
fn issue_5820_textbox_right_align_excludes_trailing_spaces() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 2, "한글 2022 정본은 2쪽이다");

    let svg = core.render_page_svg_native(1).expect("page 2 svg");
    // 로고 B (w≈119.3) — 우변이 글상자 안쪽 우단(711.6)에 닿아야 한다.
    let logo_b_x = image_x_by_width(&svg, 119.3).expect("로고 B image");
    assert!(
        (588.0..=596.0).contains(&logo_b_x),
        "글상자 RIGHT 정렬 로고는 말미 공백을 제외하고 우측 정렬돼야 한다 \
         (한글 우변 여백 4.1px ≈ x 592.3, 결함 시 559.3): {logo_b_x:.1}"
    );
}

/// 폭이 target±2 인 <image> 의 x.
fn image_x_by_width(svg: &str, target_w: f64) -> Option<f64> {
    for chunk in svg.split("<image ").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..end.min(400)];
        let (Some(x), Some(w)) = (attr(head, "x"), attr(head, "width")) else {
            continue;
        };
        if (w - target_w).abs() <= 2.0 {
            return Some(x);
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

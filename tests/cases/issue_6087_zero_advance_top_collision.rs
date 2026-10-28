//! [Issue #6087] 문서 첫머리의 높이 0 빈 문단(구역/단 정의 컨트롤 + 줄간격 0%:
//! lh 1300 + ls −1300 = 전진 0) 직후 pi=1 의 저장 vpos=0 을
//! `stored_vpos_top_collision`(#5907)이 쪽 경계로 읽어 **완전한 빈 1쪽**을
//! 만들고 의결서 표지가 2쪽으로 밀렸다 — 총 14쪽 vs 한글 2020 13쪽
//! (`samples/issue6060/30307_local_service_reform.hwp`, #6060 과 같은 재현물).
//!
//! 수정: 앞 문단의 저장 전진이 0(lh+ls≤0)이면 쪽을 점유하지 않으므로 충돌의
//! 전제("앞 문단이 단 맨 위 한 줄을 차지했다")가 성립하지 않는다 — 판정에서
//! 제외. #5907 의 원 증거 p122(전진 1600/22838 문단들의 연쇄 단독 쪽)는
//! 전진 > 0 이라 불변(구역-전용/무컨트롤 판별자는 p122 핀이 반증해 기각).
//!
//! 결함 상태에서는 14쪽 + 1쪽이 백지(텍스트 0)로 두 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6060/30307_local_service_reform.hwp";

#[test]
fn issue_6087_zero_advance_head_does_not_split_blank_first_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(
        core.page_count(),
        13,
        "한글 2020 정본은 13쪽이다 (결함 시 14쪽)"
    );

    // 1쪽 상단부터 의결서 표지가 조판되어야 한다 (결함 시 1쪽은 백지).
    let p1 = core.render_page_svg_native(0).expect("page 1 svg");
    let head: String = text_glyphs_in_band(&p1, 0.0, 200.0);
    for needle in ['국', '민', '권', '익'] {
        assert!(
            head.contains(needle),
            "1쪽 상단에 표지 '국민권익위원회' 글리프({needle})가 있어야 한다 (결함 시 백지): {head:?}"
        );
    }
}

fn text_glyphs_in_band(svg: &str, y_min: f64, y_max: f64) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(y) = attr(&chunk[..tag_end], "y") else {
            continue;
        };
        if y < y_min || y > y_max {
            continue;
        }
        if let Some(close) = chunk[tag_end + 1..].find("</text>") {
            out.push_str(&chunk[tag_end + 1..tag_end + 1 + close]);
        }
    }
    out
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

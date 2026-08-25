//! [Issue #6086] 어울림(Square) 개체 옆 빈 문단의 저장 lineseg 가 **같은 vpos
//! 의 좌/우 분할 2세그**인데 rhwp 가 세로 2줄로 적층해 높이를 ×2 계상 — 빈
//! 문단 12개(+288px)가 3쪽 예산을 넘겨 6개가 4쪽으로 넘치고 상단에 ~297pt
//! 공백, 총 16쪽 vs 한글 2020 15쪽
//! (`samples/issue6086/30098_resident_registration_reform.hwp`).
//!
//! 수정: 같은 vpos + 다른 column_start 의 연속 저장 세그(수평 분할)는 같은
//! 시각적 줄이므로 뒤 세그의 높이를 계상하지 않는다(`format_paragraph_for_flow`).
//! #6035 의 쪽-리셋 동일-vpos 쌍은 column_start/폭이 같아 게이트 밖이다.
//!
//! 결함 상태에서는 16쪽 + 4쪽 상단이 빈 문단(일반현황이 쪽 중간 388pt)으로
//! 세 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6086/30098_resident_registration_reform.hwp";

#[test]
fn issue_6086_lr_split_segs_count_as_one_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(
        core.page_count(),
        15,
        "한글 2020 정본은 15쪽이다 (결함 시 16쪽)"
    );

    // 3쪽 — 추진경과(항목 포함)까지 3쪽 안에 들어와야 한다 (결함 시 4쪽으로 밀림).
    let p3 = core.render_page_svg_native(2).expect("page 3 svg");
    let p3_text = all_text(&p3);
    assert!(
        p3_text.contains("추진경과")
            || (p3_text.contains('추')
                && p3_text.contains('진')
                && p3_text.contains('경')
                && p3_text.contains('과')),
        "3쪽에 '추진경과' 절이 있어야 한다"
    );

    // 4쪽 — 'Ⅱ 일반현황' 장 제목이 상단 여백 바로 아래(y<140px)에서 시작해야
    // 한다 (결함 시 상단 ~297pt 공백 뒤 쪽 중간에서 시작).
    let p4 = core.render_page_svg_native(3).expect("page 4 svg");
    let head = text_glyphs_in_band(&p4, 0.0, 140.0);
    for needle in ['일', '반', '현', '황'] {
        assert!(
            head.contains(needle),
            "4쪽 상단(140px 안)에 '일반현황' 글리프({needle})가 있어야 한다 (결함 시 쪽 중간 388pt): {head:?}"
        );
    }
}

fn all_text(svg: &str) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        if let Some(close) = chunk[tag_end + 1..].find("</text>") {
            out.push_str(&chunk[tag_end + 1..tag_end + 1 + close]);
        }
    }
    out
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

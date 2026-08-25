//! [Issue #6063] HWPX 문단 중간 저장 vpos 되감김(쪽 경계 신호)을 흐름 드리프트
//! 필터가 버리면 제10조 꼬리 세 줄이 5쪽 본문 칸 밖(+20~+83px)에 그려진다는
//! 보고의 회귀 핀 (`samples/issue1880_anchor_stack_sb_convert.hwpx`).
//!
//! 검증 결과 현행 devel(242c104bd 이후)에서는 재현되지 않는다 — 보고 시점의
//! integration 조합(devel + 오픈 PR 12건)에서만 나타났던 형상이다. 이 핀은
//! 그 계약을 고정한다: 5쪽 본문 줄은 Body 바닥(1028px) 안이고, 되감긴 제10조
//! 꼬리는 6쪽 상단이 소유한다. #6031(sb-누락 사다리 이중 커서)과 같은 축이라
//! 재발 시 이 두 어서션이 즉시 잡는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue1880_anchor_stack_sb_convert.hwpx";

#[test]
fn issue_6063_rewound_tail_stays_on_next_page_inside_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 5쪽 — 모든 글줄 baseline 이 본문 바닥(1028px) 안이어야 한다
    // (보고된 결함: 제10조 세 줄이 1048.3/1079.5/1110.7px).
    let p5 = core.render_page_svg_native(4).expect("page 5 svg");
    let p5_max = max_text_baseline(&p5);
    assert!(
        p5_max <= 1028.5,
        "5쪽 글줄이 본문 바닥(1028px) 안이어야 한다 (보고 결함 시 1110.7px): {p5_max:.1}"
    );

    // 6쪽 — 되감긴 '제10조(휴대용 저장매체의 분실…' 꼬리는 6쪽 상단 소유다.
    let p6 = core.render_page_svg_native(5).expect("page 6 svg");
    let head = text_glyphs_in_band(&p6, 0.0, 160.0);
    for needle in ['제', '휴', '분'] {
        assert!(
            head.contains(needle),
            "6쪽 상단에 '제10조(휴대용 저장매체의 분실…' 글리프({needle})가 있어야 한다: {head:?}"
        );
    }
}

fn max_text_baseline(svg: &str) -> f64 {
    let mut max_y: f64 = 0.0;
    for chunk in svg.split("<text").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        if let Some(y) = attr(&chunk[..end], "y") {
            max_y = max_y.max(y);
        }
    }
    max_y
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

//! [Issue #5584] 자리차지 표 앞에 있어야 할 제목이 표 **맨 아래**에 그려진다
//! (00072 국민취업지원제도 별표 1, 3232693).
//!
//! 근인: RowBreak 자리차지 표 호스트의 텍스트를 마지막 조각 뒤로 미루는 계약
//! (`defer_visible_rowbreak_host_text`)이 이 형상에도 걸렸다. 그 계약은 표
//! **아래**에 놓이는 서명란·발신명의 호스트를 위한 것인데, 여기서는 저장 기하가
//! 제목이 표 위임을 증언한다(저장 줄 vpos 3420 < 표 vertOffset 4129) — 한글
//! 2022 PDF 오라클도 제목을 1쪽 y=121.95 에 둔다.
//!
//! 수정: 호스트의 저장 줄이 **전부** 표 세로 오프셋보다 위면 지연하지 않는다
//! (`stored_host_lines_precede_float`). 수정 후 제목 1쪽 y=121.2.
//! 10k COM-free 쪽수 A/B 회귀 0.
//!
//! 잔여: 원인 ②(조각이 한 줄만 담고 쪽을 끊어 rhwp 5쪽 vs 한글 4쪽)는 측정
//! 통일 축으로 이슈에 남긴다.
//!
//! 픽스처는 원본 HWPX 구역0 문단 0..3 절단 + 스텁(14KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5584/float_host_title_above_table.hwpx";
const TITLE: &str = "취업취약계층";

#[test]
fn issue_5584_float_host_title_renders_above_table() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 1쪽에 제목이 있어야 하고, 표 첫 괘선(≈156.2)보다 위여야 한다.
    let svg = core.render_page_svg_native(0).expect("page 1 svg");
    let title_y =
        first_text_y(&svg, TITLE).expect("제목이 1쪽에 그려져야 한다 (결함 시 마지막 쪽 표 아래)");
    assert!(
        (100.0..=140.0).contains(&title_y),
        "제목이 한글 좌표(121.95) 근방이어야 한다: {title_y:.1}"
    );

    // 마지막 쪽에는 제목이 남아 있으면 안 된다(중복 그리기 방지).
    let last = core.page_count().saturating_sub(1);
    let last_svg = core.render_page_svg_native(last).expect("last page svg");
    assert!(
        first_text_y(&last_svg, TITLE).is_none(),
        "마지막 쪽에 제목이 중복으로 남았다"
    );
}

/// 글자 단위 `<text>` 를 잇대어 첫 등장 y 를 찾는다.
fn first_text_y(svg: &str, needle: &str) -> Option<f64> {
    let first_char = needle.chars().next()?;
    for chunk in svg.split("<text ").skip(1) {
        let head = &chunk[..chunk.find('>')?];
        let body_end = chunk.find("</text>")?;
        let body = &chunk[head.len() + 1..body_end];
        if !body.starts_with(first_char) {
            continue;
        }
        // 이어지는 글자들이 needle 을 이루는지 확인.
        let rest: String = svg[svg.find(chunk)?..]
            .split("<text ")
            .take(needle.chars().count() + 2)
            .filter_map(|c| {
                let h = c.find('>')?;
                let e = c.find("</text>")?;
                Some(c[h + 1..e].to_string())
            })
            .collect();
        if !rest.starts_with(needle) {
            continue;
        }
        let y = head
            .split_once("y=\"")
            .and_then(|(_, r)| r.split_once('"'))
            .and_then(|(v, _)| v.parse::<f64>().ok())
            .or_else(|| {
                head.split_once("translate(")
                    .and_then(|(_, r)| r.split_once(')'))
                    .and_then(|(args, _)| args.split(',').nth(1)?.trim().parse::<f64>().ok())
            })?;
        return Some(y);
    }
    None
}

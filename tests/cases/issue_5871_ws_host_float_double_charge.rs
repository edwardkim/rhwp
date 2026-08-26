//! [Issue #5871] 공백만 있는 host 문단의 자리차지 표가 저장 줄상자와 이중 계상돼
//! 표가 제 높이만큼 아래로 밀린다 (10895 [별표 3] 8·9쪽, 쪽 밖 이탈).
//!
//! 근인: 같은 함수의 두 술어가 "글자가 있다" 를 다르게 정의했다 —
//! `is_visible_para_float` 는 `para_has_non_whitespace_text`(공백은 글자가 아님),
//! pre-text 판정은 `!para.text.is_empty()`(공백도 글자). 공백 한 칸짜리 host 가
//! 그 틈에 빠져, 저장 줄상자(lh 가 이미 표 높이+바깥여백 = 24276HU)를 표 앞
//! 텍스트로 한 번 방출하고 그 아래에 같은 표를 다시 그렸다.
//!
//! 수정: 저장 줄상자가 **이미 표를 담고 있다는 증거**(첫 저장 줄 lh ≥ 표 높이 +
//! 위·아래 바깥여백)가 있을 때만 공백 host 를 무텍스트로 본다. 증거가 없으면
//! 종전 경로 — "공백=무텍스트" 로만 넓히면 서식 문서가 +1쪽 된다(19952675, 한글 6).
//!
//! 원본 8쪽 실측: 둘째 표 1012.7~1331.3(본문 하한 1028.1 초과) → 671.0~989.6
//! (한글 667.7~983.6). 10k 쪽수 A/B better 1 / worse 0.
//!
//! 픽스처는 원본 HWP5 구역0 문단 99..108([별표 3] 네 항목) 절단(12KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5871/ws_host_float_double_charge.hwp";

#[test]
fn issue_5871_whitespace_host_float_is_not_double_charged() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 폭 200px 이상 가로 괘선의 y 를 모은다.
    let mut rules: Vec<f64> = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let (Some(x1), Some(x2), Some(y1), Some(y2)) = (
            attr(head, "x1=\""),
            attr(head, "x2=\""),
            attr(head, "y1=\""),
            attr(head, "y2=\""),
        ) else {
            continue;
        };
        if (y1 - y2).abs() < 0.1 && (x2 - x1).abs() > 200.0 {
            rules.push(y1);
        }
    }
    assert!(rules.len() >= 8, "표 괘선을 찾아야 한다: {}", rules.len());

    // 이중 계상 시 둘째 표가 제 높이(≈318px)만큼 밀려 본문 하한을 넘는다.
    let lowest = rules.iter().copied().fold(f64::MIN, f64::max);
    assert!(
        lowest < 1000.0,
        "표가 저장 줄상자와 이중 계상돼 아래로 밀렸다 (결함 시 1331.3): {lowest:.1}"
    );

    // 둘째 표는 첫 표 바로 뒤(제목 줄 한 칸)에서 시작해야 한다 — 이중 계상이면
    // 제 높이(≈318px)만큼 아래로 밀려 이 대역이 빈다.
    assert!(
        rules.iter().any(|y| (520.0..=580.0).contains(y)),
        "둘째 표가 첫 표 바로 뒤에서 시작해야 한다 (결함 시 ≈318px 아래): {rules:?}"
    );
}

fn attr(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}

//! Issue #3765: 다단 zone 두 개가 한 쪽에 겹쳐 얹힌다.
//!
//! ## 근인
//!
//! Task #853 가드는 새 zone 이 쪽 하단 가까이서 시작하면 다음 쪽으로 넘긴다. 그런데
//! 직전 zone 의 높이를 **저장 사다리**에서 뽑는다.
//!
//! ```rust
//! if max_vpos_px <= available { max_vpos_px } else { st.current_height }
//! ```
//!
//! 사다리는 **한글의 쪽 경계** 기준이다. 한글이 이미 쪽을 끊은 자리에서는 직전 문단의
//! vpos 가 다음 쪽 상단 값이 되어, 이 쪽이 실제로 소비한 높이를 크게 밑돈다.
//!
//! ```text
//! 2990099 주파수 분배표 115쪽
//!   pi=201 vpos 903.1px → pi=202 vpos 5.3px   ← 한글의 쪽 경계
//!   zone 전환 시점 사다리값 76px  vs  이 쪽 실소비 867.4px
//!
//!   단 0 used=867.4  +  단 1 used=789.9  =  1657.3px   (본문 876.9px)
//!   → 745.7px 이 쪽 밖으로, 쪽수도 289 (한글 290)
//! ```
//!
//! 수정: 사다리값과 흐름 누적 중 **큰 쪽**을 쓴다. 흐름 누적은 이 쪽에 실제로 놓인
//! 양이다. `max_vpos_px > available` 인 문서(별지 서식처럼 stored vpos 가 섹션 누적
//! 좌표)는 종전대로 `st.current_height` 폴백을 유지해 #2019 의 완화가 그대로 산다.
//!
//! ## 계약
//!
//! 표본은 앞 25줄로 쪽을 채운 뒤 사다리를 되감고(400 HU) zone 전환을 둔다.
//!
//! ```text
//! 수정 전  1쪽에 zone 2개 (896.0 + 672.0 = 1568.0px, 본문 933.6)  쪽 밖 119글자·521.6px
//! 수정 후  2쪽으로 분리                                            쪽 밖   0글자
//! ```
//!
//! **되감기 기준값이 0 이 아니라 400 HU 인 것이 중요하다** — 정확히 0 이면 rhwp 의
//! vpos-reset 가드가 정상 발동해 쪽을 끊어 버려 이 경로를 아예 타지 않는다(실제 문서
//! pi=202 도 400 이다).
#![cfg(not(target_arch = "wasm32"))]

const SAMPLE: &str = "samples/issue3765/zone_switch_ladder_understates_page.hwpx";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// SVG 한 쪽에서 (뷰박스 높이, 쪽 높이를 넘은 글자의 y) 를 뽑는다.
fn out_of_page(svg: &str) -> (f64, Vec<f64>) {
    let height = svg
        .split("height=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let mut over = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(rest) = chunk.split(" y=\"").nth(1) else {
            continue;
        };
        let Some(y) = rest.split('"').next().and_then(|s| s.parse::<f64>().ok()) else {
            continue;
        };
        if y > height + 2.0 {
            over.push(y);
        }
    }
    (height, over)
}

/// zone 전환은 쪽이 이미 찼으면 다음 쪽에서 시작한다.
#[test]
fn zone_switch_starts_a_new_page_when_the_current_one_is_full() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    let pages = doc.page_count();
    assert!(
        pages >= 2,
        "쪽이 {pages}개다 — 두 zone 이 한 쪽에 겹쳤다는 뜻이다. \
         zone 전환 가드가 직전 zone 을 사다리로 재면 실소비를 밑돌아 이렇게 된다."
    );

    let mut offenders = Vec::new();
    for p in 0..pages {
        let svg = doc.render_page_svg(p).expect("SVG 렌더");
        let (h, over) = out_of_page(&svg);
        assert!(h > 0.0, "{}쪽 뷰박스 높이를 읽지 못했다", p + 1);
        for y in over {
            offenders.push(format!("{}쪽 y={y:.1} (초과 {:.1}px)", p + 1, y - h));
        }
    }

    assert!(
        offenders.is_empty(),
        "쪽 높이를 넘은 글자가 {}개 있다 (수정 전 실측 119개·최대 521.6px).\n  {}\n\
         두 zone 이 한 쪽에 겹쳐 얹히면 뒤 zone 이 통째로 쪽 밖으로 나간다 — 나간 글자는 \
         텍스트 추출에 남아 있어 텍스트/IR diff 로는 잡히지 않는다.",
        offenders.len(),
        offenders
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// 표본이 계약을 시험하는 형태인지 못박는다 — 사다리가 되감기되 **0 이 아니어야** 한다.
#[test]
fn the_fixture_rewinds_the_ladder_without_resetting_to_zero() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::parser::parse_document(&bytes).expect("파싱");
    let section = doc.sections.first().expect("구역");

    let vposes: Vec<i32> = section
        .paragraphs
        .iter()
        .filter_map(|p| p.line_segs.first().map(|s| s.vertical_pos))
        .collect();
    let rewind_at = vposes
        .windows(2)
        .position(|w| w[1] < w[0])
        .expect("사다리가 되감기지 않는다 — 표본이 계약을 시험하지 못한다");
    let after = vposes[rewind_at + 1];
    assert!(
        after > 0,
        "되감기 기준값이 0 이다 (vpos={after}) — 그러면 vpos-reset 가드가 먼저 쪽을 끊어 \
         zone 전환 경로를 타지 않는다. 실제 문서(2990099 pi=202)는 400 HU 다."
    );

    let has_coldef = section.paragraphs.iter().any(|p| {
        p.controls
            .iter()
            .any(|c| matches!(c, rhwp::model::control::Control::ColumnDef(_)))
    });
    assert!(
        has_coldef,
        "단정의(ColumnDef) 컨트롤이 없다 — zone 전환이 일어나지 않는다"
    );
}

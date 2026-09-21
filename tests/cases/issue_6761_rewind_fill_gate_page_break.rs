//! [#6761] 문단 경계의 저장 `vpos` 되감김을 쪽 경계로 안 써서 한 쪽이 두 쪽 몫을 담는다.
//!
//! ## 무엇이 문제였나
//!
//! 저장 사다리가 문단 경계에서 되감기면 한/글이 거기서 쪽을 끊었다는 뜻이다. rhwp 는 그
//! 신호를 `stored_vpos_rewind_break` 로 받는데, 관문이 **"쪽이 90% 이상 찼는가"**
//! (`STORED_VPOS_REWIND_MIN_FILL`) 하나였다.
//!
//! 한/글은 **덜 찬 쪽도** 끊는다 — 다음 블록(그림·표)이 남은 여백에 안 들어가면 그렇다.
//! 그 결정이 바로 사다리의 되감김인데, 채움률 관문이 그걸 막았다.
//!
//! ```text
//!   1480000-201900042  13쪽(0-기반 12)
//!     pi=132  vpos 35300   ← 채움 481.6px / 가용 895.7px = 53.8%
//!     pi=133  vpos   600   ← 되감김. 쪽 위쪽 띠에서 다시 시작 = 쪽 경계 인코딩
//!   수정 전   pi=133 을 같은 쪽에 얹는다 → 13쪽이 정본 13·14쪽을 통째로 담는다
//!
//!   그 결과는 "빽빽한 쪽" 이 아니라 **소실**이다 — 정본 14쪽 몫이 앞 쪽의 큰 그림 뒤로
//!   들어간다(render tree 실측, 0-기반 12쪽):
//!     텍스트 `최종안 제시 및 보고 자료`  y 621.9..637.9
//!     마크 원안 표                        y 674.8..979.4
//!     그 위를 덮는 그림                    y 623.2..1038.6   ← 둘 다 이 아래
//! ```
//!
//! ## 기대값의 독립 근거
//!
//! 한컴 정본(MCP engine 2024 / `13.0.0.3901` 변환, **103쪽**, sha256
//! `ba4dc3e2ba00453d422f7899fbcbf650f5fa329020ffae5a70a1f84d347a462c`)의 14쪽이
//! `최종안 제시 및 보고 자료: Design B 최종 제안 및 결정` 으로 **시작한다**. 즉 한/글은
//! 그 문단 앞에서 쪽을 끊는다. 저장 사다리의 `vpos 600` 이 같은 말을 한다.
//!
//! ## 어떻게 갈랐나 — 채움률이 아니라 **위치 일치**
//!
//! 되감김 자체는 같은 쪽 안의 부분 후퇴도 포함하므로 그대로 믿으면 안 된다. 되감김 직전
//! 문단의 **저장 끝**(`vpos + line_height + line_spacing`)이 지금 흐름 위치와 한 줄 안에서
//! 같을 때만 인정한다 — 사다리가 말한 그 경계가 지금 이 자리라는 뜻이다.
//!
//! ```text
//!   pi=132 저장끝 496.3px  →  pi=133 조판 481.6px   Δ  14.7px  ← 같은 자리 · 끊는다
//!   pi=202 저장끝 886.3px  →  pi=203 조판  12.8px   Δ 873.5px  ← 다른 쪽 · 안 끊는다
//! ```
//!
//! 뒤쪽이 반례다. `pi=203` 도 `vpos 600` 으로 되감기지만 한/글은 그 문단을 **쪽 중간**에
//! 둔다(정본 74쪽, 쪽 머리에서 128자 뒤). 위치가 어긋나므로 이 판정은 그 자리를 거른다.
//!
//! ## 이 수정이 닫지 않는 것
//!
//! 이 문서의 쪽수는 정본 103 / rhwp 104 → **105** 다. 늘어난 한 쪽이 정본 14쪽의 복원이고,
//! 남은 두 쪽 격차는 **표 제목행만 남는 빈 쪽 2건**(`#6976` 계열 표 분할 축)이라 여기서
//! 다루지 않는다. 쪽 정합은 24 → 65쪽으로 는다(정본과 같은 자리에 있는 쪽 수).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// `#6782` 가 쓰는 것과 같은 실물 원본이다 — 이 결함의 신고 문서이기도 하다.
const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
/// 정본 14쪽이 이 문단으로 시작한다(0-기반 13).
const BOUNDARY_PAGE_INDEX: usize = 13;
/// 되감김이 쪽 경계로 쓰여야 하는 문단의 머리.
const BOUNDARY_HEAD: &str = "최종안 제시 및 보고 자료";

fn page_texts() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let pages = core.page_count();
    (0..pages)
        .map(|page| core.extract_page_text_native(page).unwrap_or_default())
        .collect()
}

/// 되감김이 말한 쪽 경계에서 실제로 쪽이 끊긴다.
///
/// 수정 전에는 이 문단이 13쪽(0-기반 12) 꼬리에 얹혀 그 쪽이 정본 두 쪽 몫을 담았다.
#[test]
fn paragraph_boundary_rewind_starts_the_page_hancom_starts() {
    let pages = page_texts();
    let page = pages
        .get(BOUNDARY_PAGE_INDEX)
        .expect("14쪽이 있어야 한다")
        .replace(char::is_whitespace, "");
    let head: String = BOUNDARY_HEAD
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    assert!(
        page.contains(&head),
        "정본 14쪽은 `{BOUNDARY_HEAD}` 로 시작한다. 수정 전에는 이 문단이 13쪽 꼬리에 \
         얹혀 있었다. 14쪽 머리: {}",
        page.chars().take(40).collect::<String>()
    );
    let offset = page.find(&head).unwrap_or(usize::MAX);
    assert!(
        offset <= head.len(),
        "그 문단은 14쪽 **머리**여야 한다 — 쪽 안 위치 {offset}"
    );

    // 앞 쪽은 그 문단을 더 이상 담지 않는다(합쳐 그리던 자리).
    let prev = pages[BOUNDARY_PAGE_INDEX - 1].replace(char::is_whitespace, "");
    assert!(
        !prev.contains(&head),
        "13쪽이 아직 `{BOUNDARY_HEAD}` 를 담고 있다 — 쪽 경계가 승격되지 않았다"
    );
}

/// 반례 — 위치가 어긋난 되감김은 쪽 경계로 쓰지 않는다.
///
/// `pi=203`(`표시면의 면적이 50 cm2 …`)도 `vpos 600` 으로 되감기지만 한/글은 쪽 중간에
/// 둔다. 이 문단이 어떤 쪽의 **머리**가 되면 판정이 너무 넓어진 것이다.
#[test]
fn rewind_whose_flow_position_disagrees_does_not_start_a_page() {
    let pages = page_texts();
    let needle: String = "표시면의 면적이 50"
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let hit = pages
        .iter()
        .enumerate()
        .find_map(|(index, text)| {
            let flat = text.replace(char::is_whitespace, "");
            flat.find(&needle).map(|offset| (index, offset, flat))
        })
        .expect("그 문단을 담은 쪽");

    assert!(
        hit.1 > needle.len(),
        "이 되감김은 쪽 경계가 아니다(정본은 쪽 중간에 둔다). 그런데 {}쪽 머리에 왔다",
        hit.0 + 1
    );
}

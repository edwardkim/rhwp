//! [Issue #5585 국소형 ②] 한 문단이 품은 **형제 표**마다 문단 앵커 오프셋을 다시 물어
//! 예산을 깎던 결함의 가드.
//!
//! `vert_rel_to = Para` 의 양수 `vertOffset` 은 **문단 앵커로부터의** 거리다. 한 문단이
//! 표 여러 개를 품고 그것들이 쪽을 하나씩 차지하면, 두 번째 이후 표에게 그 오프셋은
//! **이미 앞 형제가 소진한** 값이다. 그런데 조판기는 쪽마다 다시 물어 `avail_for_rows`
//! 를 그만큼 깎았다.
//!
//! `02. 지표정의서- 주요정책부문` 실측 — 문단 `pi=72` 가 `11x21` 표 16개를 품는다.
//!
//! ```text
//!   vert_off  avail_for_rows   표 높이   결과
//!     0.0        742.7          734.8    통째 (ci=8)
//!    11.3        731.4          729.7    통째 (ci=1)
//!    11.3        731.4         ~733.7    분할! (ci=6·7·9·10·13)
//! ```
//!
//! **높이 판정이 아니다** — 34.5px 초과도 통과시키는 조판기가 25px 남는 표를 쪼갠다.
//! 갈림은 오직 `vert_off` 11.3px 이다. 그 다섯 개가 쪼개지며 65.7px 짜리 꼬리 쪽 5장이
//! 생겼다(rhwp 91쪽 / 한/글 2024 86쪽).
//!
//! 한/글 2024 실측(2-up 오라클을 논리 쪽으로 환산): 이 표들의 상단이 **15.60 / 15.31pt**
//! — 본문 상단 그대로다. rhwp 는 **22.64pt**(= +11.3px). 한/글은 이 오프셋을 안 문다.
//!
//! ⚠ 좁힘은 **앞선 형제 표가 있는 경우만**이다. 표 하나짜리 문단(`issue_2287` 핀)까지
//! 넓히면 그 문서에 조각 공백화(sliver)가 생긴다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/prism_downloads/보건복지부/
///  1351000-201000123_D0150004-2-002_02. 지표정의서- 주요정책부문.hwp`
///
/// ⚠ `.hwp` 를 `samples/` 에 넣으면 `ir_field_sweep_baseline` 이 `samples/` 전체를 스윕해
/// 무관한 직렬화 발산을 끌고 온다. `RHWP_ISSUE5585B_SAMPLE` 로 경로를 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE5585B_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        concat!(r"C:\Users\planet\hwpdocs_10k_share", r"\prism_downloads\보건복지부"),
        concat!(r"D:\hwpdocs_10k_share", r"\prism_downloads\보건복지부"),
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.contains("02. 지표정의서") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

/// 쪽수는 한/글 2024 와 같은 **86쪽**이어야 한다 — 형제 표마다 앵커 오프셋을 다시 물면
/// 표 다섯 개가 쪼개져 91쪽이 된다.
#[test]
fn sibling_tables_do_not_recharge_the_paragraph_anchor_offset() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let pages = core.page_count();
    assert_eq!(
        pages, 86,
        "쪽수는 한/글 2024 와 같은 86쪽이어야 한다 — #5585 회귀. \
         형제 표마다 앵커 오프셋(11.3px)을 다시 물면 91쪽이 된다 (got {pages})"
    );
}

/// 65.7px 짜리 꼬리 조각 쪽이 없어야 한다 — 본문은 744.6px 다.
#[test]
fn no_sliver_tail_pages_remain() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let mut slivers = Vec::new();
    // 문단 pi=72 구간(종전 p52~p72)만 본다 — 앞쪽에는 정상적으로 짧은 쪽이 있다.
    for page in 45..core.page_count().min(75) {
        let Ok(tree) = core.build_page_render_tree(page as u32) else {
            continue;
        };
        let mut deepest = 0.0f64;
        fn walk(n: &rhwp::renderer::render_tree::RenderNode, deepest: &mut f64) {
            if matches!(
                n.node_type,
                rhwp::renderer::render_tree::RenderNodeType::TextRun(_)
            ) {
                *deepest = deepest.max(n.bbox.y + n.bbox.height);
            }
            for c in &n.children {
                walk(c, deepest);
            }
        }
        walk(&tree.root, &mut deepest);
        if deepest > 0.0 && deepest < 150.0 {
            slivers.push((page + 1, (deepest * 10.0).round() / 10.0));
        }
    }
    assert!(
        slivers.is_empty(),
        "본문 744.6px 인데 150px 도 못 채운 꼬리 조각 쪽이 있다 — #5585 회귀. {slivers:?}"
    );
}

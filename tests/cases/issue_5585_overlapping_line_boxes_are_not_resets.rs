//! [Issue #5585 국소형 ①] 겹치는 줄 상자를 vpos 되감김으로 오인해 조각을 잘게 끊던
//! 결함의 가드.
//!
//! `cell_units` 의 `reset_before` 는 **"앞 줄 바닥보다 앞선 vpos"** 를 되감김으로 본다.
//! 줄 전진량이 줄 높이보다 작은 사다리(줄 상자가 서로 겹치는 문서)에서는 **평범한 한
//! 걸음**도 그 조건을 만족한다.
//!
//! `148738070_20120829_무학대선건 보도자료.hwp` 실측:
//!
//! ```text
//! p2 vpos  9800 lh 1400 (끝 11200)  →  p3 vpos 10640    840 전진인데 "되감김"
//! p3 vpos 10640 lh 1300 (끝 11940)  →  p4 vpos 10992    352 전진인데 "되감김"
//! p5 vpos 45290 lh 1200 (끝 46490)  →  p6 vpos     0    ← 이것만 진짜 되감김
//! ```
//!
//! 조각 커트가 `hard_break_before` 유닛에서 끊기므로, 933.5px 본문에 47~340px 만 담은
//! 쪽이 12장 생겼다 — rhwp 16쪽 vs 한/글 2024 **7쪽**.
//!
//! 진짜 되감김은 앞 줄의 **시작**보다 뒤로 간다. 그 규칙을 전역으로 세우면 게이트 8건이
//! 깨지므로(`overflow_cell` 2 · `off_canvas` 3 · `issue_2097` · row-cut 단위시험),
//! **겹침 걸음이 3회 이상인 셀**에서만 좁힌다.
//!
//! ⚠ 이 시험은 **쪽 배분 축만** 잠근다. 이 문서 3쪽이 꼬리말로 4.5px 넘치는 것은
//! 조판 계상(809.7px)과 페인트가 어긋나는 별개 축이고, 이슈에 분리해 남겼다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/korea_policy_downloads/148738070_20120829_무학대선건 보도자료.hwp`
///
/// ⚠ `.hwp` 를 `samples/` 에 넣으면 `ir_field_sweep_baseline` 이 `samples/` 전체를
/// 스윕해 무관한 직렬화 발산을 끌고 온다. 그래서 코퍼스에서 찾고, 없으면 건너뛴다.
/// `RHWP_ISSUE5585_SAMPLE` 로 경로를 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE5585_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        concat!(r"C:\Users\planet\hwpdocs_10k_share", r"\korea_policy_downloads"),
        concat!(r"D:\hwpdocs_10k_share", r"\korea_policy_downloads"),
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("148738070") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

/// 쪽수는 한/글 2024 와 같은 **7쪽**이어야 한다 — 겹침 걸음을 되감김으로 세면 16쪽이 된다.
#[test]
fn overlapping_line_steps_do_not_split_the_cell_into_tiny_pages() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let pages = core.page_count();
    assert_eq!(
        pages, 7,
        "쪽수는 한/글 2024 와 같은 7쪽이어야 한다 — #5585 회귀. \
         겹치는 줄 상자를 되감김으로 세면 16쪽이 된다 (got {pages})"
    );
}

/// 쪽 채움이 실제로 회복됐는지 — 본문 933.5px 에 47~340px 만 담은 쪽이 없어야 한다.
#[test]
fn fragment_pages_fill_the_body_instead_of_breaking_early() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    // 2~5쪽은 단일 1×1 표의 조각이다. 종전에는 172.9 / 908.0 / 281.3 / 340.8 px 였다.
    let mut thin = Vec::new();
    for page in 1..core.page_count().min(6) {
        let tree = core
            .build_page_render_tree(page as u32)
            .unwrap_or_else(|_| panic!("{}쪽 render tree", page + 1));
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
        if deepest < 500.0 {
            thin.push((page + 1, deepest));
        }
    }
    assert!(
        thin.is_empty(),
        "본문 933.5px 인데 절반도 못 채운 조각 쪽이 있다 — #5585 회귀. {thin:?}"
    );
}

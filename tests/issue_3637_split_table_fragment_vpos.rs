//! Issue #3637: 분할 표 조각에서 표-직후 vpos 스냅이 셀 원점을 써서 내용이 쪽 밖으로 밀린다.
//!
//! ## 근인
//!
//! `LINE_SEG.vertical_pos` 는 **셀 시작** 기준 누적값이다. `table_partial.rs` 의 표-직후
//! 보정은 그 절대값을 **이 쪽의** `text_y_start` 에 그대로 더했다.
//!
//! ```text
//! let next_vpos_y = text_y_start + hwpunit_to_px(next_seg.vertical_pos, dpi);
//! para_y = para_y.max(next_vpos_y);
//! ```
//!
//! 표가 쪽을 넘나들며 잘린 **연속 조각**에서는 원점이 어긋난다 — 앞 쪽으로 넘어간
//! 부분의 높이만큼 통째로 아래로 밀린다.
//!
//! 실측 (이 표본 2쪽, 조각 시작 vpos = 7160 HU = 95.5px):
//!
//! | 위치 | 수정 전 | 수정 후 |
//! |---|---|---|
//! | `p[19]` 뒤 | 583.1 | 487.7 |
//! | `p[27]` 뒤 | 977.2 | 881.7 |
//!
//! 두 곳 모두 오차가 정확히 95.5px 였다. 그만큼 표 직후에 빈 공간이 생기고, 뒤 내용이
//! 쪽 경계를 넘어가 **어느 렌더 경로에서도 보이지 않았다**.
//!
//! 넘어간 글자는 `export-text` 에도 SVG `<text>` 요소에도 남아 있어 텍스트 diff 나
//! IR diff 로는 잡히지 않는다. 좌표만 쪽 밖일 뿐이다.
//!
//! ## 계약
//!
//! 분할 표 조각 안에서도 글자가 쪽 높이 안에 그려진다.
//!
//! 이 표본은 완전 해소가 아니다. 그래서 "0" 이 아니라 **회귀 상한**으로 고정한다.
//!
//! ```text
//! 수정 전  762자  (2쪽 352 · 3쪽 116 · 4쪽 294)
//! 수정 후  239자  (2쪽 239, 3·4쪽 완전 해소)
//! ```
//!
//! 잔존 239자는 흐름이 저장 사다리보다 촘촘한 **별개 축**이다 — 이 수정 범위 밖이다.
#![cfg(not(target_arch = "wasm32"))]

/// 재현 문서 — 본문 전체가 1×1 표 안에 있고 그 표가 쪽 2~4 로 잘리는 보도자료.
///
/// **저장소에 넣지 않는다.** `samples/` 아래 `.hwp` 는 `ir_field_sweep_baseline` 이
/// 전수 스윕하므로(`HWP5_ROOT = "samples"`), 파일 추가만으로 그 게이트가 깨진다.
/// 기준선을 재생성하면 이 PR 과 무관한 항목까지 끌어들여 진짜 회귀를 가린다.
/// (`samples/issueNNNN/` 배치는 `.hwpx` 만 보호한다 — `HWPX_ROOT` 가 `samples/hwpx` 로
/// 좁기 때문이다.)
///
/// 코퍼스에 문서가 있으면 검증하고, 없으면 건너뛴다.
const CORPUS_DOC: &str =
    r"C:\Users\planet\hwpdocs\korea_policy_downloads\148738070_20120829_무학대선건 보도자료.hwp";

/// 재현 문서 경로를 환경변수로 덮어쓸 수 있다.
const CORPUS_ENV: &str = "RHWP_ISSUE3637_DOC";

/// 수정 후 실측 239자, 수정을 되돌리면 762자.
///
/// 상한은 그 사이(400)로 잡는다 — 0 으로 두면 잔존 별개 축 때문에 늘 실패하고,
/// 762 이상으로 두면 이 수정이 되돌려져도 통과해 회귀를 못 잡는다.
const MAX_OUT_OF_PAGE_GLYPHS: usize = 400;

/// 재현 문서 경로 (없으면 `None` → 테스트 건너뜀).
fn corpus_doc() -> Option<std::path::PathBuf> {
    let p = std::env::var(CORPUS_ENV)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(CORPUS_DOC));
    p.exists().then_some(p)
}

/// SVG 한 쪽에서 쪽 높이를 넘는 `<text>` 개수를 센다.
fn out_of_page_glyphs(svg: &str) -> (f64, usize) {
    let height = svg
        .split("height=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let mut over = 0usize;
    for chunk in svg.split("<text").skip(1) {
        let Some(rest) = chunk.split(" y=\"").nth(1) else {
            continue;
        };
        let Some(y) = rest.split('"').next().and_then(|s| s.parse::<f64>().ok()) else {
            continue;
        };
        // 여유 2px — baseline 이 경계에 걸친 글리프는 결함으로 세지 않는다.
        if y > height + 2.0 {
            over += 1;
        }
    }
    (height, over)
}

/// 분할 표 조각의 내용이 쪽 밖으로 대량 밀려나지 않는다.
#[test]
fn split_table_fragment_keeps_content_inside_page() {
    let Some(path) = corpus_doc() else {
        eprintln!(
            "건너뜀: 재현 문서가 없다. 코퍼스가 있으면 {CORPUS_ENV} 로 경로를 지정하라.\n  {CORPUS_DOC}"
        );
        return;
    };
    let bytes = std::fs::read(&path).expect("재현 문서 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    let page_count = doc.page_count();
    assert!(
        page_count >= 4,
        "쪽이 너무 적다({page_count}) — 표본이 바뀌었는지 확인하라"
    );

    let mut total = 0usize;
    let mut per_page = Vec::new();
    for p in 0..page_count {
        let svg = doc.render_page_svg(p).expect("SVG 렌더");
        let (h, over) = out_of_page_glyphs(&svg);
        assert!(h > 0.0, "{}쪽 뷰박스 높이를 읽지 못했다", p + 1);
        if over > 0 {
            per_page.push(format!("{}쪽 {over}자", p + 1));
        }
        total += over;
    }

    assert!(
        total <= MAX_OUT_OF_PAGE_GLYPHS,
        "쪽 밖에 그려진 글자가 {total}자로 상한 {MAX_OUT_OF_PAGE_GLYPHS}자를 넘는다 ({}).\n\
         분할 표 조각의 표-직후 vpos 보정이 **셀 원점**을 쓰면 연속 조각이 앞부분 높이만큼 \
         밀려 내용이 쪽 밖으로 나간다. 원점은 `cut_units` 가 정한 조각 시작 vpos 여야 한다.",
        per_page.join(", ")
    );
}

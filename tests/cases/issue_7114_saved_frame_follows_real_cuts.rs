//! [#7114] 거대 표 셀을 편집해 저장한 뒤 다시 열어도 조판이 같다.
//!
//! # 무엇이 깨져 있었나
//!
//! 셀 텍스트를 편집하면 줄 구성이 바뀌지만 저장 `LINE_SEG` 사다리가 적어 둔 쪽 경계는
//! 편집 전 줄 자리에 남는다. 조판은 이미 그걸 알고 있다 — 편집 관문이 남긴
//! `render_normalization.table_text_reflowed` 가 그 표의 저장 프레임 꼬리 흡수를 막아
//! 편집 직후 화면은 115쪽으로 옳다. 그러나 그 판단은 메모리에만 있고 파일에 실리지 않아,
//! 저장본을 다시 열면 **조판기 자신이 기각한 프레임**을 믿고 꼬리를 흡수한다.
//!
//! ```text
//!   조각 사슬(114개)은 live·재열기가 같은 유닛 수·같은 높이 — 첫 조각만 다르다
//!     live    0 -> 37   h= 944.0   avail=967.7
//!     재열기  0 -> 38   h= 969.6   avail=967.7   ← 967.7 을 넘겨 표가 통째로 2쪽으로 밀린다
//! ```
//!
//! # 기대값의 출처
//!
//! 한/글 2020 정본 `pdf/task_m100_3820_stage86_wasm_boundary_oracle/issue1949_cell_p5_append_56-2020.pdf`
//! 는 같은 56자 편집 저장본의 출력이며 **115쪽**이고, 1쪽 마지막이 `특히,` · 2쪽 시작이
//! `복합재료로` 다. 이 정본은 텍스트 층이 사실상 비어 있어(115쪽에 816낱말) 쪽별 전수
//! 대조에는 쓸 수 없으므로, 쪽수와 그 경계만 정본 근거로 쓴다.
//!
//! 나머지 쪽은 이슈 완료 기준 ①("저장 전·재열기 후 115쪽과 텍스트를 보존한다")을 그대로
//! 계약으로 삼는다 — **재열기 쪽 구성 == 편집 직후 쪽 구성**. 이건 구현이 계산한 값을
//! 재인용하는 게 아니라 같은 입력의 두 경로가 같아야 한다는 독립 계약이다.
//!
//! # 반례·대조군
//!
//! - 55자(줄바꿈이 일어나지 않는 경계 바로 앞)와 1자는 수정 전에도 재열기가 편집 직후와
//!   같았다. 되쓰기가 과적용되면 여기서 깨진다.
//! - 무편집 저장은 사다리를 건드리면 안 된다 — 재래핑된 표가 없어 되쓰기가 아예 돌지 않는다.
//! - 61자는 이슈가 HWPX 대조군으로 든 길이다. HWP 에서는 56자와 같은 결함을 냈다.
//!
//! # 미검증으로 남기는 범위
//!
//! 쪽 경계가 **중첩 표 안**에 떨어지면 host 셀의 저장 사다리로는 그 자리를 표현할 수 없다
//! — 중첩 표 행 유닛은 host 문단의 한 줄을 공유한다. 그 경계는 뒤의 첫 host 줄로 해소하므로
//! 저장 프레임이 실제 컷보다 늦다. 200자 스트레스 입력에서 그 잔차가 2쪽 남는다(되쓰기 전
//! 113쪽 → 후 2쪽). 중첩 표 자신의 사다리까지 되쓰는 것은 이 변경의 범위가 아니다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
/// 0-based `section=0, parentParagraph=0, control=2, cell=2, cellParagraph=5, offset=130`.
const PATH: (usize, usize, usize, usize, usize, usize) = (0, 0, 2, 2, 5, 130);

fn sample_bytes() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).expect("재현물 읽기")
}

/// 쪽별 본문 낱말 — 쪽 구성 비교용. 쪽수만으로는 귀속을 증명하지 못한다.
fn page_words(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| {
            core.extract_page_text_native(page)
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// 편집 → 저장 → 재열기. `(편집 직후 쪽 구성, 재열기 쪽 구성)`.
fn edit_save_reopen(insert_len: usize) -> (Vec<String>, Vec<String>) {
    let mut core = DocumentCore::from_bytes(&sample_bytes()).expect("문서 로드");
    if insert_len > 0 {
        let text: String = std::iter::repeat_n('1', insert_len).collect();
        core.insert_text_in_cell_native(PATH.0, PATH.1, PATH.2, PATH.3, PATH.4, PATH.5, &text)
            .expect("셀 텍스트 삽입");
    }
    let live = page_words(&core);
    let saved = core.export_hwp_with_adapter().expect("HWP 저장");
    let reopened = DocumentCore::from_bytes(&saved).expect("저장본 재열기");
    (live, page_words(&reopened))
}

fn first_difference(live: &[String], reopened: &[String]) -> Option<String> {
    (0..live.len().max(reopened.len())).find_map(|i| {
        let a = live.get(i).map(String::as_str).unwrap_or("<쪽 없음>");
        let b = reopened.get(i).map(String::as_str).unwrap_or("<쪽 없음>");
        (a != b).then(|| {
            let head = |s: &str| s.split_whitespace().take(6).collect::<Vec<_>>().join(" ");
            format!("{}쪽: 편집직후=[{}] 재열기=[{}]", i + 1, head(a), head(b))
        })
    })
}

/// 줄바꿈을 만드는 56자 편집 뒤 저장·재열기가 편집 직후와 **쪽 구성까지** 같다.
#[test]
fn reopened_layout_matches_the_layout_that_was_saved() {
    let (live, reopened) = edit_save_reopen(56);
    assert_eq!(
        live.len(),
        115,
        "편집 직후가 115쪽이 아니다 — 시험 전제가 깨졌다"
    );
    assert_eq!(
        reopened.len(),
        115,
        "저장본을 다시 열자 {}쪽이 됐다. 첫 조각이 낡은 저장 프레임까지 꼬리를 흡수해 \
         표가 통째로 다음 쪽으로 밀린 것이다(정본도 115쪽).",
        reopened.len()
    );
    assert_eq!(
        first_difference(&live, &reopened),
        None,
        "재열기 쪽 구성이 편집 직후와 다르다 — 저장 프레임과 실제 컷이 다른 결과를 쓴다."
    );
}

/// 같은 결함을 내던 61자도 저장·재열기가 편집 직후와 같다.
#[test]
fn the_sixty_one_character_edit_survives_the_round_trip() {
    let (live, reopened) = edit_save_reopen(61);
    assert_eq!(
        live.len(),
        115,
        "61자 편집 직후가 115쪽이 아니다 — 시험 전제가 깨졌다"
    );
    assert_eq!(
        first_difference(&live, &reopened),
        None,
        "61자에서 재열기 쪽 구성이 편집 직후와 다르다."
    );
}

/// 정본이 고정한 1·2쪽 경계 — 1쪽 끝 `특히,` / 2쪽 시작 `복합재료로`.
#[test]
fn reopened_first_page_boundary_matches_the_hancom_oracle() {
    let (_, reopened) = edit_save_reopen(56);
    let page1 = reopened.first().expect("1쪽");
    let page2 = reopened.get(1).expect("2쪽");
    assert!(
        page1.trim_end().ends_with("특히,"),
        "정본 1쪽은 `특히,` 로 끝난다. 실제 끝: [{}]",
        page1
            .split_whitespace()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" ")
    );
    assert!(
        page2.starts_with("복합재료로"),
        "정본 2쪽은 `복합재료로` 로 시작한다. 실제 시작: [{}]",
        page2
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
    );
}

/// 대조군 — 줄바꿈이 일어나지 않는 55자와 무편집은 종전에도 옳았다. 되쓰기가 과적용되면 깨진다.
#[test]
fn shorter_edit_and_untouched_save_keep_their_layout() {
    for insert_len in [0usize, 1, 55] {
        let (live, reopened) = edit_save_reopen(insert_len);
        assert_eq!(
            live.len(),
            115,
            "{insert_len}자 편집 직후가 115쪽이 아니다 — 시험 전제가 깨졌다"
        );
        assert_eq!(
            first_difference(&live, &reopened),
            None,
            "{insert_len}자에서 재열기 쪽 구성이 달라졌다 — 되쓰기가 과적용됐다."
        );
    }
}

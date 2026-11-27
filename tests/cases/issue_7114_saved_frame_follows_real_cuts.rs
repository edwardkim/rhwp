//! 검증 유형: 관계형 및 고정형. 쪽별 내용 관계와 독립 한컴 Print의 115쪽을 검사한다.
//! 주석 갱신: 2026-11-22 04:14 KST. 직전 파일 변경: 2026-10-04 19:03:36 KST, 731de9e1b4bb946d76f35108ed7e186ebe4ebecb.
//! 2026-11-22 Native Skia 및 fresh WASM Canvas 96 DPI, 색상 경계 보정 전 2px 실루엣.
//! 120자 HWP: 103쪽 Native 99.90% / WASM 99.92%, 104쪽 99.86% / 100.00%.
//! 120자 HWPX: 103쪽 Native 99.85% / WASM 99.92%, 104쪽 99.86% / 100.00%.
//! 두 원본 모두 총 115쪽. 각각의 전쪽 최저는 Native 98.46%, WASM 99.31%.
//! 독립 PDF: `pdf/issue7114-nested-save-120/`. 전쪽 TSV·직접 판독 이미지: `mydocs/pr/assets/issue_7114/`.
//!
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
//! # 중첩 표 안의 경계
//!
//! host 문단의 한 줄을 여러 유닛이 나눠 가지면(중첩 표 행이 그렇다) 그 **안**의 경계는
//! host 사다리가 가리킬 자리가 없다. 뒤의 host 줄을 대신 적으면 "경계가 그 구조 뒤에
//! 있다"는 거짓이 되고, 읽는 쪽의 꼬리 흡수가 그 말을 믿어 중첩 행을 앞 쪽으로 끌어올린다
//! (실측 74.9px, 허용치 48px 안). 그래서 그런 경계는 **쓰지 않는다** — 읽는 쪽이 제
//! 용량 컷을 쓰게 두는 것이 편집 직후 배치와 같다. `a_cut_inside_a_nested_table_...` 가
//! 그 계약을 고정한다.
//!
//! # 중첩 표 호스트 편집의 독립 기준
//!
//! 한컴 2024 Print는 102쪽에 74자, 103쪽의 8행 표 뒤에 46자를 배치한다.
//! 14행 표의 터보 과급기 행은 103쪽 끝에서 시작하며 같은 행이 104쪽 첫머리로 이어진다.
//! 104·105쪽의 그림과 105쪽 후속 제목을 함께 확인했다. 저장은 실제 자식 칸이
//! 여는 새 저장 줄을 쓰고, 자식 컷이 부모 줄을 대신 열지 않는 계약도 유지한다.

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

/// 편집이 길어져 컷이 **중첩 표 안**으로 들어가도 재열기가 편집 직후와 같다.
///
/// 200자를 넣으면 문단 81의 3행짜리 중첩 표 **안**(행 1과 행 2 사이)에서 쪽이 갈린다.
/// 그 경계를 뒤의 host 줄(문단 82)로 적으면 흡수가 행 2(74.9px)를 앞 쪽으로 끌어올린다.
#[test]
fn a_cut_inside_a_nested_table_is_not_written_as_a_later_host_line() {
    let (live, reopened) = edit_save_reopen(200);
    assert_eq!(
        first_difference(&live, &reopened),
        None,
        "컷이 중첩 표 안에 떨어지는 편집에서 재열기 쪽 구성이 편집 직후와 다르다."
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

/// The exported child-cell cut belongs to the selected child line. Saving
/// before the first paint and after a paint must produce the same round trip.
#[test]
fn edited_nested_host_keeps_its_child_cut_and_following_content() {
    type Export = fn(&DocumentCore) -> Result<Vec<u8>, rhwp::error::HwpError>;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (ext, export) in [
        ("hwp", DocumentCore::export_hwp_with_adapter as Export),
        ("hwpx", DocumentCore::export_hwpx_native as Export),
    ] {
        let fixture = root.join(format!(
            "samples/issue1949_giant_cell_nested_tables_perf.{ext}"
        ));
        let input = std::fs::read(fixture).expect("재현물 읽기");
        let mut cold_save = None;
        for warm in [false, true] {
            let mut live = DocumentCore::from_bytes(&input).expect("원본 열기");
            live.insert_text_in_cell_native(0, 0, 2, 2, 2276, 0, &"1".repeat(120))
                .expect("중첩 표 호스트 편집");
            if warm {
                live.get_page_text_layout_native(0).expect("표지 배치");
            }
            let saved = export(&live).expect("첫 저장");
            if let Some(cold) = &cold_save {
                assert_eq!(cold, &saved, "{ext} 저장 결과가 앞선 화면 배치에 의존한다");
            } else {
                cold_save = Some(saved.clone());
            }
            let reopened = DocumentCore::from_bytes(&saved).expect("첫 재열기");
            let resaved = export(&reopened).expect("재저장");
            let repeated = DocumentCore::from_bytes(&resaved).expect("두 번째 재열기");

            let live_words = page_words(&live);
            for (stage, document) in [
                ("편집", &live),
                ("재열기", &reopened),
                ("재저장", &repeated),
            ] {
                let label = format!("{ext} {stage}");
                assert_eq!(document.page_count(), 115, "{label}");
                assert_eq!(
                    first_difference(&live_words, &page_words(document)),
                    None,
                    "{label}"
                );
                assert_child_cut_ownership(document, &label);
            }
        }
    }
}

fn assert_child_cut_ownership(document: &DocumentCore, label: &str) {
    use serde_json::Value;

    let text = |page: u32| -> Value {
        serde_json::from_str(&document.get_page_text_layout_native(page - 1).unwrap()).unwrap()
    };
    let figures = |page: u32| -> Value {
        serde_json::from_str(&document.get_page_control_layout_native(page - 1).unwrap()).unwrap()
    };
    let runs = |page: &Value| page["runs"].as_array().unwrap().clone();
    let controls = |page: &Value| page["controls"].as_array().unwrap().clone();
    let digit_run = |run: &Value| {
        run["cellParaIdx"] == 2276
            && run["text"]
                .as_str()
                .is_some_and(|text| !text.is_empty() && text.bytes().all(|byte| byte == b'1'))
    };

    let first = runs(&text(102));
    let second = runs(&text(103));
    let opening = runs(&text(104));
    let following = runs(&text(105));
    let collect_digits = |runs: &[Value]| {
        runs.iter()
            .filter(|run| digit_run(run))
            .map(|run| run["text"].as_str().unwrap().len())
            .collect::<Vec<_>>()
    };
    assert_eq!(collect_digits(&first), [74], "{label} 102쪽 삽입 문자");
    assert_eq!(collect_digits(&second), [46], "{label} 103쪽 삽입 문자");
    assert!(
        collect_digits(&opening).is_empty(),
        "{label} 104쪽 삽입 문자 중복"
    );

    let controls_103 = controls(&figures(103));
    let owner = controls_103
        .iter()
        .find(|item| item["type"] == "table" && item["paraIdx"] == 2276)
        .expect("삽입 글줄 앞의 8행 표");
    let coordinate = |item: &Value, key: &str| item[key].as_f64().unwrap();
    let bottom = |item: &Value| coordinate(item, "y") + coordinate(item, "h");
    let suffix = second.iter().find(|run| digit_run(run)).unwrap();
    assert!(
        bottom(owner) <= coordinate(suffix, "y"),
        "{label} 표와 뒤 글줄 겹침"
    );

    let exact_text = |run: &Value, expected: &str| {
        run["text"]
            .as_str()
            .is_some_and(|text| text.trim() == expected)
    };
    let head = "터보 과급기";
    let tail = "(Turbosuper-chargers)";
    assert_eq!(
        second.iter().filter(|run| exact_text(run, head)).count(),
        1,
        "{label}"
    );
    assert!(!second.iter().any(|run| exact_text(run, tail)), "{label}");
    assert_eq!(
        opening.iter().filter(|run| exact_text(run, tail)).count(),
        1,
        "{label}"
    );
    assert!(!opening.iter().any(|run| exact_text(run, head)), "{label}");

    for (page, paragraph) in [(104, 2286), (105, 2290)] {
        let controls = controls(&figures(page));
        let pictures = controls
            .iter()
            .filter(|item| item["type"] == "image")
            .collect::<Vec<_>>();
        assert_eq!(pictures.len(), 1, "{label} {page}쪽 그림 수");
        let picture = pictures[0];
        assert_eq!(
            picture["cellParaIdx"], paragraph,
            "{label} {page}쪽 그림 소속"
        );
        let parent = controls
            .iter()
            .find(|item| item["type"] == "table" && item["paraIdx"] == 0)
            .expect("그림이 속한 바깥 표");
        assert!(
            bottom(picture) <= bottom(parent),
            "{label} {page}쪽 그림 clip"
        );
        if page == 105 {
            let heading = following
                .iter()
                .find(|run| exact_text(run, "11.2.9"))
                .unwrap();
            assert!(
                bottom(picture) <= coordinate(heading, "y"),
                "{label} 후속 제목 겹침"
            );
        }
    }
}

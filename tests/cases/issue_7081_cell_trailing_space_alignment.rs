//! [#7081] 표 칸 안 오른쪽·가운데 정렬이 꼬리 공백을 정렬 폭에 세어 글자가 왼쪽으로 밀린다.
//!
//! ## 무엇이 문제인가
//!
//! 오른쪽·가운데 정렬 줄의 **꼬리 공백을 정렬 폭에서 빼는 규칙**이 **표 칸 안에서만 꺼져**
//! 있었다. 한/글은 칸 안에서도 꼬리 공백을 빼고 정렬한다. 그래서 별지 서식의 서명란처럼
//! "글자 + 꼬리 공백" 으로 자리를 잡는 줄이 공백 폭만큼 왼쪽으로 밀린다.
//!
//! ## 예외의 근거가 형상을 잘못 잡았다
//!
//! 칸 예외의 근거는 `issue_1285` 였다 — *"셀 내부는 한글이 말미 공백을 포함해 정렬(수험번호
//! TAC 우단 = 셀 inner 우단 오라클 앵커)"*. 그런데 그 재현체는 **TAC 개체가 우단을 잡는
//! 줄**이고, 순수 텍스트 줄이 아니다. 게다가 Center 조건에는 이미 그 형상 가드
//! (`line_tac_offsets_for_width.is_empty()`)가 있고 **Right 에만 없었다.**
//!
//! 그래서 예외를 "칸 안"이 아니라 **TAC 형상**으로 좁히고 두 정렬의 조건을 맞췄다.
//! `issue_1285` 는 TAC 가 있어 새 조건에 안 걸리므로 종전대로 공백을 포함한다.
//!
//! ## 정본 실측 — 저장소 표본 둘이 정본에 붙는다
//!
//! ```text
//!   samples/hwpx/table-text.hwpx  머리행 '2024년' 칸 (가운데 정렬 + 꼬리 공백)
//!     정본 pdf/hwpx/table-text-2022.pdf · table-text-hwpx-2020.pdf (좌표 동일)
//!       x = 210.72 px · 516.16 px
//!     수정 전  207.85 (-2.87) · 513.39 (-2.77)
//!     수정 후  210.85 (+0.13) · 516.39 (+0.23)
//!
//!   samples/복학원서.hwp  'Name' 줄
//!     정본 pdf/복학원서-hwp-2020.pdf · 복학원서-2022.pdf (좌표 동일)   x = 426.65 px
//!     수정 전  423.42 (-3.23)      수정 후  426.92 (+0.27)
//! ```
//!
//! ## 코퍼스 실측 — 이슈의 원 신고
//!
//! 행정규칙 코퍼스 855건 중 오른쪽·가운데 정렬 문단이 꼬리 공백 2칸 이상을 갖는 문서가
//! 29건(문단 36개, 최대 23칸)이고 전부 별지 서식의 서명·기재란이다. 그 둘을 재면:
//!
//! ```text
//!   3030681 이의신청서 1쪽  '신청인(대표자)' + 공백 15칸, 칸 안 RIGHT
//!     수정 전 첫글자 207.10 (정본 304.64 대비 -97.54)
//!     수정 후 첫글자 307.10 (정본 대비 +2.46)
//!   3079571 취소신청서 1쪽  '신청하는' + 공백 5칸, 칸 안 CENTER
//!     수정 전 (정본 대비 -15.02)      수정 후 (정본 대비 -2.02)
//! ```
//!
//! 그 둘은 코퍼스 문서라 저장소에 넣지 않는다. 잔여 2px 대는 우리 공백·글자 폭이 한/글과
//! 미세하게 다른 **별개 축**이다(② 는 5 × 5.2 대 5 × 6.008).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 가운데 정렬 + 꼬리 공백. 정본 `pdf/hwpx/table-text-2022.pdf`(2020 판본 좌표 동일).
const SAMPLE_TABLE_TEXT: &str = "samples/hwpx/table-text.hwpx";
/// 오른쪽 정렬 칸. 정본 `pdf/복학원서-hwp-2020.pdf`(2022 판본 좌표 동일).
const SAMPLE_BOKHAK: &str = "samples/복학원서.hwp";

/// 한 쪽에서 `needle` 로 시작하는 런의 **첫 글자 절대 x** 를 모은다.
fn first_glyph_x(sample: &str, page: u32, needle: &str) -> Vec<f64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let raw = core
        .get_page_text_layout_native(page)
        .expect("공개 text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
    let mut out = Vec::new();
    for run in layout["runs"].as_array().into_iter().flatten() {
        let text = run["text"].as_str().unwrap_or_default();
        if !text.contains(needle) {
            continue;
        }
        let Some(off) = text.find(needle) else {
            continue;
        };
        let idx = text[..off].chars().count();
        let x = run["x"].as_f64().unwrap_or(0.0);
        let first = run["charX"]
            .as_array()
            .and_then(|a| a.get(idx))
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        out.push(x + first);
    }
    out.sort_by(|a, b| a.partial_cmp(b).expect("유한값"));
    out
}

/// 가운데 정렬 칸의 꼬리 공백은 정렬 폭에 들어가지 않는다.
///
/// 머리행의 `2024년` 칸 넷 중 **꼬리 공백이 있는 둘**이 수정 전 정본보다 2.8px 왼쪽이었다.
/// 나머지 둘은 전후 불변이라, 같은 줄에서 **움직일 것과 안 움직일 것을 함께** 잡는다.
///
/// 런은 `"2024"` 와 `"년 "` 으로 나뉘어 있어 숫자 런의 첫 글자로 앵커한다.
#[test]
fn center_aligned_cell_excludes_trailing_spaces() {
    let xs = first_glyph_x(SAMPLE_TABLE_TEXT, 0, "2024");
    let expected = [210.72_f64, 345.12, 516.16, 650.72];
    assert_eq!(
        xs.len(),
        expected.len(),
        "머리행 '2024' 칸을 {}개 봐야 한다 — 표본 전제가 깨졌다. got {xs:?}",
        expected.len()
    );
    let off: Vec<_> = xs
        .iter()
        .zip(expected)
        .filter(|(got, want)| (**got - want).abs() > 1.0)
        .collect();
    assert!(
        off.is_empty(),
        "정본(pdf/hwpx/table-text-2022.pdf) 좌표와 1.0px 안에서 맞아야 한다(수정 전 첫째·셋째가 -2.8px). \
         어긋난 것 (실측, 정본): {off:?} · 전체 {xs:?}"
    );
}

/// 오른쪽 정렬 칸의 꼬리 공백도 마찬가지다.
///
/// 수정 전에는 정본보다 3.2px 왼쪽이었다.
#[test]
fn right_aligned_cell_excludes_trailing_spaces() {
    let xs = first_glyph_x(SAMPLE_BOKHAK, 0, "Name");
    assert!(!xs.is_empty(), "'Name' 런이 없다 — 표본 전제가 깨졌다");
    let want = 426.65_f64;
    let hit = xs.iter().any(|x| (x - want).abs() <= 0.5);
    assert!(
        hit,
        "정본(pdf/복학원서-hwp-2020.pdf) 의 'Name' 첫 글자는 {want:.2}px 다(수정 전 423.42). \
         got {xs:?}"
    );
}

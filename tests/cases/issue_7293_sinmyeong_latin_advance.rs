//! [#7293] `신명 중명조` 의 라틴 전진폭이 0.5em 폴백으로 떨어지지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! 이 face 는 글꼴 규칙 레지스트리의 `layout-name`·`paint` 평면에는 규칙이 있는데
//! **`layout-metric` 평면에만 없었다.** 그래서 측정 경로만 이 face 를 모르는 채로
//! 미등록 글꼴 폴백(`text_measurement` 의 라틴 `font_size * 0.5`)에 떨어졌다.
//!
//! ```text
//!   1170000-200500003_…(최종본).hwp 4쪽 차례 줄 'Ⅲ. EU법'  (15.31px)
//!      수정 전  E 전진  7.65px = 0.500 em      ← 폴백
//!      정본     E 전진 12.54px = 0.819 em
//! ```
//!
//! 이슈 본문의 다른 가설(언어별 글꼴 슬롯 미적용)은 **반증됐다** — 그 런의 char shape 는
//! 영문 슬롯(슬롯1)도 `신명 중명조` 를 가리킨다. 문서가 라틴까지 이 face 를 요구한다.
//!
//! # 기대값의 출처
//!
//! 한/글 2020 정본(`pdf/issue7293-1170000-200500003-p4-2020.pdf`). 한/글은 이 글꼴을
//! **Type3** 로 그리므로 PDF 의 `/Widths`(`FontMatrix .001`)가 실측 폭 표 그 자체다.
//! 배치 전진과도 2% 안에서 일치한다(공백 하나만 예외라 구간에서 뺐다).
//!
//! 원본은 코퍼스 경로에 있다 — `hwpdocs_10k_share/prism_downloads/법제처/
//! 1170000-200500003_D0150004-1-001_독일의 법령체계와 입법심사기준(최종본).hwp`.
//! 이 시험은 문서를 열지 않고 face 이름으로 측정만 하므로 재현본을 저장소에 넣지 않았다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::TextStyle;

const FACE: &str = "신명 중명조";
const EM: f64 = 100.0;

fn advances(text: &str) -> Vec<f64> {
    let m = EmbeddedTextMeasurer;
    let style = TextStyle {
        font_family: FACE.to_string(),
        font_size: EM,
        ratio: 1.0,
        ..Default::default()
    };
    let p = m.compute_char_positions(text, &style);
    (1..p.len()).map(|i| (p[i] - p[i - 1]) / EM).collect()
}

fn assert_em(got: f64, want: f64, what: &str) {
    assert!(
        (got - want).abs() < 0.002,
        "{what}: 정본 {want:.3} em 인데 {got:.3} em 입니다"
    );
}

/// 정본 `/Widths` 실측값 — 이 셋이 이슈가 신고한 그 글자들이다.
#[test]
fn measured_latin_advances_match_the_oracle() {
    let a = advances("EU");
    assert_em(a[0], 0.819, "'E'");
    assert_em(a[1], 0.918, "'U'");

    // 쪽 번호도 같은 축이다 — 종전에는 숫자도 0.5em 이었다.
    for adv in advances("0123456789") {
        assert_em(adv, 0.620, "숫자");
    }
    assert_em(advances("..")[0], 0.400, "'.'");
    assert_em(advances("()")[0], 0.500, "'('");
}

/// 한글은 정본이 전부 1000/1000 이라 종전(1.0em)과 같아야 한다 — 이 수정은 한글 축을
/// 건드리지 않는다.
#[test]
fn hangul_advance_is_unchanged() {
    for adv in advances("법률의유형") {
        assert_em(adv, 1.000, "한글");
    }
}

/// **실측하지 않은 글자는 그대로 둔다.**
///
/// 정본 Type3 서브셋에 없던 글자(`O`·`Q`·`X`·`Y`·`j`·공백 등)는 구간에 넣지 않았다.
/// 조회가 `None` 으로 떨어져 종전 폴백(0.5em)을 그대로 쓴다 — 없는 값을 지어내지 않았다는
/// 것을 여기서 잠근다.
#[test]
fn unmeasured_characters_keep_the_previous_fallback() {
    for (text, what) in [("OO", "'O'"), ("QQ", "'Q'"), ("XX", "'X'"), ("jj", "'j'")] {
        assert_em(advances(text)[0], 0.500, what);
    }
    // 공백은 `/Widths` 가 1000 이라고 적지만 실제 배치 전진은 241/1000 이라 뺐다.
    assert_em(advances("  ")[0], 0.500, "공백");
}

/// 다른 face 는 움직이지 않는다 — 오버레이는 이 이름에만 붙는다.
#[test]
fn other_faces_are_untouched() {
    let m = EmbeddedTextMeasurer;
    for (fam, e, u) in [("HY신명조", 0.707, 0.791), ("한양신명조", 0.677, 0.771)] {
        let style = TextStyle {
            font_family: fam.to_string(),
            font_size: EM,
            ratio: 1.0,
            ..Default::default()
        };
        let p = m.compute_char_positions("EU", &style);
        assert!(
            ((p[1] - p[0]) / EM - e).abs() < 0.01 && ((p[2] - p[1]) / EM - u).abs() < 0.01,
            "{fam} 의 라틴 폭이 움직였습니다: E={:.3} U={:.3}",
            (p[1] - p[0]) / EM,
            (p[2] - p[1]) / EM
        );
    }
}

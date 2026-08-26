//! [#6060] 「」 반각 판정은 측정과 페인트가 같은 기준을 써야 한다.
//!
//! 페인트(skia `text_replay`, `web_canvas`)가 폰트 **이름 목록**으로 반각 여부를
//! 정하면 폭 측정과 갈린다.
//!
//! - 이름 목록 밖 고정폭(바탕체·궁서체·D2Coding): 측정은 반각 advance 인데 글리프를
//!   전각으로 그려 다음 글자와 겹친다.
//! - 이름에 `돋움체` 를 포함하지만 메트릭 DB 밖(KoPub돋움체): 측정은 전각인데 글리프만
//!   반각으로 눌려 오른쪽에 빈 공간이 남는다.
//!
//! `forces_halfwidth_cjk_quote` 는 측정 결정을 그대로 되묻으므로, 이 시험은 두 경로가
//! 갈리지 않는다는 계약을 고정한다. SVG 경로는 `textLength` 로 글리프를 advance 에
//! 맞춰 눌러 이 발산을 가리므로 SVG 증적만으로는 검출되지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::layout::forces_halfwidth_cjk_quote;

/// 본문 조판에서 흔한 크기. 「」 판정은 메트릭 비율이라 크기와 무관해야 한다.
const FONT_SIZE: f64 = 13.0;
const QUOTES: [char; 2] = ['\u{300C}', '\u{300D}'];

fn forced(font_family: &str) -> Vec<bool> {
    QUOTES
        .iter()
        .map(|c| forces_halfwidth_cjk_quote(font_family, false, false, *c, FONT_SIZE))
        .collect()
}

/// #2020 여권신청서: 한컴은 돋움체 계열 「」 를 반각으로 조판한다.
#[test]
fn issue_2020_fixed_width_faces_force_halfwidth_quote() {
    for face in ["돋움체", "DotumChe", "굴림체", "GulimChe"] {
        assert_eq!(
            forced(face),
            vec![true, true],
            "{face}: 고정폭 메트릭(「 폭 = em_size)은 반각 조판이어야 한다"
        );
    }
}

/// #6060 휴먼명조·HY헤드라인M: 비례 글꼴은 「」 가 전폭이다.
#[test]
fn issue_6060_proportional_faces_keep_fullwidth_quote() {
    for face in [
        "휴먼명조",
        "HY헤드라인M",
        "HYHeadLine-Medium",
        "돋움",
        "바탕",
    ] {
        assert_eq!(
            forced(face),
            vec![false, false],
            "{face}: 비례 글꼴은 「」 를 반각으로 누르지 않는다"
        );
    }
}

/// 회귀: 이름 목록 밖 고정폭 글꼴도 측정과 같은 반각 판정을 받아야 한다.
///
/// 이름 기반 판정(`돋움체`/`굴림체`만 반각)에서는 이 글꼴들이 advance 반각 ·
/// 글리프 전각으로 갈려 「」 뒤 글자와 겹쳤다.
#[test]
fn monospace_faces_outside_name_list_match_measurement() {
    for face in ["바탕체", "BatangChe", "궁서체", "GungsuhChe", "D2Coding"] {
        assert_eq!(
            forced(face),
            vec![true, true],
            "{face}: 고정폭 메트릭이면 이름과 무관하게 측정과 같은 반각 판정이어야 한다"
        );
    }
}

/// 회귀: 이름만 `돋움체` 를 포함하고 메트릭 DB 밖인 글꼴은 반각으로 누르지 않는다.
///
/// KoPub 계열은 전용 폭 표를 쓰고 메트릭 DB 에 없다. 이름 기반 판정에서는
/// `KoPub돋움체`.contains("돋움체") 가 참이라 글리프만 반각으로 눌렸다.
#[test]
fn name_lookalike_outside_metric_db_is_not_forced() {
    for face in ["KoPub돋움체", "KoPub Dotum"] {
        assert_eq!(
            forced(face),
            vec![false, false],
            "{face}: 메트릭 DB 밖 글꼴은 페인트에서도 반각으로 누르지 않는다"
        );
    }
}

/// CSS 체인·따옴표가 붙어도 첫 face 로 판정한다.
#[test]
fn css_chain_first_face_decides() {
    assert_eq!(forced("'돋움체', sans-serif"), vec![true, true]);
    assert_eq!(forced("\"휴먼명조\",serif"), vec![false, false]);
}

/// 낫표가 아닌 문자는 이 경로가 건드리지 않는다.
#[test]
fn non_quote_chars_are_never_forced() {
    for c in ['가', 'A', '\u{2018}', '\u{300E}'] {
        assert!(
            !forces_halfwidth_cjk_quote("돋움체", false, false, c, FONT_SIZE),
            "U+{:04X}: 「」 외 문자는 이 판정 대상이 아니다",
            c as u32
        );
    }
}

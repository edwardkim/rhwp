//! [#7418] 공백 최소값(condense)은 **이미 시작한 낱말**만 더 담는다 — 새 낱말은 줄이
//! 아직 자연폭 안에 있을 때만 시작한다.
//!
//! # 무엇이 깨져 있었나
//!
//! 재조판 줄 나눔은 condense 로 공백을 줄여야만 들어가는 토큰을, 줄에 **2.5em 이상의 틈이
//! 남았을 때만** 받았다(`condense_fit_can_pull_next_token`). 한/글의 규칙이 아니었다.
//!
//! - 이어지는 글자(글자 단위 문단에서 낱말의 둘째 글자 이후)는 틈이 0.6em 뿐이어도 한/글이
//!   공백을 줄여 담는다. rhwp 는 거부해 **일찍** 끊었다.
//! - 반대로 문턱만 걷으면 공백을 이미 줄인 줄에 **새 낱말의 첫 글자**를 끌어와 낱말 가운데를
//!   가른다. 한/글은 그러지 않는다.
//!
//! # 기대값의 출처 — 한/글이 스스로 적은 줄 시작 위치
//!
//! 두 입력은 **같은 문서**다. `condense_break_synthetic.hwpx` 는 저장 줄(`hp:linesegarray`)이
//! 없어 rhwp 가 직접 줄을 잡고, `condense_break_synthetic-hancom-2024.hwpx` 는 그 파일을
//! 한/글 2024(13.0.0.564)가 열어 저장한 본이라 한/글의 `hp:lineseg` 가 있다. 저장본의 줄은
//! 같은 세션의 한/글 PDF(`pdf/issue7418/condense_break_synthetic-2024.pdf`)와 13문단 모두
//! 글자 단위로 같다. 생성기는
//! `mydocs/tech/investigations/issue-7418/probes/make_condense_fixture.py` 다.
//!
//! 13문단은 문단 모양만 다르다(맑은 고딕 10pt, 양쪽 정렬, 가용 42520 HWPUNIT).
//!
//! | 묶음 | 줄 나눔 단위 | 낱말 | condense |
//! | --- | --- | --- | --- |
//! | A | 글자(`KEEP_WORD`, #2185) | 1~6자 | 0 · 15 · 30 · 50 · 75 |
//! | B | 글자 | 1~2자(공백 많음) | 50 · 75 |
//! | C | 글자 | 5~8자(공백 적음) | 50 · 75 |
//! | D | 낱말(`BREAK_WORD`) | 1~6자 | 0 · 30 · 50 · 75 |
//!
//! 한/글 PDF 의 글자 원점으로 세 실험(203줄)을 재 보면, 줄 끊음 전부를 한 규칙이 재현한다 —
//! 공백은 condense% 까지 줄이되 새 낱말은 그 앞 줄이 자연폭 안일 때만 시작한다. B 와 C 가
//! 가르는 것은 대안 모형이다. 새 낱말의 한도를 **축소율**로 두면 공백이 많은 B 를, **남은 틈**
//! (종전 2.5em 문턱)으로 두면 C 를 설명하지 못한다.
//!
//! # 공백을 줄여 넣을 때는 줄바꿈 여유를 얹지 않는다
//!
//! A 의 condense 15·30 문단에는 condense 후 후보 폭이 상자를 **5·30 HWPUNIT** 넘는 줄이 있다.
//! rhwp 의 줄바꿈 여유(`line_break_tolerance_hwp`, 이 상자에서 +50)를 얹으면 받게 되지만 한/글은
//! 거절한다. 그래서 줄인 폭은 여유 없이 상자와 비교한다(자연폭 판정의 여유는 그대로 둔다).
//!
//! ```text
//!   A c15 줄12: 자연 43500 → condense 후 42525   (상자 42520)
//!   A c30 줄13: 자연 44500 → condense 후 42550
//! ```
//!
//! # 이 검사가 말하지 않는 것
//!
//! 저장 줄을 믿는 경로는 바뀌지 않는다. 쪽 수(rhwp 7 / 한/글 6)는 줄 높이 축이라 보지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const NO_CACHE: &str = "samples/issue7418/condense_break_synthetic.hwpx";
const HANCOM: &str = "samples/issue7418/condense_break_synthetic-hancom-2024.hwpx";

/// 본문 문단 순서대로의 이름. 문단 0 은 구역 정의를 담은 빈 문단이다.
const LABELS: [&str; 13] = [
    "A 글자 c0",
    "A 글자 c15",
    "A 글자 c30",
    "A 글자 c50",
    "A 글자 c75",
    "B 짧은낱말 c50",
    "B 짧은낱말 c75",
    "C 긴낱말 c50",
    "C 긴낱말 c75",
    "D 낱말 c0",
    "D 낱말 c30",
    "D 낱말 c50",
    "D 낱말 c75",
];

/// 맨 앞 공백 실험 — 문단은 (condense, 맨 앞 공백 수)만 다르다.
const LEAD_NO_CACHE: &str = "samples/issue7418/condense_leading_space_synthetic.hwpx";
const LEAD_HANCOM: &str = "samples/issue7418/condense_leading_space_synthetic-hancom-2024.hwpx";
const LEAD_LABELS: [&str; 8] = [
    "c75 앞공백0",
    "c75 앞공백2",
    "c75 앞공백4",
    "c75 앞공백6",
    "c50 앞공백0",
    "c50 앞공백2",
    "c50 앞공백4",
    "c50 앞공백6",
];

fn line_starts(rel: &str, expected_body_paragraphs: usize) -> Vec<Vec<u32>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(
        paragraphs.len(),
        expected_body_paragraphs + 1,
        "{rel}: 본문 문단 수가 fixture 생성기와 다르다 — 시험 설정 오류"
    );
    paragraphs[1..]
        .iter()
        .map(|p| p.line_segs.iter().map(|seg| seg.text_start).collect())
        .collect()
}

/// rhwp 가 직접 잡은 줄이 한/글이 적어 둔 줄과 **글자 단위로** 같다.
#[test]
fn recomposed_line_starts_match_hancom_under_condense() {
    let hancom = line_starts(HANCOM, LABELS.len());
    let rhwp = line_starts(NO_CACHE, LABELS.len());

    // 전제: condense 가 실제로 줄 끊음을 바꾸는 입력이어야 한다. 같은 글을 쓴 A 의 c0 과
    // c50 이 한/글에서 갈리지 않으면 이 검사는 아무것도 잠그지 않는다.
    assert_ne!(
        hancom[0], hancom[3],
        "정답지 전제가 깨졌다 — condense 0 과 50 의 줄이 같다"
    );

    let mismatched: Vec<String> = LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(label, (h, r))| {
            let first = h
                .iter()
                .zip(r.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(h.len().min(r.len()));
            format!(
                "{label}: 한/글 {}줄 / rhwp {}줄, 줄 {first} 부터 다름 (한/글 {:?} / rhwp {:?})",
                h.len(),
                r.len(),
                h.get(first),
                r.get(first)
            )
        })
        .collect();
    assert!(
        mismatched.is_empty(),
        "condense 문단의 줄이 한/글과 다른 글자에서 갈렸다:\n{}",
        mismatched.join("\n")
    );
}

/// 반례 경계 — condense 가 없는 문단은 종전과 같이 한/글과 일치한다.
#[test]
fn paragraphs_without_condense_are_unchanged() {
    let hancom = line_starts(HANCOM, LABELS.len());
    let rhwp = line_starts(NO_CACHE, LABELS.len());
    for (i, label) in LABELS.iter().enumerate() {
        if label.ends_with(" c0") {
            assert_eq!(
                rhwp[i], hancom[i],
                "{label}: condense 0 문단이 한/글과 달라졌다"
            );
        }
    }
}

/// 줄 맨 앞 공백은 condense 로 줄지 않는다.
///
/// 문단은 맨 앞 공백 수(0·2·4·6)와 condense(75·50)만 다르다. 첫 줄은 4자 낱말 8개 뒤에
/// 12자 낱말이 오도록 짜서, 맨 앞 공백까지 줄이면 그 낱말의 글자를 1~2자 더 담는다.
/// 한/글 2024 는 맨 앞 공백을 줄이지 않는다(맨 앞 공백 0 문단은 두 해석이 같아 대조군이다).
#[test]
fn leading_spaces_are_not_condensed() {
    let hancom = line_starts(LEAD_HANCOM, LEAD_LABELS.len());
    let rhwp = line_starts(LEAD_NO_CACHE, LEAD_LABELS.len());

    // 전제: 맨 앞 공백 수가 한/글의 첫 줄 끝을 실제로 바꿔야 한다.
    assert_ne!(
        hancom[0].get(1),
        hancom[3].get(1),
        "정답지 전제가 깨졌다 — 맨 앞 공백 0 과 6 의 첫 줄 끝이 같다"
    );
    let mismatched: Vec<String> = LEAD_LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(label, (h, r))| format!("{label}: 한/글 {h:?} / rhwp {r:?}"))
        .collect();
    assert!(
        mismatched.is_empty(),
        "맨 앞 공백 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 글머리표 실험 — 같은 글, 같은 왼쪽 여백(2000)의 문단 5개.
///
/// 생성기는 `mydocs/tech/investigations/issue-7418/probes/make_condense_bullet_fixture.py`,
/// 한/글 저장본은 한/글 2024(13.0.0.564)가 연 뒤 저장한 것이다(`pdf/issue7418/condense_bullet_synthetic-2024.pdf`).
const BULLET_NO_CACHE: &str = "samples/issue7418/condense_bullet_synthetic.hwpx";
const BULLET_HANCOM: &str = "samples/issue7418/condense_bullet_synthetic-hancom-2024.hwpx";
const BULLET_LABELS: [&str; 5] = [
    "글머리표 글자 c0",
    "글머리표 글자 c20",
    "글머리표 글자 c50",
    "글머리표 낱말 c20",
    "대조군(글머리표 없음) 글자 c20",
];

/// 글머리표는 **모든 줄**의 앞을 차지한다 — 줄 나눔 상자에서 마커 폭을 뺀다.
///
/// 한/글은 마커 뒤에서 본문을 시작하고 둘째 줄부터도 같은 자리에 맞춘다(행잉). 배치는 그렇게
/// 그렸지만 줄 나눔은 마커를 몰라 줄마다 마커 폭만큼 더 담았다. condense 0 문단도 틀렸으므로
/// condense 와 별개의 결함이다. condense 가 새 낱말을 끌어오게 되자(#7418) 넘친 줄이 커져
/// `endnote-01`·`footnote-01`·`pr-1674` 의 글머리표 문단에서 드러났다.
///
/// 수정 전(마커 폭을 빼지 않던 줄 나눔)은 글머리표 문단 4개가 모두 줄 1~2 에서 한/글보다
/// 늦게 끊는다. 대조군은 수정 전후 모두 한/글과 같아, 차이가 마커에서만 난다는 것을 보인다.
#[test]
fn bullet_marker_is_excluded_from_the_line_box() {
    let hancom = line_starts(BULLET_HANCOM, BULLET_LABELS.len());
    let rhwp = line_starts(BULLET_NO_CACHE, BULLET_LABELS.len());

    // 전제: 같은 글이므로 마커 폭이 한/글의 줄을 실제로 바꿔야 한다.
    assert_ne!(
        hancom[1], hancom[4],
        "정답지 전제가 깨졌다 — 글머리표 문단과 대조군의 줄이 같다"
    );
    let mismatched: Vec<String> = BULLET_LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(label, (h, r))| format!("{label}: 한/글 {h:?} / rhwp {r:?}"))
        .collect();
    assert!(
        mismatched.is_empty(),
        "글머리표 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 표 칸 실험 — 본문 실험과 같은 13문단을 각각 1×1 표 칸(가용 41500)에 넣었다.
///
/// 생성기는 `mydocs/tech/investigations/issue-7418/probes/make_condense_cell_fixture.py`,
/// 한/글 저장본의 칸 줄은 같은 세션 PDF(`pdf/issue7418/condense_cell_synthetic-2024.pdf`)와 211줄 중
/// 210줄이 글자 단위로 같다(나머지 1줄은 PDF 텍스트 추출이 한 줄을 여러 조각으로 나눈 것).
const CELL_NO_CACHE: &str = "samples/issue7418/condense_cell_synthetic.hwpx";
const CELL_HANCOM: &str = "samples/issue7418/condense_cell_synthetic-hancom-2024.hwpx";

fn cell_line_starts(rel: &str) -> Vec<Vec<u32>> {
    use rhwp::model::control::Control;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let starts: Vec<Vec<u32>> = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .map(|table| {
            assert_eq!(
                table.cells.len(),
                1,
                "{rel}: 1×1 표여야 한다 — 시험 설정 오류"
            );
            let para = &table.cells[0].paragraphs[0];
            para.line_segs.iter().map(|seg| seg.text_start).collect()
        })
        .collect();
    assert_eq!(
        starts.len(),
        LABELS.len(),
        "{rel}: 칸 수가 fixture 생성기와 다르다 — 시험 설정 오류"
    );
    starts
}

/// 표 칸에서도 새 낱말은 앞 줄 자연폭이 상자보다 **엄격히** 좁을 때만 condense 로 시작한다.
///
/// 칸은 본문보다 상자가 좁아 앞 줄이 자연폭으로 상자를 꼭 채우는 경우가 생긴다. 앞 줄이 상자에
/// 줄바꿈 여유를 더한 한계를 넘을 때만 새 낱말을 거절하던 판정은 13문단 중 8개만 한/글과 같았고,
/// devel(2.5em 문턱)은 3개였다.
#[test]
fn cell_paragraphs_start_new_words_only_below_the_box() {
    let hancom = cell_line_starts(CELL_HANCOM);
    let rhwp = cell_line_starts(CELL_NO_CACHE);
    let mismatched: Vec<String> = LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(label, (h, r))| format!("{label}: 한/글 {h:?} / rhwp {r:?}"))
        .collect();
    assert!(
        mismatched.is_empty(),
        "칸 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 끝 공백 실험 — 한글 42자(42000)에 끝 공백 수만 다르다. 가용 42520.
///
/// 생성기는 `mydocs/tech/investigations/issue-7418/probes/make_trailing_space_fixture.py`.
const TRAILING_NO_CACHE: &str = "samples/issue7418/trailing_space_synthetic.hwpx";
const TRAILING_HANCOM: &str = "samples/issue7418/trailing_space_synthetic-hancom-2024.hwpx";
const TRAILING_LABELS: [&str; 5] = [
    "42자 + 공백 0(대조군)",
    "42자 + 공백 1",
    "42자 + 공백 2",
    "42자 + 공백 3",
    "42자 + 공백 2 + 42자(대조군)",
];

/// 문단 끝에서 넘친 공백은 줄 끝에 걸리고 빈 줄을 만들지 않는다.
///
/// 한/글은 넘친 공백 **하나**를 줄 끝에 걸고 그 뒤 공백은 새 줄로 넘긴다: 공백 2 → 1줄,
/// 공백 3 → 2줄. 종전 rhwp 는 걸린 공백이 문단의 마지막 글자여도 글자 없는 빈 행을 하나 더
/// 게시해 공백 2 문단이 2줄이었다(`rowbreak_cell_picture_only_paragraph` 2쪽 칸의 빈 줄).
/// 공백 뒤에 글이 이어지는 대조군은 수정 전후 모두 2줄이다.
#[test]
fn overflowing_trailing_space_does_not_open_an_empty_row() {
    let hancom = line_starts(TRAILING_HANCOM, TRAILING_LABELS.len());
    let rhwp = line_starts(TRAILING_NO_CACHE, TRAILING_LABELS.len());

    // 전제: 공백 2 와 3 이 한/글에서 갈라야 이 검사가 경계를 잠근다.
    assert_ne!(
        hancom[2].len(),
        hancom[3].len(),
        "정답지 전제가 깨졌다 — 끝 공백 2 와 3 의 줄 수가 같다"
    );
    let mismatched: Vec<String> = TRAILING_LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(label, (h, r))| format!("{label}: 한/글 {h:?} / rhwp {r:?}"))
        .collect();
    assert!(
        mismatched.is_empty(),
        "끝 공백 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 번호 문단 실험(#7436) — 글머리표 실험과 같은 글·모양에서 머리 모양만 번호(`^1.`)다.
///
/// 생성기는 `mydocs/tech/investigations/issue-7418/probes/make_condense_numbered_fixture.py`,
/// 한/글 저장 줄은 같은 세션 PDF(`pdf/issue7418/condense_numbered_synthetic-2024.pdf`)와 55/55 줄 같다.
const NUMBERED_NO_CACHE: &str = "samples/issue7418/condense_numbered_synthetic.hwpx";
const NUMBERED_HANCOM: &str = "samples/issue7418/condense_numbered_synthetic-hancom-2024.hwpx";
const NUMBERED_LABELS: [&str; 5] = [
    "번호 글자 c0",
    "번호 글자 c20",
    "번호 글자 c50",
    "번호 낱말 c20",
    "대조군(번호 없음) 글자 c20",
];

/// 번호도 **모든 줄**의 앞을 차지한다 — 줄 나눔 상자에서 번호 폭을 뺀다(#7436).
///
/// 번호 문자열은 문서 순서의 계수기로 정해져 줄 나눔이 알 수 없었다. 쪽 나누기 전에 문서
/// 순서로 번호를 계산해 문단에 두고(`assign_numbering_markers`), 줄 나눔과 배치가 같은 문자열을
/// 쓴다. 수정 전(번호 폭을 빼지 않던 줄 나눔)은 번호 문단 4개가 모두 한/글과 달랐고 대조군만
/// 같았다(1/5, devel 은 0/5).
#[test]
fn numbered_marker_is_excluded_from_the_line_box() {
    let hancom = line_starts(NUMBERED_HANCOM, NUMBERED_LABELS.len());
    let rhwp = line_starts(NUMBERED_NO_CACHE, NUMBERED_LABELS.len());

    // 전제: 같은 글이므로 번호 폭이 한/글의 줄을 실제로 바꿔야 한다.
    assert_ne!(
        hancom[1], hancom[4],
        "정답지 전제가 깨졌다 — 번호 문단과 대조군의 줄이 같다"
    );
    let mismatched: Vec<String> = NUMBERED_LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(label, (h, r))| format!("{label}: 한/글 {h:?} / rhwp {r:?}"))
        .collect();
    assert!(
        mismatched.is_empty(),
        "번호 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 자간 × condense 실험 — 본문 실험과 같은 13문단에 모든 글자의 자간만 −20·−12·−6·+10% 로 둔다.
///
/// 생성기는 `mydocs/tech/investigations/issue-7418/probes/make_condense_letter_spacing_fixture.py`.
/// 한/글 저장 줄은 같은 세션 PDF 와 −20·−12·−6% 는 전 줄, +10% 는 229줄 중 227줄이 같다.
const LETTER_SPACING_FIXTURES: [(&str, &str, &str); 4] = [
    (
        "자간 −20%",
        "samples/issue7418/condense_letter_spacing_m20.hwpx",
        "samples/issue7418/condense_letter_spacing_m20-hancom-2024.hwpx",
    ),
    (
        "자간 −12%",
        "samples/issue7418/condense_letter_spacing_m12.hwpx",
        "samples/issue7418/condense_letter_spacing_m12-hancom-2024.hwpx",
    ),
    (
        "자간 −6%",
        "samples/issue7418/condense_letter_spacing_m6.hwpx",
        "samples/issue7418/condense_letter_spacing_m6-hancom-2024.hwpx",
    ),
    (
        "자간 +10%",
        "samples/issue7418/condense_letter_spacing_p10.hwpx",
        "samples/issue7418/condense_letter_spacing_p10-hancom-2024.hwpx",
    ),
];

/// 공백 최소값(condense)은 자간 **적용 전** 공백 폭의 c% 를 줄인다.
///
/// 한/글 규칙은 넷이다: 줄에 들어가는가는 줄 끝 글자의 자간을 빼고 재고(#5678), 새 낱말 허용은
/// 공백 앞 글자의 자간을 넣고 재며, 공백의 자간은 공백 폭 비례이고, condense 는 자간 전
/// 공백 폭(반각)을 기준으로 줄인다. 넷째 규칙 대신 자간 적용 후 폭을 쓰면 음수 자간 문단이
/// 한/글보다 한 줄에 덜 담는다(수정 전 −20% 7/13, −12% 7/13, −6% 10/13, +10% 9/13 문단).
#[test]
fn condense_saves_from_the_space_width_before_letter_spacing() {
    let mut mismatched = Vec::new();
    for (name, no_cache, hancom) in LETTER_SPACING_FIXTURES {
        let h = line_starts(hancom, LABELS.len());
        let r = line_starts(no_cache, LABELS.len());
        for (label, (hl, rl)) in LABELS.iter().zip(h.iter().zip(r.iter())) {
            if hl != rl {
                mismatched.push(format!("{name} {label}: 한/글 {hl:?} / rhwp {rl:?}"));
            }
        }
    }
    assert!(
        mismatched.is_empty(),
        "자간 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 글머리표 머리 모양 실험 — 문단마다 글머리표 정의 하나(`❍`, 맑은 고딕 10pt, 왼쪽 여백
/// 4000 HWPUNIT)를 두고 너비 보정·본문과의 거리·들여쓰기·자동 내어쓰기·정렬만 바꾼다.
///
/// 생성기는 `mydocs/tech/investigations/issue-7418/probes/make_list_marker_head_fixture.py`,
/// 한/글 2024 저장본과 같은 세션 PDF(`pdf/issue7418/list_marker_head_synthetic-2024.pdf`)가 있다.
const HEAD_NO_CACHE: &str = "samples/issue7418/list_marker_head_synthetic.hwpx";
const HEAD_HANCOM: &str = "samples/issue7418/list_marker_head_synthetic-hancom-2024.hwpx";

/// (너비 보정, 거리 %, 들여쓰기, 자동 내어쓰기, 정렬, 한/글 첫 줄 본문, 한/글 둘째 줄) —
/// 본문 x 는 PDF 글자 원점을 문단 왼쪽 여백 기준 HWPUNIT 으로 잰 값이다(쪽 척도 0.99894 보정).
const HEAD_CASES: [(i32, i32, i32, bool, &str, i32, i32); 43] = [
    (0, 50, 0, true, "LEFT", 1507, 1507),
    (0, 50, -1200, true, "LEFT", 1507, 2708),
    (0, 50, 1000, true, "LEFT", 1507, 511),
    (0, 0, 0, true, "LEFT", 1015, 1015),
    (0, 0, -1200, true, "LEFT", 1015, 2216),
    (0, 0, 1000, true, "LEFT", 1015, 6),
    (0, 100, 0, true, "LEFT", 2012, 2012),
    (0, 100, -1200, true, "LEFT", 2012, 3212),
    (0, 100, 1000, true, "LEFT", 2012, 1015),
    (-1000, 50, 0, true, "LEFT", 511, 511),
    (-1000, 50, -1200, true, "LEFT", 511, 1711),
    (-1000, 50, 1000, true, "LEFT", 1015, 6),
    (-1000, 0, 0, true, "LEFT", 6, 6),
    (-1000, 0, -1200, true, "LEFT", 6, 1207),
    (-1000, 0, 1000, true, "LEFT", 1015, 6),
    (-1000, 100, 0, true, "LEFT", 1015, 1015),
    (-1000, 100, -1200, true, "LEFT", 1015, 2216),
    (-1000, 100, 1000, true, "LEFT", 1015, 6),
    (1000, 50, 0, true, "LEFT", 2516, 2516),
    (1000, 50, -1200, true, "LEFT", 2516, 3716),
    (1000, 50, 1000, true, "LEFT", 2516, 1507),
    (1000, 0, 0, true, "LEFT", 2012, 2012),
    (1000, 0, -1200, true, "LEFT", 2012, 3212),
    (1000, 0, 1000, true, "LEFT", 2012, 1015),
    (1000, 100, 0, true, "LEFT", 3008, 3008),
    (1000, 100, -1200, true, "LEFT", 3008, 4209),
    (1000, 100, 1000, true, "LEFT", 3008, 2012),
    (3000, 50, 0, true, "LEFT", 4509, 4509),
    (3000, 50, -1200, true, "LEFT", 4509, 5709),
    (3000, 50, 1000, true, "LEFT", 4509, 3512),
    (3000, 0, 0, true, "LEFT", 4017, 4017),
    (3000, 0, -1200, true, "LEFT", 4017, 5217),
    (3000, 0, 1000, true, "LEFT", 4017, 3008),
    (3000, 100, 0, true, "LEFT", 5013, 5013),
    (3000, 100, -1200, true, "LEFT", 5013, 6214),
    (3000, 100, 1000, true, "LEFT", 5013, 4017),
    (0, 50, 0, false, "LEFT", 1507, 6),
    (0, 50, -1200, false, "LEFT", 1507, 1207),
    (0, 50, 1000, false, "LEFT", 2516, 6),
    (0, 50, 0, true, "CENTER", 1507, 1507),
    (3000, 50, 0, true, "CENTER", 4509, 4509),
    (0, 50, 0, true, "RIGHT", 1507, 1507),
    (3000, 50, 0, true, "RIGHT", 4509, 4509),
];

fn head_label(case: &(i32, i32, i32, bool, &str, i32, i32)) -> String {
    let (wa, to, ind, auto, align, _, _) = case;
    format!("보정 {wa} 거리 {to}% 들여쓰기 {ind} 자동 {auto} {align}")
}

/// 줄 나눔 상자는 머리 모양이 정한 줄별 본문 시작을 뺀 폭이다.
///
/// 종전에는 마커 글자와 공백 한 칸의 폭을 모든 줄에서 똑같이 뺐다 — 너비 보정·본문과의 거리
/// 단위·자동 내어쓰기를 몰라, 들여쓰기가 양수이거나 보정이 있는 문단의 줄이 한/글과 달랐다.
/// 수정 전(12d08c5ea, 마커+공백 한 칸 폭을 모든 줄에서 뺌)은 43문단 중 9문단만 한/글과 같았다.
#[test]
fn list_marker_head_sets_the_line_box() {
    let hancom = line_starts(HEAD_HANCOM, HEAD_CASES.len());
    let rhwp = line_starts(HEAD_NO_CACHE, HEAD_CASES.len());
    let mismatched: Vec<String> = HEAD_CASES
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(_, (h, r))| h != r)
        .map(|(case, (h, r))| format!("{}: 한/글 {h:?} / rhwp {r:?}", head_label(case)))
        .collect();
    assert!(
        mismatched.is_empty(),
        "머리 모양 문단의 줄이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 첫 줄과 둘째 줄의 본문 시작이 한/글 PDF 와 같다.
///
/// | | 첫 줄 본문 | 이어지는 줄 |
/// |---|---|---|
/// | 자동 내어쓰기 | `max(영역, max(들여쓰기,0))` | 첫 줄 − 들여쓰기 |
/// | 끔 | `max(들여쓰기,0) + 영역` | `max(−들여쓰기,0)` |
///
/// 영역 = 마커 글자 폭 + 너비 보정 + 글자 크기 × 거리%.
#[test]
fn list_marker_head_places_first_and_following_lines() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(HEAD_NO_CACHE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("fixture 읽기")).expect("문서 로드");
    let section = &core.document().sections[0];
    let page = &section.section_def.page_def;
    let body_left_hu = (page.margin_left + page.margin_gutter) as f64 + 4000.0;
    let x_hu = |para: usize, offset: usize| -> f64 {
        let json = core
            .get_cursor_rect_native(0, para, offset)
            .unwrap_or_else(|e| panic!("문단 {para} 캐럿 {offset}: {e:?}"));
        let value: serde_json::Value = serde_json::from_str(&json).expect("캐럿 JSON");
        value["x"].as_f64().expect("캐럿 x") * 75.0 - body_left_hu
    };
    // PDF 원점은 문단 여백에서 6 HWPUNIT 떨어져 있다(마커가 여백에 붙은 문단 전부 6).
    const PDF_ORIGIN_HU: f64 = 6.0;
    // PDF 글자 원점은 HWPUNIT 로 ±8 안팎 흔들린다(같은 영역의 문단끼리도 1501·1507·1509).
    // 종전 규칙(마커+공백 한 칸, 보정·단위·자동 내어쓰기 무시)은 이 문단들에서 수백~1000
    // HWPUNIT 어긋난다.
    const TOLERANCE_HU: f64 = 15.0;
    let mut mismatched = Vec::new();
    for (i, case) in HEAD_CASES.iter().enumerate() {
        let para = i + 1;
        let paragraph = &section.paragraphs[para];
        let second_start = paragraph.line_segs[1].text_start as usize;
        let chars: Vec<char> = paragraph.text.chars().collect();
        let is_syllable = |i: usize| ('가'..='힣').contains(&chars[i]);
        assert!(
            is_syllable(4) && is_syllable(5) && is_syllable(second_start),
            "{}: 첫 낱말이나 둘째 줄 첫 글자가 음절이 아니다 — 시험 설정 오류",
            head_label(case)
        );
        let first = x_hu(para, 0);
        // 줄 경계의 캐럿은 앞 줄 끝에 설 수 있으므로 둘째 줄 첫 글자 뒤 캐럿에서 한 음절 폭을
        // 뺀다. 음절 폭은 첫 줄 첫 낱말(`B00 가나다`) 안 두 음절 사이 간격이다 — 양쪽 정렬은
        // 공백만 늘린다.
        let syllable = x_hu(para, 5) - x_hu(para, 4);
        let second = x_hu(para, second_start + 1) - syllable;
        let (want_first, want_second) =
            (case.5 as f64 - PDF_ORIGIN_HU, case.6 as f64 - PDF_ORIGIN_HU);
        if (first - want_first).abs() > TOLERANCE_HU || (second - want_second).abs() > TOLERANCE_HU
        {
            mismatched.push(format!(
                "{}: 한/글 ({want_first:.0}, {want_second:.0}) / rhwp ({first:.0}, {second:.0})",
                head_label(case)
            ));
        }
    }
    assert!(
        mismatched.is_empty(),
        "머리 모양 문단의 본문 시작이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

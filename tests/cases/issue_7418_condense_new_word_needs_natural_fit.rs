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
//! # 검사에서 빼는 두 문단 — 줄바꿈 여유 대역
//!
//! A 의 condense 15·30 문단은 뺀다. 한 줄씩, condense 후 후보 폭이 상자를 **5·30 HWPUNIT**
//! 넘는데 rhwp 의 줄바꿈 여유(`line_break_tolerance_hwp`, 이 상자에서 +50 HWPUNIT) 안이라
//! rhwp 는 받고 한/글은 거절한다.
//!
//! ```text
//!   A c15 줄12: 자연 43500 → condense 후 42525   (상자 42520, 여유 끝 42570)
//!   A c30 줄13: 자연 44500 → condense 후 42550
//! ```
//!
//! 이 여유는 rhwp 의 폭 **추정** 오차를 흡수하려고 코퍼스로 정한 별개 계약이다. 이 검사는 그
//! 계약을 잠그지 않는다.
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
/// 줄바꿈 여유 대역에 줄이 걸려 이 검사에서 빼는 문단(모듈 주석 참조).
const IN_TOLERANCE_BAND: [&str; 2] = ["A 글자 c15", "A 글자 c30"];

fn line_starts(rel: &str) -> Vec<Vec<u32>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(
        paragraphs.len(),
        LABELS.len() + 1,
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
    let hancom = line_starts(HANCOM);
    let rhwp = line_starts(NO_CACHE);

    // 전제: condense 가 실제로 줄 끊음을 바꾸는 입력이어야 한다. 같은 글을 쓴 A 의 c0 과
    // c50 이 한/글에서 갈리지 않으면 이 검사는 아무것도 잠그지 않는다.
    assert_ne!(
        hancom[0], hancom[3],
        "정답지 전제가 깨졌다 — condense 0 과 50 의 줄이 같다"
    );

    let mismatched: Vec<String> = LABELS
        .iter()
        .zip(hancom.iter().zip(rhwp.iter()))
        .filter(|(label, _)| !IN_TOLERANCE_BAND.contains(label))
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
    let hancom = line_starts(HANCOM);
    let rhwp = line_starts(NO_CACHE);
    for (i, label) in LABELS.iter().enumerate() {
        if label.ends_with(" c0") {
            assert_eq!(rhwp[i], hancom[i], "{label}: condense 0 문단이 한/글과 달라졌다");
        }
    }
}

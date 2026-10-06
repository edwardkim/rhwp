//! [#7288] 잉크 없는 빈 문단이 쪽번호만 남은 쪽을 열지 않는다 — 정본 쪽 구성 일치.
//!
//! # 무엇이 깨져 있었나
//!
//! 값 1 «셀 단위로 나눔» 을 정본대로 적용하면 `참고2` 표 조각이 커진다(638.6 → 774.8px).
//! 그러면 표 뒤를 따르던 **빈 문단 `pi=101`** 이 잔여를 조금 넘겨 다음 쪽으로 밀리는데,
//! 바로 다음 `pi=102`(`참고3` 표)는 저장 vpos 가 본문을 넘어 어차피 새 쪽에서 시작한다.
//! 그 사이에 낀 빈 문단 하나가 **꼬리말 `- 8 -` 만 있는 쪽**을 열고 이후 전 쪽이 +1 밀렸다.
//!
//! ```text
//!   pi=101  빈 문단  저장 vpos 71584 -> 954.5px
//!   pi=102  참고3 표  저장 vpos 73760 -> 983.5px  > 본문 977.8px  ← 이 쪽에 있을 수 없다
//! ```
//!
//! [#6854] 가 **같은 형상**(잉크 없는 빈 문단이 미세 초과로 고아 쪽을 염)을 이미 막고
//! 있었고 대상만 좁았다 — 「선언된」 쪽나누기(`column_type`)만 인정했다. 여기서 경계를
//! 만드는 것은 [#6132] 이 쓰는 사실, 곧 **다음 문단의 저장 vpos 가 본문 높이를 넘는다**는
//! 문서가 적은 값이다. 사다리 패턴에서 추론한 경계가 아니므로 그 주석의 경고
//! ("추론 경계까지 받으면 쪽 이득 없이 넘침만 는다")에 해당하지 않는다.
//!
//! # 기대값의 출처
//!
//! 정본 `pdf/156482639_startup_ir_contest-2020.pdf` 는 **10쪽**이다. 같은 원문을 engine
//! 2024 로도 변환해 **둘 다 10쪽**임을 확인했으므로 버전 차이가 아니다(2020 Hancom
//! 11.0.0.9136 · 2024 Hancom 13.0.0.3901).
//!
//! 쪽수만으로는 쪽 귀속을 증명하지 못하므로 **쪽별 글자 수**로 대조한다. 수정 뒤 10쪽
//! 전부 정본과 일치한다 — 850 / 705 / 126 / 831 / 558 / 626 / 566 / 942 / 26 / 325.
//! 종전에는 5·6·7쪽이 각각 631 / 652 / 467 로 어긋났고(표를 행 안에서 잘라 더 채움),
//! 값 1 만 적용한 중간 상태에서는 8쪽이 빈 쪽이 되어 11쪽이었다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6132/156482639_startup_ir_contest.hwp";
/// 정본 `pdf/156482639_startup_ir_contest-2020.pdf` 의 쪽별 글자 수(공백 제외).
/// engine 2024 변환도 같은 10쪽이다.
const ORACLE_CHARS: [usize; 10] = [850, 705, 126, 831, 558, 626, 566, 942, 26, 325];

fn page_chars() -> Vec<usize> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    (0..core.page_count())
        .map(|page| {
            core.extract_page_text_native(page)
                .unwrap_or_default()
                .chars()
                .filter(|c| !c.is_whitespace())
                .count()
        })
        .collect()
}

/// 쪽 구성이 정본과 같다 — 빈 쪽이 끼지 않는다.
#[test]
fn page_composition_matches_the_hancom_oracle() {
    let got = page_chars();
    assert_eq!(
        got.len(),
        ORACLE_CHARS.len(),
        "정본은 {}쪽이다 — 빈 문단 하나가 쪽을 열면 여기서 늘어난다. got={got:?}",
        ORACLE_CHARS.len()
    );
    assert_eq!(
        got,
        ORACLE_CHARS.to_vec(),
        "쪽별 글자 수가 정본과 달라졌다. 쪽수만으로는 귀속을 증명하지 못하므로 이 검사가 \
         본체다. got={got:?} 정본={ORACLE_CHARS:?}"
    );
}

/// 어떤 쪽도 본문 글자가 0 이면 안 된다 — 꼬리말만 남은 쪽의 직접 검사.
#[test]
fn no_page_is_left_without_body_text() {
    let got = page_chars();
    let empty: Vec<usize> = got
        .iter()
        .enumerate()
        .filter(|(_, n)| **n == 0)
        .map(|(i, _)| i)
        .collect();
    assert!(
        empty.is_empty(),
        "본문 글자가 없는 쪽이 있다 — 잉크 없는 빈 문단이 연 고아 쪽이다. \
         빈 쪽(0-based)={empty:?} 쪽별 글자 수={got:?}"
    );
}

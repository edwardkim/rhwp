//! 관계형: 재조판한 줄 시작이 한/글 2020·2024 가 저장한 줄 시작과 글자 단위로 같다.
//! 주석 갱신: 2026-11-06T01:10:00+09:00 (새 파일).
//! 합성 입력(시각 출력 검증 아님): `samples/issue8639/trailing_space_overflow_synthetic.hwpx`
//! (생성기 `mydocs/tech/investigations/issue-8639/probes/make_trailing_space_overflow_fixture.py`,
//! 저장 줄 없음)를 MCP 로 한/글 2020(11.0.0.9136)·2024(13.0.0.3901)가 열어 저장한 본
//! `…-hancom-2020.hwp`·`…-hancom-2024.hwp` 의 LineSeg 가 정답이다. 같은 작업의 Print PDF
//! `pdf/issue8639/trailing_space_overflow_synthetic-{2020,2024}.pdf`(각 1쪽)와 줄 수가 같다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const NO_CACHE: &str = "samples/issue8639/trailing_space_overflow_synthetic.hwpx";
const HANCOM: [&str; 2] = [
    "samples/issue8639/trailing_space_overflow_synthetic-hancom-2020.hwp",
    "samples/issue8639/trailing_space_overflow_synthetic-hancom-2024.hwp",
];
/// X = 한글 41자 + `W1`: 마지막 글자가 상자(42520 HU) 안에 겨우 들고 첫 공백부터 넘친다.
const LABELS: [&str; 9] = [
    "X + 공백 1",
    "X + 공백 2",
    "X + 공백 1 + 굵게 공백 1",
    "X + 공백 3",
    "42자 + 공백 2(#7418 대조군)",
    "42자 + 공백 3(#7418 대조군)",
    "X + 공백 1 + 글",
    "X + 공백 2 + 글",
    "X + 공백 3 + 글",
];

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

/// 넘친 공백이 낱말 바로 뒤의 첫 공백이면 다음 공백 하나도 같은 줄 끝에 걸린다.
///
/// 한/글은 마지막 글자 뒤 공백을 둘까지 같은 줄에 둔다 — 첫 공백부터 넘쳐도 공백 2 는 1줄,
/// 공백 3 은 셋째 공백에서, 공백 2 + 글은 글에서 새 줄을 시작한다. 종전 rhwp 는 첫 공백만
/// 걸고 둘째 공백을 새 줄로 넘겨 공백만 든 줄을 만들었다(docs#636 중간보고서 41쪽 표 칸).
/// 첫 공백이 상자 안에 드는 #7418 경계(42자 + 공백 2·3)는 그대로다.
#[test]
fn word_attached_overflowing_space_hangs_with_the_next_space() {
    let rhwp = line_starts(NO_CACHE);
    for hancom_rel in HANCOM {
        let hancom = line_starts(hancom_rel);
        // 전제: 정답지가 이 경계를 실제로 가른다.
        assert_eq!(
            hancom[1],
            vec![0],
            "{hancom_rel}: X + 공백 2 가 1줄이 아니다"
        );
        assert_eq!(
            hancom[3],
            vec![0, 45],
            "{hancom_rel}: X + 공백 3 이 셋째 공백에서 갈리지 않는다"
        );
        let mismatched: Vec<String> = LABELS
            .iter()
            .zip(hancom.iter().zip(rhwp.iter()))
            .filter(|(_, (h, r))| h != r)
            .map(|(label, (h, r))| format!("{label}: 한/글 {h:?} / rhwp {r:?}"))
            .collect();
        assert!(
            mismatched.is_empty(),
            "{hancom_rel} 와 끝 공백 줄이 다르다:\n{}",
            mismatched.join("\n")
        );
    }
}

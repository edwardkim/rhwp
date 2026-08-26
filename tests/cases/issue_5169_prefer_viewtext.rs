//! [Issue #5169] `BodyText` 와 `ViewText` 가 갈라진 비배포 문서에서 rhwp 가 한글이
//! 렌더하지 않는 `BodyText` 를 읽던 문제.
//!
//! HWP5 문서에 `BodyText` 와 `ViewText` 가 둘 다 있고 **배포용 플래그가 꺼져 있으면**
//! (예: 변경 추적 0x4000), 종전 규칙은 배포용(0x04)일 때만 `ViewText` 를 읽어서 rhwp 가
//! `BodyText` 를, 한글이 `ViewText` 를 보는 어긋남이 생겼다. 두 스트림이 크게 다르면 rhwp 는
//! 문서의 다른 판본을 읽는다.
//!
//! 재현 표본 `samples/issue5169_viewtext_changetracking.hwp` (외교부 연구계획서, PRISM 공개):
//! `FileHeader` = 압축 + 변경 추적(0x4001, 배포용 아님). `BodyText/Section0` 은 표 4개·3,540자,
//! `ViewText/Section0` 은 표 10개·개체 2개·10,210자다. 한글은 `ViewText` 를 렌더한다.
//!
//! 계약: `ViewText` 가 존재하고 정상 복호되면 rhwp 도 그것을 읽어야 한다 — 표 개수가
//! `ViewText`(10) 쪽이어야 하고 `BodyText`(4) 여선 안 된다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

const SAMPLE: &str = "samples/issue5169_viewtext_changetracking.hwp";

/// 문단(셀 안 포함)에 든 표 컨트롤 수를 재귀적으로 센다.
fn count_tables(paras: &[Paragraph]) -> usize {
    let mut n = 0;
    for p in paras {
        for c in &p.controls {
            if let Control::Table(t) = c {
                n += 1;
                for cell in &t.cells {
                    n += count_tables(&cell.paragraphs);
                }
            }
        }
    }
    n
}

#[test]
fn hwp5_prefers_viewtext_over_bodytext_when_not_distribution() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join(SAMPLE);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let doc = rhwp::parser::parse_document(&data).expect("파싱");
    let tables: usize = doc
        .sections
        .iter()
        .map(|s| count_tables(&s.paragraphs))
        .sum();

    // ViewText(표 10) 를 읽으면 8 이상, BodyText(표 4) 를 읽으면 4 근처다.
    assert!(
        tables >= 8,
        "표 {tables}개 — BodyText(4)를 읽고 있다. ViewText(10)를 우선해야 한다 (#5169 회귀)"
    );
}

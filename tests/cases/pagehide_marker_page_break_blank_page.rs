//! 구역 시작 PageHide marker 뒤 빈 문단의 쪽 나누기는 빈 쪽을 만든다.
//!
//! `2025 행정업무운영 편람(최종)` 구역 10(제5장)과 구역 11(부록)은 같은 꼴로 시작한다.
//!
//! ```text
//!   p0  구역나누기 + 감추기(PageHide) + 장 간지 묶음
//!   p1  빈 문단
//!   p2  쪽나누기 + 감추기(PageHide)   ← 빈 문단, 개체 없음
//!   p3  쪽나누기 + 장식 묶음 host     ← 다음 내용
//! ```
//!
//! 독립 정본 `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`(한컴 2024 + KoPub 내장,
//! 383쪽; hwpx 판도 같다)은 p2 가 연 쪽을 비워 둔다 — 간지(277·309쪽) 바로 뒤 278·310쪽에
//! 글자가 하나도 없고, p3 의 내용은 279·311쪽에서 시작한다. 같은 문서의 KoPub 미설치 출력
//! (`-hwp-2024.pdf`, 384쪽)도 같은 자리에 빈 쪽이 있다.
//!
//! 종전 rhwp 는 p2 를 「앞 marker 가 이미 쪽을 열었으니 중복」으로 보고 배치하지 않아 p3 이
//! 그 쪽을 차지했고, 제5장 이후 내용이 한 쪽씩 앞당겨졌다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLES: [&str; 2] = [
    "samples/2025 행정업무운영 편람(최종).hwp",
    "samples/2025 행정업무운영 편람(최종).hwpx",
];

fn page_texts(sample: &str) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    (0..core.page_count())
        .map(|page| {
            core.extract_page_text_native(page)
                .unwrap_or_default()
                .split_whitespace()
                .collect::<String>()
        })
        .collect()
}

#[test]
fn chapter_five_divider_is_followed_by_the_pagehide_blank_page() {
    for sample in SAMPLES {
        let pages = page_texts(sample);
        // 정본 277쪽: 제5장 간지(목차 세 줄).
        assert!(
            pages[276].contains("질의및답변") && pages[276].contains("3.관인관리분야"),
            "{sample}: 277쪽은 제5장 간지여야 한다: {:?}",
            pages[276].chars().take(40).collect::<String>()
        );
        // 정본 278쪽: 빈 쪽.
        assert!(
            pages[277].is_empty(),
            "{sample}: 278쪽은 PageHide 빈 쪽이어야 한다: {:?}",
            pages[277].chars().take(40).collect::<String>()
        );
        // 정본 279쪽: 제5장 질의 목차가 시작한다.
        assert!(
            pages[278].contains("1.문서관리분야")
                && pages[278].contains("1.공문서란무엇을말하나요?"),
            "{sample}: 279쪽은 제5장 질의 목차로 시작해야 한다: {:?}",
            pages[278].chars().take(40).collect::<String>()
        );
    }
}

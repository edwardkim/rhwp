//! [#8081] 쪽 나눔 표의 각주가 모두 첫 조각 밖 행에 있으면 첫 조각은 그 각주 공간을
//! 예약하지 않는다.
//!
//! 입력 `samples/issue8081/span_{36,38}.hwpx` 는 생성기 `gen_fn_table.py` 의 출력이다.
//! 채움 문단 뒤 쪽 바닥 근처에서 2열 24행 표(왼쪽 열 10행 병합)가 시작하고, 각주 여섯 개는
//! 14~19행 오른쪽 칸에 있다. 정본 `pdf/issue8081/*-2024.pdf`(한/글 2024 13.0.0.564 Print)는
//! 두 입력 모두 2쪽이고, 1쪽 바닥에 표 첫 행들을 둔 뒤 각주 행부터는 2쪽에 둔다.
//!
//! | 입력 | 한/글 1쪽 마지막 행 | 수정 전 rhwp 1쪽 |
//! |---|---|---|
//! | `span_36` | R06 | R00 만 — 첫 조각 일반 행 프레임 상한이 예약을 뺀 0 |
//! | `span_38` | R04 | 표 없음 — 첫 유닛 판정이 표 전체 각주 예약으로 표를 통째 이월 |
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const ROWS: usize = 24;

fn compact_page_text(core: &DocumentCore, page: u32) -> String {
    core.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트 추출: {e}", page + 1))
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

#[test]
fn rows_before_the_first_footnote_anchor_fill_the_opening_page() {
    for (name, last_on_page1) in [("span_36", 6usize), ("span_38", 4)] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue8081")
            .join(format!("{name}.hwpx"));
        let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse");
        assert_eq!(core.page_count(), 2, "{name}: 한/글 2024 정본 쪽수");
        let page1 = compact_page_text(&core, 0);
        let page2 = compact_page_text(&core, 1);
        for row in 0..ROWS {
            let label = format!("행R{row:02}내용");
            let (owner, other, owner_no) = if row <= last_on_page1 {
                (&page1, &page2, 1)
            } else {
                (&page2, &page1, 2)
            };
            assert!(
                owner.matches(label.as_str()).count() == 1 && !other.contains(label.as_str()),
                "{name}: {label} 은 한/글처럼 {owner_no}쪽에 한 번만 있어야 한다\n--- 1쪽 ---\n{page1}\n--- 2쪽 ---\n{page2}"
            );
        }
        assert!(
            page2.contains("표뒤TAIL") && !page1.contains("표뒤TAIL"),
            "{name}: 표 뒤 문단은 2쪽"
        );
    }
}

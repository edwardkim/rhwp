//! [#7695] 각주가 있는 쪽 끝, 저장 줄이 각주 영역 위에 있는 여러 줄 문단.
//!
//! 쪽에 각주가 있으면 분할 예산에서 각주 안전마진(40px)이 빠진다. 저장 줄이 각주 영역 위에
//! 있으면 그 마진 몫의 초과를 통과시키는 판정(#4054)이 있는데, 저장 줄과 대조하는 흐름 위치로
//! **문단 시작**을 써서 조각 둘째 줄부터는 앵커 대조가 늘 실패했다. 그래서 한/글이 그 쪽에 두는
//! 둘째·셋째 줄을 rhwp 는 다음 쪽으로 넘겨 쪽 경계가 한/글과 어긋났다. 컨트롤이 없는 문단은
//! 별도의 각주 저장 리셋 갈래가 받아 주므로, 이 입력은 여러 줄 문단에 책갈피를 넣었다.
//!
//! 입력 `samples/issue7695/footnote-page-tail-2024.hwp` 는 생성기 `gen_footnote_page_tail.py` 의
//! HWPX(동봉)를 한/글 2024(13.0.0.564)로 HWP 저장한 것이고, 정본
//! `pdf/issue7695/footnote-page-tail-2024.pdf` 는 같은 한/글의 Print 출력이다(32쪽). 정본의 쪽별
//! 본문은 각 여러 줄 문단을 **저장 사다리가 쪽 첫머리(vpos 되감김)로 표시한 줄** 앞에서 나눈다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue7695/footnote-page-tail-2024.hwp";

#[test]
fn multi_line_paragraph_splits_at_stored_reset_above_footnote_area() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    assert_eq!(core.page_count(), 32, "정본 PDF 와 같은 32쪽");

    let paragraphs = &core.document().sections[0].paragraphs;
    // 저장 사다리가 문단 안에서 되감기는 줄(다음 쪽 첫머리) — 여러 줄 문단마다 하나.
    let resets: Vec<(usize, usize)> = paragraphs
        .iter()
        .enumerate()
        .filter_map(|(index, para)| {
            para.line_segs
                .windows(2)
                .position(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
                .map(|line| (index, line + 1))
        })
        .collect();

    let pages = core.dump_page_items_json(None);
    let first_fragment_end = |para_index: usize| -> Option<(u64, usize)> {
        pages.as_array()?.iter().find_map(|page| {
            page["columns"].as_array()?.iter().find_map(|column| {
                column["items"].as_array()?.iter().find_map(|item| {
                    (item["kind"] == "partialParagraph"
                        && item["paraIndex"].as_u64() == Some(para_index as u64)
                        && item["startLine"].as_u64() == Some(0))
                    .then(|| {
                        (
                            page["pageIndex"].as_u64().unwrap_or(u64::MAX),
                            item["endLine"].as_u64().unwrap() as usize,
                        )
                    })
                })
            })
        })
    };

    let mut checked = 0;
    for (para_index, reset_line) in resets {
        let Some((page, end_line)) = first_fragment_end(para_index) else {
            panic!("문단 {para_index}: 저장 되감김(줄 {reset_line})이 있는데 쪽에서 나뉘지 않았다");
        };
        assert_eq!(
            end_line, reset_line,
            "문단 {para_index}: {page}쪽 조각은 저장 사다리가 다음 쪽 첫머리로 둔 줄 {reset_line} \
             앞에서 끝나야 한다(각주 영역 위 저장 줄을 안전마진으로 넘기지 않는다)"
        );
        checked += 1;
    }
    assert!(
        (8..=16).contains(&checked),
        "검사한 쪽 끝 분할 문단 수가 사례 범위를 벗어났다: {checked}"
    );
}

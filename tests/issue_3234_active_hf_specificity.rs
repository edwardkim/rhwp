//! Task #3234: 쪽별 활성 머리말은 **구체성**으로 고르고, 등장 순서에 좌우되지 않는다.
//!
//! 홀수/짝수 전용은 양 쪽보다 구체적이므로 해당 홀짝 쪽에서 우선한다. 한 변수에 덮어쓰며
//! 누적하면 "마지막에 일치한 것" 이 이겨서, **양 쪽 머리말을 나중에 추가했다는 이유만으로
//! 홀수 전용 머리말이 홀수 쪽에서 사라진다.**
//!
//! `renderer/typeset.rs::finalize_pages` 가 그 형태였다 — 주석은 `engine.rs` 와 "동일" 이라고
//! 적혀 있었지만 규칙이 갈라져 있었고, 이제 두 경로가 `ActiveHeaderFooter` 를 공유한다.
//!
//! 같은 규칙의 사본이 `queries/rendering.rs` 의 구역 보정에도 있었다. 그쪽은 홀짝만 보고
//! 구체성을 몰라서, 구역 간 쪽번호 이어짐(carry)이 홀수인 다구역 문서에서 같은 증상이
//! 남았다 — 이제 그 자리도 `ActiveHeaderFooter` 를 쓴다.
//!
//! 검증은 **실제로 그려지는 글자**로 한다. 두 머리말에 서로 다른 표식을 넣고 쪽 렌더 트리에
//! 어느 쪽 글자가 나오는지 본다 — 사용자가 겪는 증상 그대로다.

use rhwp::wasm_api::HwpDocument;

const ODD_MARK: &str = "홀수머리말";
const BOTH_MARK: &str = "양쪽머리말";
const EVEN_MARK: &str = "짝수머리말";

/// `pages` 쪽 문서를 만들고, 머리말을 `order` 순서대로 생성해 각자 표식을 넣는다.
fn doc_with_headers(pages: usize, order: &[(u8, &str)]) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank");
    doc.insert_text_native(0, 0, 0, "본문").expect("text");
    for i in 0..pages - 1 {
        doc.insert_page_break_native(0, i, if i == 0 { 2 } else { 0 })
            .expect("page break");
    }
    for (apply_to, mark) in order {
        doc.create_header_footer_native(0, true, *apply_to)
            .expect("create header");
        doc.insert_text_in_header_footer_native(0, true, *apply_to, 0, 0, mark)
            .expect("fill header");
    }
    doc
}

/// 구역 0 을 `first_pages` 쪽으로 두고, 머리말은 **구역 1** 에만 `order` 순서로 만든다.
///
/// 구역 1 의 쪽번호는 구역 0 의 마지막 번호에서 이어진다 — `first_pages` 가 홀수면
/// 구역 1 의 지역 홀짝이 최종 홀짝과 반대가 된다.
fn doc_two_sections(first_pages: usize, order: &[(u8, &str)]) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank");
    doc.insert_text_native(0, 0, 0, "본문").expect("text");
    for i in 0..first_pages - 1 {
        doc.insert_page_break_native(0, i, if i == 0 { 2 } else { 0 })
            .expect("구역 0 쪽 나누기");
    }

    // 구역 1 = 구역 0 복제 (구역 나누기 편집 API 가 없어 IR 로 만든다)
    let mut ir = doc.document().clone();
    let sec0 = ir.sections[0].clone();
    ir.sections.push(sec0);
    doc.set_document(ir);

    doc.insert_page_break_native(1, 0, 2)
        .expect("구역 1 쪽 나누기");
    for (apply_to, mark) in order {
        doc.create_header_footer_native(1, true, *apply_to)
            .expect("create header");
        doc.insert_text_in_header_footer_native(1, true, *apply_to, 0, 0, mark)
            .expect("fill header");
    }
    doc
}

/// 그 쪽 렌더 트리에 어떤 머리말 표식이 그려지는지.
fn drawn_mark(doc: &HwpDocument, page: u32) -> Option<&'static str> {
    let tree = doc.get_page_render_tree(page).expect("render tree");
    for mark in [ODD_MARK, BOTH_MARK, EVEN_MARK] {
        // 렌더 트리는 글자를 run 단위로 쪼개 실을 수 있어, 표식의 앞 두 글자로 찾는다.
        let probe: String = mark.chars().take(2).collect();
        if tree.contains(&probe) {
            return Some(mark);
        }
    }
    None
}

#[test]
fn odd_header_wins_on_odd_pages_regardless_of_creation_order() {
    let orders: [&[(u8, &str)]; 2] = [
        &[(2, ODD_MARK), (0, BOTH_MARK)], // 홀수 먼저 — 수정 전 실패하던 순서
        &[(0, BOTH_MARK), (2, ODD_MARK)], // 양 쪽 먼저
    ];

    for pages in [2usize, 3, 4] {
        for order in orders {
            let doc = doc_with_headers(pages, order);
            for p in 0..pages as u32 {
                let page_number = p + 1; // 쪽 인덱스 0 = 쪽 번호 1
                let expected = if page_number % 2 == 1 {
                    ODD_MARK
                } else {
                    BOTH_MARK
                };
                assert_eq!(
                    drawn_mark(&doc, p),
                    Some(expected),
                    "pages={} 생성순서={:?} 쪽번호={} — 홀수 쪽은 홀수 전용이 그려져야 한다",
                    pages,
                    order.iter().map(|(a, _)| *a).collect::<Vec<_>>(),
                    page_number
                );
            }
        }
    }
}

/// 구역 간 쪽번호 이어짐(carry)이 홀짝을 뒤집어도 구체성이 이긴다.
///
/// pagination 은 구역마다 1부터 세는 **지역** 쪽번호로 머리말을 고르고, 앞 구역 번호를
/// 이어 붙이는 일은 그 뒤에 일어난다. 그래서 carry 가 홀수면 지역 짝수 쪽에 배정된
/// `양 쪽` 이 최종 홀수 쪽에 그대로 남아, 같은 구역의 `홀수` 전용이 사라졌다.
///
/// 같은 문서 모양에서 **앞 구역 쪽수만 1↔2 로 바꿔도 결과가 갈리는 것**이 이 결함의 표식이다.
#[test]
fn odd_header_wins_across_section_page_number_carry() {
    for first_pages in [1usize, 2] {
        let doc = doc_two_sections(first_pages, &[(2, ODD_MARK), (0, BOTH_MARK)]);
        for k in 0..2u32 {
            let page_idx = first_pages as u32 + k;
            let page_number = page_idx + 1;
            let expected = if page_number % 2 == 1 {
                ODD_MARK
            } else {
                BOTH_MARK
            };
            assert_eq!(
                drawn_mark(&doc, page_idx),
                Some(expected),
                "앞 구역 {}쪽 → 최종 {}쪽 — 홀짝에 더 구체적인 머리말이 그려져야 한다",
                first_pages,
                page_number
            );
        }
    }
}

/// 홀수 전용만 있으면 짝수 쪽에는 아무것도 그려지지 않아야 한다.
///
/// 수정 전에도 통과하던 성질이다(고친 결함이 아니라 **유지해야 할 성질**) — 누적 방식을
/// 종류별 칸으로 바꾸면서 홀짝 판정이 흐트러지지 않았는지 함께 고정한다.
#[test]
fn odd_only_header_does_not_leak_onto_even_pages() {
    let doc = doc_with_headers(2, &[(2, ODD_MARK)]);
    assert_eq!(drawn_mark(&doc, 0), Some(ODD_MARK), "1쪽(홀수)에 홀수 전용");
    assert_eq!(drawn_mark(&doc, 1), None, "2쪽(짝수)에는 그려지지 않는다");
}

#[test]
fn even_only_header_does_not_leak_onto_odd_pages() {
    let doc = doc_with_headers(2, &[(1, EVEN_MARK)]);
    assert_eq!(
        drawn_mark(&doc, 0),
        None,
        "1쪽(홀수)에는 적용되는 머리말이 없다"
    );
    assert_eq!(
        drawn_mark(&doc, 1),
        Some(EVEN_MARK),
        "2쪽(짝수)에는 짝수 전용이 그려진다"
    );
}

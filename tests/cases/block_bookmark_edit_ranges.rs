//! 한컴 12.30 꼴 블록 책갈피(%bmk 필드)가 글 편집에도 범위를 지키는지 본다.
//! 한컴 12.30에서 확인: 블록 경계에서 친 글은 블록 밖, 블록 안에서 나눈 문단은 두 문단에 걸친다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

/// "가나다라마바사"·"아자차카타파하", 블록 책갈피 "한" = 첫 문단 2..5("다라마").
fn fixture() -> DocumentCore {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/block_bookmarks/one-paragraph-block.hwp"
    ))
    .unwrap();
    DocumentCore::from_bytes(&bytes).unwrap()
}

/// 본문 블록 책갈피의 (문단, 시작, 끝).
fn field_range(core: &DocumentCore) -> Vec<(u64, u64, u64)> {
    serde_json::from_str::<Value>(&core.get_field_list_json())
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["fieldType"] == "bookmark")
        .map(|f| {
            (
                f["location"]["paraIndex"].as_u64().unwrap(),
                f["startCharIdx"].as_u64().unwrap(),
                f["endCharIdx"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn reopened(core: &DocumentCore, hwpx: bool) -> DocumentCore {
    let bytes = if hwpx {
        core.export_hwpx_native().unwrap()
    } else {
        core.export_hwp_native().unwrap()
    };
    DocumentCore::from_bytes(&bytes).unwrap()
}

#[test]
fn typing_at_block_edges_stays_outside() {
    let mut core = fixture();
    assert_eq!(field_range(&core), [(0, 2, 5)]);
    core.insert_text_native(0, 0, 2, "앞").unwrap();
    // "가나앞[다라마]바사": 시작에 친 글은 블록 밖이다.
    assert_eq!(field_range(&core), [(0, 3, 6)]);
    core.insert_text_native(0, 0, 6, "뒤").unwrap();
    assert_eq!(field_range(&core), [(0, 3, 6)]);
    for hwpx in [false, true] {
        assert_eq!(
            field_range(&reopened(&core, hwpx)),
            [(0, 3, 6)],
            "hwpx={hwpx}"
        );
    }
}

#[test]
fn splitting_inside_a_block_and_merging_back_keeps_its_range() {
    let mut core = fixture();
    // "가나다|라마바사": 블록이 두 문단에 걸쳤다가, 다시 합치면 처음 범위로 돌아온다.
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    for hwpx in [false, true] {
        let mut saved = reopened(&core, hwpx);
        saved.merge_paragraph_native(0, 1).unwrap();
        assert_eq!(field_range(&saved), [(0, 2, 5)], "hwpx={hwpx}");
    }
    core.merge_paragraph_native(0, 1).unwrap();
    assert_eq!(field_range(&core), [(0, 2, 5)]);
}

/// 두 블록 책갈피의 문서: 한 문단 블록 "한"(0,2)-(0,5)과 두 문단에 걸친 "여러"(0,6)-(1,3).
fn two_blocks() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다라마바사").unwrap();
    core.split_paragraph_native(0, 0, 7, None).unwrap();
    core.insert_text_native(0, 1, 0, "아자차카타파하").unwrap();
    let body = rhwp::document_core::queries::BookmarkPlace {
        section: 0,
        para: 0,
        cell_path: Vec::new(),
        header_footer: None,
    };
    for (start, end_para, end, name) in [(2, 0, 5, "한"), (6, 1, 3, "여러")] {
        let r = core
            .add_block_bookmark_native(&body, start, end_para, end, name)
            .unwrap();
        assert!(r.contains(r#""ok":true"#), "{r}");
    }
    core
}

/// (이름, 종류, 시작 문단, 시작, 끝 문단, 끝)
fn listed(core: &DocumentCore) -> Vec<(String, String, u64, u64, Option<u64>, Option<u64>)> {
    let items: Vec<serde_json::Value> =
        serde_json::from_str(&core.get_bookmarks_native().unwrap()).unwrap();
    items
        .iter()
        .map(|b| {
            (
                b["name"].as_str().unwrap().to_string(),
                b["kind"].as_str().unwrap().to_string(),
                b["place"]["para"].as_u64().unwrap(),
                b["start"].as_u64().unwrap(),
                b["endPara"].as_u64(),
                b["end"].as_u64(),
            )
        })
        .collect()
}

/// 지금 목록이 두 저장 형식을 거쳐도 같고 글도 같은가.
fn survives_save(core: &DocumentCore) {
    let texts = |core: &DocumentCore| -> Vec<String> {
        core.document().sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect()
    };
    for hwpx in [false, true] {
        let bytes = if hwpx {
            core.export_hwpx_native().unwrap()
        } else {
            core.export_hwp_native().unwrap()
        };
        let saved = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(listed(&saved), listed(core), "hwpx={hwpx}");
        assert_eq!(texts(&saved), texts(core), "hwpx={hwpx}");
    }
}

fn block(
    name: &str,
    para: u64,
    start: u64,
    end_para: u64,
    end: u64,
) -> (String, String, u64, u64, Option<u64>, Option<u64>) {
    (
        name.into(),
        "block".into(),
        para,
        start,
        Some(end_para),
        Some(end),
    )
}

#[test]
fn splitting_inside_a_block_makes_it_span_both_paragraphs() {
    let mut core = two_blocks();
    // 한컴 HWPX의 끝 표지 fieldid는 나누고 합쳐도 남는다.
    let paragraphs = &mut core.document_mut().sections[0].paragraphs;
    paragraphs[0].field_ranges[0].end_field_id = 777;
    // "가나다|라마바사": 한(2..5)은 두 문단에 걸치고, 여러는 둘째 문단으로 간다.
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 1, 2), block("여러", 1, 3, 2, 3)]
    );
    assert_eq!(
        core.document().sections[0].paragraphs[1].orphan_field_ends[0].field_id,
        777
    );
    survives_save(&core);
    // 나눈 문단을 다시 합치면 처음 범위로 돌아온다.
    core.merge_paragraph_native(0, 1).unwrap();
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 5), block("여러", 0, 6, 1, 3)]
    );
    assert!(core.document().sections[0].paragraphs[0]
        .field_ranges
        .iter()
        .any(|range| range.end_field_id == 777));
    survives_save(&core);
}

#[test]
fn splitting_after_a_spanning_block_end_moves_the_end_with_the_text() {
    let mut core = two_blocks();
    // "아|자차카타파하": 여러의 끝 표지(3)는 새 문단의 2로 간다.
    core.split_paragraph_native(0, 1, 1, None).unwrap();
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 5), block("여러", 0, 6, 2, 2)]
    );
    survives_save(&core);
    core.merge_paragraph_native(0, 2).unwrap();
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 5), block("여러", 0, 6, 1, 3)]
    );
    survives_save(&core);
}

#[test]
fn typing_at_block_edges_stays_outside_and_moves_later_edges() {
    let mut core = two_blocks();
    // 한컴 12.30: 블록 시작·끝 경계에서 친 글은 블록 밖이다. "가나|다라마|바사" → "가나X다라마Y바사"
    core.insert_text_native(0, 0, 5, "Y").unwrap();
    core.insert_text_native(0, 0, 2, "X").unwrap();
    // 두 문단에 걸친 블록의 시작(여러, 이제 "사" 앞 8) 경계와 끝 문단의 끝 앞에도 친다.
    core.insert_text_native(0, 0, 8, "Z").unwrap();
    core.insert_text_native(0, 1, 0, "W").unwrap();
    assert_eq!(
        core.document().sections[0].paragraphs[0].text,
        "가나X다라마Y바Z사"
    );
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 3, 0, 6), block("여러", 0, 9, 1, 4)]
    );
    survives_save(&core);
}

#[test]
fn deleting_text_pulls_block_edges_and_keeps_an_emptied_block() {
    let mut core = two_blocks();
    // 끝 문단의 끝 앞 글 하나("아")를 지우면 여러의 끝이 당겨진다.
    core.delete_text_native(0, 1, 0, 1).unwrap();
    // 한의 글("다라마")을 모두 지워도 한컴처럼 빈 블록으로 남는다.
    core.delete_text_native(0, 0, 2, 3).unwrap();
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 2), block("여러", 0, 3, 1, 2)]
    );
    survives_save(&core);
}

#[test]
fn typing_into_an_emptied_block_fills_it_and_keeps_later_edges() {
    let mut core = two_blocks();
    core.delete_text_native(0, 0, 2, 3).unwrap();
    // 한컴 12.30처럼 블록 글을 모두 지운 자리에 친 글은 그 블록에 든다. 뒤 블록 시작도 글과 함께 민다.
    core.insert_text_native(0, 0, 2, "새").unwrap();
    assert_eq!(core.document().sections[0].paragraphs[0].text, "가나새바사");
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 3), block("여러", 0, 4, 1, 3)]
    );
    survives_save(&core);

    // 문단 끝의 빈 블록도 친 글을 품는다.
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다").unwrap();
    let body = rhwp::document_core::queries::BookmarkPlace {
        section: 0,
        para: 0,
        cell_path: Vec::new(),
        header_footer: None,
    };
    let r = core
        .add_block_bookmark_native(&body, 1, 0, 3, "끝")
        .unwrap();
    assert!(r.contains(r#""ok":true"#), "{r}");
    core.delete_text_native(0, 0, 1, 2).unwrap();
    core.insert_text_native(0, 0, 1, "새").unwrap();
    assert_eq!(core.document().sections[0].paragraphs[0].text, "가새");
    assert_eq!(listed(&core), vec![block("끝", 0, 1, 0, 2)]);
    survives_save(&core);
}

#[test]
fn splitting_at_an_emptied_block_spans_the_paragraph_break() {
    // 한컴 12.30: 블록 글을 모두 지운 자리(또는 블록 전체를 고른 채)에서 Enter를 치면
    // 블록 시작은 앞 문단 끝에 남고 끝은 새 문단 맨 앞으로 가 블록이 문단 나누기를 품는다.
    let mut core = two_blocks();
    core.delete_text_native(0, 0, 2, 3).unwrap();
    core.split_paragraph_native(0, 0, 2, None).unwrap();
    let texts: Vec<String> = core.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect();
    assert_eq!(texts, ["가나", "바사", "아자차카타파하"]);
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 1, 0), block("여러", 1, 1, 2, 3)]
    );
    survives_save(&core);
    // 다시 합치면 빈 블록으로 돌아온다.
    core.merge_paragraph_native(0, 1).unwrap();
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 2), block("여러", 0, 3, 1, 3)]
    );
    survives_save(&core);

    // 문단 끝의 빈 블록도 같다.
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다").unwrap();
    let body = rhwp::document_core::queries::BookmarkPlace {
        section: 0,
        para: 0,
        cell_path: Vec::new(),
        header_footer: None,
    };
    let r = core
        .add_block_bookmark_native(&body, 1, 0, 3, "끝")
        .unwrap();
    assert!(r.contains(r#""ok":true"#), "{r}");
    core.delete_text_native(0, 0, 1, 2).unwrap();
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    assert_eq!(listed(&core), vec![block("끝", 0, 1, 1, 0)]);
    survives_save(&core);
}

#[test]
fn typing_after_a_spanning_block_that_ends_the_paragraph_stays_outside() {
    let mut core = two_blocks();
    let body = rhwp::document_core::queries::BookmarkPlace {
        section: 0,
        para: 0,
        cell_path: Vec::new(),
        header_footer: None,
    };
    let (place, ctrl) = {
        let items: Vec<serde_json::Value> =
            serde_json::from_str(&core.get_bookmarks_native().unwrap()).unwrap();
        let item = items.iter().find(|b| b["name"] == "여러").unwrap().clone();
        (
            item["place"].clone(),
            item["ctrlIdx"].as_u64().unwrap() as usize,
        )
    };
    let place: rhwp::document_core::queries::BookmarkPlace = serde_json::from_value(place).unwrap();
    core.delete_bookmark_at_native(&place, ctrl).unwrap();
    // 둘째 문단 끝("아자차카타파하" 끝)에서 끝나는 두 문단 블록을 넣고 그 끝에 친다.
    let r = core
        .add_block_bookmark_native(&body, 6, 1, 7, "끝까지")
        .unwrap();
    assert!(r.contains(r#""ok":true"#), "{r}");
    core.insert_text_native(0, 1, 7, "밖").unwrap();
    let last = &core.document().sections[0].paragraphs[1];
    assert_eq!(last.text, "아자차카타파하밖");
    // 원시 축에서도 끝 표지(8유닛)는 "하"와 "밖" 사이에 있다.
    assert_eq!(last.char_offsets[7] - last.char_offsets[6], 1 + 8);
    assert_eq!(
        listed(&core),
        vec![block("한", 0, 2, 0, 5), block("끝까지", 0, 6, 1, 7)]
    );
    survives_save(&core);
}

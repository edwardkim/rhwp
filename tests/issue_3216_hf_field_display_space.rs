//! Task #3216: 머리말/꼬리말 필드의 표시 치환이 모델 오프셋 공간을 깨면 안 된다.
//!
//! 필드(쪽 번호·전체 쪽수·파일 이름)는 모델에서 제어문자 **1자**지만 화면에는
//! `새 문서.hwp` 처럼 여러 자로 보인다. 치환 결과를 `run.text` 에 덮어쓰면 그 런의
//! 글자 수가 표시 길이가 되고, `char_start` 는 모델 기준 그대로라 히트테스트가
//! **두 공간이 섞인 오프셋**을 돌려준다. 그 값으로 편집하면
//!
//!   - 삽입은 문단 끝으로 밀려 사용자가 클릭한 자리와 다른 곳에 들어가고,
//!   - 삭제는 범위 밖이라 아무것도 지우지 않으면서 성공으로 보고된다(무언 무시).
//!
//! 저장소는 이 상황을 위한 기제를 이미 갖고 있다 — `convert_pua_display_text`
//! (`renderer/composer.rs`)가 세운 규약대로 `text` 는 모델 그대로 두고 `display_text`
//! 에만 표시값을 담는다. 이 테스트는 머리말 필드도 그 규약을 지키는지 고정한다.

use rhwp::wasm_api::HwpDocument;

/// 파일 이름이 여러 글자로 치환되는 문서 — 머리말에 파일명 필드 하나.
fn doc_with_file_name_field() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.set_file_name("보고서초안.hwp");
    doc.create_header_footer_native(0, true, 0)
        .expect("create header");
    // 필드 뒤에 본문 글자를 둬서, 필드 폭만큼 밀리는지 볼 수 있게 한다.
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "AB")
        .expect("header text");
    doc.insert_field_in_hf_native(0, true, 0, 0, 0, 3)
        .expect("file-name field at the head");
    doc
}

/// 머리말 문단의 모델 글자 수.
fn model_char_count(doc: &HwpDocument) -> usize {
    let info = doc
        .get_header_footer_para_info_native(0, true, 0, 0)
        .expect("para info");
    let value: serde_json::Value = serde_json::from_str(&info).expect("para info json");
    value["charCount"].as_u64().expect("charCount") as usize
}

/// 머리말 오른쪽 끝을 클릭했을 때 히트테스트가 주는 오프셋.
fn hit_offset_at_far_right(doc: &HwpDocument) -> usize {
    let hit = doc
        .hit_test_in_header_footer_native(0, true, 5000.0, 0.0)
        .expect("hit test");
    let value: serde_json::Value = serde_json::from_str(&hit).expect("hit json");
    assert_eq!(value["hit"], true, "머리말 영역을 맞혀야 한다: {hit}");
    value["charOffset"].as_u64().expect("charOffset") as usize
}

/// 파일명 필드가 있어도 캐럿 오프셋은 모델 길이를 넘지 않는다.
///
/// 치환이 `run.text` 를 덮어쓰면 이 오프셋이 표시 길이(`보고서초안.hwp` + `AB`)까지
/// 올라가 모델 길이 3 을 넘는다.
#[test]
fn caret_offset_stays_within_model_length() {
    let doc = doc_with_file_name_field();
    let model_len = model_char_count(&doc);
    assert_eq!(model_len, 3, "마커 1자 + AB 2자");

    let offset = hit_offset_at_far_right(&doc);
    assert!(
        offset <= model_len,
        "캐럿 오프셋 {offset} 이 모델 길이 {model_len} 를 넘었다 — 표시 공간 오프셋이다"
    );
}

/// 클릭한 자리에서 글자를 넣으면 그 자리에 들어간다.
///
/// 히트테스트가 준 오프셋을 그대로 뮤테이션에 넘기는, 실제 편집과 같은 경로다. 그 값이
/// 표시 공간이면 삽입이 문단 끝으로 밀려 `AB` 뒤에 붙는다.
#[test]
fn typing_after_the_field_lands_where_the_caret_is() {
    let mut doc = doc_with_file_name_field();
    let caret = hit_offset_at_far_right(&doc);
    // 오른쪽 끝을 클릭했으니 캐럿은 문단 끝(모델 3)이다.
    assert_eq!(caret, 3, "문단 끝 = 마커 1자 + AB 2자");

    doc.insert_text_in_header_footer_native(0, true, 0, 0, caret, "X")
        .expect("insert at caret");

    // 마커는 JSON 문자열 안에서 이스케이프돼 나오므로 파싱해서 본다.
    let content = doc
        .get_header_footer_native(0, true, 0)
        .expect("header content");
    let value: serde_json::Value = serde_json::from_str(&content).expect("header json");
    assert_eq!(
        value["text"].as_str().expect("text"),
        "\u{0017}ABX",
        "클릭한 문단 끝(마커·AB 뒤)에 들어가야 한다"
    );
}

/// 모델을 보존해도 화면에는 치환된 값이 그대로 나간다.
///
/// 렌더 트리는 모델 `text` 와 표시 `displayText` 를 함께 실어 보내고, 폭도 표시 기준으로
/// 잰다. 스튜디오는 `displayText` 가 있으면 그것으로 그린다.
#[test]
fn the_field_still_renders_its_substituted_value() {
    let doc = doc_with_file_name_field();
    let json = doc.get_page_layer_tree_native(0).expect("layer tree");

    assert!(
        json.contains("\"displayText\":\"보고서초안.hwp\""),
        "머리말 필드가 파일 이름으로 그려져야 한다"
    );
    // 마커 런은 모델 1자를 유지한다 — 이것이 오프셋 공간을 지키는 근거다.
    // 제어문자는 JSON 규격대로 이스케이프돼 나가야 파서가 받는다.
    assert!(
        json.contains("\"text\":\"\\u0017\""),
        "마커 런의 모델 텍스트는 이스케이프된 제어문자 1자여야 한다"
    );
    serde_json::from_str::<serde_json::Value>(&json).expect("레이어 트리 JSON 은 파싱돼야 한다");
}

/// 쪽 텍스트 추출은 필드 값을 내보낸다 — 제어문자가 새어 나가면 안 된다.
///
/// 렌더 트리에서 **문자열을 만들어 내보내는** 소비자다. 정본(`text`)이 아니라 표시
/// 텍스트를 써야 한다 — 그리기·측정과 같은 부류이고 오프셋 계산 쪽이 아니다.
/// 출시 CLI 의 쪽 텍스트·마크다운 내보내기가 이 경로를 쓴다.
#[test]
fn page_text_extraction_emits_the_field_value() {
    let doc = doc_with_file_name_field();
    let text = doc.extract_page_text_native(0).expect("page text");

    assert!(
        text.contains('\u{0017}') == false,
        "치환되지 않은 마커가 추출물에 새면 안 된다: {text:?}"
    );
    assert!(
        text.contains("보고서초안.hwp"),
        "머리말 필드 값이 추출물에 있어야 한다: {text:?}"
    );
}

/// 마커를 사이에 두고 쪼갠 조각이 원본 런의 표시 문자열을 물려받지 않는다.
///
/// `convert_pua_display_text` 가 런 전체에 대해 만든 값을 조각이 그대로 들고 가면,
/// 조각마다 남의 글자를 그려 주변 글자가 두 번 나온다. 형제 함수
/// `substitute_page_auto_numbers_in_composed` 도 `text` 를 고친 뒤 표시값을 무효화한다.
#[test]
fn split_pieces_do_not_inherit_the_whole_run_display_text() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.set_file_name("F.hwp");
    doc.create_header_footer_native(0, true, 0)
        .expect("create header");
    // 표시 확장이 있는 PUA 글자(U+F012B → "(인)")와 필드 마커를 한 런에 둔다.
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "󰄫Z")
        .expect("header text");

    let json = doc.get_page_layer_tree_native(0).expect("layer tree");
    let value: serde_json::Value = serde_json::from_str(&json).expect("layer tree json");

    let mut displays: Vec<String> = Vec::new();
    fn walk(node: &serde_json::Value, out: &mut Vec<String>) {
        if let Some(ops) = node.get("ops").and_then(|v| v.as_array()) {
            for op in ops {
                if op.get("type").and_then(|v| v.as_str()) == Some("textRun") {
                    if let Some(d) = op.get("displayText").and_then(|v| v.as_str()) {
                        out.push(d.to_string());
                    }
                }
            }
        }
        for child in node
            .get("children")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            walk(child, out);
        }
    }
    walk(value.get("root").unwrap_or(&value), &mut displays);

    // 어떤 조각도 치환되지 않은 마커를 표시값으로 들고 있으면 안 된다.
    for d in &displays {
        assert!(
            !d.contains(''),
            "조각이 원본 런의 표시 문자열을 물려받았다: {displays:?}"
        );
    }
    // 필드 조각은 파일 이름을, PUA 조각은 자기 확장만 갖는다.
    assert!(
        displays.iter().any(|d| d == "F.hwp"),
        "필드 조각의 표시값: {displays:?}"
    );
}

//! 관계형·고정형. 쪽마다 그리는 바탕쪽(쪽 소속)과 본문·바탕쪽 글의 위아래 순서는 관계형, HWP에 저장하는
//! 바탕쪽 목록 종류·속성 값과 구역 정의 비트·끝 목록 수는 고정형이다.
//! 주석 갱신: 2026-11-09T22:25:00+09:00 (새 파일).
//! 2026-11-09 측정 raw(96dpi, visual_sweep --silhouette-only, PDF 래스터 pypdfium2):
//! samples/issue8020/master_even_only.hwp 1·2·3쪽 Native 100.00% / fresh WASM 100.00%,
//! samples/issue8020/master_last_optional_overlap.hwp 1·2·3쪽 Native 100.00% / fresh WASM 100.00%;
//! 두 문서 모두 전 3쪽, 두 경로 전쪽 최저 100.00% / 100.00%.
//! 기준 PDF: 한컴오피스 한글 12.30.0(6544) macOS 파일 > 인쇄 > PDF로 저장(pdf/issue8020/).
//! 근거: mydocs/pr/assets/issue_8020/README.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::{DocumentCore, MASTER_PAGE_APPLY_BASE};
use rhwp::parser::tags::HWPTAG_LIST_HEADER;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

fn master_texts(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .section_def
        .master_pages
        .iter()
        .map(|master| {
            master
                .paragraphs
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

/// 바탕쪽 글이 그려진 첫 글자의 (x, y). SVG의 `<text x=".." y="..">글</text>`에서 찾는다.
fn glyph_at(svg: &str, ch: char) -> Option<(f64, f64)> {
    let needle = format!(">{}</text>", ch);
    let end = svg.find(&needle)?;
    let tag = &svg[svg[..end].rfind("<text")?..end];
    let attr = |name: &str| -> Option<f64> {
        let start = tag.find(&format!(" {}=\"", name))? + name.len() + 3;
        tag[start..].split('"').next()?.parse().ok()
    };
    Some((attr("x")?, attr("y")?))
}

/// 쪽에 그린 글자를 위에서부터 차례로(같은 줄은 왼쪽부터) 모은다.
fn drawn_lines(core: &DocumentCore, page: u32) -> Vec<String> {
    let svg = core.render_page_svg_native(page).unwrap();
    let mut glyphs: Vec<(f64, f64, char)> = Vec::new();
    for (end, _) in svg.match_indices("</text>") {
        let Some(ch) = svg[..end].chars().last() else {
            continue;
        };
        if let Some((x, y)) = glyph_at(&svg[..end + 7], ch) {
            glyphs.push((y, x, ch));
        }
    }
    glyphs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut lines: Vec<(f64, String)> = Vec::new();
    for (y, _, ch) in glyphs {
        match lines.last_mut() {
            Some((line_y, text)) if (*line_y - y).abs() < 1.0 => text.push(ch),
            _ => lines.push((y, ch.to_string())),
        }
    }
    lines.into_iter().map(|(_, text)| text).collect()
}

/// 한컴 12.30 인쇄(pdf/issue8020/)와 같은 바탕쪽을 같은 차례로 그린다.
#[test]
fn issue_samples_draw_the_masters_hancom_prints() {
    let open = |name: &str| {
        DocumentCore::from_bytes(&std::fs::read(format!("samples/issue8020/{name}.hwp")).unwrap())
            .unwrap()
    };
    // 짝수 쪽 바탕쪽 하나만 둔 구역: 2쪽에만 그린다.
    let even = open("master_even_only");
    assert_eq!(drawn_lines(&even, 0), ["본문"]);
    assert_eq!(drawn_lines(&even, 1), ["본문", "짝수바탕"]);
    assert_eq!(drawn_lines(&even, 2), ["본문"]);
    // 양 쪽, 마지막 쪽, 임의 2쪽(겹치게 하기), 임의 3쪽(맨 위로 보내기): 3쪽은 마지막 쪽 바탕쪽만.
    let ext = open("master_last_optional_overlap");
    assert_eq!(drawn_lines(&ext, 0), ["본문", "가양쪽"]);
    assert_eq!(drawn_lines(&ext, 1), ["본문", "가양쪽", "다둘째"]);
    assert_eq!(drawn_lines(&ext, 2), ["본문", "나끝쪽"]);
}

#[test]
fn master_page_text_is_edited_through_header_footer_api_and_survives_both_formats() {
    let mut core = blank();
    let created = core
        .create_master_page_native(0, "{\"applyTo\":0}")
        .unwrap();
    assert!(created.contains("\"applyTo\":16"));
    let target = MASTER_PAGE_APPLY_BASE;
    assert!(core
        .create_master_page_native(0, "{\"applyTo\":0}")
        .is_err());

    core.insert_text_in_header_footer_native(0, false, target, 0, 0, "바탕글")
        .unwrap();
    core.split_paragraph_in_header_footer_native(0, false, target, 0, 2, None)
        .unwrap();
    assert_eq!(master_texts(&core), ["바탕/글"]);
    core.merge_paragraph_in_header_footer_native(0, false, target, 1)
        .unwrap();
    core.delete_text_in_header_footer_native(0, false, target, 0, 2, 1)
        .unwrap();
    core.insert_text_in_header_footer_native(0, false, target, 0, 2, "쪽")
        .unwrap();
    assert_eq!(master_texts(&core), ["바탕쪽"]);
    // 머리말·꼬리말은 생기지 않는다.
    assert!(core
        .get_header_footer_native(0, true, 0)
        .unwrap()
        .contains("\"exists\":false"));

    // 한컴처럼 바탕쪽 글은 본문 첫 글자 자리(본문 영역 왼쪽 위)에 그린다.
    let page = &core.document().sections[0].section_def.page_def;
    let px = |hu: u32| f64::from(hu) * 96.0 / 7200.0;
    let body_x = px(page.margin_left + page.margin_gutter);
    let body_y = px(page.margin_top + page.margin_header);
    let (x, y) = glyph_at(&core.render_page_svg_native(0).unwrap(), '바').unwrap();
    assert!((x - body_x).abs() < 1.0, "x={x} body_x={body_x}");
    assert!(y > body_y && y < body_y + 20.0, "y={y} body_y={body_y}");

    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(master_texts(&reopened), ["바탕쪽"]);
        assert!(reopened
            .render_page_svg_native(0)
            .unwrap()
            .contains(">쪽</text>"));
    }
}

#[test]
fn editing_a_master_page_read_from_hwp_saves_the_edit() {
    let mut core = blank();
    core.create_master_page_native(0, "{\"applyTo\":0}")
        .unwrap();
    core.insert_text_in_header_footer_native(0, false, MASTER_PAGE_APPLY_BASE, 0, 0, "처음")
        .unwrap();
    let mut reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    // HWP에서 읽은 바탕쪽은 원본 레코드를 지닌다.
    assert!(!reopened.document().sections[0]
        .section_def
        .extra_child_records
        .is_empty());
    reopened
        .insert_text_in_header_footer_native(0, false, MASTER_PAGE_APPLY_BASE, 0, 2, "고침")
        .unwrap();
    reopened
        .create_master_page_native(0, "{\"applyTo\":2}")
        .unwrap();
    let saved = DocumentCore::from_bytes(&reopened.export_hwp_native().unwrap()).unwrap();
    assert_eq!(master_texts(&saved), ["처음고침", ""]);
    // 한컴처럼 홀수 쪽 바탕쪽이 양 쪽 바탕쪽보다 앞서 1쪽(홀수)은 빈 홀수 쪽 바탕쪽을 쓴다.
    assert!(!saved
        .render_page_svg_native(0)
        .unwrap()
        .contains(">고</text>"));
}

#[test]
fn base_and_tail_masters_of_a_hwp_with_a_section_tail_are_edited_and_saved() {
    // exam_math_8.hwp: 기본 바탕쪽은 구역 정의 자식, 임의 쪽 바탕쪽은 구역 끝 목록에 저장돼 있다.
    let bytes = std::fs::read("samples/exam_math_8.hwp").unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let masters = &core.document().sections[0].section_def.master_pages;
    assert!(!masters[0].is_extension && masters[1].is_extension);
    let page = masters[1].optional_page_number;
    assert!(page.is_some());
    assert!(core.document().sections[0].trailing_lists.is_some());
    core.insert_text_in_header_footer_native(0, false, MASTER_PAGE_APPLY_BASE, 0, 0, "바탕고침")
        .unwrap();
    core.insert_text_in_header_footer_native(0, false, MASTER_PAGE_APPLY_BASE + 1, 0, 0, "끝")
        .unwrap();
    let saved = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    let masters = &saved.document().sections[0].section_def.master_pages;
    assert!(masters[0].paragraphs[0].text.starts_with("바탕고침"));
    // 끝 목록의 임의 쪽 바탕쪽도 고친 글로 같은 쪽에 남는다.
    assert!(masters[1].is_extension);
    assert_eq!(masters[1].optional_page_number, page);
    assert!(masters[1].paragraphs[0].text.starts_with('끝'));
    assert_eq!(masters.len(), 2);
}

/// 본문만 고친 HWP를 저장해도 구역 정의의 끝 바탕쪽 목록 수가 원본 목록 수와 같다. 이 값이 0이면
/// 한컴 12.30은 구역 끝의 마지막 쪽·임의 쪽 바탕쪽을 읽지 않는다.
#[test]
fn body_edit_keeps_the_section_tail_master_count() {
    let tail_count = |core: &DocumentCore| -> Option<u16> {
        let extra = &core.document().sections[0].section_def.raw_ctrl_extra;
        extra.get(2..4).map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    let mut core =
        DocumentCore::from_bytes(&std::fs::read("samples/exam_math_8.hwp").unwrap()).unwrap();
    // exam_math_8.hwp: 구역 끝 목록에 임의 쪽 바탕쪽 하나.
    assert_eq!(tail_count(&core), Some(1));
    core.insert_text_native(0, 1, 0, "가").unwrap();
    let saved = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(tail_count(&saved), Some(1));
    let masters = &saved.document().sections[0].section_def.master_pages;
    assert_eq!(masters.iter().filter(|m| m.is_extension).count(), 1);
}

/// 세 쪽짜리 빈 문서(쪽마다 "본문" 한 문단).
fn three_pages() -> DocumentCore {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "본문").unwrap();
    for para in 0..2 {
        core.insert_page_break_native(0, para, 2).unwrap();
        core.insert_text_native(0, para + 1, 0, "본문").unwrap();
    }
    assert_eq!(core.page_count(), 3);
    core
}

fn set_master_text(core: &mut DocumentCore, slot: usize, text: &str) {
    core.insert_text_in_header_footer_native(
        0,
        false,
        MASTER_PAGE_APPLY_BASE + slot as u8,
        0,
        0,
        text,
    )
    .unwrap();
}

/// 쪽에 그린 바탕쪽 글(바탕쪽마다 서로 다른 첫 글자로 찾는다).
fn drawn(core: &DocumentCore, page: u32, marks: &[char]) -> String {
    let svg = core.render_page_svg_native(page).unwrap();
    marks
        .iter()
        .filter(|mark| svg.contains(&format!(">{}</text>", mark)))
        .collect()
}

/// 바탕쪽 LIST_HEADER의 종류 값(byte 18-19)과 속성(byte 22-25) — 저장 차례대로.
fn hwp_master_lists(bytes: &[u8]) -> Vec<(u16, u32)> {
    let core = DocumentCore::from_bytes(bytes).unwrap();
    let section = &core.document().sections[0];
    let mut raws: Vec<&[u8]> = section
        .section_def
        .extra_child_records
        .iter()
        .chain(
            section
                .trailing_lists
                .iter()
                .flat_map(|tail| tail.records.iter()),
        )
        .filter(|raw| raw.tag_id == HWPTAG_LIST_HEADER && raw.data.len() >= 26)
        .map(|raw| raw.data.as_slice())
        .collect();
    raws.retain(|data| data[8..16] != [0u8; 8]);
    raws.iter()
        .map(|data| {
            (
                u16::from_le_bytes([data[18], data[19]]),
                u32::from_le_bytes([data[22], data[23], data[24], data[25]]),
            )
        })
        .collect()
}

#[test]
fn last_and_optional_page_masters_follow_hancom_storage_and_drawing() {
    // 한컴 12.30 저장본 e-ext·b-ext·d-ext·c-front와 같은 순서로 만든다: 임의 2쪽(겹치게), 양 쪽,
    // 마지막 쪽, 임의 3쪽.
    let mut core = three_pages();
    core.create_master_page_native(0, r#"{"kind":"page","pageNumber":2,"overlap":true}"#)
        .unwrap();
    core.create_master_page_native(0, r#"{"applyTo":0}"#)
        .unwrap();
    core.create_master_page_native(0, r#"{"kind":"last"}"#)
        .unwrap();
    core.create_master_page_native(0, r#"{"kind":"page","pageNumber":3,"front":true}"#)
        .unwrap();
    // 같은 마지막 쪽·같은 임의 쪽은 둘 만들지 않는다.
    assert!(core
        .create_master_page_native(0, r#"{"kind":"last"}"#)
        .is_err());
    assert!(core
        .create_master_page_native(0, r#"{"kind":"page","pageNumber":2}"#)
        .is_err());
    assert!(core
        .create_master_page_native(0, r#"{"kind":"page","pageNumber":0}"#)
        .is_err());
    // 저장·이동 차례: 양 쪽, 마지막 쪽, 임의 쪽 번호 차례(한컴 12.30).
    let json = core.get_master_pages_native(0).unwrap();
    let order: Vec<_> = core.document().sections[0]
        .section_def
        .master_pages
        .iter()
        .map(|m| (m.is_extension, m.optional_page_number))
        .collect();
    assert_eq!(
        order,
        [
            (false, None),
            (true, None),
            (true, Some(2)),
            (true, Some(3))
        ],
        "{json}"
    );
    for (slot, text) in ["가양", "나끝", "다둘", "라셋"].iter().enumerate() {
        set_master_text(&mut core, slot, text);
    }
    let marks = ['가', '나', '다', '라'];
    // 한컴 12.30처럼 겹쳐 그린 쪽(2쪽)에서 고칠 바탕쪽은 임의 2쪽 바탕쪽이다.
    assert!(core
        .get_master_page_edit_target_native(1)
        .unwrap()
        .contains(&format!("\"applyTo\":{}", MASTER_PAGE_APPLY_BASE + 2)));
    // 1쪽 양 쪽, 2쪽 양 쪽 위에 임의 2쪽(겹치게 하기), 3쪽은 마지막 쪽 바탕쪽만(같은 쪽 임의 3쪽은 그리지 않음).
    let expect = |core: &DocumentCore| {
        assert_eq!(drawn(core, 0, &marks), "가");
        assert_eq!(drawn(core, 1, &marks), "가다");
        assert_eq!(drawn(core, 2, &marks), "나");
    };
    expect(&core);

    let hwp = core.export_hwp_native().unwrap();
    // 한컴 12.30 HWP: 기본 바탕쪽은 구역 정의 아래 종류 0, 확장 바탕쪽은 구역 끝 목록에 종류 3(마지막 쪽)·
    // n + 3(임의 쪽)과 속성 bit 0 겹치게 하기·bit 1 맨 위로 보내기.
    assert_eq!(hwp_master_lists(&hwp), [(0, 0), (3, 0), (5, 1), (6, 2)]);
    // 구역 정의의 대표Language 다음 UINT16은 구역 끝 바탕쪽 목록 수다(한컴 12.30 저장본·samples HWP).
    // 0이면 한컴이 구역 끝 바탕쪽을 읽지 않는다.
    let saved = DocumentCore::from_bytes(&hwp).unwrap();
    let extra = &saved.document().sections[0].section_def.raw_ctrl_extra;
    assert_eq!(extra.get(2..4), Some(&3u16.to_le_bytes()[..]));
    let hwpx = core.export_hwpx_native().unwrap();
    for bytes in [&hwp, &hwpx] {
        let reopened = DocumentCore::from_bytes(bytes).unwrap();
        expect(&reopened);
        let masters = &reopened.document().sections[0].section_def.master_pages;
        assert_eq!(
            masters
                .iter()
                .map(|m| (m.optional_page_number, m.replace_base, m.page_front))
                .collect::<Vec<_>>(),
            [
                (None, false, false),
                (None, true, false),
                (Some(2), false, false),
                (Some(3), true, true)
            ]
        );
    }
    // HWPX에서 연 바탕쪽을 HWP로 저장해도(앱 저장 경로) 같은 종류·속성이다.
    let via_hwpx = DocumentCore::from_bytes(&hwpx).unwrap();
    assert_eq!(
        hwp_master_lists(&via_hwpx.export_hwp_with_adapter_snapshot().unwrap()),
        [(0, 0), (3, 0), (5, 1), (6, 2)]
    );
}

#[test]
fn front_master_is_drawn_above_the_body() {
    let mut core = three_pages();
    core.create_master_page_native(0, r#"{"applyTo":0,"front":true}"#)
        .unwrap();
    set_master_text(&mut core, 0, "앞");
    let svg = core.render_page_svg_native(0).unwrap();
    let master = svg.find(">앞</text>").unwrap();
    let body = svg.find(">본</text>").unwrap();
    assert!(
        master > body,
        "맨 위로 보내기 바탕쪽은 본문 뒤에 그린다(위에 놓인다)"
    );
    let mut back = three_pages();
    back.create_master_page_native(0, r#"{"applyTo":0}"#)
        .unwrap();
    set_master_text(&mut back, 0, "뒤");
    let svg = back.render_page_svg_native(0).unwrap();
    assert!(svg.find(">뒤</text>").unwrap() < svg.find(">본</text>").unwrap());
}

#[test]
fn hwp_stores_master_kinds_in_section_flags_in_hancom_order() {
    use rhwp::model::header_footer::HeaderFooterApply::{Both, Even, Odd};
    // 한컴 12.30 저장본: 양 쪽·짝수 쪽·홀수 쪽 차례의 목록과 구역 정의 bit 29·30·31.
    for (kinds, flags) in [
        (vec![0u8], 0x2000_0000u32),
        (vec![1], 0x4000_0000),
        (vec![2], 0x8000_0000),
        (vec![2, 0], 0xa000_0000),
        (vec![2, 1], 0xc000_0000),
        (vec![2, 1, 0], 0xe000_0000),
    ] {
        let mut core = blank();
        for kind in &kinds {
            core.create_master_page_native(0, &format!("{{\"applyTo\":{kind}}}"))
                .unwrap();
        }
        for slot in 0..kinds.len() {
            core.insert_text_in_header_footer_native(
                0,
                false,
                MASTER_PAGE_APPLY_BASE + slot as u8,
                0,
                0,
                &format!(
                    "{:?}",
                    core.document().sections[0].section_def.master_pages[slot].apply_to
                ),
            )
            .unwrap();
        }
        let order: Vec<_> = core.document().sections[0]
            .section_def
            .master_pages
            .iter()
            .map(|m| m.apply_to)
            .collect();
        let expected: Vec<_> = [Both, Even, Odd]
            .into_iter()
            .filter(|kind| order.contains(kind))
            .collect();
        assert_eq!(order, expected, "만든 차례와 관계없이 한컴 차례로 둔다");
        let saved = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
        let section = &saved.document().sections[0];
        assert_eq!(section.section_def.flags & 0xe000_0000, flags);
        for master in &section.section_def.master_pages {
            assert_eq!(master.paragraphs[0].text, format!("{:?}", master.apply_to));
        }
        assert_eq!(section.section_def.master_pages.len(), kinds.len());
    }
}

#[test]
fn master_page_edit_view_draws_only_that_master_page_and_answers_caret_queries() {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "본문").unwrap();
    core.create_master_page_native(0, "{\"applyTo\":0}")
        .unwrap();
    let target = MASTER_PAGE_APPLY_BASE;
    core.insert_text_in_header_footer_native(0, false, target, 0, 0, "바탕")
        .unwrap();

    let page = core.render_page_svg_native(0).unwrap();
    assert!(page.contains(">본</text>") && page.contains(">바</text>"));
    let tree = core
        .build_header_footer_edit_preview_tree(0, 0, false, target)
        .unwrap();
    let mut text = String::new();
    fn collect(node: &rhwp::renderer::render_tree::RenderNode, out: &mut String) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        node.children.iter().for_each(|child| collect(child, out));
    }
    collect(&tree.root, &mut text);
    assert_eq!(text, "바탕");

    let rect = core
        .get_cursor_rect_in_header_footer_native(0, false, target, 0, 1, 0)
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&rect).unwrap();
    let (x, y) = (json["x"].as_f64().unwrap(), json["y"].as_f64().unwrap());
    let hit: serde_json::Value = serde_json::from_str(
        &core
            .hit_test_in_header_footer_target_native(0, 0, false, target, x + 0.5, y + 4.0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(hit["hit"], true);
    assert_eq!(hit["paraIndex"], 0);
    assert_eq!(hit["charOffset"], 1);
    let rects = core
        .get_selection_rects_in_header_footer_native(0, false, target, 0, 0, 0, 0, 2)
        .unwrap();
    assert!(rects.contains("\"width\""), "{rects}");

    let edit_target = core.get_master_page_edit_target_native(0).unwrap();
    assert!(edit_target.contains("\"sectionIndex\":0,\"applyTo\":16"));
    core.delete_master_page_native(0, target).unwrap();
    assert!(master_texts(&core).is_empty());
    assert!(!core
        .render_page_svg_native(0)
        .unwrap()
        .contains(">바</text>"));
    assert!(core
        .get_master_page_edit_target_native(0)
        .unwrap()
        .contains("\"found\":false"));
}

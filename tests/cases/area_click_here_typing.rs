//! 관계형 검사. 선택한 누름틀의 글자 소유, 인라인 표 뒤 실제 삽입 위치, 머리말·꼬리말의 글줄과 표의 배치를 검증한다.
//! 2026-11-24 15:55 KST 갱신. 직전 파일 변경은 2026-11-23 16:45:02 KST의 fc4e2cf07a1439760d2cfb67c8230cb3140e2a78이다.
//! 2026-11-24 한컴 2024 Print, 96 DPI의 보정 전 2px 실루엣으로 측정했다. 줄 너비 표가 있는 머리말·꼬리말,
//! 누름틀을 뺀 머리말, 한컴에서 재저장한 머리말 및 좁은 표로 바꾼 머리말·꼬리말은 각각 1쪽이다.
//! 여섯 원본 각각의 전쪽 최저는 Native와 fresh WASM 모두 100.00%다.
//! 쪽별 시각 근거는 `mydocs/pr/assets/semanticist21-9337-9354-9377/focus-pages.tsv`에 있다.
//!
//! 본문·표 셀 입력은 활성 누름틀과 누름틀 슬롯 규칙을 따르지만, 각주·머리말 입력은 일반 삽입을 써서 같은 자리의
//! 빈 누름틀이 모두 친 글을 품고(겹친 필드로 저장된다) 활성 누름틀도 보지 않았다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

fn blank(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core
}

fn json(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap()
}

/// 각주·머리말 첫 문단의 누름틀 범위: (컨트롤 번호, 시작, 끝).
fn ranges(core: &DocumentCore, para: usize, control: usize) -> Vec<(usize, usize, usize)> {
    let paragraphs = match &core.document().sections[0].paragraphs[para].controls[control] {
        Control::Footnote(note) => &note.paragraphs,
        Control::Header(header) => &header.paragraphs,
        _ => panic!("각주·머리말이 아니다"),
    };
    paragraphs[0]
        .field_ranges
        .iter()
        .map(|range| (range.control_idx, range.start_char_idx, range.end_char_idx))
        .collect()
}

/// 저장한 HWP·HWPX를 다시 열어도 같은 범위인지 본다.
fn assert_saved(core: &DocumentCore, para: usize, control: usize) {
    let expected = ranges(core, para, control);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(ranges(&reopened, para, control), expected);
    }
}

#[test]
fn header_typing_keeps_the_active_field_and_fills_one_empty_field() {
    let mut core = blank("본문");
    let header = json(&core.create_header_footer_native(0, true, 0).unwrap());
    let (hp, hc) = (
        header["paraIndex"].as_u64().unwrap() as usize,
        header["controlIndex"].as_u64().unwrap() as usize,
    );
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리 끝")
        .unwrap();
    for name in ["다", "라"] {
        core.insert_click_here_field_at_by_path(0, hp, &[(hc, 0, 0)], 3, "안내", "", name, true)
            .unwrap();
    }
    let [(c, _, _), (d, _, _)] = ranges(&core, hp, hc)[..] else {
        panic!("누름틀 둘이 있어야 한다");
    };
    // 같은 자리의 빈 누름틀 둘 가운데 첫 것만 친 글을 품는다.
    core.insert_text_in_header_footer_native(0, true, 0, 0, 3, "값")
        .unwrap();
    assert_eq!(ranges(&core, hp, hc), vec![(c, 3, 4), (d, 4, 4)]);

    // 활성으로 둔 다의 끝에서 친 글은 그 자리의 빈 라가 아니라 다에 든다.
    assert!(core.set_active_field_by_path(0, hp, &[(hc, 0, 0)], 3));
    core.insert_text_in_header_footer_native(0, true, 0, 0, 4, "더")
        .unwrap();
    assert_eq!(ranges(&core, hp, hc), vec![(c, 3, 5), (d, 5, 5)]);
    assert_saved(&core, hp, hc);
}

#[test]
fn typing_after_a_trailing_inline_table_returns_the_actual_text_end() {
    for (header, source) in [
        (
            true,
            include_bytes!("../fixtures/issue9354/trailing-header.hwpx").as_slice(),
        ),
        (
            false,
            include_bytes!("../fixtures/issue9354/trailing-footer.hwpx").as_slice(),
        ),
    ] {
        let mut core = DocumentCore::from_bytes(source).expect("누름틀 둘과 끝 표가 있는 원본");
        let read = |doc: &DocumentCore| {
            json(
                &doc.get_header_footer_para_info_native(0, header, 0, 0)
                    .unwrap(),
            )["text"]
                .as_str()
                .unwrap()
                .to_owned()
        };
        let original = read(&core);
        let text_end = original.chars().count();
        let caret_after_table = text_end + 1;
        for typed in ["한", "😀"] {
            let inserted = json(
                &core
                    .insert_text_in_header_footer_native(0, header, 0, 0, caret_after_table, typed)
                    .unwrap(),
            );
            assert_eq!(read(&core), format!("{original}{typed}"));
            let end = inserted["charOffset"].as_u64().unwrap() as usize;
            assert_eq!(end, text_end + typed.chars().count());
            core.delete_text_in_header_footer_native(
                0,
                header,
                0,
                0,
                end - typed.chars().count(),
                typed.chars().count(),
            )
            .unwrap();
            assert_eq!(read(&core), original);
        }
    }
}

#[test]
fn header_footer_text_and_table_share_the_printed_line_layout() {
    use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

    fn collect<'a>(
        node: &'a RenderNode,
        text: &mut Vec<(&'a str, BoundingBox)>,
        tables: &mut Vec<BoundingBox>,
    ) {
        match &node.node_type {
            RenderNodeType::Table(_) => {
                tables.push(node.bbox);
                return;
            }
            RenderNodeType::TextRun(run) if !run.text.is_empty() => {
                text.push((&run.text, node.bbox));
            }
            _ => {}
        }
        for child in &node.children {
            collect(child, text, tables);
        }
    }

    for (header, wraps, bytes) in [
        (
            true,
            true,
            include_bytes!("../fixtures/issue9354/trailing-header.hwpx").as_slice(),
        ),
        (
            false,
            true,
            include_bytes!("../fixtures/issue9354/trailing-footer.hwpx").as_slice(),
        ),
        (
            true,
            true,
            include_bytes!("../fixtures/issue9354/wide-no-fields.hwpx").as_slice(),
        ),
        (
            true,
            true,
            include_bytes!("../fixtures/issue9354/wide-hancom-saved.hwpx").as_slice(),
        ),
        (
            true,
            false,
            include_bytes!("../fixtures/issue9354/short-header.hwpx").as_slice(),
        ),
        (
            false,
            false,
            include_bytes!("../fixtures/issue9354/short-footer.hwpx").as_slice(),
        ),
    ] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        let page = core.build_page_render_tree(0).unwrap();
        let area = page
            .root
            .children
            .iter()
            .find(|n| {
                if header {
                    matches!(n.node_type, RenderNodeType::Header)
                } else {
                    matches!(n.node_type, RenderNodeType::Footer)
                }
            })
            .expect("표가 있는 머리말 또는 꼬리말");
        let mut text = Vec::new();
        let mut tables = Vec::new();
        collect(area, &mut text, &mut tables);
        let [table] = tables.as_slice() else {
            panic!("표 한 개")
        };
        assert_eq!(
            text.iter().map(|(value, _)| *value).collect::<String>(),
            "앞뒤"
        );
        let first = text.first().unwrap().1;
        let last = text.last().unwrap().1;
        assert!((first.y - last.y).abs() < 1.0, "두 글자는 같은 줄 소속");
        let last_right = last.x + last.width;
        if wraps {
            assert!(
                first.y + first.height <= table.y + 1.0,
                "표는 두 글자 다음 줄에 놓인다"
            );
            assert!(
                (last_right - (area.bbox.x + area.bbox.width)).abs() < first.height,
                "양쪽 정렬한 앞줄의 마지막 글자는 오른쪽 끝에 놓인다"
            );
        } else {
            assert!(
                last_right <= table.x + 1.0,
                "좁은 표는 글자 오른쪽에 놓인다"
            );
            assert!(
                first.y < table.y + table.height && table.y < first.y + first.height,
                "글자와 표가 같은 줄을 차지한다"
            );
        }
    }
}

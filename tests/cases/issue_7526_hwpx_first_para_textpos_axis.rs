//! [#7526] HWP/HWP3 원본에서 만든 HWPX는 저장 전의 줄 나눔을 보존한다.
//! 재저장과 다시 연 문단의 편집도 같은 왕복 계약을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const WORD_BREAK_TEXT: &str = "짧은 문단에서 한글어절 나누기를 확인합니다";

fn collect_lines(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        for child in &node.children {
            if let RenderNodeType::TextRun(run) = &child.node_type {
                text.push_str(&run.text);
            }
        }
        out.push(text);
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

/// 첫 쪽에 그려진 줄마다의 글자.
fn page_lines(core: &DocumentCore) -> Vec<String> {
    let mut lines = Vec::new();
    collect_lines(
        &core.build_page_render_tree(0).expect("첫 쪽 렌더").root,
        &mut lines,
    );
    lines
}

fn blank_with_first_paragraph(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서");
    core.insert_text_native(0, 0, 0, text).expect("글자 입력");
    core
}

fn reopen_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장")).expect("HWPX 열기")
}

#[test]
fn word_break_paragraph_keeps_its_line_starts_after_hwpx_round_trip() {
    let mut core = blank_with_first_paragraph(WORD_BREAK_TEXT);
    core.apply_para_format_native(
        0,
        0,
        r#"{"alignment":"left","marginRight":58500,"koreanBreakUnit":0}"#,
    )
    .expect("문단 모양");
    let edited = page_lines(&core);
    assert_eq!(
        edited.first().map(|line| line.trim_end()),
        Some("짧은 문단에서 한글어절"),
        "전제: 어절 나눔으로 '나누기를' 앞에서 끊겨야 한다, got {edited:?}"
    );

    let reopened = page_lines(&reopen_hwpx(&core));
    assert_eq!(
        reopened, edited,
        "HWPX 로 저장해 다시 연 구역 첫 문단의 줄이 바뀌었다 — 8유닛 일찍 끊기면 '짧은 문단' 뒤에서 끊긴다"
    );
}

#[test]
fn long_hangul_paragraph_keeps_its_line_starts_after_hwpx_round_trip() {
    let core = blank_with_first_paragraph(&"가나다라마바사아자차카타파하".repeat(8));
    let edited = page_lines(&core);
    assert!(
        edited.len() >= 3,
        "전제: 112자 문단은 세 줄 이상이다, got {edited:?}"
    );

    let reopened = reopen_hwpx(&core);
    assert_eq!(
        page_lines(&reopened),
        edited,
        "HWPX 로 저장해 다시 연 112자 문단의 줄 시작이 바뀌었다"
    );
    assert_eq!(
        page_lines(&reopen_hwpx(&reopened)),
        edited,
        "다시 연 HWPX 를 한 번 더 저장하면 줄이 움직였다 — 재수출 고정점이 깨졌다"
    );
    let as_hwp = DocumentCore::from_bytes(&reopened.export_hwp_native().expect("HWP 저장"))
        .expect("HWP 열기");
    assert_eq!(
        page_lines(&as_hwp),
        edited,
        "다시 연 HWPX 를 HWP 로 저장하면 줄이 바뀌었다 — HWP5 축으로 올리는 폭이 모자라다"
    );
    assert_eq!(
        page_lines(&reopen_hwpx(&as_hwp)),
        edited,
        "HWPX→HWP→HWPX 변환 계보에서도 줄 나눔을 보존한다"
    );
}

/// 마커가 있는 HWPX 를 열어 구역 첫 문단을 고치면 그 문단은 다시 조판한 HWP5 축이다.
/// 저장기가 날값으로 내면 다시 열 때 줄이 늦게 끊긴다.
#[test]
fn edited_first_paragraph_of_an_rhwp_hwpx_keeps_its_line_starts() {
    let mut reopened = reopen_hwpx(&blank_with_first_paragraph("가나다라마바사아자차카타파하"));
    reopened
        .insert_text_native(0, 0, 0, &"가나다라마바사아자차카타파하".repeat(7))
        .expect("다시 연 문서에 글자 입력");
    let edited = page_lines(&reopened);
    assert!(
        edited.len() >= 3,
        "전제: 편집한 문단은 세 줄 이상이다, got {edited:?}"
    );

    assert_eq!(
        page_lines(&reopen_hwpx(&reopened)),
        edited,
        "rhwp HWPX 에서 고친 구역 첫 문단을 다시 저장해 열면 줄 시작이 바뀌었다"
    );
}

/// HWP3 원본은 `rhwp-hwp3-origin` 마커만 싣는다. 저장기 계약은 같다.
#[test]
fn hwp3_origin_first_paragraph_keeps_its_line_starts_after_hwpx_round_trip() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-pagedef-1915.hwp");
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("HWP3 샘플")).expect("HWP3 열기");
    core.insert_text_native(0, 0, 0, &"가나다라마바사아자차카타파하".repeat(8))
        .expect("글자 입력");
    let edited = page_lines(&core);

    assert_eq!(
        page_lines(&reopen_hwpx(&core)),
        edited,
        "HWP3 원본을 HWPX 로 저장해 다시 연 구역 첫 문단의 줄 시작이 바뀌었다"
    );
}

/// 독립 한컴2020 저장본과 같은 문단 슬롯 축: 첫 run secPr/colPr 뒤 43·86자.
/// 자체 왕복 성공이 잘못된 XML textpos를 감추지 않도록 실제 ZIP을 검사한다.
#[test]
fn published_long_paragraph_uses_the_independent_hancom_textpos_axis() {
    use std::io::Read;
    let core = blank_with_first_paragraph(&"가나다라마바사아자차카타파하".repeat(8));
    let bytes = core.export_hwpx_native().unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let starts: Vec<u32> = xml
        .split("<hp:lineseg ")
        .skip(1)
        .map(|tag| {
            let value = tag
                .split("textpos=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            value.parse().unwrap()
        })
        .collect();
    // MCP의 한컴 저장 결과 0,59,102와 Print의 첫 줄 43자에서 독립적으로 확인했다.
    assert_eq!(
        starts,
        vec![0, 59, 102],
        "XML 문단의 실제 제어 슬롯을 보존해야 한다"
    );
}

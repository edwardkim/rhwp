//! Issue #3930/#3820 — HWPX 저장 뒤 표 분할·바탕쪽과 PDF page owner를 보존한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::header_footer::{HeaderFooterApply, MasterPage};
use rhwp::model::shape::ShapeObject;
use rhwp::model::style::BorderLineType;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const FIXTURE: &str = "samples/2025 행정업무운영 편람(최종).hwpx";
const PAGE_30: u32 = 29;
const PAGE_144: u32 = 143;
const PAGE_145: u32 = 144;
const ATTACHMENT_GUIDANCE: &str = "기안문에 작성한 붙임 문서를 첨부";

fn page_tree(document: &HwpDocument, page: u32) -> String {
    document
        .get_page_render_tree(page)
        .unwrap_or_else(|error| panic!("p{} render tree: {error:?}", page + 1))
}

/// PDF p144 안에서 끝나는 붙임 표가 `page tree`에만 남고 물리적으로 쪽 밖으로
/// 잘리는 퇴행을 막는다. 새 DocumentCore로 독립 렌더해 앞선 tree 조회의 카운터를
/// 섞지 않는다 (#3820 Stage 65).
fn page_overflow_cell_lines(bytes: &[u8], page: u32) -> u32 {
    let document = DocumentCore::from_bytes(bytes).expect("overflow fixture parse");
    let _ = document.take_overflow_cell_lines();
    document
        .render_page_svg_native(page)
        .unwrap_or_else(|error| panic!("p{} render: {error:?}", page + 1));
    document.take_overflow_cell_lines()
}

fn collect_stamp_placeholder_tables(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table)
            if table.row_count == 1
                && table.col_count == 1
                && (node.bbox.width - 56.7).abs() <= 0.2
                && (node.bbox.height - 56.7).abs() <= 0.2
    ) {
        out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    for child in &node.children {
        collect_stamp_placeholder_tables(child, out);
    }
}

fn master_page_text(master_page: &MasterPage) -> String {
    let mut text = String::new();
    for paragraph in &master_page.paragraphs {
        text.push_str(&paragraph.text);
        for control in &paragraph.controls {
            let Control::Shape(shape) = control else {
                continue;
            };
            let Some(text_box) = shape
                .drawing()
                .and_then(|drawing| drawing.text_box.as_ref())
            else {
                continue;
            };
            for text_box_paragraph in &text_box.paragraphs {
                text.push_str(&text_box_paragraph.text);
            }
        }
    }
    text
}

#[test]
fn issue_3930_preserves_page_count_and_inherited_even_master_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    // CLI가 사용하는 native HwpDocument 래퍼까지 동일하게 통과해야 한다.
    let mut source = HwpDocument::from_bytes(&bytes).expect("HWPX fixture parse");

    // 한컴 2024 PDF p144에는 "붙임 파일에 직인 날인 방법" 표의 안내·예시가
    // 모두 있어야 한다. raw `treatAsChar=1`만 보고 block table을 조기 분할하면
    // p145로 이월되어 이후 page owner가 연쇄적으로 한 쪽씩 밀린다 (#3820).
    assert_eq!(
        source.page_count(),
        386,
        "p144 조기 이월 보정 뒤 원본 편람 쪽수"
    );
    let source_p30_tree = page_tree(&source, PAGE_30);
    let source_p144_tree = page_tree(&source, PAGE_144);
    let source_p145_tree = page_tree(&source, PAGE_145);
    assert!(
        source_p30_tree.contains("\"text\":\"2025 \"")
            && source_p30_tree.contains("\"text\":\"행정업무운영 편람\""),
        "원본 p30 바탕쪽은 책 제목이어야 한다"
    );
    assert!(
        !source_p30_tree.contains("제2장. 공문서 관리"),
        "원본 p30 바탕쪽은 장 제목으로 바뀌면 안 된다"
    );
    assert!(
        source_p144_tree.contains(ATTACHMENT_GUIDANCE),
        "한컴 PDF p144와 같이 붙임 안내 블록은 원본 p144에 있어야 한다"
    );
    assert!(
        !source_p145_tree.contains(ATTACHMENT_GUIDANCE),
        "원본 p145는 앞 표의 붙임 안내 블록을 다시 갖지 않아야 한다"
    );
    assert_eq!(
        page_overflow_cell_lines(&bytes, PAGE_144),
        0,
        "PDF p144에 완결된 붙임 표의 하위 안내·caption은 쪽 밖으로 clip되면 안 된다"
    );
    let source_border_fill = &source.document().doc_info.border_fills[67];
    assert_eq!(
        source_border_fill.borders[0].line_type,
        BorderLineType::Dot,
        "HWPX DASH 테두리는 Hancom HWP5 code 3 점선으로 읽어야 한다"
    );
    // CLI/MCP 저장 경로도 배포용 해제 단계를 먼저 거치므로 같은 순서로 검증한다.
    source
        .convert_to_editable_native()
        .expect("편집 가능 문서 정규화");
    let saved = source.export_hwp_with_adapter().expect("HWP 저장");

    // HWPX에는 HWP5 SECTION_DEF의 raw tail이 없지만, HWP 2020은 바탕쪽이 있는
    // 구역에 19 byte tail(CTRL_HEADER 전체 47 byte)을 쓴다. 이 값이 10 byte
    // 기본값으로 남으면 HWP 2020이 LIST_HEADER 바탕쪽을 무시할 수 있다.
    let section_index = 10;
    let section = &source.document().sections[section_index];
    assert_eq!(
        section.section_def.raw_ctrl_extra.len(),
        19,
        "구역 {section_index} root SectionDef HWP5 바탕쪽 tail"
    );
    let inline_section_def = section.paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::SectionDef(section_def) => Some(section_def.as_ref()),
            _ => None,
        })
        .expect("첫 문단 SectionDef");
    assert_eq!(
        inline_section_def.raw_ctrl_extra.len(),
        19,
        "구역 {section_index} inline SectionDef HWP5 바탕쪽 tail"
    );
    let reloaded = HwpDocument::from_bytes(&saved).expect("저장 HWP 재로드");

    assert_eq!(
        reloaded.document().sections[10]
            .section_def
            .raw_ctrl_extra
            .len(),
        19,
        "직렬화된 구역 10 SectionDef도 HWP 2020 바탕쪽 tail을 보존해야 한다"
    );

    assert_eq!(
        reloaded.page_count(),
        source.page_count(),
        "저장 HWP도 p144 table owner를 HWPX 원본과 같게 보존해야 한다"
    );
    for (page, source_tree) in [
        (PAGE_30, source_p30_tree),
        (PAGE_144, source_p144_tree),
        (PAGE_145, source_p145_tree),
    ] {
        assert_eq!(
            page_tree(&reloaded, page),
            source_tree,
            "저장 HWP p{} 조판 tree는 원본 HWPX와 같아야 한다",
            page + 1
        );
    }

    let section = &reloaded.document().sections[2].section_def;
    let base_master_pages: Vec<&MasterPage> = section
        .master_pages
        .iter()
        .filter(|master_page| !master_page.is_extension)
        .collect();
    assert_eq!(base_master_pages.len(), 1, "HWP 2020 단일 Odd 저장 슬롯");
    assert_eq!(base_master_pages[0].apply_to, HeaderFooterApply::Odd);
    // 한컴 2020은 아래 SECTION_DEF 0x80000000 플래그로 이전 구역의 짝수 바탕쪽을
    // 상속한다. HWP5 parser도 이 단일 Odd 계약을 그대로 복원해야 한다.
    assert!(
        master_page_text(base_master_pages[0]).contains("제2장. 공문서 관리"),
        "홀수 쪽은 현재 구역 장 제목 바탕쪽을 사용해야 한다"
    );
    assert_eq!(
        section.flags & 0xe000_0000,
        0x8000_0000,
        "단일 Odd 슬롯은 한컴 2020의 이전 짝수 쪽 상속 플래그여야 한다"
    );
    assert_eq!(
        reloaded.document().doc_info.border_fills[67].borders[0].line_type,
        BorderLineType::Dot,
        "저장 HWP도 날인 상자의 점선 BORDER_FILL을 유지해야 한다"
    );

    let first_picture = reloaded.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Picture(picture) => Some(picture.as_ref()),
            _ => None,
        })
        .expect("첫 그림");
    let grouped_picture = reloaded.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Group(group) => group.children.iter().find_map(|child| match child {
                    ShapeObject::Picture(picture) => Some(picture.as_ref()),
                    _ => None,
                }),
                _ => None,
            },
            _ => None,
        })
        .expect("묶음 내부 그림");
    for picture in [first_picture, grouped_picture] {
        assert_eq!(
            picture.raw_picture_extra.len(),
            18,
            "HWPX 그림의 HWP5 SC_PICTURE extra 길이"
        );
        assert_eq!(
            &picture.raw_picture_extra[9..17],
            &[0; 8],
            "한컴 HWPX 저장본처럼 SC_PICTURE original image size는 0으로 쓴다"
        );
    }
    assert_eq!(grouped_picture.image_attr.brightness, 0);
    assert_eq!(grouped_picture.image_attr.contrast, 8);
}

/// PDF p144의 자동날인 안내는 같은 빈 host paragraph의 `BehindText` 1×1 table 세 개를
/// `horzOffset=4868,13553,22830HU`로 한 줄에 놓는다. nested non-TAC의 generic flow가
/// 각 table 높이만큼 cursor를 전진하면 세 점선 상자가 세로로 쌓여, page owner가 맞아도
/// 눈에 보이는 fidelity가 깨진다 (#3820 Stage 66).
#[test]
fn issue_3820_hwpx_behind_text_stamp_placeholders_keep_common_y_and_offsets() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX fixture parse");
    let page = core
        .build_page_render_tree(PAGE_144)
        .expect("render PDF p144");
    let mut stamps = Vec::new();
    collect_stamp_placeholder_tables(&page.root, &mut stamps);
    stamps.sort_by(|left, right| left.0.total_cmp(&right.0));

    assert_eq!(
        stamps.len(),
        3,
        "p144 automatic-stamp guide must retain three 1×1 placeholder tables: {stamps:?}"
    );
    let expected_x = [182.0, 297.8, 421.5];
    for ((x, y, width, height), expected_x) in stamps.iter().zip(expected_x) {
        assert!(
            (*x - expected_x).abs() <= 0.3,
            "p144 HWPX horzOffset anchor mismatch: x={x:.1}, expected={expected_x:.1}, stamps={stamps:?}"
        );
        assert!(
            (*y - stamps[0].1).abs() <= 0.3,
            "p144 BehindText placeholders must share one paragraph y: stamps={stamps:?}"
        );
        assert!(
            (*width - 56.7).abs() <= 0.2 && (*height - 56.7).abs() <= 0.2,
            "p144 placeholder physical size must preserve the PDF's 4251HU square: {stamps:?}"
        );
    }
}

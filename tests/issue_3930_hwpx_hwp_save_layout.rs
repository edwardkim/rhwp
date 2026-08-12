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
const HWP_FIXTURE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
const PAGE_30: u32 = 29;
const PAGE_144: u32 = 143;
const PAGE_145: u32 = 144;
const PAGE_314: u32 = 313;
const PAGE_365: u32 = 364;
const PAGE_366: u32 = 365;
const PAGE_367: u32 = 366;
const PAGE_283: u32 = 282;
const PAGE_284: u32 = 283;
const PAGE_285: u32 = 284;
const PAGE_286: u32 = 285;
const PAGE_287: u32 = 286;
const PAGE_290: u32 = 289;
const PAGE_291: u32 = 290;
const PAGE_294: u32 = 293;
const PAGE_295: u32 = 294;
const PAGE_296: u32 = 295;
const Q5_RESPONSE_FIRST_LINE: &str = "문서는 결재권자의 결재가 완료된 시점에";
const Q9_TITLE: &str = "보조기관, 보좌기관, 합의제행정기관의 의미";
const Q10_TITLE: &str = "공문서 작성시 연·월·일의 정확한 표기방법";
const Q16_TITLE: &str = "문서의 결재과정에서 협조자는 문서 수정이나 반려가 가능한지요?";
const Q27_TITLE: &str = "소방서장이 지시한 업무에 대해서 소방파출소장이 문서를";
const Q29_TITLE: &str = "구청 내의 중요사항을 계획하고 각 부서로 시행을 할 경우에도";
const Q30_TITLE: &str = "직속기관, 사업소, 출장소, 구청";
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
    assert_eq!(
        source.page_count(),
        383,
        "HWPX Q&A PageHide/목차 tail 보정 뒤 Hancom PDF 쪽수"
    );

    // 한컴 2024 PDF p144에는 "붙임 파일에 직인 날인 방법" 표의 안내·예시가
    // 모두 있어야 한다. raw `treatAsChar=1`만 보고 block table을 조기 분할하면
    // p145로 이월되어 이후 page owner가 연쇄적으로 한 쪽씩 밀린다 (#3820).
    let source_p30_tree = page_tree(&source, PAGE_30);
    let source_p144_tree = page_tree(&source, PAGE_144);
    let source_p145_tree = page_tree(&source, PAGE_145);
    let source_p314_tree = page_tree(&source, PAGE_314);
    let source_p365_tree = page_tree(&source, PAGE_365);
    let source_p366_tree = page_tree(&source, PAGE_366);
    let source_p367_tree = page_tree(&source, PAGE_367);
    let source_p283_tree = page_tree(&source, PAGE_283);
    let source_p284_tree = page_tree(&source, PAGE_284);
    let source_p285_tree = page_tree(&source, PAGE_285);
    let source_p286_tree = page_tree(&source, PAGE_286);
    let source_p287_tree = page_tree(&source, PAGE_287);
    let source_p290_tree = page_tree(&source, PAGE_290);
    let source_p291_tree = page_tree(&source, PAGE_291);
    let source_p294_tree = page_tree(&source, PAGE_294);
    let source_p295_tree = page_tree(&source, PAGE_295);
    let source_p296_tree = page_tree(&source, PAGE_296);
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
    assert!(
        source_p314_tree.contains("공문서 관리"),
        "HWPX 병렬 규정 표 뒤의 PDF p314는 제2장 공문서 관리로 시작해야 한다"
    );
    assert!(
        !source_p314_tree.contains("정책실명제"),
        "HWPX 병렬 규정 표 제3조 r5 tail은 PDF p314로 넘어가면 안 된다"
    );
    let source_p365_has_education = source_p365_tree.contains("행정업무 운영에 관한 교육");
    let source_p365_has_nis_special_case =
        source_p365_tree.contains("국가정보원의 업무운영에 대한 특례");
    assert!(
        source_p365_has_education && !source_p365_has_nis_special_case,
        "HWPX 병렬 규정 표의 PDF p365는 제65~68조 owner여야 한다: \
         교육={source_p365_has_education}, 국가정보원 특례={source_p365_has_nis_special_case}"
    );
    assert!(
        source_p366_tree.contains("국가정보원의 업무운영에 대한 특례")
            && source_p366_tree.contains("권한의 위임")
            && source_p366_tree.contains("이 규칙에 따른 서식의 전자적 관리")
            && source_p366_tree.contains("세부 사항")
            && !source_p366_tree.contains("부  칙"),
        "HWPX 병렬 규정 표의 PDF p366은 제69·70조와 시행규칙 제47·48조를 소유해야 한다"
    );
    assert!(
        source_p367_tree.contains("부  칙")
            && !source_p367_tree.contains("이 규칙에 따른 서식의 전자적 관리"),
        "HWPX 병렬 규정 표의 PDF p367은 시행규칙 부칙부터 시작해야 한다"
    );
    assert!(
        source_p283_tree.contains(Q5_RESPONSE_FIRST_LINE),
        "HWPX Q5의 saved-frame response 첫 줄은 PDF/native HWP와 같이 p283에 있어야 한다"
    );
    assert!(
        !source_p284_tree.contains(Q5_RESPONSE_FIRST_LINE),
        "HWPX Q5의 saved-frame response 첫 줄은 p284로 밀리면 안 된다"
    );
    assert!(
        source_p285_tree.contains("홈페이지상의 질의에 대하여"),
        "HWPX Q8 표제는 PDF physical p285와 같이 Q7 tail 뒤 같은 쪽에서 시작해야 한다"
    );
    assert!(
        source_p286_tree.contains(Q9_TITLE),
        "HWPX Q9 표제는 PDF/native HWP와 같이 p286에서 시작해야 한다"
    );
    assert!(
        source_p287_tree.contains(Q10_TITLE),
        "HWPX Q10 표제는 PDF/native HWP와 같이 p287에서 시작해야 한다"
    );
    assert!(
        source_p290_tree.contains(Q16_TITLE),
        "HWPX Q16 표와 trailing blank-bottom row는 PDF/native HWP와 같이 p290에서 끝나야 한다"
    );
    assert!(
        !source_p291_tree.contains(Q16_TITLE),
        "HWPX Q16 표는 p291로 분할되어 반복되면 안 된다"
    );
    assert!(
        !source_p294_tree.contains(Q27_TITLE),
        "HWPX Q26의 3+3줄 응답 tail은 PDF/native HWP와 같이 p294에서 끝나야 한다"
    );
    assert!(
        source_p295_tree.contains(Q27_TITLE),
        "HWPX Q27 표제는 PDF/native HWP와 같이 p295에서 시작해야 한다"
    );
    assert!(
        source_p295_tree.contains(Q29_TITLE),
        "HWPX Q29의 두 줄 response는 PDF/native HWP와 같이 p295에서 끝나야 한다"
    );
    assert!(
        !source_p296_tree.contains(Q29_TITLE),
        "HWPX Q29 표는 p296으로 분할되어 반복되면 안 된다"
    );
    assert!(
        source_p296_tree.contains(Q30_TITLE),
        "HWPX Q30 표제는 PDF/native HWP와 같이 p296에서 시작해야 한다"
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
        (PAGE_314, source_p314_tree),
        (PAGE_283, source_p283_tree),
        (PAGE_284, source_p284_tree),
        (PAGE_285, source_p285_tree),
        (PAGE_286, source_p286_tree),
        (PAGE_287, source_p287_tree),
        (PAGE_290, source_p290_tree),
        (PAGE_291, source_p291_tree),
        (PAGE_294, source_p294_tree),
        (PAGE_295, source_p295_tree),
        (PAGE_296, source_p296_tree),
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

/// native HWP 원본의 6x5 Q&A RowBreak 표에서 짧은 마지막 응답 tail은 저장
/// frame owner를 유지한다. Stage 131의 terminal spacer/guide 보정은 386쪽을 383쪽으로 낮추며,
/// HWPX fixture의 저장/roundtrip page-count 계약과는 별도로 고정한다 (#3820).
#[test]
fn issue_3820_hwp5_qa_rowbreak_tail_reduces_page_count() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP_FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let source = HwpDocument::from_bytes(&bytes).expect("HWP fixture parse");

    assert!(
        page_tree(&source, 284).contains("홈페이지상의 질의에 대하여"),
        "Hancom PDF physical p285와 같이 Q8 표제는 Q7 tail 뒤 같은 쪽에서 시작해야 한다"
    );
    assert_eq!(
        source.page_count(),
        383,
        "native HWP Q&A PageHide/RowBreak owner 보정 뒤 Hancom PDF 쪽수"
    );
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

//! 실제 HWP3 비밀번호 fixture의 복호화 회귀 계약.
//!
//! HWP3 암호 문서는 DES 복호화 뒤 raw DEFLATE 본문 앞에 256바이트 암호 확인 블록을 둔다.
//! 합성 crypto test만으로는 그 경계·실제 글꼴/문단 구조·공용 열기 API 회귀를 막을 수
//! 없으므로, 무입력·오입력·성공·저장 후 평문 재열기를 실제 fixture로 함께 고정한다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use rhwp::model::control::Control;
use rhwp::model::style::FillType;
use rhwp::parser::{parse_document, ParseError};
use rhwp::{parse_document_with_password, wasm_api::HwpDocument};

const FIXTURE: &str = "samples/HWP3-password-123456.hwp";
const HWPX_COMPARISON_FIXTURE: &str = "samples/HWP5-nopassword-123456.hwpx";
const WRONG_PASSWORD_MESSAGE: &str = "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다";
const FIXTURE_PASSWORD: &[u8] = &[49, 50, 51, 52, 53, 54];

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

fn fixture_bytes() -> Vec<u8> {
    std::fs::read(fixture_path()).expect("암호 HWP3 fixture를 읽어야 함")
}

fn comparison_hwpx_bytes() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(HWPX_COMPARISON_FIXTURE))
        .expect("HWP3 비교용 HWPX fixture를 읽어야 함")
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn run_with_password_stdin(args: &[&str], password: &[u8]) -> Output {
    let mut child = Command::new(rhwp_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    let mut stdin = child.stdin.take().expect("stdin pipe");
    stdin.write_all(password).expect("비밀번호 쓰기");
    stdin.write_all(b"\n").expect("개행 쓰기");
    drop(stdin);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

#[test]
fn actual_hwp3_password_fixture_requires_the_password_and_preserves_structure() {
    let bytes = fixture_bytes();

    assert!(matches!(
        parse_document(&bytes),
        Err(ParseError::EncryptedDocument)
    ));

    let wrong = parse_document_with_password(&bytes, b"wrong-fixture-password")
        .expect_err("잘못된 비밀번호는 문서를 열면 안 됨");
    assert!(
        wrong.to_string().contains(WRONG_PASSWORD_MESSAGE),
        "wrong password error: {wrong}"
    );

    let document = parse_document_with_password(&bytes, FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    assert_eq!(document.header.version.major, 3);
    assert!(document.header.encrypted);
    assert!(document.header.compressed);
    assert_eq!(document.sections.len(), 1);
    assert_eq!(
        document
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>(),
        365
    );
    // 추가정보 블록 #6의 쪽 배경 BMP까지 BinData로 복원한다. 이전에는 이
    // 배경을 버려 HWP3 원본만 본문 중간의 큰 색상 그림이 사라졌다.
    assert_eq!(document.bin_data_content.len(), 3);
    let page_border_fill_id = document.sections[0]
        .section_def
        .page_border_fill
        .border_fill_id;
    assert!(
        page_border_fill_id > 0,
        "HWP3 쪽 배경 BorderFill을 연결해야 함"
    );
    let page_border_fill = &document.doc_info.border_fills[(page_border_fill_id - 1) as usize];
    let page_background = page_border_fill
        .fill
        .image
        .as_ref()
        .expect("HWP3 쪽 배경 이미지 채우기를 복원해야 함");
    assert_eq!(
        page_background.fill_mode,
        rhwp::model::style::ImageFillMode::Center
    );
    assert_eq!(
        (page_background.brightness, page_background.contrast),
        (-15, 50)
    );
    assert_eq!(
        page_background.effect, 0,
        "원본 REAL_PIC 효과를 보존해야 함"
    );
    let background_bin = &document.bin_data_content[(page_background.bin_data_id - 1) as usize];
    assert_eq!(background_bin.extension, "bmp");
    assert!(
        background_bin.data.load().starts_with(b"BM"),
        "복원한 쪽 배경은 BMP payload여야 함"
    );

    // HWP3 원문의 본문 첫 도형은 화면에서는 U+FFFC 하나의 마커로 표현되지만,
    // 공통 IR stream에서는 HWP5와 동일하게 8 code unit을 차지해야 한다.
    // 그렇지 않으면 뒤에 저장된 HWP3 LineInfo.start_pos가 도형 뒤 본문에서
    // 7 unit 앞당겨져 줄 경계·글자 모양이 어긋난다.
    let body_with_floating_shape = document.sections[0]
        .paragraphs
        .iter()
        .find(|paragraph| paragraph.text.contains("감사드립니다. \u{FFFC}저희"))
        .expect("첫 본문 떠다니는 도형 문단을 찾아야 함");
    let marker_index = body_with_floating_shape
        .text
        .chars()
        .position(|ch| ch == '\u{FFFC}')
        .expect("떠다니는 도형 마커를 찾아야 함");
    assert_eq!(
        body_with_floating_shape.char_offsets[marker_index + 1]
            - body_with_floating_shape.char_offsets[marker_index],
        8,
        "HWP3 가시 개체 컨트롤은 IR stream에서 8 code unit을 차지해야 함"
    );

    let comparison =
        parse_document(&comparison_hwpx_bytes()).expect("비교용 HWPX fixture를 열어야 함");
    let hwp3_text = document.sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect::<String>();
    let hwpx_text = comparison.sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect::<String>();
    // HWP3 조합형 0xD3C5는 아래아를 포함한 "ᄒᆞᆫ"이다. 기존에는 지원하지
    // 않는 중성으로 간주해 첫 글자를 버렸고, 제목이 "글 97"로 시작했다.
    // 같은 문서의 HWPX는 이 자모열을 명시하므로 두 fixture로 회귀를 고정한다.
    assert!(hwp3_text.contains("ᄒᆞᆫ글 97 안내문"));
    assert!(hwpx_text.contains("ᄒᆞᆫ글\u{2007}97 안내문"));

    // HWP3 원본 머리말의 0x37C0..=0x37C5 graphic char는 HWPX 변환본과
    // 같은 한컴 PUA로 보존해야 한다. 이후 렌더러 공통 표가 이를
    // "한글과컴퓨터"로 투영한다. 이 회귀가 없으면 HWP3만 머리말 좌측이 빈다.
    let hwp3_header_text: String = document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .filter_map(|control| match control {
            rhwp::model::control::Control::Header(header) => Some(
                header
                    .paragraphs
                    .iter()
                    .map(|paragraph| paragraph.text.as_str())
                    .collect::<String>(),
            ),
            _ => None,
        })
        .collect();
    assert!(
        hwp3_header_text.contains("\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4}"),
        "HWP3 머리말 PUA: {hwp3_header_text:?}"
    );

    let hwp_document = HwpDocument::from_bytes_with_password(&bytes, FIXTURE_PASSWORD)
        .expect("공개 HwpDocument API도 fixture를 열어야 함");
    assert_eq!(hwp_document.page_count(), 24);

    let saved = hwp_document
        .export_hwp_native()
        .expect("암호 문서를 일반 HWP로 저장해야 함");
    let reparsed = parse_document(&saved).expect("저장한 일반 HWP를 비밀번호 없이 다시 열어야 함");
    assert!(!reparsed.header.encrypted);
    assert_eq!(reparsed.sections.len(), 1);
    assert_eq!(
        reparsed
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>(),
        365
    );
}

#[test]
fn actual_hwp3_password_fixture_keeps_white_shaded_table_cells_white() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let table = document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 4 && table.col_count == 2 => {
                Some(table.as_ref())
            }
            _ => None,
        })
        .expect("운영 체제/권장 사양 4×2 표를 찾아야 함");

    for cell in table.cells.iter().filter(|cell| cell.col == 1) {
        let fill = &document.doc_info.border_fills[(cell.border_fill_id - 1) as usize].fill;
        assert_eq!(
            fill.fill_type,
            FillType::Solid,
            "우측 셀은 단색 채움이어야 함"
        );
        assert_eq!(
            fill.solid.expect("우측 셀 단색 채움").background_color,
            0x00FF_FFFF,
            "HWP3 표의 색상=흰색·음영=100%는 검정이 아니라 흰 배경이어야 함"
        );
    }
}

#[test]
fn actual_hwp3_password_fixture_preserves_table_triangle_bullets() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let table = document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 4 && table.col_count == 2 => {
                Some(table.as_ref())
            }
            _ => None,
        })
        .expect("운영 체제/권장 사양 4×2 표를 찾아야 함");

    let right_cell_texts: Vec<String> = table
        .cells
        .iter()
        .filter(|cell| cell.col == 1)
        .map(|cell| {
            cell.paragraphs
                .iter()
                .map(|paragraph| paragraph.text.as_str())
                .collect()
        })
        .collect();
    assert_eq!(right_cell_texts.len(), 4, "우측 셀은 네 개여야 함");
    for text in right_cell_texts {
        assert!(
            text.starts_with("▸ "),
            "HWP3 사적 글머리표 0x2F67은 ▸로 보존해야 함: {text:?}"
        );
    }
}

#[test]
fn actual_hwp3_password_fixture_preserves_p3_inline_object_vertical_contract() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let paragraphs = &document.sections[0].paragraphs;

    // HWP5 변환본과 한컴 PDF의 3쪽 저장 흐름 계약. 제목 양옆의 작은 사각형은
    // 일반 제목 텍스트와 한 줄을 공유하므로 160% 줄간격을 유지하고, inline 표는
    // 표 자체 높이 + 2mm 고정 후행간격만 차지한다. 첫 표의 spacing_before=568 HU는
    // 선행 제목의 spacing_after로 이미 반영되어 이중 적용하면 안 된다.
    let expected = [
        (23, 0, 1_600, 960),
        (25, 5_152, 12_920, 600),
        (27, 21_972, 1_600, 960),
        (30, 29_292, 17_188, 600),
        (31, 47_648, 1_000, 600),
    ];

    for (paragraph_index, vertical_pos, line_height, line_spacing) in expected {
        let line = paragraphs[paragraph_index]
            .line_segs
            .first()
            .unwrap_or_else(|| panic!("문단 {paragraph_index}에 저장 줄이 있어야 함"));
        assert_eq!(
            (line.vertical_pos, line.line_height, line.line_spacing),
            (vertical_pos, line_height, line_spacing),
            "p3 문단 {paragraph_index}의 HWP3→HWP5 세로 흐름 계약"
        );
    }
}

#[test]
fn actual_hwp3_password_fixture_preserves_toc_inline_shape_vertical_contract() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let paragraphs = &document.sections[0].paragraphs;

    // HWP3 차례 항목은 inline 도형에 제목을 두고 본문에는 쪽 번호만 둔다.
    // HWP5 변환본과 한컴 PDF의 항목 간 피치는 text_height + 840 HU다. 종전에는
    // 일반 160% 문단 간격(1629/1682 HU)을 쌓아 1–2쪽 목차가 행마다 더 아래로 밀렸다.
    let expected = [
        (12, 51_564, 2_328, 840),
        (13, 55_300, 2_328, 840),
        (14, 59_036, 2_404, 840),
        (15, 62_848, 2_328, 840),
        (16, 0, 2_328, 840),
        (17, 3_736, 2_328, 840),
        (18, 7_472, 2_328, 840),
        (19, 11_208, 2_328, 840),
        (20, 14_944, 2_328, 840),
        (21, 18_680, 2_328, 840),
        (22, 22_416, 2_328, 840),
    ];

    for (paragraph_index, vertical_pos, line_height, line_spacing) in expected {
        let paragraph = &paragraphs[paragraph_index];
        assert!(
            paragraph.text.chars().any(|ch| ch.is_ascii_digit())
                && paragraph
                    .text
                    .chars()
                    .all(|ch| ch.is_ascii_digit() || ch.is_whitespace() || ch == '\u{FFFC}'),
            "차례 항목 {paragraph_index}의 본문은 marker·공백·쪽 번호여야 함: {:?}",
            paragraph.text
        );
        assert!(matches!(
            paragraph.controls.as_slice(),
            [Control::Shape(shape)] if shape.common().treat_as_char
        ));
        let line = paragraph
            .line_segs
            .first()
            .unwrap_or_else(|| panic!("문단 {paragraph_index}에 저장 줄이 있어야 함"));
        assert_eq!(
            (line.vertical_pos, line.line_height, line.line_spacing),
            (vertical_pos, line_height, line_spacing),
            "차례 문단 {paragraph_index}의 HWP3→HWP5 세로 흐름 계약"
        );
    }
}

#[test]
fn actual_hwp3_password_fixture_anchors_inline_folder_table_to_paragraph() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let table = document.sections[0].paragraphs[30]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 1 && table.col_count == 4 => {
                Some(table.as_ref())
            }
            _ => None,
        })
        .expect("폴더 구성 1×4 표를 찾아야 함");

    assert!(
        table.common.treat_as_char,
        "HWP3 ref_pos=0 표는 inline이어야 함"
    );
    assert_eq!(
        table.common.horz_rel_to,
        rhwp::model::shape::HorzRelTo::Para,
        "inline 표의 수평 기준은 종이가 아니라 문단이어야 함"
    );
    assert_eq!(
        table.common.vert_rel_to,
        rhwp::model::shape::VertRelTo::Para,
        "inline 표의 수직 기준은 종이가 아니라 문단이어야 함"
    );
}

#[test]
fn actual_hwp3_password_fixture_keeps_icon_outline_at_column_origin_without_fill() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let paragraph = &document.sections[0].paragraphs[8];

    let picture = paragraph
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Picture(picture) => Some(picture.as_ref()),
            _ => None,
        })
        .expect("아이콘 그림을 찾아야 함");
    assert_eq!(
        picture.common.horz_rel_to,
        rhwp::model::shape::HorzRelTo::Column,
        "HWP3 ref_pos=1 그림은 문단 여백을 중복 적용하지 않는 단 기준이어야 함"
    );

    let rectangle = paragraph
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                rhwp::model::shape::ShapeObject::Rectangle(rectangle) => Some(rectangle),
                _ => None,
            },
            _ => None,
        })
        .expect("아이콘 테두리 사각형을 찾아야 함");
    assert_eq!(
        rectangle.common.horz_rel_to,
        rhwp::model::shape::HorzRelTo::Column,
        "테두리도 그림과 같은 단 기준 원점이어야 함"
    );
    assert_eq!(
        rectangle.drawing.fill.fill_type,
        FillType::None,
        "0x10000000 HWP3 사각형 marker는 아이콘을 덮는 흰 채움이 아니라 no-fill이어야 함"
    );
    assert!(
        rectangle.drawing.fill.solid.is_none(),
        "no-fill 사각형은 단색 채움 데이터를 만들면 안 됨"
    );
    assert_eq!(
        paragraph
            .line_segs
            .iter()
            .take(2)
            .map(|line| (line.column_start, line.segment_width))
            .collect::<Vec<_>>(),
        vec![(3500, 36520), (3500, 36520)],
        "Square 그림 옆 첫 두 줄은 한컴 HWP5 변환본과 같은 cs/sw를 가져야 함"
    );

    let heading_rectangles = document.sections[0].paragraphs[10]
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                rhwp::model::shape::ShapeObject::Rectangle(rectangle) => Some(rectangle),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        heading_rectangles.len(),
        2,
        "차례 제목 양쪽의 inline 사각형 두 개를 찾아야 함"
    );
    assert!(
        heading_rectangles
            .iter()
            .all(|rectangle| rectangle.drawing.border_line.attr & 0x3F == 0),
        "0x10000000 선색 marker는 검정 테두리가 아니라 no-line이어야 함"
    );
}

#[test]
fn actual_hwp3_password_fixture_keeps_regular_page_background_opaque() {
    let mut document = HwpDocument::from_bytes_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let svg = document
        .render_page_svg(0)
        .expect("첫 쪽 SVG를 렌더해야 함");

    assert!(
        svg.contains("rhwp-img-bc-b50c-15"),
        "배경의 HWP 밝기 50·대비 -15 색조는 유지해야 함"
    );
    assert!(
        !svg.contains("<g opacity=\"0.17\">"),
        "일반 쪽 배경의 밝기·대비를 legacy watermark opacity로 처리하면 안 됨"
    );
}

#[test]
fn actual_hwp3_password_fixture_normalizes_hanging_indent_first_line_margin() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");
    let paragraph = &document.sections[0].paragraphs[32];
    assert!(paragraph.text.starts_with("\\HNC\t\t\t"));

    let para_shape = &document.doc_info.para_shapes[paragraph.para_shape_id as usize];
    assert_eq!(
        para_shape.margin_left, 7000,
        "HWP3 음수 들여쓰기의 첫 줄은 raw left_margin+indent 기준이어야 함"
    );
    assert_eq!(
        para_shape.indent, -16456,
        "내어쓰기 폭은 원시 HWP3 값을 그대로 보존해야 함"
    );
}

#[test]
fn actual_hwp3_password_fixture_keeps_hyperlink_internal_vpos_reset_on_next_page() {
    let document = HwpDocument::from_bytes_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 HWP3 fixture를 열어야 함");

    // 문단 258은 hyperlink marker 하나를 포함하지만, 저장 LINE_SEG는 두 번째 줄을
    // 다음 쪽 상단(vpos=0)으로 명시한다. marker를 flow 개체처럼 취급하면 17쪽에
    // 두 줄을 과배치하고 18쪽 첫 줄이 사라진다.
    let page_17 = document.dump_page_items(Some(16));
    let page_18 = document.dump_page_items(Some(17));
    assert!(
        page_17.contains("PartialParagraph  pi=258  lines=0..1"),
        "17쪽에는 reset 전 첫 줄만 남아야 함\n--- page 17 ---\n{page_17}"
    );
    assert!(
        page_18.contains("PartialParagraph  pi=258  lines=1..3"),
        "18쪽은 hyperlink 뒤 저장 reset 줄부터 시작해야 함\n--- page 18 ---\n{page_18}"
    );
}

#[test]
fn cli_password_exit_contract_uses_the_actual_hwp3_fixture() {
    let fixture = fixture_path();
    let fixture = fixture.to_str().expect("utf-8 fixture path");

    let missing = run(&["info", fixture]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("비밀번호가 필요한 암호 문서"),
        "missing password stderr: {}",
        String::from_utf8_lossy(&missing.stderr)
    );

    let wrong = run_with_password_stdin(
        &["info", fixture, "--password-stdin"],
        b"wrong-fixture-password",
    );
    assert_eq!(wrong.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&wrong.stderr).contains(WRONG_PASSWORD_MESSAGE),
        "wrong password stderr: {}",
        String::from_utf8_lossy(&wrong.stderr)
    );

    let opened = run_with_password_stdin(&["info", fixture, "--password-stdin"], FIXTURE_PASSWORD);
    assert_eq!(opened.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&opened.stdout);
    assert!(stdout.contains("암호화: 예"), "CLI stdout: {stdout}");
    assert!(stdout.contains("페이지 수: 24"), "CLI stdout: {stdout}");
}

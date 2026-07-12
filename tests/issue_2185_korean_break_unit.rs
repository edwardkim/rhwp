//! Issue #2185: 한글 줄 나눔 단위(attr1 bit 7) 의미 반전 회귀.
//!
//! 공개 #1949 거대 셀 샘플에서 대상 문단 끝에 `1` 하나를 입력할 때 앞선 줄 경계가
//! 다시 계산되어 문단 모양이 바뀌던 문제를 고정한다. HWP/HWPX 모두 Studio와 같은
//! 지연 페이지네이션 입력 경로를 거친 뒤 원본 형식으로 저장·재로드한다.

use std::fs;
use std::path::Path;
use std::time::Instant;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::{detect_format, FileFormat};
use rhwp::wasm_api::HwpDocument;

const HWP_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const HWPX_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

const SECTION: usize = 0;
const PARENT_PARAGRAPH: usize = 0;
const TABLE_CONTROL: usize = 2;
const CELL: usize = 2;
const TARGET_PARAGRAPH: usize = 5;
const INSERT_OFFSET: usize = 130;
const EXPECTED_LINE_STARTS: [u32; 4] = [0, 44, 84, 122];
const EXPECTED_NEXT_PARAGRAPH_VPOS: i32 = 17_160;
const EXPECTED_PAGE_COUNT: u32 = 115;

#[derive(Clone, Copy)]
enum SampleFormat {
    Hwp,
    Hwpx,
}

impl SampleFormat {
    fn label(self) -> &'static str {
        match self {
            Self::Hwp => "HWP",
            Self::Hwpx => "HWPX",
        }
    }

    fn file_format(self) -> FileFormat {
        match self {
            Self::Hwp => FileFormat::Hwp,
            Self::Hwpx => FileFormat::Hwpx,
        }
    }
}

fn load_sample(relative_path: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {relative_path}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {relative_path}: {e}"))
}

fn cell_paragraphs(core: &DocumentCore) -> &[Paragraph] {
    match &core.document().sections[SECTION].paragraphs[PARENT_PARAGRAPH].controls[TABLE_CONTROL] {
        Control::Table(table) => &table.cells[CELL].paragraphs,
        other => panic!("대상 컨트롤이 표가 아님: {other:?}"),
    }
}

fn target_paragraph(core: &DocumentCore) -> &Paragraph {
    &cell_paragraphs(core)[TARGET_PARAGRAPH]
}

fn line_starts(paragraph: &Paragraph) -> Vec<u32> {
    paragraph
        .line_segs
        .iter()
        .map(|line| line.text_start)
        .collect()
}

fn next_paragraph_vpos(core: &DocumentCore) -> i32 {
    cell_paragraphs(core)[TARGET_PARAGRAPH + 1]
        .line_segs
        .first()
        .expect("대상 다음 셀 문단 LINE_SEG")
        .vertical_pos
}

fn assert_character_break_unit(core: &DocumentCore, label: &str) {
    let paragraph = target_paragraph(core);
    let para_shape = &core.document().doc_info.para_shapes[paragraph.para_shape_id as usize];
    assert_eq!(
        (para_shape.attr1 >> 7) & 1,
        1,
        "{label}: 대상 문단 attr1 bit 7은 글자 단위여야 함 (attr1=0x{:08X})",
        para_shape.attr1
    );
}

fn assert_line_starts(core: &DocumentCore, label: &str) {
    assert_eq!(
        line_starts(target_paragraph(core)),
        EXPECTED_LINE_STARTS,
        "{label}: 앞선 LINE_SEG 경계를 보존해야 함"
    );
}

fn save_in_source_format(doc: &HwpDocument, format: SampleFormat) -> Vec<u8> {
    match format {
        SampleFormat::Hwp => doc.export_hwp_native().expect("편집 HWP 저장"),
        SampleFormat::Hwpx => doc.export_hwpx_native().expect("편집 HWPX 저장"),
    }
}

fn assert_edit_and_roundtrip(relative_path: &str, format: SampleFormat) {
    let label = format.label();
    let load_started = Instant::now();
    let mut doc = load_sample(relative_path);
    let load_elapsed = load_started.elapsed();

    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGE_COUNT,
        "{label}: 입력 전 쪽수"
    );
    assert_character_break_unit(&doc, label);
    assert_line_starts(&doc, label);

    let original_text = target_paragraph(&doc).text.clone();
    assert_eq!(
        original_text.chars().count(),
        INSERT_OFFSET,
        "{label}: 재현 입력 문자 인덱스"
    );
    assert_eq!(
        original_text.encode_utf16().count(),
        INSERT_OFFSET,
        "{label}: 이 BMP 픽스처의 UTF-16 위치도 입력 인덱스와 같아야 함"
    );
    assert!(
        original_text.ends_with("하여 적용한다."),
        "{label}: 재현 대상 문단 확인"
    );
    let original_next_vpos = next_paragraph_vpos(&doc);
    assert_eq!(
        original_next_vpos, EXPECTED_NEXT_PARAGRAPH_VPOS,
        "{label}: 재현 대상 다음 문단의 기준 vpos"
    );

    let edit_started = Instant::now();
    let edit_result = doc
        .insert_text_in_cell_native_deferred_pagination(
            SECTION,
            PARENT_PARAGRAPH,
            TABLE_CONTROL,
            CELL,
            TARGET_PARAGRAPH,
            INSERT_OFFSET,
            "1",
        )
        .unwrap_or_else(|e| panic!("{label}: 지연 페이지네이션 입력: {e}"));
    let edit_elapsed = edit_started.elapsed();
    assert!(
        edit_result.contains("\"charOffset\":131"),
        "{label}: 입력 후 커서 위치가 131이어야 함: {edit_result}"
    );

    assert_eq!(
        target_paragraph(&doc).text,
        format!("{original_text}1"),
        "{label}: 입력한 한 글자만 문단 끝에 추가되어야 함"
    );
    assert_character_break_unit(&doc, &format!("{label} 편집 직후"));
    assert_line_starts(&doc, &format!("{label} 편집 직후"));
    assert_eq!(
        next_paragraph_vpos(&doc),
        original_next_vpos,
        "{label}: 편집 직후 다음 문단 vpos를 보존해야 함"
    );

    let flush_started = Instant::now();
    doc.flush_deferred_pagination()
        .unwrap_or_else(|_| panic!("{label}: 지연 페이지네이션 flush"));
    let flush_elapsed = flush_started.elapsed();
    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGE_COUNT,
        "{label}: 편집 후 쪽수"
    );
    assert_line_starts(&doc, &format!("{label} pagination 후"));
    assert_eq!(
        next_paragraph_vpos(&doc),
        original_next_vpos,
        "{label}: pagination 후 다음 문단 vpos를 보존해야 함"
    );

    let save_started = Instant::now();
    let saved = save_in_source_format(&doc, format);
    let save_elapsed = save_started.elapsed();
    assert_eq!(
        detect_format(&saved),
        format.file_format(),
        "{label}: 원본 형식으로 저장되어야 함"
    );
    drop(doc);

    let reload_started = Instant::now();
    let reopened = HwpDocument::from_bytes(&saved)
        .unwrap_or_else(|e| panic!("{label}: 편집 문서 재로드: {e}"));
    let reload_elapsed = reload_started.elapsed();

    assert_eq!(
        reopened.page_count(),
        EXPECTED_PAGE_COUNT,
        "{label}: 재로드 후 쪽수"
    );
    assert_character_break_unit(&reopened, &format!("{label} 재로드"));
    assert_line_starts(&reopened, &format!("{label} 재로드"));
    assert_eq!(
        target_paragraph(&reopened).text,
        format!("{original_text}1"),
        "{label}: 재로드 후 편집 텍스트 보존"
    );
    assert_eq!(
        next_paragraph_vpos(&reopened),
        original_next_vpos,
        "{label}: 재로드 후 다음 문단 vpos를 보존해야 함"
    );

    eprintln!(
        "[issue2185] {label}: load={load_elapsed:?} edit={edit_elapsed:?} \
         flush={flush_elapsed:?} save={save_elapsed:?} reload={reload_elapsed:?}"
    );
}

#[test]
fn single_character_edit_keeps_giant_cell_lines_in_hwp_and_hwpx() {
    assert_edit_and_roundtrip(HWP_SAMPLE, SampleFormat::Hwp);
    assert_edit_and_roundtrip(HWPX_SAMPLE, SampleFormat::Hwpx);
}

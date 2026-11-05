//! 관계형 회귀: 각주·미주 구분선의 원본 모양 속성을 보존한다. PR #8536과 같은 SHA의 note-formats 원본·PDF를 재사용한다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! note-formats.hwp (8536-note-formats-hwp-2022): 전 19쪽, Native 최저 99.98307%, fresh WASM 최저 99.98279%.
//! note-formats.hwpx (8536-note-formats-hwpx-2022): 전 19쪽, Native 최저 99.98307%, fresh WASM 최저 99.98279%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 주석 구분선 종류의 HWP 코드와 HWPX 이름.
//!
//! 시험 문서는 한/글 12.30(Mac)이 연 뒤 HWP·HWPX로 저장한 19구역 문서다. 구역마다 각주
//! 구분선 종류가 다르다. 한/글은 같은 구분선을 HWP에 아래 코드로, HWPX에 아래 이름으로 쓴다.
//! 테두리 선과 같은 표이며 HWP 파선 2가 `DOT`, 점선 3이 `DASH`다. 인쇄 PDF
//! (`pdf/note_separator/`)는 2를 긴 파선, 3을 점선으로 그린다. 한/글은 `DOUBLE_WAVE`를
//! 구분선으로 쓰지 않아 그 구역(13)과 기호 번호 구역(18·19)은 실선(1)이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const DIR: &str = "tests/fixtures/note_separator";

const HANCOM: [(u8, &str); 19] = [
    (1, "SOLID"),
    (2, "DOT"),
    (3, "DASH"),
    (4, "DASH_DOT"),
    (5, "DASH_DOT_DOT"),
    (6, "LONG_DASH"),
    (7, "CIRCLE"),
    (8, "DOUBLE_SLIM"),
    (9, "SLIM_THICK"),
    (10, "THICK_SLIM"),
    (11, "SLIM_THICK_SLIM"),
    (12, "WAVE"),
    (1, "SOLID"),
    (14, "THICK3D"),
    (15, "THICKREV3D"),
    (16, "3D"),
    (17, "REV3D"),
    (1, "SOLID"),
    (1, "SOLID"),
];

fn read(name: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR).join(name)).unwrap()
}

fn line_codes(core: &DocumentCore) -> Vec<u8> {
    core.document()
        .sections
        .iter()
        .map(|section| section.section_def.footnote_shape.separator_line_type)
        .collect()
}

/// 구역 XML마다 `footNotePr`의 `noteLine` 종류
fn line_names(hwpx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    (0..HANCOM.len())
        .map(|index| {
            let mut xml = String::new();
            std::io::Read::read_to_string(
                &mut zip
                    .by_name(&format!("Contents/section{index}.xml"))
                    .expect("section"),
                &mut xml,
            )
            .unwrap();
            let note = &xml[xml.find("<hp:footNotePr>").expect("footNotePr")..];
            let line = &note[note.find("<hp:noteLine ").unwrap()..];
            let start = line.find(r#"type=""#).unwrap() + 6;
            line[start..start + line[start..].find('"').unwrap()].to_string()
        })
        .collect()
}

#[test]
fn note_separator_types_match_hancom_hwp_and_hwpx() {
    let codes: Vec<u8> = HANCOM.iter().map(|(code, _)| *code).collect();
    let names: Vec<&str> = HANCOM.iter().map(|(_, name)| *name).collect();

    let hwp = DocumentCore::from_bytes(&read("line-types.hwp")).expect("open hwp");
    assert_eq!(line_codes(&hwp), codes);
    assert_eq!(line_names(&read("line-types.hwpx")), names);

    // 한/글 HWP → HWPX 저장은 한/글이 쓴 이름과 같다.
    assert_eq!(line_names(&hwp.export_hwpx_native().unwrap()), names);
    // 한/글 HWPX → HWP 저장은 한/글이 쓴 코드와 같다.
    let hwpx = DocumentCore::from_bytes(&read("line-types.hwpx")).expect("open hwpx");
    assert_eq!(line_codes(&hwpx), codes);
    let saved = DocumentCore::from_bytes(&hwpx.export_hwp_native().unwrap()).expect("reopen");
    assert_eq!(line_codes(&saved), codes);
}

//! 관계형·고정형: 왕복 전후의 줄표·셀 방향·쪽별 본문 비교(관계)와, 원본의 줄표 문자·셀 방향
//! 값을 한컴 저장본·Print 에서 정한 값(`'\0'`, `'-'`, 2, 1)으로 직접 확인(고정)한다.
//! 주석 갱신: 2026-10-31T06:25:00+09:00; 직전 파일 커밋: 2026-10-31T05:08:14+09:00 (2d17f0045).
//! 2026-10-31 devel 55f40ecd9 위 head 측정 raw(96dpi, pdftoppm, `scripts/visual_sweep.py
//! --compat 2022`, head 의 HWPX 저장본은 `rhwp export-hwpx` 로 생성), 핵심 쪽 Native / fresh
//! WASM, 전체 쪽 최저 Native / fresh WASM:
//! - `samples/hwp-3.0-HWPML.hwp` HWPX 저장본 ↔ `pdf/hwp-3.0-HWPML-2020.pdf`: 56쪽 100.00000% /
//!   100.00000%; 전122쪽 최저 92.77% / 92.76% (원본 HWP 도 같은 값).
//! - `samples/table-vpos-01.hwpx` 원본·HWPX 저장본 ↔ `pdf/table-vpos-01-hwpx-2020.pdf`: 2쪽
//!   99.75829% / 99.83761%; 전5쪽 최저 99.29% / 99.41%.
//! - `samples/k-water-rfp.hwp` HWPX 저장본 ↔ `pdf/k-water-rfp-2022.pdf`: 13쪽 99.48712% /
//!   99.50864%; 전27쪽 최저 97.16% / 97.16%.
//! - `samples/hwpx/k-water-rfp.hwpx` ↔ `pdf/k-water-rfp-2022.pdf`: 13쪽 99.36856% / 99.41063%;
//!   전27쪽 최저 97.16% / 97.16%.
//! - `samples/issue6029/3200477_icao_procedure.hwpx` 원본·HWPX 저장본 ↔
//!   `pdf/issue6029-3200477_icao_procedure-2024.pdf`: 1쪽 97.77789% / 98.54904%; 전1쪽 같은 값.
//!
//! 근거: mydocs/pr/assets/issue_7633_8334/{native,wasm}/ 의 `*_silhouette.tsv`·review·overlay PNG,
//! mydocs/pr/assets/issue_7633_8334/zoom/.
//!
//! [Issue #7633 · #8334] HWPX 읽기·쓰기가 쪽 번호 줄표와 셀 세로쓰기 방향을 보존한다.
//!
//! - #7633: HWP5 `pgnp` 의 줄표 문자가 0 인 쪽 번호(한컴 PDF `1`)를 HWPX 에
//!   `sideChar="-"` 로 써서 `- 1 -` 가 됐다. 한/글은 `sideChar=""`·공백·속성 없음을 줄표
//!   없음으로, `"-"` 만 줄표로 그린다(#7633 실측). 기준: `pdf/hwp-3.0-HWPML-2020.pdf`.
//! - #8334: 셀 방향 값 2(세로 영문 세움)를 HWPX 에 `HORIZONTAL` 로 쓰고, 한컴 HWPX 의
//!   `VERTICALALL` 을 가로로 읽었다. 같은 문서의 한컴 저장본 HWP·HWPX 대응(k-water-rfp)에서
//!   값 2 인 세로 셀 3개가 `VERTICALALL` 3건이다. 기준: `pdf/k-water-rfp-2022.pdf`.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::Cell;

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

fn reopen_as_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기")
}

/// 쪽마다 본문 글 — 쪽 소속·순서·누락·중복을 함께 본다.
fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count() as usize)
        .map(|page| core.page_text(page).expect("쪽 글"))
        .collect()
}

/// 문서의 모든 쪽 번호 위치 컨트롤의 줄표 문자 (본문·표 셀 포함).
fn page_number_dashes(core: &DocumentCore) -> Vec<char> {
    fn walk(paragraphs: &[Paragraph], out: &mut Vec<char>) {
        for para in paragraphs {
            for control in &para.controls {
                match control {
                    Control::PageNumberPos(pn) => out.push(pn.dash_char),
                    Control::Table(table) => {
                        for cell in &table.cells {
                            walk(&cell.paragraphs, out);
                        }
                    }
                    Control::Header(header) => walk(&header.paragraphs, out),
                    Control::Footer(footer) => walk(&footer.paragraphs, out),
                    _ => {}
                }
            }
        }
    }
    let mut out = Vec::new();
    for section in &core.document().sections {
        walk(&section.paragraphs, &mut out);
    }
    out
}

/// 첫 문단 글이 `text` 로 시작하는 표 셀의 방향 (본문 표와 중첩 표 포함).
fn cell_directions(core: &DocumentCore, texts: &[&str]) -> Vec<u8> {
    fn walk<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<&'a Cell>) {
        for para in paragraphs {
            for control in &para.controls {
                if let Control::Table(table) = control {
                    for cell in &table.cells {
                        out.push(cell);
                        walk(&cell.paragraphs, out);
                    }
                }
            }
        }
    }
    let mut cells = Vec::new();
    for section in &core.document().sections {
        walk(&section.paragraphs, &mut cells);
    }
    texts
        .iter()
        .map(|text| {
            cells
                .iter()
                .find(|cell| cell.paragraphs.first().is_some_and(|p| p.text == *text))
                .unwrap_or_else(|| panic!("셀 {text:?} 이 없다"))
                .text_direction
        })
        .collect()
}

const KWATER_VERTICAL_CELLS: [&str; 3] = ["인프라(서버)", "데이터", "분석모델"];

/// #7633: 줄표 없는 쪽 번호는 HWPX 왕복 뒤에도 줄표 없이 그려진다.
#[test]
fn page_number_without_dash_survives_hwpx_roundtrip() {
    let original = load("samples/hwp-3.0-HWPML.hwp");
    let dashes = page_number_dashes(&original);
    assert!(
        !dashes.is_empty() && dashes.iter().all(|c| *c == '\0'),
        "원본은 줄표 없음: {dashes:?}"
    );

    let hwpx = reopen_as_hwpx(&original);
    assert_eq!(page_number_dashes(&hwpx), dashes, "HWPX 왕복 뒤 줄표 문자");
    assert_eq!(
        page_texts(&hwpx),
        page_texts(&original),
        "쪽 번호를 포함한 쪽별 본문이 원본과 같아야 한다"
    );
}

/// #7633 반례: 한컴 HWPX 의 `sideChar="-"` 는 줄표로 읽고, 다시 저장해도 줄표를 둔다.
#[test]
fn hancom_dash_page_number_keeps_dash() {
    let original = load("samples/table-vpos-01.hwpx");
    let dashes = page_number_dashes(&original);
    assert!(
        !dashes.is_empty() && dashes.iter().all(|c| *c == '-'),
        "{dashes:?}"
    );

    let hwpx = reopen_as_hwpx(&original);
    assert_eq!(page_number_dashes(&hwpx), dashes);
    assert_eq!(
        page_texts(&hwpx),
        page_texts(&original),
        "쪽 번호를 포함한 쪽별 본문이 원본과 같아야 한다"
    );
}

/// #8334: 세로 영문 세움 셀(값 2)은 HWPX 왕복 뒤에도 세로로 남는다.
#[test]
fn vertical_all_cell_survives_hwpx_roundtrip() {
    let original = load("samples/k-water-rfp.hwp");
    assert_eq!(
        cell_directions(&original, &KWATER_VERTICAL_CELLS),
        [2, 2, 2]
    );

    let hwpx = reopen_as_hwpx(&original);
    assert_eq!(cell_directions(&hwpx, &KWATER_VERTICAL_CELLS), [2, 2, 2]);
    assert_eq!(
        page_texts(&hwpx),
        page_texts(&original),
        "쪽별 본문이 원본과 같아야 한다"
    );
}

/// #8334: 한컴이 저장한 HWPX 의 `VERTICALALL` 셀은 같은 문서의 HWP 와 같게 열린다.
#[test]
fn hancom_vertical_all_cell_reads_like_hwp() {
    let hwp = load("samples/k-water-rfp.hwp");
    let hwpx = load("samples/hwpx/k-water-rfp.hwpx");
    assert_eq!(cell_directions(&hwpx, &KWATER_VERTICAL_CELLS), [2, 2, 2]);
    let (hwp_texts, hwpx_texts) = (page_texts(&hwp), page_texts(&hwpx));
    assert_eq!(hwpx_texts.len(), hwp_texts.len());
    assert_eq!(hwpx_texts[12], hwp_texts[12], "세로 셀이 있는 13쪽 본문");
}

/// #8334 반례: 세로(영문 눕힘, `VERTICAL`) 셀은 종전대로 값 1 로 읽고 `VERTICAL` 로 쓴다.
#[test]
fn vertical_cell_keeps_value_one() {
    fn directions(core: &DocumentCore) -> Vec<u8> {
        fn walk(paragraphs: &[Paragraph], out: &mut Vec<u8>) {
            for para in paragraphs {
                for control in &para.controls {
                    if let Control::Table(table) = control {
                        for cell in &table.cells {
                            if cell.text_direction != 0 {
                                out.push(cell.text_direction);
                            }
                            walk(&cell.paragraphs, out);
                        }
                    }
                }
            }
        }
        let mut out = Vec::new();
        for section in &core.document().sections {
            walk(&section.paragraphs, &mut out);
        }
        out
    }
    let original = load("samples/issue6029/3200477_icao_procedure.hwpx");
    assert_eq!(directions(&original), [1, 1, 1]);
    assert_eq!(directions(&reopen_as_hwpx(&original)), [1, 1, 1]);
}

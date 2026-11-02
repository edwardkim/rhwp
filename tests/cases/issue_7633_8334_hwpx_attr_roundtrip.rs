//! 관계형·고정형: 왕복 내용·속성 관계와 독립 저장본의 줄표·방향 값을 확인한다.
//! 주석 갱신: 2026-11-02T20:27:59+09:00; 직전 파일 커밋: 2026-11-01T19:38:13+09:00 (a2541933cb).
//! 줄표 문자와 셀 방향은 한컴 저장본·Print에서 정한 값(`'\0'`, `'-'`, 2, 1)으로 확인한다.
//! WMF 공백 요청과 글꼴 cmap 검사는 원본과 라이선스 확인 fixture의 비조판 계약이다.
//! 2026-11-02 보정 전 독립 Print 대비 HWPX 저장본 178쪽: 94쪽 및 전쪽 최저 Native 92.66%,
//! fresh WASM 92.56% (96dpi raw). 글머리표 문자와 문단 소유는 관계로 검사한다.
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

/// WMF 글꼴 요청은 렌더링이 아닌 원본의 text/cmap 공급 계약이다.
#[test]
fn wmf_space_requests_remain_visible_to_font_selection() {
    let document = load("mydocs/pr/assets/issue_7633_8334/followup8438/hwpspec-saved.hwpx");
    let json = document
        .get_metafile_font_requests_native()
        .expect("WMF 요청");
    let requests: Vec<serde_json::Value> = serde_json::from_str(&json).expect("WMF 요청 JSON");
    assert!(!requests.is_empty(), "원본 WMF의 요청이 있어야 한다");
    assert!(
        requests
            .iter()
            .all(|item| item["text"].as_str().is_some_and(|text| !text.is_empty())),
        "내용이 없는 텍스트는 전체 대체 글꼴 요청을 만들지 않는다"
    );
    assert!(
        requests.iter().any(
            |item| item["families"].as_array().is_some_and(|families| families
                .first()
                .and_then(|family| family.as_str())
                == Some("Times New Roman"))
                && item["text"].as_str() == Some(" ")
        ),
        "원본 WMF의 독립 공백은 글꼴 선택에 전달한다"
    );
    assert!(
        requests.iter().any(|item| item["text"]
            .as_str()
            .is_some_and(|text| text.contains(' ')
                && text.chars().any(|character| !character.is_whitespace()))),
        "문자와 공백이 같은 글꼴 요청에 함께 남는다"
    );
}

/// 설치 글꼴 없이 저장소의 라이선스 확인 fixture로 공백 cmap을 판별한다.
#[test]
fn wmf_space_coverage_uses_the_supplied_program() {
    use rhwp::renderer::font_symbol_coverage::{font_missing_text, font_missing_text_with_spaces};
    let no_space = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let has_space = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    assert_eq!(font_missing_text(no_space, 0, " A").as_deref(), Some(""));
    assert_eq!(
        font_missing_text_with_spaces(no_space, 0, " A").as_deref(),
        Some(" ")
    );
    assert_eq!(
        font_missing_text_with_spaces(has_space, 0, " A").as_deref(),
        Some("")
    );
}

/// 참조 없는 그림 글머리표는 한컴 Print처럼 문자를 사용하고 원본 속성은 보존한다.
#[test]
fn hwpx_bullet_without_image_reference_uses_character_and_roundtrips() {
    let core = load("mydocs/pr/assets/issue_7633_8334/followup8438/hwpspec-saved.hwpx");
    let (paragraph_index, paragraph) = core.document().sections[3]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.starts_with("대각선 merge cell은"))
        .expect("독립 Print에 대응하는 문단");
    let style = &core.document().doc_info.para_shapes[paragraph.para_shape_id as usize];
    let bullet = &core.document().doc_info.bullets[usize::from(style.numbering_id) - 1];
    assert_eq!(bullet.bullet_char, '●');
    assert_eq!(
        bullet.image_bullet, 0,
        "참조가 없는데 다른 그림을 선택하면 안 된다"
    );
    assert!(
        bullet.hwpx_image_without_ref,
        "원래 useImage 속성은 보존해야 한다"
    );

    let tree: serde_json::Value = serde_json::from_str(
        &core
            .build_page_render_tree(93)
            .expect("94쪽 조판")
            .root
            .to_json(),
    )
    .unwrap();
    fn walk<'a>(value: &'a serde_json::Value, out: &mut Vec<&'a serde_json::Value>) {
        out.push(value);
        if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
            for child in children {
                walk(child, out);
            }
        }
    }
    let mut nodes = Vec::new();
    walk(&tree, &mut nodes);
    let owned: Vec<_> = nodes
        .into_iter()
        .filter(|v| v["pi"].as_u64() == Some(paragraph_index as u64))
        .collect();
    assert!(
        owned
            .iter()
            .any(|v| v["type"] == "TextRun" && v["text"] == "●"),
        "해당 문단의 문자 마커"
    );
    assert!(
        !owned.iter().any(|v| v["type"] == "Image"),
        "참조 없는 그림은 출력하지 않는다"
    );
    assert!(
        owned
            .iter()
            .any(|v| v["type"] == "TextRun"
                && v["text"].as_str().is_some_and(|t| t.contains("대각선"))),
        "본문 보존"
    );

    let reopened = reopen_as_hwpx(&core);
    let same = &reopened.document().doc_info.bullets[usize::from(style.numbering_id) - 1];
    assert_eq!(same.bullet_char, '●');
    assert_eq!(same.image_bullet, 0);
    assert!(
        same.hwpx_image_without_ref,
        "useImage=1이 왕복에서도 유지된다"
    );
}

/// 명시한 그림 참조와 일반 문자 글머리표는 각각 기존 계약대로 보존한다.
#[test]
fn hwpx_bullet_explicit_image_and_plain_character_remain_distinct() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?><hh:head xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head"><hh:refList><hh:bullets itemCnt="2"><hh:bullet id="1" char="●" useImage="1"><hh:img binaryItemIDRef="image3"/><hh:paraHead level="0"/></hh:bullet><hh:bullet id="2" char="■" useImage="0"><hh:paraHead level="0"/></hh:bullet></hh:bullets></hh:refList></hh:head>"#;
    let (info, _) = rhwp::parser::hwpx::header::parse_hwpx_header(xml).expect("HWPX 머리글");
    assert_eq!(info.bullets[0].image_bullet, 3);
    assert!(!info.bullets[0].hwpx_image_without_ref);
    assert_eq!(info.bullets[1].bullet_char, '■');
    assert_eq!(info.bullets[1].image_bullet, 0);
    assert!(!info.bullets[1].hwpx_image_without_ref);
}

/// 이어진 표 뒤 두 문단의 페이지 안 상대 위치는 원본 저장 사다리를 따른다.
#[test]
fn continued_table_successor_lines_preserve_source_origin_relationship() {
    let core = load("mydocs/pr/assets/issue_7633_8334/followup8438/hwpspec-saved.hwpx");
    let paragraphs = &core.document().sections[3].paragraphs;
    let (first_index, first) = paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.starts_with("하이퍼텍스트의 경우에는"))
        .expect("표 다음 본문");
    let blank_index = first_index.checked_sub(1).expect("앞 빈 문단");
    let blank = &paragraphs[blank_index];
    assert!(blank.text.is_empty());
    let blank_source = f64::from(blank.line_segs[0].vertical_pos);
    let first_source = f64::from(first.line_segs[0].vertical_pos);
    assert!(blank_source > 0.0 && first_source > blank_source);
    let tree: serde_json::Value = serde_json::from_str(
        &core
            .build_page_render_tree(93)
            .expect("94쪽 조판")
            .root
            .to_json(),
    )
    .unwrap();
    fn walk<'a>(v: &'a serde_json::Value, out: &mut Vec<&'a serde_json::Value>) {
        out.push(v);
        if let Some(children) = v.get("children").and_then(|x| x.as_array()) {
            for child in children {
                walk(child, out);
            }
        }
    }
    let mut nodes = Vec::new();
    walk(&tree, &mut nodes);
    let body = nodes
        .iter()
        .find(|n| n["type"] == "Body")
        .expect("본문 영역");
    let body_top = body["bbox"]["y"].as_f64().unwrap();
    let line_top = |index| {
        nodes
            .iter()
            .find(|n| n["type"] == "TextLine" && n["pi"].as_u64() == Some(index as u64))
            .expect("문단 첫 줄")["bbox"]["y"]
            .as_f64()
            .unwrap()
    };
    let blank_offset = line_top(blank_index) - body_top;
    let first_offset = line_top(first_index) - body_top;
    assert!(blank_offset > 0.0 && first_offset > blank_offset);
    let observed = blank_offset / first_offset;
    let expected = blank_source / first_source;
    assert!(
        (observed - expected).abs() < 0.002,
        "표 종료 뒤 빈 문단과 본문의 본문 영역 대비 위치 관계: 실제={observed}, 원본={expected}"
    );
}

//! 관계형·고정형: HWPX 저장본과 원본의 쪽별 본문 비교(관계)와, 저장한 `sideChar` 값·쪽 번호
//! 글(`1`, `- 2 -`, `1`..`5`)을 한컴 2024 Print 에서 정한 값으로 직접 확인(고정)한다.
//! 주석 갱신: 2026-10-31T00:20:00+09:00; 직전 파일 커밋: 2026-10-30T21:55:47+09:00 (a7abcc41a).
//! 2026-10-30 측정 raw(96dpi, pdftoppm, `scripts/visual_sweep.py --compat 2022`), 핵심 쪽
//! Native / fresh WASM, 전체 쪽 최저 Native / fresh WASM:
//! - `samples/hwpspec.hwp` ↔ `pdf/hwpspec-2024.pdf`: 11쪽 99.70223% / 99.70229%; 전178쪽 최저
//!   92.39% / 92.38%. HWPX 저장본 ↔ `pdf/page-number-side-char/hwpspec-rhwp-hwpx-2024.pdf`:
//!   11쪽 99.67885% / 99.67892%; 전178쪽 최저 92.78% / 92.77%.
//! - `samples/hwp-3.0-HWPML.hwp` HWPX 저장본 ↔
//!   `pdf/page-number-side-char/hwp-3.0-HWPML-rhwp-hwpx-2024.pdf`: 20쪽 99.97955% / 99.99319%;
//!   전122쪽 최저 92.77% / 92.76%.
//! - `samples/issue6924/148751598-briefing.hwp` ↔ `pdf/148751598-briefing-2020.pdf`: 2쪽
//!   97.97793% / 97.96957%; 전6쪽 최저 96.25% / 96.19%. HWPX 저장본 ↔
//!   `pdf/page-number-side-char/148751598-briefing-rhwp-hwpx-2024.pdf`: 2쪽 98.07079% / 98.06380%;
//!   전6쪽 최저 96.27% / 96.20%.
//! - `samples/page-number-side-char/table-vpos-01-no-dash.hwpx` ↔
//!   `pdf/page-number-side-char/table-vpos-01-no-dash-2024.pdf`: 2쪽 99.75823% / 99.83757%;
//!   전5쪽 최저 99.29% / 99.41%.
//!
//! 근거: mydocs/pr/assets/issue_7633_8334/sidechar/{native,wasm}/ 의 `*_silhouette.tsv`·review·
//! overlay PNG, mydocs/pr/assets/issue_7633_8334/zoom/page_numbers_pdf_native_wasm_devel.png.
//!
//! [Issue #7633] 쪽 번호 줄표(HWP5 PageNumberPos 의 마지막 WCHAR)와 HWPX `sideChar` 를 같은 값으로
//! 왕복한다. `issue_7633_8334_hwpx_attr_roundtrip.rs` 의 HWPML 왕복을 한컴 Print 기준으로 보강한다.
//!
//! 한컴 2024(13.0.0.3901)로 대응을 확인했다(`hwpx_sample2.hwp` → 한컴 HWPX → `sideChar=""` 편집 →
//! 한컴 HWP 의 `pgnp` 줄표 0 → 한컴 HWPX `sideChar=""`).
//!
//! ```text
//!   HWP5 줄표 0x2d  <->  <hp:pageNum sideChar="-"/>   Print: "- 2 -"
//!   HWP5 줄표 0     <->  <hp:pageNum sideChar=""/>    Print: "2"
//! ```
//!
//! 종전 rhwp 는 HWPX 저장에서 줄표 0 을 `-` 로 채우고, 읽기에서 빈 `sideChar` 를 `-` 로
//! 읽었다. 그래서 줄표 없는 문서(`samples/hwpspec.hwp`, `samples/hwp-3.0-HWPML.hwp`)를 HWPX 로
//! 저장하면 쪽 번호가 `- 1 -` 이 됐다. 원본의 한컴 Print(`pdf/hwpspec-2024.pdf`)와 rhwp HWPX
//! 저장본의 한컴 Print(`pdf/page-number-side-char/hwpspec-rhwp-hwpx-2024.pdf`)는 모두 `1` 이다.
//!
//! `samples/page-number-side-char/table-vpos-01-no-dash.hwpx` 는 한컴이 저장한 `samples/table-vpos-01.hwpx`
//! 에서 `sideChar="-"` 하나만 `""` 로 바꾼 입력이다. 한컴 2024 Print
//! (`pdf/page-number-side-char/table-vpos-01-no-dash-2024.pdf`)는 5쪽 모두 줄표 없는 `1`..`5` 다.
//! 원본의 Print(`pdf/table-vpos-01-2022.pdf`)는 `- 1 -` 이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::io::{Cursor, Read};

fn read(rel: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).expect("표본 읽기")
}

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("문서 열기")
}

/// 저장한 HWPX 의 모든 `hp:pageNum` 의 `sideChar` 값.
fn side_chars(hwpx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(hwpx)).expect("HWPX zip");
    let names: Vec<String> = (0..zip.len())
        .map(|index| {
            zip.by_index(index)
                .expect("엔트리")
                .name()
                .expect("엔트리 이름")
                .to_string()
        })
        .filter(|name| name.starts_with("Contents/section") && name.ends_with(".xml"))
        .collect();
    let mut sides = Vec::new();
    for name in names {
        let mut xml = String::new();
        zip.by_name(&name)
            .expect("section")
            .read_to_string(&mut xml)
            .expect("section 읽기");
        for element in xml.split("<hp:pageNum ").skip(1) {
            let side = element
                .split("sideChar=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .expect("sideChar");
            sides.push(side.to_owned());
        }
    }
    sides
}

/// 쪽마다 마지막 비지 않은 줄 — 꼬리말 가운데 쪽 번호가 있는 문서에서 쪽 번호.
fn last_lines(core: &DocumentCore) -> Vec<String> {
    page_texts(core)
        .iter()
        .map(|text| {
            text.lines()
                .map(str::trim)
                .rfind(|line| !line.is_empty())
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| core.extract_page_text_native(page).expect("쪽 본문"))
        .collect()
}

#[test]
fn hwp_page_number_dash_survives_hwpx_save() {
    // (원본, 기대 sideChar, 한컴 Print 의 (쪽 번호, 꼬리말 쪽 번호))
    for (rel, side, number) in [
        ("samples/hwpspec.hwp", "", Some((11, "1"))),
        // 머리말 문구가 쪽의 마지막 줄이라 쪽 번호 값은 전체 본문 비교로 확인한다.
        ("samples/hwp-3.0-HWPML.hwp", "", None),
        // 줄표가 있는 대조군. `pdf/148751598-briefing-2020.pdf` 2쪽은 `- 2 -`.
        (
            "samples/issue6924/148751598-briefing.hwp",
            "-",
            Some((2, "- 2 -")),
        ),
    ] {
        let source = open(&read(rel));
        let hwpx = source.export_hwpx_native().expect("HWPX 저장");
        let sides = side_chars(&hwpx);
        assert!(!sides.is_empty(), "{rel}: hp:pageNum");
        assert!(sides.iter().all(|s| s == side), "{rel}: sideChar {sides:?}");

        let saved = open(&hwpx);
        if let Some((page, expected)) = number {
            assert_eq!(
                last_lines(&source)[page - 1],
                expected,
                "{rel}: 원본 {page}쪽 쪽 번호"
            );
        }
        // 쪽 번호를 포함한 쪽 본문 전체가 원본과 같다(공백 제외).
        let compact = |core: &DocumentCore| -> Vec<String> {
            page_texts(core)
                .iter()
                .map(|text| text.chars().filter(|c| !c.is_whitespace()).collect())
                .collect()
        };
        assert_eq!(
            compact(&saved),
            compact(&source),
            "{rel}: HWPX 저장본 쪽 본문"
        );
    }
}

#[test]
fn hancom_hwpx_without_side_char_draws_plain_page_numbers() {
    let core = open(&read(
        "samples/page-number-side-char/table-vpos-01-no-dash.hwpx",
    ));
    // 한컴 Print 는 5쪽 모두 아래 가운데에 줄표 없는 쪽 번호를 둔다.
    assert_eq!(
        last_lines(&core),
        ["1", "2", "3", "4", "5"],
        "줄표 없는 쪽 번호"
    );

    // 다시 저장해도 빈 sideChar 를 유지한다.
    let hwpx = core.export_hwpx_native().expect("HWPX 저장");
    assert!(
        side_chars(&hwpx).iter().all(String::is_empty),
        "다시 저장한 sideChar"
    );
}

//! 관계형·고정형 — [#8810] 쪽 기준 자리차지 표의 첫 조각 오프셋과 칸의 낡은 저장 사다리.
//! 주석 갱신: 2026-11-09 11:50 KST (새 파일)
//!
//! 입력: 공개 결재문서 36400259 원본. 기준: 한컴 engine 2020 Print PDF
//! `pdf/issue8810/36400259-stale-cell-ladder-2020.pdf`(2쪽).
//! 본문 전체가 쪽 기준(`vertRelTo=PAGE`, 세로 오프셋 −1426HU) 자리차지 9×11 표 하나이고, 그 마지막
//! 칸(`r=8`)에서 가린 글(`****`) 문단에는 `linesegarray` 가 없다. 나머지 문단의 저장 vpos 는 가리기
//! 전 글로 조판한 값이라 흐름보다 1565HU 앞서 있고, 붙임 문단에서 0 으로 되감긴다.
//! - 고정형: 전체 쪽수 2.
//! - 관계형 1쪽: 표 위끝은 본문 위끝 + 저장 세로 오프셋이다(한/글 PDF 첫 글줄 97.0px, 본문 위 113.4).
//! - 관계형 1쪽: 중첩 표 host(`pi=11`) 다음 문단 `pi=12` 는 host 줄 위끝에서 저장 줄 높이 + 줄간격
//!   (15867 + 720 HU) 아래다 — 낡은 절대 vpos(38987)가 아니다.
//! - 관계형 쪽 소유: 붙임 1·2(`pi=23`·`pi=24`)는 1쪽 본문 안, 2쪽은 `pi=25`(붙임 3)부터 시작한다.
//!
//! 측정(2026-11-09, 이 파일과 같은 커밋의 코드, base = PR #8794 head 85abd69fbb,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource`):
//! - Native: 2쪽 / 2쪽, p1 99.80 · p2 99.98, 최저 99.80 (base: p1 31.60 · p2 17.32)
//! - fresh WASM: 2쪽 / 2쪽, p1 99.74 · p2 99.99, 최저 99.74
//! - 증거: `mydocs/pr/assets/issue_8810/`(native·wasm·native_base TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8810/36400259-stale-cell-ladder.hwpx";

/// HWPUNIT → px(96 DPI).
fn px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 원본 저장값: 바깥 표 세로 오프셋, 중첩 표 host 줄의 높이·줄간격, 중첩 표 바깥 위 여백.
const OUTER_VERTICAL_OFFSET_HU: i32 = -1426;
const NESTED_HOST_LINE_HEIGHT_HU: i32 = 15867;
const NESTED_HOST_LINE_SPACING_HU: i32 = 720;
const NESTED_OUTER_MARGIN_TOP_HU: i32 = 283;

#[derive(Debug, Clone)]
struct Extent {
    page: usize,
    kind: String,
    top: f64,
    bottom: f64,
    rest: String,
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn parse_extent(page: usize, line: &str) -> Option<Extent> {
    let trimmed = line.trim_start();
    let kind = trimmed.split_whitespace().next()?.to_string();
    let y_start = trimmed.find("y=")? + 2;
    let (top, after_top) = trimmed[y_start..].trim_start().split_once("..")?;
    let bottom = after_top.split_whitespace().next()?;
    Some(Extent {
        page,
        kind,
        top: top.trim().parse().ok()?,
        bottom: bottom.trim().parse().ok()?,
        rest: trimmed.to_string(),
    })
}

fn extents() -> (usize, Vec<Extent>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    assert!(path.is_file(), "fixture 누락: {}", path.display());
    let out = Command::new(rhwp_bin())
        .arg("dump-extents")
        .arg(&path)
        .output()
        .expect("dump-extents 실행");
    assert!(out.status.success(), "dump-extents 실패");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut page = 0usize;
    let mut pages = 0usize;
    let mut items = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("=== ") {
            page = rest
                .split('쪽')
                .next()
                .and_then(|n| n.trim().parse().ok())
                .expect("쪽 머리");
            pages = pages.max(page);
            continue;
        }
        if page > 0 {
            if let Some(extent) = parse_extent(page, line) {
                items.push(extent);
            }
        }
    }
    (pages, items)
}

fn find<'a>(items: &'a [Extent], page: usize, kind: &str, tag: &str) -> Option<&'a Extent> {
    items
        .iter()
        .find(|e| e.page == page && e.kind == kind && e.rest.contains(tag))
}

fn body(items: &[Extent], page: usize) -> &Extent {
    find(items, page, "Body", "").unwrap_or_else(|| panic!("{page}쪽 Body 없음"))
}

#[test]
fn page_relative_first_fragment_starts_at_its_stored_offset() {
    let (pages, items) = extents();
    assert_eq!(pages, 2, "한컴 Print PDF 는 2쪽");

    let body = body(&items, 1);
    let table = find(&items, 1, "Table", "pi=0 ci=3 ").expect("1쪽 바깥 표");
    let expected = body.top + px(OUTER_VERTICAL_OFFSET_HU);
    assert!(
        (table.top - expected).abs() <= 0.5,
        "쪽 기준 표 첫 조각 위끝 {:.1} 은 본문 위끝 {:.1} + 오프셋 {:.1} = {:.1}",
        table.top,
        body.top,
        px(OUTER_VERTICAL_OFFSET_HU),
        expected
    );
}

#[test]
fn successor_of_nested_table_host_follows_the_host_line_not_the_stale_ladder() {
    let (_, items) = extents();
    let nested = find(&items, 1, "Table", "pi=11 ci=0 ").expect("1쪽 중첩 표");
    let successor = find(&items, 1, "TextLine", "pi=12 line=0 ").expect("1쪽 pi=12 줄");
    let host_line_top = nested.top - px(NESTED_OUTER_MARGIN_TOP_HU);
    let advance = px(NESTED_HOST_LINE_HEIGHT_HU + NESTED_HOST_LINE_SPACING_HU);
    assert!(
        (successor.top - (host_line_top + advance)).abs() <= 0.5,
        "pi=12 위끝 {:.1} 은 host 줄 위끝 {:.1} + 줄 전진 {:.1} = {:.1}",
        successor.top,
        host_line_top,
        advance,
        host_line_top + advance
    );
}

#[test]
fn stale_rewind_does_not_push_attachment_lines_to_the_next_page() {
    let (pages, items) = extents();
    assert_eq!(pages, 2, "한컴 Print PDF 는 2쪽");
    let body1 = body(&items, 1);
    for para in ["pi=23 line=0 ", "pi=24 line=0 "] {
        let line =
            find(&items, 1, "TextLine", para).unwrap_or_else(|| panic!("1쪽에 `{para}` 없음"));
        assert!(
            line.bottom <= body1.bottom + 0.5,
            "1쪽 `{para}` 아래끝 {:.1} 은 본문 아래끝 {:.1} 안",
            line.bottom,
            body1.bottom
        );
    }
    let first_on_page2 = items
        .iter()
        .find(|e| e.page == 2 && e.kind == "TextLine" && e.rest.contains("pi="))
        .expect("2쪽 첫 글줄");
    assert!(
        first_on_page2.rest.contains("pi=25 line=0 "),
        "2쪽은 붙임 3(`pi=25`)부터: {}",
        first_on_page2.rest
    );
    assert!(
        find(&items, 2, "TextLine", "pi=23 ").is_none()
            && find(&items, 2, "TextLine", "pi=24 ").is_none(),
        "붙임 1·2 가 2쪽에 중복되면 안 된다"
    );
}

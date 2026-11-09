//! 관계형·고정형 — [#8841] 글자처럼 취급 표 줄의 음수 줄간격과 쪽 아래 고정 틀.
//! 주석 갱신: MEASURE_TS (새 파일)
//!
//! 입력: 공개 결재문서 36414883(한컴오피스 2020 저장). 마지막 문단(`pi=7`)은 TAC 표 줄(줄 높이
//! 15811HU, 줄간격 −240HU)과 뒤 글줄 하나, 그리고 쪽 기준·아래 정렬 고정 틀(`ci=1`, 28991HU)로 되어
//! 있다. 저장 줄은 뒤 글줄을 28437 + 15811 − 240 = 44008HU 에 둔다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8841/36414883-negative-tac-line-spacing-2020.pdf`(1쪽).
//! - 고정형: 전체 쪽수 1.
//! - 관계형 1쪽: TAC 표 → 뒤 글줄 → 고정 틀 순서로 겹치지 않고, 고정 틀은 본문 아래끝 안에 있다.
//!
//! MEASUREMENTS
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8841/36414883-negative-tac-line-spacing.hwpx";
const TOLERANCE_PX: f64 = 0.5;

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

fn find<'a>(items: &'a [Extent], kind: &str, tags: &[&str]) -> &'a Extent {
    items
        .iter()
        .find(|e| {
            e.kind == kind
                && tags
                    .iter()
                    .all(|tag| e.rest.split_whitespace().any(|word| word == *tag))
        })
        .unwrap_or_else(|| panic!("{kind} {tags:?} 없음"))
}

#[test]
fn trailing_line_after_negative_spacing_tac_row_stays_above_the_bottom_frame() {
    let (pages, items) = extents();
    assert_eq!(pages, 1, "한컴 Print PDF 는 1쪽");

    let body = find(&items, "Body", &[]);
    let table = find(&items, "Table", &["pi=7", "ci=0"]);
    let line = find(&items, "TextLine", &["pi=7", "line=1"]);
    let frame = find(&items, "Table", &["pi=7", "ci=1"]);
    for item in [table, line, frame] {
        assert_eq!(item.page, 1, "{} 는 1쪽", item.rest);
    }
    assert!(
        table.bottom <= line.top + TOLERANCE_PX,
        "TAC 표 아래에 뒤 글줄: {table:?} / {line:?}"
    );
    assert!(
        line.bottom <= frame.top + TOLERANCE_PX,
        "뒤 글줄은 고정 틀 위에 들어간다: {line:?} / {frame:?}"
    );
    assert!(
        frame.bottom <= body.bottom + TOLERANCE_PX,
        "고정 틀은 본문 아래끝 안: {frame:?} / {body:?}"
    );
}

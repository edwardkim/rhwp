//! 관계형·고정형 — [#8850] 한 문단에 차례로 든 글자처럼 취급 표 둘 — 둘째 표는 첫 표 바로 아래.
//! 주석 갱신: MEASURE_TS (새 파일)
//!
//! 입력: 공개 결재문서 36474334 의 축소본(일상경비집행의뢰·지출결의서 두 TAC 표를 든 `pi=3` 만
//! 남기고 구역 정의를 그 문단으로 옮김, 칸 글자는 같은 수의 공백으로 바꿈). 이 문단은 저장 줄이
//! 없어 로드 시 표마다 한 줄씩(vpos 0·25586) 합성된다.
//! 기준: 같은 축소본의 한컴 engine 2020 Print PDF
//! `pdf/issue8850/36474334-two-tac-tables-reduced-2020.pdf`(1쪽).
//! - 고정형: 전체 쪽수 1.
//! - 관계형 1쪽: 둘째 표(`ci=3`)는 첫 표(`ci=2`) 아래에 바깥 여백만 두고 붙고, 본문 아래끝 안에 있다.
//!
//! MEASUREMENTS
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8850/36474334-two-tac-tables-reduced.hwpx";
/// 두 표 사이에 허용하는 간격 — 바깥 여백(위·아래 각 140HU ≈ 1.9px)보다 크고 표 높이보다 훨씬 작다.
const MARGIN_GAP_PX: f64 = 5.0;
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
fn second_tac_table_follows_the_first_in_the_same_paragraph() {
    let (pages, items) = extents();
    assert_eq!(pages, 1, "한컴 Print PDF 는 1쪽");

    let body = find(&items, "Body", &[]);
    let first = find(&items, "Table", &["pi=0", "ci=2"]);
    let second = find(&items, "Table", &["pi=0", "ci=3"]);
    assert!(
        first.bottom <= second.top + TOLERANCE_PX,
        "둘째 표는 첫 표 아래: {first:?} / {second:?}"
    );
    assert!(
        second.top - first.bottom <= MARGIN_GAP_PX,
        "둘째 표는 첫 표 바로 아래({:.1}px 간격, 첫 표 높이 {:.1}px 를 다시 더하면 안 된다)",
        second.top - first.bottom,
        first.bottom - first.top,
    );
    assert!(
        second.bottom <= body.bottom + TOLERANCE_PX,
        "둘째 표는 본문 아래끝 안: {second:?} / {body:?}"
    );
}

//! 관계형·고정형 — [#8753] 같은 문단의 어울림 표와 TAC 표, 표 옆 차선의 합성 줄 문단.
//! 주석 갱신: 2026-11-08 07:40 KST (새 파일)
//!
//! 고정형: 전체 쪽수 6 (한컴 engine 2020 Print PDF `pdf/issue8753/36458608-square-tac-same-paragraph-2020.pdf`).
//! 관계형: 3쪽 `pi=8` 의 어울림 표(`ci=0`)와 TAC 제목 표(`ci=1`)는 세로로 겹치고 가로로 비켜 있다.
//! 뒤 문단 `pi=9`(저장 차선 줄)·`pi=10`(저장 줄 없는 합성 줄)은 표 왼쪽 차선에서 순서대로 놓이고,
//! 밴드 뒤 `pi=15` 는 표 아래에서 시작한다. 4쪽 끝의 합성 줄 문단 `pi=29` 는 5쪽에 속한다.
//!
//! 측정(2026-11-08, `scripts/visual_sweep.py --silhouette-only --dpi 96`, raw =
//! `silhouette_raw_match_percent`, 문서 face 만 공급한 글꼴 디렉터리):
//! - Native(head 184d207aa3, native-skia release): 6쪽 / 6쪽, p1 100.00 · p2 99.97 · p3 97.67 ·
//!   p4 95.96 · p5 99.18 · p6 100.00, 최저 95.96 (base 5077b216cb 는 7쪽으로 쪽수 불일치)
//! - fresh WASM(head 184d207aa3, wasm sha256 b25997ac…): 6쪽 / 6쪽, p1 100.00 · p2 100.00 · p3 98.53 ·
//!   p4 98.40 · p5 99.15 · p6 100.00, 최저 98.40
//! - 증거: `mydocs/pr/assets/issue_8753/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8753/36458608-square-tac-same-paragraph.hwpx";

#[derive(Debug, Clone)]
struct Extent {
    page: usize,
    kind: String,
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
    rest: String,
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn number_after(line: &str, key: &str) -> Option<f64> {
    let start = line.find(key)? + key.len();
    line[start..].split_whitespace().next()?.parse().ok()
}

fn parse_extent(page: usize, line: &str) -> Option<Extent> {
    let trimmed = line.trim_start();
    let kind = trimmed.split_whitespace().next()?.to_string();
    let y_start = trimmed.find("y=")? + 2;
    let y_text = trimmed[y_start..].trim_start();
    let (top, after_top) = y_text.split_once("..")?;
    let bottom = after_top.split_whitespace().next()?;
    let left = number_after(trimmed, "x=")?;
    let width = number_after(trimmed, "w=")?;
    Some(Extent {
        page,
        kind,
        top: top.trim().parse().ok()?,
        bottom: bottom.trim().parse().ok()?,
        left,
        right: left + width,
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

fn find<'a>(items: &'a [Extent], page: usize, kind: &str, tag: &str) -> &'a Extent {
    items
        .iter()
        .find(|e| e.page == page && e.kind == kind && e.rest.contains(tag))
        .unwrap_or_else(|| panic!("{page}쪽 {kind} `{tag}` 없음"))
}

fn pages_of(items: &[Extent], kind: &str, tag: &str) -> Vec<usize> {
    let mut pages: Vec<usize> = items
        .iter()
        .filter(|e| e.kind == kind && e.rest.contains(tag))
        .map(|e| e.page)
        .collect();
    pages.dedup();
    pages
}

#[test]
fn square_and_tac_tables_of_one_paragraph_share_the_side_lane() {
    let (pages, items) = extents();
    assert_eq!(pages, 6, "한컴 Print PDF 는 6쪽");

    let square = find(&items, 3, "Table", "pi=8 ci=0 ");
    let title = find(&items, 3, "Table", "pi=8 ci=1 ");
    // TAC 제목 표는 어울림 표 아래로 쌓이지 않고 그 옆 host 줄에 놓인다.
    assert!(
        title.top < square.bottom && title.bottom > square.top,
        "제목 표가 어울림 표와 세로로 겹쳐야 한다: title={title:?} square={square:?}"
    );
    assert!(
        title.right <= square.left,
        "제목 표는 어울림 표 왼쪽 차선에 있어야 한다"
    );

    let lane_head = find(&items, 3, "TextLine", "pi=9 line=0");
    let lane_next = find(&items, 3, "TextLine", "pi=10 line=0");
    assert!(
        lane_head.top >= title.bottom,
        "pi=9 는 host 줄 아래에서 시작한다"
    );
    assert!(
        lane_next.top >= lane_head.bottom - 0.5,
        "합성 줄 문단 pi=10 은 pi=9 와 겹치지 않고 뒤를 잇는다: {lane_head:?} {lane_next:?}"
    );
    for line in [lane_head, lane_next] {
        assert!(
            line.top < square.bottom,
            "차선 문단은 어울림 표 밴드 안에서 시작한다: {line:?}"
        );
    }
    let lane_text_right = items
        .iter()
        .filter(|e| {
            e.page == 3
                && e.kind == "TextRun"
                && (e.rest.contains("pi=9 ") || e.rest.contains("pi=10 "))
                && !e.rest.ends_with("\"\"")
        })
        .map(|e| e.right)
        .fold(f64::MIN, f64::max);
    assert!(
        lane_text_right <= square.left,
        "차선 문단 글자는 어울림 표 왼쪽에 있어야 한다"
    );

    let after_band = find(&items, 3, "TextLine", "pi=15 line=0");
    assert!(
        after_band.top >= square.bottom - 0.5,
        "밴드 뒤 문단은 어울림 표 아래에서 시작한다: {after_band:?} {square:?}"
    );
}

#[test]
fn synthetic_paragraph_after_synthetic_tac_row_keeps_hancom_page() {
    let (_, items) = extents();
    // 한컴 Print PDF 5쪽은 이 문단으로 시작한다(4쪽 표 뒤 줄간격 전량).
    assert_eq!(
        pages_of(&items, "TextLine", "pi=29 line=0 vpos=75195"),
        vec![5]
    );
    let first_body_line = items
        .iter()
        .filter(|e| e.page == 5 && e.kind == "TextLine" && e.rest.contains(" pi="))
        .min_by(|a, b| a.top.total_cmp(&b.top))
        .expect("5쪽 본문 줄");
    assert!(
        first_body_line.rest.contains("pi=29 line=0"),
        "5쪽 본문은 pi=29 로 시작한다: {first_body_line:?}"
    );
}

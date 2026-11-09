//! 관계형·고정형 — [#8859] 빈 host 양수 오프셋 표 위 틈에 저장 줄을 가진 뒤 문단을 둔다.
//! 주석 갱신: 2026-11-10 04:10 KST (새 파일, devel b5064fae1e 로 rebase 후 재측정)
//!
//! 입력: 공개 결재문서 36379333 원본. 기준: 한컴 engine 2020 Print PDF
//! `pdf/issue8859/36379333-stored-text-above-offset-float-2020.pdf`(2쪽).
//! 2쪽 `pi=11` 은 빈 문단에 앵커된 23×9 자리차지 표(문단 기준 오프셋 3629HU)다. 뒤 문단
//! `pi=12`(「□ 3교대 근무 현황」)의 저장 첫 줄은 host 저장 줄 끝(vpos 0 + 1400HU)에서 바로
//! 시작한다 — 표 위 틈이다. 한/글 PDF 도 제목을 쪽 맨 위(표 위)에 둔다.
//! - 고정형: 전체 쪽수 2.
//! - 관계형 2쪽: 제목 줄 위끝 = 본문 위끝 + host 저장 줄(1400HU), 제목 아래끝 ≤ 표 위끝.
//! - 관계형 2쪽: 다음 표(`pi=13`)는 앞 표(`pi=11`) 아래끝보다 위에서 시작하지 않는다.
//!
//! 측정(2026-11-10, 이 파일과 같은 커밋의 코드, base = upstream/devel b5064fae1e,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 2쪽 / 2쪽, p1 99.84 · p2 99.91, 최저 99.84 (base: p1 99.84 · p2 79.37)
//! - fresh WASM: 2쪽 / 2쪽, p1 99.80 · p2 99.91, 최저 99.80
//! - 증거: `mydocs/pr/assets/issue_8859/`(native·wasm·native_base TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8859/36379333-stored-text-above-offset-float.hwpx";

/// HWPUNIT → px(96 DPI).
fn px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 원본 저장값: 빈 host `pi=11` 의 저장 줄 높이(줄간격 0).
const HOST_ROW_HU: i32 = 1400;

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

fn find<'a>(items: &'a [Extent], page: usize, kind: &str, tag: &str) -> &'a Extent {
    items
        .iter()
        .find(|e| e.page == page && e.kind == kind && e.rest.contains(tag))
        .unwrap_or_else(|| panic!("{page}쪽 {kind} `{tag}` 없음"))
}

#[test]
fn stored_text_fills_the_gap_above_the_offset_float() {
    let (pages, items) = extents();
    assert_eq!(pages, 2, "한컴 Print PDF 는 2쪽");

    let body = find(&items, 2, "Body", "");
    let title = find(&items, 2, "TextLine", "pi=12 line=0 ");
    let table = find(&items, 2, "Table", "pi=11 ci=0 ");
    let expected = body.top + px(HOST_ROW_HU);
    assert!(
        (title.top - expected).abs() <= 0.5,
        "제목 위끝 {:.1} 은 본문 위끝 {:.1} + host 줄 = {:.1}",
        title.top,
        body.top,
        expected
    );
    assert!(
        title.bottom <= table.top + 0.5,
        "제목 아래끝 {:.1} 은 표 위끝 {:.1} 위",
        title.bottom,
        table.top
    );
}

#[test]
fn flow_after_the_gap_text_resumes_below_the_offset_float() {
    let (_, items) = extents();
    let float = find(&items, 2, "Table", "pi=11 ci=0 ");
    let next = find(&items, 2, "Table", "pi=13 ci=0 ");
    assert!(
        next.top + 0.5 >= float.bottom,
        "다음 표 위끝 {:.1} 은 앞 표 아래끝 {:.1} 보다 위에서 시작하지 않는다",
        next.top,
        float.bottom
    );
}

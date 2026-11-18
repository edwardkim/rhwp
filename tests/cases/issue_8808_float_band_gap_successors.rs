//! 관계형·고정형 — [#8808] 문단 기준 자리차지 표 위 빈 공간의 후속 문단.
//! 주석 갱신: 2026-11-10 07:30 KST (최신 devel 재측정, 저장 띠 아래끝 계약 검사 추가)
//!
//! 입력: 공개 공문서 36341965 의 축소본(1쪽 결재 표 문단과 일정 표 뒤 문단 삭제, 구역 정의는
//! 둘째 문단으로 옮김). 보안 마스킹 문단에 저장 줄이 없어 로드 시 구역 vpos 재계산이 돈다.
//! 기준: 같은 축소본의 한컴 engine 2020 Print PDF
//! `pdf/issue8808/36341965-float-band-gap-reduced-2020.pdf`(2쪽).
//! - 고정형: 전체 쪽수 2.
//! - 관계형 1쪽: 표 host `pi=14`(입찰방법) 뒤 `pi=15`·`pi=16` 은 같은 쪽에서 host 아래·표
//!   (`pi=14 ci=0`, 문단 기준 세로 오프셋 7665HU) 위끝 위에 순서대로 놓인다.
//! - 관계형 2쪽: 표 이어진 조각 뒤 `pi=17`·`pi=18` 과 일정 표(`pi=19`)가 같은 쪽 본문 안에 놓인다.
//!
//! 측정(2026-11-10, head 84b6d1b2a5, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 문서 face 만 공급한 글꼴 디렉터리):
//! - Native: 2쪽 / 2쪽, p1 99.82 · p2 99.98, 최저 99.82 (base 는 3쪽으로 쪽수 불일치)
//! - fresh WASM: 2쪽 / 2쪽, p1 99.10 · p2 99.96, 최저 99.10
//! - 증거: `mydocs/pr/assets/issue_8808/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8808/36341965-float-band-gap-reduced.hwpx";

/// 단(`Column`) 바로 아래 본문 항목의 들여쓰기. 표 칸 안 문단(같은 `pi=` 번호)과 가른다.
const BODY_DEPTH: usize = 6;

/// 쪽 본문 아래끝(위 여백 3600HU + 본문 높이, 96dpi).
const BODY_BOTTOM_PX: f64 = 96.0 + 930.5;

#[derive(Debug, Clone)]
struct Extent {
    page: usize,
    depth: usize,
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
    let depth = line.len() - trimmed.len();
    let kind = trimmed.split_whitespace().next()?.to_string();
    let y_start = trimmed.find("y=")? + 2;
    let (top, after_top) = trimmed[y_start..].trim_start().split_once("..")?;
    let bottom = after_top.split_whitespace().next()?;
    Some(Extent {
        page,
        depth,
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

/// 본문 깊이의 `kind` 항목 중 `tag` 로 시작하는 꼬리표를 가진 것(쪽 순).
fn body_items<'a>(items: &'a [Extent], kind: &str, tag: &str) -> Vec<&'a Extent> {
    items
        .iter()
        .filter(|e| {
            e.kind == kind
                && e.depth == BODY_DEPTH
                && e.rest.split_whitespace().any(|word| word == tag)
        })
        .collect()
}

fn text_line<'a>(items: &'a [Extent], para: usize) -> &'a Extent {
    let tag = format!("pi={para}");
    let lines = body_items(items, "TextLine", &tag);
    assert_eq!(lines.len(), 1, "{tag} 본문 줄은 한 줄");
    lines[0]
}

fn table_fragments<'a>(items: &'a [Extent], para: usize) -> Vec<&'a Extent> {
    body_items(items, "Table", &format!("pi={para}"))
}

#[test]
fn successors_fill_the_gap_above_the_paragraph_float_table() {
    let (pages, items) = extents();
    assert_eq!(pages, 2, "한컴 Print PDF 는 2쪽");

    let host = text_line(&items, 14);
    let first = text_line(&items, 15);
    let second = text_line(&items, 16);
    let fragments = table_fragments(&items, 14);
    assert_eq!(fragments.len(), 2, "pi=14 표는 두 조각");
    let table = fragments[0];

    for line in [host, first, second] {
        assert_eq!(line.page, 1, "{} 는 표 첫 조각과 같은 1쪽", line.rest);
    }
    assert_eq!(table.page, 1, "표 첫 조각은 1쪽");
    assert!(
        host.bottom <= first.top && first.bottom <= second.top,
        "host → pi=15 → pi=16 순서: {host:?} {first:?} {second:?}"
    );
    assert!(
        second.bottom <= table.top,
        "후속 문단은 표 위끝 위에 들어간다: {second:?} / {table:?}"
    );
}

#[test]
fn continuation_page_keeps_the_schedule_table() {
    let (_, items) = extents();
    let fragments = table_fragments(&items, 14);
    let continuation = fragments[1];
    assert_eq!(continuation.page, 2, "표 이어진 조각은 2쪽");

    let blank = text_line(&items, 17);
    let heading = text_line(&items, 18);
    let schedule = table_fragments(&items, 19);
    assert_eq!(schedule.len(), 1, "일정 표는 나뉘지 않는다");
    let schedule = schedule[0];

    for item in [blank, heading, schedule] {
        assert_eq!(item.page, 2, "{} 는 2쪽", item.rest);
    }
    assert!(
        continuation.bottom <= blank.top
            && blank.bottom <= heading.top
            && heading.bottom <= schedule.top,
        "이어진 조각 → pi=17 → pi=18 → 일정 표 순서"
    );
    assert!(
        schedule.bottom <= BODY_BOTTOM_PX + 0.5,
        "일정 표는 2쪽 본문 안: {schedule:?}"
    );
}

/// 앞 문단(저장 줄) → 저장 줄 없는 표 host → 저장 줄 후속 문단을 HWPX 로 저장했다가 다시 연다.
/// 표는 문단 기준으로 13666HU 아래에 놓인 자리차지 표이고, 띠 아래끝은 host 첫 줄 + 29682HU 다.
/// 반환값은 로드 시 구역 vpos 재계산을 거친 (host vpos, 후속 문단 vpos).
fn reflowed_float_band_successor_vpos(successor_vpos: i32) -> (i32, i32) {
    use rhwp::model::control::Control;
    use rhwp::model::paragraph::{LineSeg, Paragraph};
    use rhwp::model::shape::{TextWrap, VertRelTo};
    use rhwp::model::table::{Cell, Table};
    use rhwp::parser::hwpx::parse_hwpx;
    use rhwp::serializer::hwpx::serialize_hwpx;

    let stored_line = |vertical_pos| LineSeg {
        vertical_pos,
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 850,
        segment_width: 40_000,
        ..Default::default()
    };
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            width: 40_000,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }],
        outer_margin_top: 140,
        outer_margin_bottom: 140,
        ..Default::default()
    };
    table.common.width = 40_000;
    table.common.height = 15_736;
    table.common.vertical_offset = 13_666;
    table.common.vert_rel_to = VertRelTo::Para;
    table.common.text_wrap = TextWrap::TopAndBottom;

    let text = "host".to_string();
    let host = Paragraph {
        char_offsets: (0..4).collect(),
        char_count: 5,
        text,
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };
    let previous = Paragraph {
        line_segs: vec![stored_line(0)],
        ..Default::default()
    };
    let successor = Paragraph {
        line_segs: vec![stored_line(successor_vpos)],
        ..Default::default()
    };

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let mut document = parse_hwpx(&bytes).expect("샘플 파싱");
    document.sections.truncate(1);
    document.sections[0].paragraphs = vec![previous, host, successor];
    let saved = serialize_hwpx(&document).expect("HWPX 저장");
    let core = rhwp::document_core::DocumentCore::from_bytes(&saved).expect("다시 열기");
    let paragraphs = &core.document().sections[0].paragraphs;
    (
        paragraphs[1].line_segs[0].vertical_pos,
        paragraphs[2].line_segs[0].vertical_pos,
    )
}

/// 한컴이 띠 아래에 저장한 후속 줄은 띠 위 틈에 들어갈 높이여도 띠 아래에 둔다.
/// 저장 관계의 근거: 결재문서 36383989 pi=8(저장 60767 = host 31085 + 오프셋 13666 +
/// 표 15736 + 바깥 여백 280, 한컴 Print 도 표 아래). 그 문서는 다른 원인으로 1쪽이 90% 미만이라
/// 입력으로 등록하지 못해 저장 관계만 옮겼다.
#[test]
fn float_band_successor_stored_below_band_stays_below() {
    const BAND_FROM_HOST: i32 = 13_666 + 15_736 + 280;
    let (host, successor) = reflowed_float_band_successor_vpos(1_000 + BAND_FROM_HOST);
    assert_eq!(successor, host + BAND_FROM_HOST);
}

/// 띠 위 틈에 저장된 후속 줄은 그 틈에 남는다.
#[test]
fn float_band_successor_stored_in_gap_stays_above_band() {
    const BAND_TOP_FROM_HOST: i32 = 13_666;
    let (host, successor) = reflowed_float_band_successor_vpos(3_000);
    assert!(
        successor + 1_000 <= host + BAND_TOP_FROM_HOST,
        "후속 줄 {successor} 은 host {host} 아래 띠 위끝 앞에 들어가야 한다"
    );
}

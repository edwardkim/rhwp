//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8522의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T20:32:33+09:00 (`6500b32ed8c558e03ccf60d8c079721f2e7945e3`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8507/field_fit.hwpx`; 독립 Print `pdf/issue8507/field_fit-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_fit_rb.hwpx`; 독립 Print `pdf/issue8507/field_fit_rb-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_over1414.hwpx`; 독립 Print `pdf/issue8507/field_over1414-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_over1414.hwpx`; 독립 Print `pdf/issue8507/field_over1414-2024.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_over19.hwpx`; 독립 Print `pdf/issue8507/field_over19-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_over19.hwpx`; 독립 Print `pdf/issue8507/field_over19-2024.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_over19_rb.hwpx`; 독립 Print `pdf/issue8507/field_over19_rb-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/field_over19-saved-2020.hwp`; 독립 Print `pdf/issue8507/field_over19-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/tac_adjacent-saved.hwpx`; 독립 Print `pdf/issue8507/tac_adjacent-saved-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8507/tac_adjacent_generator.hwpx`; 독립 Print `pdf/issue8507/tac_adjacent_generator-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8522_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03 15:35 KST (새 파일)
//!
//! 측정 근거(측정일 2026-11-03, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` — 색상 경계 보정 전 2px 내용 실루엣, 이진화는 스크립트 기본값,
//! 색상 경계 대조 픽셀 `silhouette_boundary_reconciled_pixels` 는 전부 0). 후보 source 는
//! `task/8507-tac-declared-above-rows` head(`upstream/devel` `7d06d5c706` 위 리베이스, devel 대비 src diff
//! sha256 `aef9dcec61d3d593`), Native = native-skia release(sha256 `d8f46a2973c720f0`), fresh WASM = 같은
//! source 의 `scripts/wasm-pack-locked.sh --target web`(`rhwp_bg.wasm` sha256 `165d20fcd8870227`).
//! 리베이스 전(source `cd294db40d67d9b5`, Native `2bbd342df64d31d3`, WASM `7332ae31e59643db`)과 TSV 가 바이트 동일.
//! 글꼴 `RHWP_FONT_PATH` = 한컴 `ttfs/hwp`·`ttfs/windows`. 각 원본은 1쪽, 기준 PDF 도 1쪽이다.
//!
//! | 원본 → 기준 Print | Native p1 | fresh WASM p1 |
//! |---|---|---|
//! | `tac_adjacent-saved.hwpx` → `tac_adjacent-saved-2020.pdf` | 100.00 | 100.00 |
//! | `tac_adjacent_generator.hwpx` → `tac_adjacent_generator-2020.pdf` | 100.00 | 100.00 |
//! | `field_fit.hwpx` → `field_fit-2020.pdf` | 100.00 | 100.00 |
//! | `field_fit_rb.hwpx` → `field_fit_rb-2020.pdf` | 100.00 | 100.00 |
//! | `field_over19.hwpx` → `field_over19-2020.pdf` | 100.00 | 100.00 |
//! | `field_over19.hwpx` → `field_over19-2024.pdf`(`--compat 2024`) | 100.00 | 100.00 |
//! | `field_over19_rb.hwpx` → `field_over19_rb-2020.pdf` | 100.00 | 100.00 |
//! | `field_over1414.hwpx` → `field_over1414-2020.pdf` | 100.00 | 100.00 |
//! | `field_over1414.hwpx` → `field_over1414-2024.pdf`(`--compat 2024`) | 100.00 | 100.00 |
//! | `field_over19-saved-2020.hwp` → `field_over19-2020.pdf` | 100.00 | 100.00 |
//!
//! 두 경로 모두 전쪽 최저 100.00%. 정식 증거: `mydocs/pr/assets/issue_8507/{native,wasm}/*_silhouette.tsv`,
//! 대표 review·overlay `mydocs/pr/assets/issue_8507/png/`. 기준 PDF 는 `pdf/issue8507/`(한/글 2020
//! 11.0.0.9136, 2024 13.0.0.3901 Print). 좌표는 고정하지 않고 아래 관계만 검사한다.
//!
//! #8507 줄 없는 글자처럼 취급 표 host 의 줄은 선언 높이가 아니라 그리는 표를 담는다.
//!
//! 입력(공개 합성, `samples/issue8507/`):
//! - `tac_adjacent.hwpx`: `gen_tac_adjacent.py` 출력. 3행 표(행 2512HU, 합 7536HU) A·B 가 붙어 있다.
//! - `tac_adjacent-saved.hwpx`: 위 파일을 한/글 2020 이 HWP(`tac_adjacent-2020.hwp`)를 거쳐 HWPX 로
//!   저장한 일관된 저장본(선언 = 행 합, 저장 줄·vpos 모두 한/글 계산).
//! - `tac_adjacent_generator.hwpx`: `derive_generator_ladder.py` 로 저장본에서 표 A 선언만 8160HU 로
//!   올리고, A host 의 `linesegarray` 를 지우고, 뒤 최상위 저장 vpos 를 +624HU 옮긴 파일. 결재문서
//!   생성기가 남기는 꼴이다(host 줄 없음, 뒤 사다리는 선언 + 바깥 여백 기준).
//!
//! 독립 기준은 두 파일의 한/글 2020 Print(`pdf/issue8507/`)다. 두 출력은 래스터가 같다 — 한/글은
//! 표 A 를 행 높이로 그리고 줄도 그만큼만 차지하며, 뒤 문단의 낡은 저장 vpos 를 따르지 않는다.
//! 수정 전 rhwp 는 줄 없는 host 의 줄을 `max(측정, 선언) + 바깥 여백` 으로 만들어 표 B 와 뒤
//! 문단을 624HU(8.3px) 아래에 두었다.
//!
//! 검사는 두 파일 사이의 관계다 — 쪽수, 표 A·B 의 그린 높이, A 바닥→B 위끝, B 바닥→뒤 글줄 간격이
//! 같고, A host 줄 높이도 같다. 절대 좌표는 고정하지 않는다.
//!
//! 둘째 검사(필드 시작 뒤로 넘기는 표): `field_*.hwpx` 는 `tac_adjacent_generator.hwpx` 에서
//! `widen_table_a.py`(표 A 바깥 폭을 줄 폭 42520HU 에 맞춤 / 19HU·1414HU 넘침),
//! `wrap_table_a_in_field.py`(표 A 앞에 누름틀 시작, 다음 문단 첫머리에 끝),
//! `table_a_rowbreak.py`(쪽나눔 CELL) 로 만든 공개 합성이다. 한/글 2020·2024 Print 는 누름틀
//! 시작 뒤 표가 남은 폭에 들지 않으면 표를 다음 줄로 넘기고 필드 줄을 글자 높이의 빈 줄로
//! 남긴다(폭이 맞거나 누름틀이 없으면 넘기지 않는다). `field_over19-saved-2020.hwp` 는 한/글
//! 2020 이 `field_over19.hwpx` 를 HWP 로 저장한 파일이며 그 저장 줄(빈 줄 + ts=8 표 줄)이
//! 독립 기준이다. 수정 전 rhwp 는 누름틀 시작을 줄 내용으로 보지 않아 표를 첫 줄에 두었다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn text_of(node: &RenderNode) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            text.push_str(&run.text);
        }
        stack.extend(n.children.iter().rev());
    }
    text
}

/// 본문의 최상위 표와 표 밖 글줄 — (종류, 글자, 위끝, 아래끝) 을 위치 순서로.
fn flow(node: &RenderNode, out: &mut Vec<(&'static str, String, f64, f64)>) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            out.push((
                "table",
                text_of(node),
                node.bbox.y,
                node.bbox.y + node.bbox.height,
            ));
            return;
        }
        RenderNodeType::TextLine(_) => {
            out.push((
                "line",
                text_of(node),
                node.bbox.y,
                node.bbox.y + node.bbox.height,
            ));
            return;
        }
        _ => {}
    }
    for child in &node.children {
        flow(child, out);
    }
}

struct Layout {
    pages: usize,
    items: Vec<(&'static str, String, f64, f64)>,
    host_line_height: i32,
}

fn layout(path: &str) -> Layout {
    layout_with(path, false)
}

fn layout_with(path: &str, hangul2024: bool) -> Layout {
    let mut core = open(path);
    core.set_hangul2024_compat(hangul2024);
    let tree = core.build_page_render_tree(0).expect("첫 쪽 배치");
    let mut items = Vec::new();
    flow(&tree.root, &mut items);
    let host = &core.document().sections[0].paragraphs[1];
    Layout {
        pages: core.page_count() as usize,
        items,
        host_line_height: host.line_segs.first().expect("표 A host 줄").line_height,
    }
}

fn find<'a>(
    items: &'a [(&'static str, String, f64, f64)],
    kind: &str,
    text: &str,
) -> &'a (&'static str, String, f64, f64) {
    items
        .iter()
        .find(|(k, t, _, _)| *k == kind && t.contains(text))
        .unwrap_or_else(|| panic!("{kind} {text:?} 없음: {items:?}"))
}

#[test]
fn tac_host_without_line_occupies_drawn_rows_not_declared_height() {
    let saved = layout("samples/issue8507/tac_adjacent-saved.hwpx");
    let generated = layout("samples/issue8507/tac_adjacent_generator.hwpx");
    assert_eq!(saved.pages, 1);
    assert_eq!(generated.pages, saved.pages);

    // 내용 보존·순서: 표 A, 표 B, 「B 뒤 첫 줄」, 「끝」.
    for l in [&saved, &generated] {
        let a = find(&l.items, "table", "A0가");
        let b = find(&l.items, "table", "B0가");
        let after = find(&l.items, "line", "B 뒤 첫 줄");
        let end = find(&l.items, "line", "끝");
        assert!(
            a.3 <= b.2 && b.3 <= after.2 && after.3 <= end.2,
            "{:?}",
            l.items
        );
    }

    let gap = |l: &Layout| {
        let a = find(&l.items, "table", "A0가");
        let b = find(&l.items, "table", "B0가");
        let after = find(&l.items, "line", "B 뒤 첫 줄");
        (a.3 - a.2, b.3 - b.2, b.2 - a.3, after.2 - b.3)
    };
    let (sa, sb, s_ab, s_after) = gap(&saved);
    let (ga, gb, g_ab, g_after) = gap(&generated);
    // 그린 표는 같은 행 높이다(선언 8160HU 로 늘지 않는다).
    assert!(
        (ga - sa).abs() < 0.5 && (gb - sb).abs() < 0.5,
        "표 높이 {ga}/{gb} vs {sa}/{sb}"
    );
    // 한/글은 두 파일을 같게 찍는다 — 표 A 뒤 간격이 선언 높이만큼 벌어지지 않는다.
    assert!(
        (g_ab - s_ab).abs() < 0.5,
        "A→B 간격 {g_ab} vs 저장본 {s_ab}"
    );
    assert!(
        (g_after - s_after).abs() < 0.5,
        "B→뒤 글줄 간격 {g_after} vs 저장본 {s_after}"
    );
    // 줄 없는 host 에 만든 줄도 한/글이 저장한 줄(그린 표 + 바깥 여백)과 같다.
    assert_eq!(generated.host_line_height, saved.host_line_height);
}

/// 표 A host 의 줄 — (text_start, line_height, line_spacing).
fn host_lines(path: &str) -> Vec<(u32, i32, i32)> {
    let core = open(path);
    core.document().sections[0].paragraphs[1]
        .line_segs
        .iter()
        .map(|seg| (seg.text_start, seg.line_height, seg.line_spacing))
        .collect()
}

/// 표 A·B 위끝과 「B 뒤 첫 줄」 위끝.
fn anchors(path: &str) -> (f64, f64, f64) {
    anchors_with(path, false)
}

fn anchors_with(path: &str, hangul2024: bool) -> (f64, f64, f64) {
    let l = layout_with(path, hangul2024);
    assert_eq!(l.pages, 1, "{path}");
    let a = find(&l.items, "table", "A0가");
    let b = find(&l.items, "table", "B0가");
    let after = find(&l.items, "line", "B 뒤 첫 줄");
    let end = find(&l.items, "line", "끝");
    assert!(
        a.3 <= b.2 && b.3 <= after.2 && after.3 <= end.2,
        "{path}: {:?}",
        l.items
    );
    (a.2, b.2, after.2)
}

#[test]
fn field_begin_pushes_overflowing_tac_table_to_next_line() {
    // 한/글 2020 이 field_over19 를 HWP 로 저장한 줄: 필드 시작만 남은 빈 줄 + 표 줄(ts=8).
    let stored = host_lines("samples/issue8507/field_over19-saved-2020.hwp");
    assert_eq!(stored.len(), 2, "{stored:?}");
    assert_eq!(stored[1].0, 8, "{stored:?}");

    // 줄 없는 생성본도 같은 줄 구성으로 재조판한다(RowBreak 여부와 무관).
    for path in [
        "samples/issue8507/field_over19.hwpx",
        "samples/issue8507/field_over19_rb.hwpx",
    ] {
        assert_eq!(host_lines(path), stored, "{path}");
    }
    // 표가 줄에 들면 필드가 있어도 한 줄이다.
    assert_eq!(host_lines("samples/issue8507/field_fit.hwpx").len(), 1);

    // 그린 위치는 한/글 저장본을 rhwp 가 그린 위치와 같다.
    let saved = anchors("samples/issue8507/field_over19-saved-2020.hwp");
    let fit = anchors("samples/issue8507/field_fit.hwpx");
    for path in [
        "samples/issue8507/field_over19.hwpx",
        "samples/issue8507/field_over19_rb.hwpx",
        "samples/issue8507/field_over1414.hwpx",
    ] {
        let (a, b, after) = anchors(path);
        assert!(
            (a - saved.0).abs() < 0.5 && (b - saved.1).abs() < 0.5 && (after - saved.2).abs() < 0.5,
            "{path}: {:?} vs 저장본 {saved:?}",
            (a, b, after)
        );
        // 필드 줄만큼(저장 줄 간 전진) 내려가고, 표 사이 간격은 폭이 맞는 경우와 같다.
        let step = rhwp::renderer::hwpunit_to_px(stored[0].1 + stored[0].2, 96.0);
        assert!(
            (a - fit.0 - step).abs() < 0.5,
            "{path}: A {a} fit {} step {step}",
            fit.0
        );
        assert!(((b - a) - (fit.1 - fit.0)).abs() < 0.5, "{path}: A→B");
    }

    // 한/글 2024 Print 도 2020 과 래스터가 같다(field_over19·field_over1414). 2024 세대의 앞
    // 앵커 줄 회수가 필드 줄을 거두지 않는다.
    for path in [
        "samples/issue8507/field_over19.hwpx",
        "samples/issue8507/field_over1414.hwpx",
    ] {
        let (a, b, after) = anchors_with(path, true);
        assert!(
            (a - saved.0).abs() < 0.5 && (b - saved.1).abs() < 0.5 && (after - saved.2).abs() < 0.5,
            "{path} (2024): {:?} vs 저장본 {saved:?}",
            (a, b, after)
        );
    }
}

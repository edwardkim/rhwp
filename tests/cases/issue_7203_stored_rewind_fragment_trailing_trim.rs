//! [Issue #7203] 자리차지 표의 조각 컷 — 저장 사다리 되감김 경계의 줄간격 계상.
//!
//! 쪼개지는 빈-host 자리차지 표의 첫 조각이 셀 줄을 하나 잃었다. 컷 예산이 마지막 줄의
//! **줄간격**까지 요구했기 때문이다. 조각 상자는 그 줄의 줄 높이에서 끝나고 뒤따르는
//! 줄간격은 다음 쪽의 것이므로, 그 항은 쪽 예산에 들어가지 않는다.
//!
//! # 기대값의 출처 — 문서 자신과 정본이 같은 말을 한다
//!
//! `samples/hwpctl_API_v2.4.hwp` 문단 1274 의 1×1 RowBreak 칸은 셀 줄 사다리가
//! `0 · 1600 · 3200` 뒤 `0` 으로 되감긴다(`p[2].ls[1]`). 즉 저장본이 **세 줄 뒤**를
//! 물리 쪽 경계로 지목한다. 표의 선언 높이는 `4482 HU` 이고, 그 값은 세 줄을 줄간격
//! 트림과 함께 담은 상자와 정확히 같다.
//!
//! ```text
//!   pad 141 + 1600 + 1600 + lh 1000 + pad 141 = 4482 HU   (= 선언 높이)
//!   pad 141 + 1600 + 1600 + 1600    + pad 141 = 5082 HU   (트림 없이 요구한 값)
//! ```
//!
//! 한/글 engine 2020 정본 `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 52쪽(0-based 51)도 같다 —
//! 조각 상자 괘선 `940.73 ~ 1000.51`(= 59.78px = 4482 HU)이고 그 안에 세 줄
//! (빈 줄 + 코드 두 줄)이 들어간다.
//!
//! ```text
//!   정본 글자 줄   964.91 · 986.19        (빈 첫 줄은 글자가 없다)
//!   수정 전 rhwp   964.90 만              마지막 줄을 0.32px 예산 초과로 잃었다
//!   수정 후 rhwp   964.90 · 986.30
//! ```
//!
//! # 이 시험이 잠그는 것
//!
//! 1. 저장 되감김 경계의 마지막 줄이 첫 조각에 남는다.
//! 2. 그 줄이 이어받는 조각에서 **사라지거나 겹치지 않는다** — 유닛 보존.
//! 3. 조각이 커져도 본문 바닥을 넘지 않으며, **뒤따르는 표**(문단 1296)가 넘치지 않는다.
//!    수정 전에는 그 표가 본문 바닥을 10.05px 넘었다.
//! 4. 넉넉한 예산으로 이미 12유닛을 수용하던 문단 1342도 끝 줄간격을 상자에 더하지 않는다.
//!    PDF 55쪽 괘선은 731.04~982.61px(251.57px)이다. 컷/이어받기 내용은 불변이다.
//!
//! # 발화 조건 — 되감김만으로는 부족하다
//!
//! 문단 안 되감김은 물리 쪽 프레임일 수도, 공간이 남은 **로컬 재시작**일 수도 있다.
//! 되감김만 요구한 첫 구현은 쪽수 정답지 3개 파티션과 본문 넘침 래칫을 포함해 8건을
//! 깼다. 그래서 트림은 위 선언 높이 동일성(조각 상자 == `common.height`)을 함께
//! 요구한다 — 문서 자신이 그 되감김을 자기 첫 물리 조각의 경계로 확인해 준 경우만이다.
//!
//! 메인터너 보정: 첫 조각의 상자 높이도 저장 4482 HU와 독립 PDF 높이로 검사한다.
//! 컷에 적용한 끝 간격을 예약 높이/paint가 함께 소비해야 한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpctl_API_v2.4.hwp";

/// 본문 바닥 — `bodyArea` y=132.267 + height=876.880 (`dump-pages --json` 실측).
const BODY_BOTTOM_PX: f64 = 1009.147;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// `page_index`(0-based) 에서 문단 `para_index` 가 앵커인 표의 `(윗변, 높이)`.
fn table_box(core: &DocumentCore, page_index: u32, para_index: usize) -> (f64, f64) {
    let page = core
        .build_page_render_tree(page_index)
        .unwrap_or_else(|_| panic!("{page_index}쪽 render tree"));
    let mut found = Vec::new();
    fn walk(node: &RenderNode, para_index: usize, out: &mut Vec<(f64, f64)>) {
        if let RenderNodeType::Table(meta) = &node.node_type {
            if meta.para_index == Some(para_index) {
                out.push((node.bbox.y, node.bbox.height));
            }
        }
        for child in &node.children {
            walk(child, para_index, out);
        }
    }
    walk(&page.root, para_index, &mut found);
    *found
        .first()
        .unwrap_or_else(|| panic!("{page_index}쪽에서 문단 {para_index} 의 표를 찾지 못했다"))
}

/// `page_index` 에서 문단 `para_index` 표 **안**의 글자 줄 — `(y, 텍스트)`.
fn runs_in_table(core: &DocumentCore, page_index: u32, para_index: usize) -> Vec<(f64, String)> {
    let page = core
        .build_page_render_tree(page_index)
        .unwrap_or_else(|_| panic!("{page_index}쪽 render tree"));
    let mut out = Vec::new();
    fn walk(node: &RenderNode, want: usize, inside: bool, out: &mut Vec<(f64, String)>) {
        let inside = inside
            || matches!(&node.node_type, RenderNodeType::Table(meta)
                if meta.para_index == Some(want));
        if inside {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                if !run.text.trim().is_empty() {
                    out.push((node.bbox.y, run.text.clone()));
                }
            }
        }
        for child in &node.children {
            walk(child, want, inside, out);
        }
    }
    walk(&page.root, para_index, false, &mut out);
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// 같은 y 의 글자 조각을 한 줄로 묶는다 — `(y, 줄 전체 텍스트)`.
fn lines_in_table(core: &DocumentCore, page_index: u32, para_index: usize) -> Vec<(f64, String)> {
    let mut lines: Vec<(f64, String)> = Vec::new();
    for (y, text) in runs_in_table(core, page_index, para_index) {
        match lines.last_mut() {
            Some((last_y, line)) if (*last_y - y).abs() <= 0.5 => line.push_str(&text),
            _ => lines.push((y, text)),
        }
    }
    lines
}

/// 저장 되감김이 지목한 세 번째 줄은 첫 조각에 남는다.
#[test]
fn stored_rewind_keeps_its_last_line_in_the_fragment() {
    let core = core();
    let runs = runs_in_table(&core, 51, 1274);
    let hit = runs
        .iter()
        .find(|(_, text)| text.contains("MousePosSet"))
        .unwrap_or_else(|| {
            panic!(
                "52쪽(0-based 51) 조각에 저장 되감김 직전 줄이 없다. 실제 줄: {:?}",
                runs.iter()
                    .map(|(y, t)| (*y, t.as_str()))
                    .collect::<Vec<_>>()
            )
        });
    // 정본 986.19 (engine 2020). 글리프 상자 대 베이스라인 차이만 허용한다.
    let delta = hit.0 - 986.19;
    assert!(
        delta.abs() <= 1.5,
        "저장 되감김 직전 줄 y={:.2} — 정본 986.19 대비 {delta:+.2}px",
        hit.0,
    );
}

/// 첫 조각이 얻은 유닛을 이어받는 조각이 **잃지도 겹치지도** 않는다.
#[test]
fn the_continuation_neither_repeats_nor_drops_the_moved_unit() {
    let core = core();
    let (_, partial_spacing_h) = table_box(&core, 11, 176);
    assert!(
        (partial_spacing_h - 7879.0 / 75.0).abs() < 0.5,
        "12쪽 조각: PDF 105.01px / 저장 7879 HU, 실제 {partial_spacing_h}"
    );
    let (_, first_h) = table_box(&core, 51, 1274);
    let (_, tail_h) = table_box(&core, 52, 1274);
    // 유닛 하나(1600 HU = 21.33px)가 이어받는 조각에서 첫 조각으로 옮겨진다.
    assert!(
        (first_h - 4482.0 / 75.0).abs() <= 0.6,
        "첫 조각 높이 {first_h:.2} — 세 줄 조각은 저장된 4482 HU(59.76px)와 같아야 한다",
    );
    assert!(
        (tail_h - 314.73).abs() <= 0.6,
        "이어받는 조각 높이 {tail_h:.2} — 유닛 하나만큼 줄어야 한다 (수정 전 336.03)",
    );
    // 이어받는 조각의 첫 줄은 컷 **바로 다음** 유닛이어야 한다. 옮겨진 줄
    // (`var MousePosSet = pHwpCtrl.GetMousePos(0, 0);`)이 여기 다시 나오면 중복이고,
    // 그다음 줄이 나오면 유닛이 사라진 것이다.
    let tail_lines = lines_in_table(&core, 52, 1274);
    let first_tail = tail_lines
        .first()
        .map(|(_, line)| line.trim().to_owned())
        .unwrap_or_default();
    assert!(
        first_tail.starts_with("var xrelto"),
        "이어받는 조각의 첫 줄이 {first_tail:?} 다 — 컷 다음 유닛(`var xrelto = ...`)이어야 한다",
    );
}

/// 조각이 커져도 본문 바닥을 넘지 않고, **뒤따르는 표**도 넘치지 않는다.
#[test]
fn neither_the_fragment_nor_the_following_table_overflows_the_body() {
    let core = core();
    let (top, height) = table_box(&core, 51, 1274);
    assert!(
        top + height <= BODY_BOTTOM_PX + 0.5,
        "첫 조각 하단 {:.2} 가 본문 바닥 {BODY_BOTTOM_PX:.2} 를 넘었다",
        top + height,
    );
    // 수정 전 문단 1296 의 표는 938.13 + 81.09 = 1019.22 로 본문을 10.05px 넘었다.
    let (following_top, following_height) = table_box(&core, 52, 1296);
    assert!(
        following_top + following_height <= BODY_BOTTOM_PX + 0.5,
        "뒤따르는 문단 1296 표 하단 {:.2} 가 본문 바닥 {BODY_BOTTOM_PX:.2} 를 넘었다 \
         (수정 전 1019.22)",
        following_top + following_height,
    );
}

/// 예산 실패 여부와 무관하게 같은 컷은 같은 높이를 예약하고 그린다 — 문단 1342.
#[test]
fn an_already_selected_boundary_uses_the_same_trimmed_box() {
    let core = core();
    let (first_top, first_h) = table_box(&core, 54, 1342);
    let (tail_top, tail_h) = table_box(&core, 55, 1342);
    assert!(
        (first_top - 731.80).abs() <= 0.6 && (first_h - (982.61 - 731.04)).abs() <= 0.6,
        "문단 1342 첫 조각 ({first_top:.2}, {first_h:.2}) — 독립 PDF 괘선은 (731.04, 251.57), 컷은 12유닛",
    );
    assert!(
        (tail_top - 136.00).abs() <= 0.6 && (tail_h - 145.30).abs() <= 0.6,
        "문단 1342 이어받는 조각 ({tail_top:.2}, {tail_h:.2}) — 불변 계약은 (136.00, 145.30)",
    );
}

/// 총쪽수는 정답지와 같다.
#[test]
fn the_page_count_matches_the_oracle() {
    assert_eq!(core().page_count(), 105, "한/글 정답지 105쪽과 같아야 한다",);
}

/// 마지막 조각 뒤 column.usedHeight는 paint한 끝점과 같아야 한다.
/// 두 조각은 각각 좁은 예산/넉넉한 예산 경로이며 다음 쪽 컷을 함께 잠근다.
#[test]
fn reserved_bottom_matches_painted_bottom_and_continuation_cuts() {
    let core = core();
    for (page, para, cut) in [(11u32, 176usize, 5usize), (51, 1274, 3), (54, 1342, 12)] {
        let pages = core.dump_page_items_json(Some(page));
        let info = &pages[0];
        let column = &info["columns"][0];
        let last = column["items"].as_array().unwrap().last().unwrap();
        assert_eq!(last["paraIndex"], para);
        assert_eq!(last["endCut"][0], cut);
        let reserved_bottom =
            info["bodyArea"]["y"].as_f64().unwrap() + column["usedHeight"].as_f64().unwrap();
        let (top, height) = table_box(&core, page, para);
        assert!(
            (reserved_bottom - top - height).abs() < 0.5,
            "문단 {para}: 예약 끝 {reserved_bottom:.3} != 실제 끝 {:.3}",
            top + height
        );
        let next = core.dump_page_items_json(Some(page + 1));
        let continuation = next[0]["columns"][0]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["paraIndex"] == para && item["kind"] == "partialTable")
            .unwrap();
        assert_eq!(
            continuation["startCut"][0], cut,
            "앞 조각 다음 유닛부터 이어받는다"
        );
    }
}

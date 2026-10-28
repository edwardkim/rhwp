//! [#7702] 한/글 2020 이전 제품이 저장한 줄이 공백 최소값(condense) 낱말 흡수 없이 끊긴 칸 문단.
//!
//! 한/글은 공백을 condense% 까지 줄여 이미 시작한 낱말의 글자를 앞 줄에 더 담는다(#7418).
//! 한/글 2014(9.1.1.3933)로 저장한 1342000 교육과정 지도는 그 흡수 없이 낱말 끝 글자만 다음
//! 줄로 보낸 저장 줄을 가진다. 정본 한/글 2020 Print
//! (`mydocs/pr/assets/issue7445/education-curriculum-map/current-source-hancom2020-print.pdf`)는
//! 그 꼬리를 앞 줄에 붙인다.
//!
//! ```text
//!   151쪽 구역 5 표 r=56 c=8 p[1]  저장 `…순서에 맞게 시연하` / `기`   정본 한 줄(629.6 → 963.4px)
//!   154쪽 구역 7 표 r=97 c=8       저장 `…관한 찬반 토` / `론`          정본 한 줄
//! ```
//!
//! 수정 전 rhwp 는 저장 줄을 그대로 써 행이 한 줄씩 높았고(151쪽 r=56 57.1px, 선언·정본 43.5/44px),
//! 이 편차가 쌓여 문서가 416쪽(정본 415쪽)이 됐다.
//!
//! 반례:
//! - 새 줄 나눔이 마지막 꼬리 말고 앞 줄 경계까지 옮기면 저장 줄을 따른다(114쪽 `[12융과04-02]`
//!   칸 `…정보|가`, 정본도 저장 줄 그대로).
//! - 다시 나눠도 꼬리가 앞 줄에 들어가지 않으면 두 줄 그대로다(1342000 25쪽 `…포함 논의 필` /
//!   `요`, 정본도 두 줄). 이 문단은 condense 가 없지만 번호 문단이라 한/글이 다시 나누는
//!   대상이다(#8105) — 새 줄 나눔에서도 `요` 가 앞 줄에 들지 않는다.
//! - 한/글 2020 으로 저장한 문서는 저장 줄을 따른다(`21_언어_기출_편집가능본.hwp` 11.0.0.2129
//!   1쪽 `…문항 수를 확인하` / `십시오.`, 정본 `pdf/21_언어_기출_편집가능본-hwp-2020.pdf` 도 두 줄).
//!   원본 HWP5 요약 정보(한/글 2010)를 그대로 실은 HWPX 도 `version.xml` 의 마지막 저장(11.0)을
//!   따른다(`36388711…청년오피스.hwpx` 1쪽 `…○ 과학` / `기술(150)`). 그 쪽은 정본 대조
//!   실루엣이 90% 미만(86.2%, 수정 전후 같음)이라 정식 검사로 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";
const SAMPLE_2020_SAVED: &str = "samples/21_언어_기출_편집가능본.hwp";

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

struct Run {
    x: f64,
    y: f64,
    right: f64,
    text: String,
}

fn collect_runs(node: &RenderNode, out: &mut Vec<Run>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(Run {
            x: node.bbox.x,
            y: node.bbox.y,
            right: node.bbox.x + node.bbox.width,
            text: run.text.clone(),
        });
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 같은 윗변에 놓인 글자 조각을 x 순으로 이어 한 글줄로 만든다. 칸 사이처럼 20px 넘게 떨어진
/// 조각은 다른 글줄이다. 공백은 지운다(양쪽 정렬 공백 폭은 비교 대상이 아니다).
fn lines_of(node: &RenderNode) -> Vec<String> {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs);
    runs.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut last: Option<(f64, f64)> = None;
    for run in &runs {
        let same_line =
            last.is_some_and(|(y, right)| (run.y - y).abs() < 0.5 && run.x - right < 20.0);
        if !same_line && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        current.extend(run.text.chars().filter(|c| !c.is_whitespace()));
        last = Some((run.y, run.right));
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// `needle` 을 담은 글줄이 있는 첫 쪽(0-기준)과 그 쪽의 글줄.
fn page_with(core: &DocumentCore, needle: &str) -> (u32, Vec<String>) {
    (0..core.page_count())
        .find_map(|page| {
            let tree = core.build_page_render_tree(page).expect("render page");
            let lines = lines_of(&tree.root);
            lines
                .iter()
                .any(|line| line.contains(needle))
                .then_some((page, lines))
        })
        .unwrap_or_else(|| panic!("`{needle}` 을 담은 쪽이 있어야 한다"))
}

/// `needle` 글자 조각을 품은 표 칸 노드.
fn cell_with<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TableCell(_))
        && lines_of(node).iter().any(|line| line.contains(needle))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| cell_with(child, needle))
}

/// 모델에서 `needle` 을 담은 칸의 선언 높이(px).
fn declared_cell_height_px(core: &DocumentCore, needle: &str) -> f64 {
    core.document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .flat_map(|table| &table.cells)
        .find(|cell| cell.paragraphs.iter().any(|p| p.text.contains(needle)))
        .map(|cell| cell.height as f64 * 96.0 / 7200.0)
        .unwrap_or_else(|| panic!("`{needle}` 칸이 모델에 있어야 한다"))
}

#[test]
fn pre_2020_stored_word_tail_joins_the_condensed_line() {
    let core = core(SAMPLE_1342000);
    for (joined, split_end) in [
        ("순서에맞게시연하기", "순서에맞게시연하"),
        ("이성친구문제에관한찬반토론", "관한찬반토"),
    ] {
        let (page, lines) = page_with(&core, &joined[..joined.len() - 3]);
        assert!(
            lines.iter().any(|line| line.contains(joined)),
            "{}쪽: 낱말 꼬리가 앞 줄에 붙어야 한다 — `{joined}`\n{lines:#?}",
            page + 1
        );
        assert!(
            !lines.iter().any(|line| line.ends_with(split_end)),
            "{}쪽: 저장 줄의 끊김 `{split_end}` 이 남았다",
            page + 1
        );
    }
}

#[test]
fn joined_tail_row_keeps_its_declared_height() {
    // 글줄을 다시 끊은 칸의 행 높이도 새 줄로 잰다 — 저장 줄 사다리(3줄)의 extent 로 키우지
    // 않는다. 정본 151쪽 r=56 은 44px 로 선언 43.5px 와 같다.
    let core = core(SAMPLE_1342000);
    let (page, _) = page_with(&core, "순서에맞게시연하기");
    let tree = core.build_page_render_tree(page).expect("render page");
    let cell = cell_with(&tree.root, "순서에맞게시연하기").expect("r=56 c=8 칸");
    let declared = declared_cell_height_px(&core, "순서에 맞게 시연하");
    assert!(
        cell.bbox.height <= declared + 1.0,
        "{}쪽 r=56: 칸 높이 {:.1}px, 선언 {declared:.1}px",
        page + 1,
        cell.bbox.height
    );
}

#[test]
fn fresh_break_that_moves_an_earlier_boundary_keeps_the_stored_rows() {
    // 114쪽 `[12융과04-02]` 칸(한/글 2014 저장 4줄, 마지막 줄 `다.`)은 새로 끊으면 앞 경계
    // `…정보|가` 까지 옮긴다. 정본 한/글 2020 은 저장 줄 넷을 모두 따른다
    // (`…전달기기에서 정보` / `가 다른 … 설명할 수 있` / `다.`).
    let core = core(SAMPLE_1342000);
    let (page, lines) = page_with(&core, "전달기기에서정보");
    assert!(
        lines.iter().any(|line| line.ends_with("전달기기에서정보")),
        "{}쪽: 저장 경계 `…정보|가` 를 따른다\n{lines:#?}",
        page + 1
    );
    assert!(
        lines.iter().any(|line| line.ends_with("과정을설명할수있")),
        "{}쪽: 저장 경계 `…있|다.` 를 따른다\n{lines:#?}",
        page + 1
    );
}

#[test]
fn recomposed_tail_that_does_not_fit_stays_on_the_next_line() {
    let core = core(SAMPLE_1342000);
    let (page, lines) = page_with(&core, "포함논의필");
    assert!(
        lines.iter().any(|line| line.ends_with("포함논의필")),
        "{}쪽: 새 줄 나눔에서도 꼬리 `요` 가 앞 줄에 들지 않는다\n{lines:#?}",
        page + 1
    );
}

#[test]
fn hancom_2020_saved_word_tail_keeps_the_stored_break() {
    let core = core(SAMPLE_2020_SAVED);
    let tree = core.build_page_render_tree(0).expect("render page");
    let lines = lines_of(&tree.root);
    assert!(
        lines.iter().any(|line| line.ends_with("문항수를확인하")),
        "1쪽: 한/글 2020 저장 줄 `…확인하` / `십시오.` 를 따른다\n{lines:#?}"
    );
    assert!(
        lines.iter().any(|line| line.starts_with("십시오.")),
        "1쪽: `십시오.` 는 다음 줄에서 시작한다\n{lines:#?}"
    );
}

#[test]
fn stored_row_generation_comes_from_the_container_last_save() {
    // 저장 줄의 제품 세대는 컨테이너가 기록한 마지막 저장에서 읽는다. 렌더 결과가 아니라
    // 문서 메타데이터 판정이다.
    // - 1342000: HWP5 요약 9.1.1.3933(한/글 2014) → 이전 세대
    // - 21_언어: HWP5 요약 11.0.0.2129 → 아님
    // - 36388711 HWPX: 원본 HWP5 요약 8.0.0.466 을 싣고 있지만 version.xml 11.0.0.8227 → 아님
    for (sample, before_2020) in [
        (SAMPLE_1342000, true),
        (SAMPLE_2020_SAVED, false),
        (
            "samples/hwpx/opengov/36388711_사회보장제도 신설 협의요청서(청년오피스)_260624.hwpx",
            false,
        ),
    ] {
        assert_eq!(
            core(sample)
                .document()
                .provenance
                .stored_rows_before_hancom_2020,
            before_2020,
            "{sample}"
        );
    }
}

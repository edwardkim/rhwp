//! [#7416] 제 선언 폭과 **모순되는** 저장 줄 사다리는 믿지 않고 다시 조판한다.
//!
//! # 무엇이 깨져 있었나
//!
//! `samples/issue6639/rhwp-table-cell-minimal-repro.hwp` 의 칸 31 열 문단은 저장 사다리가
//!
//! ```text
//! horzsize = 39208   textpos = [0, 5, 47, 89, 131, …]   (연속 줄마다 42 자)
//! ```
//!
//! 이다. 그 문단은 9pt·장평 95%(11.40px/자)·내어쓰기 1000 이라 연속 줄의 선언 폭
//! 38208 HWPUNIT(509.44px)에 **44 자**가 들어간다. 42 자에서 끊은 줄은 어절 안(한글–한글)
//! 에서 끊겼고, 공백도 없고, 왼쪽 정렬인데도 두 글자 반(30.4px)을 남긴다 — «다음 글자가
//! 안 들어가서 끊었다» 는 줄 나눔의 유일한 이유와 모순이다. rhwp 는 이 사다리를 그대로
//! 믿어 칸이 28 줄이 됐고, 한/글은 25 줄이다.
//!
//! # 기대값의 출처 — 한/글
//!
//! - 같은 원본을 한/글이 다시 저장한 `issue6639-hancom-160.hwpx` 는 같은 `horzsize` 로
//!   연속 줄 44 자 사다리를 적었다. 그 줄별 `textpos` 가 이 검사의 기대값이다.
//! - 한/글은 원본의 사다리를 쓰지 않는다 — 원본 PDF 와, 사다리만 지운 입력의 PDF 가
//!   96dpi 래스터에서 화소 단위로 같다(`samples/issue6639/README.md`).
//!
//! # 반례 — 포화된 정상 사다리는 그대로 믿는다
//!
//! 한/글 저장본은 같은 선언 폭에서 0.7 자만 남겨(포화) 판별자에 걸리지 않는다. 저장
//! 사다리를 그대로 수용하므로 렌더 줄이 제 `textpos` 와 같아야 한다.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 칸 높이·괘선 위치는 여기서 잠그지 않는다(Visual Sweep 으로 확인). 판별자의 문턱(2 글자)이
//! 코퍼스 밖에서 유지되는지도 말하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const ORIGINAL: &str = "samples/issue6639/rhwp-table-cell-minimal-repro.hwp";
const HANCOM: &str = "samples/issue6639/issue6639-hancom-160.hwpx";
/// 대상 칸 — 열 문단이 `paraPr 18`(긴 한글 연속 + 내어쓰기)을 함께 쓴다.
const CELL: usize = 31;

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 칸 31 의 문단별 저장 줄 텍스트(공백 제거).
fn stored_lines(core: &DocumentCore) -> Vec<Vec<String>> {
    let Some(Control::Table(table)) = core.document().sections[0].paragraphs[0].controls.get(2)
    else {
        panic!("표를 찾지 못했다 — 시험 설정 오류");
    };
    table
        .cells
        .get(CELL)
        .expect("대상 칸")
        .paragraphs
        .iter()
        .map(|para| {
            let starts: Vec<u32> = para.line_segs.iter().map(|seg| seg.text_start).collect();
            let chars: Vec<char> = para.text.chars().collect();
            (0..starts.len())
                .map(|i| {
                    let lo = starts[i];
                    let hi = starts.get(i + 1).copied().unwrap_or(u32::MAX);
                    chars
                        .iter()
                        .enumerate()
                        .filter(|(ci, _)| {
                            let off = para.char_offsets.get(*ci).copied().unwrap_or(u32::MAX);
                            off >= lo && off < hi
                        })
                        .map(|(_, c)| *c)
                        .filter(|c| !c.is_whitespace())
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn collect_runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((node.bbox.y, node.bbox.x, run.text.clone()));
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 1 쪽에 그려진 줄(같은 baseline 의 run 을 모아 공백 제거).
fn rendered_lines(core: &DocumentCore) -> Vec<String> {
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut runs = Vec::new();
    collect_runs(&page.root, &mut runs);
    runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut lines: Vec<(f64, String)> = Vec::new();
    for (y, _x, text) in runs {
        match lines.last_mut() {
            Some((prev_y, buf)) if (*prev_y - y).abs() < 0.5 => buf.push_str(&text),
            _ => lines.push((y, text)),
        }
    }
    lines
        .into_iter()
        .map(|(_, text)| text.chars().filter(|c| !c.is_whitespace()).collect())
        .collect()
}

/// 한/글 사다리의 연속 줄(44 자)이 원본 렌더에서 한 줄씩 그대로 나온다.
#[test]
fn a_self_contradicting_ladder_is_reflowed_to_the_hancom_breaks() {
    let expected = stored_lines(&load(HANCOM));
    // 전제: 정답지에 어절 안에서 갈린 긴 줄이 실제로 있어야 이 검사가 무엇이든 잠근다.
    let long: Vec<&String> = expected
        .iter()
        .flatten()
        .filter(|line| line.chars().count() >= 40)
        .collect();
    assert!(
        long.len() >= 10,
        "정답지 전제가 깨졌다 — 40 자 이상인 한/글 줄이 {} 개뿐이다. 한/글={expected:?}",
        long.len()
    );

    let original = load(ORIGINAL);
    // 전제: 원본 사다리는 연속 줄 42 자다(이 검사가 겨누는 모순 사다리).
    let stored = stored_lines(&original);
    assert!(
        stored
            .iter()
            .flatten()
            .any(|line| line.chars().count() == 42),
        "원본 사다리 전제가 깨졌다 — 42 자 줄이 없다. 원본={stored:?}"
    );

    let rendered = rendered_lines(&original);
    let missing: Vec<&&String> = long
        .iter()
        .filter(|line| !rendered.iter().any(|r| r == **line))
        .collect();
    assert!(
        missing.is_empty(),
        "한/글 줄 {} 개 중 {} 개가 원본 렌더에서 한 줄로 안 나온다 — 제 선언 폭에 두 글자          이상을 남기고 어절 안에서 끊긴 사다리를 그대로 믿으면 줄이 42 자에서 갈린다.          빠진 줄={missing:?}",
        long.len(),
        missing.len()
    );
}

/// 사다리를 버려 줄 수가 줄었으면, 뒤 문단은 버린 사다리의 저장 vpos 가 아니라 다시 조판한
/// 앞 문단 바로 뒤에 놓인다 — 문단 사이에 빈 줄 띠가 생기지 않는다.
///
/// 문단 2 는 저장 4 줄 → 재조판 3 줄이다. 저장 vpos 를 앵커로 쓰면 `문단 3` 머리가 4 줄 자리에
/// 놓여 앞 줄과의 간격이 줄 피치의 두 배가 된다(한/글 PDF 는 한 피치).
#[test]
fn following_paragraphs_flow_after_the_reflowed_rows() {
    let lines = rendered_lines_with_y(&load(ORIGINAL));
    let long_gaps: Vec<f64> = lines
        .windows(2)
        .filter(|w| w[0].1.chars().count() >= 40 && w[1].1.chars().count() >= 40)
        .map(|w| w[1].0 - w[0].0)
        .collect();
    assert!(
        long_gaps.len() >= 5,
        "전제: 연속 긴 줄 쌍 {}개",
        long_gaps.len()
    );
    let pitch = long_gaps.iter().cloned().fold(f64::INFINITY, f64::min);
    let idx = lines
        .iter()
        .position(|(_, text)| text == "문단3")
        .expect("`문단 3` 머리 줄을 찾지 못했다 — 시험 설정 오류");
    assert!(idx > 0, "`문단 3` 앞 줄이 없다");
    let gap = lines[idx].0 - lines[idx - 1].0;
    assert!(
        gap <= pitch * 1.5,
        "`문단 3` 이 앞 줄보다 {gap:.1}px 아래(줄 피치 {pitch:.1}px)에 놓였다 — 버린 사다리의          저장 vpos 를 앵커로 쓰면 다시 조판해 줄어든 줄만큼 빈 띠가 생긴다."
    );
}

fn rendered_lines_with_y(core: &DocumentCore) -> Vec<(f64, String)> {
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut runs = Vec::new();
    collect_runs(&page.root, &mut runs);
    runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut lines: Vec<(f64, String)> = Vec::new();
    for (y, _x, text) in runs {
        match lines.last_mut() {
            Some((prev_y, buf)) if (*prev_y - y).abs() < 0.5 => buf.push_str(&text),
            _ => lines.push((y, text)),
        }
    }
    lines
        .into_iter()
        .map(|(y, text)| (y, text.chars().filter(|c| !c.is_whitespace()).collect()))
        .collect()
}

/// 반례: 같은 선언 폭에서 포화된 한/글 사다리는 그대로 수용된다.
#[test]
fn a_saturated_hancom_ladder_is_kept_as_stored() {
    let hancom = load(HANCOM);
    let stored = stored_lines(&hancom);
    let rendered = rendered_lines(&hancom);
    let all: Vec<&String> = stored.iter().flatten().filter(|l| !l.is_empty()).collect();
    assert!(all.len() >= 20, "정답지 전제가 깨졌다 — 줄 {}개", all.len());
    let missing: Vec<&&String> = all
        .iter()
        .filter(|line| !rendered.iter().any(|r| r == **line))
        .collect();
    assert!(
        missing.is_empty(),
        "포화된 한/글 사다리가 수용되지 않았다 — 빠진 줄={missing:?}"
    );
}

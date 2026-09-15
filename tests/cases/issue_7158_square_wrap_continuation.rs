//! [Issue #7158] 어울림 표 옆에서 시작해 같은 쪽 표 아래로 이어지는 문단의 나머지
//! 줄이 표 높이를 한 번 더 타고 떨어진다.
//!
//! `156492236` 9쪽 실측 — `pi=81` 의 `ls[0]` 은 표 옆 레인에 정확히 놓이는데
//! (`y=396.4`, 저장 `396.5`) `ls[1]` 이 `521.9` 로 **+95.5** 떨어진다. 그 뒤 내용이
//! 전부 같은 만큼 밀려 여러 쪽이 본문 바닥을 넘고 쪽번호와 겹친다.
//!
//! # 원인
//!
//! 줄 전진은 정상이다(계측: `flow_step` 이 모든 줄에서 20.0, `ls[1]→ls[2]→ls[3]` 이
//! 정확히 30px). 어긋남은 **항목 시작 y** 에 있다 — 흐름 커서가 어울림 표
//! (`PageItem::Table pi=76`)를 지나며 표 높이 `+375.7` 을 태워 `y_offset` 이
//! `146.2 → 521.9` 가 되고, 이어지는 조각이 그 자리에서 시작한다.
//!
//! `#6778` 이 같은 형상(렌더에 host-줄-전진의 짝이 없어 표 높이를 통째로 탄다)을 이미
//! 다루지만 `next_is_lane` — 다음 항목 **전체**가 표 옆 레인일 때 — 만 되돌린다.
//! 이 문서는 후속이 `cs=0` 전폭이라 그 술어를 통과하지 못한다. 그 겹은 그대로 두고
//! (제거하면 이 문서 글자겹침이 4 → 64건이 된다고 같은 주석이 적고 있다), 성격이 다른
//! 축 — 앞줄만 옆에 놓이고 나머지가 표 아래로 이어지는 **같은 문단** — 만 연다.
//!
//! # 기대값의 출처
//!
//! 구현과 독립된 두 근거가 같은 자리를 가리킨다.
//!
//! 1. **저장 사다리** — 이 문단의 `LINE_SEG.vertical_pos` 는 균일하다
//!    (24064 → 26316 → 28568 → 30820, 2252HU = 30.03px 등간격). 줄 사이에
//!    95px 짜리 턱이 있을 근거가 저장본에 없다.
//! 2. **한/글 기준 PDF** — `pi=81 ls[1]` 을 `423` 에 놓는다(저장 환산 `426.5`).
//!
//! 그래서 이 시험은 그려진 줄 간격이 **저장 간격을 따라가는지**로 잠근다. 절대 좌표가
//! 아니라 간격을 보므로 문단 시작 위치를 정하는 다른 축과 얽히지 않는다.
//!
//! # 잠그지 않는 것
//!
//! 이 문서에는 다른 원인의 넘침이 남아 있다(수정 뒤에도 `layout-anomaly` 넘침 36건).
//! 이 수정의 축이 아니라 여기서 주장하지 않는다 — 그래서 "본문 바닥을 넘지 않는다"가
//! 아니라 이 문단의 줄 간격만 잠근다. 전수 감소분은 `body_overflow_baseline` 래칫이
//! 따로 잠근다(이 문서 `23 → 0`).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue4090/156492236_규제샌드박스_min.hwpx";

/// 어울림 표 옆에서 시작해 표 아래로 이어지는 문단.
const WRAP_PARA: usize = 81;

/// HWPUNIT → px (렌더 트리와 같은 환산: 7200 HWPUNIT = 1인치, 96 DPI).
fn hu_to_px(hu: i32) -> f64 {
    hu as f64 / 7200.0 * 96.0
}

/// 한 줄의 (줄 서수, 그려진 윗변 y, 저장 vpos).
type Line = (u32, f64, i32);

fn collect_lines(node: &RenderNode, para: usize, out: &mut Vec<Line>) {
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if tl.para_index == Some(para) {
            if let (Some(idx), Some(vpos)) = (tl.line_index, tl.vpos) {
                out.push((idx, node.bbox.y, vpos));
            }
        }
    }
    for c in &node.children {
        collect_lines(c, para, out);
    }
}

#[test]
fn square_wrap_continuation_tracks_the_stored_ladder() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = DocumentCore::from_bytes(&std::fs::read(path).expect("표본 읽기")).expect("파싱");

    // 이 문단이 그려진 쪽을 찾는다. 쪽 서수는 0-based 다.
    let mut lines: Vec<Line> = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("{page}쪽 렌더 실패: {e:?}"));
        collect_lines(&tree.root, WRAP_PARA, &mut lines);
    }
    assert!(
        lines.len() >= 4,
        "pi={WRAP_PARA} 의 줄을 찾지 못했다 ({}줄) — 표본이나 문단 서수가 바뀌었다면 \
         이 시험의 전제부터 다시 세워야 한다",
        lines.len()
    );
    lines.sort_by_key(|(idx, _, _)| *idx);

    // 이웃한 줄 사이에서 그려진 간격과 저장 간격을 맞댄다. 이 문단은 쪽을 넘지
    // 않으므로 모든 줄이 같은 좌표계 위에 있다.
    let mut offenders = Vec::new();
    for pair in lines.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if b.0 != a.0 + 1 {
            continue;
        }
        let drawn = b.1 - a.1;
        let stored = hu_to_px(b.2 - a.2);
        if (drawn - stored).abs() > 2.0 {
            offenders.push((a.0, b.0, drawn, stored));
        }
    }

    assert!(
        offenders.is_empty(),
        "표 옆에서 이어지는 문단의 줄 간격이 저장 사다리와 어긋난다 ({:?}) — \
         이어지는 조각이 표 전진(+375.7px)을 한 번 더 타면 ls[0]→ls[1] 이 \
         저장 30.0px 대신 125.5px 가 된다",
        offenders
            .iter()
            .map(|(a, b, d, s)| format!("줄 {a}→{b}: 그림 {d:.1}px vs 저장 {s:.1}px"))
            .collect::<Vec<_>>()
    );
}

fn rendered_lines(doc: &DocumentCore, para: usize) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut owning_pages = 0;
    for page in 0..doc.page_count() {
        let before = lines.len();
        collect_lines(
            &doc.build_page_render_tree(page).expect("렌더").root,
            para,
            &mut lines,
        );
        owning_pages += usize::from(lines.len() > before);
    }
    assert_eq!(owning_pages, 1, "이 계약의 문단은 한 쪽에 있어야 한다");
    lines.sort_by_key(|line| line.0);
    lines
}

fn assert_complete_ladder(doc: &DocumentCore, para: usize) {
    let lines = rendered_lines(doc, para);
    let expected = &doc.document().sections[0].paragraphs[para].line_segs;
    assert_eq!(lines.len(), expected.len(), "pi={para} 줄 누락·중복");
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(line.0 as usize, index, "pi={para} 줄 소유");
    }
    for pair in lines.windows(2) {
        let drawn = pair[1].1 - pair[0].1;
        let stored = hu_to_px(pair[1].2 - pair[0].2);
        assert!(
            (drawn - stored).abs() < 0.15,
            "pi={para}: drawn={drawn}, stored={stored}"
        );
    }
}

#[test]
fn continuation_and_following_paragraph_preserve_moved_anchor() {
    use rhwp::model::control::Control;
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let source = DocumentCore::from_bytes(&bytes).unwrap();
    let original = rendered_lines(&source, WRAP_PARA);
    let following = rendered_lines(&source, WRAP_PARA + 1);
    for margin_delta in [750_i16, 1500] {
        let mut document = source.document().clone();
        let Control::Table(table) = &mut document.sections[0].paragraphs[75].controls[0] else {
            panic!("선행 제목 표");
        };
        table.outer_margin_bottom += margin_delta;
        let mut moved = DocumentCore::new_empty();
        moved.set_document(document);
        assert_complete_ladder(&moved, WRAP_PARA);
        for (before, after) in original.iter().zip(rendered_lines(&moved, WRAP_PARA)) {
            assert!((after.1 - before.1 - hu_to_px(margin_delta as i32)).abs() < 0.15);
        }
        let moved_following = rendered_lines(&moved, WRAP_PARA + 1);
        assert_eq!(following.len(), moved_following.len());
        for (before, after) in following.iter().zip(moved_following) {
            assert!(
                (after.1 - before.1 - hu_to_px(margin_delta as i32)).abs() < 0.15,
                "후속 문단도 같은 흐름 이동을 보존해야 한다"
            );
        }
    }
}

#[test]
fn all_same_page_wrap_handoffs_keep_every_line_and_its_pitch() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let doc = DocumentCore::from_bytes(&bytes).unwrap();
    for para in [45, 81, 96, 104, 111, 117, 125, 139, 156, 162, 171, 186, 197] {
        assert_complete_ladder(&doc, para);
    }
}

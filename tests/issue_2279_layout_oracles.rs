//! Issue #2279 (PR #2284) — 측정 정합 4수정의 직접 회귀 oracle.
//!
//! 페이지 수 pin(issue_1891)만으로는 같은 쪽수 안에서 되돌아가는 회귀를 잡지 못하므로
//! (maintainer 리뷰 P1), 각 수정의 관측 가능한 페이지-내 배치를 render tree 로 고정한다.
//! 입력: `samples/86712_regulatory_analysis.hwp`. #7195에서 메인테이너가 수용한
//! 한컴과의 표 분할 차이는 절대 쪽수 대신 내용 소유권과 실제 경계로 보호한다.
//! 페이지 인덱스는 0-based (`build_page_render_tree(N)` = N+1쪽).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/86712_regulatory_analysis.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 86712_regulatory_analysis.hwp")
}

fn page_contains(core: &DocumentCore, page: u32, needle: &str) -> bool {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree p{page}: {e:?}"));
    find_text(&tree.root, needle)
}

/// SVG renderer와 같은 클리핑을 적용해, 실제로 쪽에 칠해지는 텍스트만 찾는다.
///
/// Render tree는 디버그·재조판 관찰을 위해 부모 `TableCell`의 clip 밖 자식도
/// 보존한다. 따라서 단순 재귀 검색은 셀 하단에서 잘린 내부 표를 "이 쪽에
/// 있다"고 오판할 수 있다. `SvgRenderer`가 여는 Body/TableCell/TextBox clip을
/// 여기에도 적용해야 PDF 대조용 페이지 oracle이 실화면을 판정한다.
fn page_contains_paintable_text(core: &DocumentCore, page: u32, needle: &str) -> bool {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree p{page}: {e:?}"));
    find_paintable_text(&tree.root, needle, None, true)
}

fn clipped_intersection(a: BoundingBox, b: BoundingBox) -> Option<BoundingBox> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > left && bottom > top).then(|| BoundingBox::new(left, top, right - left, bottom - top))
}

fn find_paintable_text(
    node: &RenderNode,
    needle: &str,
    inherited_clip: Option<BoundingBox>,
    inherited_visible: bool,
) -> bool {
    let visible = inherited_visible && node.visible;
    if !visible {
        return false;
    }

    let own_clip = match &node.node_type {
        RenderNodeType::Body {
            clip_rect: Some(clip),
        } => Some(*clip),
        RenderNodeType::TableCell(cell) if cell.clip => Some(node.bbox),
        RenderNodeType::TextBox => Some(node.bbox),
        _ => None,
    };
    let clip = match (inherited_clip, own_clip) {
        (Some(parent), Some(own)) => match clipped_intersection(parent, own) {
            Some(intersection) => Some(intersection),
            None => return false,
        },
        (Some(parent), None) => Some(parent),
        (None, Some(own)) => Some(own),
        (None, None) => None,
    };

    if let RenderNodeType::TextRun(run) = &node.node_type {
        let paints_inside_clip = clip.as_ref().is_none_or(|clip| clip.intersects(&node.bbox));
        if paints_inside_clip && run.text.contains(needle) {
            return true;
        }
    }

    node.children
        .iter()
        .any(|child| find_paintable_text(child, needle, clip, visible))
}

fn find_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| find_text(c, needle))
}

/// 표 셀에서 줄/글자모양 경계로 갈라진 TextRun을 source 순서로 이어 검증한다.
/// 같은 header가 두 run으로 저장될 수 있으므로 단일-run substring만으로 표 존재를
/// 판정하면 거짓 음성이 된다.
fn table_contains_text_sequence(node: &RenderNode, rows: u16, cols: u16, needle: &str) -> bool {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows && table.col_count == cols {
            let mut text = String::new();
            collect_subtree_text(node, &mut text);
            if text.contains(needle) {
                return true;
            }
        }
    }
    node.children
        .iter()
        .any(|child| table_contains_text_sequence(child, rows, cols, needle))
}

fn collect_subtree_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_subtree_text(child, out);
    }
}

fn collect_body_lines<'a>(node: &'a RenderNode, pi: usize, out: &mut Vec<&'a RenderNode>) {
    // 셀/글상자의 로컬 문단 번호를 본문 문단 번호와 혼동하지 않는다.
    if matches!(
        node.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.section_index == Some(0) && line.para_index == Some(pi))
    {
        out.push(node);
    }
    for c in &node.children {
        collect_body_lines(c, pi, out);
    }
}

/// #7195 작업지시자 시각 판정(2026-09-16):
/// 21쪽의 표를 한 페이지에 담는 rhwp 배치를 수용했다. 따라서 한컴 PDF의
/// 절대 p27~29를 강제하지 않는다. 과거 측정 폭/보상 오차 조사는
/// mydocs/working/task_m100_7195_stage2.md 및 Git 이력에 보존한다.
///
/// 기존 계약의 의미(산식 재등장 금지, 근거설명과 다음 쪽 내부 표의 소유,
/// 3×12 뒤 5×4 표 보존)는 상대 페이지와 실제 표시 영역으로 계속 검사한다.
/// 새 구현의 총 페이지 수를 기대값으로 복사한 계약이 아니다.
#[test]
fn issue_2279_nested_cell_units_preserve_relative_fragment_ownership() {
    let core = core();
    let pages_for = |needle: &str| {
        (0..core.page_count())
            .filter(|page| page_contains_paintable_text(&core, *page, needle))
            .collect::<Vec<_>>()
    };
    let formula = pages_for("2891017");
    let rationale = pages_for("편익 수혜자");
    let data = pages_for("88.2");
    assert_eq!(formula.len(), 1, "산식 행은 한 쪽에서만 표시: {formula:?}");
    assert_eq!(rationale.len(), 1, "근거설명 시작 중복/누락: {rationale:?}");
    assert_eq!(data.len(), 1, "3×12 표의 값 중복/누락: {data:?}");
    assert_eq!(
        formula[0] + 1,
        rationale[0],
        "산식 다음 조각에서 근거설명 시작"
    );
    assert_eq!(
        rationale[0] + 1,
        data[0],
        "내부 표는 근거설명 다음 조각에서 표시"
    );

    // 동일 쪽 내 재방출도 거부한다. 쉼표가 있는 원문 값 2,891,017과
    // 산식 전용 2891017은 의도적으로 구별한다.
    for (page, needle) in [
        (formula[0], "2891017"),
        (rationale[0], "편익 수혜자"),
        (data[0], "88.2"),
    ] {
        let tree = core
            .build_page_render_tree(page)
            .expect("render marker page");
        let mut text = String::new();
        collect_subtree_text(&tree.root, &mut text);
        assert_eq!(
            text.matches(needle).count(),
            1,
            "같은 쪽 source 중복: {needle}"
        );
    }

    let tree = core
        .build_page_render_tree(data[0])
        .expect("render nested tables");
    assert!(table_contains_text_sequence(&tree.root, 3, 12, "88.2"));
    assert!(table_contains_text_sequence(
        &tree.root,
        5,
        4,
        "주민대표단 구성"
    ));
    let mut boxes = Vec::new();
    collect_nested_table_bounds(&tree.root, None, &mut boxes);
    assert_eq!(boxes.len(), 2, "3×12 및 5×4 표는 각각 한 번 온전히 표시");
    assert_eq!((boxes[0].0, boxes[0].1), (3, 12));
    assert_eq!((boxes[1].0, boxes[1].1), (5, 4));
    assert!(
        boxes[0].2.y + boxes[0].2.height <= boxes[1].2.y + 0.5,
        "3×12 표 다음에 5×4 표가 겹침 없이 배치"
    );
}

/// 관심 내부 표의 전체 외곽이 Body 및 부모 셀 clip 안에 있는지 검사한다.
/// 일부 글자만 clip과 교차해도 성공하는 substring oracle의 빈틈을 보완한다.
fn collect_nested_table_bounds(
    node: &RenderNode,
    parent_clip: Option<BoundingBox>,
    out: &mut Vec<(u16, u16, BoundingBox)>,
) {
    let own_clip = match &node.node_type {
        RenderNodeType::Body { .. } => Some(node.bbox),
        RenderNodeType::TableCell(cell) if cell.clip => Some(node.bbox),
        RenderNodeType::TextBox => Some(node.bbox),
        _ => None,
    };
    let clip = match (parent_clip, own_clip) {
        (Some(a), Some(b)) => Some(clipped_intersection(a, b).expect("nonempty parent clip")),
        (a, b) => a.or(b),
    };
    if let RenderNodeType::Table(table) = &node.node_type {
        if matches!((table.row_count, table.col_count), (3, 12) | (5, 4)) {
            let b = node.bbox;
            let c = clip.expect("nested table must have a body/owner");
            assert!(
                node.visible
                    && b.x >= c.x - 0.5
                    && b.y >= c.y - 0.5
                    && b.x + b.width <= c.x + c.width + 0.5
                    && b.y + b.height <= c.y + c.height + 0.5,
                "내부 표를 clip 아래에서 소비하지 않음: table={b:?}, clip={c:?}"
            );
            out.push((table.row_count, table.col_count, b));
        }
    }
    for child in &node.children {
        collect_nested_table_bounds(child, clip, out);
    }
}

/// #7195 작업지시자가 한컴의 분할 대신 수용한 21쪽 마지막 표.
/// source pi109의 표를 한 번에 배치하되, 모든 셀과 원문 및 본문 경계를 보존한다.
/// 절대21쪽이나 전체64쪽을 고정하지 않아 앞쪽의 유효한 조판 변화는 허용한다.
#[test]
fn issue_2279_fitting_cost_benefit_table_stays_whole() {
    assert_fitting_table_stays_whole(&core(), 109, (10, 8));
}

fn assert_fitting_table_stays_whole(core: &DocumentCore, pi: usize, shape: (u16, u16)) {
    use rhwp::model::control::Control;
    let Control::Table(source) = &core.document().sections[0].paragraphs[pi].controls[0] else {
        panic!("표 source pi={pi}");
    };
    assert_eq!((source.row_count, source.col_count), shape);
    let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let mut expected = source
        .cells
        .iter()
        .map(|c| {
            (
                c.row,
                c.col,
                c.row_span,
                c.col_span,
                c.paragraphs
                    .iter()
                    .map(|p| normalize(&p.text))
                    .collect::<String>(),
            )
        })
        .collect::<Vec<_>>();
    expected.sort();
    fn collect<'a>(node: &'a RenderNode, pi: usize, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(pi))
        {
            out.push(node);
        }
        if matches!(
            node.node_type,
            RenderNodeType::Table(_) | RenderNodeType::TextBox
        ) {
            return;
        }
        for child in &node.children {
            collect(child, pi, out);
        }
    }
    fn find_body(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return Some(node);
        }
        node.children.iter().find_map(find_body)
    }
    fn assert_cell_lines(node: &RenderNode, owner: BoundingBox) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            assert!(
                node.visible
                    && node.bbox.y >= owner.y - 0.5
                    && node.bbox.y + node.bbox.height <= owner.y + owner.height + 0.5,
                "글줄이 셀 밖에서 소비되면 안 됨: {:?}, owner={owner:?}",
                node.bbox
            );
        }
        for child in &node.children {
            assert_cell_lines(child, owner);
        }
    }
    let mut occurrences = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render");
        let mut tables = Vec::new();
        collect(&tree.root, pi, &mut tables);
        for table in tables {
            occurrences += 1;
            let b = find_body(&tree.root).expect("body").bbox;
            let t = table.bbox;
            assert!(
                table.visible && t.y >= b.y - 0.5 && t.y + t.height <= b.y + b.height + 0.5,
                "한 번에 배치한 표도 본문 안에 있어야 함: p{}, {t:?}, {b:?}",
                page + 1
            );
            let mut actual = Vec::new();
            for child in &table.children {
                if let RenderNodeType::TableCell(cell) = &child.node_type {
                    assert!(child.visible);
                    assert_cell_lines(child, child.bbox);
                    let mut content = String::new();
                    collect_subtree_text(child, &mut content);
                    actual.push((
                        cell.row,
                        cell.col,
                        cell.row_span,
                        cell.col_span,
                        normalize(&content),
                    ));
                }
            }
            actual.sort();
            assert_eq!(actual, expected, "전체 셀/원문을 한 조각에 보존");
        }
    }
    assert_eq!(
        occurrences, 1,
        "수용된 표를 불필요하게 분할하거나 중복/누락하지 않음"
    );
}
/// [수정 2] 본문 NO_LS 폴백의 글자모양 보존 + 전체-문단 재래핑 렌더
/// (재래핑 후 end_line 확장) —
/// 혼합 크기 문단(pi22: "ㅇ "=15pt + 본문 14pt)의 마지막 줄이 렌더에서 소실되지
/// 않아야 한다 (측정 4줄 fit vs 렌더 3줄 발산 회귀 검출).
#[test]
fn issue_2279_body_rewrap_keeps_paragraph_tail() {
    let core = core();
    assert!(
        page_contains(&core, 9, "규정하려는 것임"),
        "p10에 pi22 마지막 줄 부재 — 재래핑 줄수/end_line 클램프 회귀 (렌더 꼬리 소실)"
    );
}

/// 교체 입력의 저장 글줄 계약. 원본 15pt/14pt와 160%를 독립 기대값으로 사용한다.
/// TextRun의 x/분할 개수나 페이지 내 고정 창이 아니라 본문 문단의 인접 TextLine을 검사한다.
/// NO_LS 경로와 동일하다고 주장하지 않는다.
#[test]
fn issue_2279_per_line_pitch_uses_line_max_font_size() {
    let core = core();
    for pi in [20, 21, 22] {
        let source = &core.document().sections[0].paragraphs[pi];
        let style = &core.document().doc_info.para_shapes[source.para_shape_id as usize];
        assert_eq!(
            style.line_spacing_type,
            rhwp::model::style::LineSpacingType::Percent
        );
        assert_eq!(style.line_spacing, 160);
        assert!(source.line_segs.len() >= 3, "mixed-size source lines");
        let mut count = 0;
        let mut content = String::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).expect("render body");
            let mut lines = Vec::new();
            collect_body_lines(&tree.root, pi, &mut lines);
            for line in &lines {
                collect_subtree_text(line, &mut content);
            }
            // 페이지 경계는 pitch가 아니다. 동일 페이지의 동일 문단만 대조한다.
            for pair in lines.windows(2) {
                let RenderNodeType::TextLine(line) = &pair[0].node_type else {
                    unreachable!()
                };
                let index = line.line_index.expect("source line index") as usize;
                let seg = &source.line_segs[index];
                let size_hu = if index == 0 { 1500 } else { 1400 };
                assert_eq!(seg.line_height, size_hu);
                assert_eq!(seg.line_spacing, size_hu * 60 / 100);
                let expected = f64::from(size_hu) * 96.0 / 7200.0 * 1.6;
                let actual = pair[1].bbox.y - pair[0].bbox.y;
                assert!(
                    (actual - expected).abs() < 0.1,
                    "pi={pi} line={index} pitch {actual} != source {expected}"
                );
                count += 1;
            }
        }
        assert_eq!(
            count,
            source.line_segs.len() - 1,
            "all adjacent source lines checked"
        );
        let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        assert_eq!(
            normalize(&content),
            normalize(&source.text),
            "source exactly once"
        );
    }
}

/// 사용자 교체본의 pi30은 저장 LS가 있으며 전체 표가 본문에 들어간다.
/// 원래 NO_LS 회귀는 cases/issue_7195_no_ls_rowbreak_contract.rs의 합성 경계로 분리한다.
#[test]
fn issue_2279_saved_alternatives_table_preserves_all_cells() {
    let core = core();
    assert!(!core.document().sections[0].paragraphs[30]
        .line_segs
        .is_empty());
    assert_fitting_table_stays_whole(&core, 30, (4, 3));
}

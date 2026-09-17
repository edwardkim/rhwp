//! Issue #2279 (PR #2284) — 측정 정합 4수정의 직접 회귀 oracle.
//!
//! 페이지 수 pin(issue_1891)만으로는 같은 쪽수 안에서 되돌아가는 회귀를 잡지 못하므로
//! (maintainer 리뷰 P1), 각 수정의 관측 가능한 페이지-내 배치를 render tree 로 고정한다.
//! 기준 문서: `samples/86712_regulatory_analysis.hwp` (규제영향분석서, 한컴 2024 13.0.0.3901 = 64쪽).
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

    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && clip.as_ref().is_none_or(|clip| clip.intersects(&node.bbox))
    {
        let mut text = String::new();
        collect_subtree_text(node, &mut text);
        let normalized: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let expected: String = needle.chars().filter(|c| !c.is_whitespace()).collect();
        if normalized.contains(&expected) {
            return true;
        }
    }
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

fn collect_text_ys(node: &RenderNode, x_min: f64, y_range: (f64, f64), out: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty()
            && node.bbox.x >= x_min
            && node.bbox.y >= y_range.0
            && node.bbox.y <= y_range.1
        {
            out.push(node.bbox.y);
        }
    }
    for c in &node.children {
        collect_text_ys(c, x_min, y_range, out);
    }
}

/// 정상 한컴 2024 저장본과 직접 변환한 PDF(64쪽)의 실제 이어받기 계약.
/// p25의 근거설명은 p26 첫 두 줄로 이어지고, 편익 근거설명은 p27에서 시작한다.
/// 기존 65쪽 PDF/NO_LS 변형 입력의 p28·p29 핀은 이 원본의 oracle이 아니다.
#[test]
fn issue_2279_saved_nested_frame_keeps_source_page_ownership() {
    let core = core();
    assert_eq!(core.page_count(), 64, "정상 원본 한컴 2024 PDF 64쪽");
    assert!(page_contains_paintable_text(&core, 24, "비용부담자"));
    assert!(!page_contains_paintable_text(
        &core,
        24,
        "구성 승인 시점부터"
    ));
    assert!(
        page_contains_paintable_text(&core, 25, "구성 승인 시점부터"),
        "p26의 저장 이어받기 두 줄이 누락되거나 앞 쪽으로 잘못 소유됨"
    );
    assert!(!page_contains_paintable_text(&core, 25, "편익 수혜자"));
    assert!(page_contains_paintable_text(&core, 26, "편익 수혜자"));
    assert!(!page_contains_paintable_text(&core, 26, "88.2"));
    assert!(page_contains_paintable_text(&core, 27, "88.2"));
    assert!(table_contains_text_sequence(
        &core.build_page_render_tree(27).expect("p28").root,
        5,
        4,
        "주민대표단 구성"
    ));
}

/// 직접 한컴 PDF p26/p27에서 산식은 두 줄씩 나뉜다. 앞쪽에 두 줄이
/// 있는데도 orphan 보호가 끝줄을 되감아 1+3줄로 바꾸면 p27 표 머리까지 밀린다.
#[test]
fn issue_2279_saved_equation_split_keeps_two_lines_on_each_page() {
    for sample in [
        "samples/86712_regulatory_analysis.hwp",
        "samples/issue1891/86712_regulatory_analysis.hwpx",
    ] {
        let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).unwrap();
        let doc = DocumentCore::from_bytes(&bytes).unwrap();
        fn cell_text(node: &RenderNode, row: u16, col: u16) -> Option<String> {
            if let RenderNodeType::TableCell(cell) = &node.node_type {
                if cell.row == row && cell.col == col {
                    fn text(n: &RenderNode, out: &mut String) {
                        if let RenderNodeType::TextRun(run) = &n.node_type {
                            out.push_str(&run.text);
                        }
                        for child in &n.children {
                            text(child, out);
                        }
                    }
                    let mut out = String::new();
                    text(node, &mut out);
                    return Some(out);
                }
            }
            node.children.iter().find_map(|n| cell_text(n, row, col))
        }
        let page26 = doc.build_page_render_tree(25).unwrap();
        let text26 = cell_text(&page26.root, 26, 2).expect("2035 formula cell");
        assert!(
            text26.contains("2032년 기본형건축비)"),
            "{sample}: first two lines"
        );
        assert!(
            !text26.contains("주택면적"),
            "{sample}: third line belongs to p27: {text26}"
        );
        let page27 = doc.build_page_render_tree(26).unwrap();
        let text27 = cell_text(&page27.root, 26, 2).expect("continued 2035 formula cell");
        assert!(
            text27.contains("주택면적") && text27.contains("이자율"),
            "{sample}: last two lines: {text27}"
        );
    }
    let core = core();
    assert!(page_contains_paintable_text(
        &core,
        25,
        "2032년 기본형건축비)"
    ));
    assert!(!page_contains_paintable_text(
        &core,
        26,
        "2032년 기본형건축비)"
    ));
    assert!(table_contains_text_sequence(
        &core.build_page_render_tree(26).expect("p27").root,
        3,
        12,
        "구 분"
    ));
}

/// 직접 PDF p28의 정성 편익 설명 첫 조각과 p29의 이어받기.
/// 작은 선언 프레임의 저장 reset을 버리면 p28은 비고 p29에는 글자가 겹친다.
#[test]
fn issue_2279_small_saved_frame_preserves_qualitative_benefit_pages() {
    let core = core();
    assert!(page_contains_paintable_text(
        &core,
        27,
        "정비사업 후 조기 입주"
    ));
    assert!(page_contains_paintable_text(&core, 27, "빠른 입주에 따른"));
    assert!(!page_contains_paintable_text(
        &core,
        28,
        "정비사업 후 조기 입주"
    ));
    assert!(page_contains_paintable_text(
        &core,
        28,
        "노후계획도시는 조성 후"
    ));
}

/// 정상 저장본의 혼합 글자 크기 본문 문단 마지막 줄이 누락되지 않아야 한다.
/// 한컴 직접 PDF p10의 마지막 본문 줄과 대조한다. NO_LS 재조판 증거로 사용하지 않는다.
#[test]
fn issue_2279_saved_body_keeps_paragraph_tail() {
    let core = core();
    assert!(
        page_contains(&core, 9, "규정하려는 것임"),
        "p10에 pi22 마지막 줄 부재 — 재래핑 줄수/end_line 클램프 회귀 (렌더 꼬리 소실)"
    );
}

/// 정상 저장본 p10의 14pt 본문 줄 간격: 한컴 PDF 22.32~22.44pt
/// (96dpi 29.76~29.92px). 15pt 첫 줄의 24pt 간격과 구별한다.
#[test]
fn issue_2279_saved_body_preserves_line_pitch() {
    let core = core();
    let tree = core.build_page_render_tree(9).expect("render tree p10");
    // 정상 원본의 첫 본문 문단, 둘째 줄 이후. PDF 원점 y=146.70~258.66pt.
    let mut ys = Vec::new();
    collect_text_ys(&tree.root, 90.0, (190.0, 350.0), &mut ys);
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 3.0);
    let mut gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    // 문단 사이 간격(빈 줄 포함, > 34px)은 제외 — 줄 pitch 만.
    gaps.retain(|g| *g > 20.0 && *g < 34.0);
    assert!(
        gaps.len() == 5,
        "pitch 표본 부족 ({}개) — 페이지 구성 변화 시 창 조정 필요: {ys:?}",
        gaps.len()
    );
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = gaps[gaps.len() / 2];
    assert!(
        (29.76..=29.93).contains(&median),
        "본문 줄 pitch 중앙값 {median:.2}px — 정상 원본 PDF의 29.76~29.92px와 불일치"
    );
}

/// 정상 원본의 대안 표 전체가 p10에 위치한다. 이전 NO_LS 변형본의
/// p10/p11 분할 기대값을 폐기하고 직접 변환 PDF p10/p11로 교체했다.
#[test]
fn issue_2279_saved_alternatives_table_stays_on_page_ten() {
    let core = core();
    for text in ["대안명", "주민대표단의 법적", "준 준용", "단원선출"] {
        assert!(
            page_contains_paintable_text(&core, 9, text),
            "p10 대안 표 내용 누락: {text}"
        );
    }
    assert!(!page_contains_paintable_text(&core, 10, "준 준용"));
}

/// 정상 원본의 법령 비교표는 PDF p4~8·32~33의 본문 안에서 끝난다.
/// 저장 frame 끝의 줄 뒤 간격과 이어받기에서 소비한 빈 꼬리를 표 높이에
/// 다시 더하면 p6 표가 629px 늘고, 보이는 내용이 없는 셀도 페이지 밖으로 커진다.
#[test]
fn issue_2279_saved_comparison_table_fragments_stay_inside_body() {
    fn bounds(node: &RenderNode, body_bottom: &mut Option<f64>, table_bottom: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            *body_bottom = Some(node.bbox.y + node.bbox.height);
        }
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            *table_bottom = table_bottom.max(node.bbox.y + node.bbox.height);
        }
        for child in &node.children {
            bounds(child, body_bottom, table_bottom);
        }
    }
    for sample in [
        "samples/86712_regulatory_analysis.hwp",
        "samples/issue1891/86712_regulatory_analysis.hwpx",
    ] {
        let doc = DocumentCore::from_bytes(
            &fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).unwrap(),
        )
        .unwrap();
        for page in [3, 4, 5, 6, 7, 25, 31, 32] {
            let tree = doc.build_page_render_tree(page).unwrap();
            let (mut body_bottom, mut table_bottom) = (None, 0.0);
            bounds(&tree.root, &mut body_bottom, &mut table_bottom);
            assert!(table_bottom > 0.0, "comparison table must exist");
            assert!(
                table_bottom <= body_bottom.unwrap() + 0.5,
                "{sample} p{}: table bottom {table_bottom} exceeds body {:?}",
                page + 1,
                body_bottom,
            );
        }
    }
}

//! 원본 2022년 10월 입력의 문단·미주 소유, 같은 단의 순서와 점유 관계.
//! 실제 위치·수식 모양은 같은 원본의 한컴 Print PDF 전체 페이지로 별도 검증한다.
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

type ColumnFrame = (u16, BoundingBox);
type LocatedNode<'a> = (&'a RenderNode, Option<ColumnFrame>);

fn document() -> HwpDocument {
    HwpDocument::from_bytes(&std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("원본 입력"))
        .expect("원본 파싱")
}
fn visit<'a>(node: &'a RenderNode, column: Option<ColumnFrame>, out: &mut Vec<LocatedNode<'a>>) {
    let column = match node.node_type {
        RenderNodeType::Column(index) => Some((index, node.bbox)),
        _ => column,
    };
    out.push((node, column));
    for child in &node.children {
        visit(child, column, out);
    }
}
fn nodes(root: &RenderNode) -> Vec<LocatedNode<'_>> {
    let mut v = Vec::new();
    visit(root, None, &mut v);
    v
}
fn line(root: &RenderNode, para: usize, index: u32) -> (&RenderNode, (u16, BoundingBox)) {
    let v: Vec<_> = nodes(root)
        .into_iter()
        .filter(|(node, _)| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.para_index == Some(para) && line.line_index == Some(index))
        })
        .collect();
    assert_eq!(v.len(), 1, "문단 {para}의 줄 {index}이 한 번 표시되어야 함");
    (v[0].0, v[0].1.expect("실제 단 소속"))
}
fn equation(root: &RenderNode, para: usize) -> (&RenderNode, (u16, BoundingBox)) {
    let v: Vec<_> = nodes(root)
        .into_iter()
        .filter(|(node, _)| {
            matches!(&node.node_type, RenderNodeType::Equation(equation)
                if equation.para_index == Some(para) && equation.cell_index.is_none())
        })
        .collect();
    assert_eq!(v.len(), 1, "문단 {para}의 수식 소유가 유일해야 함");
    (v[0].0, v[0].1.expect("수식 단 소속"))
}
fn before(a: &RenderNode, b: &RenderNode) {
    assert!(
        a.bbox.y + a.bbox.height <= b.bbox.y,
        "앞 요소 점유 뒤에 다음 요소가 이어져야 함: {:?} → {:?}",
        a.bbox,
        b.bbox
    );
}
fn title(root: &RenderNode, number: u16) -> (&RenderNode, (u16, BoundingBox)) {
    let prefix = format!("문{number}");
    let v: Vec<_> = nodes(root)
        .into_iter()
        .filter(|(n, _)| {
            matches!(n.node_type, RenderNodeType::TextLine(_))
                && n.children
                    .iter()
                    .filter_map(|c| {
                        if let RenderNodeType::TextRun(t) = &c.node_type {
                            Some(t.text.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<String>()
                    .trim_start()
                    .starts_with(&prefix)
        })
        .collect();
    assert_eq!(v.len(), 1, "문항 {number} 제목이 해당 쪽에서 유일해야 함");
    (v[0].0, v[0].1.expect("제목 단 소속"))
}
fn note_lines<'a>(
    doc: &HwpDocument,
    root: &'a RenderNode,
    number: u16,
) -> Vec<(&'a RenderNode, (u16, BoundingBox))> {
    let mut bases = Vec::new();
    for (node, _) in nodes(root) {
        if let RenderNodeType::Equation(e) = &node.node_type {
            if let (Some(r), Some(pi)) = (&e.note_ref, e.para_index) {
                if r.kind == "endnote" && e.cell_index.is_none() {
                    if let Some(Control::Endnote(note)) = doc.document().sections[r.section_index]
                        .paragraphs[r.para_index]
                        .controls
                        .get(r.control_index)
                    {
                        if note.number == number {
                            let base = pi
                                .checked_sub(r.note_para_index)
                                .expect("미주 내부 문단 매핑");
                            let key = (
                                r.section_index,
                                r.para_index,
                                r.control_index,
                                base,
                                note.paragraphs.len(),
                            );
                            if !bases.contains(&key) {
                                bases.push(key);
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        bases.len(),
        1,
        "미주 {number}의 원본 소유와 가상 문단 매핑이 해당 쪽에서 유일해야 함"
    );
    let (section, _, _, base, len) = bases[0];
    let result: Vec<_> = nodes(root)
        .into_iter()
        .filter_map(|(n, col)| match &n.node_type {
            RenderNodeType::TextLine(l)
                if l.section_index == Some(section)
                    && l.para_index
                        .is_some_and(|pi| (base..base + len).contains(&pi)) =>
            {
                Some((n, col.expect("미주 본문 단 소속")))
            }
            _ => None,
        })
        .collect();
    assert!(!result.is_empty(), "미주 본문 줄이 실제로 검출되어야 함");
    result
}
#[test]
fn issue7642_2022_oct_page17_endnote_drag_selection_covers_equation_tail_lines() {
    let d = document();
    let t = d.build_page_render_tree(16).expect("17쪽");
    let rs: Value =
        serde_json::from_str(&d.get_selection_rects(0, 915, 0, 921, 3).expect("선택 영역"))
            .expect("선택 JSON");
    let rs = rs.as_array().expect("사각형 목록");
    for pi in 915..=921 {
        let (l, _) = line(&t.root, pi, 0);
        assert!(
            rs.iter().any(|r| r["pageIndex"].as_u64() == Some(16)
                && r["width"].as_f64().is_some_and(|w| w > 0.)
                && r["y"]
                    .as_f64()
                    .zip(r["height"].as_f64())
                    .is_some_and(|(y, h)| y <= l.bbox.y + l.bbox.height && y + h >= l.bbox.y)),
            "수식 꼬리 문단 {pi}의 줄이 선택 영역에 포함되어야 함"
        );
    }
}
#[test]
fn issue7642_2022_oct_page5_question28_choices_stay_below_condition_box() {
    let d = document();
    let t = d.build_page_render_tree(4).expect("5쪽");
    let boxes:Vec<_>=nodes(&t.root).into_iter().filter(|(n,_)|matches!(&n.node_type,RenderNodeType::Rectangle(r) if r.para_index==Some(306)&&r.control_index==Some(0))).collect();
    assert_eq!(boxes.len(), 1, "조건 박스 소유");
    let (choice, col) = line(&t.root, 306, 1);
    assert_eq!(boxes[0].1.expect("박스 단").0, col.0);
    before(boxes[0].0, choice);
}
#[test]
fn issue7642_2022_oct_page11_endnote_question_gaps_match_pdf() {
    let d = document();
    let t = d.build_page_render_tree(10).expect("11쪽");
    for (previous, next) in [(18, 19), (19, 20)] {
        let ls = note_lines(&d, &t.root, previous);
        let (tail, col) = ls
            .into_iter()
            .max_by(|a, b| {
                (a.0.bbox.y + a.0.bbox.height).total_cmp(&(b.0.bbox.y + b.0.bbox.height))
            })
            .expect("앞 미주 내용 끝");
        let (head, hcol) = title(&t.root, next);
        assert_eq!(col.0, hcol.0, "같은 단의 미주 경계");
        before(tail, head);
        assert!(
            head.bbox.y > tail.bbox.y + tail.bbox.height,
            "독립 PDF처럼 미주 사이 공간을 보존해야 함"
        );
    }
}
#[test]
fn issue7642_2022_oct_page11_question20_equation_tail_keeps_pdf_bleed() {
    let d = document();
    let t = d.build_page_render_tree(10).expect("11쪽");
    let ls = note_lines(&d, &t.root, 20);
    let (tail, col) = ls
        .into_iter()
        .max_by(|a, b| (a.0.bbox.y + a.0.bbox.height).total_cmp(&(b.0.bbox.y + b.0.bbox.height)))
        .expect("문20 끝");
    assert_eq!(col.0, 1, "문20 꼬리는 오른쪽 단에 있어야 함");
    assert!(
        tail.bbox.y + tail.bbox.height <= col.1.y + col.1.height,
        "마지막 줄의 점유가 실제 단 안에 있어야 함: {:?}, {:?}",
        tail.bbox,
        col.1
    );
}
#[test]
fn issue7642_2022_oct_page11_question20_formula_does_not_overlap_next_text() {
    let d = document();
    let t = d.build_page_render_tree(10).expect("11쪽");
    let (eq, col) = equation(&t.root, 584);
    let (next, ncol) = line(&t.root, 585, 0);
    assert_eq!(col.0, ncol.0);
    before(eq, next);
}
#[test]
fn issue7642_2022_oct_page15_question28_formula_does_not_overlap_case_label() {
    let d = document();
    let t = d.build_page_render_tree(14).expect("15쪽");
    let (eq, col) = equation(&t.root, 816);
    let (next, ncol) = line(&t.root, 817, 0);
    assert_eq!(col.0, ncol.0);
    let RenderNodeType::Equation(e) = &eq.node_type else {
        unreachable!()
    };
    let source = e.note_ref.as_ref().expect("수식의 원본 미주 소유");
    let label_sources: Vec<_> = next
        .children
        .iter()
        .filter_map(|n| {
            if let RenderNodeType::Equation(e) = &n.node_type {
                e.note_ref.as_ref()
            } else {
                None
            }
        })
        .collect();
    assert!(!label_sources.is_empty());
    assert!(
        label_sources
            .iter()
            .all(|r| r.section_index == source.section_index
                && r.para_index == source.para_index
                && r.control_index == source.control_index
                && r.note_para_index == source.note_para_index + 1),
        "같은 원본 미주의 수식 다음에 (ii) 본문이 이어져야 함"
    );
    before(eq, next);
}
#[test]
fn issue7642_2022_oct_page14_question25_tail_matches_pdf_frame() {
    let d = document();
    let t = d.build_page_render_tree(13).expect("14쪽");
    let (head, col) = title(&t.root, 25);
    assert_eq!(col.0, 1);
    for (l, lcol) in note_lines(&d, &t.root, 25) {
        assert_eq!(lcol.0, col.0);
        assert!(l.bbox.y >= head.bbox.y);
        assert!(
            l.bbox.y + l.bbox.height <= col.1.y + col.1.height,
            "문25 본문은 실제 단 안에 남아야 함"
        );
    }
}
#[test]
fn issue7642_2022_oct_page17_question29_tail_matches_pdf_frame() {
    let d = document();
    let t = d.build_page_render_tree(16).expect("17쪽");
    let (q28, c28) = title(&t.root, 28);
    let (q29, c29) = title(&t.root, 29);
    assert_eq!(c28.0, 1);
    assert_eq!(c28.0, c29.0);
    before(q28, q29);
    for (l, col) in note_lines(&d, &t.root, 29) {
        assert_eq!(col.0, c29.0);
        assert!(l.bbox.y >= q29.bbox.y);
        assert!(
            l.bbox.y + l.bbox.height <= col.1.y + col.1.height,
            "문29 꼬리는 실제 단 안에 있어야 함"
        );
    }
}
#[test]
fn issue7642_2022_oct_page16_question30_title_tail_continues_next_column() {
    let d = document();
    let t = d.build_page_render_tree(15).expect("16쪽");
    let (head, col) = title(&t.root, 30);
    let (first, fcol) = line(&t.root, 842, 0);
    assert_eq!(col.0, 0, "독립 Print의 왼쪽 단 소속");
    assert_eq!(col.0, fcol.0);
    before(head, first);
    assert!(
        first.bbox.y + first.bbox.height <= col.1.y + col.1.height,
        "첫 본문 줄도 제목 아래 같은 단에 남아야 함"
    );
}

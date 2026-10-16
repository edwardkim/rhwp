//! #4138: 부모 셀 분할 뒤 자식의 옛 물리 프레임과 현재 글줄 소유를 구분한다.
//! 원본115/분할197/현재197쪽의 Native/fresh WASM 전쪽90%와 직접 판독을 확인했다.
//! 두 분할본177쪽은 Native99.82990% / fresh WASM99.83114%다.
//! 증적: mydocs/working/task_m100_7445_rebase_repair_stage5.md C51 및
//! mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/child-frame-20261016/README.md.
//! 픽셀 기대값: tab-period/current-ascii PDF177쪽의 괘선·글자 원점(72pt→96CSSpx).
//! capture192DPI, 좌표96CSSpx, 허용±2CSSpx. 관계 검사도 함께 유지한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use std::{fs, path::Path};
const ORIGINAL: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const SAVED:[&str;2]=["tests/fixtures/issue4138/tab-period-split.hwp","mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp"];
fn load(path: &str) -> DocumentCore {
    let mut d = DocumentCore::from_bytes(
        &fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(true);
    d
}
fn body(d: &DocumentCore) -> &rhwp::model::table::Cell {
    let Control::Table(t) = &d.document().sections[0].paragraphs[0].controls[2] else {
        panic!("원문 표")
    };
    t.cells.iter().find(|c| c.row == 2 && c.col == 0).unwrap()
}
fn walk<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        walk(c, out)
    }
}
fn select(root: &RenderNode, p: impl Fn(&RenderNode) -> bool) -> &RenderNode {
    let mut all = Vec::new();
    walk(root, &mut all);
    let v = all.into_iter().filter(|n| p(n)).collect::<Vec<_>>();
    assert_eq!(v.len(), 1, "내용과 원본 소유로 선택한 대상의 단일 배치");
    v[0]
}
fn table(root: &RenderNode, pi: usize) -> &RenderNode {
    select(
        root,
        |n| matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(pi)&&t.control_index==Some(0)),
    )
}
fn line(root: &RenderNode, pi: usize) -> &RenderNode {
    select(
        root,
        |n| matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(pi)&&l.line_index==Some(0)),
    )
}
fn text(n: &RenderNode) -> String {
    let mut s = match &n.node_type {
        RenderNodeType::TextRun(r) => r.display_or_text().to_owned(),
        _ => String::new(),
    };
    for c in &n.children {
        s.push_str(&text(c))
    }
    s
}
fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{label}: actual={actual:.6}, independent Print={expected:.6}, tolerance2CSSpx"
    )
}
fn images(root: &RenderNode) -> Vec<&RenderNode> {
    let mut a = Vec::new();
    walk(root, &mut a);
    a.into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .collect()
}
#[test]
fn original_nested_table_closes_its_host_slot_once() {
    let d = load(ORIGINAL);
    assert_eq!(d.page_count(), 115);
    let b = body(&d);
    let host = &b.paragraphs[626];
    let Control::Table(t) = &host.controls[0] else {
        panic!("원문 중첩 표")
    };
    let slot = &host.line_segs[0];
    let min = f64::from(i32::from(t.outer_margin_bottom) + slot.line_height) * 96.0 / 7200.0;
    let max = f64::from(i32::from(t.outer_margin_bottom) + slot.line_height + slot.line_spacing)
        * 96.0
        / 7200.0;
    let tree = d.build_page_render_tree(28).unwrap();
    let t = table(&tree.root, 626);
    let next = line(&tree.root, 627);
    let gap = next.bbox.y - (t.bbox.y + t.bbox.height);
    assert!(
        gap >= min && gap <= max,
        "후속 빈 줄은 원본 호스트 한 줄과 아래 여백만큼 전진: gap={gap},source=[{min},{max}]"
    );
    let title = line(&tree.root, 628);
    assert!(title.bbox.y >= next.bbox.y + next.bbox.height);
    assert!(text(title).starts_with("5.1.8"));
}
#[test]
fn split_child_end_and_following_heading_match_completed_print_page() {
    for sample in SAVED {
        let d = load(sample);
        assert_eq!(d.page_count(), 197);
        let root = d.build_page_render_tree(176).unwrap().root;
        let t = table(&root, 2280);
        near(
            t.bbox.y + t.bbox.height,
            525.185343,
            "177쪽 자식 표의 마지막 괘선",
        );
        let heading = line(&root, 2282);
        assert_eq!(compact(&text(heading)), "11.2.8조종장치와조작장치의손잡이");
        let r = select(
            heading,
            |n| matches!(&n.node_type,RenderNodeType::TextRun(r)if r.display_or_text().trim_start().starts_with("11.2.8")),
        );
        let RenderNodeType::TextRun(run) = &r.node_type else {
            unreachable!()
        };
        near(r.bbox.x, 87.792419, "독립 Print의 제목 시작 x");
        near(
            r.bbox.y + run.baseline,
            566.101318,
            "독립 Print의 제목 baseline",
        );
        assert!(heading.bbox.y > t.bbox.y + t.bbox.height);
    }
}
#[test]
fn a_blank_prefix_line_does_not_paint_the_next_pages_picture() {
    for sample in SAVED {
        let d = load(sample);
        let b = body(&d);
        let host = &b.paragraphs[2286];
        assert!(host.text.chars().all(char::is_whitespace));
        assert_eq!(host.line_segs.len(), 2);
        let owner = host
            .line_segs
            .iter()
            .rposition(|l| l.text_start as usize <= host.control_text_positions()[0])
            .unwrap();
        assert_eq!(owner, 1, "그림은 첫 공백 줄이 아니라 둘째 저장 줄 소유");
        let p = d.build_page_render_tree(176).unwrap().root;
        let q = d.build_page_render_tree(177).unwrap().root;
        assert!(
            images(&p).is_empty(),
            "독립 Print177쪽은 그림을 표시하지 않음"
        );
        let imgs = images(&q);
        assert_eq!(
            imgs.len(),
            2,
            "독립 Print178쪽의 두 원문 그림을 한 번씩 보존"
        );
        assert!(
            imgs[0].bbox.y + imgs[0].bbox.height <= imgs[1].bbox.y,
            "원문 그림 순서와 비겹침"
        );
        let mut lines = Vec::new();
        walk(&p, &mut lines);
        let actual = lines
            .into_iter()
            .filter(
                |n| matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(2284)),
            )
            .map(text)
            .collect::<String>();
        assert_eq!(
            compact(&actual),
            compact(&b.paragraphs[2284].text),
            "앞 쪽 본문 내용은 누락·중복 없이 보존"
        );
    }
}

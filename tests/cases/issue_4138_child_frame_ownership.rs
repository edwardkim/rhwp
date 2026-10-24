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
fn parallel_picture_captions_close_before_the_following_source_line() {
    for sample in SAVED {
        let d = load(sample);
        let source = body(&d);
        let host = &source.paragraphs[310];
        let successor = &source.paragraphs[311];
        let source_band = host
            .controls
            .iter()
            .map(|control| {
                let Control::Picture(picture) = control else {
                    panic!("원본의 병렬 그림")
                };
                let caption = picture.caption.as_ref().expect("독립 Print의 아래 캡션");
                let caption_ink = caption
                    .paragraphs
                    .iter()
                    .flat_map(|p| &p.line_segs)
                    .map(|line| line.vertical_pos + line.line_height)
                    .max()
                    .unwrap();
                picture.common.vertical_offset as i64
                    + i64::from(picture.common.height)
                    + i64::from(picture.common.margin.top)
                    + i64::from(picture.common.margin.bottom)
                    + i64::from(caption.spacing)
                    + i64::from(caption_ink)
            })
            .max()
            .unwrap();
        assert!(
            i64::from(successor.line_segs[0].vertical_pos - host.line_segs[0].vertical_pos)
                >= source_band,
            "원본 저장 사다리도 그림·캡션의 물리 끝 뒤에서 다음 줄을 시작한다"
        );
        let root = d.build_page_render_tree(23).unwrap().root;
        let painted_host = line(&root, 310);
        let next = line(&root, 311);
        let body_start = line(&root, 312);
        assert!(next.bbox.y + 1e-6 >= painted_host.bbox.y + source_band as f64 * 96.0 / 7200.0,
            "측정에서 예약한 그림·아래 캡션의 끝을 실제 후속 줄이 침범하지 않음: sourceHU={source_band}, host={}, successor={}",painted_host.bbox.y,next.bbox.y);
        assert!(body_start.bbox.y >= next.bbox.y + next.bbox.height);
        assert!(compact(&text(body_start)).starts_with("3.4.6.3횡동요각도"));
    }
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

// C71: 원본 HWP/HWPX115쪽과 두 분할본197쪽의 양쪽 전쪽624쪽 검증 후 추가.
// 편집본7~10쪽은 관계로 보호하고, 원본 HWPX4쪽(양쪽99.98%대)은 독립 좌표를 함께 보호.
// 증적: stage5 C59 및 host-total-frame-20261016/README.md.
#[test]
fn split_parent_keeps_child_row_occupancy_and_the_following_blank_owner() {
    for sample in SAVED {
        let d = load(sample);
        assert_eq!(d.page_count(), 197);
        let source = body(&d);
        let Control::Table(child) = &source.paragraphs[81].controls[0] else {
            panic!("source child")
        };
        let p = d.build_page_render_tree(6).unwrap().root;
        let q = d.build_page_render_tree(7).unwrap().root;
        let t = table(&p, 81);
        let last = t
            .children
            .iter()
            .filter(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==2))
            .collect::<Vec<_>>();
        assert_eq!(last.len(), 3, "원본 마지막 행의 세 셀 소유 보존");
        for cell in last {
            let RenderNodeType::TableCell(c) = &cell.node_type else {
                unreachable!()
            };
            let declared = child
                .cells
                .iter()
                .find(|x| x.row == c.row && x.col == c.col)
                .unwrap();
            assert!(
                cell.bbox.height + 96.0 / 7200.0 >= f64::from(declared.height) * 96.0 / 7200.0,
                "분할 뒤 자식의 마지막 행 점유를 옛 총높이로 압축하지 않음: actual={}, sourceHU={}",
                cell.bbox.height,
                declared.height
            );
        }
        let mut a = Vec::new();
        walk(&p, &mut a);
        assert!(
            !a.iter().any(
                |n| matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(86))
            ),
            "다음 쪽의 빈 줄 소유를 앞 쪽에 소비하지 않음"
        );
        let blank = line(&q, 86);
        assert!(text(blank).trim().is_empty());
        let next = line(&q, 87);
        assert!(next.bbox.y >= blank.bbox.y + blank.bbox.height);
        let mut all = Vec::new();
        walk(&p, &mut all);
        let actual = all
            .into_iter()
            .filter(|n| matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(85)))
            .map(text)
            .collect::<String>();
        assert_eq!(
            compact(&actual),
            compact(&source.paragraphs[85].text),
            "앞 쪽 마지막 문단을 누락·중복 없이 보존"
        );
        let before_formula = d.build_page_render_tree(8).unwrap().root;
        let formula_page = d.build_page_render_tree(9).unwrap().root;
        let mut prev = Vec::new();
        walk(&before_formula, &mut prev);
        assert!(
            !prev.iter().any(
                |n| matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(110))
            ),
            "수식 앞 빈 줄은 독립 Print와 같은 다음 쪽 소유"
        );
        let blank = line(&formula_page, 110);
        let formula = line(&formula_page, 111);
        assert!(text(blank).trim().is_empty());
        assert!(
            formula.bbox.y >= blank.bbox.y + blank.bbox.height,
            "빈 줄의 점유 뒤에 수식 줄을 배치"
        );
    }
    for (sample, compat24) in [
        (ORIGINAL, true),
        (
            "samples/issue1949_giant_cell_nested_tables_perf.hwpx",
            false,
        ),
    ] {
        let mut d = DocumentCore::from_bytes(
            &fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).unwrap(),
        )
        .unwrap();
        d.set_hangul2024_compat(compat24);
        assert_eq!(d.page_count(), 115);
        let Control::Table(child) = &body(&d).paragraphs[81].controls[0] else {
            panic!("original child")
        };
        let p = d.build_page_render_tree(3).unwrap().root;
        let t = table(&p, 81);
        assert!(
            (t.bbox.height - f64::from(child.common.height) * 96.0 / 7200.0).abs()
                <= 2.0 * 96.0 / 7200.0,
            "편집하지 않은 호스트의 유효한 자식 총높이 프레임 유지"
        );
        for rendered in &t.children {
            let RenderNodeType::TableCell(c) = &rendered.node_type else {
                continue;
            };
            let original = child
                .cells
                .iter()
                .find(|cell| cell.row == c.row && cell.col == c.col)
                .unwrap();
            let expected = original
                .paragraphs
                .iter()
                .map(|para| para.text.as_str())
                .collect::<String>();
            assert_eq!(
                compact(&text(rendered)),
                compact(&expected),
                "원본 중첩 셀의 내용 순서·누락·중복 보호"
            );
        }
        if !compat24 {
            // 같은 child81의 편집 전 총높이 대조군. 다른 고득점 쪽은 이 표의 프레임을 검증하지 않는다.
            // 독립 한컴2020 Print4쪽, 72pt→96CSSpx. Native99.98878/WASM99.98877%.
            near(t.bbox.y, 969.658691, "원본 HWPX4쪽의 중첩 표 시작 괘선");
            near(
                t.bbox.y + t.bbox.height,
                1071.308024,
                "원본 HWPX4쪽의 중첩 표 마지막 괘선",
            );
        }
    }
}

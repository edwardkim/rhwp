//! 전체 피델리티 미달의 차단 검사만 #7445(comment5981655880)로 이관했습니다. 나머지 검사는 유지합니다.
//! [#6776] 칸 안 **줄이 0개인 문단의 글자처럼 취급(TAC) 그림**이 `cell_units`
//! 회계에서 통째로 빠져, 조각이 자기 프레임을 넘어 용지 밖까지 그린다.
//!
//! 단독 수정 당시 `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` 19쪽은 바깥
//! 7×2 표의 마지막 행 칸에 1×1 중첩 표를 담고, 그 자식 칸(33문단)의 `pi=12`·`pi=23`
//! 이 **글자 없는 문단에 TAC 그림 하나**씩만 담는다(312.4px · 725.4px).
//!
//! `cell_units` 는 그 문단들을 `line_count == 0` 가지에서 유닛화하면서 `Control::Table`
//! 만 세었다. 그림 1,037.8px 가 회계에서 빠지니 컷만 짧아지고, 페인트는 그림 높이만큼
//! 자리를 잡으므로 조각이 프레임을 넘었다.
//!
//! | 항목 | 수정 전 | 수정 후 | engine 2020 정본 |
//! | --- | --- | --- | --- |
//! | 19쪽 725.4px 그림 상단 `y` | 1,327.7 (용지 1,122.5 밖 930.5) | 21쪽 79.0 | — |
//! | 문서 전체 용지 밖 | 1 | **0** | — |
//! | 총 글자 | 27,216 | **27,312** | 27,308 |
//!
//! ⚠⚠ **줄이 있는 문단에서는 세지 않는다** — 그때 TAC 그림은 이미 그 줄 높이에 들어
//! 있어 이중 계상이 된다. 관문 없이 켜면 `table_giant_cell_overfill` 계열이 넘침
//! 3 → 15건·용지 밖 2 → 3건으로 무너진다.
//!
//! ⚠ 총 쪽수는 74 → 75 로 늘지만 이 축의 회귀가 아니다. 정본과 쪽 단위로 맞춰 보면
//! **수정 전에도 9쪽부터 이미 한 쪽 밀려 있었고**(rhwp 9쪽은 꼬리말만 있는 빈 쪽),
//! 그 +1 이 이 결함으로 잃던 −1 과 상계돼 총합만 74 로 맞았다. 빈 9쪽은 `devel` 의
//! 별개 결함이었다. #6854 통합 뒤에도 NO_LS 빈 문단과 TAC 표 뒤 간격을 잃으면
//! 그래프가 18쪽에 잘못 들어간다. 새 한컴 PDF와 대조한 18/19쪽 경계를 함께 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::{
    model::control::Control,
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::{RenderNode, RenderNodeType},
};

/// 양성 — 자식 칸이 TAC 그림만 담은 문단을 갖는다.
const SAMPLE: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
/// 음성 대조 — 칸 안 그림이 **어울림(비-TAC)** 이라 회계에 들어가면 안 된다.
const NEGATIVE: &str = "samples/issue6776/36367506-water-facility-approval.hwpx";

/// 이 문서가 조판하는 A4 세로 종이 높이(px, 96dpi).
const PAPER_HEIGHT_PX: f64 = 1122.5;
/// 자식 1×1 표의 첫 조각을 식별하는 작은 TAC 그림의 높이(px).
/// 회계에서 빠져 있던 큰 TAC 그림의 높이(px). 726,000HWPUNIT 은 아니고 저장 원본 값이다.
const BIG_PICTURE_H_PX: f64 = 725.4;

fn open(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}")))
        .unwrap_or_else(|e| panic!("open {sample}: {e}"))
}

/// SVG 의 `<image …>` 를 `(y, height)` 로 걷는다.
fn images(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<image ").skip(1) {
        let head = &chunk[..chunk.find('>').unwrap_or(chunk.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = head[s..].find('"')?;
            head[s..s + e].parse::<f64>().ok()
        };
        if let (Some(y), Some(h)) = (attr("y"), attr("height")) {
            out.push((y, h));
        }
    }
    out
}

fn page_svg(core: &DocumentCore, page: u32) -> String {
    core.render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 svg: {e}", page + 1))
}

#[test]
fn issue_6776_big_picture_is_placed_whole_on_exactly_one_page() {
    let core = open(SAMPLE);
    let page_count = u32::try_from(core.page_count()).expect("page count fits u32");
    let mut hits = Vec::new();
    for page in 0..page_count {
        for (y, h) in images(&page_svg(&core, page)) {
            if (h - BIG_PICTURE_H_PX).abs() <= 1.0 {
                hits.push((page, y, h));
            }
        }
    }
    assert_eq!(
        hits.len(),
        1,
        "{BIG_PICTURE_H_PX}px TAC 그림은 딱 한 쪽에 한 번만 그려져야 한다 \
         — 잃어도(0) 중복돼도(2+) 안 된다. 실측 {hits:?}"
    );
    let (page, y, h) = hits[0];
    assert!(
        y >= -0.5 && y + h <= PAPER_HEIGHT_PX + 0.5,
        "{}쪽 그림이 종이 안에 온전히 들어가야 한다 (y={y:.1} h={h:.1})",
        page + 1
    );
}

#[test]
fn issue_6776_negative_cell_wrapped_picture_is_not_charged() {
    // 이 문서의 칸 그림은 어울림(비-TAC)이라 `para_non_inline_h` 소관이다. 회계에
    // 넣으면 이중 계상돼 3쪽 배분이 무너지고 용지 밖 1건·넘침 1건이 생긴다.
    // engine 2020 정본도 3쪽이고 쪽별 글자 수 [112, 359, 326] 로 같다.
    let core = open(NEGATIVE);
    assert_eq!(
        core.page_count(),
        3,
        "음성 대조: 어울림 그림은 칸 회계에 들어가면 안 된다 — 들어가면 배분이 무너진다"
    );
    let page_count = u32::try_from(core.page_count()).expect("page count fits u32");
    for page in 0..page_count {
        for (y, h) in images(&page_svg(&core, page)) {
            assert!(
                y + h <= PAPER_HEIGHT_PX + 0.5,
                "음성 대조 {}쪽 그림이 종이 밖으로 나가면 안 된다 (y={y:.1} h={h:.1})",
                page + 1
            );
        }
    }
    assert_generated_tac_local_successor(&core);
}

// #7445 보정33: 같은 원본 전3쪽 Native/fresh ≥90%,3쪽 양쪽raw100%와 직접 판독.
// 독립 Print 기준선 ±2CSSpx 및 뒤 문단의 소속·순서·누락·중복을 기존 음성 대조에 보완한다.
fn nodes<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        nodes(c, out)
    }
}
fn ops<'a>(n: &'a LayerNode, out: &mut Vec<&'a PaintOp>) {
    match &n.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                ops(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            ops(child, out)
        }
        LayerNodeKind::Leaf { ops: o } => out.extend(o),
    }
}
fn compact(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{fffc}')
        .collect()
}
fn assert_generated_tac_local_successor(core: &DocumentCore) {
    let c = core;
    let (owner, t) = c.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(p, a)| {
            a.controls.iter().find_map(|v| match v {
                Control::Table(t)
                    if t.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|p| p.text.contains("구경산출"))
                    }) =>
                {
                    Some((p, t))
                }
                _ => None,
            })
        })
        .unwrap();
    let host = &c.document().sections[0].paragraphs[owner];
    let next = &c.document().sections[0].paragraphs[owner + 1];
    let line = &host.line_segs[0];
    let gap = (next.line_segs[0].vertical_pos - line.line_height) as f64 / 75.;
    assert_eq!(gap, line.line_spacing as f64 / 75.);
    let bottom_margin = f64::from(t.outer_margin_bottom) / 75.;
    let tree = c.build_page_render_tree(2).unwrap();
    let mut ns = vec![];
    nodes(&tree.root, &mut ns);
    let tables:Vec<_>=ns.iter().filter(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(owner)&&t.cell_context.is_none())).collect();
    let starts: Vec<_> = ns
        .iter()
        .filter(
            |n| matches!(&n.node_type,RenderNodeType::TextLine(t)if t.para_index==Some(owner+1)),
        )
        .collect();
    assert_eq!(tables.len(), 1);
    assert_eq!(starts.len(), 1);
    let actual = starts[0].bbox.y - tables[0].bbox.y - tables[0].bbox.height;
    assert!(
        (actual - bottom_margin - gap).abs() <= 1. / 75.,
        "원본 후속 줄이 닫은 표 바깥여백·후행 간격: {actual}/{bottom_margin}/{gap}"
    );
    for p in owner + 1..c.document().sections[0].paragraphs.len() {
        let expected = &c.document().sections[0].paragraphs[p];
        let mut texts = String::new();
        let mut pages = vec![];
        for page in 0..c.page_count() {
            let t = c.build_page_render_tree(page).unwrap();
            let mut ns = vec![];
            nodes(&t.root, &mut ns);
            let mut lines = 0;
            for n in ns {
                match &n.node_type {
                    RenderNodeType::TextRun(r)
                        if r.para_index == Some(p) && r.cell_context.is_none() =>
                    {
                        texts.push_str(&r.text)
                    }
                    RenderNodeType::TextLine(r) if r.para_index == Some(p) => lines += 1,
                    _ => {}
                }
            }
            if lines > 0 {
                pages.push((page, lines))
            }
        }
        assert_eq!(
            compact(&texts),
            compact(&expected.text),
            "후속 원문 문단{p} 순서·누락·중복"
        );
        assert_eq!(
            pages,
            vec![(2, expected.line_segs.len())],
            "뒤 원문의 쪽·줄 소속"
        );
    }
    let heading = c.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.text == "□ 조치의견")
        .unwrap();
    let painted = c
        .build_page_layer_tree_with_profile(2, RenderProfile::Print)
        .unwrap();
    let mut all = vec![];
    ops(&painted.root, &mut all);
    let hits:Vec<_>=all.into_iter().filter(|op|matches!(op,PaintOp::TextRun{run,..}if run.para_index==Some(heading)&&run.text=="조치의견")).collect();
    assert_eq!(hits.len(), 1);
    let op = hits[0];
    let PaintOp::TextRun { bbox, run, .. } = op else {
        unreachable!()
    };
    let y = bbox.y + run.baseline;
    let final_y = op
        .glyph_paint_transform()
        .map_or(y, |t| t.b * bbox.x + t.d * y + t.f);
    // 독립 한컴2020 Print3쪽 cm=[.12,0,0,-.12,0,841], tm.y=1979.
    assert!(
        (final_y - 1979. * 0.12 * 96. / 72.).abs() <= 2.,
        "후속 제목 실제 Print 기준선 ±2CSSpx: {final_y}"
    );
}

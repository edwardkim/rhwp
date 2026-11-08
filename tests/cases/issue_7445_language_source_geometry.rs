//! 언어 원본의 저장 줄 여백과 자동 표시 대체가 보존해야 할 원본 HFT 전진.
//! 동일 원본 전15쪽 Native/fresh WASM ≥90%,2·14쪽 각각 양쪽 ≥98%와 review/overlay 판독.
//! 독립 Print의 Type3 자연 전진 및 이어받기 원점 ±2CSSpx와 내용·줄 소유를 보호한다.
//! 실제 공급·전체 TSV·수정 전 FAIL/후 PASS 근거: #7445 보정32(stage32-20261021).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
use serde_json::json;
use std::collections::BTreeMap;
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
fn text(n: &RenderNode, owner: usize, out: &mut String, lines: &mut Vec<u32>) {
    match &n.node_type {
        RenderNodeType::TextRun(r) if r.para_index == Some(owner) => out.push_str(&r.text),
        RenderNodeType::TextLine(l) if l.para_index == Some(owner) => {
            lines.push(l.line_index.unwrap())
        }
        _ => {}
    }
    for c in &n.children {
        text(c, owner, out, lines)
    }
}
fn compact(t: &str) -> String {
    t.chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{fffc}')
        .collect()
}
fn automatic_environment(c: &DocumentCore) -> FontEnvironment {
    // 가용성 프로토콜만 모형화한다. 실제 글꼴 설치/공급 증거는 전쪽 시각 검증의 별도 producer다.
    // 다른 원본은 설치된 것으로 두고, 원본 한양신명조가 없는 경우의 문서 지정 바탕만 공급한다.
    let mut programs = BTreeMap::new();
    for request in FontEnvironment::document_requests(&c.document().doc_info) {
        if request.source == "한양신명조" {
            continue;
        }
        if let Some(query) = request.primary.first() {
            let id = format!("availability-control-{}", programs.len());
            programs.entry(query.clone()).or_insert_with(
                || json!({"query":query,"family":query,"programId":id,"trueType":true}),
            );
        }
    }
    programs.insert("바탕".into(),json!({"query":"바탕","family":"Batang","programId":"availability-control-declared-batang","trueType":true}));
    FontEnvironment::from_available_programs(
        &c.document().doc_info,
        &serde_json::to_string(&programs.into_values().collect::<Vec<_>>()).unwrap(),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn automatic_display_fallback_keeps_original_hft_bracket_advance_and_following_text() {
    let mut c = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/21_언어_기출_편집가능본.hwp"
        ))
        .unwrap(),
    )
    .unwrap();
    c.set_hangul2024_compat(false);
    let found: Vec<_> = c.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.text.starts_with("3. <보기>의 사례에"))
        .collect();
    assert_eq!(found.len(), 1);
    let owner = found[0].0;
    let original = found[0].1.text.clone();
    let following = original.find('의').unwrap();
    let offset = original[..following].encode_utf16().count();
    let angle_offset = original[..original.find('<').unwrap()]
        .encode_utf16()
        .count() as u32;
    let source_shape_id = found[0]
        .1
        .char_shapes
        .iter()
        .rev()
        .find(|s| s.start_pos <= angle_offset)
        .unwrap()
        .char_shape_id as usize;
    let shape = &c.document().doc_info.char_shapes[source_shape_id];
    let source_size_hu = f64::from(shape.base_size) * f64::from(shape.relative_sizes[1]) / 100.0;
    let selected = automatic_environment(&c);
    let bytes = c.export_hwp_native().unwrap();
    c.set_font_environment(Some(selected)).unwrap();
    assert_eq!(
        c.export_hwp_native().unwrap(),
        bytes,
        "표시 공급 선택은 원본/직렬화 정보를 바꾸지 않는다"
    );
    assert_eq!(c.page_count(), 15, "독립 Print 전15쪽");
    let trace: serde_json::Value = serde_json::from_str(
        &c.get_font_decision_trace_native(1, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let glyphs: Vec<_> = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            r["source"]["paragraphIndex"].as_u64() == Some(owner as u64)
                && r["source"]["charOffset"].as_u64() == Some(u64::from(angle_offset))
                && r["source"]["character"] == "<"
        })
        .collect();
    assert_eq!(glyphs.len(), 1);
    // 같은 독립 Print의 Type3 '<' /Widths=729, FontMatrix=.001.
    // 조판 표 조회 결과를 기대값으로 재사용하지 않고 원본 크기와 독립 전진을 대조한다.
    let advance = glyphs[0]["layoutMetric"]["baseAdvanceHwpunit"]
        .as_f64()
        .unwrap();
    assert!(
        (advance - source_size_hu * 729.0 / 1000.0).abs() <= 1.0,
        "독립 HFT 자연 전진과 원본 크기: actual={advance}, sourceSize={source_size_hu}"
    );
    let page = c
        .build_page_layer_tree_with_profile(1, RenderProfile::Print)
        .unwrap();
    let mut all = vec![];
    ops(&page.root, &mut all);
    let found:Vec<_>=all.into_iter().filter(|op|matches!(op,PaintOp::TextRun{run,..}if run.para_index==Some(owner)&&run.char_start==Some(offset)&&run.text=="의")).collect();
    assert_eq!(found.len(), 1, "괄호 뒤 원본 글자의 소유·누락·중복");
    let op = found[0];
    let PaintOp::TextRun { bbox, run, .. } = op else {
        unreachable!()
    };
    let x = op.glyph_paint_transform().map_or(bbox.x, |t| {
        t.a * bbox.x + t.c * (bbox.y + run.baseline) + t.e
    });
    // 독립 한컴2020 Print2쪽: cm[.12,0,0,-.12,0,1190], '의' tm.x=1154.
    // PDF point→96DPI CSS 변환. 후보bbox나 snapshot에서 기대값을 만들지 않는다.
    let expected_x = 1154.0 * 0.12 * 96.0 / 72.0;
    assert!(
        (x - expected_x).abs() <= 2.0,
        "원본 HFT 괄호 뒤 실제 Print 원점 ±2CSSpx: actual={x}, PDF={expected_x}"
    );
    let mut actual = String::new();
    let mut lines = vec![];
    for p in 0..c.page_count() {
        text(
            &c.build_page_render_tree(p).unwrap().root,
            owner,
            &mut actual,
            &mut lines,
        )
    }
    assert_eq!(
        compact(&actual),
        compact(&original),
        "원본 문단 내용·순서·누락·중복"
    );
    assert_eq!(lines, vec![0], "같은 원본 줄 소속");
}

fn tree_nodes<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        tree_nodes(c, out)
    }
}
#[test]
fn a_later_decoration_keeps_plain_row_margin_and_independent_print_origin() {
    let mut c = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/21_언어_기출_편집가능본.hwp"
        ))
        .unwrap(),
    )
    .unwrap();
    c.set_hangul2024_compat(false);
    let environment = automatic_environment(&c);
    c.set_font_environment(Some(environment)).unwrap();
    assert_eq!(c.page_count(), 15, "독립 Print의 전체 쪽수");
    let d = c.document();
    let found: Vec<_> = d.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.text.starts_with("최근에는 기존의 법학방법론적"))
        .collect();
    assert_eq!(found.len(), 1);
    let (owner, p) = found[0];
    let style = &d.doc_info.para_shapes[p.para_shape_id as usize];
    assert!(style.margin_left > 0 && style.indent > 0);
    assert!(!p.controls.is_empty());
    let tree = c.build_page_render_tree(13).unwrap();
    let mut all = vec![];
    tree_nodes(&tree.root, &mut all);
    let line = |idx| {
        let found:Vec<_>=all.iter().copied().filter(|n|matches!(&n.node_type,RenderNodeType::TextLine(t)if t.para_index==Some(owner)&&t.line_index==Some(idx))).collect();
        assert_eq!(found.len(), 1);
        found[0]
    };
    let a = line(0);
    let b = line(1);
    assert!(b.bbox.y > a.bbox.y);
    let first = &p.line_segs[0];
    let next = &p.line_segs[1];
    // HWP 문단 여백/indent는2배 단위이며 LineSeg의 두 시작 축은 HWPUNIT이다.
    // 서로 다른 소유의 cs와 문단 여백을 비교한다. 후보 화면좌표를 기대값으로 복사하지 않는다.
    let source_advance =
        f64::from(first.column_start - next.column_start) + f64::from(style.indent) / 2.0;
    let expected_ratio = source_advance / f64::from(first.line_height);
    let actual_ratio = (a.bbox.x - b.bbox.x) / a.bbox.height;
    assert!(
        (actual_ratio - expected_ratio).abs() < 1e-8,
        "첫줄/후속줄의 source margin 소유: actual={actual_ratio}, source={expected_ratio}"
    );
    let mut actual = String::new();
    let mut owned_lines = vec![];
    for page in 0..c.page_count() {
        let tree = c.build_page_render_tree(page).unwrap();
        let mut ns = vec![];
        tree_nodes(&tree.root, &mut ns);
        for n in ns {
            match &n.node_type {
                RenderNodeType::TextRun(t) if t.para_index == Some(owner) => {
                    actual.push_str(&t.text)
                }
                RenderNodeType::TextLine(t) if t.para_index == Some(owner) => {
                    owned_lines.push(t.line_index.unwrap())
                }
                _ => {}
            }
        }
    }
    assert_eq!(
        compact(&actual),
        compact(&p.text),
        "원본 문단 내용·순서·누락·중복 보존"
    );
    assert_eq!(
        owned_lines,
        (0..p.line_segs.len() as u32).collect::<Vec<_>>()
    );
    let page = c
        .build_page_layer_tree_with_profile(13, RenderProfile::Print)
        .unwrap();
    let mut painted = vec![];
    ops(&page.root, &mut painted);
    let following: Vec<_> = painted.into_iter().filter(|op| matches!(op,
        PaintOp::TextRun { run, .. } if run.para_index == Some(owner) && run.text == "하나의"
    )).collect();
    assert_eq!(following.len(), 1, "원본 이어받기 줄의 단일 소유");
    let op = following[0];
    let PaintOp::TextRun { bbox, run, .. } = op else {
        unreachable!()
    };
    let x = op.glyph_paint_transform().map_or(bbox.x, |t| {
        t.a * bbox.x + t.c * (bbox.y + run.baseline) + t.e
    });
    // 독립 한컴2020 Print14쪽 cm.a=.12, '하나의' tm.x=3697, point→96DPI CSS.
    assert!(
        (x - 3697.0 * 0.12 * 96.0 / 72.0).abs() <= 2.0,
        "독립 Print 이어받기 원점 ±2CSSpx: {x}"
    );
}

//! 정상 한컴 저장본의 인라인 줄 소유·간격과 수식 본체·첨자 관계를 보호한다.
//! 전쪽 진단 최저 Native 98.89677 / fresh WASM 99.24726%, 쪽수 일치.
//! 원본 PrEP 140쪽은 최저 91.54965 / 92.43307%, 완료 33쪽 99.46565 / 99.47442%.
//! 9쪽은 양쪽 100%. 독립 Print·96dpi·직접 판독 증거는 1621단계 보고서에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::{BTreeMap, BTreeSet};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/equation-font-metrics/hancom-structure-saved-2024.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    core
}

fn equations<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a EquationNode)>) {
    if let RenderNodeType::Equation(value) = &node.node_type {
        out.push((node, value));
    }
    for child in &node.children {
        equations(child, out);
    }
}

fn lines<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(value) if value.para_index.is_some()) {
        out.push(node);
    }
    for child in &node.children {
        lines(child, out);
    }
}

// 실제 glyph 원점과 기준선을 자식의 상대 위치까지 누적한다.
fn glyphs(value: &LayoutBox, x: f64, y: f64, out: &mut Vec<(char, f64, f64)>) {
    let x = x + value.x;
    let y = y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            for (glyph, position) in run.glyphs.chars().zip(&run.positions) {
                out.push((glyph, x + position, y + value.baseline + run.baseline_shift));
            }
        }
        LayoutKind::Row(items) => {
            for child in items {
                glyphs(child, x, y, out);
            }
        }
        LayoutKind::Fraction { numer, denom, .. } => {
            glyphs(numer, x, y, out);
            glyphs(denom, x, y, out);
        }
        LayoutKind::Subscript { base, sub } => {
            glyphs(base, x, y, out);
            glyphs(sub, x, y, out);
        }
        LayoutKind::Superscript { base, sup } => {
            glyphs(base, x, y, out);
            glyphs(sup, x, y, out);
        }
        LayoutKind::SubSup { base, sub, sup } => {
            glyphs(base, x, y, out);
            glyphs(sub, x, y, out);
            glyphs(sup, x, y, out);
        }
        LayoutKind::Decoration { body, .. }
        | LayoutKind::FontStyle { body, .. }
        | LayoutKind::Color { body, .. } => glyphs(body, x, y, out),
        _ => {}
    }
}

#[test]
fn saved_and_uncached_inline_rows_keep_source_owners_spacing_and_all_equations() {
    let core = load();
    assert_eq!(
        core.page_count(),
        16,
        "독립 Print 전체 쪽수와 내용 소유를 함께 검사"
    );
    let paragraphs = &core.document().sections[0].paragraphs;
    let mut source = BTreeMap::new();
    for (pi, p) in paragraphs.iter().enumerate() {
        for (ci, c) in p.controls.iter().enumerate() {
            if let Control::Equation(e) = c {
                assert!(e.common.treat_as_char);
                assert!(source.insert((pi, ci), e.script.clone()).is_none());
                if pi >= 40 {
                    assert!(p.line_segs.is_empty(), "재조판이 필요한 실제 저장본");
                }
            }
        }
    }
    assert_eq!(source.len(), 128);
    let mut seen = BTreeSet::new();
    for page in 0..16 {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut actual = Vec::new();
        equations(&tree.root, &mut actual);
        assert_eq!(actual.len(), 8, "Print의 쪽별 수식 누락·중복");
        for (node, e) in actual {
            assert_eq!(e.section_index, Some(0));
            let key = (e.para_index.unwrap(), e.control_index.unwrap());
            assert!(seen.insert(key), "중복 소유: {key:?}");
            assert_eq!(source.get(&key), Some(&e.script));
            assert_eq!(key.0 / 8, page as usize, "원본 내용의 쪽 소유");
            assert!(node.bbox.height > 0.0);
        }
        let mut rows = Vec::new();
        lines(&tree.root, &mut rows);
        rows.sort_by_key(|node| match &node.node_type {
            RenderNodeType::TextLine(value) => value.para_index.unwrap(),
            _ => unreachable!(),
        });
        assert_eq!(rows.len(), 8);
        for pair in rows.windows(2) {
            let indices: Vec<_> = pair
                .iter()
                .map(|node| match &node.node_type {
                    RenderNodeType::TextLine(value) => value.para_index.unwrap(),
                    _ => unreachable!(),
                })
                .collect();
            let saved: Vec<_> = indices
                .iter()
                .map(|pi| &core.document().sections[0].paragraphs[*pi].line_segs)
                .collect();
            if let (Some(first), Some(second)) = (saved[0].first(), saved[1].first()) {
                let expected = (second.vertical_pos - first.vertical_pos) as f64 * 96.0 / 7200.0;
                let advance = pair[1].bbox.y - pair[0].bbox.y;
                assert!(
                    (advance - expected).abs() <= 0.2,
                    "유효한 저장 줄의 상대 전진: {advance} / {expected}"
                );
            } else {
                // 실제 표시 줄 상자는 글리프의 하강부까지 포함한다. 단일 글자
                // 크기만으로 그 높이를 재계산하지 않고 앞 줄의 실제 점유 뒤에
                // 양수 간격이 남는지 검사한다. 원래 결함에서는 정확히 0이었다.
                let spacing = pair[1].bbox.y - pair[0].bbox.y - pair[0].bbox.height;
                assert!(
                    spacing > 0.0,
                    "쪽 {page}, 문단 {indices:?}: 앞 줄 점유 뒤 원본 간격 소실 {spacing}"
                );
            }
        }
    }
    assert_eq!(seen.len(), source.len(), "전체 수식 보존");
}

#[test]
fn hancom_fraction_subscript_stays_below_denominator_at_each_source_size() {
    let core = load();
    let mut count = 0;
    for page in [1, 3, 5, 7] {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut actual = Vec::new();
        equations(&tree.root, &mut actual);
        let (_, e) = actual
            .into_iter()
            .find(|(_, e)| e.script.split_whitespace().collect::<String>() == "{{1}over{N}}_{i}")
            .unwrap();
        let mut ink = Vec::new();
        glyphs(&e.layout_box, 0.0, 0.0, &mut ink);
        let n = ink.iter().find(|v| v.0 == '\u{e00d}').unwrap();
        let i = ink.iter().find(|v| v.0 == '\u{e0ed}').unwrap();
        assert!(i.1 > n.1, "분수 본체 뒤 첨자의 좌우 순서");
        assert!(i.2 > n.2, "첨자는 분모 기준선 아래에 있어야 함");
        // 독립 Print의 본체·첨자 간 약 1/4em 하강. 절대 화면 위치가 아니다.
        assert!(
            (i.2 - n.2 - e.font_size * 0.25).abs() <= 0.3,
            "독립 Print와 첨자 기준선 관계: {} / {}",
            i.2 - n.2,
            e.font_size
        );
        count += 1;
    }
    assert_eq!(count, 4);
}

fn near_print(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {label} ±2 CSS px: {actual} / {expected}"
    );
}

#[test]
fn prep_completed_hancom_formula_keeps_sum_fraction_and_hat_body_print_origins() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 140);
    let tree = core.build_page_render_tree(32).unwrap();
    let mut actual = Vec::new();
    equations(&tree.root, &mut actual);
    assert_eq!(actual.len(), 5, "독립 Print 33쪽 수식의 누락·중복");
    let matches: Vec<_> = actual
        .iter()
        .filter(|(_, e)| e.script.contains("sum _"))
        .collect();
    assert_eq!(matches.len(), 1, "원본 스크립트로 소유한 합·분수 수식 선택");
    let (node, e) = *matches[0];
    let pi = e.para_index.unwrap();
    let ci = e.control_index.unwrap();
    let source = &core.document().sections[e.section_index.unwrap()].paragraphs[pi];
    let Control::Equation(source_eq) = &source.controls[ci] else {
        panic!("수식 소유")
    };
    assert_eq!(source_eq.script, e.script);
    let mut ink = Vec::new();
    glyphs(&e.layout_box, node.bbox.x, node.bbox.y, &mut ink);
    // 독립 Print PDF 33쪽 실제 glyph 원점. 96dpi·좌상단 CSS px.
    // 장식 본체·분모·합과 상한·분자·첨자 분모를 각각 보호한다.
    let anchors = [
        ('\u{e007}', 116.0, 243.2, "장식 본체 H"),
        ('\u{e0af}', 154.4, 256.8, "첫 분모 tau"),
        ('\u{e067}', 232.32, 247.68, "합 기호"),
        ('\u{e0f2}', 239.36, 224.96, "합 상한 n"),
        ('\u{e0fd}', 256.64, 229.44, "두 번째 분자 y"),
        ('\u{e0e8}', 261.44, 256.8, "두 번째 분모 d"),
    ];
    let mut selected = Vec::new();
    for (code, x, y, label) in anchors {
        let found: Vec<_> = ink.iter().filter(|v| v.0 == code).collect();
        assert_eq!(found.len(), 1, "고유 수식 glyph의 보존: {label}");
        near_print(found[0].1, x, label);
        near_print(found[0].2, y, label);
        selected.push(*found[0]);
    }
    assert!(
        selected.windows(2).all(|pair| pair[1].1 > pair[0].1),
        "수식 내용의 좌우 순서"
    );
    assert!(selected[3].2 < selected[2].2, "상한은 합 기호 위쪽");
    assert!(selected[4].2 < selected[5].2, "같은 분수의 분자·분모 소속");
}

#[test]
fn completed_uncached_rows_keep_independent_print_glyph_baselines() {
    let core = load();
    let tree = core.build_page_render_tree(8).unwrap();
    let mut actual = Vec::new();
    equations(&tree.root, &mut actual);
    // 전쪽 선행 조건과 직접 판독을 마친 9쪽은 양쪽 실루엣 100%다.
    // source 문단 소유로 선택하고 독립 Print의 glyph 원점을 보호한다.
    for (paragraph, code, x, y) in [
        (64, '\u{e007}', 217.32222, 106.60350),
        (65, '\u{e0fd}', 216.04291, 130.09782),
        (67, '\u{e007}', 240.02991, 176.92664),
        (68, '\u{e007}', 238.11094, 200.42098),
    ] {
        let found: Vec<_> = actual
            .iter()
            .filter(|(_, e)| e.para_index == Some(paragraph))
            .collect();
        assert_eq!(found.len(), 1, "원본 문단의 고유 수식 소유");
        let (node, e) = *found[0];
        let mut ink = Vec::new();
        glyphs(&e.layout_box, node.bbox.x, node.bbox.y, &mut ink);
        let target: Vec<_> = ink.iter().filter(|v| v.0 == code).collect();
        assert_eq!(target.len(), 1, "원본 줄의 고유 glyph 보존");
        near_print(target[0].1, x, "저장 줄 없는 호스트의 수평 원점");
        near_print(target[0].2, y, "여러 줄의 순서와 기준선");
    }
}

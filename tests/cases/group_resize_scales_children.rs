//! 관계형 검사. 묶음의 원래 크기·변경 배율에 대해 자식 상자와 두 저장본의 행렬을 대조한다.
//! 주석 갱신 2026-11-21T15:21:50+09:00. 직전 파일 변경 7ec3e1e94fe323a03180af574f739ae17da1cdd6 2026-11-20T21:26:24+09:00.
//! 2026-11-21 한컴 2020 독립 인쇄본과 96 DPI에서 색상 경계 보정 전 2px 실루엣을 측정했다.
//! 아래는 쪽 순서대로 Native Skia와 fresh WASM Canvas2D의 raw 일치율(%)이다.
//! `resized-group.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `resized-group.hwpx` 1쪽. Native [100.00000], WASM [100.00000].
//! 이 범위의 전쪽 최저는 Native 100.00000%, WASM 100.00000%다.
//! 입력·PDF 해시와 쪽별 근거는 `mydocs/pr/assets/semanticist21-20261121/resized-group/pages.tsv`에 있다.
//!
//! 묶음 크기를 바꾸면 한컴 12.30처럼 자식이 묶음 상자를 따라 늘고 준다.
//! 렌더 트리의 자식 상자와 HWP/HWPX 저장 후 다시 연 문서, 저장 행렬 꼴을 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::{json, Value};

const RED: u32 = 0x0000ff;
const GREEN: u32 = 0x00ff00;

fn add(doc: &mut DocumentCore, kind: &str, x: u32, y: u32, w: u32, h: u32, fill: u32) -> usize {
    let result: Value = serde_json::from_str(
        &doc.create_shape_control_native(
            0,
            0,
            0,
            w,
            h,
            x,
            y,
            false,
            "InFrontOfText",
            kind,
            false,
            false,
            &[],
        )
        .unwrap(),
    )
    .unwrap();
    let ci = result["controlIdx"].as_u64().unwrap() as usize;
    doc.set_shape_properties_native(0, 0, ci, &json!({ "fillBgColor": fill }).to_string())
        .unwrap();
    ci
}

/// 색으로 구분한 도형의 쪽 상자(px)를 색 순서로 모은다.
fn boxes(doc: &DocumentCore) -> Vec<(u32, [f64; 4])> {
    fn visit(node: &RenderNode, out: &mut Vec<(u32, [f64; 4])>) {
        let fill = match &node.node_type {
            RenderNodeType::Rectangle(r) => r.style.fill_color,
            RenderNodeType::Ellipse(e) => e.style.fill_color,
            _ => None,
        };
        if let Some(fill @ (RED | GREEN)) = fill {
            let b = node.bbox;
            out.push((fill, [b.x, b.y, b.width, b.height]));
        }
        for child in &node.children {
            visit(child, out);
        }
    }
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut out = Vec::new();
    visit(&tree.root, &mut out);
    out.sort_by_key(|(c, _)| *c);
    out
}

fn assert_boxes(label: &str, doc: &DocumentCore, expected: &[(u32, [f64; 4])]) {
    let actual = boxes(doc);
    assert_eq!(actual.len(), expected.len(), "{label}: 도형 수");
    for ((c, a), (e_c, e)) in actual.iter().zip(expected) {
        assert_eq!(c, e_c, "{label}: 색");
        for i in 0..4 {
            assert!(
                (a[i] - e[i]).abs() < 0.6,
                "{label}: 색 {c:06x} 상자 {a:?} ≠ 기대 {e:?}"
            );
        }
    }
}

fn group(doc: &DocumentCore) -> &rhwp::model::shape::GroupShape {
    doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Group(g) => Some(g),
                _ => None,
            },
            _ => None,
        })
        .expect("묶음")
}

fn matrix(raw: &[u8], index: usize) -> [f64; 6] {
    let mut m = [0.0; 6];
    for (i, v) in m.iter_mut().enumerate() {
        let o = 2 + index * 48 + i * 8;
        *v = f64::from_le_bytes(raw[o..o + 8].try_into().unwrap());
    }
    m
}

#[test]
fn group_resize_scales_children_like_hancom() {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    add(&mut doc, "rectangle", 9000, 12000, 8000, 6000, RED);
    add(&mut doc, "ellipse", 21000, 24000, 6000, 4000, GREEN);
    let targets: Vec<_> = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c, Control::Shape(_)))
        .map(|(i, _)| (0, i))
        .collect();
    doc.group_shapes_native(0, &targets).unwrap();
    let before = boxes(&doc);
    assert_eq!(before.len(), 2, "묶은 두 도형이 그려져야 한다");
    let (w, h) = (group(&doc).common.width, group(&doc).common.height);
    // 오른쪽 아래 손잡이로 너비 1.5배, 높이 2배 — 왼쪽 위는 제자리다.
    let index = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Shape(s) if matches!(s.as_ref(), ShapeObject::Group(_))))
        .expect("묶음 자리");
    doc.set_shape_properties_native(
        0,
        0,
        index,
        &json!({ "width": w * 3 / 2, "height": h * 2 }).to_string(),
    )
    .unwrap();
    let (left, top) = (
        before
            .iter()
            .map(|(_, b)| b[0])
            .fold(f64::INFINITY, f64::min),
        before
            .iter()
            .map(|(_, b)| b[1])
            .fold(f64::INFINITY, f64::min),
    );
    let expected: Vec<_> = before
        .iter()
        .map(|(c, b)| {
            (
                *c,
                [
                    left + (b[0] - left) * 1.5,
                    top + (b[1] - top) * 2.0,
                    b[2] * 1.5,
                    b[3] * 2.0,
                ],
            )
        })
        .collect();
    assert_boxes("크기 바꾼 직후", &doc, &expected);
    let hwp = DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    assert_boxes("HWP 저장 후", &hwp, &expected);
    let hwpx = DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap();
    assert_boxes("HWPX 저장 후", &hwpx, &expected);

    // 한컴 12.30 저장본 꼴: 묶음 current만 늘고 original은 그대로, 자식 첫 단계 크기 행렬은
    // [k, 0, tx(k−1); 0, k, ty(k−1)], 자식 current도 같은 배율이다.
    let g = group(&hwp);
    assert_eq!(
        (g.shape_attr.current_width, g.shape_attr.current_height),
        (w * 3 / 2, h * 2)
    );
    assert_eq!(
        (g.shape_attr.original_width, g.shape_attr.original_height),
        (w, h)
    );
    for child in &g.children {
        let sa = child.shape_attr();
        let (trans, scale) = (matrix(&sa.raw_rendering, 0), matrix(&sa.raw_rendering, 1));
        let expected = [1.5, 0.0, trans[2] * 0.5, 0.0, 2.0, trans[5]];
        for i in 0..6 {
            assert!(
                (scale[i] - expected[i]).abs() < 1e-6,
                "자식 첫 단계 크기 행렬 {scale:?} ≠ {expected:?}"
            );
        }
        assert_eq!(sa.current_width, sa.original_width * 3 / 2);
        assert_eq!(sa.current_height, sa.original_height * 2);
    }
}

//! 개체 묶기는 자식이 그려진 모습(크기·회전·자리)과 겹침 순서를 바꾸지 않아야 한다.
//! 묶기 전·후와 HWP/HWPX 저장 후 다시 연 문서의 실제 렌더 트리 윤곽을 비교한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::PathCommand;
use serde_json::{json, Value};

const RED: u32 = 0x0000ff;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0xff0000;

fn blank() -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "앞쪽 가나다 뒤쪽").unwrap();
    doc
}

/// 도형을 만들고 속성을 준 뒤 (문단, 컨트롤) 위치를 돌려준다.
fn add(doc: &mut DocumentCore, kind: &str, x: u32, y: u32, w: u32, h: u32, props: Value) {
    let result: Value = serde_json::from_str(
        &doc.create_shape_control_native(
            0,
            0,
            2,
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
    doc.set_shape_properties_native(0, 0, ci, &props.to_string())
        .unwrap();
}

fn shape_indices(doc: &DocumentCore) -> Vec<usize> {
    doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c, Control::Shape(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 색으로 구분한 도형마다 실제 출력 윤곽을 촘촘히 샘플링한 점(쪽 px)을 그리기 순서대로 모은다.
/// 묶기 전에는 회전 transform을 가진 Ellipse/Rectangle, 묶은 뒤에는 Path가 될 수 있어
/// 노드 종류가 아니라 최종 윤곽을 비교한다.
fn outlines(doc: &DocumentCore) -> Vec<(u32, Vec<(f64, f64)>)> {
    fn visit(node: &RenderNode, out: &mut Vec<(u32, Vec<(f64, f64)>)>) {
        let b = node.bbox;
        let (fill, transform, points) = match &node.node_type {
            RenderNodeType::Ellipse(e) => {
                let (cx, cy, rx, ry) = (
                    b.x + b.width / 2.0,
                    b.y + b.height / 2.0,
                    b.width / 2.0,
                    b.height / 2.0,
                );
                let points = (0..512)
                    .map(|i| {
                        let t = i as f64 * std::f64::consts::TAU / 512.0;
                        (cx + rx * t.cos(), cy + ry * t.sin())
                    })
                    .collect();
                (e.style.fill_color, e.transform, points)
            }
            RenderNodeType::Rectangle(r) => {
                let corners = [
                    (b.x, b.y),
                    (b.x + b.width, b.y),
                    (b.x + b.width, b.y + b.height),
                    (b.x, b.y + b.height),
                    (b.x, b.y),
                ];
                (r.style.fill_color, r.transform, densify(&corners))
            }
            RenderNodeType::Path(p) => {
                let mut points = Vec::new();
                let mut cur = (0.0, 0.0);
                let mut start = (0.0, 0.0);
                for command in &p.commands {
                    match *command {
                        PathCommand::MoveTo(x, y) => {
                            cur = (x, y);
                            start = cur;
                        }
                        PathCommand::LineTo(x, y) => {
                            points.extend(densify(&[cur, (x, y)]));
                            cur = (x, y);
                        }
                        PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                            for i in 0..=128 {
                                let t = i as f64 / 128.0;
                                let u = 1.0 - t;
                                let f = |a: f64, b: f64, c: f64, d: f64| {
                                    u * u * u * a
                                        + 3.0 * u * u * t * b
                                        + 3.0 * u * t * t * c
                                        + t * t * t * d
                                };
                                points.push((f(cur.0, x1, x2, x), f(cur.1, y1, y2, y)));
                            }
                            cur = (x, y);
                        }
                        PathCommand::ClosePath => {
                            points.extend(densify(&[cur, start]));
                            cur = start;
                        }
                        PathCommand::ArcTo(..) => panic!("합성 도형에 호가 없어야 한다"),
                    }
                }
                (p.style.fill_color, p.transform, points)
            }
            _ => {
                for child in &node.children {
                    visit(child, out);
                }
                return;
            }
        };
        let Some(fill @ (RED | GREEN | BLUE)) = fill else {
            return;
        };
        assert!(!transform.horz_flip && !transform.vert_flip);
        let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
        let (sin, cos) = transform.rotation.to_radians().sin_cos();
        let points = points
            .into_iter()
            .map(|(x, y)| {
                (
                    cx + (x - cx) * cos - (y - cy) * sin,
                    cy + (x - cx) * sin + (y - cy) * cos,
                )
            })
            .collect();
        out.push((fill, points));
    }
    fn densify(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        points
            .windows(2)
            .flat_map(|w| {
                (0..=128).map(move |i| {
                    let t = i as f64 / 128.0;
                    (
                        w[0].0 + (w[1].0 - w[0].0) * t,
                        w[0].1 + (w[1].1 - w[0].1) * t,
                    )
                })
            })
            .collect()
    }
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut out = Vec::new();
    visit(&tree.root, &mut out);
    out
}

/// 두 윤곽 점 집합의 하우스도르프 거리 (px).
fn distance(a: &[(f64, f64)], b: &[(f64, f64)]) -> f64 {
    let one_way = |a: &[(f64, f64)], b: &[(f64, f64)]| {
        a.iter()
            .map(|p| {
                b.iter()
                    .map(|q| (p.0 - q.0).hypot(p.1 - q.1))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    one_way(a, b).max(one_way(b, a))
}

fn assert_same_drawing(
    label: &str,
    before: &[(u32, Vec<(f64, f64)>)],
    doc: &DocumentCore,
    order: bool,
) {
    let after = outlines(doc);
    let colors = |o: &[(u32, Vec<(f64, f64)>)]| o.iter().map(|(c, _)| *c).collect::<Vec<_>>();
    if order {
        assert_eq!(
            colors(&after),
            colors(before),
            "{label}: 그리기 순서(겹침)가 같아야 한다"
        );
    }
    assert_eq!(after.len(), before.len(), "{label}: 도형 수");
    for (color, a) in before {
        let (_, b) = after
            .iter()
            .find(|(c, _)| c == color)
            .expect("같은 색 도형");
        let d = distance(a, b);
        assert!(
            d < 1.0,
            "{label}: 색 {color:06x} 도형 윤곽이 {d:.2}px 달라졌다"
        );
    }
}

/// 묶기 전 모습을 기록하고, 묶은 직후·HWP·HWPX 저장 후 다시 연 문서와 비교한다.
/// 기준이 다른 개체는 종이 기준 층이 따로 그려져 묶기 전 순서가 z-order와 다를 수 있으므로
/// `order`가 참일 때만 그리기 순서를 비교한다.
fn group_and_check(mut doc: DocumentCore, targets: &[(usize, usize)], order: bool) {
    let before = outlines(&doc);
    assert_eq!(
        before.len(),
        targets.len(),
        "묶기 전 도형이 모두 그려져야 한다"
    );
    doc.group_shapes_native(0, targets).unwrap();
    assert_same_drawing("묶은 직후", &before, &doc, order);
    let hwp = DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    assert_same_drawing("HWP 저장 후", &before, &hwp, order);
    let hwpx = DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap();
    assert_same_drawing("HWPX 저장 후", &before, &hwpx, order);
}

#[test]
fn grouping_keeps_scaled_and_rotated_children_drawn_in_place() {
    let mut doc = blank();
    // 원래 크기의 두 배로 늘린 타원, 30° 돌린 타원, 30° 돌린 사각형
    add(
        &mut doc,
        "ellipse",
        6000,
        15000,
        8250,
        7425,
        json!({"width": 16500, "height": 14850, "fillBgColor": RED}),
    );
    add(
        &mut doc,
        "ellipse",
        26000,
        15000,
        8250,
        7425,
        json!({"rotationAngle": 30, "fillBgColor": GREEN}),
    );
    add(
        &mut doc,
        "rectangle",
        30000,
        30000,
        6000,
        3000,
        json!({"width": 9000, "rotationAngle": 30, "fillBgColor": BLUE}),
    );
    let targets: Vec<_> = shape_indices(&doc).into_iter().map(|ci| (0, ci)).collect();
    group_and_check(doc, &targets, true);
}

#[test]
fn grouping_keeps_stacking_order_regardless_of_target_order() {
    let mut doc = blank();
    // 나중에 만든 초록이 빨강 위에 겹친다. 선택은 위 개체부터 넘긴다.
    add(
        &mut doc,
        "rectangle",
        6000,
        15000,
        9000,
        6000,
        json!({"fillBgColor": RED}),
    );
    add(
        &mut doc,
        "rectangle",
        9000,
        18000,
        9000,
        6000,
        json!({"fillBgColor": GREEN}),
    );
    let before = outlines(&doc);
    assert_eq!(
        before.iter().map(|(c, _)| *c).collect::<Vec<_>>(),
        [RED, GREEN]
    );
    let mut targets: Vec<_> = shape_indices(&doc).into_iter().map(|ci| (0, ci)).collect();
    targets.sort_by_key(|&(pi, ci)| {
        match &doc.document().sections[0].paragraphs[pi].controls[ci] {
            Control::Shape(s) => std::cmp::Reverse(s.common().z_order),
            _ => unreachable!(),
        }
    });
    group_and_check(doc, &targets, true);
}

#[test]
fn grouping_keeps_children_positioned_against_different_bases() {
    let bases = [
        json!({"horzRelTo": "Paper", "horzAlign": "Left", "vertRelTo": "Paper", "vertAlign": "Top"}),
        json!({"horzRelTo": "Page", "horzAlign": "Right", "vertRelTo": "Page", "vertAlign": "Bottom"}),
        json!({"horzRelTo": "Column", "horzAlign": "Center", "vertRelTo": "Para", "vertAlign": "Top"}),
    ];
    let colors = [RED, GREEN, BLUE];
    // 문서상 첫 개체(묶음이 놓일 자리·기준을 물려줄 개체)가 기준마다 한 번씩 오게 돌린다.
    for shift in 0..bases.len() {
        let mut doc = blank();
        for i in 0..3 {
            add(
                &mut doc,
                "rectangle",
                4000 + 9000 * i as u32,
                12000 + 4000 * i as u32,
                6000,
                4000,
                json!({}),
            );
        }
        for (k, ci) in shape_indices(&doc).into_iter().enumerate() {
            let mut props = bases[(k + shift) % bases.len()].clone();
            props["fillBgColor"] = json!(colors[k]);
            props["horzOffset"] = json!(3000 + 2000 * k as u32);
            props["vertOffset"] = json!(5000 + 1500 * k as u32);
            doc.set_shape_properties_native(0, 0, ci, &props.to_string())
                .unwrap();
        }
        let targets: Vec<_> = shape_indices(&doc).into_iter().map(|ci| (0, ci)).collect();
        group_and_check(doc, &targets, false);
    }
}

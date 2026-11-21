//! 관계형: 글자 모양 참조 없는 글머리표 마커와 문단 끝 글자 모양·본문 시작·고리 윤곽의 관계.
//! 주석 갱신: 2026-11-20T05:29:53+09:00. 새 파일이므로 직전 변경 커밋은 없다.
//! 측정 2026-11-20, base upstream/devel 9f605a0d0a + 이 수정의 작업 트리, 96 DPI·Print 프로필,
//! 보정 전 2px raw 실루엣(threshold_boundary_color_support_v1, 이진화 232, 경계 보정 0px).
//! samples/issue614/issue614-bullet-end-char-shape.hwp ↔ pdf/issue614/issue614-bullet-end-char-shape-2022.pdf
//! (한컴 2022 12.0.0.4547 Print), 전체 45쪽이 Native·fresh WASM·PDF 모두 일치한다.
//! 검사 2쪽: Native 100.00% / fresh WASM 100.00% (수정 전 Native 98.58%).
//! 대조 13쪽: Native 99.99% / fresh WASM 100.00%. 전쪽 최저 Native 99.55%(4쪽)·fresh WASM 99.55%(4쪽).
//! 같은 쪽 review·overlay와 독립 PDF를 확대 판독했다. 마커 원점·본문 시작은 원본 글자 모양·
//! 거리 50%에서 계산한 관계이고, 고리 반지름은 독립 PDF의 Wingdings 윤곽(UPEM 2048) 비율이다.
//! 특정 글꼴 설치에 의존하지 않는다(고리는 글꼴 없이 도형으로 그린다).
//! 증거: mydocs/pr/assets/issue_614/ (native·wasm·base-native TSV, review/overlay PNG).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode},
    DocumentCore,
};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue614/issue614-bullet-end-char-shape.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("issue614 원본")).unwrap()
}

fn collect<'a>(node: &'a RenderNode, output: &mut Vec<(&'a RenderNode, &'a TextRunNode)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        output.push((node, run));
    }
    for child in &node.children {
        collect(child, output);
    }
}

struct BulletParagraph {
    marker_size: f64,
    marker_bold: bool,
    marker_x: f64,
    body_x: f64,
    first_size: f64,
    end_size: f64,
    end_bold: bool,
    distance_percent: f64,
    body_text: String,
}

/// 쪽의 첫 글머리표 문단. 원본 BULLET 레코드가 글자 모양을 참조하지 않음도 확인한다.
fn bullet_paragraph(core: &DocumentCore, page: u32) -> BulletParagraph {
    let tree = core.build_page_render_tree(page).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    let (marker_node, marker) = runs
        .iter()
        .copied()
        .find(|(_, run)| run.char_start.is_none() && run.para_index.is_some())
        .expect("글머리표 마커");
    let section = marker.section_index.unwrap();
    let para_index = marker.para_index.unwrap();
    let document = core.document();
    let para = &document.sections[section].paragraphs[para_index];
    let shape = &document.doc_info.para_shapes[para.para_shape_id as usize];
    let bullet = &document.doc_info.bullets[shape.numbering_id as usize - 1];
    assert_eq!(bullet.char_shape_id, u32::MAX, "참조 없는 글머리표");
    assert_eq!(bullet.bullet_char, '\u{F0A1}');
    let char_shape = |id: u32| &document.doc_info.char_shapes[id as usize];
    let first = char_shape(para.char_shapes.first().unwrap().char_shape_id);
    let end = char_shape(para.char_shapes.last().unwrap().char_shape_id);
    let body: Vec<_> = runs
        .iter()
        .filter(|(_, run)| {
            run.char_start.is_some()
                && run.section_index == Some(section)
                && run.para_index == Some(para_index)
        })
        .collect();
    BulletParagraph {
        marker_size: marker.style.font_size,
        marker_bold: marker.style.bold,
        marker_x: marker_node.bbox.x,
        body_x: body
            .iter()
            .map(|(node, _)| node.bbox.x)
            .fold(f64::MAX, f64::min),
        first_size: f64::from(first.base_size) / 75.0,
        end_size: f64::from(end.base_size) / 75.0,
        end_bold: end.bold,
        distance_percent: f64::from(bullet.text_distance),
        body_text: body.iter().map(|(_, run)| run.text.as_str()).collect(),
    }
}

/// Wingdings 0xA1 은 ○ 글꼴의 가는 원이 아니라 원 프로그램 윤곽의 굵은 고리다.
/// 독립 Print glyph 3: advance 1825, 바깥 (173,0)..(1653,1480), 안쪽 (321,148)..(1505,1332).
/// 고리 중심은 마커 원점에서 913/2048em 오른쪽이다.
fn assert_ring(core: &DocumentCore, page: u32, p: &BulletParagraph) {
    let svg = core.render_page_svg_native(page).unwrap();
    let (center_x, outer, inner) = svg
        .split("<path d=\"M")
        .skip(1)
        .find_map(|tail| {
            let head = &tail[..tail.find('>')?];
            if !head.contains("fill-rule=\"evenodd\"") {
                return None;
            }
            let d = &head[..head.find('"')?];
            let mut contours = d.split('M');
            let outer = contours.next()?;
            let left: f64 = outer.split(' ').next()?.parse().ok()?;
            let rx: f64 = outer.split('a').nth(1)?.split(' ').next()?.parse().ok()?;
            let inner: f64 = contours
                .next()?
                .split('a')
                .nth(1)?
                .split(' ')
                .next()?
                .parse()
                .ok()?;
            Some((left + rx, rx, inner))
        })
        .expect("Wingdings 0xA1 고리 경로");
    let em = p.marker_size;
    assert!(
        (outer / em - 740.0 / 2048.0).abs() < 0.002,
        "바깥 반지름 {outer}"
    );
    assert!(
        (inner / outer - 0.8).abs() < 0.002,
        "안쪽/바깥 {inner}/{outer}"
    );
    assert!(
        (center_x - p.marker_x - em * 913.0 / 2048.0).abs() < 0.5,
        "고리 중심 x {center_x}, 마커 {}",
        p.marker_x
    );
    assert!(!svg.contains('\u{F0A1}'), "원 PUA 를 글자로 그리지 않는다");
}

/// 2쪽: 첫 글자 11pt 보통, 끝 16pt 진하게. 독립 Print 는 끝 크기·채움만으로 마커를 그린다.
#[test]
fn unreferenced_bullet_uses_end_size_without_end_bold() {
    let core = load();
    let p = bullet_paragraph(&core, 1);
    assert!(p.end_size > p.first_size, "끝 모양이 더 큰 원본 조건");
    assert!(p.end_bold, "끝 모양은 진하게");
    assert!(
        (p.marker_size - p.end_size).abs() < 0.01,
        "마커 크기 = 문단 끝 크기"
    );
    assert!(
        !p.marker_bold,
        "참조 없는 글머리표는 끝 모양의 진하게를 쓰지 않는다"
    );
    // 본문은 마커 영역(1em + 끝 크기의 거리%) 뒤에서 시작한다.
    let area = p.end_size * (1.0 + p.distance_percent / 100.0);
    assert!(
        (p.body_x - p.marker_x - area).abs() < 0.5,
        "본문 시작 - 마커 = {} (영역 {area})",
        p.body_x - p.marker_x
    );
    assert!(
        p.body_text.contains("추후작성예정"),
        "본문 보존: {}",
        p.body_text
    );
    assert_ring(&core, 1, &p);
}

/// 13쪽 대조군: 첫·끝 모두 11pt 보통인 같은 글머리표는 그대로 11pt 보통이다.
#[test]
fn same_bullet_with_plain_end_shape_is_unchanged() {
    let core = load();
    let p = bullet_paragraph(&core, 12);
    assert!((p.end_size - p.first_size).abs() < 0.01);
    assert!(!p.end_bold);
    assert!((p.marker_size - p.end_size).abs() < 0.01);
    assert!(!p.marker_bold);
    let area = p.end_size * (1.0 + p.distance_percent / 100.0);
    assert!((p.body_x - p.marker_x - area).abs() < 0.5);
    assert!(
        p.body_text.contains("추후작성예정"),
        "본문 보존: {}",
        p.body_text
    );
    assert_ring(&core, 12, &p);
}

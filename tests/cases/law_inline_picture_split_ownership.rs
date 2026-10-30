//! 관계형·고정형: 그림 문단의 내용 소유·순서·겹침과 독립 Print 쪽 수.
//! 주석 갱신: 2026-10-31T02:20:22+09:00; 직전 파일 커밋: 2026-10-31T00:41:14+09:00 (`12fec16b4b7ef7855e2f3f909ad1573c527e368a`).
//! 2026-10-30 측정 raw(96 DPI, PDFium, 2px 이웃): 1쪽 Native 99.97% / WASM 99.90%,
//! 6쪽 Native 99.98% / WASM 100.00%, 2쪽 Native 98.86% / WASM 98.19%,
//! 3쪽 Native 99.80% / WASM 99.82%; 전 6쪽 최저 Native 98.86% / WASM 98.19%.
//! 근거: mydocs/pr/assets/issue_8371/law-split-20261030/README.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    for child in node["children"].as_array().into_iter().flatten() {
        walk(child, out);
    }
}

#[test]
fn paired_angle_opener_stays_with_the_following_line() {
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let mut doc = HwpDocument::from_bytes(input).unwrap();
    doc.set_hangul2024_compat(false);
    let page: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let mut nodes = Vec::new();
    walk(&page, &mut nodes);
    let lines: Vec<String> = nodes
        .iter()
        .filter(|node| node["type"] == "TextLine" && node["pi"] == 10)
        .map(|line| {
            line["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|node| node["text"].as_str())
                .collect()
        })
        .collect();
    assert_eq!(lines.len(), 2, "원문의 문단과 내용 소유를 유지한다");
    assert!(!lines[0].trim_end().ends_with('<'));
    assert!(lines[1].trim_start().starts_with("<개정"));
}

#[test]
fn missing_paragraph_shape_uses_hancom_default_without_changing_explicit_shapes() {
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let core = rhwp::DocumentCore::from_bytes(input).unwrap();
    let document = core.document();
    let paragraphs = &document.sections[0].paragraphs;
    let missing = &paragraphs[156];
    let explicit = &paragraphs[155];
    assert!(missing.text.is_empty());
    assert_eq!(
        document.doc_info.para_shapes[missing.para_shape_id as usize].line_spacing,
        160
    );
    assert_eq!(
        document.doc_info.para_shapes[explicit.para_shape_id as usize].line_spacing,
        130
    );

    let mut doc = HwpDocument::from_bytes(input).unwrap();
    doc.set_hangul2024_compat(false);
    let page: Value = serde_json::from_str(&doc.get_page_render_tree(5).unwrap()).unwrap();
    let mut nodes = Vec::new();
    walk(&page, &mut nodes);
    let line = |paragraph| {
        nodes
            .iter()
            .find(|node| node["type"] == "TextLine" && node["pi"] == paragraph)
            .expect("빈 문단과 후속 문장은 같은 쪽에 각각 한 번 배치된다")
    };
    let empty = line(156);
    let following = line(157);
    let top = empty["bbox"]["y"].as_f64().unwrap();
    let height = empty["bbox"]["h"].as_f64().unwrap();
    let next_top = following["bbox"]["y"].as_f64().unwrap();
    assert!(
        (next_top - top - height * 1.6).abs() < height / 100.0,
        "참조를 생략한 빈 문단은 한컴 기본 줄 간격을 소비한다"
    );
}

#[test]
fn default_shape_is_scoped_through_nested_hwpx_cells() {
    use rhwp::model::control::Control;
    use std::io::{Cursor, Read, Write};

    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let section = r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"><hp:p><hp:run charPrIDRef="0"><hp:t/></hp:run></hp:p><hp:p paraPrIDRef="0"><hp:tbl><hp:tr><hp:tc><hp:p><hp:run charPrIDRef="0"><hp:t>셀</hp:t></hp:run></hp:p></hp:tc></hp:tr></hp:tbl></hp:p></hs:sec>"#;
    let mut source = zip::ZipArchive::new(Cursor::new(input.as_slice())).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        let name = entry.name().unwrap().into_owned();
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            output.add_directory(name, options).unwrap();
            continue;
        }
        output.start_file(&name, options).unwrap();
        if name == "Contents/section0.xml" {
            output.write_all(section.as_bytes()).unwrap();
        } else {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            output.write_all(&bytes).unwrap();
        }
    }
    let core = rhwp::DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap();
    let document = core.document();
    let paragraphs = &document.sections[0].paragraphs;
    let default_id = paragraphs[0].para_shape_id;
    assert_ne!(default_id, 0, "누락된 참조는 명시된 ID 0과 다르다");
    assert_eq!(
        document.doc_info.para_shapes[default_id as usize].line_spacing,
        160
    );
    assert_eq!(paragraphs[1].para_shape_id, 0);
    let Control::Table(table) = &paragraphs[1].controls[0] else {
        panic!("중첩 표가 보존돼야 한다");
    };
    assert_eq!(table.cells[0].paragraphs[0].para_shape_id, default_id);
    let standalone = rhwp::parser::hwpx::section::parse_hwpx_section(section).unwrap();
    assert_eq!(
        standalone.paragraphs[0].para_shape_id, 0,
        "다른 파싱에 범위가 새지 않는다"
    );
}

#[test]
fn saved_float_frame_is_preserved_when_the_transform_changes() {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn image(node: &RenderNode) -> Option<(f64, f64)> {
        if let RenderNodeType::Image(value) = &node.node_type {
            if value.para_index == Some(0) && value.control_index == Some(2) {
                return Some((node.bbox.width, node.bbox.height));
            }
        }
        node.children.iter().find_map(image)
    }
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let mut core = rhwp::DocumentCore::from_bytes(input).unwrap();
    let mut document = core.document().clone();
    let Control::Picture(picture) = &mut document.sections[0].paragraphs[0].controls[2] else {
        panic!("원본 QR 그림이 보존돼야 한다");
    };
    let common = (
        f64::from(picture.common.width),
        f64::from(picture.common.height),
    );
    let current = (
        f64::from(picture.shape_attr.current_width),
        f64::from(picture.shape_attr.current_height),
    );
    assert!(
        current.0 > common.0 && current.1 > common.1,
        "원본의 중간 크기는 저장 프레임보다 크다"
    );
    let before = image(&core.build_page_render_tree(0).unwrap().root).expect("원본 QR 그림");
    assert!(
        (before.0 / before.1 - common.0 / common.1).abs() < 0.01,
        "합성 틀의 종횡비를 보존한다"
    );
    picture.shape_attr.render_sx = 1.0;
    core.set_document(document);
    let saved_float = image(&core.build_page_render_tree(0).unwrap().root).expect("변형한 QR 그림");
    assert_eq!(
        saved_float, before,
        "저장된 부동 그림은 변환이 달라도 저장 프레임을 쓴다"
    );
}

#[test]
fn explicit_break_keeps_its_empty_line_while_synthetic_break_can_hide_it() {
    use rhwp::model::page::{ColumnDef, PageDef};
    use rhwp::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};
    use rhwp::renderer::composer::compose_paragraph;
    use rhwp::renderer::pagination::{PaginationOpts, Paginator};
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let page = PageDef {
        width: 59528,
        height: 84188,
        margin_left: 8504,
        margin_right: 8504,
        margin_top: 5669,
        margin_bottom: 4252,
        margin_header: 4252,
        margin_footer: 4252,
        ..Default::default()
    };
    let column = ColumnDef::default();
    let styles = ResolvedStyleSet::default();
    let line = LineSeg {
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 800,
        ..Default::default()
    };
    let first = Paragraph {
        text: "앞".into(),
        char_count: 2,
        char_offsets: vec![0],
        line_segs: vec![line.clone()],
        ..Default::default()
    };
    let last = Paragraph {
        text: "뒤".into(),
        char_count: 2,
        char_offsets: vec![0],
        line_segs: vec![line.clone()],
        ..Default::default()
    };
    for kind in [ColumnBreakType::Page, ColumnBreakType::Section] {
        for synthetic in [false, true] {
            for has_stored_line in [false, true] {
                let middle = Paragraph {
                    column_type: kind,
                    page_break_synthesized: synthetic,
                    line_segs: if has_stored_line {
                        vec![line.clone()]
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                };
                let paragraphs = vec![first.clone(), middle, last.clone()];
                let composed: Vec<_> = paragraphs.iter().map(compose_paragraph).collect();
                let paginator = Paginator::with_default_dpi();
                let (_, measured) =
                    paginator.paginate(&paragraphs, &composed, &styles, &page, &column, 0);
                let opts = PaginationOpts {
                    hide_empty_line: true,
                    ..Default::default()
                };
                let result = paginator.paginate_with_measured_opts(
                    &paragraphs,
                    &measured,
                    &page,
                    &column,
                    0,
                    &styles.para_styles,
                    opts,
                );
                assert_eq!(
                    result.hidden_empty_paras.contains(&1),
                    synthetic && has_stored_line,
                    "명시 경계와 합성 경계의 빈 줄 소유"
                );
            }
        }
    }
}

#[test]
fn text_before_inline_picture_stays_on_current_page_and_picture_moves_once() {
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let mut doc = HwpDocument::from_bytes(input).unwrap();
    doc.set_hangul2024_compat(false);
    assert_eq!(doc.page_count(), 6, "독립 한컴 Print의 전체 쪽 수");
    let pages: Vec<Value> = (0..doc.page_count())
        .map(|page| serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap())
        .collect();
    let mut matching_text_pages = Vec::new();
    let mut owner_image_pages = Vec::new();
    for (page, root) in pages.iter().enumerate() {
        let mut nodes = Vec::new();
        walk(root, &mut nodes);
        let text = nodes
            .iter()
            .filter(|n| n["type"] == "TextRun" && n["pi"] == 59)
            .filter_map(|n| n["text"].as_str())
            .collect::<String>();
        if text.contains("합산배제 대상 전통사찰보존지") {
            matching_text_pages.push(page);
        }
        owner_image_pages.extend(
            nodes
                .iter()
                .filter(|n| n["type"] == "Image" && n["pi"] == 59)
                .map(|n| (page, *n)),
        );
    }
    assert_eq!(
        matching_text_pages,
        [1],
        "그림 앞 본문은 이전 쪽에 한 번만 남는다"
    );
    assert_eq!(owner_image_pages.len(), 1, "그림 누락·중복");
    assert_eq!(owner_image_pages[0].0, 2, "그림은 다음 쪽에 이어진다");
    let image = owner_image_pages[0].1;
    let mut nodes = Vec::new();
    walk(&pages[2], &mut nodes);
    let following = nodes
        .iter()
        .find(|n| n["type"] == "TextRun" && n["pi"] == 60)
        .expect("그림 뒤 본문을 보존한다");
    let bottom = image["bbox"]["y"].as_f64().unwrap() + image["bbox"]["h"].as_f64().unwrap();
    assert!(
        bottom <= following["bbox"]["y"].as_f64().unwrap(),
        "그림과 후속 본문은 겹치지 않는다"
    );
}

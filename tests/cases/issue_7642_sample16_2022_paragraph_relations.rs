//! 2022 저장본의 전64쪽 Native/fresh WASM 최저94.14342/94.60213%와 Print 쪽수를 확인했다.
//! 고정 화면 위치 대신 원본 문단·표 소유, 꼬리 내용과 포함·겹침을 검사한다.
//! 표 경계 검사는 source5ac의 실제 출력에서 앞 문단과의 겹침을 검출한다.
//! BCP 3쪽은 양쪽 raw99.48430/99.81839%와 확대 판독 후 독립 기준선도 ±2px로 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwp3-sample16-hwp5-2022.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("독립 Print 대응 원본"))
        .expect("2022 저장본 로드");
    assert_eq!(core.page_count(), 64, "독립 Print 전체 쪽수");
    core
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        // 원본 내용 소유는 논리 문자열로 비교한다. PUA의 표시 대체는 Print로 판독한다.
        return run.text.clone();
    }
    node.children.iter().map(text).collect()
}

fn body_lines<'a>(node: &'a RenderNode, body: bool, out: &mut Vec<&'a RenderNode>) {
    let body = body || matches!(node.node_type, RenderNodeType::Body { .. });
    if body
        && matches!(
            node.node_type,
            RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
        )
    {
        return;
    }
    if body && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node);
    }
    for child in &node.children {
        body_lines(child, body, out);
    }
}

fn owned_lines<'a>(lines: &[&'a RenderNode], owner: (usize, usize)) -> Vec<&'a RenderNode> {
    lines
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
            if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)))
        })
        .collect()
}

#[test]
fn sample16_2022_bcp_tail_keeps_print_line_ownership_and_source_successor() {
    let core = load();
    let sources: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, data)| {
            data.paragraphs
                .iter()
                .enumerate()
                .filter_map(move |(paragraph, para)| {
                    normalized(&para.text)
                        .contains("공사정보처리연속성확보를위한비상대응체계(BCP:BusinessContinuityPlanning)수립")
                        .then_some((section, paragraph, para))
                })
        })
        .collect();
    assert_eq!(sources.len(), 1, "BCP 꼬리 문단의 원본 소유");
    let (section, paragraph, source) = sources[0];
    let page = core.build_page_render_tree(2).expect("Print 3쪽 본문");
    let mut lines = Vec::new();
    body_lines(&page.root, false, &mut lines);
    let bcp = owned_lines(&lines, (section, paragraph));
    // 독립 한컴2020 Print의 실제 경계는 `수|립`이다. 이전 한 줄 기대값은
    // 이 문구만 합치던 예외의 출력을 옮긴 것이므로 원본 저장 줄과 Print를 보호한다.
    // PDF SHA6737b57a, 원본 SHAfec3671c; stage8 C432의 전쪽 TSV·확대 PNG 참조.
    assert_eq!(bcp.len(), 2, "독립 Print의 같은 원본 문단 두 줄 소유");
    let actual_text: String = bcp.iter().map(|line| text(line)).collect();
    assert_eq!(
        normalized(&actual_text),
        normalized(&source.text),
        "원본 내용 순서·누락·중복"
    );
    assert_eq!(
        normalized(&text(bcp[0])),
        normalized(&source.text)
            .strip_suffix('립')
            .expect("원본 꼬리"),
        "Print 첫 줄은 `수`까지 보존한다"
    );
    assert_eq!(normalized(&text(bcp[1])), "립", "Print 둘째 줄의 꼬리 소유");
    assert!(
        bcp[0].bbox.y + bcp[0].bbox.height <= bcp[1].bbox.y,
        "같은 문단 두 줄의 점유가 겹치지 않는다"
    );
    // PDF text origin(기준선)의 pt / 0.75를 CSS px로 환산한다.
    // 잉크 bbox 상단을 글줄 상단과 비교하지 않는다. 192DPI raster는 CSS의 2배다.
    for (line, baseline_css_px) in bcp.iter().zip([903.20, 930.88]) {
        let RenderNodeType::TextLine(owner) = &line.node_type else {
            unreachable!()
        };
        assert!(
            (line.bbox.y + owner.baseline - baseline_css_px).abs() <= 2.0,
            "독립 Print BCP 기준선 ±2 CSS px"
        );
    }
    assert!(
        (bcp[1].bbox.x - 143.36).abs() <= 2.0,
        "독립 Print의 이어지는 글줄 원점 ±2 CSS px"
    );
    let successor = owned_lines(&lines, (section, paragraph + 1));
    assert!(
        !successor.is_empty(),
        "다음 원본 문단도 같은 Print 쪽에 보존"
    );
    assert!(
        bcp[1].bbox.y + bcp[1].bbox.height <= successor[0].bbox.y,
        "본문 꼬리와 다음 문단이 겹치지 않는다"
    );
    let source_next = &core.document().sections[section].paragraphs[paragraph + 1];
    let actual_next: String = successor.iter().map(|node| text(node)).collect();
    assert_eq!(normalized(&actual_next), normalized(&source_next.text));
}

fn tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node);
    }
    for child in &node.children {
        tables(child, out);
    }
}

fn lines_in<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node);
    }
    for child in &node.children {
        lines_in(child, out);
    }
}

#[test]
fn sample16_2022_continuation_table_clears_predecessor_and_preserves_owned_rows() {
    let core = load();
    let mut sources = Vec::new();
    for (section, data) in core.document().sections.iter().enumerate() {
        for (paragraph, para) in data.paragraphs.iter().enumerate() {
            for (control, item) in para.controls.iter().enumerate() {
                let Control::Table(table) = item else {
                    continue;
                };
                if table
                    .cells
                    .iter()
                    .flat_map(|cell| &cell.paragraphs)
                    .any(|p| normalized(&p.text).contains("기술제안서제출수량"))
                {
                    sources.push((section, paragraph, control, table));
                }
            }
        }
    }
    assert_eq!(sources.len(), 1, "제출 수량 표의 원본 소유");
    let (section, paragraph, control, source) = sources[0];
    assert!(source.common.treat_as_char, "본문 흐름에 참여하는 원본 표");
    let page = core.build_page_render_tree(6).expect("Print 7쪽 이어받기");
    let mut found = Vec::new();
    tables(&page.root, &mut found);
    let found: Vec<_> = found
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if (table.section_index, table.para_index, table.control_index)
                == (Some(section), Some(paragraph), Some(control)))
        })
        .collect();
    assert_eq!(found.len(), 1, "표 소유·누락·중복");
    let table = found[0];
    let mut body = Vec::new();
    body_lines(&page.root, false, &mut body);
    let previous = owned_lines(&body, (section, paragraph.checked_sub(1).expect("앞 문단")));
    assert!(!previous.is_empty());
    let previous_bottom = previous
        .iter()
        .map(|line| line.bbox.y + line.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        table.bbox.y >= previous_bottom,
        "이어받은 표가 앞 문단의 실제 점유를 침범한다"
    );
    let mut rows = Vec::new();
    lines_in(table, &mut rows);
    assert_eq!(rows.len(), 3, "독립 Print의 제출 수량·CD·작성 기준 세 줄");
    let source_text: String = source
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(
        normalized(&text(table)),
        normalized(&source_text),
        "표 내용 순서·보존"
    );
    for pair in rows.windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "표 내부 줄 겹침"
        );
    }
    for line in rows {
        let roundoff = f64::EPSILON * table.bbox.width.max(table.bbox.height).max(1.0) * 128.0;
        assert!(
            line.bbox.x + roundoff >= table.bbox.x
                && line.bbox.y + roundoff >= table.bbox.y
                && line.bbox.x + line.bbox.width <= table.bbox.x + table.bbox.width + roundoff
                && line.bbox.y + line.bbox.height <= table.bbox.y + table.bbox.height + roundoff,
            "원본 표가 내부 줄의 실제 점유를 포함한다"
        );
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn sample16_2022_compound_page_border_keeps_both_print_coverage_bands() {
    let core = load();
    // 같은 source/Print 전64쪽: Native/fresh WASM 최저94.59411/94.60213%.
    // 63쪽99.80596/99.80728%, 64쪽99.66659/99.67064%를 확대 판독했다.
    // source SHAfec3671c / Print SHA6737b57a; stage8 C433의 전체 TSV·PNG 참조.
    for (page_index, owner_text) in [
        (62, "공동수급체는공동수급체구성원을위원으로하는운영위원회를"),
        (63, "합의각서"),
    ] {
        let sources: Vec<_> = core
            .document()
            .sections
            .iter()
            .enumerate()
            .flat_map(|(section, data)| {
                data.paragraphs
                    .iter()
                    .enumerate()
                    .filter_map(move |(paragraph, source)| {
                        (if page_index == 63 {
                            normalized(&source.text) == owner_text
                        } else {
                            normalized(&source.text).contains(owner_text)
                        })
                        .then_some((section, paragraph, source))
                    })
            })
            .collect();
        assert_eq!(sources.len(), 1, "같은 Print 쪽 본문 문단의 원본 소유");
        let (section, paragraph, source) = sources[0];
        let id = core.document().sections[section]
            .section_def
            .page_border_fill
            .border_fill_id;
        let border = &core.document().doc_info.border_fills
            [usize::from(id.checked_sub(1).expect("원본 쪽 테두리"))];
        assert_eq!(
            border.borders[3].line_type,
            rhwp::model::style::BorderLineType::Double,
            "원본 아래 테두리는 이중선"
        );
        let tree = core.build_page_render_tree(page_index).unwrap();
        let mut lines = Vec::new();
        body_lines(&tree.root, false, &mut lines);
        let body = owned_lines(&lines, (section, paragraph));
        assert!(!body.is_empty(), "같은 Print 쪽에 원본 문단이 보존된다");
        let actual: String = body.iter().map(|line| text(line)).collect();
        assert_eq!(
            normalized(&actual),
            normalized(&source.text),
            "같은 Print 쪽 본문 내용 순서·누락·중복"
        );

        // 독립 Print63/64쪽의 아래 이중선: 192DPI 2127/2128행과2130/2131행.
        // 원본 중간의 빈 띠와 각 획의 이웃 픽셀 coverage를 보존한다.
        // 이중선을 분해한 뒤 각 획을 픽셀 중심에 재배치하면 한 행씩으로 줄어 실패한다.
        let bytes = core
            .render_page_png_native_with_profile_and_export_options(
                page_index,
                rhwp::paint::RenderProfile::Print,
                &rhwp::document_core::queries::rendering::PngExportOptions {
                    dpi: Some(192.0),
                    scale: Some(2.0),
                    ..Default::default()
                },
            )
            .expect("Native Print PNG");
        let pixels = image::load_from_memory(&bytes).unwrap().to_rgb8();
        let x = pixels.width() / 2;
        let mut bands: Vec<Vec<u32>> = Vec::new();
        // 독립 PDF 좌표 ±2 CSSpx = ±4 raster pixels를 포함한다.
        for y in 2123..=2135 {
            if pixels
                .get_pixel(x, y)
                .0
                .into_iter()
                .any(|value| value < 232)
            {
                if bands
                    .last()
                    .and_then(|band| band.last())
                    .is_some_and(|last| *last + 1 == y)
                {
                    bands.last_mut().unwrap().push(y);
                } else {
                    bands.push(vec![y]);
                }
            }
        }
        assert_eq!(
            bands.len(),
            2,
            "독립 Print의 분리된 두 coverage 띠: {bands:?}"
        );
        for (band, center_css) in bands.iter().zip([1063.75_f64, 1065.25]) {
            assert!(
                band.len() >= 2,
                "이중선 각 획은 독립 Print처럼 이웃 픽셀에 coverage를 보존한다: {bands:?}"
            );
            let actual_css = f64::from(band[0] + band[band.len() - 1]) / 4.0;
            assert!(
                (actual_css - center_css).abs() <= 2.0,
                "독립 Print 선 띠 중심 ±2 CSSpx"
            );
        }
    }
}

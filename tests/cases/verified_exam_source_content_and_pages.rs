//! 시험지 원본 IR의 수식 보존·소유와 독립 Print의 본문 위치를 검사한다.
//! 과거 시험지 공용 corpus와 독립적인 새 이름으로 자동 suite에 등록한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::{control::Control, document::Document, paragraph::Paragraph, shape::ShapeObject};
use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const DATA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/verified_exam_source_pages.tsv"
));
fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}
#[derive(Default)]
struct Original {
    equations: BTreeMap<String, usize>,
    probe_paragraphs: usize,
}
fn source_shape(shape: &ShapeObject, probe: &str, original: &mut Original) {
    if let Some(text) = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
    {
        source_paragraphs(&text.paragraphs, probe, original);
    }
    if let ShapeObject::Group(group) = shape {
        for child in &group.children {
            source_shape(child, probe, original);
        }
    }
}
fn source_paragraphs(paragraphs: &[Paragraph], probe: &str, original: &mut Original) {
    for paragraph in paragraphs {
        if compact(&paragraph.text).contains(probe) {
            original.probe_paragraphs += 1;
        }
        for control in &paragraph.controls {
            match control {
                Control::Equation(equation) if !equation.script.trim().is_empty() => {
                    *original
                        .equations
                        .entry(equation.script.trim().to_owned())
                        .or_default() += 1;
                }
                Control::Table(table) => {
                    for cell in &table.cells {
                        source_paragraphs(&cell.paragraphs, probe, original);
                    }
                }
                Control::Endnote(note) => source_paragraphs(&note.paragraphs, probe, original),
                Control::Footnote(note) => source_paragraphs(&note.paragraphs, probe, original),
                Control::Shape(shape) => source_shape(shape, probe, original),
                // 반복되는 머리말·꼬리말과 인쇄하지 않는 숨은 설명은 본문 집합이 아니다.
                _ => {}
            }
        }
    }
}
fn equation_owner<'a>(
    document: &'a Document,
    source_map: &SourceMap,
    equation: &rhwp::renderer::render_tree::EquationNode,
) -> &'a Paragraph {
    if let Some(note) = &equation.note_ref {
        let host = &document.sections[note.section_index].paragraphs[note.para_index].controls
            [note.control_index];
        return match (note.kind.as_str(), host) {
            ("endnote", Control::Endnote(source)) => &source.paragraphs[note.note_para_index],
            ("footnote", Control::Footnote(source)) => &source.paragraphs[note.note_para_index],
            _ => panic!("각주·미주 종류와 원본 소유가 다르다"),
        };
    }
    let section = equation.section_index.expect("수식의 원본 구역");
    let paragraph = equation.para_index.expect("수식의 원본 문단");
    if let Some(&(si, pi, ci, ni)) = source_map.get(&(section, paragraph)) {
        let Control::Endnote(note) = &document.sections[si].paragraphs[pi].controls[ci] else {
            panic!("덤프의 미주 원본 컨트롤");
        };
        return &note.paragraphs[ni];
    }
    &document.sections[section].paragraphs[paragraph]
}
// 미주 표 안 수식은 편집 note_ref가 없으므로 공개 pagination 원본 연결을 사용한다.
// 합성 배치 인덱스를 원본 본문 인덱스로 간주하지 않는다.
type SourceMap = BTreeMap<(usize, usize), (usize, usize, usize, usize)>;
fn endnote_sources(core: &DocumentCore) -> SourceMap {
    let mut sources = SourceMap::new();
    for page in core.dump_page_items_json(None).as_array().unwrap() {
        let section = page["section"].as_u64().unwrap() as usize;
        for column in page["columns"].as_array().unwrap() {
            for item in column["items"].as_array().unwrap() {
                let source = &item["endnoteSource"];
                if source.is_object() {
                    let value = |key: &str| source[key].as_u64().unwrap() as usize;
                    let key = (section, item["paraIndex"].as_u64().unwrap() as usize);
                    let address = (
                        value("section"),
                        value("para"),
                        value("controlIndex"),
                        value("noteParaIndex"),
                    );
                    if let Some(previous) = sources.insert(key, address) {
                        assert_eq!(previous, address);
                    }
                }
            }
        }
    }
    sources
}
type NoteOrder = BTreeMap<String, Vec<(usize, usize, usize)>>;
fn original_note_order(document: &Document) -> NoteOrder {
    let mut expected = NoteOrder::new();
    for (section, data) in document.sections.iter().enumerate() {
        for (paragraph, value) in data.paragraphs.iter().enumerate() {
            for (control, item) in value.controls.iter().enumerate() {
                let note = match item {
                    Control::Endnote(note) => Some(("endnote", note.paragraphs.as_slice())),
                    Control::Footnote(note) => Some(("footnote", note.paragraphs.as_slice())),
                    _ => None,
                };
                if let Some((kind, paragraphs)) = note {
                    let mut source = Original::default();
                    source_paragraphs(paragraphs, "", &mut source);
                    if !source.equations.is_empty() {
                        expected
                            .entry(kind.to_owned())
                            .or_default()
                            .push((section, paragraph, control));
                    }
                }
            }
        }
    }
    expected
}
#[derive(Clone)]
struct Character {
    value: char,
    x: f64,
    owner: Option<(usize, usize)>,
}
fn line_characters(node: &RenderNode, output: &mut Vec<Character>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.rotation != 0.0 || run.is_vertical {
            return;
        }
        let text = run.display_or_text();
        let measured = EmbeddedTextMeasurer.compute_char_positions(text, &run.style);
        let positions = run
            .layout_positions
            .as_ref()
            .filter(|positions| positions.len() == text.chars().count() + 1)
            .unwrap_or(&measured);
        for (index, value) in text.chars().enumerate() {
            if !value.is_whitespace() && !value.is_control() && value != '\u{fffc}' {
                output.push(Character {
                    value,
                    x: node.bbox.x + positions[index],
                    owner: run.section_index.zip(
                        run.cell_context
                            .as_ref()
                            .map(|context| context.parent_para_index)
                            .or(run.para_index),
                    ),
                });
            }
        }
        return;
    }
    for child in &node.children {
        if !matches!(
            child.node_type,
            RenderNodeType::TextLine(_)
                | RenderNodeType::Table(_)
                | RenderNodeType::TableCell(_)
                | RenderNodeType::TextBox
        ) {
            line_characters(child, output);
        }
    }
}
#[derive(Default)]
struct Observed {
    equations: BTreeMap<String, usize>,
    anchors: Vec<(u32, Vec<Character>)>,
    notes: NoteOrder,
}
fn rendered(
    node: &RenderNode,
    document: &Document,
    probe: &str,
    page: u32,
    source_map: &SourceMap,
    observed: &mut Observed,
) {
    if matches!(
        node.node_type,
        RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage
    ) {
        return;
    }
    if let RenderNodeType::Equation(equation) = &node.node_type {
        if !equation.script.trim().is_empty() {
            if let Some(note) = &equation.note_ref {
                let key = (note.section_index, note.para_index, note.control_index);
                let order = observed.notes.entry(note.kind.clone()).or_default();
                if !order.contains(&key) {
                    order.push(key);
                }
            }
            if equation.note_ref.is_none() {
                if let Some(&(si, pi, ci, _)) = source_map.get(&(
                    equation.section_index.unwrap(),
                    equation.para_index.unwrap(),
                )) {
                    let order = observed.notes.entry("endnote".to_owned()).or_default();
                    let key = (si, pi, ci);
                    if !order.contains(&key) {
                        order.push(key);
                    }
                }
            }
            let source = equation_owner(document, source_map, equation);
            let mut owned = Original::default();
            source_paragraphs(std::slice::from_ref(source), probe, &mut owned);
            assert!(
                owned.equations.contains_key(equation.script.trim()),
                "수식은 해당 원본 본문/각주/미주 문단에 있어야 한다"
            );
            assert!(equation.control_index.is_some(), "원본 control 소유 연결");
            *observed
                .equations
                .entry(equation.script.trim().to_owned())
                .or_default() += 1;
        }
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut chars = Vec::new();
        line_characters(node, &mut chars);
        let text: String = chars.iter().map(|c| c.value).collect();
        for (offset, _) in text.match_indices(probe) {
            let start = text[..offset].chars().count();
            observed
                .anchors
                .push((page, chars[start..start + probe.chars().count()].to_vec()));
        }
    }
    for child in &node.children {
        rendered(child, document, probe, page, source_map, observed);
    }
}
fn check(id: &str) {
    let mut lines = DATA.lines();
    let header: Vec<_> = lines.next().unwrap().split('\t').collect();
    let row: Vec<_> = lines
        .find(|line| line.split('\t').next() == Some(id))
        .expect("독립 Print 증거 행")
        .split('\t')
        .collect();
    let field = |key: &str| row[header.iter().position(|name| *name == key).unwrap()];
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(field("input"));
    let bytes = std::fs::read(input).unwrap();
    assert_eq!(
        Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        field("input_sha256"),
        "검증한 불변 원본"
    );
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(field("compat") == "2024");
    let pages: u32 = field("pages").parse().unwrap();
    assert_eq!(core.page_count(), pages, "같은 원본 독립 Print 전체 쪽수");
    let probe = field("probe");
    let mut original = Original::default();
    for section in &core.document().sections {
        source_paragraphs(&section.paragraphs, probe, &mut original);
    }
    assert!(
        original.probe_paragraphs > 0,
        "위치가 아니라 원본 내용으로 선택한 본문"
    );
    let mut observed = Observed::default();
    let source_map = endnote_sources(&core);
    for page in 0..pages {
        let tree = core.build_page_render_tree(page).unwrap();
        rendered(
            &tree.root,
            core.document(),
            probe,
            page + 1,
            &source_map,
            &mut observed,
        );
    }
    let Observed {
        equations,
        anchors,
        notes,
    } = observed;
    if equations != original.equations {
        let differences: Vec<_> = original
            .equations
            .iter()
            .filter(|(script, count)| equations.get(*script) != Some(*count))
            .take(8)
            .collect();
        panic!(
            "원본 IR 수식 집합의 전쪽 누락·중복: 원본{} / 출력{}, 차이 표본{differences:?}",
            original.equations.values().sum::<usize>(),
            equations.values().sum::<usize>()
        );
    }
    assert_eq!(
        notes,
        original_note_order(core.document()),
        "각주·미주별 원본 소유와 전쪽 읽기 순서"
    );
    assert_eq!(anchors.len(), 1, "독립 Print의 고유 본문 누락·중복 방지");
    let (page, chars) = &anchors[0];
    assert_eq!(
        *page,
        field("page").parse::<u32>().unwrap(),
        "독립 Print 본문 쪽 소속"
    );
    assert!(
        chars.iter().all(|c| c.owner.is_some()),
        "본문의 구역·문단 소유 연결"
    );
    for &(section, paragraph) in &chars
        .iter()
        .map(|c| c.owner.unwrap())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let document = core.document();
        let owner = if let Some(&(si, pi, ci, ni)) = source_map.get(&(section, paragraph)) {
            let Control::Endnote(note) = &document.sections[si].paragraphs[pi].controls[ci] else {
                panic!("미주 본문 소유");
            };
            &note.paragraphs[ni]
        } else {
            &document.sections[section].paragraphs[paragraph]
        };
        let mut found = Original::default();
        source_paragraphs(std::slice::from_ref(owner), probe, &mut found);
        assert!(
            found.probe_paragraphs > 0,
            "표시한 고유 본문의 문단 소유가 원본 내용과 일치해야 한다: section={section}, paragraph={paragraph}, source={:?}, probe={probe:?}", owner.text
        );
    }
    assert!(
        chars.windows(2).all(|pair| pair[0].x < pair[1].x),
        "원본 문자열의 전진 순서"
    );
    if field("px_guard").eq_ignore_ascii_case("true") {
        assert!(
            field("native_raw").parse::<f64>().unwrap() >= 98.0
                && field("wasm_raw").parse::<f64>().unwrap() >= 98.0
        );
        let page_index = field("page").parse::<u32>().unwrap() - 1;
        let tree = core.build_page_render_tree(page_index).unwrap();
        let vertical = field("boundary_vertical").eq_ignore_ascii_case("true");
        let mut boundaries = Vec::new();
        fn collect(node: &RenderNode, output: &mut Vec<[f64; 4]>) {
            if matches!(
                node.node_type,
                RenderNodeType::Line(_) | RenderNodeType::Rectangle(_) | RenderNodeType::Path(_)
            ) {
                let b = node.bbox;
                output.push([b.x, b.y, b.x + b.width, b.y + b.height]);
            }
            // 문단 테두리는 Rectangle의 네 변으로 그린다. 선 노드만 찾으면 놓친다.
            if let RenderNodeType::Rectangle(rectangle) = &node.node_type {
                if rectangle.style.stroke_color.is_some() && rectangle.style.stroke_width > 0.0 {
                    let b = node.bbox;
                    output.extend([
                        [b.x, b.y, b.x + b.width, b.y],
                        [b.x, b.y + b.height, b.x + b.width, b.y + b.height],
                        [b.x, b.y, b.x, b.y + b.height],
                        [b.x + b.width, b.y, b.x + b.width, b.y + b.height],
                    ]);
                }
            }
            for child in &node.children {
                collect(child, output);
            }
        }
        collect(&tree.root, &mut boundaries);
        let width = tree.root.bbox.width;
        let height = tree.root.bbox.height;
        let boundary = if vertical {
            boundaries
                .into_iter()
                .filter(|b| (b[2] - b[0]).abs() <= 2.0 && b[3] - b[1] >= height / 2.0)
                .min_by(|a, b| {
                    (((a[0] + a[2]) / 2.0 - width / 2.0).abs())
                        .total_cmp(&((b[0] + b[2]) / 2.0 - width / 2.0).abs())
                        .then_with(|| (b[3] - b[1]).total_cmp(&(a[3] - a[1])))
                })
        } else {
            boundaries
                .into_iter()
                .filter(|b| (b[3] - b[1]).abs() <= 2.0 && b[2] - b[0] >= width / 3.0)
                .min_by(|a, b| {
                    a[1].total_cmp(&b[1])
                        .then_with(|| (b[2] - b[0]).total_cmp(&(a[2] - a[0])))
                })
        }
        .expect("독립 Print와 같은 단 구분선 또는 본문 상단 경계");
        let coordinate = |name: &str| field(name).parse::<f64>().unwrap();
        let expected = [
            coordinate("boundary_x0"),
            coordinate("boundary_y0"),
            coordinate("boundary_x1"),
            coordinate("boundary_y1"),
        ];
        let (actual, independent) = if vertical {
            (
                [(boundary[0] + boundary[2]) / 2.0, boundary[1]],
                [(expected[0] + expected[2]) / 2.0, expected[1]],
            )
        } else {
            (
                [boundary[0], (boundary[1] + boundary[3]) / 2.0],
                [expected[0], (expected[1] + expected[3]) / 2.0],
            )
        };
        for (observed, reference) in actual.into_iter().zip(independent) {
            assert!(
                (observed - reference).abs() <= 2.0,
                "완료 페이지 독립 Print 구조 경계: 실제{observed} / 독립{reference}"
            );
        }
    }
}

#[test]
fn exam_all_01_source_equations_and_print_page() {
    check("all-01");
}

#[test]
fn exam_all_02_source_equations_and_print_page() {
    check("all-02");
}

#[test]
fn exam_all_03_source_equations_and_print_page() {
    check("all-03");
}

#[test]
fn exam_all_04_source_equations_and_print_page() {
    check("all-04");
}

#[test]
fn exam_all_05_source_equations_and_print_page() {
    check("all-05");
}

#[test]
fn exam_all_06_source_equations_and_print_page() {
    check("all-06");
}

#[test]
fn exam_all_07_source_equations_and_print_page() {
    check("all-07");
}

#[test]
fn exam_all_08_source_equations_and_print_page() {
    check("all-08");
}

#[test]
fn exam_all_09_source_equations_and_print_page() {
    check("all-09");
}

#[test]
fn exam_all_10_source_equations_and_print_page() {
    check("all-10");
}

#[test]
fn exam_all_11_source_equations_and_print_page() {
    check("all-11");
}

#[test]
fn exam_all_12_source_equations_and_print_page() {
    check("all-12");
}

#[test]
fn exam_all_13_source_equations_and_print_page() {
    check("all-13");
}

#[test]
fn exam_all_14_source_equations_and_print_page() {
    check("all-14");
}

#[test]
fn exam_all_15_source_equations_and_print_page() {
    check("all-15");
}

#[test]
fn exam_all_16_source_equations_and_print_page() {
    check("all-16");
}

#[test]
fn exam_all_17_source_equations_and_print_page() {
    check("all-17");
}

#[test]
fn exam_all_18_source_equations_and_print_page() {
    check("all-18");
}

#[test]
fn exam_all_19_source_equations_and_print_page() {
    check("all-19");
}

#[test]
fn exam_all_20_source_equations_and_print_page() {
    check("all-20");
}

#[test]
fn exam_all_21_source_equations_and_print_page() {
    check("all-21");
}

#[test]
fn exam_all_22_source_equations_and_print_page() {
    check("all-22");
}

#[test]
fn exam_all_23_source_equations_and_print_page() {
    check("all-23");
}

#[test]
fn exam_all_24_source_equations_and_print_page() {
    check("all-24");
}

#[test]
fn exam_all_25_source_equations_and_print_page() {
    check("all-25");
}

#[test]
fn exam_all_26_source_equations_and_print_page() {
    check("all-26");
}

#[test]
fn exam_all_27_source_equations_and_print_page() {
    check("all-27");
}

#[test]
fn exam_all_28_source_equations_and_print_page() {
    check("all-28");
}

#[test]
fn exam_all_29_source_equations_and_print_page() {
    check("all-29");
}

#[test]
fn exam_all_30_source_equations_and_print_page() {
    check("all-30");
}

#[test]
fn exam_all_31_source_equations_and_print_page() {
    check("all-31");
}

#[test]
fn exam_all_32_source_equations_and_print_page() {
    check("all-32");
}

#[test]
fn exam_all_33_source_equations_and_print_page() {
    check("all-33");
}

#[test]
fn exam_all_34_source_equations_and_print_page() {
    check("all-34");
}

#[test]
fn exam_all_35_source_equations_and_print_page() {
    check("all-35");
}

#[test]
fn exam_all_36_source_equations_and_print_page() {
    check("all-36");
}

#[test]
fn exam_all_37_source_equations_and_print_page() {
    check("all-37");
}

#[test]
fn exam_all_38_source_equations_and_print_page() {
    check("all-38");
}

#[test]
fn exam_all_39_source_equations_and_print_page() {
    check("all-39");
}

#[test]
fn exam_all_40_source_equations_and_print_page() {
    check("all-40");
}

#[test]
fn exam_all_41_source_equations_and_print_page() {
    check("all-41");
}

#[test]
fn exam_all_42_source_equations_and_print_page() {
    check("all-42");
}

#[test]
fn exam_all_43_source_equations_and_print_page() {
    check("all-43");
}

#[test]
fn exam_all_44_source_equations_and_print_page() {
    check("all-44");
}

#[test]
fn exam_all_45_source_equations_and_print_page() {
    check("all-45");
}

#[test]
fn exam_all_46_source_equations_and_print_page() {
    check("all-46");
}

#[test]
fn exam_all_47_source_equations_and_print_page() {
    check("all-47");
}

#[test]
fn exam_all_48_source_equations_and_print_page() {
    check("all-48");
}

#[test]
fn exam_all_49_source_equations_and_print_page() {
    check("all-49");
}

#[test]
fn exam_all_50_source_equations_and_print_page() {
    check("all-50");
}

#[test]
fn exam_all_51_source_equations_and_print_page() {
    check("all-51");
}

#[test]
fn exam_all_52_source_equations_and_print_page() {
    check("all-52");
}

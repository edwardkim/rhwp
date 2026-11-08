//! #8160: sample16 회귀 검사 통합. 이전 등록: tests/cases/verified_hwp3_original_print_boundaries.rs
//! HWP3 원본의 제목·확장 제어·점선·목차를 독립 Print의 소유와 중요 경계로 보호한다.
//! 전 64쪽 Native 최저 92.25853%, fresh WASM 최저 95.76171%, 쪽수 일치.
//! 3쪽은 각각 99.94303%, 99.92693%이며 같은 배율의 확대 판독을 완료했다.
//! 96dpi 좌상단·독립 좌표·SHA·TSV·PNG: mydocs/working/task_m100_7642_stage1533.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, shape::ShapeObject};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::StrokeDash;
use rhwp::DocumentCore;

type Owner = (usize, usize);

fn load() -> DocumentCore {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16.hwp"),
    )
    .unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 64, "독립 원본 Print 전체 쪽수");
    core
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}

fn owner(core: &DocumentCore, title: &str) -> Owner {
    let mut owners = Vec::new();
    for (section, s) in core.document().sections.iter().enumerate() {
        for (paragraph, p) in s.paragraphs.iter().enumerate() {
            if normalized(&p.text) == title {
                owners.push((section, paragraph));
            }
        }
    }
    assert_eq!(owners.len(), 1, "원문의 고유한 본문 제목: {title}");
    owners[0]
}

fn text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        text(child, out);
    }
}

fn lines<'a>(node: &'a RenderNode, source: Owner, found: &mut Vec<&'a RenderNode>) {
    if matches!(
        node.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line) if (line.section_index,line.para_index)==(Some(source.0),Some(source.1)))
    {
        found.push(node);
    }
    for child in &node.children {
        lines(child, source, found);
    }
}

fn one_line(node: &RenderNode, source: Owner) -> &RenderNode {
    let mut found = Vec::new();
    lines(node, source, &mut found);
    assert_eq!(found.len(), 1, "단일 제목의 누락·중복 방지");
    found[0]
}

fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {label} ±2 CSS px: {actual} / {expected}"
    );
}

fn token_x(core: &DocumentCore, source: Owner, token: &str) -> f64 {
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(2).unwrap()).unwrap();
    let mut characters = Vec::new();
    for run in layout["runs"].as_array().unwrap() {
        if run["secIdx"].as_u64() != Some(source.0 as u64)
            || run["paraIdx"].as_u64() != Some(source.1 as u64)
            || !run["cellContext"].is_null()
        {
            continue;
        }
        let content = run["text"].as_str().unwrap();
        let positions = run["charX"].as_array().unwrap();
        assert_eq!(
            positions.len(),
            content.chars().count() + 1,
            "가시 문자·제어 슬롯의 원점 배열"
        );
        for (c, x) in content.chars().zip(positions) {
            characters.push((c, run["x"].as_f64().unwrap() + x.as_f64().unwrap()));
        }
    }
    let content: Vec<_> = characters.iter().map(|(c, _)| *c).collect();
    let token: Vec<_> = token.chars().collect();
    let matches: Vec<_> = content
        .windows(token.len())
        .enumerate()
        .filter_map(|(i, s)| (s == token).then_some(i))
        .collect();
    assert_eq!(matches.len(), 1, "원문 토큰의 유일한 배치");
    characters[matches[0]].1
}

#[test]
fn issue_8160_original_hwp3_heading_keeps_authored_space_and_extended_control_axis() {
    let core = load();
    let (section, paragraph) = owner(&core, "1.추진목적");
    let source = &core.document().sections[section].paragraphs[paragraph];
    let authored: String = source.text.chars().filter(|c| *c != '\u{fffc}').collect();
    assert_eq!(
        authored, "1. 추진목적",
        "원문 hchar와 독립 Print의 작성 공백"
    );
    assert_eq!(
        source.char_offsets.len(),
        source.text.chars().count(),
        "합성 장식 축을 원문에 붙이지 않음"
    );
    assert_eq!(
        source.controls.len(),
        3,
        "원문의 책갈피·새번호·쪽번호 위치 보존"
    );
    assert!(source
        .controls
        .iter()
        .any(|c| matches!(c, Control::Bookmark(_))));
    assert!(source
        .controls
        .iter()
        .any(|c| matches!(c, Control::NewNumber(_))));
    assert!(source
        .controls
        .iter()
        .any(|c| matches!(c, Control::PageNumberPos(_))));
    let mut expected = 0;
    for (c, &offset) in source.text.chars().zip(&source.char_offsets) {
        assert_eq!(offset, expected, "원문의 문자와 확장 제어 위치");
        // 공통 IR의 확장 제어는 원문 문자와 구분한 8 UTF-16 유닛이다.
        expected += if c == '\u{fffc}' {
            8
        } else {
            c.len_utf16() as u32
        };
    }
    assert_eq!(source.char_count, expected + 1, "문단 끝 마커 포함");
    let page = core.build_page_render_tree(2).unwrap();
    let line = one_line(&page.root, (section, paragraph));
    let mut rendered = String::new();
    text(line, &mut rendered);
    assert_eq!(
        normalized(&rendered),
        normalized(&authored),
        "제목의 누락·중복·장식 주입 방지"
    );
    near(
        token_x(&core, (section, paragraph), "추진목적"),
        109.9199930826823,
        "번호 뒤 제목 시작",
    );
}

#[test]
fn issue_8160_original_hwp3_completed_page_keeps_print_headings_and_owned_reading_order() {
    let core = load();
    let sources = [
        owner(&core, "1.추진목적"),
        owner(&core, "2.추진방향"),
        owner(&core, "3.주요추진내용"),
    ];
    for source in sources {
        let mut placements = Vec::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut found = Vec::new();
            lines(&tree.root, source, &mut found);
            placements.extend(std::iter::repeat_n(page, found.len()));
        }
        assert_eq!(placements, vec![2], "독립 Print 3쪽의 제목 소유·누락·중복");
    }
    let page = core.build_page_render_tree(2).unwrap();
    let mut previous = None;
    for (source, baseline) in
        sources
            .into_iter()
            .zip([159.19999186197916, 359.0399983723958, 771.199951171875])
    {
        let line = one_line(&page.root, source);
        let mut rendered = String::new();
        text(line, &mut rendered);
        assert_eq!(
            normalized(&rendered),
            normalized(&core.document().sections[source.0].paragraphs[source.1].text)
        );
        let mut seen = false;
        for child in &line.children {
            if let RenderNodeType::TextRun(run) = &child.node_type {
                if run
                    .text
                    .chars()
                    .any(|c| !c.is_whitespace() && c != '\u{fffc}')
                {
                    near(child.bbox.y + run.baseline, baseline, "제목 기준선");
                    seen = true;
                }
            }
        }
        assert!(seen, "가시 제목이 없는 줄의 통과 금지");
        near(line.bbox.x, 83.36000061035156, "제목 번호 시작");
        if let Some(end) = previous {
            assert!(end < line.bbox.y, "제목과 후속 내용의 읽기 순서·비겹침");
        }
        previous = Some(line.bbox.y + line.bbox.height);
    }
}

fn rectangles<'a>(
    node: &'a RenderNode,
    source: (usize, usize, usize),
    found: &mut Vec<&'a RenderNode>,
) {
    if matches!(&node.node_type,RenderNodeType::Rectangle(r) if (r.section_index,r.para_index,r.control_index)==(Some(source.0),Some(source.1),Some(source.2)) && r.cell_index.is_none() && r.cell_para_index.is_none() && r.outer_table_control_index.is_none())
    {
        found.push(node);
    }
    for child in &node.children {
        rectangles(child, source, found);
    }
}

#[test]
fn issue_8160_original_hwp3_completed_page_keeps_dotted_business_frame_and_its_text() {
    let core = load();
    let heading = owner(&core, "1.추진목적");
    let next = owner(&core, "2.추진방향");
    assert_eq!(heading.0, next.0);
    let mut sources = Vec::new();
    for (paragraph, p) in core.document().sections[heading.0]
        .paragraphs
        .iter()
        .enumerate()
        .take(next.1)
        .skip(heading.1 + 1)
    {
        for (control, c) in p.controls.iter().enumerate() {
            if let Control::Shape(shape) = c {
                if let ShapeObject::Rectangle(r) = shape.as_ref() {
                    sources.push((paragraph, control, r));
                }
            }
        }
    }
    assert_eq!(sources.len(), 1, "두 제목 사이 원문 도형의 소유");
    let (paragraph, control, source) = sources[0];
    assert_eq!(
        source.drawing.border_line.attr as u32 & 0x3f,
        3,
        "독립 Print에 보이는 점선의 공통 IR 종류"
    );
    let mut placements = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut found = Vec::new();
        rectangles(&tree.root, (heading.0, paragraph, control), &mut found);
        placements.extend(std::iter::repeat_n(page, found.len()));
    }
    assert_eq!(placements, vec![2], "원문 도형의 쪽 소유·누락·중복");
    let page = core.build_page_render_tree(2).unwrap();
    let mut found = Vec::new();
    rectangles(&page.root, (heading.0, paragraph, control), &mut found);
    let frame = found[0];
    let RenderNodeType::Rectangle(rect) = &frame.node_type else {
        unreachable!()
    };
    assert!(
        matches!(rect.style.stroke_dash, StrokeDash::HwpDot),
        "점선의 최종 paint 계약을 실선으로 바꾸지 않음"
    );
    assert!(
        rect.style.stroke_width > 0.0 && rect.gradient.is_some(),
        "선과 원문 그라데이션 보존"
    );
    let boxes: Vec<_> = frame
        .children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextBox))
        .collect();
    assert_eq!(boxes.len(), 1, "도형에 소속된 원문 글상자");
    let content = boxes[0];
    assert!(
        frame.bbox.x <= content.bbox.x
            && frame.bbox.y <= content.bbox.y
            && content.bbox.x + content.bbox.width <= frame.bbox.x + frame.bbox.width
            && content.bbox.y + content.bbox.height <= frame.bbox.y + frame.bbox.height,
        "글상자의 도형 내부 포함"
    );
    let expected: String = source
        .drawing
        .text_box
        .as_ref()
        .unwrap()
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect();
    let mut actual = String::new();
    text(content, &mut actual);
    assert_eq!(
        normalized(&actual),
        normalized(&expected),
        "도형 내부 원문의 순서·누락·중복"
    );
    assert!(
        one_line(&page.root, heading).bbox.y + one_line(&page.root, heading).bbox.height
            <= frame.bbox.y
            && frame.bbox.y + frame.bbox.height <= one_line(&page.root, next).bbox.y,
        "제목·도형·다음 제목의 순서·비겹침"
    );
    // 독립 PDF의 검정 점선 윤곽은 채워진 벡터 경로 2049개 구간으로 출력됐다.
    for (actual, expected) in [
        frame.bbox.x,
        frame.bbox.y,
        frame.bbox.width,
        frame.bbox.height,
    ]
    .into_iter()
    .zip([
        102.98399861653645,
        174.68937174479166,
        612.1519877115885,
        130.89599609375,
    ]) {
        near(actual, expected, "점선 박스 중요 외곽");
    }
}

#[test]
fn issue_8160_original_hwp3_toc_preserves_owned_page_numbers_and_relative_right_alignment() {
    let core = load();
    let start = owner(&core, "목차");
    let end = owner(&core, "1.추진목적");
    assert_eq!(start.0, end.0);
    // 독립 Print 2쪽 목차의 26개 페이지 번호다. 별첨 제목의 1·2는 페이지 번호가 아니다.
    let expected = [
        "1", "1", "1", "3", "5", "6", "8", "9", "9", "12", "13", "14", "15", "16", "17", "20",
        "23", "24", "25", "33", "34", "35", "36", "37", "37", "38",
    ];
    let entries: Vec<_> = core.document().sections[start.0]
        .paragraphs
        .iter()
        .enumerate()
        .take(end.1)
        .skip(start.1 + 1)
        .filter_map(|(paragraph, source)| {
            let (_, suffix) = source.text.rsplit_once('\t')?;
            let number = suffix.trim();
            (!number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
                .then_some((paragraph, source, number))
        })
        .collect();
    assert_eq!(
        entries
            .iter()
            .map(|(_, _, number)| *number)
            .collect::<Vec<_>>(),
        expected,
        "독립 Print의 목차 번호·순서·누락·중복"
    );
    let page = core.build_page_render_tree(1).unwrap();
    let mut right_edges = Vec::new();
    for (paragraph, source, number) in entries {
        let line = one_line(&page.root, (start.0, paragraph));
        let mut actual = String::new();
        text(line, &mut actual);
        assert_eq!(
            normalized(&actual),
            normalized(&source.text),
            "목차의 원문 내용 보존"
        );
        let number_run = line.children.iter().rev().find(|child| {
            matches!(&child.node_type, RenderNodeType::TextRun(run)
                if !run.text.trim_matches(|c: char| c.is_whitespace() || c == '\u{fffc}').is_empty())
        }).expect("원문에 소속된 마지막 페이지 번호 런");
        let RenderNodeType::TextRun(run) = &number_run.node_type else {
            unreachable!()
        };
        assert_eq!(
            run.text
                .trim_matches(|c: char| c.is_whitespace() || c == '\u{fffc}'),
            number,
            "화면 x 영역 대신 원문 문단·끝 토큰으로 페이지 번호 선택"
        );
        assert_eq!(
            (run.section_index, run.para_index),
            (Some(start.0), Some(paragraph))
        );
        assert!(run.cell_context.is_none());
        assert!(
            line.bbox.x <= number_run.bbox.x
                && number_run.bbox.x + number_run.bbox.width <= line.bbox.x + line.bbox.width,
            "페이지 번호는 자기 목차 줄 내부에 포함"
        );
        right_edges.push(number_run.bbox.x + number_run.bbox.width);
    }
    let min = right_edges.iter().copied().fold(f64::INFINITY, f64::min);
    let max = right_edges
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max - min <= 1.5,
        "목차 페이지 번호의 상대 오른쪽 정렬: {right_edges:?}"
    );
}

// 독립 2020 Print 40쪽의 U+119E는 다음 박과 같은 원점을 쓰고 전문인까지 같은 줄이다.
// 원본 저장 컷·문자 IR을 고치지 않고 선택 프로그램의 줄 소속을 함께 갱신해야 한다.
#[test]
fn issue_8160_selected_jamo_program_keeps_print_word_partition_and_source_ownership() {
    let mut core = load();
    core.set_hangul2024_compat(false);
    // None은 portable default다. 실제 Print와 같이 2020 프로그램 선택 세션을 연다.
    core.set_font_environment(Some(
        rhwp::renderer::font_environment::FontEnvironment::from_json(
            r#"{"id":"hancom-2020-jamo-program-selection","substitutions":{}}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    let sources: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, body)| {
            body.paragraphs
                .iter()
                .enumerate()
                .filter_map(move |(paragraph, source)| {
                    source
                        .text
                        .contains("100여명의 석ᆞ박사급 전문인력")
                        .then_some((section, paragraph))
                })
        })
        .collect();
    assert_eq!(sources.len(), 1, "원본 자모 문단의 고유 소유");
    let source = sources[0];
    let original = &core.document().sections[source.0].paragraphs[source.1];
    let authored = original.text.clone();
    let stored: Vec<_> = original
        .line_segs
        .iter()
        .map(|line| (line.text_start, line.vertical_pos))
        .collect();
    let page = core.build_page_render_tree(39).unwrap();
    let mut found = Vec::new();
    lines(&page.root, source, &mut found);
    assert_eq!(found.len(), 6, "같은 원본 문단의 여섯 글줄·누락·중복");
    let rendered: Vec<_> = found
        .iter()
        .map(|line| {
            let mut value = String::new();
            text(line, &mut value);
            value
        })
        .collect();
    assert_eq!(
        normalized(&rendered.concat()),
        normalized(&authored),
        "원문 문자·순서·소유 보존"
    );
    assert!(
        rendered[2].trim_end().ends_with("석ᆞ박사급 전문인"),
        "독립 Print의 같은 줄 끝: {rendered:?}"
    );
    assert!(
        rendered[3].starts_with("력이 수자원의"),
        "무전진 자모 뒤 현재 줄 소속"
    );
    assert!(rendered[4].starts_with("로 수행하고"));
    assert!(rendered[5].starts_with("는 보조 및"));
    for pair in found.windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "새 글줄의 순서와 비겹침"
        );
    }
    for line in &found {
        for child in &line.children {
            if let RenderNodeType::TextRun(run) = &child.node_type {
                assert_eq!(
                    (run.section_index, run.para_index),
                    (Some(source.0), Some(source.1))
                );
                assert!(
                    run.cell_context.is_none(),
                    "본문 자모의 소유를 셀로 옮기지 않음"
                );
                let start = run.char_start.expect("같은 원문의 문자 시작");
                assert_eq!(
                    run.text,
                    authored
                        .chars()
                        .skip(start)
                        .take(run.text.chars().count())
                        .collect::<String>()
                );
            }
        }
    }
    assert_eq!(
        core.document().sections[source.0].paragraphs[source.1].text,
        authored
    );
    assert_eq!(
        core.document().sections[source.0].paragraphs[source.1]
            .line_segs
            .iter()
            .map(|line| (line.text_start, line.vertical_pos))
            .collect::<Vec<_>>(),
        stored,
        "파생 줄은 원본 저장 IR을 덮어쓰지 않는다"
    );
}

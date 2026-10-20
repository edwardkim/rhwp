//! PrEP 27·33쪽은 전체 90% 선행 조건을 충족했지만 양쪽 98% 완료 쪽이 아니다.
//! 절대 좌표 대신 원본 소유·내용·저장 줄 메트릭·앞뒤 흐름을 보호한다.
//! 독립 Print와 전쪽 증적: mydocs/working/task_m100_7642_stage1581.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use rhwp::renderer::render_tree::{CaptionControlKind, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

type Owner = (usize, usize);

fn load() -> DocumentCore {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    core
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}

fn text(node: &RenderNode) -> String {
    let mut result = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push_str(&run.text);
    }
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn walk<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if predicate(node) {
        out.push(node);
    }
    for child in &node.children {
        walk(child, predicate, out);
    }
}

fn lines(node: &RenderNode, owner: Owner) -> Vec<&RenderNode> {
    let mut found = Vec::new();
    walk(
        node,
        &|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(l)
        if l.caption_owner.is_none() && (l.section_index,l.para_index)==(Some(owner.0),Some(owner.1)))
        },
        &mut found,
    );
    found
}

fn owner(core: &DocumentCore, prefix: &str) -> Owner {
    let mut found = Vec::new();
    for (section, s) in core.document().sections.iter().enumerate() {
        for (paragraph, p) in s.paragraphs.iter().enumerate() {
            if compact(&p.text).starts_with(&compact(prefix)) {
                found.push((section, paragraph));
            }
        }
    }
    assert_eq!(found.len(), 1, "고유 원문 문단: {prefix}");
    found[0]
}

fn paragraph(core: &DocumentCore, owner: Owner) -> &Paragraph {
    &core.document().sections[owner.0].paragraphs[owner.1]
}

fn complete_content(nodes: &[&RenderNode], source: &Paragraph) {
    let actual: String = nodes.iter().map(|n| compact(&text(n))).collect();
    assert_eq!(actual, compact(&source.text), "전체 원문 내용·순서 보존");
    for pair in nodes.windows(2) {
        assert!(
            pair[1].bbox.y >= pair[0].bbox.y + pair[0].bbox.height,
            "줄 순서·겹침"
        );
    }
}

fn source_advance(actual: f64, expected_hwp: i32, label: &str) {
    // 유효한 저장본과 독립 Print로 확인한 상대 전진이다. 화면의 절대 위치가 아니다.
    let expected = expected_hwp as f64 * 96.0 / 7200.0;
    let quantization = 2.0 * 96.0 / 7200.0;
    assert!(
        (actual - expected).abs() <= quantization,
        "저장 줄의 {label}: {actual} / {expected}"
    );
}

#[test]
fn prep_chart_caption_keeps_empty_line_occupancy_and_source_spacing() {
    let core = load();
    let title = "[그림 6] 서울시민 성생활 비율과 성생활을 하지 않는 이유 (여성)";
    let mut owners = Vec::new();
    for (section, s) in core.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                let Control::Shape(shape) = c else { continue };
                let ShapeObject::Ole(ole) = shape.as_ref() else {
                    continue;
                };
                let Some(caption) = &ole.caption else {
                    continue;
                };
                let content: String = caption
                    .paragraphs
                    .iter()
                    .map(|p| compact(&p.text))
                    .collect();
                if content == compact(title) {
                    owners.push((section, pi, ci));
                }
            }
        }
    }
    assert_eq!(owners.len(), 1, "원문의 고유한 OLE 캡션 소유");
    let (section, pi, ci) = owners[0];
    let next_owner = owner(&core, "- 남성에서는 20대의 성생활 비율");
    assert_eq!(next_owner, (section, pi + 2));
    let empty_owner = (section, pi + 1);
    let host = paragraph(&core, (section, pi));
    let empty = paragraph(&core, empty_owner);
    assert!(empty.text.trim().is_empty(), "진짜 빈 원문 문단");
    assert_eq!(empty.line_segs.len(), 1);
    let page = core.build_page_render_tree(26).unwrap();
    let mut charts = Vec::new();
    walk(
        &page.root,
        &|n| {
            matches!(&n.node_type, RenderNodeType::RawSvg(r)
        if r.control_ref.as_ref().is_some_and(|o| (o.section_index,o.para_index,o.control_index)==(section,pi,ci)))
        },
        &mut charts,
    );
    assert_eq!(charts.len(), 1, "차트의 원본 소유와 쪽 보존");
    let mut captions = Vec::new();
    walk(
        &page.root,
        &|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(l)
        if l.caption_owner.as_ref().is_some_and(|o| (o.sec_idx,o.para_idx,o.control_idx)==(section,pi,ci)
            && o.control_kind==CaptionControlKind::Shape))
        },
        &mut captions,
    );
    assert_eq!(captions.len(), 1, "독립 Print의 한 줄 캡션");
    let caption = captions[0];
    assert_eq!(compact(&text(caption)), compact(title));
    assert!(
        caption.bbox.y >= charts[0].bbox.y + charts[0].bbox.height,
        "차트 뒤 캡션"
    );
    let empty_lines = lines(&page.root, empty_owner);
    assert_eq!(empty_lines.len(), 1, "빈 문단의 물리 줄 소유");
    assert!(empty_lines[0].bbox.height > 0.0, "빈 줄도 물리 공간을 점유");
    source_advance(
        empty_lines[0].bbox.y - caption.bbox.y - caption.bbox.height,
        host.line_segs.last().unwrap().line_spacing,
        "캡션 뒤 저장 간격",
    );
    let following = lines(&page.root, next_owner);
    assert_eq!(following.len(), 1, "독립 Print의 다음 본문 한 줄");
    complete_content(&following, paragraph(&core, next_owner));
    source_advance(
        following[0].bbox.y - empty_lines[0].bbox.y,
        empty.line_segs[0].line_height + empty.line_segs[0].line_spacing,
        "빈 줄 높이와 뒤 간격",
    );
}

#[test]
fn prep_stored_six_rows_preserve_complete_words_and_neighbor_pages() {
    let core = load();
    let target = owner(&core, "그러나 gNSUM에는 몇 가지 중요한 제약도 존재함.");
    let prior_owner = (target.0, target.1 - 1);
    // 차례에도 같은 제목이 있으므로 원본에서 바로 이어지는 문단의 소유로 찾는다.
    let heading_owner = (target.0, target.1 + 1);
    assert_eq!(
        compact(&paragraph(&core, heading_owner).text),
        compact("제5절. 성연결망(Sexaul network) 분석")
    );
    let source = paragraph(&core, target);
    assert_eq!(source.line_segs.len(), 6, "정상 한컴 저장 여섯 줄");
    let page = core.build_page_render_tree(32).unwrap();
    let following_page = core.build_page_render_tree(33).unwrap();
    let actual = lines(&page.root, target);
    // 독립 Print 33쪽의 여섯 줄을 그대로 대조한다. 절대 px 좌표는 사용하지 않는다.
    let expected = [
        "그러나 gNSUM에는 몇 가지 중요한 제약도 존재함. 숨겨진 집단의 일부로부터 실제 표본을 확보해야 한다는",
        "점에서 설계 복잡도가 증가하며, 단순 NSUM보다 조사 운영 비용, 시간, 접근성, 윤리적 고려 등 실무 부담이",
        "커짐. 또한 숨겨진 집단 구성원에게 요구되는 가시성 문항은 본인의 정체성과 관련된 민감한 내용을 포함하기",
        "때문에 응답 부담과 응답 회피(non-response)가 늘어날 가능성이 있음. 응답자가 자신의 정체성 노출 여부를",
        "과소･과대평가하거나 특정 지인 집단(alter groups) 규모를 제대로 파악하지 못하면, 가시성 추정 모델 자체가",
        "왜곡될 수 있음.",
    ];
    assert_eq!(
        actual.len(),
        expected.len(),
        "줄 누락·중복과 완결된 마지막 낱말"
    );
    for (line, expected) in actual.iter().zip(expected) {
        assert_eq!(compact(&text(line)), compact(expected));
    }
    complete_content(&actual, source);
    for (pair, saved) in actual.windows(2).zip(source.line_segs.windows(2)) {
        source_advance(
            pair[1].bbox.y - pair[0].bbox.y,
            saved[1].vertical_pos - saved[0].vertical_pos,
            "문단 내부 저장 줄 전진",
        );
    }
    let prior = lines(&page.root, prior_owner);
    complete_content(&prior, paragraph(&core, prior_owner));
    let last = prior.last().expect("선행 문단 보존");
    assert!(
        actual[0].bbox.y >= last.bbox.y + last.bbox.height,
        "선행 문단 뒤의 본문"
    );
    let prior_saved = paragraph(&core, prior_owner).line_segs.last().unwrap();
    source_advance(
        actual[0].bbox.y - last.bbox.y,
        source.line_segs[0].vertical_pos - prior_saved.vertical_pos,
        "선행 문단 뒤 저장 전진",
    );
    assert!(
        lines(&following_page.root, target).is_empty(),
        "마지막 낱말을 다음 쪽에 중복하지 않음"
    );
    assert!(
        lines(&page.root, heading_owner).is_empty(),
        "독립 Print의 다음 쪽 제목 소유"
    );
    let heading = lines(&following_page.root, heading_owner);
    assert_eq!(heading.len(), 1);
    complete_content(&heading, paragraph(&core, heading_owner));
}

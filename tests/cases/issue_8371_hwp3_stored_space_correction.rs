//! 관계형: 한글 3.0 저장 줄의 공백 보정과 편집 무효화 계약.
//! 주석 갱신: 2026-10-30T17:28:27+09:00; 신규 파일로 직전 파일 커밋 없음.
//! 2026-10-30 중간 측정 raw(96 DPI, PDFium, 2px 이웃): 28쪽 Native 99.06% / fresh WASM 99.02%,
//! 193쪽 Native 98.29% / fresh WASM 99.99%; 전 763쪽 최저 Native 98.29% / WASM 99.02%.
//! 근거: mydocs/pr/assets/issue_8371/sample10-native.tsv, mydocs/pr/assets/issue_8371/sample10-wasm.tsv,
//! mydocs/working/task_m100_8371_stage1.md. 최종 head 재측정 필요.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
        TextStyle,
    },
    DocumentCore,
};
use serde_json::Value;

fn space_advance(trace: &Value, paragraph: u64, offset: u64) -> i64 {
    trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["source"]["paragraphIndex"] == paragraph
                && entry["source"]["charOffset"] == offset
                && entry["source"]["character"] == " "
        })
        .expect("원본 공백과 출력 글줄의 소유 관계")["layoutMetric"]["finalAdvanceHwpunit"]
        .as_i64()
        .unwrap()
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(nodes(child));
    }
    result
}

#[test]
fn saved_hwp3_space_corrections_follow_their_rows_and_expire_on_edit() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample10.hwp"))
        .expect("한글 3.0 원본");
    let paragraphs = &core.document().sections[0].paragraphs;
    let zero = &paragraphs[1194];
    assert_eq!(zero.hwp3_saved_space_corrections, [(0, 0), (73, 0)]);
    let corrected = &paragraphs[7414];
    assert_eq!(
        corrected.hwp3_saved_space_corrections,
        [(0, 4), (68, 13), (123, 14), (176, 0)]
    );

    let trace28: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(27, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        space_advance(&trace28, 1194, 2),
        space_advance(&trace28, 1194, 75),
        "저장 보정이 0인 첫 줄과 마지막 줄의 같은 공백 전진은 같다"
    );
    let trace193: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(192, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    assert!(
        space_advance(&trace193, 7414, 11) > space_advance(&trace193, 7414, 177),
        "양수 저장 보정이 있는 줄의 내부 공백은 보정 없는 줄보다 넓다"
    );
    let tree = core.build_page_render_tree(27).unwrap();
    let line = nodes(&tree.root)
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .find(|line| {
            line.children.iter().any(|child| {
                matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text == "RDBMS ")
            })
        })
        .expect("표 머리글의 첫 줄");
    let run = line.children
        .iter()
        .find(|child| matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text == "RDBMS "))
        .unwrap();
    assert!(
        run.bbox.width > line.bbox.width * 0.9,
        "내부 공백이 없는 단어는 저장 공백 보정 때문에 글자 사이 배분이 사라지면 안 된다"
    );
    let page = core.build_page_render_tree(307).unwrap();
    let (line, node, run) = nodes(&page.root)
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .find_map(|line| {
            line.children.iter().find_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(11301) && run.text.contains("DEPTNO") =>
                {
                    Some((line, node, run))
                }
                _ => None,
            })
        })
        .expect("308쪽 반복 대시와 후속 열 이름");
    let index = run.text.split("DEPTNO").next().unwrap().chars().count();
    let measured;
    let positions = if let Some(positions) = &run.layout_positions {
        positions
    } else {
        measured = EmbeddedTextMeasurer.compute_char_positions(run.display_or_text(), &run.style);
        &measured
    };
    assert!(
        node.bbox.x + positions[index] > line.bbox.x + line.bbox.width * 0.85,
        "Print에서 오른쪽 끝에 있는 열 이름은 반복 대시 복원 뒤에도 그 줄의 오른쪽에 남는다"
    );
    let mut edited = zero.clone();
    edited.invalidate_layout_inputs();
    assert!(
        edited.hwp3_saved_space_corrections.is_empty(),
        "편집 후 이전 줄 보정은 무효다"
    );
}

#[test]
fn hwp3_title_punctuation_keeps_latin_glyph_owner_with_supplied_programs() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample10.hwp"))
        .expect("한글 3.0 원본");
    let mut document = core.document().clone();
    let shape = &document.doc_info.char_shapes[55];
    let korean_id = shape.font_ids[0] as usize;
    let latin_id = shape.font_ids[1] as usize;
    let korean = &mut document.doc_info.font_faces[0][korean_id];
    korean.name = "RHWP Test Korean".into();
    korean.alt_type = 2;
    let latin = &mut document.doc_info.font_faces[1][latin_id];
    latin.name = "RHWP Test Latin".into();
    latin.alt_type = 2;
    core.set_document(document);

    let mut korean_requested = false;
    let mut latin_requested = false;
    core.prepare_native_otf_fonts_with(|request| match request.family.as_str() {
        "RHWP Test Korean" => {
            korean_requested = true;
            Some((
                include_bytes!("../fixtures/fonts/RHWPHftOutline.otf").to_vec(),
                0,
            ))
        }
        "RHWP Test Latin" => {
            latin_requested = true;
            Some((
                include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf").to_vec(),
                0,
            ))
        }
        _ => None,
    })
    .unwrap();
    assert!(
        korean_requested && latin_requested,
        "두 원본 슬롯의 프로그램 요청"
    );
    let page = core.build_page_render_tree(192).unwrap();
    let line = nodes(&page.root)
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .find(|line| {
            line.children.iter().any(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run) if run.para_index == Some(7403) && run.text == "(")
            })
        })
        .expect("제목의 여는 괄호는 영문 프로그램을 소유하는 별도 런");
    let runs: Vec<_> = line
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.para_index == Some(7403) => Some((node, run)),
            _ => None,
        })
        .collect();
    let index = runs.iter().position(|(_, run)| run.text == "(").unwrap();
    assert!(index > 0 && index + 1 < runs.len());
    assert!(runs[index - 1].1.text.ends_with("조치방법"));
    assert!(runs[index + 1].1.text.starts_with("MAXDATAFILES"));
    assert_eq!(runs[index].1.style.font_family, "RHWP Test Latin");
    assert!(runs[index - 1].0.bbox.x < runs[index].0.bbox.x);
    assert!(runs[index].0.bbox.x < runs[index + 1].0.bbox.x);
}

#[test]
fn hwp3_source_double_rule_keeps_halfwidth_and_line_ownership() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample10.hwp"))
        .expect("한글 3.0 원본");
    let page = core.build_page_render_tree(760).unwrap();
    let run = nodes(&page.root)
        .into_iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(26707) && run.text.contains('\u{f0827}') =>
            {
                Some(run)
            }
            _ => None,
        })
        .expect("761쪽 제목 아래 원본 이중선");
    let chars: Vec<_> = run.text.chars().collect();
    let first_rule = chars.iter().position(|ch| *ch == '\u{f0827}').unwrap();
    assert_eq!(
        chars[..first_rule].iter().filter(|ch| **ch == '=').count(),
        8
    );
    assert_eq!(
        chars[first_rule..]
            .iter()
            .filter(|ch| **ch == '\u{f0827}')
            .count(),
        30
    );
    let measured;
    let positions = if let Some(positions) = &run.layout_positions {
        positions
    } else {
        measured = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        &measured
    };
    let ascii_mean = (positions[first_rule] - positions[0]) / first_rule as f64;
    let rule_mean =
        (positions[chars.len()] - positions[first_rule]) / (chars.len() - first_rule) as f64;
    assert!(
        (0.9..=1.1).contains(&(rule_mean / ascii_mean)),
        "독립 Print의 원본 이중선은 같은 줄의 반각 등호와 비슷한 전진을 갖는다"
    );
    let style = TextStyle {
        font_family: "시험용 표시 글꼴".into(),
        original_glyph_metric_family: Some("명조".to_string().into()),
        font_size: 16.0,
        ..Default::default()
    };
    let source = EmbeddedTextMeasurer.compute_char_positions("\u{f0827}", &style);
    let literal = EmbeddedTextMeasurer.compute_char_positions("═", &style);
    assert!(
        source[1] * 1.9 < literal[1],
        "원본 반각 기호와 전각 Unicode 문자는 서로 다른 프로그램 폭을 갖는다"
    );
}

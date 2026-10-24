//! C449: 원문/독립 Print74쪽의 Native/fresh WASM 전쪽 raw90과 직접 판독 후 보강한다.
//! p8은 양쪽99.9988% 이상이므로 독립 PDF Tm/cm의 핵심 좌표를 ±2 CSS px로 보호한다.
//! 과거7807021b76에서 분리했던 8쪽 줄 소속/74쪽 계약도 원문 소유 관계와 함께 복원한다.
//! 전쪽 TSV·글꼴 프로그램 공급·source hash 근거는 단계8 C449 보고서를 참조한다.
//! [#7418] 영문 슬롯이 옛 한컴 영문 글꼴이면 ASCII 구두점도 한/글이 **영문 슬롯** 글꼴로 재고
//! 그린다.
//!
//! # 무엇이 깨져 있었나
//!
//! rhwp 는 ASCII 구두점을 언어 중립으로 보고 앞 글자 언어를 물려받았다. 한글 뒤 `-` 는 한글
//! 슬롯의 휴먼명조로 재여 9.63px(722 HWPUNIT)가 됐다. 한/글 PDF 에서 같은 `-` 는 영문 슬롯의
//! Palatino Linotype(영문 슬롯 `HCI Poppy` 의 치환)으로 그려지고 전진폭은 ≈6.4px 다.
//!
//! `78494-virtual-convergence-industry-decree` 8쪽 `pi=73` 첫 줄 `  - 해당 사업자는 … 수반되나,
//! 사업` 이 그 3.2px 때문에 `업` 을 213 HWPUNIT 넘겨 `사`/`업` 으로 갈렸다. 문단이 정본 4줄 대신
//! 5줄이 되어 +29px, 그 뒤 빈 문단 `pi=86` 이 혼자 9쪽을 열어 75쪽(정본 74)이 됐다.
//!
//! # 근거
//!
//! 말뭉치 `pdf/` 1,359개에서 한글 뒤 ASCII 구두점의 글꼴을 쟀다. 한글 글꼴이 휴먼명조인 줄에서
//! `(` 2278:17 · `,` 1733:6 · `)` 819:8 · `.` 525:10 · `:` 252:15 · `-` 77:0 (영문 글꼴:한글 글꼴)
//! — 99% 가 영문 슬롯이다. 구두점 뒤 공백은 88% 가 한글 글꼴이라 앞 글자 언어를 그대로 잇는다.
//! 한글 글꼴이 바탕·맑은 고딕이면 대부분 한글 글꼴 그대로다(2404:329 · 622:70) — 그래서 규칙은
//! 영문 슬롯이 옛 한컴 영문 글꼴(`LegacyLatin` 치환 — 이 문서는 휴먼명조 + `HCI Poppy`)일 때만
//! 건다. 반례는 마지막 검사다.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 저장 줄(`LINE_SEG`)을 쓰는 문단의 줄 나눔은 바뀌지 않는다. `samples/` 1,176개 중 쪽 경계가
//! 바뀐 문서는 이 문서 하나다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const DOC: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
/// 0-based. 한/글 출력 PDF 의 8쪽.
const PAGE: u32 = 7;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{DOC} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 원문 하이픈은 영문 슬롯으로 전진하고, 한글과 런이 나뉘어도 원문 소속을 보존한다.
///
/// 공개 text-layout의 원문 주소와 `charX`를 함께 검사한다. 독립 Print의 하이픈 전진은
/// 글자 크기의 약 0.32배이고, 한글 슬롯의 반각 전진인 약 0.48배보다 작다.
#[test]
fn the_hyphen_in_a_hangul_run_advances_with_latin_slot_metrics() {
    let core = load();
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON");
    let runs = layout["runs"].as_array().expect("runs");
    let paragraph: Vec<_> = runs
        .iter()
        .filter(|run| run["secIdx"] == 0 && run["paraIdx"] == 73)
        .collect();
    let source = &core.document().sections[0].paragraphs[73];
    let mut next = 0;
    let mut actual_text = String::new();
    for run in &paragraph {
        assert_eq!(run["charStart"].as_u64(), Some(next), "원문 순서·누락·중복");
        let text = run["text"].as_str().expect("소유 런 내용");
        next += text.encode_utf16().count() as u64;
        actual_text.push_str(text);
    }
    assert_eq!(actual_text, source.text, "원문 문단의 전체 내용 보존");
    // Restore the independent first-line/continuation contract removed in
    // 7807021b76, selecting by source paragraph and relative line ownership.
    let first_y = &paragraph[0]["y"];
    let first_line: String = paragraph
        .iter()
        .take_while(|item| &item["y"] == first_y)
        .map(|item| item["text"].as_str().unwrap())
        .collect();
    assert_eq!(
        first_line.split_whitespace().collect::<String>(),
        "-해당사업자는신고업무처리를위한행정적부담이수반되나,사업",
        "독립 Print8쪽 첫 줄은 사업까지 같은 줄에 둔다"
    );
    let next_line: String = paragraph
        .iter()
        .skip_while(|item| &item["y"] == first_y)
        .map(|item| item["text"].as_str().unwrap())
        .collect();
    assert!(
        next_line.trim_start().starts_with("일반현황,"),
        "다음 source 줄의 첫 내용"
    );
    let matches: Vec<_> = paragraph
        .iter()
        .filter(|run| run["charStart"] == 2)
        .collect();
    assert_eq!(matches.len(), 1, "원문 하이픈의 소유 런은 유일하다");
    let run = matches[0];
    assert_eq!(run["text"], "-");
    assert_eq!(
        run["fontFamily"], "Palatino Linotype",
        "독립 Print의 영문 슬롯"
    );
    let text: Vec<char> = run["text"].as_str().unwrap().chars().collect();
    let hyphen = text.iter().position(|c| *c == '-').expect("`-`");
    assert_eq!(run["charStart"].as_u64().unwrap() + hyphen as u64, 2);
    assert_eq!(
        run["fontFamily"], "Palatino Linotype",
        "독립 Print의 구두점 face"
    );
    let char_x: Vec<f64> = run["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|x| x.as_f64().expect("문자 경계"))
        .collect();
    let advance = char_x[hyphen + 1] - char_x[hyphen];
    let font_size = run["fontSize"].as_f64().expect("원문 글자 크기");
    let advance_em = advance / font_size;
    assert!(
        (0.275..0.35).contains(&advance_em),
        "`-` 전진/글자 크기 {advance_em:.3} — 독립 Print의 약 0.32em, 한글 반각 약 0.48em"
    );
    let following = paragraph
        .iter()
        .find(|following| following["charStart"] == 3)
        .expect("하이픈 다음의 한글 소유 런");
    assert!(following["text"]
        .as_str()
        .unwrap()
        .starts_with(" 해당 사업자는"));
    assert_eq!(
        rhwp::renderer::style_resolver::primary_font_name(
            following["fontFamily"]
                .as_str()
                .expect("CSS face 후보 사다리")
        ),
        "휴먼명조",
        "원본 한글 face는 보존하며 공개 API의 CSS 후보 사다리와 구분한다"
    );
    assert_eq!(
        following["charShapeId"], run["charShapeId"],
        "같은 원문 글자모양"
    );
    assert_eq!(following["y"], run["y"], "하이픈과 뒤 본문은 같은 줄");
    assert!(
        (following["x"].as_f64().unwrap() - run["x"].as_f64().unwrap() - advance).abs() / font_size
            < 0.01,
        "영문 전진 끝에서 한글 본문을 이어 그린다"
    );

    // Independent Print p8: cm=(0.12,0,0,-0.12,0,841), Tm x=597/697,
    // y=3366. Convert PDF points to CSS px; these are not renderer snapshots.
    // Enable only after current full-document raw90 and p8 N/W raw98+ reading.
    let hyphen_x = run["x"].as_f64().unwrap();
    let first_hangul_x = following["x"].as_f64().unwrap()
        + following["charX"][1]
            .as_f64()
            .expect("뒤 공백 다음 원문 글자 경계");
    assert!((hyphen_x - 95.52).abs() <= 2.0, "독립 Print 하이픈 원점");
    assert!(
        (first_hangul_x - 111.52).abs() <= 2.0,
        "독립 Print 첫 한글 원점"
    );
    let page = core.build_page_render_tree(PAGE).unwrap();
    assert_eq!(
        core.page_count(),
        74,
        "독립 Print 전74쪽과 원문 문단의 소속을 함께 보존"
    );
    let owned: Vec<_> = decree_nodes(&page.root)
        .into_iter()
        .filter_map(|node| {
            let rhwp::renderer::render_tree::RenderNodeType::TextRun(text) = &node.node_type else {
                return None;
            };
            (text.section_index == Some(0)
                && text.para_index == Some(73)
                && text.cell_context.is_none()
                && text.char_start == Some(2)
                && text.text == "-")
                .then_some((node, text))
        })
        .collect();
    assert_eq!(
        owned.len(),
        1,
        "독립 하이픈은 실제 paint에도 유일한 원문 소유자"
    );
    assert!(
        (owned[0].0.bbox.y + owned[0].1.baseline - 538.56).abs() <= 2.0,
        "독립 Print의 실제 paint 기준선"
    );
}

fn decree_nodes(
    node: &rhwp::renderer::render_tree::RenderNode,
) -> Vec<&rhwp::renderer::render_tree::RenderNode> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(decree_nodes(child));
    }
    result
}

/// Source s0/p331/c0: the nested third column does not own the plain second
/// column's closing gap. Independent Print p36 closes the label paragraph;
/// p37 continues only the separate article-number paragraph.
#[test]
fn plain_label_keeps_its_closing_line_beside_a_nested_continuation() {
    use rhwp::{
        model::control::Control,
        renderer::{font_environment::FontEnvironment, render_tree::RenderNodeType},
    };
    let mut core = load();
    core.set_hangul2024_compat(false);
    // Portable layout selection model; actual glyph availability/program bytes
    // are checked separately by the Native/fresh WASM full visual evidence.
    let programs = serde_json::json!([
        {"query":"HY중고딕","family":"HYGothic-Medium","programId":"layout-control-H2gtrM","trueType":true},
        {"query":"함초롬돋움","family":"HCR Dotum","programId":"layout-control-HCRDotum","trueType":true}
    ]);
    let selection =
        FontEnvironment::from_available_programs(&core.document().doc_info, &programs.to_string())
            .unwrap();
    core.set_font_environment(selection).unwrap();
    let Control::Table(source) = &core.document().sections[0].paragraphs[331].controls[0] else {
        panic!("원본 표")
    };
    let label = &source.cells[7];
    assert_eq!(
        (label.row, label.col, label.row_span, label.col_span),
        (2, 1, 1, 1)
    );
    assert!(label
        .paragraphs
        .iter()
        .all(|paragraph| paragraph.line_segs.is_empty() && paragraph.controls.is_empty()));
    assert!(source.cells[8].paragraphs.iter().any(|paragraph| paragraph
        .controls
        .iter()
        .any(|control| matches!(control, Control::Table(_)))));
    let expected: Vec<_> = label
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.clone())
        .collect();
    assert_eq!(
        expected,
        ["1인 창조기업 육성에 관한 법률 시행령", "제4조, [별표2]"]
    );
    let mut fragments = Vec::new();
    for page in [35, 36] {
        let tree = core.build_page_render_tree(page).unwrap();
        let table = decree_nodes(&tree.root).into_iter().find(|node| matches!(&node.node_type,
            RenderNodeType::Table(table) if table.section_index == Some(0) && table.para_index == Some(331)
                && table.control_index == Some(0) && table.cell_context.is_none())).expect("현재 원본 표 조각");
        let cell = table
            .children
            .iter()
            .find(|node| {
                matches!(&node.node_type,
            RenderNodeType::TableCell(cell) if cell.model_cell_index == Some(7))
            })
            .expect("라벨의 물리 셀 조각");
        let mut parts = vec![String::new(), String::new()];
        for node in decree_nodes(cell) {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                continue;
            };
            let Some(context) = &run.cell_context else {
                continue;
            };
            if context.parent_para_index != 331 || context.path.len() != 1 {
                continue;
            }
            let address = &context.path[0];
            if address.control_index != 0 || address.cell_index != 7 {
                continue;
            }
            let paragraph = address.cell_para_index;
            assert!(
                paragraph < parts.len(),
                "원본 문단 범위를 벗어난 소유자가 없어야 한다"
            );
            assert!(
                node.bbox.y >= cell.bbox.y
                    && node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height,
                "선택한 라벨 줄은 실제 clip 안에 완전히 포함되어야 한다"
            );
            parts[paragraph].push_str(&run.text);
        }
        fragments.push(parts);
    }
    assert_eq!(core.page_count(), 74, "독립 Print 전쪽 수");
    assert_eq!(
        fragments[0][0], expected[0],
        "36쪽은 시행령까지 첫 문단을 닫는다"
    );
    assert!(fragments[0][1].is_empty(), "조문 번호는 다음 조각 소유");
    assert!(
        fragments[1][0].is_empty(),
        "닫은 문단을 다음 조각에 중복하지 않는다"
    );
    assert_eq!(
        fragments[1][1], expected[1],
        "37쪽은 조문 번호만 이어받는다"
    );
}

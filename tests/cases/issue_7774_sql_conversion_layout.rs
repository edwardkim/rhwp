//! 관계형·고정형 회귀 — #7774의 원본 저장 글줄, 내용 대응, 본문·꼬리말 분리와 쪽수.
//! 주석 갱신 2026-11-24 00:34 KST. 직전 파일 변경 2026-11-24 00:01:22 KST
//! `515eb18849e6eba1b5373547607fa1ffafc6fd0e`.
//! 2026-11-23~24 독립 한컴 2024 Print, 96 DPI, 2px raw 실루엣.
//! HWP5와 HWPX 각각 전763쪽 Native/fresh WASM 최저 99.57/99.34%.
//! 489쪽 각각 100.00%/99.98%, 본문·꼬리말 대상 7쪽은 양쪽 100.00%.
//! 전진폭 보정 후 16쪽은 두 변환본 모두 Native/fresh WASM 각각 99.99%다.
//! 근거는 `mydocs/pr/assets/issue7774/sql-sample10-regressions-20261015/`에 있다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURES: &str = "mydocs/pr/assets/issue7774/sql-sample10-regressions-20261015/";
const INPUTS: [&str; 2] = ["hwp3-sample10-hwp5.hwp", "hwp3-sample10-hwpx.hwpx"];
const TABLESPACE: &str =
    "TABLESPACE(ROLLBACK_DATA), TEMPORARY TABLESPACE(TEMPORARY_DATA),USER TABLESPACE(USER_DATA)";

fn visible_runs(node: &RenderNode) -> impl Iterator<Item = &RenderNode> {
    let mut pending = vec![node];
    std::iter::from_fn(move || {
        while let Some(item) = pending.pop() {
            if matches!(&item.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
            {
                return Some(item);
            }
            pending.extend(item.children.iter().rev());
        }
        None
    })
}

fn check_tablespace(root: &RenderNode, input: &str) {
    let mut pending = vec![root];
    let mut owner_lines = 0;
    while let Some(item) = pending.pop() {
        pending.extend(item.children.iter().rev());
        let RenderNodeType::TextLine(line) = &item.node_type else {
            continue;
        };
        let line_text: String = item
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect();
        if !line_text.contains(TABLESPACE) {
            continue;
        }
        owner_lines += 1;
        assert_eq!(line.para_index, Some(17686), "{input} SQL 문장의 원본 문단");
        assert_eq!(
            line_text.trim(),
            format!("{TABLESPACE}가"),
            "{input} 문장 순서"
        );

        // 공백 런 분리 여부와 무관하게 ASCII 전진량을 같은 저장 줄에서 합한다.
        let mut measured = 0.0;
        let mut expected = 0.0;
        for node in &item.children {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                continue;
            };
            if !run.text.is_empty() && run.text.is_ascii() {
                measured += node.bbox.width;
                // 한컴 Print의 489쪽 SQL 낱말을 재면 자당 약 0.502em이다.
                expected += 0.5 * run.text.len() as f64 * run.style.font_size * run.style.ratio;
            }
        }
        assert!(expected > 0.0, "{input} SQL 측정 대상");
        assert!(
            (measured / expected - 1.0).abs() <= 0.04,
            "{input} ASCII 반각 전진폭"
        );
        assert!(
            measured <= item.bbox.width,
            "{input} ASCII가 소유 글줄 폭에 포함된다"
        );
    }
    assert_eq!(owner_lines, 1, "{input} 489쪽 SQL 문장을 한 번 그린다");
}

#[test]
fn sql_converted_twins_keep_terminal_body_and_legacy_line_ownership() {
    for name in INPUTS {
        let document = DocumentCore::from_bytes(
            &std::fs::read(format!("{FIXTURES}{name}")).expect("SQL 변환 원본"),
        )
        .expect("SQL 변환본 열기");
        assert_eq!(document.page_count(), 763, "{name} 독립 Print의 전체 쪽수");
        let sql = document
            .build_page_render_tree(488)
            .expect("489쪽 SQL 문장");
        check_tablespace(&sql.root, name);

        for printed_page in [125, 243, 339, 397, 554, 588, 744] {
            let page = document
                .build_page_render_tree(printed_page - 1)
                .expect("본문·꼬리말");
            let body = page
                .root
                .children
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
                .expect("본문 영역");
            let footer = page
                .root
                .children
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Footer))
                .expect("꼬리말 영역");
            let body_end = visible_runs(body)
                .map(|node| node.bbox.y + node.bbox.height)
                .reduce(f64::max)
                .expect("본문 글줄");
            let footer_runs: Vec<_> = visible_runs(footer).collect();
            assert_eq!(
                footer_runs.len(),
                1,
                "{name} {printed_page}쪽 꼬리말 쪽번호"
            );
            let RenderNodeType::TextRun(page_number) = &footer_runs[0].node_type else {
                unreachable!();
            };
            assert_eq!(page_number.text.trim(), format!("- {} -", printed_page - 6));
            assert!(
                body_end <= footer_runs[0].bbox.y,
                "{name} {printed_page}쪽 본문·꼬리말 분리"
            );
        }
    }
}

#[test]
fn sql_contents_brackets_keep_the_original_halfwidth_advance() {
    use rhwp::renderer::font_environment::FontEnvironment;

    // 이 공개 표시 프로그램의 대괄호 폭은 독립 Print의 원본 명조 0.5em과 다르다.
    let alternate = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    for name in INPUTS {
        let mut document =
            DocumentCore::from_bytes(&std::fs::read(format!("{FIXTURES}{name}")).unwrap())
                .expect("SQL 변환본 열기");
        document.set_hangul2024_compat(true);
        let environment = FontEnvironment::from_json(
            r#"{"id":"sql-contents-display-family","preserveSourceMetrics":true,"substitutions":{"명조":"Myeongjo"}}"#,
        )
        .expect("공개 표시 글꼴 환경");
        document
            .set_font_environment(Some(environment))
            .expect("표시 계열 설정");
        assert!(document
            .prepare_native_otf_fonts_with(|request| {
                (request.bold
                    && request.italic
                    && !request.source_ttf
                    && request.family == "Myeongjo")
                    .then(|| (alternate.to_vec(), 0))
            })
            .expect("표시 프로그램 등록"));

        let trace = document
            .get_font_decision_trace_native(15, r#"{"maxCharacters":4096}"#)
            .expect("16쪽 목차의 글자 전진");
        let trace: serde_json::Value = serde_json::from_str(&trace).expect("글꼴 추적 JSON");
        let mut brackets = Vec::new();
        for record in trace["records"].as_array().expect("쪽 전체 글자 추적") {
            let source = &record["source"];
            let (Some(paragraph), Some(character @ ("[" | "]"))) = (
                source["paragraphIndex"].as_u64(),
                source["character"].as_str(),
            ) else {
                continue;
            };
            if !matches!(paragraph, 702 | 724) {
                continue;
            }
            assert_eq!(record["document"]["face"], "명조");
            assert_eq!(
                record["layoutMetric"]["baseAdvanceHwpunit"], 450,
                "{name} {paragraph} 문단 대괄호는 독립 Print의 0.5em으로 전진한다"
            );
            brackets.push((paragraph, character));
        }
        assert_eq!(brackets, [(702, "["), (702, "]"), (724, "["), (724, "]")]);
    }
}

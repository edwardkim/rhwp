//! Issue #1105: HWP3를 HWP5로 변환한 문서가 sample16 21쪽 주변의 한컴 쪽 나눔을 보존한다.

use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn svg_text_rows(svg: &str) -> Vec<(f64, String)> {
    let mut rows: Vec<(f64, String)> = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<text ") {
        let tag_start = search_from + rel;
        search_from = tag_start + 6;
        let Some(close_rel) = svg[tag_start..].find('>') else {
            break;
        };
        let attrs = &svg[tag_start..tag_start + close_rel];
        let content_start = tag_start + close_rel + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else {
            break;
        };
        let Some(y) = attr_f64(attrs, "y") else {
            continue;
        };
        let text = decode_svg_text(&svg[content_start..content_start + end_rel]);
        let key = (y * 10.0).round() / 10.0;
        if let Some((_, row_text)) = rows
            .iter_mut()
            .find(|(row_y, _)| (*row_y - key).abs() < 0.01)
        {
            row_text.push_str(&text);
        } else {
            rows.push((key, text));
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows
}

fn attr_f64(attrs: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = attrs.find(&needle)? + needle.len();
    let end = attrs[start..].find('"')?;
    attrs[start..start + end].parse().ok()
}

fn decode_svg_text(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

#[test]
fn task1105_hwp3_sample16_page23_square_bullet_matches_hancom() {
    let doc = load_doc("samples/hwp3-sample16.hwp");
    assert_eq!(doc.page_count(), 64);

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must start page 23 in the HWP3 source:\n{page23}"
    );
    assert!(
        page23.contains("\u{F03C5} 계약상대자는"),
        "HWP3 private bullet 0x3366 must be preserved as U+F03C5 before rendering:\n{page23}"
    );
    assert!(
        page23.contains("PartialParagraph  pi=460  lines=0..3"),
        "HWP3 source must honor the line-level page break inside paragraph 460:\n{page23}"
    );
    assert!(
        !page23.contains("○ 계약상대자는"),
        "HWP3 private bullet 0x3366 must not be lowered to a white circle:\n{page23}"
    );

    let svg = doc
        .render_page_svg_native(22)
        .expect("render hwp3-sample16 page 23");
    assert!(
        svg.contains("□"),
        "rendered HWP3 page 23 must display the Hancom square bullet"
    );
    assert!(
        !svg.contains("○ 계약상대자는"),
        "rendered HWP3 page 23 must not display the old white-circle bullet"
    );
}

fn assert_sample16_hwp5_business_selection_starts_next_page(rel_path: &str) {
    let doc = load_doc(rel_path);
    assert_eq!(doc.page_count(), 64, "{rel_path}");

    let page4 = doc.dump_page_items(Some(3));
    assert!(
        page4.contains("FullParagraph  pi=118"),
        "WAN line must stay on page 4 for {rel_path}:\n{page4}"
    );

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        !page5.contains("pi=118"),
        "page 5 must start at the information-system importance paragraph, not the WAN line, for {rel_path}:\n{page5}"
    );
    assert!(
        page5.contains("FullParagraph  pi=119"),
        "information-system importance paragraph must start page 5 for {rel_path}:\n{page5}"
    );
    assert!(
        page5.contains("FullParagraph  pi=140"),
        "joint supply paragraph must close page 5 for {rel_path}:\n{page5}"
    );
    assert!(
        !page5.contains("pi=141"),
        "business selection heading must not remain on page 5 for {rel_path}:\n{page5}"
    );

    let page6 = doc.dump_page_items(Some(5));
    assert!(
        page6.contains("FullParagraph  pi=141"),
        "business selection heading must start page 6 for {rel_path}:\n{page6}"
    );
    assert!(
        page6.contains("FullParagraph  pi=142"),
        "business selection body must follow the heading for {rel_path}:\n{page6}"
    );
    assert!(
        page6.contains("FullParagraph  pi=144"),
        "successful bidder heading must remain on page 6 for {rel_path}:\n{page6}"
    );
}

fn assert_sample16_hwp5_server_requirements_page_matches_hancom(rel_path: &str) {
    let doc = load_doc(rel_path);
    assert_eq!(doc.page_count(), 64, "{rel_path}");

    let page22 = doc.dump_page_items(Some(21));
    assert!(
        page22.contains("FullParagraph  pi=449"),
        "RDBMS paragraph must close page 22 for {rel_path}:\n{page22}"
    );
    assert!(
        !page22.contains("pi=450"),
        "firewall paragraph must not leak into page 22 for {rel_path}:\n{page22}"
    );

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must start page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        page23.contains("FullParagraph  pi=451"),
        "hardware/software paragraph must follow on page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        page23.contains("PartialParagraph  pi=460  lines=0..3"),
        "page 23 must contain the first three lines of the integrated DB cluster paragraph for {rel_path}:\n{page23}"
    );
    assert!(
        !page23.contains("FullParagraph  pi=460"),
        "integrated DB cluster paragraph must not remain whole on page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        !page23.contains("pi=461"),
        "next target-system paragraph must not remain on page 23 for {rel_path}:\n{page23}"
    );
}

#[test]
fn task1105_sample16_hwp5_2022_business_selection_break_matches_hancom() {
    // 2022 독립 Print는 WAN 문단을 5쪽에 놓고 공동도급 본문을 5/6쪽에 나눈다.
    let bytes =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5-2022.hwp"))
            .unwrap();
    let original = rhwp::parse_document(&bytes).unwrap();
    let core = rhwp::DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 64, "독립 한컴 Print의 전체 쪽수");
    let trees: Vec<_> = (3..=5)
        .map(|page| core.build_page_render_tree(page).unwrap())
        .collect();
    let text = |tree: &rhwp::renderer::render_tree::PageRenderTree, paragraph| {
        let mut result = String::new();
        sample16_2022_body_text(&tree.root, paragraph, &mut result);
        result
    };
    assert!(
        text(&trees[0], 118).is_empty(),
        "WAN 문단은 4쪽에 남지 않는다"
    );
    assert!(
        text(&trees[1], 118).contains("WAN"),
        "WAN 문단은 5쪽이 소유한다"
    );
    assert!(
        text(&trees[1], 119).contains("중요도"),
        "중요도 문단은 같은 쪽에 뒤따른다"
    );
    let first = text(&trees[1], 140);
    let continuation = text(&trees[2], 140);
    assert!(
        first.trim_end().ends_with("이어"),
        "공동도급 앞 조각의 독립 Print 경계"
    );
    assert!(
        continuation.starts_with("야 하며"),
        "다음 쪽은 이어받은 본문으로 시작한다"
    );
    assert_eq!(
        format!("{first}{continuation}").trim_end(),
        original.sections[0].paragraphs[140].text.trim_end(),
        "공동도급 조각은 원문을 누락이나 중복 없이 보존한다"
    );
    assert!(
        text(&trees[1], 141).is_empty(),
        "사업자 선정 제목은 5쪽에 남지 않는다"
    );
    let mut page6 = String::new();
    for paragraph in [140, 141, 142, 144] {
        let owned = text(&trees[2], paragraph);
        assert!(!owned.is_empty(), "6쪽이 해당 본문을 소유한다: {paragraph}");
        page6.push_str(&owned);
    }
    assert!(page6.find("야 하며").unwrap() < page6.find("사업자 선정방식").unwrap());
    assert!(page6.find("사업자 선정방식").unwrap() < page6.find("본 사업의").unwrap());
    assert!(page6.find("본 사업의").unwrap() < page6.find("낙찰자 선정").unwrap());
}

fn sample16_2022_body_text(
    node: &rhwp::renderer::render_tree::RenderNode,
    paragraph: usize,
    text: &mut String,
) {
    if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
        if run.cell_context.is_none() && run.para_index == Some(paragraph) {
            text.push_str(&run.text);
        }
    }
    for child in &node.children {
        sample16_2022_body_text(child, paragraph, text);
    }
}

#[test]
fn task1105_sample16_hwp5_2022_server_requirements_page_matches_hancom() {
    // 23/24쪽의 독립 Print는 Heart Bit와 구성은 사이에서 본문을 이어받는다.
    let bytes =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5-2022.hwp"))
            .unwrap();
    let original = rhwp::parse_document(&bytes).unwrap();
    let core = rhwp::DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 64, "독립 한컴 Print의 전체 쪽수");
    let mut fragments = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut text = String::new();
        sample16_2022_body_text(&tree.root, 460, &mut text);
        if !text.is_empty() {
            fragments.push((page, text));
        }
    }
    assert_eq!(
        fragments.iter().map(|(page, _)| *page).collect::<Vec<_>>(),
        vec![22, 23],
        "클러스터 문단은 한컴과 같은 두 쪽이 소유한다"
    );
    assert!(
        fragments[0].1.trim_end().ends_with("Heart Bit"),
        "앞 조각에서 다음 낱말의 첫 글자를 빼앗지 않는다"
    );
    assert!(
        fragments[1].1.starts_with("구성은"),
        "뒤 조각은 원문 낱말 전체로 시작한다"
    );
    let joined: String = fragments.iter().map(|(_, text)| text.as_str()).collect();
    assert_eq!(
        joined.trim_end(),
        original.sections[0].paragraphs[460].text.trim_end(),
        "모든 쪽에서 재구성한 문단은 원문과 같고 누락/중복이 없다"
    );
    let tree = core.build_page_render_tree(23).unwrap();
    let mut next = String::new();
    sample16_2022_body_text(&tree.root, 461, &mut next);
    assert!(
        next.contains("목표시스템 구성"),
        "다음 요구사항도 뒤 쪽에 보존한다"
    );
    assert_sample16_2022_onepoint_program(&core);
}

fn assert_sample16_2022_onepoint_program(core: &rhwp::DocumentCore) {
    // 전64쪽 Native/fresh 최저91.77231/91.77652%. 같은 한점 리더 동작은
    // 양쪽 최솟값99.20113%인59쪽이 가장 높아 우선 선택했다(다음60쪽99.14881%).
    // 독립 Print T2/HFT97의13개 원점과1em 프로그램, source-program-and-inline-spacing 증적.
    let svg = core.render_page_svg_native(58).expect("독립 Print59쪽");
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let mut ink: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("ellipse"))
        .map(|n| {
            (
                n.attribute("cx").unwrap().parse::<f64>().unwrap(),
                n.attribute("cy").unwrap().parse::<f64>().unwrap(),
            )
        })
        .collect();
    ink.sort_by(|a, b| a.1.total_cmp(&b.1));
    let baselines = [
        298.24, 337.76, 369.28, 408.80, 456.00, 494.56, 572.00, 610.56, 649.76, 688.80, 727.84,
        759.36, 806.56,
    ];
    assert_eq!(
        ink.len(),
        baselines.len(),
        "독립 Print의 모든13개 기호 잉크를 한 번 표시"
    );
    let mut source: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text") && n.text() == Some("․"))
        .collect();
    source.sort_by(|a, b| {
        a.attribute("y")
            .unwrap()
            .parse::<f64>()
            .unwrap()
            .total_cmp(&b.attribute("y").unwrap().parse::<f64>().unwrap())
    });
    assert_eq!(
        source.len(),
        baselines.len(),
        "원문 기호·검색·순서와 잉크의 일대일 소유"
    );
    for (((cx, cy), node), baseline) in ink.iter().zip(source).zip(baselines) {
        assert_eq!(
            node.attribute("fill-opacity"),
            Some("0"),
            "대체 글리프를 중복 그리지 않음"
        );
        let origin = node.attribute("x").unwrap().parse::<f64>().unwrap();
        assert!(
            (origin - 136.0).abs() <= 2.0,
            "독립 Print의 원문 기호 원점 ±2 CSSpx"
        );
        assert!(
            (cx - 144.0).abs() <= 2.0,
            "독립1em 상자 안 기호 중심 ±2 CSSpx"
        );
        assert!(
            (cy - (baseline - 16.0 * 0.3475)).abs() <= 2.0,
            "독립 기호 세로 중심 ±2 CSSpx"
        );
    }
}

#[test]
fn task1105_sample16_hwp5_2018_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2018.hwp");
}

#[test]
fn task1105_sample16_hwp5_2018_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2018.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2024_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2024.hwp");
}

#[test]
fn task1105_sample16_hwp5_2024_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2024.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2024_clean_linesegs_reflow_is_noop() {
    let mut doc = load_doc("samples/hwp3-sample16-hwp5-2024.hwp");
    assert_eq!(doc.page_count(), 64);
    assert!(
        doc.get_validation_warnings()
            .contains(r#""count":0,"summary":{}"#),
        "2024 fixture must not request lineseg reflow"
    );

    let reflowed = doc.reflow_linesegs();
    assert_eq!(
        reflowed, 0,
        "clean HWP5 linesegs must not be regenerated by on-demand reflow"
    );
    assert_eq!(doc.page_count(), 64);

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must still start page 23 after no-op reflow:\n{page23}"
    );
    assert!(
        page23.contains("PartialParagraph  pi=460  lines=0..3"),
        "integrated DB cluster paragraph split must survive no-op reflow:\n{page23}"
    );
    assert!(
        !page23.contains("pi=461"),
        "target-system paragraph must not move into page 23 after no-op reflow:\n{page23}"
    );
}

#[test]
fn task1105_k_water_rfp_2024_page_count_matches_hancom_pdf() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");
    assert_eq!(doc.page_count(), 27);
}

#[test]
fn task1105_k_water_rfp_2024_first_rowspan_table_keeps_line_reset_split() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        page5.contains("PartialTable   pi=52 ci=0  rows=0..4"),
        "the first large rowspan table must split inside its last row:\n{page5}"
    );
    assert!(
        page5.contains("end_cut=[3, 4, 2, 4, 4, 2, 20]"),
        "the first large rowspan table must cut before the orphan `유의사항` line on page 5:\n{page5}"
    );

    let svg = doc
        .render_page_svg_native(4)
        .expect("render k-water-rfp-2024 page 5");
    let rows = svg_text_rows(&svg);
    assert!(
        rows.iter()
            .any(|(_, row)| row.contains("규격") && row.contains("A4") && row.contains("가로방향")),
        "page 5 must still end near the `A4 가로방향` row:\n{rows:?}"
    );
    assert!(
        rows.iter().all(|(_, row)| !row.contains("유의사항")),
        "page 5 must not include the next `유의사항` row from the same reset paragraph:\n{rows:?}"
    );
}

#[test]
fn task1105_k_water_rfp_2024_cover_hides_first_page_footer() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");
    let svg = doc
        .render_page_svg_native(0)
        .expect("render k-water-rfp-2024 page 1");

    assert!(
        !svg.contains(
            r##"<line x1="80" y1="1034.8666666666668" x2="713.96" y2="1034.8666666666668" stroke="#787878" stroke-width="1.5"/>"##
        ),
        "first page footer table line must be hidden by SectionDef first-page footer hide"
    );
}

// 독립 한컴 PDF의21~24쪽 내용 경계다. 줄 수와 화면 좌표는 고정하지 않는다.
// 선행 전64쪽 Native/fresh 검증과 수정 전65/64 실패는
// mydocs/pr/assets/issue7445/resolution-20261005/
// sample16-semantic-regression-restoration-analysis.json에 기록했다.
fn sample16_original_and_alias() -> [rhwp::wasm_api::HwpDocument; 2] {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = fs::read(root.join("samples/hwp3-sample16-hwp5.hwp")).expect("기본 원본");
    let alias =
        fs::read(root.join("samples/hwp3-sample16-hwp5-2010.hwp")).expect("같은 원본의 별칭");
    assert_eq!(
        original, alias,
        "2010 이름을 별도 저장 버전으로 해석하지 않는다"
    );
    [original, alias]
        .map(|bytes| rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("보존 원본 파싱"))
}

fn sample16_rendered_text(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> String {
    let svg = doc.render_page_svg_native(page - 1).expect("원본 쪽 출력");
    let xml = roxmltree::Document::parse(&svg).expect("유효한 SVG");
    xml.descendants()
        .filter(|node| node.has_tag_name("text"))
        .flat_map(|node| node.descendants().filter(|child| child.is_text()))
        .filter_map(|node| node.text())
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn sample16_assert_content_order(text: &str, anchors: &[&str]) {
    let mut previous = None;
    for anchor in anchors {
        assert_eq!(
            text.matches(anchor).count(),
            1,
            "내용은 한 번만 표시해야 한다: {anchor}"
        );
        let at = text.find(anchor).expect("독립 PDF의 내용 표시");
        if let Some(previous) = previous {
            assert!(
                at > previous,
                "독립 PDF의 내용 순서를 유지해야 한다: {anchor}"
            );
        }
        previous = Some(at);
    }
}

#[test]
fn task1105_sample16_hwp5_page_break_before_section_4_matches_hancom() {
    for doc in sample16_original_and_alias() {
        assert_eq!(doc.page_count(), 64, "독립 PDF는64쪽이다");
        let page21 = sample16_rendered_text(&doc, 21);
        let page22 = sample16_rendered_text(&doc, 22);
        let page23 = sample16_rendered_text(&doc, 23);
        let section = "4.서버통합및원격지재해복구센터시스템구성요건";
        let firewall = "방화벽내․외부간DATA";
        assert!(
            page21.contains("RemoteControl관리체계구축"),
            "앞 절의 마지막 내용은21쪽에 남아야 한다"
        );
        assert!(
            !page21.contains(section),
            "다음 절 제목을 앞 쪽에 남기지 않는다"
        );
        sample16_assert_content_order(
            &page22,
            &[section, "가.전체시스템구성을위한공통요건", "RDBMS는ORACLE"],
        );
        assert!(
            !page22.contains(firewall),
            "방화벽 문단은23쪽에서 시작해야 한다"
        );
        assert!(
            !page23.contains(section),
            "절 제목을 다음 쪽에 중복하지 않는다"
        );
        sample16_assert_content_order(
            &page23,
            &[
                firewall,
                "나.주전산센터시스템구성요건",
                "ORACLERAC",
                "HeartBit",
            ],
        );
        assert_sample16_cluster_continuation(&doc);
    }
}

fn sample16_partial_paragraph_range(dump: &str, paragraph: usize) -> (usize, usize) {
    let marker = format!("PartialParagraph  pi={paragraph}  lines=");
    let range = dump
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(&marker))
        .expect("앞뒤 쪽에 같은 문단의 일부가 있어야 한다")
        .split_whitespace()
        .next()
        .expect("분할 범위");
    let (start, end) = range.split_once("..").expect("줄 범위 형식");
    (start.parse().expect("시작 줄"), end.parse().expect("끝 줄"))
}

fn assert_sample16_cluster_continuation(doc: &rhwp::wasm_api::HwpDocument) {
    let page23 = sample16_rendered_text(doc, 23);
    let page24 = sample16_rendered_text(doc, 24);
    assert!(
        page24.starts_with("구성은GigaBase"),
        "앞 쪽의 Heart Bit 문장을24쪽 처음에 이어야 한다"
    );
    assert!(
        !page23.contains("GigaBase"),
        "이어받는 내용을 앞 쪽에 중복하지 않는다"
    );
    assert!(
        !page24.contains("HeartBit"),
        "이미 표시한 내용을 다시 표시하지 않는다"
    );
    sample16_assert_content_order(
        &page24,
        &[
            "GigaBase",
            "Take-over",
            "다.재해복구센터시스템구성요건",
            "HotSite",
            "복구시간4시간이내24시간이내14일이내",
        ],
    );
    let before = doc.dump_page_items(Some(22));
    let after = doc.dump_page_items(Some(23));
    let (start, cut) = sample16_partial_paragraph_range(&before, 460);
    let (resume, end) = sample16_partial_paragraph_range(&after, 460);
    assert_eq!(start, 0, "분할 첫 조각은 문단 시작을 소유한다");
    assert!(
        cut > start && end > resume,
        "앞뒤 조각은 실제 내용을 소유한다"
    );
    assert_eq!(cut, resume, "분할 컷과 이어받기가 같아야 누락·중복이 없다");
    assert!(
        !before.contains("pi=461"),
        "후속 문단을 앞 쪽에 당기지 않는다"
    );
    assert!(
        after.contains("FullParagraph  pi=461"),
        "이어받기 뒤 후속 문단을 보존한다"
    );
}

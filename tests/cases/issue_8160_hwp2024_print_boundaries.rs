//! #8160: sample16 회귀 검사 통합. 이전 등록: tests/cases/verified_sample16_2024_print_boundaries.rs
//! sample16 한컴 2024 저장본의 제목·BCP 꼬리·후속 본문을 보호한다.
//! 검사 도입 시 전64쪽 최저 Native96.22428%, fresh WASM96.07248%,
//! 3쪽99.92012/99.90360%의 근거: task_m100_7642_stage1351.md.
//! 현재 전64쪽·입력/PDF SHA·쪽별 점수와 재검토: task_m100_7445_rebase_repair_stage8.md C419–C420.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

type Owner = (usize, usize);

fn load() -> DocumentCore {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5.hwp");
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공식 원본")).expect("원본 로드");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 64, "독립 Print의 전체 쪽수");
    core
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn source_owner(core: &DocumentCore, needle: &str) -> Owner {
    // 목차와 뒤쪽 과업 목록에도 같은 문구가 있다. 원본 책갈피로 사업개요 본문을
    // 식별하고 다음 장까지의 소유 범위에서 찾는다. 화면 위치로 대상을 고르지 않는다.
    let starts: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, data)| {
            data.paragraphs
                .iter()
                .enumerate()
                .filter_map(move |(paragraph, para)| {
                    para.controls
                        .iter()
                        .any(|item| {
                            matches!(item, Control::Bookmark(bookmark)
                        if normalized(&bookmark.name) == "1.추진목적")
                        })
                        .then_some((section, paragraph))
                })
        })
        .collect();
    assert_eq!(starts.len(), 1, "사업개요 본문의 원본 책갈피");
    let (section, start) = starts[0];
    let paragraphs = &core.document().sections[section].paragraphs;
    let end = (start + 1..paragraphs.len())
        .find(|&index| normalized(&paragraphs[index].text) == "Ⅱ.제안일반사항")
        .expect("원본의 다음 장 경계");
    let mut found = Vec::new();
    for (paragraph, source) in paragraphs.iter().enumerate().take(end).skip(start) {
        if normalized(&source.text).contains(needle) {
            found.push((section, paragraph));
        }
    }
    assert_eq!(found.len(), 1, "내용으로 식별한 단일 원본 문단: {needle}");
    found[0]
}

fn body(node: &RenderNode) -> &RenderNode {
    node.children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::Body { .. }))
        .expect("쪽의 본문 영역")
}

fn owned_lines<'a>(node: &'a RenderNode, owner: Owner, out: &mut Vec<&'a RenderNode>) {
    // 셀·글상자의 로컬 문단 번호를 본문 소유와 섞지 않는다.
    if matches!(
        node.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)))
    {
        out.push(node);
    }
    for child in &node.children {
        owned_lines(child, owner, out);
    }
}

fn runs(line: &RenderNode) -> Vec<&RenderNode> {
    let found: Vec<_> = line
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextRun(_)))
        .collect();
    assert!(!found.is_empty(), "가시 원문을 가진 줄");
    found
}

fn check_line(core: &DocumentCore, line: &RenderNode, owner: Owner, container: &RenderNode) {
    let mut actual = String::new();
    let mut prior_end = None;
    let epsilon = f64::EPSILON * container.bbox.width.max(1.0) * 128.0;
    for node in runs(line) {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert_eq!(
            (run.section_index, run.para_index),
            (Some(owner.0), Some(owner.1))
        );
        assert!(run.cell_context.is_none(), "본문 원문 소유");
        actual.push_str(&run.text);
        assert!(
            node.bbox.x + epsilon >= container.bbox.x
                && node.bbox.x + node.bbox.width
                    <= container.bbox.x + container.bbox.width + epsilon,
            "원문 런은 본문 좌우 경계에 포함된다"
        );
        if let Some(end) = prior_end {
            assert!(node.bbox.x + epsilon >= end, "앞뒤 원문 런 비겹침");
        }
        prior_end = Some(node.bbox.x + node.bbox.width);
    }
    assert_eq!(
        normalized(&actual),
        normalized(&core.document().sections[owner.0].paragraphs[owner.1].text),
        "원본 내용 순서·누락·중복"
    );
    assert!(
        line.bbox.y + epsilon >= container.bbox.y
            && line.bbox.y + line.bbox.height <= container.bbox.y + container.bbox.height + epsilon,
        "줄의 세로 점유는 본문 안에 있다"
    );
}

fn assert_only_page_three(core: &DocumentCore, owners: &[Owner]) {
    let mut occurrences = vec![Vec::new(); owners.len()];
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .expect("전체 쪽의 실제 출력");
        for (index, &owner) in owners.iter().enumerate() {
            let mut lines = Vec::new();
            owned_lines(body(&tree.root), owner, &mut lines);
            occurrences[index].extend(std::iter::repeat_n(page, lines.len()));
        }
    }
    for (owner, pages) in owners.iter().zip(occurrences) {
        assert_eq!(
            pages,
            [2],
            "Print 3쪽에만 한 줄을 한 번 소유한다: {owner:?}"
        );
    }
}

fn single_line(node: &RenderNode, owner: Owner) -> &RenderNode {
    let mut found = Vec::new();
    owned_lines(node, owner, &mut found);
    assert_eq!(found.len(), 1, "원본의 단일 줄·누락·중복: {owner:?}");
    found[0]
}

fn near_print(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print의 {label}, ±2 CSS px: actual={actual}, expected={expected}"
    );
}

fn check_baselines(line: &RenderNode, expected: f64) {
    for node in runs(line) {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        near_print(node.bbox.y + run.baseline, expected, "줄 기준선");
    }
}

#[test]
fn issue_8160_sample16_2024_completed_page_keeps_print_headings_and_owned_content_order() {
    let core = load();
    let headings = [
        source_owner(&core, "2.추진방향"),
        source_owner(&core, "3.주요추진내용"),
    ];
    assert_only_page_three(&core, &headings);
    let page = core.build_page_render_tree(2).unwrap();
    let container = body(&page.root);
    let mut previous = None;
    for (owner, baseline) in headings.into_iter().zip([359.04, 771.20]) {
        let line = single_line(container, owner);
        check_line(&core, line, owner, container);
        // PDF cm=[.12,0,0,-.12,0,841]; tm 번호 x=521, y=2244/4820.
        near_print(runs(line)[0].bbox.x, 83.36, "제목 번호 시작");
        check_baselines(line, baseline);
        if let Some(end) = previous {
            assert!(end < line.bbox.y, "제목 사이의 본문 영역과 내용 순서 보존");
        }
        previous = Some(line.bbox.y + line.bbox.height);
        let next_owner = (owner.0, owner.1 + 1);
        let mut following = Vec::new();
        owned_lines(container, next_owner, &mut following);
        assert!(
            !following.is_empty(),
            "각 제목의 후속 원문도 같은 쪽에 보존한다"
        );
        assert!(
            line.bbox.y + line.bbox.height <= following[0].bbox.y,
            "제목과 후속 본문의 비겹침"
        );
    }
}

fn token_origin(core: &DocumentCore, owner: Owner, token: &str) -> f64 {
    let json: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(2).unwrap()).unwrap();
    let mut characters = Vec::new();
    for run in json["runs"].as_array().expect("공개 배치의 런") {
        if run["secIdx"].as_u64() != Some(owner.0 as u64)
            || run["paraIdx"].as_u64() != Some(owner.1 as u64)
            || !run["cellContext"].is_null()
        {
            continue;
        }
        let positions = run["charX"].as_array().expect("실제 문자 전진");
        let text = run["text"].as_str().unwrap();
        assert_eq!(positions.len(), text.chars().count() + 1);
        characters.extend(
            text.chars().zip(positions).map(|(character, x)| {
                (character, run["x"].as_f64().unwrap() + x.as_f64().unwrap())
            }),
        );
    }
    let text: Vec<_> = characters.iter().map(|(c, _)| *c).collect();
    let token: Vec<_> = token.chars().collect();
    let matches: Vec<_> = text
        .windows(token.len())
        .enumerate()
        .filter_map(|(index, value)| (value == token).then_some(index))
        .collect();
    assert_eq!(matches.len(), 1, "소유 문단의 고유한 원문 토큰");
    characters[matches[0]].1
}

#[test]
fn issue_8160_sample16_2024_completed_page_keeps_bcp_tail_and_successor_at_print_origins() {
    let core = load();
    let bcp = source_owner(&core, "비상대응체계(BCP:BusinessContinuityPlanning)수립");
    let successor = source_owner(&core, "서버통합및재해복구센터목표시스템아키텍쳐");
    assert_eq!(successor, (bcp.0, bcp.1 + 1), "원본에서 바로 뒤따르는 본문");
    assert_only_page_three(&core, &[bcp, successor]);
    let page = core.build_page_render_tree(2).unwrap();
    let container = body(&page.root);
    let line = single_line(container, bcp);
    let next = single_line(container, successor);
    check_line(&core, line, bcp, container);
    check_line(&core, next, successor, container);
    assert!(
        line.bbox.y + line.bbox.height <= next.bbox.y,
        "꼬리와 다음 문단 비겹침"
    );
    // PDF의 수립 원점 (510.72,163.60)pt, 서버통합 (100.44,139.96)pt.
    near_print(token_origin(&core, bcp, "수립"), 680.96, "BCP 끝 글자 시작");
    near_print(
        token_origin(&core, successor, "서버통합"),
        133.92,
        "다음 본문 시작",
    );
    check_baselines(line, 903.20);
    check_baselines(next, 934.72);
}

#[test]
fn issue_8160_sample16_2024_page_tail_preserves_owned_paragraphs_and_empty_line_without_overlap() {
    let core = load();
    let start = source_owner(&core, "비상대응체계(BCP:BusinessContinuityPlanning)수립");
    let end = source_owner(
        &core,
        "활용가능기존장비는용도변경및재해복구센터(DR)활용방안수립",
    );
    assert_eq!(start.0, end.0);
    let paragraphs = &core.document().sections[start.0].paragraphs;
    let blank = (end.0, end.1 + 1);
    assert!(
        paragraphs[blank.1].text.trim().is_empty(),
        "원본의 후행 빈 문단"
    );
    let owners: Vec<_> = (start.1..=blank.1).map(|index| (start.0, index)).collect();
    assert_only_page_three(&core, &owners);
    let page = core.build_page_render_tree(2).unwrap();
    let container = body(&page.root);
    let mut previous_end = None;
    for owner in owners {
        let line = single_line(container, owner);
        if owner != blank {
            check_line(&core, line, owner, container);
        } else {
            assert!(
                line.children.iter().all(|node| {
                    matches!(&node.node_type, RenderNodeType::TextRun(run)
                        if normalized(&run.text).is_empty()
                            && run.display_text.as_deref().is_none_or(|text| normalized(text).is_empty()))
                }),
                "후행 빈 줄에 가시 내용을 만들지 않는다"
            );
            assert!(line.bbox.height > 0.0, "빈 줄도 원본 줄 상자를 보존한다");
            assert!(line.bbox.y + line.bbox.height <= container.bbox.y + container.bbox.height);
        }
        if let Some(end) = previous_end {
            assert!(
                end <= line.bbox.y,
                "마지막 본문·후행 빈 문단의 순서와 비겹침"
            );
        }
        previous_end = Some(line.bbox.y + line.bbox.height);
    }
}

/// The absorbed blank paragraph still belongs to the native editor caret API.
#[test]
fn issue_8160_sample16_2024_empty_tail_keeps_its_editor_caret_owner() {
    let core = load();
    let end = source_owner(
        &core,
        "활용가능기존장비는용도변경및재해복구센터(DR)활용방안수립",
    );
    let blank = (end.0, end.1 + 1);
    assert!(core.document().sections[blank.0].paragraphs[blank.1]
        .text
        .trim()
        .is_empty());
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(2)
            .expect("원본 후행 빈 문단의 편집 좌표"),
    )
    .unwrap();
    let carets: Vec<_> = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| {
            run["secIdx"] == blank.0 && run["paraIdx"] == blank.1 && run["cellIdx"].is_null()
        })
        .collect();
    assert_eq!(carets.len(), 1, "빈 문단의 편집 소유 누락·중복 방지");
    assert_eq!(carets[0]["charStart"], 0);
    assert_eq!(carets[0]["text"], "");
    // The empty line's hit-test frame may cover its available width. Its
    // character advances, rather than that frame, own the zero-length caret.
    assert_eq!(carets[0]["charX"], serde_json::json!([0.0]));
    assert!(carets[0]["h"].as_f64().unwrap() > 0.0);
    let page = core.build_page_render_tree(2).unwrap();
    let line = single_line(body(&page.root), blank);
    assert!(line.bbox.height > 0.0, "빈 원본 줄도 편집 공간을 가진다");
    let cursor: serde_json::Value = serde_json::from_str(
        &core
            .get_cursor_rect_native(blank.0, blank.1, 0)
            .expect("빈 원본 문단의 실제 커서 질의"),
    )
    .unwrap();
    assert_eq!(cursor["pageIndex"], 2, "원본 후행 문단의 쪽 소유");
    assert_eq!(cursor["x"], carets[0]["x"]);
    assert_eq!(cursor["y"], carets[0]["y"]);
    assert_eq!(cursor["height"], carets[0]["h"]);
}

#[test]
fn issue_8160_completed_page_six_keeps_the_evaluation_table_and_its_source_neighbors_in_the_print_frame(
) {
    // C417: 전64쪽 Native/fresh WASM >=90%, p6=98.16591/98.56406%, 직접 review/overlay.
    // 같은 원본/PDF SHA·192DPI PNG·96CSSpx 좌표·독립 PDF path 근거: stage8 C419.
    let core = load();
    let mut owners = Vec::new();
    for (section, data) in core.document().sections.iter().enumerate() {
        for (paragraph, para) in data.paragraphs.iter().enumerate() {
            for (control, item) in para.controls.iter().enumerate() {
                let Control::Table(table) = item else {
                    continue;
                };
                let content: String = table
                    .cells
                    .iter()
                    .flat_map(|cell| &cell.paragraphs)
                    .map(|para| normalized(&para.text))
                    .collect();
                if content.contains("기술능력평가")
                    && content.contains("입찰가격평가")
                    && content.contains("제출한가격입찰서")
                {
                    owners.push((section, paragraph, control));
                }
            }
        }
    }
    assert_eq!(owners.len(), 1, "내용으로 식별한 평가 비율 표의 원본 소유");
    let (section, paragraph, control) = owners[0];
    let paragraphs = &core.document().sections[section].paragraphs;
    assert!(
        normalized(&paragraphs[paragraph - 1].text).contains("기술제안서평가및입찰가격평가비율")
    );
    assert!(normalized(&paragraphs[paragraph + 1].text).contains("입찰가격평점산식"));

    fn collect<'a>(
        node: &'a RenderNode,
        owner: (usize, usize, usize),
        out: &mut Vec<&'a RenderNode>,
    ) {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if (table.section_index, table.para_index, table.control_index)
                == (Some(owner.0), Some(owner.1), Some(owner.2))
                && table.cell_context.is_none())
        {
            out.push(node);
        }
        for child in &node.children {
            collect(child, owner, out);
        }
    }

    let owner = (section, paragraph, control);
    let mut page_owners = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut found = Vec::new();
        collect(body(&tree.root), owner, &mut found);
        page_owners.extend(std::iter::repeat_n(page, found.len()));
    }
    assert_eq!(page_owners, [5], "독립 Print 6쪽의 표 소유·누락·중복 방지");
    let tree = core.build_page_render_tree(5).unwrap();
    let container = body(&tree.root);
    let mut found = Vec::new();
    collect(container, owner, &mut found);
    let table = found[0];
    let caption = single_line(container, (section, paragraph - 1));
    let following = single_line(container, (section, paragraph + 1));
    assert!(caption.bbox.y + caption.bbox.height <= table.bbox.y);
    assert!(table.bbox.y + table.bbox.height <= following.bbox.y);
    assert!(
        table.bbox.x >= container.bbox.x
            && table.bbox.x + table.bbox.width <= container.bbox.x + container.bbox.width
    );
    // PDF의 표 외곽 stroke 중심(pt→96CSSpx). 그림/글자 ROI로 대상을 선택하지 않는다.
    for (name, actual, expected) in [
        ("평가 비율 표 왼쪽", table.bbox.x, 117.697337),
        ("평가 비율 표 위쪽", table.bbox.y, 603.340007),
        (
            "평가 비율 표 오른쪽",
            table.bbox.x + table.bbox.width,
            709.218669,
        ),
        (
            "평가 비율 표 아래쪽",
            table.bbox.y + table.bbox.height,
            733.277344,
        ),
    ] {
        near_print(actual, expected, name);
    }
}

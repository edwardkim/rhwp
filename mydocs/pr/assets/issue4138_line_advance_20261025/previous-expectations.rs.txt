//! Issue #4138: 셀 나누기 후 stale line_segs·vpos 사다리 붕괴로 인한
//! glyph truncation + 잘못된 페이지네이션 회귀 가드.
//!
//! 재현 문서: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
//! (3×1 RowBreak 표가 PartialTable continuation 으로 115쪽에 걸침).
//!
//! 결함 2단:
//! 1. `Table::split_cell_into` 가 셀 폭을 절반으로 바꾸면서 저장 line_segs 를
//!    그대로 둔다 → 렌더러가 옛 폭(44508) 기준 줄을 새 셀(22395) 클립 경계에
//!    그대로 그려 glyph 가 잘린다. (수정 전 실측: 4,321 seg 전부 stale)
//! 2. per-para reflow 만 배선하면 각 문단의 vpos 원점이 보존된 채 줄 수만
//!    늘어나 사다리가 역행하고, 컷 기계가 이를 RowBreak hard break 로 오판해
//!    페이지를 과소 적재한다. (실측: 118→222쪽)
//!
//! 정정: `reflow_stale_cells_after_split`(table_ops.rs) — stale 셀 재래핑 후
//! `rebuild_table_cell_vpos_ladder_native` 로 사다리를 단조 재구축한다.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::table::Table;

const SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
/// 대상 표: (section 0, 문단 0, control 2). 분할 대상 셀: (row 2, col 0).
const CTRL_IDX: usize = 2;
const TARGET_ROW: u16 = 2;

fn split_body_paragraph_text(node: &serde_json::Value, para_index: u64) -> String {
    if node["type"] == "Cell"
        && node["row"].as_u64() == Some(u64::from(TARGET_ROW))
        && node["col"].as_u64() == Some(0)
    {
        return node["children"]
            .as_array()
            .expect("본문 셀의 줄")
            .iter()
            .filter(|line| line["type"] == "TextLine" && line["pi"] == para_index)
            .flat_map(|line| line["children"].as_array().expect("줄의 글자 런"))
            .filter_map(|run| run["text"].as_str())
            .collect();
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|child| split_body_paragraph_text(child, para_index))
        .collect()
}

fn source_node<'a>(
    node: &'a serde_json::Value,
    kind: &str,
    key: &str,
    owner: u64,
) -> Option<&'a serde_json::Value> {
    if node["type"] == kind && node[key].as_u64() == Some(owner) {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|child| source_node(child, kind, key, owner))
}

/// 현재 분할 저장본을 한컴 2024에서 Print한 독립 PDF는 1.1.3의
/// `는 주기관`부터 둘째 쪽에 둔다. 저장 사다리의 인접 줄은 물리 쪽 경계의
/// 증거가 아니므로, 글자 상자만 들어간다는 이유로 다음 쪽 줄을 앞당기지 않는다.
/// 원본 문단을 두 조각이 정확히 한 번 나누는지도 함께 검사한다.
#[test]
fn saved_split_keeps_the_first_continuation_line_on_its_own_page() {
    let mut doc = load();
    doc.split_table_cell_into_native(0, 0, CTRL_IDX, TARGET_ROW, 0, 1, 2, false, false)
        .expect("split_cell_into 1×2");
    let saved = saved_hwp(&doc);
    // 독립 한컴 Print의 1.4.1.5는 마지막 `다.`를 별도 줄에 둔다.
    // 새 셀의 실제 후속 줄 상자를 넘는 끝 토큰을 측정 공차로 수용하면
    // 다음 문단과 이후 쪽의 소속도 앞당겨진다.
    let body = target_table(&saved)
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 0)
        .expect("분할 원문 셀");
    // 독립 Print12쪽의 ℓ는 HFT 전각 기호이며 뒤의 `수면비행선박`은 다음 줄이다.
    let length = &body.paragraphs[151];
    let next_start = length.line_segs[1].text_start as usize;
    let first_length_line = String::from_utf16(
        &length
            .text
            .encode_utf16()
            .take(next_start)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert_eq!(
        first_length_line.trim_end(),
        "2.4.2.2 계류 로프의 길이 ℓ(미터)는",
        "독립 Print의 전각 기호와 다음 낱말 소속"
    );
    let tail = &body.paragraphs[31];
    let last_start = tail.line_segs.last().expect("문단 끝 줄").text_start;
    let tail_text = String::from_utf16(
        &tail
            .text
            .encode_utf16()
            .skip(last_start as usize)
            .collect::<Vec<_>>(),
    )
    .expect("줄의 원문 UTF-16");
    assert_eq!(tail_text, "다.", "독립 Print의 문단 끝 줄 소속");
    let source = &target_table(&saved)
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 0)
        .expect("분할 원문 셀")
        .paragraphs[9]
        .text;
    let boundary = source
        .find("는 주기관")
        .expect("독립 Print의 이어받기 내용");
    let normalize = |text: &str| {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let first: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(0).expect("첫 쪽 트리")).unwrap();
    let second: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(1).expect("둘째 쪽 트리")).unwrap();
    let first_text = normalize(&split_body_paragraph_text(&first, 9));
    let second_text = normalize(&split_body_paragraph_text(&second, 9));
    assert_eq!(
        first_text,
        normalize(&source[..boundary]),
        "첫 쪽의 종료 줄 소속"
    );
    assert_eq!(
        second_text,
        normalize(&source[boundary..]),
        "다음 쪽의 시작 줄 소속"
    );
    assert_eq!(
        first_text + &second_text,
        normalize(source),
        "문단의 누락·중복 없음"
    );
    // 이어받기 물리 조각은 끝 줄 뒤의 다음 줄 간격을 소유하지 않는다.
    // 독립 Print는 2.3.2.6의 첫 줄을 10쪽에, `이어져야`부터 11쪽에 둔다.
    // 독립 Print의 그림 1/2 뒤 본문은 그림·바깥여백·하단 캡션의 점유 끝 뒤다.
    // 폭 재조판이 그림 host를 일반 빈 줄로 만들면 이 저장 흐름을 잃는다.
    let picture_host = &body.paragraphs[310];
    let picture_origin = picture_host.line_segs[0].vertical_pos;
    let successor_origin = body.paragraphs[311].line_segs[0].vertical_pos;
    for control in &picture_host.controls {
        if let Control::Picture(picture) = control {
            let caption = picture.caption.as_ref().expect("독립 Print의 하단 캡션");
            let caption_height: i32 = caption
                .paragraphs
                .iter()
                .flat_map(|p| &p.line_segs)
                .map(|line| line.line_height)
                .sum();
            let occupied = picture.common.vertical_offset as i32
                + picture.common.height as i32
                + i32::from(picture.common.margin.top)
                + i32::from(picture.common.margin.bottom)
                + caption.spacing as i32
                + caption_height;
            assert!(
                successor_origin >= picture_origin + occupied,
                "그림·캡션 뒤 문단의 흐름 소유: successor={}, occupied_end={}",
                successor_origin,
                picture_origin + occupied
            );
        }
    }
    // 같은 저장본의 독립 Print24쪽은 그림1을 같은 쪽에 표시한다.
    // 자식 표 내부 나눔을 부모 frame 재시작으로 오인해 그림을 뒤쪽으로 보내지 않는다.
    fn has_image(node: &serde_json::Value) -> bool {
        node["type"] == "Image"
            || node["children"]
                .as_array()
                .is_some_and(|children| children.iter().any(has_image))
    }
    let picture_page: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(23).expect("그림1의24쪽 트리")).unwrap();
    assert!(has_image(&picture_page), "독립 Print24쪽의 그림 소유");
    let host_line = source_node(&picture_page, "TextLine", "pi", 310).expect("그림의 원문 앵커");
    let figure = source_node(&picture_page, "Image", "ci", 1).expect("그림1의 control 소유");
    assert!(
        figure["bbox"]["y"].as_f64().unwrap() >= host_line["bbox"]["y"].as_f64().unwrap(),
        "양수 문단 기준 그림 원점은 같은 쪽의 앵커 앞에 배치하지 않는다"
    );

    // 독립 Print25쪽의 침수율 표는 공백 줄 다음의 내어쓰기 줄이 소유한다.
    // 큰 표를 현재 셀 첫 줄의 왼쪽으로 되돌리지 않고 소유 줄의 가로 원점을 따른다.
    let twenty_fifth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(24).expect("25쪽 트리")).unwrap();
    let parent = source_node(&twenty_fifth, "Cell", "row", 2).unwrap();
    let inundation = source_node(parent, "Table", "pi", 322).expect("침수율 중첩 표");
    let host = &body.paragraphs[322];
    let control_position = host.control_text_positions()[0];
    let owner = host
        .line_segs
        .iter()
        .rposition(|line| line.text_start as usize <= control_position)
        .unwrap();
    assert_eq!(owner, 1, "표 앞 공백 줄은 표의 소유 줄이 아니다");
    let shape = &saved.document().doc_info.para_shapes[host.para_shape_id as usize];
    assert!(shape.indent < 0);
    let left = f64::from(shape.margin_left - shape.indent) * 96.0 / 7200.0 / 2.0;
    let Control::Table(child) = &host.controls[0] else {
        panic!("침수율 표 원본");
    };
    let expected = parent["bbox"]["x"].as_f64().unwrap()
        + f64::from(body.padding.left) * 96.0 / 7200.0
        + left
        + f64::from(child.outer_margin_left) * 96.0 / 7200.0;
    assert!(
        (inundation["bbox"]["x"].as_f64().unwrap() - expected).abs() <= 0.2,
        "같은 문단의 이어받기 줄 내어쓰기와 안/바깥 여백: actual={:?}, source={expected}",
        inundation["bbox"]
    );

    let continuation_source = &body.paragraphs[131].text;
    let continuation_boundary = continuation_source.find("이어져야").expect("이어받기 원문");
    let tenth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(9).expect("10쪽 트리")).unwrap();
    let eleventh: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(10).expect("11쪽 트리")).unwrap();
    let tenth_text = normalize(&split_body_paragraph_text(&tenth, 131));
    let eleventh_text = normalize(&split_body_paragraph_text(&eleventh, 131));
    assert_eq!(
        tenth_text,
        normalize(&continuation_source[..continuation_boundary]),
        "이어받기 종료 줄 소속"
    );
    assert_eq!(
        eleventh_text,
        normalize(&continuation_source[continuation_boundary..]),
        "다음 조각의 시작 줄 소속"
    );
    assert_eq!(
        tenth_text + &eleventh_text,
        normalize(continuation_source),
        "이어받기 문단의 누락·중복 없음"
    );
    // 동일 저장본의 독립 Print는 Fst 설명을 13쪽에서 시작한다.
    // 10쪽의 종료 줄을 수용하는 것과 12쪽의 다음 줄을 앞당기는 것은 별개다.
    let twelfth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(11).expect("12쪽 트리")).unwrap();
    let thirteenth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(12).expect("13쪽 트리")).unwrap();
    assert!(
        normalize(&split_body_paragraph_text(&twelfth, 165)).is_empty(),
        "후속 설명 문단은 이전 조각에 앞당기지 않는다"
    );
    assert_eq!(
        normalize(&split_body_paragraph_text(&thirteenth, 165)),
        normalize(&body.paragraphs[165].text),
        "다음 조각에서 설명 문단 전체를 보존한다"
    );
    let quote_line =
        source_node(&thirteenth, "TextLine", "pi", 176).expect("독립 Print13쪽의 따옴표 포함 문단");
    let quote_text: String = quote_line["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|run| run["text"].as_str())
        .collect();
    assert_eq!(
        normalize(&quote_text),
        normalize("3.1.1.1 \"예상최악조건\"이란 수면비행"),
        "독립 Print의 큰따옴표 폭과 실제 첫 줄 소속"
    );
    let twentieth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(19).expect("20쪽 트리")).unwrap();
    let twenty_first: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(20).expect("21쪽 트리")).unwrap();
    assert_eq!(
        normalize(&split_body_paragraph_text(&twentieth, 265)),
        normalize(&body.paragraphs[265].text),
        "독립 Print의 소제목은 뒤 문단 간격 때문에 다음 쪽으로 밀리지 않는다"
    );
    assert!(
        normalize(&split_body_paragraph_text(&twenty_first, 265)).is_empty(),
        "앞 조각의 소제목을 다음 조각에서 반복하지 않는다"
    );
    let forty_fourth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(43).expect("44쪽 트리")).unwrap();
    let forty_fifth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(44).expect("45쪽 트리")).unwrap();
    assert!(
        source_node(&forty_fourth, "TextLine", "pi", 558).is_some(),
        "독립 Print의 종료 빈 줄 상자는 앞 조각이 소유한다"
    );
    assert!(
        source_node(&forty_fifth, "TextLine", "pi", 558).is_none(),
        "종료 빈 줄을 다음 쪽 시작 공간으로 반복하지 않는다"
    );
    // 같은 입력의 독립 Print는 빈 문단1384를 105쪽 첫 슬롯에 둔다.
    // 빈 줄에도 물리 줄간 밴드가 있으므로 잉크가 없다는 이유로 종료 간격을
    // 전부 제외해 이전 조각에 넣으면 이후 본문이 한 줄씩 앞당겨진다.
    let hundred_fourth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(103).expect("104쪽 트리")).unwrap();
    let hundred_fifth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(104).expect("105쪽 트리")).unwrap();
    assert!(
        source_node(&hundred_fourth, "TextLine", "pi", 1384).is_none(),
        "독립 Print의 다음 쪽 시작 빈 줄을 앞 쪽에 흡수하지 않는다"
    );
    let opening_blank = source_node(&hundred_fifth, "TextLine", "pi", 1384)
        .expect("독립 Print105쪽의 첫 빈 줄 슬롯");
    let following =
        source_node(&hundred_fifth, "TextLine", "pi", 1385).expect("첫 빈 줄 뒤의 본문");
    let blank_slot = &body.paragraphs[1384].line_segs[0];
    let expected_advance =
        f64::from(blank_slot.line_height + blank_slot.line_spacing) * 96.0 / 7200.0;
    assert!(
        (following["bbox"]["y"].as_f64().unwrap()
            - opening_blank["bbox"]["y"].as_f64().unwrap()
            - expected_advance)
            .abs()
            <= 0.2,
        "빈 시작 줄의 실제 슬롯을 후속 본문이 소비한다"
    );
    fn collect_paragraph_lines(node: &serde_json::Value, owner: u64, out: &mut Vec<String>) {
        if node["type"] == "TextLine" && node["pi"].as_u64() == Some(owner) {
            out.push(
                node["children"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|run| run["text"].as_str())
                    .collect(),
            );
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                collect_paragraph_lines(child, owner, out);
            }
        }
    }
    // 같은 입력의 독립 Print는 `규정하고 있는 것과`부터 29쪽에 둔다.
    // 90쪽의 이어받기 종료 줄을 넣기 위해 정상 28쪽의 다음 줄까지 앞당기지 않는다.
    let twenty_eighth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(27).expect("28쪽 트리")).unwrap();
    let regulatory_source = &body.paragraphs[382].text;
    let regulatory_boundary = regulatory_source.find("규정하고 있는 것과").unwrap();
    assert_eq!(
        normalize(&split_body_paragraph_text(&twenty_eighth, 382)),
        normalize(&regulatory_source[..regulatory_boundary]),
        "독립 Print28쪽의 종료 내용 소속"
    );
    let twenty_ninth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(28).expect("29쪽 트리")).unwrap();
    let mut celsius_lines = Vec::new();
    collect_paragraph_lines(&twenty_ninth, 390, &mut celsius_lines);
    assert_eq!(
        normalize(&celsius_lines[1]),
        normalize("관 등 온도가 220℃를 초과하"),
        "독립 Print의 섭씨 기호 폭과 줄 종료 소속"
    );
    assert!(
        normalize(&celsius_lines[2]).starts_with('고'),
        "뒤 글자는 다음 줄이 소유한다"
    );
    let forty_ninth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(48).expect("49쪽 트리")).unwrap();
    let outer = source_node(&forty_ninth, "Table", "ci", 2).expect("분할된 바깥 표");
    let cells = outer["children"].as_array().unwrap();
    let cell = |col: u64| {
        cells
            .iter()
            .find(|node| {
                node["type"] == "Cell"
                    && node["row"].as_u64() == Some(2)
                    && node["col"].as_u64() == Some(col)
            })
            .expect("분할 행의 셀")
    };
    let left = cell(0);
    let right = cell(1);
    assert!(
        left["bbox"]["x"].as_f64().unwrap() + left["bbox"]["w"].as_f64().unwrap()
            <= right["bbox"]["x"].as_f64().unwrap(),
        "분할 전 host 줄 폭은 새 셀 경계를 넓힐 권한이 아니다"
    );
    let host = &body.paragraphs[626];
    assert_eq!(host.text, "    ");
    assert_eq!(host.control_text_positions(), vec![0]);
    let Control::Table(child) = &host.controls[0] else {
        panic!("공백 호스트의 블록 표");
    };
    let last_table = source_node(&forty_ninth, "Table", "pi", 626).unwrap();
    let successor = source_node(&forty_ninth, "TextLine", "pi", 627).unwrap();
    let host_line = &host.line_segs[0];
    let trailing_flow = f64::from(
        i32::from(child.outer_margin_bottom)
            + host_line.line_height
            + host_line.line_spacing.max(0),
    ) * 96.0
        / 7200.0;
    assert!(
        successor["bbox"]["y"].as_f64().unwrap()
            >= last_table["bbox"]["y"].as_f64().unwrap()
                + f64::from(child.common.height) * 96.0 / 7200.0
                + trailing_flow
                // Render-tree JSON rounds origins to one decimal place.
                - 0.1,
        "블록 표 뒤 공백 글줄은 바깥 아래 여백과 자기 줄 상자를 소유한다: table={:?}, next={:?}, slot={trailing_flow}, seg={host_line:?}, object={:?}",
        last_table["bbox"], successor["bbox"], child.common
    );
    let ninetieth: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(89).expect("90쪽 트리")).unwrap();
    let ninety_first: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(90).expect("91쪽 트리")).unwrap();
    let mut closing_lines = Vec::new();
    collect_paragraph_lines(&ninetieth, 1205, &mut closing_lines);
    let mut continuation_lines = Vec::new();
    collect_paragraph_lines(&ninety_first, 1205, &mut continuation_lines);
    assert_eq!(
        closing_lines.len(),
        4,
        "독립 Print90쪽은 8.2.2의 네 번째 줄까지 소유한다"
    );
    assert!(
        normalize(&continuation_lines[0]).starts_with("(SOLAS1974)"),
        "다음 쪽은 이미 앞 쪽에 속한 줄을 반복하지 않는다"
    );

    // 같은 분할 저장본의 독립 한컴 Print/재저장 LineSeg(HU)다.
    // 큰 수식이 줄간격 계산에 참여하지 않는 혼합 줄도 점유 높이는 보존한다.
    // Native/fresh WASM 전115/197쪽의 90% 선행 검증은 C32의 정확한 현재 생성본을 따른다.
    for (paragraph, height, baseline, spacing) in [
        (1685, 2106, 1411, 720),
        (1698, 2232, 1540, 1340),
        (1701, 1260, 1020, 720),
        (1704, 1386, 1040, 832),
    ] {
        let line = &body.paragraphs[paragraph].line_segs[0];
        assert_eq!(
            (line.line_height, line.baseline_distance, line.line_spacing),
            (height, baseline, spacing),
            "독립 한컴 수식 줄의 높이·기준선·간격: 문단{paragraph}"
        );
    }
    let equation_page: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(128).expect("수식의129쪽 트리")).unwrap();
    let equation_cell = source_node(&equation_page, "Cell", "row", u64::from(TARGET_ROW)).unwrap();
    let equation_lines: Vec<_> = equation_cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|line| line["type"] == "TextLine" && line["pi"].as_u64() == Some(1685))
        .collect();
    assert_eq!(equation_lines.len(), 2, "수식과 뒤 본문의 줄 소속");
    let independent_advance = f64::from(2106 + 720) * 96.0 / 7200.0;
    assert!(
        (equation_lines[1]["bbox"]["y"].as_f64().unwrap()
            - equation_lines[0]["bbox"]["y"].as_f64().unwrap()
            - independent_advance)
            .abs()
            <= 0.2,
        "최종 배치도 독립 줄 점유와 간격을 한 번 소비한다"
    );
}

/// 독립 Print49쪽의 내어쓰기 자동 탭은 첫 본문과 다음 글줄의 시작을 맞춘다.
/// 탭 앞 항목 번호·공백은 뒤 본문의 양쪽 정렬 여분을 받지 않는다.
#[test]
fn saved_split_preserves_the_hanging_auto_tab_anchor() {
    let mut doc = load();
    doc.split_table_cell_into_native(0, 0, CTRL_IDX, TARGET_ROW, 0, 1, 2, false, false)
        .unwrap();
    let saved = saved_hwp(&doc);
    let table = target_table(&saved);
    let body = table
        .cells
        .iter()
        .find(|c| c.row == TARGET_ROW && c.col == 0)
        .unwrap();
    let paragraph = &body.paragraphs[628];
    assert_eq!(paragraph.tab_extended[0][0], 32, "원본 탭의 작은 전진 폭");
    let page: serde_json::Value =
        serde_json::from_str(&saved.get_page_render_tree(48).unwrap()).unwrap();
    let cell = source_node(&page, "Cell", "row", u64::from(TARGET_ROW)).unwrap();
    let lines: Vec<_> = cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "TextLine" && n["pi"].as_u64() == Some(628))
        .collect();
    let first_body = lines[0]["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| {
            n["text"]
                .as_str()
                .is_some_and(|t| t.starts_with("절연구조"))
        })
        .unwrap();
    let hanging_start = lines[1]["bbox"]["x"].as_f64().unwrap();
    let original_tab_gap = f64::from(paragraph.tab_extended[0][0]) * 96.0 / 7200.0;
    assert!(
        (first_body["bbox"]["x"].as_f64().unwrap() - hanging_start).abs() <= original_tab_gap + 0.1,
        "첫 본문의 탭 원점과 다음 글줄의 내어쓰기 원점을 맞춘다"
    );
    assert!(
        first_body["text"]
            .as_str()
            .unwrap()
            .trim_end()
            .ends_with("절연저"),
        "독립 Print의 자동 탭과 첫 줄 종료 소속"
    );
}

/// 전115/197쪽 Native·fresh WASM 최저90% 이상, 마지막197쪽 양쪽99.11469%.
/// 같은 마지막 이어받기 동작을 보호하는 페이지 중 가장 높은 쪽을 선정했다.
/// 독립 Print의 마지막 본문 원점·종료 baseline은 CSS96 좌표로 ±2px를 허용한다.
/// 출처·쪽별 수치·확대 판독: stage5 C32 / regression-policy-20261015.
#[test]
fn saved_split_final_continuation_keeps_its_source_tail_and_print_origin() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn source_cell(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(0) && table.control_index == Some(CTRL_IDX)
                && table.cell_context.is_none())
        {
            return node.children.iter().find(|child| {
                matches!(&child.node_type, RenderNodeType::TableCell(cell)
                    if cell.row == TARGET_ROW && cell.col == 0)
            });
        }
        node.children.iter().find_map(source_cell)
    }
    fn text(node: &RenderNode) -> String {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            return run.display_or_text().to_owned();
        }
        node.children.iter().map(text).collect()
    }
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let mut doc = load();
    doc.split_table_cell_into_native(0, 0, CTRL_IDX, TARGET_ROW, 0, 1, 2, false, false)
        .unwrap();
    let mut saved = saved_hwp(&doc);
    saved.set_hangul2024_compat(true);
    assert_eq!(saved.page_count(), 197, "독립 Print의 마지막 쪽 소속");
    let body = target_table(&saved)
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 0)
        .unwrap();
    let owner = body
        .paragraphs
        .iter()
        .rposition(|p| !p.text.trim().is_empty())
        .unwrap();
    let source = body.paragraphs[owner].text.clone();
    assert!(source.ends_with("달한 것으로 간주한다."));
    let preceding = saved.build_page_render_tree(195).unwrap();
    let last = saved.build_page_render_tree(196).unwrap();
    let owned_lines = |cell: &RenderNode| {
        cell.children.iter().filter(|line| {
        matches!(&line.node_type, RenderNodeType::TextLine(data) if data.para_index == Some(owner))
    }).map(text).collect::<String>()
    };
    let before = owned_lines(source_cell(&preceding.root).unwrap());
    let cell = source_cell(&last.root).unwrap();
    let tail = owned_lines(cell);
    assert!(
        tail.starts_with("용률"),
        "독립 Print의 마지막 이어받기 시작 내용"
    );
    assert_eq!(
        normalize(&(before + &tail)),
        normalize(&source),
        "마지막 문단을 중복·누락 없이 분할"
    );
    let lines: Vec<_> = cell.children.iter().filter(|line| {
        matches!(&line.node_type, RenderNodeType::TextLine(data) if data.para_index == Some(owner))
    }).collect();
    assert_eq!(lines.len(), 6, "독립 Print의 마지막 여섯 줄 소속");
    let baseline = |line: &RenderNode| {
        let RenderNodeType::TextLine(data) = &line.node_type else {
            unreachable!()
        };
        line.bbox.y + data.baseline
    };
    // 독립 Print trace: x966×0.119935pt, y592/1392×0.119869pt.
    for (actual, independent) in [
        (lines[0].bbox.x, 154.47628),
        (baseline(lines[0]), 94.61659733),
        (baseline(lines[5]), 222.476864),
    ] {
        assert!(
            (actual - independent).abs() <= 2.0,
            "마지막 이어받기 원점/종료 baseline: 실제{actual}, 독립 Print{independent}"
        );
    }
    for pair in lines.windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "마지막 이어받기의 원문 줄 상자는 겹치지 않음"
        );
    }
    assert!(
        lines.iter().all(|line| line.bbox.y >= cell.bbox.y
            && line.bbox.y + line.bbox.height <= cell.bbox.y + cell.bbox.height),
        "마지막 줄의 종료가 원본 소유 셀 안에 있음"
    );
}

fn load() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read sample");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn target_table(doc: &rhwp::wasm_api::HwpDocument) -> &Table {
    match &doc.document().sections[0].paragraphs[0].controls[CTRL_IDX] {
        Control::Table(t) => t,
        other => panic!("controls[{CTRL_IDX}] 이 표가 아님: {other:?}"),
    }
}

/// 편집한 native HWP의 제품 경로는 저장 뒤 다시 여는 것이다. 메모리 `page_count()`는
/// serializer가 정규화하는 line segment/문단 상태를 거치지 않으므로, 한컴 PDF와의
/// 쪽수 oracle에는 저장본을 재파싱한 값만 사용한다.
fn saved_hwp(doc: &rhwp::wasm_api::HwpDocument) -> rhwp::wasm_api::HwpDocument {
    let bytes = doc.export_hwp_native().expect("분할 HWP 저장");
    if let Ok(dump) = std::env::var("RHWP_DUMP_4138") {
        std::fs::write(dump, &bytes).expect("분할 HWP 덤프");
    }
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("분할 HWP 재파싱")
}

fn paragraph_instance_id(para: &rhwp::model::paragraph::Paragraph) -> Option<u32> {
    para.raw_header_extra
        .get(6..10)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("instanceId 4바이트")))
}

/// `Cell::new_from_template`가 만든 오른쪽 빈 peer의 zeroed instanceId는 HWP5
/// 저장·재파싱 뒤에도 남는다. 이 값과 clone metadata가 renderer의 strict-cut
/// provenance이므로 실제 제품 저장 경계에서 직접 고정한다.
fn saved_hwp_page_count_and_split_provenance(doc: &rhwp::wasm_api::HwpDocument) -> u32 {
    let reparsed = saved_hwp(doc);
    let table = target_table(&reparsed);
    let source = table
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 0)
        .expect("분할 원문 셀");
    let peer = table
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 1)
        .expect("분할 빈 peer");
    let source_instance_id = paragraph_instance_id(&source.paragraphs[0])
        .expect("원문 셀 첫 문단은 instanceId 4바이트를 보존해야 한다");
    assert_ne!(
        source_instance_id, 0,
        "원문 셀 첫 문단은 0이 아닌 기존 instanceId를 보존해야 한다"
    );
    assert_eq!(
        paragraph_instance_id(&peer.paragraphs[0]),
        Some(0),
        "new_from_template 빈 peer는 저장 뒤에도 zeroed instanceId여야 한다"
    );
    reparsed.page_count()
}

/// 저장 seg 폭이 셀 폭을 넘는(=옛 폭 기준 stale) 문단 수.
///
/// 텍스트 없이 컨트롤만 호스팅하는 문단은 제외한다: `reflow_line_segs` 는 이
/// 문단에서 원본 seg 폭 template 을 보존하고, 중첩 표 자체를 분할 시 리스케일할지는
/// 한컴 오라클 미확인(#4138 미해결 항목 (a))이라 이 가드의 계약 밖이다.
fn stale_text_paras(table: &Table) -> usize {
    table
        .cells
        .iter()
        .flat_map(|cell| {
            cell.paragraphs
                .iter()
                .filter(|p| !(p.text.is_empty() && !p.controls.is_empty()))
                .map(move |p| (cell.width, p))
        })
        .filter(|(cell_width, p)| {
            p.line_segs
                .iter()
                .any(|ls| (ls.segment_width as i64) > (*cell_width as i64))
        })
        .count()
}

/// 분할 대상 행 셀들의 vpos 사다리 역행 수 (0 이어야 단조).
fn ladder_regressions(table: &Table, row: u16) -> usize {
    let mut regressions = 0usize;
    for cell in table.cells.iter().filter(|c| c.row == row) {
        let mut prev_end: i64 = i64::MIN;
        for p in &cell.paragraphs {
            if let (Some(first), Some(last)) = (p.line_segs.first(), p.line_segs.last()) {
                if (first.vertical_pos as i64) < prev_end {
                    regressions += 1;
                }
                prev_end = last.vertical_pos as i64;
            }
        }
    }
    regressions
}

/// 좁아진 셀에서 본문 뒤에 남은 폭을 넘는 inline control은 독립 줄을 가져야 한다.
///
/// 한컴 2020이 분할 HWP를 HWPX로 재저장한 오라클에서 p288(중간 nested table),
/// p322/p2001(nested table), p2286(trailing picture)는 모두 text prefix와 control
/// source line이 나뉜다. 이 경계가 합쳐지면 객체가 셀 우측 경계에서 clip되고 이후
/// RowBreak page owner도 달라진다.
fn split_inline_control_line_regressions(table: &Table, row: u16) -> usize {
    let Some(cell) = table
        .cells
        .iter()
        .find(|cell| cell.row == row && cell.col == 0)
    else {
        return 1;
    };
    [288usize, 322, 2001, 2286]
        .into_iter()
        .filter(|para_idx| {
            cell.paragraphs
                .get(*para_idx)
                .is_none_or(|para| para.line_segs.len() != 2)
        })
        .count()
}

/// 셀 나누기(1×2) 뒤: stale seg 0, 사다리 단조, 페이지 흐름 복원.
#[test]
fn split_cell_into_reflows_stale_segs_and_rebuilds_ladder() {
    let mut doc = load();
    assert_eq!(doc.page_count(), 115, "issue1949 기준 쪽수 핀");

    doc.split_table_cell_into_native(0, 0, CTRL_IDX, TARGET_ROW, 0, 1, 2, false, false)
        .expect("split_cell_into 1×2");

    let table = target_table(&doc);
    let stale = stale_text_paras(table);
    assert_eq!(
        stale, 0,
        "#4138 회귀: 분할 뒤 옛 폭 기준 stale line_segs 문단이 {stale}개 남음 \
         (수정 원복 시 실측 2,498문단/4,321seg). glyph 가 셀 클립 경계에서 잘린다."
    );

    let regressions = ladder_regressions(table, TARGET_ROW);
    assert_eq!(
        regressions, 0,
        "#4138 회귀: 재래핑된 셀의 vpos 사다리가 {regressions}회 역행. \
         컷 기계가 hard break 로 오판해 페이지를 과소 적재한다."
    );
    assert_eq!(
        split_inline_control_line_regressions(table, TARGET_ROW),
        0,
        "#4138 회귀: text + inline control source line이 다시 합쳐짐"
    );

    // 한글 2024 COM 재실측(2026-08-25): 같은 1×2 분할 저장본 197쪽. #6023 수정 뒤
    // 드러난 세로 축 잔존(196, 한글 197)은 #6046 이후의 row grow/near-top 보정과 함께
    // 다시 오라클 쪽수로 정합됐다. 이 핀은 분할-reflow 계약과 저장 뒤 재파싱 쪽수의
    // 동시 회귀 검출이 목적이므로 한글 2024 오라클(197)을 고정한다.
    let pages = saved_hwp_page_count_and_split_provenance(&doc);
    assert_eq!(
        pages, 197,
        "#4138 회귀: 저장 뒤 재파싱 쪽수 {pages} (한글 2024 실측 197)"
    );
}

/// 범위 분할 경로(`split_table_cells_in_range_native`)도 같은 계약을 따른다.
#[test]
fn split_cells_in_range_reflows_stale_segs() {
    let mut doc = load();

    doc.split_table_cells_in_range_native(
        0, 0, CTRL_IDX, TARGET_ROW, 0, TARGET_ROW, 0, 1, 2, false,
    )
    .expect("split_cells_in_range 1×2");

    let table = target_table(&doc);
    assert_eq!(
        stale_text_paras(table),
        0,
        "#4138 회귀(범위 분할): stale line_segs 잔존"
    );
    assert_eq!(
        ladder_regressions(table, TARGET_ROW),
        0,
        "#4138 회귀(범위 분할): vpos 사다리 역행"
    );
    assert_eq!(
        split_inline_control_line_regressions(table, TARGET_ROW),
        0,
        "#4138 회귀(범위 분할): text + inline control source line 재결합"
    );
    assert_eq!(
        saved_hwp_page_count_and_split_provenance(&doc),
        197,
        "#4138 회귀(범위 분할): 저장 뒤 재파싱 쪽수 불일치 (한글 2024 실측 197)"
    );
}

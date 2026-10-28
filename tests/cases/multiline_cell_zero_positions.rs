//! 저장 세로 위치가 모두 0인 두 줄은 한 줄 높이로 가운데 정렬하지 않는다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::model::table::VerticalAlign;
use serde_json::Value;

const FIXTURE: &[u8] = include_bytes!("../fixtures/multiline_cell_zero_positions/two_lines.hwpx");

fn check(core: &DocumentCore, align: VerticalAlign, height: u32) {
    let controls: Value =
        serde_json::from_str(&core.get_page_control_layout_native(0).unwrap()).unwrap();
    let cell = &controls["controls"][0]["cells"][0];
    let top = cell["y"].as_f64().unwrap();
    let bottom = top + f64::from(height) / 75.0;
    assert!(
        (cell["h"].as_f64().unwrap() - f64::from(height) / 75.0).abs() < 0.2,
        "내용이 꽉 찬 셀의 높이를 줄이지 않는다"
    );
    let text: Value = serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    let runs: Vec<_> = text["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| matches!(run["text"].as_str(), Some("First line" | "Second line")))
        .collect();
    assert_eq!(runs.len(), 2, "두 줄의 텍스트를 보존한다");
    let first = runs[0]["y"].as_f64().unwrap();
    let last = runs[1]["y"].as_f64().unwrap() + runs[1]["h"].as_f64().unwrap();
    assert!(
        first >= top && last <= bottom + 0.2,
        "{align:?}: 두 줄 {first}..{last}가 셀 {top}..{bottom} 안에 있어야 한다"
    );
    assert!(
        (runs[1]["y"].as_f64().unwrap() - first - 1000.0 / 75.0).abs() < 0.2,
        "두 줄의 간격은 저장된 1000 HU이다"
    );
    let expected = match align {
        VerticalAlign::Top => top + 140.0 / 75.0,
        VerticalAlign::Center => top + (f64::from(height) - 2000.0) / 150.0,
        VerticalAlign::Bottom => bottom - 2140.0 / 75.0,
    };
    assert!(
        (first - expected).abs() < 0.2,
        "{align:?}: first={first}, expected={expected}"
    );
}

#[test]
fn shrinking_table_uses_slack_without_clipping_two_lines() {
    let source = DocumentCore::from_bytes(FIXTURE).unwrap();
    let mut doc = source.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
        panic!("합성 표");
    };
    table.cells[0].vertical_align = VerticalAlign::Top;
    let mut slack = table.cells[0].clone();
    slack.row = 1;
    slack.height = 4200;
    let paragraph = &mut slack.paragraphs[0];
    paragraph.text = "Slack".into();
    paragraph.char_count = 6;
    paragraph.char_offsets = (0..5).collect();
    paragraph.line_segs.truncate(1);
    paragraph.source_line_seg_vertical_pos = None;
    table.cells.push(slack);
    table.row_count = 2;
    table.row_sizes = vec![1, 1];
    table.common.height = 5600;
    table.rebuild_grid();
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    check(&core, VerticalAlign::Top, 2280);
    let controls: Value =
        serde_json::from_str(&core.get_page_control_layout_native(0).unwrap()).unwrap();
    assert!(
        (controls["controls"][0]["h"].as_f64().unwrap() - 5600.0 / 75.0).abs() < 0.2,
        "여유 행을 줄여 표의 지정 높이를 유지한다"
    );
}

#[test]
fn duplicate_and_reset_segments_preserve_stored_shrink_floor() {
    // 같은 줄의 중복과 저장 page/column reset은 새 줄의 앵커 부재가 아니다.
    // 하한은 각 사례의 max(vpos + 1000 HU) + 상하 여백 280 HU이다.
    for (name, first_vpos, second_vpos, flags, duplicate, stored_floor) in [
        ("duplicate", 0, 1000, 0, true, 2280.0),
        ("position reset", 2000, 0, 0, false, 3280.0),
        (
            "page start",
            0,
            0,
            LineSeg::TAG_FIRST_LINE_OF_PAGE,
            false,
            1280.0,
        ),
        (
            "column start",
            0,
            0,
            LineSeg::TAG_FIRST_LINE_OF_COLUMN,
            false,
            1280.0,
        ),
    ] {
        let source = DocumentCore::from_bytes(FIXTURE).unwrap();
        let mut doc = source.document().clone();
        let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
            panic!("합성 표");
        };
        table.cells[0].vertical_align = VerticalAlign::Top;
        table.cells[0].height = 4200;
        let mut slack = table.cells[0].clone();
        slack.row = 1;
        let paragraph = &mut slack.paragraphs[0];
        paragraph.text = "Slack".into();
        paragraph.char_count = 6;
        paragraph.char_offsets = (0..5).collect();
        paragraph.line_segs.truncate(1);
        paragraph.source_line_seg_vertical_pos = None;
        let paragraph = &mut table.cells[0].paragraphs[0];
        paragraph.line_segs[0].vertical_pos = first_vpos;
        paragraph.line_segs[1].vertical_pos = second_vpos;
        paragraph.line_segs[1].tag |= flags;
        if duplicate {
            paragraph
                .line_segs
                .insert(1, paragraph.line_segs[0].clone());
        }
        paragraph.source_line_seg_vertical_pos = None;
        table.cells.push(slack);
        table.row_count = 2;
        table.row_sizes = vec![1, 1];
        table.common.height = 5600;
        table.rebuild_grid();
        let mut core = DocumentCore::new_empty();
        core.set_document(doc);
        let controls: Value =
            serde_json::from_str(&core.get_page_control_layout_native(0).unwrap()).unwrap();
        let table = &controls["controls"][0];
        // 8400 HU 중 2800 HU를 두 행의 저장 하한 위 여유에 비례해서 줄인다.
        let first_slack = 4200.0 - stored_floor;
        let second_slack = 4200.0 - 1280.0;
        let expected = (4200.0 - 2800.0 * first_slack / (first_slack + second_slack)) / 75.0;
        assert!(
            (table["cells"][0]["h"].as_f64().unwrap() - expected).abs() < 0.2,
            "{name}: 저장 줄의 중복·리셋으로 행의 축소 하한을 바꾸지 않는다"
        );
        assert!(
            (table["h"].as_f64().unwrap() - 5600.0 / 75.0).abs() < 0.2,
            "{name}: 표의 지정 높이를 유지한다"
        );
    }
}

#[test]
fn two_lines_with_zero_positions_keep_their_full_alignment_height() {
    for align in [
        VerticalAlign::Top,
        VerticalAlign::Center,
        VerticalAlign::Bottom,
    ] {
        for height in [2280, 4280] {
            for second_position in [0, 1000] {
                let source = DocumentCore::from_bytes(FIXTURE).unwrap();
                let mut doc = source.document().clone();
                let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
                    panic!("합성 표");
                };
                table.common.height = height;
                table.cells[0].height = height;
                table.cells[0].vertical_align = align;
                table.cells[0].paragraphs[0].line_segs[1].vertical_pos = second_position;
                let mut core = DocumentCore::new_empty();
                core.set_document(doc);
                check(&core, align, height);
                let reopened =
                    DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
                check(&reopened, align, height);
            }
        }
    }
}

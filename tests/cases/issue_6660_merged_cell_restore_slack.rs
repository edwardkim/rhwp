//! [#6660] 병합 칸 보호 뒤 남은 단일행 여유가 후속 표를 밀지 않아야 한다.
//!
//! exam_science.hwp 1쪽 문단 23의 표는 선언 131.6px에 맞춰 축소한 뒤
//! 병합 제목 칸을 복원하면서 135.939px로 다시 커졌다. 본문 행에는 여전히
//! 2.472px의 여유가 남았다. 여유 회수와 함께, 선언에 딱 맞는 병합 제목에는
//! 비활성 fallback 하단 여백을 추가하지 않는다. PDF 위치 계약은 별도 테스트다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::{Table, VerticalAlign};
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use rhwp::renderer::style_resolver::resolve_styles;
use rhwp::DocumentCore;

fn measure(para_index: usize, declared_height: Option<u32>) -> MeasuredTable {
    measure_with(para_index, |table| {
        if let Some(height) = declared_height {
            table.common.height = height;
        }
    })
}

fn measure_with(para_index: usize, edit: impl FnOnce(&mut Table)) -> MeasuredTable {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_science.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("시험 원본")).expect("HWP 열기");
    let doc = core.document();
    let mut para = doc.sections[0].paragraphs[para_index].clone();
    let Control::Table(table) = &mut para.controls[0] else {
        panic!("인라인 표가 있어야 한다");
    };
    edit(table);
    let composed = compose_paragraph(&para);
    let styles = resolve_styles(&doc.doc_info, 96.0);
    HeightMeasurer::new(96.0)
        .with_native_hwp5(true)
        .measure_section(&[para], &[composed], &styles, None)
        .get_measured_table(0, 0)
        .expect("표 측정")
        .clone()
}

#[test]
fn issue_6660_reclaims_slack_without_shrinking_merged_or_body_content() {
    // 제목 글자와 상단 여백은 보존한다. 비활성 하단 여백은 선언을 늘리지 않는다.
    let merged_floor = (1150.0 + 141.0) / 75.0;
    for (para_index, body_end, declared) in
        [(23, 4298.0 + 2580.0, 9870.0), (30, 3894.0 + 1148.0, 8032.0)]
    {
        let measured = measure(para_index, None);
        let body_floor = (body_end + 1700.0) / 75.0;
        assert_eq!(measured.row_heights.len(), 3);
        assert!(measured.row_heights[0] + measured.row_heights[1] >= merged_floor - 0.5);
        assert!(
            measured.row_heights[2] >= body_floor - 0.01,
            "문단 {para_index}: 남는 여유만 회수해야 한다: {:?}",
            measured.row_heights
        );
        assert!(
            measured.total_height <= declared / 75.0 + 1.0 / 75.0 + 0.01,
            "문단 {para_index}: fallback 여백이 후속 내용을 밀었다: {:?}",
            measured.row_heights
        );
    }
}

#[test]
fn issue_6660_stops_reclaiming_at_declared_height_when_slack_is_sufficient() {
    let measured = measure(23, Some(10200));
    assert!((measured.total_height - 136.0).abs() < 0.01);
    assert!(measured.row_heights[2] > (6878.0 + 1700.0) / 75.0);
    assert!(measured.row_heights[0] + measured.row_heights[1] >= 1291.0 / 75.0 - 0.5);
}

#[test]
fn issue_6660_preserves_explicit_padding_and_real_content_overflow() {
    for variant in 0..6 {
        let measured = measure_with(23, |table| {
            if variant == 1 {
                table.padding.top = 141;
                table.padding.bottom = 141;
            }
            let cell = &mut table.cells[1];
            match variant {
                0 => cell.apply_inner_margin = true,
                1 => {}
                2 => cell.height -= 10,
                3 => cell.vertical_align = VerticalAlign::Center,
                4 => cell.paragraphs[0].line_segs[0].text_height -= 1,
                5 => {
                    let mut line = cell.paragraphs[0].line_segs[0].clone();
                    line.vertical_pos = 1150;
                    cell.paragraphs[0].line_segs.push(line);
                }
                _ => unreachable!(),
            }
        });
        assert!(
            measured.row_heights[0] + measured.row_heights[1] >= 1432.0 / 75.0 - 0.5,
            "변형 {variant}: 명시적 여백/실제 내용 하한을 축소했다: {:?}",
            measured.row_heights
        );
    }
}

#[test]
fn issue_6660_does_not_force_stale_small_declarations_onto_real_content() {
    // 선언이 내용의 2/3 미만이면 #1835 보호가 우선한다. 축소하지 않은 표는
    // 병합 복원 뒤의 잔여 여유 회수 대상도 아니다.
    let measured = measure(23, Some(4000));
    assert!((measured.total_height - 10526.0 / 75.0).abs() < 0.01);
}

#[test]
fn issue_6660_preserves_following_picture_table_row_heights() {
    let measured = measure(28, None);
    assert_eq!(measured.row_heights.len(), 2);
    assert!((measured.row_heights[0] - (4470.0 + 282.0) / 75.0).abs() < 0.01);
    assert!((measured.row_heights[1] - 1432.0 / 75.0).abs() < 0.01);
}

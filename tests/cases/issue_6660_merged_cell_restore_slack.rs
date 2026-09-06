//! [#6660] 병합 칸 보호 뒤 남은 단일행 여유가 후속 표를 밀지 않아야 한다.
//!
//! exam_science.hwp 1쪽 문단 23의 표는 선언 131.6px에 맞춰 축소한 뒤
//! 병합 제목 칸을 복원하면서 135.939px로 다시 커졌다. 본문 행에는 여전히
//! 2.472px의 여유가 남았다. 저장 내용+여백 하한은 보존하되 이 여유만 회수한다.
//! 한컴 PDF와의 전체 정합을 주장하는 핀이 아니라 높이 회계의 회귀 계약이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use rhwp::renderer::style_resolver::resolve_styles;
use rhwp::DocumentCore;

fn measure(para_index: usize, declared_height: Option<u32>) -> MeasuredTable {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_science.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("시험 원본")).expect("HWP 열기");
    let doc = core.document();
    let mut para = doc.sections[0].paragraphs[para_index].clone();
    if let Some(height) = declared_height {
        let Control::Table(table) = &mut para.controls[0] else {
            panic!("인라인 표가 있어야 한다");
        };
        table.common.height = height;
    }
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
    // 병합 제목: 저장 줄 1150 + 상하 여백 141+141 HU.
    let merged_floor = (1150.0 + 282.0) / 75.0;
    for (para_index, body_end) in [(23, 4298.0 + 2580.0), (30, 3894.0 + 1148.0)] {
        let measured = measure(para_index, None);
        let body_floor = (body_end + 1700.0) / 75.0;
        assert_eq!(measured.row_heights.len(), 3);
        assert!((measured.row_heights[0] + measured.row_heights[1] - merged_floor).abs() < 0.01);
        assert!(
            (measured.row_heights[2] - body_floor).abs() < 0.01,
            "문단 {para_index}: 남는 여유만 회수해야 한다: {:?}",
            measured.row_heights
        );
        assert!((measured.total_height - merged_floor - body_floor).abs() < 0.01);
    }
}

#[test]
fn issue_6660_stops_reclaiming_at_declared_height_when_slack_is_sufficient() {
    let measured = measure(23, Some(10200));
    assert!((measured.total_height - 136.0).abs() < 0.01);
    assert!(measured.row_heights[2] > (6878.0 + 1700.0) / 75.0);
    assert!((measured.row_heights[0] + measured.row_heights[1] - 1432.0 / 75.0).abs() < 0.01);
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

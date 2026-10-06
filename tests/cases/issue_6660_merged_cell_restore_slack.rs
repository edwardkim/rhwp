//! [#6660] 병합 제목의 물리 하한과 본문 점유량을 함께 보존한다.
//!
//! exam_science.hwp 1쪽 문단23의 원문 필요량은9870HU(131.6px)다.
//! 원본 저장 글줄 끝·명시적 본문 여백·두 제목 행으로 독립 계산한다.
//! 제목에 비활성 하단 여백을 추가하거나 완전 셀 마지막 줄간격을 중복하면
//! 다음 그림과 표가 밀린다. 일반 측정과 TAC 축소 뒤 복원은 같은 하한을 쓴다.
//! 정상 PDF 전체4쪽 비교와 그림 위치 계약은 별도 증거이며,
//! common.height만 수동 변경한 두 대조군은 정상 생성본으로 간주하지 않는다.
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
    // 제목 글줄1150HU는 원문 두 행의 물리 높이646HU씩 안에 들어간다.
    // hasMargin=false인 셀의 보존 여백은 상하 모두 선언을 늘리지 않는다.
    let merged_floor = 2.0 * 646.0 / 75.0;
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
    // common.height만 수동 변경한 합성 대조군이다. 저장 셀·글줄은 원문 그대로다.
    // 마지막 줄간격을 높이로 가산하거나 비활성 여백으로 키우지 않는다.
    // 원문 물리 하한: 앞 두 행646HU씩 + 본문 끝6878HU + 명시적 여백1700HU.
    let stored_required = (2.0 * 646.0 + 6878.0 + 1700.0) / 75.0;
    assert!((measured.total_height - stored_required).abs() < 0.01);
    assert!((measured.row_heights[2] - (6878.0 + 1700.0) / 75.0).abs() < 0.01);
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
        // 정상 원문 전체4쪽 Native/fresh WASM의 최저91.87033% 확인 뒤
        // 잘못된 비활성 여백 핀을 수정한다. 이 변형들은 수동 합성 계약이다.
        let required_hu = match variant {
            // 실제 활성 셀 여백 또는 명시적 표 여백:1150+141+141HU.
            0 | 1 => 1432.0,
            // 높이·정렬·textheight만 바꿔도 비활성 여백은 활성화되지 않는다.
            // 나머지 단일 행 셀의 원문 높이646HU씩을 보존한다.
            2..=4 => 2.0 * 646.0,
            // 추가한 둘째 줄의 실제 끝은1150+1150HU다. 기존1432HU 검사는
            // 이 내용이 잘려도 통과할 수 있었으므로 실제 점유량을 요구한다.
            5 => 2.0 * 1150.0,
            _ => unreachable!(),
        };
        assert!(
            measured.row_heights[0] + measured.row_heights[1] >= required_hu / 75.0 - 0.5,
            "변형 {variant}: 명시적 여백/실제 내용 하한을 축소했다: {:?}",
            measured.row_heights
        );
        assert!(measured.row_heights[2] >= (6878.0 + 1700.0) / 75.0 - 0.01);
    }
}

#[test]
fn issue_6660_does_not_force_stale_small_declarations_onto_real_content() {
    // 선언이 내용의 2/3 미만이면 #1835 보호가 우선한다. 축소하지 않은 표는
    // 병합 복원 뒤의 잔여 여유 회수 대상도 아니다.
    let measured = measure(23, Some(4000));
    // 구 기대값10526HU는 마지막 줄간격과 비활성 여백까지 실제 내용으로 셌다.
    // 저장 글줄·행·명시적 여백으로 독립적으로 얻은 필요량은9870HU다.
    let stored_required = (2.0 * 646.0 + 6878.0 + 1700.0) / 75.0;
    assert!((measured.total_height - stored_required).abs() < 0.01);
}

#[test]
fn issue_6660_preserves_following_picture_table_row_heights() {
    let measured = measure(28, None);
    assert_eq!(measured.row_heights.len(), 2);
    assert!((measured.row_heights[0] - (4470.0 + 282.0) / 75.0).abs() < 0.01);
    assert!((measured.row_heights[1] - 1432.0 / 75.0).abs() < 0.01);
}

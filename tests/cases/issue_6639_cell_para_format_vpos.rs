//! #6639 — 셀 문단 줄 간격 batch 적용 후 후속 문단 vpos 재계산.
//!
//! `apply_para_format_in_cell_native`는 대상 문단의 LineSeg만 리플로우하고
//! 후속 문단 사다리를 다시 만들지 않는다 — 셀 텍스트 입력/삭제 경로의
//! `recalculate_cell_paragraph_vpos_native` 호출이 서식 경로에 없다. 줄 간격을
//! 160% → 140%로 낮추면 각 문단 높이는 줄지만 후속 시작 위치가 옛 값에 남아
//! 문단 사이 공백이 커지고, 텍스트 편집 왕복(공백 입력/삭제)이 뒤늦게 정상화한다.
//!
//! 검출은 한컴 대조 없이 자기 정합으로 한다: 서식 적용 직후 vpos가 텍스트 편집
//! 왕복 뒤와 같아야 한다. 수정 전에는 왕복이 위치를 바꾸므로 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use serde_json::Value;

const PARA_COUNT: usize = 4;

/// 1행 1열 표를 만들고 (부모 문단, 컨트롤) 좌표를 돌려준다.
fn create_single_cell(core: &mut DocumentCore) -> (usize, usize) {
    core.create_blank_document_native().expect("빈 문서 생성");
    let created = core
        .create_table_native(0, 0, 0, 1, 1)
        .expect("1x1 표 생성");
    let parsed: Value = serde_json::from_str(&created).expect("createTable JSON");
    (
        parsed["paraIdx"].as_u64().expect("paraIdx") as usize,
        parsed["controlIdx"].as_u64().expect("controlIdx") as usize,
    )
}

/// 셀 문단들의 줄 `vertical_pos` 전체를 순서대로 읽는다.
fn cell_vpos(core: &DocumentCore, para: usize, ctrl: usize) -> Vec<i32> {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("표 컨트롤이어야 함");
    };
    table.cells[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.line_segs.iter().map(|seg| seg.vertical_pos))
        .collect()
}

/// 셀 문단들의 `para_shape_id`를 순서대로 읽는다.
fn cell_shape_ids(core: &DocumentCore, para: usize, ctrl: usize) -> Vec<u16> {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("표 컨트롤이어야 함");
    };
    table.cells[0]
        .paragraphs
        .iter()
        .map(|p| p.para_shape_id)
        .collect()
}

/// 텍스트 한 글자를 넣었다 지우는 왕복 — 편집 경로의 vpos 재계산을 강제한다.
fn text_roundtrip(core: &mut DocumentCore, para: usize, ctrl: usize) {
    core.insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, " ")
        .expect("공백 입력");
    core.delete_text_in_cell_native(0, para, ctrl, 0, 0, 0, 1)
        .expect("공백 삭제");
}

/// 이 fixture는 문단 앞뒤 간격이 0이므로 모든 경계 틈도 정확히 0이어야 한다.
fn boundary_gaps(core: &DocumentCore, para: usize, ctrl: usize) -> Vec<i32> {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("표 컨트롤이어야 함");
    };
    let paras = &table.cells[0].paragraphs;
    paras
        .windows(2)
        .map(|pair| {
            let prev_last = pair[0].line_segs.last().expect("이전 문단 줄");
            let bottom = prev_last.vertical_pos + prev_last.line_height + prev_last.line_spacing;
            pair[1]
                .line_segs
                .first()
                .expect("다음 문단 줄")
                .vertical_pos
                - bottom
        })
        .collect()
}

fn build_four_para_cell(core: &mut DocumentCore, para: usize, ctrl: usize) {
    core.insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "첫째 문단 내용")
        .expect("문단0 입력");
    for idx in 1..PARA_COUNT {
        let prev_len = core
            .get_cell_paragraph_length_native(0, para, ctrl, 0, idx - 1)
            .expect("문단 길이");
        core.split_paragraph_in_cell_native(0, para, ctrl, 0, idx - 1, prev_len, None)
            .expect("셀 문단 분할");
        core.insert_text_in_cell_native(0, para, ctrl, 0, idx, 0, &format!("문단{idx} 내용"))
            .expect("분할 문단 입력");
    }
    assert_eq!(
        core.get_cell_paragraph_count_native(0, para, ctrl, 0)
            .expect("문단 수"),
        PARA_COUNT
    );
    // 편집 경로 재계산으로 기준 사다리를 확정한다.
    text_roundtrip(core, para, ctrl);
}

#[test]
fn batch_line_spacing_shrink_keeps_vpos_ladder_continuous() {
    let mut core = DocumentCore::new_empty();
    let (para, ctrl) = create_single_cell(&mut core);
    build_four_para_cell(&mut core, para, ctrl);

    // 160% 기준선을 깔고 편집 왕복으로 확정한다.
    for idx in 0..PARA_COUNT {
        core.apply_para_format_in_cell_native(
            0,
            para,
            ctrl,
            0,
            idx,
            r#"{"lineSpacing":160,"lineSpacingType":"Percent"}"#,
        )
        .expect("160% 적용");
    }
    text_roundtrip(&mut core, para, ctrl);
    let base_vpos = cell_vpos(&core, para, ctrl);
    let base_ids = cell_shape_ids(&core, para, ctrl);
    assert_eq!(
        boundary_gaps(&core, para, ctrl),
        vec![0; PARA_COUNT - 1],
        "160% 기준선은 연속 사다리여야 한다"
    );

    // Studio ApplyParaFormatCommand와 같은 batch로 140%를 적용한다.
    core.begin_batch_native().expect("배치 시작");
    for idx in 0..PARA_COUNT {
        core.apply_para_format_in_cell_native(
            0,
            para,
            ctrl,
            0,
            idx,
            r#"{"lineSpacing":140,"lineSpacingType":"Percent"}"#,
        )
        .expect("140% 적용");
    }
    core.end_batch_native().expect("배치 종료");

    // 변경이 실제로 줄 높이에 반영됐는지 확인한다 (무연산 통과 방지).
    let new_ids = cell_shape_ids(&core, para, ctrl);
    for (para_idx, id) in new_ids.iter().enumerate() {
        let shape = &core.document().doc_info.para_shapes[*id as usize];
        assert_eq!(shape.line_spacing, 140, "문단{para_idx} 줄 간격 140% 반영");
    }
    let after_vpos = cell_vpos(&core, para, ctrl);
    assert!(
        *after_vpos.last().unwrap() < *base_vpos.last().unwrap(),
        "줄 간격을 낮추면 마지막 줄이 위로 올라와야 한다"
    );
    assert_eq!(
        boundary_gaps(&core, para, ctrl),
        vec![0; PARA_COUNT - 1],
        "140% 적용 직후에도 경계 간격이 균일해야 한다 (stale vpos면 어긋난다)"
    );

    // 적용 직후 텍스트 왕복은 vpos를 더 이상 바꾸지 않아야 한다.
    text_roundtrip(&mut core, para, ctrl);
    assert_eq!(
        cell_vpos(&core, para, ctrl),
        after_vpos,
        "서식 적용 직후 vpos가 이미 최종이어야 한다 — 왕복 후 이동은 재계산 누락이다"
    );

    // undo 경로(set_cell_para_shape_id)도 사다리를 되돌려야 한다.
    for (idx, id) in base_ids.iter().enumerate() {
        core.set_cell_para_shape_id_native(0, para, ctrl, 0, idx, *id)
            .expect("모양 ID 복원");
    }
    assert_eq!(
        cell_vpos(&core, para, ctrl),
        base_vpos,
        "undo 복원 뒤 vpos가 기준선으로 돌아와야 한다"
    );
    assert_eq!(
        boundary_gaps(&core, para, ctrl),
        vec![0; PARA_COUNT - 1],
        "undo 뒤에도 경계 간격이 균일해야 한다"
    );
}

/// 저장된 한 줄 문단을 모사한다. 10pt 높이 + 160% 줄 간격 = 1600 HWPUNIT.
fn stored_cell(positions: &[i32]) -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    let (para, ctrl) = create_single_cell(&mut core);
    core.insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "문단")
        .unwrap();
    core.apply_para_format_in_cell_native(
        0,
        para,
        ctrl,
        0,
        0,
        r#"{"lineSpacing":160,"spacingBefore":0,"spacingAfter":0}"#,
    )
    .unwrap();
    let Control::Table(table) =
        &mut core.document_mut().sections[0].paragraphs[para].controls[ctrl]
    else {
        panic!("table");
    };
    let template = table.cells[0].paragraphs[0].clone();
    table.cells[0].paragraphs = positions
        .iter()
        .map(|&vpos| {
            let mut paragraph = template.clone();
            assert_eq!(paragraph.line_segs.len(), 1);
            let line = &mut paragraph.line_segs[0];
            line.vertical_pos = vpos;
            line.line_height = 1000;
            line.text_height = 1000;
            line.baseline_distance = 850;
            line.line_spacing = 600;
            line.tag &= !rhwp::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
            paragraph
        })
        .collect();
    (core, para, ctrl)
}

fn paragraphs(core: &DocumentCore, para: usize, ctrl: usize) -> &[Paragraph] {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("table");
    };
    &table.cells[0].paragraphs
}

#[test]
fn batch_defers_cell_ladder_until_end_and_uses_earliest_change() {
    let positions: Vec<_> = (0..1024).map(|i| i * 1600).collect();
    let (mut core, para, ctrl) = stored_cell(&positions);
    core.begin_batch_native().unwrap();
    // 역순과 중복 선택도 조각의 첫 변경부터 한 번만 처리한다.
    for idx in (0..positions.len()).rev().chain([512, 0]) {
        core.apply_para_format_in_cell_native(0, para, ctrl, 0, idx, r#"{"spacingAfter":200}"#)
            .unwrap();
        assert_eq!(
            paragraphs(&core, para, ctrl).last().unwrap().line_segs[0].vertical_pos,
            *positions.last().unwrap(),
            "setter가 전체 셀을 다시 순회하면 안 된다"
        );
    }
    core.end_batch_native().unwrap();
    // ParaShape 간격은 2배 스케일로 저장된다: spacingAfter=200 -> 100 HWPUNIT.
    assert_eq!(
        cell_vpos(&core, para, ctrl),
        (0..1024).map(|i| i * 1700).collect::<Vec<_>>()
    );
    core.begin_batch_native().unwrap();
    core.end_batch_native().unwrap();
    assert_eq!(
        cell_vpos(&core, para, ctrl),
        (0..1024).map(|i| i * 1700).collect::<Vec<_>>()
    );
}

#[test]
fn alignment_and_its_undo_preserve_stored_vpos() {
    let positions = [100, 1900, 3800, 5900];
    for batched in [false, true] {
        let (mut core, para, ctrl) = stored_cell(&positions);
        let ids = cell_shape_ids(&core, para, ctrl);
        if batched {
            core.begin_batch_native().unwrap();
        }
        for idx in 0..positions.len() {
            core.apply_para_format_in_cell_native(
                0,
                para,
                ctrl,
                0,
                idx,
                r#"{"alignment":"center"}"#,
            )
            .unwrap();
        }
        if batched {
            core.end_batch_native().unwrap();
        }
        assert_eq!(cell_vpos(&core, para, ctrl), positions);
        if batched {
            core.begin_batch_native().unwrap();
        }
        for (idx, id) in ids.into_iter().enumerate() {
            core.set_cell_para_shape_id_native(0, para, ctrl, 0, idx, id)
                .unwrap();
        }
        if batched {
            core.end_batch_native().unwrap();
        }
        assert_eq!(cell_vpos(&core, para, ctrl), positions);
    }
}

#[test]
fn rowbreak_fragments_keep_origins_and_only_changed_fragments_move() {
    let positions = [100, 1700, 3300, 200, 1800, 3400, 80, 2000];
    for batched in [false, true] {
        let (mut core, para, ctrl) = stored_cell(&positions);
        let ids = cell_shape_ids(&core, para, ctrl);
        if batched {
            core.begin_batch_native().unwrap();
        }
        for idx in [4, 3, 1] {
            core.apply_para_format_in_cell_native(
                0,
                para,
                ctrl,
                0,
                idx,
                r#"{"spacingBefore":100,"spacingAfter":200}"#,
            )
            .unwrap();
        }
        if batched {
            core.end_batch_native().unwrap();
        }
        // 조각 첫 문단의 저장 원점(100/200/80), 변경하지 않은 마지막 조각의 틈을 보존한다.
        assert_eq!(
            cell_vpos(&core, para, ctrl),
            [100, 1750, 3450, 200, 1950, 3650, 80, 2000]
        );
        let formatted_ids = cell_shape_ids(&core, para, ctrl);
        if batched {
            core.begin_batch_native().unwrap();
        }
        for idx in [1, 3, 4] {
            core.set_cell_para_shape_id_native(0, para, ctrl, 0, idx, ids[idx])
                .unwrap();
        }
        if batched {
            core.end_batch_native().unwrap();
        }
        assert_eq!(cell_vpos(&core, para, ctrl), positions);
        if batched {
            core.begin_batch_native().unwrap();
        }
        for idx in [1, 3, 4] {
            core.set_cell_para_shape_id_native(0, para, ctrl, 0, idx, formatted_ids[idx])
                .unwrap();
        }
        if batched {
            core.end_batch_native().unwrap();
        }
        assert_eq!(
            cell_vpos(&core, para, ctrl),
            [100, 1750, 3450, 200, 1950, 3650, 80, 2000]
        );
    }
}

#[test]
fn line_spacing_reflow_preserves_rowbreak_origins() {
    for batched in [false, true] {
        let (mut core, para, ctrl) = stored_cell(&[100, 1700, 3300, 200, 1800, 3400]);
        if batched {
            core.begin_batch_native().unwrap();
        }
        for idx in 0..6 {
            core.apply_para_format_in_cell_native(0, para, ctrl, 0, idx, r#"{"lineSpacing":140}"#)
                .unwrap();
        }
        if batched {
            core.end_batch_native().unwrap();
        }
        assert_eq!(
            cell_vpos(&core, para, ctrl),
            [100, 1500, 2900, 200, 1600, 3000]
        );
    }
}

#[test]
fn snapshot_restore_discards_pending_format_positions() {
    let positions = [100, 1900, 3800, 5900];
    let (mut core, para, ctrl) = stored_cell(&positions);
    let snapshot = core.save_snapshot_native();
    core.begin_batch_native().unwrap();
    core.apply_para_format_in_cell_native(0, para, ctrl, 0, 0, r#"{"spacingAfter":200}"#)
        .unwrap();
    core.restore_snapshot_native(snapshot).unwrap();
    core.end_batch_native().unwrap();
    assert_eq!(cell_vpos(&core, para, ctrl), positions);
}

#[test]
fn mixed_batch_insertion_keeps_pending_cell_format() {
    let (mut core, para, ctrl) = stored_cell(&[0, 1600, 3200]);
    core.begin_batch_native().unwrap();
    core.apply_para_format_in_cell_native(0, para, ctrl, 0, 0, r#"{"spacingAfter":200}"#)
        .unwrap();
    core.insert_paragraph_native(0, para).unwrap();
    core.end_batch_native().unwrap();
    assert_eq!(cell_vpos(&core, para + 1, ctrl), [0, 1700, 3300]);
}

#[test]
fn snapshot_inside_batch_contains_completed_formatting() {
    let (mut core, para, ctrl) = stored_cell(&[0, 1600, 3200]);
    core.begin_batch_native().unwrap();
    core.apply_para_format_in_cell_native(0, para, ctrl, 0, 0, r#"{"spacingAfter":200}"#)
        .unwrap();
    let snapshot = core.save_snapshot_native();
    core.end_batch_native().unwrap();
    core.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(cell_vpos(&core, para, ctrl), [0, 1700, 3300]);
}

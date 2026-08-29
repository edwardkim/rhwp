//! [#6388] 표 위치 편집이 `raw_ctrl_data` 를 0 확장해 표 기하를 저장에서 파괴하는 문제.
//!
//! 표는 `raw_ctrl_data` 가 정본이라(`serializer/control.rs::serialize_table`) raw 가 비어
//! 있으면 저장기가 `common` 합성 경로로 간다 — HWPX 파스본·신설 표를 위해 #1916 이 세운
//! 계약이다. 종전 `while len < 필요길이 { push(0) }` 은 **빈 raw 를 "있는" raw 로 바꿔** 그
//! 계약을 끊고 12바이트짜리 CTRL_HEADER 를 방출했다. offset 12 이후의 width·height·여백·
//! z-order·instance_id 가 그 순간 사라진다.
//!
//! 발동 집합은 raw 가 빈 표다 — `samples/**/*.hwp` 529개 표 6719개의 길이 분포는
//! `{0: 11, 40: 541, 42: 4716, 44: 1449, 48: 2}` 로 1~39바이트가 없고, HWPX 파서는 표
//! `raw_ctrl_data` 를 채우지 않으므로 HWPX 파스본 표는 전부 해당한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;

/// 표 4개 전부 `raw_ctrl_data` 가 비어 있는 HWP5 표본.
const EMPTY_RAW_SAMPLE: &str = "samples/hwp5-tbl-attr-1916.hwp";
/// HWPX 파스본 — 표 raw 가 항상 비어 있는 실물 경로.
const HWPX_SAMPLES: [&str; 2] = [
    "samples/hwpx/basic-table-01.hwpx",
    "samples/hwpx/business_overview.hwpx",
];

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn first_table_coord(core: &DocumentCore) -> (usize, usize, usize) {
    for (si, s) in core.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Table(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("표가 없다");
}

fn table_at(core: &DocumentCore, c: (usize, usize, usize)) -> &Table {
    match &core.document().sections[c.0].paragraphs[c.1].controls[c.2] {
        Control::Table(t) => t,
        other => panic!("표가 아니다: {other:?}"),
    }
}

/// 빈 raw 표를 옮겨도 raw 가 비어 있어야 한다 — 그래야 저장기가 `common` 합성 경로를 쓴다.
#[test]
fn issue_6388_moving_empty_raw_table_does_not_grow_raw() {
    let mut core = DocumentCore::from_bytes(&read_fixture(EMPTY_RAW_SAMPLE)).expect("파싱");
    let c = first_table_coord(&core);
    assert!(
        table_at(&core, c).raw_ctrl_data.is_empty(),
        "전제: 이 표본의 표는 raw 가 비어 있다"
    );

    core.move_table_offset_native(c.0, c.1, c.2, 1000, 1000)
        .expect("표 이동");

    assert!(
        table_at(&core, c).raw_ctrl_data.is_empty(),
        "빈 raw 를 0 확장하면 저장기가 12바이트 CTRL_HEADER 를 방출한다 (#6388)"
    );
}

/// 이동이 표의 크기·여백을 저장에서 파괴하지 않는다.
///
/// 수정 전에는 45152x58826 → 0x0 이 되고 여백도 전부 0 이 됐다(파일 63488 → 62976 B).
#[test]
fn issue_6388_moving_empty_raw_table_preserves_geometry_through_save() {
    let mut core = DocumentCore::from_bytes(&read_fixture(EMPTY_RAW_SAMPLE)).expect("파싱");
    let c = first_table_coord(&core);
    let (w, h, margin) = {
        let t = table_at(&core, c);
        (t.common.width, t.common.height, t.common.margin.clone())
    };
    assert!(w > 0 && h > 0, "전제: 표에 크기가 있다");

    core.move_table_offset_native(c.0, c.1, c.2, 1000, 1000)
        .expect("표 이동");
    let saved = core.export_hwp_native().expect("저장");

    let reparsed = DocumentCore::from_bytes(&saved).expect("재파싱");
    let rc = first_table_coord(&reparsed);
    let t = table_at(&reparsed, rc);
    assert_eq!(
        (t.common.width, t.common.height),
        (w, h),
        "이동 후 저장·재파싱에서 표 크기가 사라졌다 (#6388)"
    );
    assert_eq!(
        (
            t.common.margin.left,
            t.common.margin.right,
            t.common.margin.top,
            t.common.margin.bottom
        ),
        (margin.left, margin.right, margin.top, margin.bottom),
        "이동 후 저장·재파싱에서 표 여백이 사라졌다 (#6388)"
    );
}

/// 이동 자체는 계속 반영된다 — 파괴만 막고 기능은 유지한다.
#[test]
fn issue_6388_move_still_applies_offsets_without_raw() {
    let mut core = DocumentCore::from_bytes(&read_fixture(EMPTY_RAW_SAMPLE)).expect("파싱");
    let c = first_table_coord(&core);
    let (v0, h0) = {
        let t = table_at(&core, c);
        (
            t.common.vertical_offset as i32,
            t.common.horizontal_offset as i32,
        )
    };

    core.move_table_offset_native(c.0, c.1, c.2, 700, 0)
        .expect("가로 이동");

    let t = table_at(&core, c);
    assert_eq!(
        t.common.horizontal_offset as i32,
        h0 + 700,
        "가로 오프셋이 적용돼야 한다"
    );
    assert_eq!(
        t.common.vertical_offset as i32, v0,
        "세로 오프셋은 그대로여야 한다"
    );
}

/// HWPX 파스본 표도 같다 — 실제 사용 경로의 회귀 가드.
#[test]
fn issue_6388_hwpx_tables_survive_move_and_save() {
    for sample in HWPX_SAMPLES {
        let mut core = DocumentCore::from_bytes(&read_fixture(sample)).expect("파싱");
        let c = first_table_coord(&core);
        let (w, h) = {
            let t = table_at(&core, c);
            assert!(
                t.raw_ctrl_data.is_empty(),
                "{sample}: 전제 — HWPX 파스본 표는 raw 가 비어 있다"
            );
            (t.common.width, t.common.height)
        };
        assert!(w > 0 && h > 0, "{sample}: 전제 — 표에 크기가 있다");

        core.move_table_offset_native(c.0, c.1, c.2, 1000, 1000)
            .expect("표 이동");
        let saved = core.export_hwp_native().expect("저장");

        let reparsed = DocumentCore::from_bytes(&saved).expect("재파싱");
        let rc = first_table_coord(&reparsed);
        let t = table_at(&reparsed, rc);
        assert_eq!(
            (t.common.width, t.common.height),
            (w, h),
            "{sample}: 이동 후 저장·재파싱에서 표 크기가 사라졌다 (#6388)"
        );
    }
}

/// 위치 속성 setter 도 같은 계약이다 — `vertOffset`/`horzOffset` 경로의 0 확장.
#[test]
fn issue_6388_position_props_do_not_grow_empty_raw() {
    let mut core = DocumentCore::from_bytes(&read_fixture(EMPTY_RAW_SAMPLE)).expect("파싱");
    let c = first_table_coord(&core);
    let (w, h) = {
        let t = table_at(&core, c);
        (t.common.width, t.common.height)
    };

    core.set_table_properties_native(c.0, c.1, c.2, r#"{"vertOffset":2000,"horzOffset":1500}"#)
        .expect("위치 속성 설정");

    {
        let t = table_at(&core, c);
        assert!(
            t.raw_ctrl_data.is_empty(),
            "위치 속성 setter 가 빈 raw 를 0 확장하면 안 된다 (#6388)"
        );
        assert_eq!(t.common.vertical_offset, 2000, "세로 오프셋 반영");
        assert_eq!(t.common.horizontal_offset, 1500, "가로 오프셋 반영");
    }

    let saved = core.export_hwp_native().expect("저장");
    let reparsed = DocumentCore::from_bytes(&saved).expect("재파싱");
    let rc = first_table_coord(&reparsed);
    let t = table_at(&reparsed, rc);
    assert_eq!(
        (t.common.width, t.common.height),
        (w, h),
        "위치 속성 변경 후 저장·재파싱에서 표 크기가 사라졌다 (#6388)"
    );
    assert_eq!(t.common.vertical_offset, 2000, "세로 오프셋이 저장에 반영됐다");
    assert_eq!(
        t.common.horizontal_offset, 1500,
        "가로 오프셋이 저장에 반영됐다"
    );
}

/// raw 가 있는 표(한컴 파스본)의 dual-write 는 종전대로 유지된다.
#[test]
fn issue_6388_populated_raw_still_dual_written() {
    const SAMPLE: &str = "samples/ta-pic-001-r.hwp";
    let mut core = DocumentCore::from_bytes(&read_fixture(SAMPLE)).expect("파싱");
    let c = first_table_coord(&core);
    let raw_len = {
        let t = table_at(&core, c);
        assert!(
            t.raw_ctrl_data.len() >= 20,
            "전제: 한컴 파스본 표는 raw 를 가진다"
        );
        t.raw_ctrl_data.len()
    };

    core.move_table_offset_native(c.0, c.1, c.2, 500, 300)
        .expect("표 이동");

    let t = table_at(&core, c);
    assert_eq!(t.raw_ctrl_data.len(), raw_len, "raw 길이는 변하지 않는다");
    let raw_h = i32::from_le_bytes(t.raw_ctrl_data[8..12].try_into().unwrap());
    assert_eq!(
        raw_h, t.common.horizontal_offset as i32,
        "raw 와 common 이 함께 갱신돼야 한다(dual-write 유지)"
    );
}

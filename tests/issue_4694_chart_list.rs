//! #4694 [차트/B1-UI] Stage 1 — 차트 열거 계약 (`list_charts_native`).
//!
//! studio 는 이 JSON 을 `matchChartRef` 로 선택 컨트롤과 대조해 정본 주소(문서 순번)를
//! 얻는다. 필드명·구조가 바뀌면 wasm 경계 건너편이 조용히 깨지므로 계약을 코어
//! 테스트로 고정한다.

use std::path::{Path, PathBuf};

use rhwp::document_core::queries::chart_extract::collect_charts;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table};

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn core_of(path: &Path) -> DocumentCore {
    let bytes = std::fs::read(path).expect("샘플 읽기");
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{}: 코어 {e:?}", path.display()))
}

fn listed(core: &DocumentCore) -> Vec<serde_json::Value> {
    serde_json::from_str(&core.list_charts_native().expect("열거")).expect("JSON 배열")
}

/// 열거 JSON 은 `collect_charts` 의 직렬화 그 자체다 — 별도 가공 층이 없어야
/// CLI(`--chart N`)·코어(by_index)·studio 가 같은 순번을 본다.
#[test]
fn the_listing_is_the_serialized_enumeration() {
    let base = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    for path in [base.clone(), base.with_extension("hwp")] {
        let core = core_of(&path);
        assert_eq!(
            serde_json::Value::Array(listed(&core)),
            serde_json::to_value(collect_charts(core.document())).expect("직렬화"),
            "{}: 열거 JSON 이 collect_charts 직렬화와 다르다",
            path.display()
        );
    }
}

/// studio `matchChartRef` 가 의존하는 wire 필드명 — 본문 직속 항목의 모양.
///
/// HWPX 는 ①② 슬롯이 다 있고, HWP5 는 `Chart/*.xml` 파트가 없어 ②만 있다.
/// 본문 직속이면 `container` 키 자체가 없다(`skip_serializing_if`).
#[test]
fn top_level_entries_follow_the_wire_contract() {
    let base = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    for (path, has_zip_part) in [(base.clone(), true), (base.with_extension("hwp"), false)] {
        let entries = listed(&core_of(&path));
        assert_eq!(entries.len(), 1, "{}: 차트 수", path.display());
        let entry = entries[0].as_object().expect("객체");

        for key in ["index", "section", "paragraph", "control"] {
            assert!(
                entry.get(key).is_some_and(serde_json::Value::is_u64),
                "{}: {key} 숫자 필드",
                path.display()
            );
        }
        assert_eq!(entry["index"], 0, "{}: 문서 순번 0-based", path.display());
        assert_eq!(
            entry.get("zipPart").is_some(),
            has_zip_part,
            "{}: ① 슬롯",
            path.display()
        );
        assert!(
            entry.get("nestedCopy").is_some(),
            "{}: ② 슬롯",
            path.display()
        );
        assert!(
            entry.get("container").is_none(),
            "{}: 본문 직속은 container 키가 없어야 한다",
            path.display()
        );
    }
}

/// 표 셀 안 차트는 3인자 주소로 표현할 수 없고 `container` 경로 + 순번으로만 닿는다 —
/// studio 의 셀 내부 선택(cellPath)이 대조할 그 경로다.
#[test]
fn a_chart_inside_a_table_cell_carries_its_container_path() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let chart = collect_charts(core.document())[0].clone();
    assert!(chart.is_top_level(), "전제: 코퍼스 차트는 본문 직속이다");
    let (sec, para, ctrl) = (chart.section, chart.paragraph, chart.control);

    // 본문 직속 차트를 1x1 표의 셀 문단 안으로 옮긴다.
    let doc = core.document_mut();
    let ole = doc.sections[sec].paragraphs[para].controls.remove(ctrl);
    let cell_para = Paragraph {
        controls: vec![ole],
        ..Default::default()
    };
    let table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            paragraphs: vec![cell_para],
            ..Default::default()
        }],
        ..Default::default()
    };
    doc.sections[sec].paragraphs[para]
        .controls
        .insert(ctrl, Control::Table(Box::new(table)));

    let entries = listed(&core);
    assert_eq!(entries.len(), 1, "차트 수");
    let entry = &entries[0];
    assert_eq!(entry["paragraph"], para, "본문(루트) 문단 인덱스");
    assert_eq!(entry["control"], 0, "셀 문단 안 컨트롤 인덱스");

    let container = entry["container"].as_array().expect("container 경로");
    assert_eq!(container.len(), 1, "중첩 한 단계");
    assert_eq!(container[0]["kind"], "tableCell");
    assert_eq!(
        container[0]["control"], ctrl,
        "본문 문단 안 표 컨트롤 인덱스"
    );
    assert_eq!(container[0]["paragraph"], 0, "셀 안 문단 인덱스");
    assert_eq!(container[0]["cell"], 0, "셀 인덱스");

    // 그 항목의 순번으로 by_index 읽기가 그대로 된다 — studio 의 정본 경로.
    let data: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("순번 읽기"))
            .expect("JSON");
    assert_eq!(data["ok"], true, "셀 안 차트도 순번으로 읽힌다");
}

fn slot_bytes(core: &DocumentCore) -> Vec<Vec<u8>> {
    core.document()
        .bin_data_content
        .iter()
        .map(|c| c.data.load())
        .collect()
}

/// R1(#4694 계획서 §8) — 스냅샷 복원이 차트 편집을 바이트 단위로 되돌린다.
///
/// 차트 편집은 IR 이 아니라 `bin_data_content` 슬롯 바이트 변이다. 스냅샷이
/// `Document` 통째 clone 이라 슬롯도 함께 복원된다 — studio 의 Ctrl+Z(snapshot
/// 라우팅)가 이 사실 위에 선다. 이 테스트가 깨지면 B1-UI 의 undo 전제가 무너진 것이다.
#[test]
fn snapshot_restore_rolls_back_a_chart_edit_byte_for_byte() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let original = slot_bytes(&core);

    let snapshot = core.save_snapshot_native();

    // 읽은 데이터에서 첫 계열 첫 값만 sentinel 로 바꾼 edits 를 만든다(#4055 의 91.7).
    let data: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON");
    let mut series: Vec<serde_json::Value> = data["series"]
        .as_array()
        .expect("계열")
        .iter()
        .map(|s| serde_json::json!({ "values": s["values"] }))
        .collect();
    series[0]["values"][0] = serde_json::json!("91.7");
    let set: serde_json::Value = serde_json::from_str(
        &core
            .set_chart_data_by_index_native(0, &serde_json::json!({ "series": series }).to_string())
            .expect("쓰기"),
    )
    .expect("JSON");
    assert_eq!(set["ok"], true, "편집 수용: {set}");
    assert!(
        set["changedCount"].as_u64().is_some_and(|n| n >= 1),
        "실변경이어야 한다: {set}"
    );
    assert_ne!(original, slot_bytes(&core), "편집이 슬롯 바이트를 바꿨다");

    core.restore_snapshot_native(snapshot).expect("복원");
    assert_eq!(
        original,
        slot_bytes(&core),
        "복원 후 슬롯 바이트가 원형이다"
    );
}
